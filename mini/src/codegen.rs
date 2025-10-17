//! LLVM IR generation for Mini programs using Inkwell.

use anyhow::{anyhow, Result};
use inkwell::{
    builder::Builder,
    context::Context as LlvmContext,
    module::Linkage,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, TargetMachine, TargetTriple},
    values::{FunctionValue, IntValue, PointerValue},
    AddressSpace, OptimizationLevel,
};
use std::collections::HashMap;

use crate::ast::{Expr, Program, Stmt};

/// Representation of a Mini variable during codegen.
#[derive(Clone, Copy)]
enum Var<'ctx> {
    Int { alloca: inkwell::values::PointerValue<'ctx> }, // i32*
    Str { alloca: inkwell::values::PointerValue<'ctx> }, // i8*
}

/// Generates LLVM IR, keeps track of intrinsics, and records local bindings.
pub struct Codegen<'ctx> {
    ctx: &'ctx LlvmContext,
    builder: Builder<'ctx>,
    module: inkwell::module::Module<'ctx>,
    printf: FunctionValue<'ctx>,
    fmt_int: PointerValue<'ctx>,
    fmt_str: PointerValue<'ctx>,
    vars: HashMap<String, Var<'ctx>>,
}

impl<'ctx> Codegen<'ctx> {
    /// Create a new code generator configured for the supplied target triple.
    pub fn new(ctx: &'ctx LlvmContext, triple: &TargetTriple) -> Self {
        let module = ctx.create_module("mini");
        module.set_triple(triple);
        let builder = ctx.create_builder();

        // declare i32 @printf(i8*, ...)
        let i32_t = ctx.i32_type();
        let i8ptr_t = ctx.i8_type().ptr_type(AddressSpace::default());
        let printf_ty = i32_t.fn_type(&[i8ptr_t.into()], true);
        let printf = module.add_function("printf", printf_ty, Some(Linkage::External));

        // Tiny init fn for global strings; terminate it to keep module valid
        let void_t = ctx.void_type();
        let init_fn = module.add_function("__mini_init", void_t.fn_type(&[], false), None);
        let init_bb = ctx.append_basic_block(init_fn, "entry");
        builder.position_at_end(init_bb);
        let fmt_int = builder.build_global_string_ptr("%d\n", ".fmt_int").unwrap().as_pointer_value();
        let fmt_str = builder.build_global_string_ptr("%s\n", ".fmt_str").unwrap().as_pointer_value();
        builder.build_return(None).unwrap();

        Self { ctx, builder, module, printf, fmt_int, fmt_str, vars: HashMap::new() }
    }

    /// Walk the AST, build the `main` function, and populate the module.
    pub fn emit_program(&mut self, program: &Program) -> Result<()> {
        let i32_t = self.ctx.i32_type();
        let i8_t = self.ctx.i8_type();
        let i8ptr_t = i8_t.ptr_type(AddressSpace::default());

        let main_fn = self.module.add_function("main", i32_t.fn_type(&[], false), None);
        let entry = self.ctx.append_basic_block(main_fn, "entry");
        self.builder.position_at_end(entry);

        for stmt in &program.stmts {
            match stmt {
                Stmt::Let { name, expr } => {
                    match infer_expr_kind(expr) {
                        ExprKind::Int => {
                            let v = self.gen_expr_int(expr)?;
                            let alloca = self.builder.build_alloca(i32_t, name).unwrap();
                            self.builder.build_store(alloca, v).unwrap();
                            self.vars.insert(name.clone(), Var::Int { alloca });
                        }
                        ExprKind::Str => {
                            let s = expect_string_literal(expr)?;
                            let gptr = self.builder
                                .build_global_string_ptr(&s, &format!(".str{}", self.vars.len()))
                                .unwrap()
                                .as_pointer_value();
                            let alloca = self.builder.build_alloca(i8ptr_t, name).unwrap();
                            self.builder.build_store(alloca, gptr).unwrap();
                            self.vars.insert(name.clone(), Var::Str { alloca });
                        }
                    }
                }
                Stmt::Print { name } => {
                    match *self.vars.get(name).ok_or_else(|| anyhow!(format!("undefined variable `{}`", name)))? {
                        Var::Int { alloca } => {
                            let v = self.builder.build_load(i32_t, alloca, "ival").unwrap();
                            self.builder.build_call(self.printf, &[self.fmt_int.into(), v.into()], "").unwrap();
                        }
                        Var::Str { alloca } => {
                            let v = self.builder.build_load(i8ptr_t, alloca, "sval").unwrap();
                            self.builder.build_call(self.printf, &[self.fmt_str.into(), v.into()], "").unwrap();
                        }
                    }
                }
            }
        }

        self.builder.build_return(Some(&i32_t.const_zero())).unwrap();
        Ok(())
    }

