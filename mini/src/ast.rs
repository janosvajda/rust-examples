/// Abstract syntax tree nodes for the Mini language.
#[derive(Debug, Clone)]
pub enum Expr {
    /// whole number (32-bit), e.g. `42`
    Int(i32),
    /// decimal, stored as millionths: `2.5` is `2_500_000`
    Dec(i64),
    /// `true` or `false`
    Bool(bool),
    /// text (only as the whole right-hand side of a `let`)
    Str(String),
    /// variable reference
    Var(String),
    /// `-x` or `not x`
    Unary(UnaryOp, Box<Expr>),
    /// `a + b`, `a < b`, `a and b`, …
    Binary(BinOp, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, // -
    Not, // not
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,     // + - * /
    Eq, Ne, Lt, Le, Gt, Ge, // == != < <= > >=
    And, Or,                // and or
}

impl BinOp {
    /// How the operator is written in Mini source, for error messages.
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let` declaration with an expression initializer; codegen works out its type.
    Let { name: String, expr: Expr },
    /// `print` an identifier (string literals are future work).
    Print { name: String },
}

/// Top-level container for a parsed Mini program.
#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
