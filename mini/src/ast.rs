/// Abstract syntax tree nodes for the Mini language.
#[derive(Debug, Clone)]
pub enum Expr {
    // integer literals (32-bit for now)
    Int(i32),
    // variable reference
    Var(String),
    // unary minus, e.g. `-a`
    UnaryNeg(Box<Expr>),
    // arithmetic binary operators
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    // strings (literal only for now)
    Str(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let` declaration with an expression initializer; codegen infers the concrete type.
    Let { name: String, expr: Expr },
    /// `print` an identifier (string literals are future work).
    Print { name: String },
}

/// Top-level container for a parsed Mini program.
#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
