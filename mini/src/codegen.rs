//! LLVM IR generation for Mini programs.
//!
//! The output is LLVM IR as plain text, the same thing you'd find in a `.ll` file.
//! For `let x = 2 + 3; print x;` it looks like this:
//!
//! ```text
//! define i32 @main() {
//! entry:
//!   %t0 = add i32 2, 3
//!   %x.0 = alloca i32
//!   store i32 %t0, ptr %x.0
//!   %t1 = load i32, ptr %x.0
//!   call i32 (ptr, ...) @printf(ptr @.fmt_int, i32 %t1)
//!   ret i32 0
//! }
//! ```
//!
//! The installed LLVM then turns this text into machine code (see `llvm.rs`).

use anyhow::{anyhow, bail, Result};
use std::collections::HashMap;

use crate::ast::{BinOp, Expr, Program, Stmt, UnaryOp};

/// Decimal `*`, `/` and printing, written in LLVM IR. Added only to programs that use decimals.
const INTEGER_RUNTIME: &str = include_str!("integer-runtime.ll");
const RUNTIME: &str = include_str!("runtime.ll");

/// The type of a Mini value.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Int,  // whole number
    Dec,  // decimal, counted in millionths
    Bool, // true or false
    Str,  // text
}

impl Kind {
    /// The LLVM type that holds a value of this kind.
    fn llvm_type(self) -> &'static str {
        match self {
            Kind::Int => "i32",
            Kind::Dec => "i64",
            Kind::Bool => "i1", // a single bit: 1 is true, 0 is false
            Kind::Str => "ptr", // the address of the text
        }
    }

    /// How the kind is named in error messages.
    fn name(self) -> &'static str {
        match self {
            Kind::Int => "number",
            Kind::Dec => "decimal",
            Kind::Bool => "boolean",
            Kind::Str => "string",
        }
    }
}

/// A Mini variable: its kind, and the name of the stack slot (`alloca`) that holds it.
struct Var {
    kind: Kind,
    slot: String,
}

/// Turn a parsed Mini program into LLVM IR text.
pub fn generate(program: &Program) -> Result<String> {
    let mut cg = Codegen::default();
    for stmt in &program.stmts {
        cg.statement(stmt)?;
    }
    Ok(cg.finish())
}

#[derive(Default)]
struct Codegen {
    globals: String, // string constants, written above `main`
    body: String,    // the instructions inside `main`
    next_tmp: usize, // numbers the temporaries: %t0, %t1, …
    next_id: usize,  // numbers string constants and variable slots
    uses_integer_division: bool,
    uses_decimals: bool, // whether to add the decimal runtime
    vars: HashMap<String, Var>,
}

