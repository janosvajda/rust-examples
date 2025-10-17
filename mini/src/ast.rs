#[derive(Debug, Clone)]
pub enum Expr {
    // integers
    Int(i32),
    Var(String),
    UnaryNeg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),

    // strings (literal only for now)
    Str(String),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, expr: Expr }, // type will be inferred at codegen
    Print { name: String },           // still identifier-only for simplicity
}

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}