    /// Generate an integer value for the given expression, enforcing type checks.
    ///
    /// Expressions are evaluated eagerly; every branch either returns a concrete
    /// `i32` value or fails with a semantic error (e.g. using a string in math).
    fn gen_expr_int(&mut self, expr: &Expr) -> Result<IntValue<'ctx>> {
        let i32_t = self.ctx.i32_type();

        Ok(match expr {
            // literal integers map directly to LLVM constants
            Expr::Int(v) => i32_t.const_int(*v as i64 as u64, true),
            Expr::Var(name) => {
                match *self.vars.get(name).ok_or_else(|| anyhow!(format!("undefined variable `{}`", name)))? {
                    // load previously stored integer variable
                    Var::Int { alloca } => self.builder.build_load(i32_t, alloca, "loadi").unwrap().into_int_value(),
                    // prohibit mixing string bindings inside arithmetic expressions
                    Var::Str { .. } => anyhow::bail!("type error: `{}` is a string, expected integer", name),
                }
            }
            Expr::UnaryNeg(e) => {
                // recursively evaluate RHS and negate
                let v = self.gen_expr_int(e)?;
                self.builder.build_int_neg(v, "neg").unwrap()
            }
            Expr::Add(a, b) => {
                // evaluate operands left-to-right and build the arithmetic instruction
                let l = self.gen_expr_int(a)?;
                let r = self.gen_expr_int(b)?;
                self.builder.build_int_add(l, r, "add").unwrap()
            }
            Expr::Sub(a, b) => {
                let l = self.gen_expr_int(a)?;
                let r = self.gen_expr_int(b)?;
                self.builder.build_int_sub(l, r, "sub").unwrap()
            }
            Expr::Mul(a, b) => {
                let l = self.gen_expr_int(a)?;
                let r = self.gen_expr_int(b)?;
                self.builder.build_int_mul(l, r, "mul").unwrap()
            }
            Expr::Div(a, b) => {
                let l = self.gen_expr_int(a)?;
                let r = self.gen_expr_int(b)?;
                self.builder.build_int_signed_div(l, r, "div").unwrap()
            }
            Expr::Str(_) => anyhow::bail!("type error: string literal not allowed in integer expression"),
        })
    }

    /// Verify the module and write out an object file using the host target machine.
    pub fn write_object(&self, triple: &TargetTriple, out_obj: &std::path::Path) -> Result<()> {
        self.module.verify().map_err(|e| anyhow!(e.to_string()))?;
        inkwell::targets::Target::initialize_all(&InitializationConfig::default());
        let target = inkwell::targets::Target::from_triple(triple).map_err(|e| anyhow!(e.to_string()))?;
        let tm = target
            .create_target_machine(
                triple,
                "generic",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or_else(|| anyhow!("create target machine failed"))?;
        tm.write_to_file(&self.module, FileType::Object, out_obj)
            .map_err(|e| anyhow!(e.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExprKind { Int, Str }

fn infer_expr_kind(e: &Expr) -> ExprKind {
    match e {
        Expr::Str(_) => ExprKind::Str,
        _ => ExprKind::Int, // everything else is int-typed in this phase
    }
}

fn expect_string_literal(e: &Expr) -> Result<String> {
    match e {
        Expr::Str(s) => Ok(s.clone()),
        _ => anyhow::bail!("expected string literal"),
    }
}

/// Grab the default target triple for the build machine.
pub fn host_triple() -> TargetTriple {
    TargetMachine::get_default_triple()
}