impl Codegen {
    fn statement(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, expr } => {
                // Work out the value first: `let x = x + 1` reads the old `x`.
                let (kind, value) = self.expr(expr)?;
                let ty = kind.llvm_type();
                let slot = format!("%{name}.{}", self.fresh_id());
                self.emit(format!("{slot} = alloca {ty}"));
                self.emit(format!("store {ty} {value}, ptr {slot}"));
                self.vars.insert(name.clone(), Var { kind, slot });
            }
            Stmt::Print { name } => {
                let (kind, value) = self.load(name)?;
                match kind {
                    Kind::Int => self.emit(format!(
                        "call i32 (ptr, ...) @printf(ptr @.fmt_int, i32 {value})"
                    )),
                    Kind::Str => self.emit(format!(
                        "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {value})"
                    )),
                    Kind::Dec => self.emit(format!("call void @mini_print_dec(i64 {value})")),
                    Kind::Bool => {
                        // choose the text "true" or "false", then print it
                        let text = self.tmp();
                        self.emit(format!(
                            "{text} = select i1 {value}, ptr @.true, ptr @.false"
                        ));
                        self.emit(format!(
                            "call i32 (ptr, ...) @printf(ptr @.fmt_str, ptr {text})"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate an expression. Returns its kind and the operand that holds the result:
    /// a constant like `42` or `true`, a global like `@.str.0`, or a temporary like `%t3`.
    fn expr(&mut self, expr: &Expr) -> Result<(Kind, String)> {
        match expr {
            Expr::Int(v) => Ok((Kind::Int, v.to_string())),
            Expr::Dec(v) => {
                self.uses_decimals = true;
                Ok((Kind::Dec, v.to_string()))
            }
            Expr::Bool(b) => Ok((Kind::Bool, b.to_string())), // LLVM writes i1 constants as true/false
            Expr::Str(text) => Ok((Kind::Str, self.string_constant(text))),
            Expr::Var(name) => self.load(name),
            Expr::Unary(op, e) => {
                let (kind, value) = self.expr(e)?;
                let result = self.tmp();
                match (op, kind) {
                    // LLVM has no "negate" instruction: -x is 0 - x
                    (UnaryOp::Neg, Kind::Int | Kind::Dec) => {
                        self.emit(format!("{result} = sub {} 0, {value}", kind.llvm_type()))
                    }
                    // flipping a bit: true xor true is false, false xor true is true
                    (UnaryOp::Not, Kind::Bool) => {
                        self.emit(format!("{result} = xor i1 {value}, true"))
                    }
                    (UnaryOp::Neg, _) => bail!("type error: cannot negate a {}", kind.name()),
                    (UnaryOp::Not, _) => {
                        bail!("type error: `not` needs a boolean, found a {}", kind.name())
                    }
                }
                Ok((kind, result))
            }
            Expr::Binary(op, a, b) => {
                // Evaluate both sides first (left to right), then combine them.
                let (left_kind, left) = self.expr(a)?;
                let (right_kind, right) = self.expr(b)?;
                if left_kind != right_kind {
                    bail!(
                        "type error: cannot use `{}` on a {} and a {}",
                        op.symbol(),
                        left_kind.name(),
                        right_kind.name()
                    );
                }
                self.binary(*op, left_kind, &left, &right)
            }
        }
    }

    /// Combine two values of the same kind with an operator.
    fn binary(&mut self, op: BinOp, kind: Kind, left: &str, right: &str) -> Result<(Kind, String)> {
        use BinOp::*;
        let ty = kind.llvm_type();
        let result = self.tmp();
        let result_kind = match (op, kind) {
            // whole numbers: one machine instruction each
            (Div, Kind::Int) => {
                self.uses_integer_division = true;
                self.emit(format!(
                    "{result} = call i32 @mini_int_div(i32 {left}, i32 {right})"
                ));
                Kind::Int
            }
            (Add | Sub | Mul, Kind::Int) => {
                let instr = match op {
                    Add => "add",
                    Sub => "sub",
                    Mul => "mul",
                    _ => "sdiv",
                }; // sdiv: signed division
                self.emit(format!("{result} = {instr} i32 {left}, {right}"));
                Kind::Int
            }
            // decimals: + and - work directly on the millionths; * and / need rescaling (runtime.ll)
            (Add | Sub, Kind::Dec) => {
                let instr = if op == Add { "add" } else { "sub" };
                self.emit(format!("{result} = {instr} i64 {left}, {right}"));
                Kind::Dec
            }
            (Mul | Div, Kind::Dec) => {
                let helper = if op == Mul {
                    "mini_dec_mul"
                } else {
                    "mini_dec_div"
                };
                self.emit(format!(
                    "{result} = call i64 @{helper}(i64 {left}, i64 {right})"
                ));
                Kind::Dec
            }
            // comparisons give a boolean; `icmp` compares integers, `s` means signed
            (Eq | Ne, Kind::Int | Kind::Dec | Kind::Bool)
            | (Lt | Le | Gt | Ge, Kind::Int | Kind::Dec) => {
                let cond = match op {
                    Eq => "eq",
                    Ne => "ne",
                    Lt => "slt",
                    Le => "sle",
                    Gt => "sgt",
                    _ => "sge",
                };
                self.emit(format!("{result} = icmp {cond} {ty} {left}, {right}"));
                Kind::Bool
            }
            // logic on single bits
            (And | Or, Kind::Bool) => {
                let instr = if op == And { "and" } else { "or" };
                self.emit(format!("{result} = {instr} i1 {left}, {right}"));
                Kind::Bool
            }
            _ => bail!(
                "type error: cannot use `{}` on {}s",
                op.symbol(),
                kind.name()
            ),
        };
        Ok((result_kind, result))
    }

    /// Load a variable's current value from its stack slot.
    fn load(&mut self, name: &str) -> Result<(Kind, String)> {
        let var = self
            .vars
            .get(name)
            .ok_or_else(|| anyhow!("undefined variable `{name}`"))?;
        let (kind, slot) = (var.kind, var.slot.clone());
        let value = self.tmp();
        self.emit(format!("{value} = load {}, ptr {slot}", kind.llvm_type()));
        Ok((kind, value))
    }

    /// Add a constant, zero-terminated string and return its global name, like `@.str.0`.
    fn string_constant(&mut self, text: &str) -> String {
        let name = format!("@.str.{}", self.fresh_id());
        self.globals.push_str(&global_string(&name, text));
        name
    }

    fn tmp(&mut self) -> String {
        self.next_tmp += 1;
        format!("%t{}", self.next_tmp - 1)
    }

    fn fresh_id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id - 1
    }

    fn emit(&mut self, instruction: String) {
        self.body.push_str("  ");
        self.body.push_str(&instruction);
        self.body.push('\n');
    }

    /// Put the pieces together into one complete LLVM module.
    fn finish(self) -> String {
        format!(
            "; Mini program, compiled to LLVM IR\n\n\
             {}{}{}{}{}\n\
             declare i32 @printf(ptr, ...)\n\n\
             declare void @llvm.trap()\n\n\
             define i32 @main() {{\n\
             entry:\n\
             {}  ret i32 0\n\
             }}\n{}",
            global_string("@.fmt_int", "%d\n"),
            global_string("@.fmt_str", "%s\n"),
            global_string("@.true", "true"),
            global_string("@.false", "false"),
            self.globals,
            self.body,
            format_args!(
                "{}{}",
                if self.uses_integer_division {
                    INTEGER_RUNTIME
                } else {
                    ""
                },
                if self.uses_decimals { RUNTIME } else { "" }
            ),
        )
    }
}

/// A global string constant: `@name = private unnamed_addr constant [N x i8] c"…\00"`.
/// Quotes, backslashes and non-printable bytes are written as `\XX` hex escapes.
fn global_string(name: &str, text: &str) -> String {
    let mut escaped = String::new();
    for byte in text.bytes() {
        if byte == b' ' || (byte.is_ascii_graphic() && byte != b'"' && byte != b'\\') {
            escaped.push(byte as char);
        } else {
            escaped.push_str(&format!("\\{byte:02X}"));
        }
    }
    let len = text.len() + 1; // +1 for the terminating zero byte that C's printf needs
    format!("{name} = private unnamed_addr constant [{len} x i8] c\"{escaped}\\00\"\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn ir(src: &str) -> String {
        generate(&Parser::parse(src).unwrap()).unwrap()
    }

    fn type_error(src: &str) -> String {
        generate(&Parser::parse(src).unwrap())
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn arithmetic_becomes_instructions() {
        let ir = ir("let x = 2 + 3 * 4;\nprint x;");
        assert!(ir.contains("%t0 = mul i32 3, 4"));
        assert!(ir.contains("%t1 = add i32 2, %t0"));
        assert!(ir.contains("@printf(ptr @.fmt_int"));
        assert!(
            !ir.contains("mini_dec_mul"),
            "no decimals, so no decimal runtime"
        );
    }

    #[test]
    fn decimals_and_booleans() {
        let ir =
            ir("let price = 2.5 * 3.0;\nlet cheap = price < 10.0 and not false;\nprint cheap;");
        assert!(ir.contains("call i64 @mini_dec_mul(i64 2500000, i64 3000000)"));
        assert!(ir.contains("icmp slt i64"));
        assert!(ir.contains("xor i1 false, true"));
        assert!(
            ir.contains("define private i64 @mini_dec_mul"),
            "the decimal runtime is included"
        );
    }

    #[test]
    fn strings_are_escaped_and_zero_terminated() {
        assert_eq!(
            global_string("@s", "hi \"x\"\n"),
            "@s = private unnamed_addr constant [8 x i8] c\"hi \\22x\\22\\0A\\00\"\n"
        );
    }

    #[test]
    fn errors_are_reported() {
        assert!(type_error("print y;").contains("undefined variable `y`"));
        assert!(type_error("let s = \"a\";\nlet n = s * 2;")
            .contains("cannot use `*` on a string and a number"));
        assert!(type_error("let n = 1 + 2.5;").contains("cannot use `+` on a number and a decimal"));
        assert!(type_error("let b = true + true;").contains("cannot use `+` on booleans"));
        assert!(type_error("let b = 1 and 2;").contains("cannot use `and` on numbers"));
        assert!(type_error("let b = true < false;").contains("cannot use `<` on booleans"));
        assert!(type_error("let b = not 1;").contains("`not` needs a boolean, found a number"));
    }
}
