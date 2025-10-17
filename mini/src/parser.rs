//! Hand-rolled parser for Mini source: statement scanning plus a Pratt expression parser.

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::ast::{Expr, Program, Stmt};

/// Entry point for turning source code into an AST.
pub struct Parser;

impl Parser {
    /// Parse a complete Mini program from raw source text.
    ///
    /// This handles line-oriented statements (`let`, `print`) and delegates to the
    /// Pratt parser for arithmetic expressions.
    pub fn parse(src: &str) -> Result<Program> {
        let let_re = Regex::new(r#"^let\s+([A-Za-z_]\w*)\s*=\s*(.+);\s*$"#).unwrap();
        let print_re = Regex::new(r#"^print\s+([A-Za-z_]\w*)\s*;\s*$"#).unwrap();

        let mut stmts = Vec::new();

        for (lineno, raw) in src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if let Some(caps) = let_re.captures(line) {
                let name = caps[1].to_string();
                let rhs = caps[2].trim();

                // string literal?
                if rhs.starts_with('"') && rhs.ends_with('"') {
                    let s = parse_string(rhs)
                        .with_context(|| format!("line {} string literal", lineno + 1))?;
                    stmts.push(Stmt::Let { name, expr: Expr::Str(s) });
                    continue;
                }

                // otherwise: integer expression
                let expr = parse_int_expr(rhs)
                    .with_context(|| format!("line {}: bad expression `{}`", lineno + 1, rhs))?;
                stmts.push(Stmt::Let { name, expr });
                continue;
            }

            if let Some(caps) = print_re.captures(line) {
                stmts.push(Stmt::Print { name: caps[1].to_string() });
                continue;
            }

            bail!("line {}: unrecognized syntax", lineno + 1);
        }

        Ok(Program { stmts })
    }
}

// =============== expression parser (integers) ==================
//
// Grammar (Pratt parser / precedence climbing):
//   expr  := parse_bp(0)
// operators:
//   prefix:      '-'        (unary minus)     binding power: 9
//   infix left:  '*','/'    (mul/div)         binding power: 7
//   infix left:  '+','-'    (add/sub)         binding power: 5
// atoms: INT, IDENT, '(' expr ')'

/// Parse an arithmetic expression into an AST node, rejecting trailing tokens.
fn parse_int_expr(s: &str) -> Result<Expr> {
    let mut it = Lexer::new(s).peekable();
    let expr = parse_bp(&mut it, 0)?;
    // ensure no trailing tokens
    if let Some(tok) = it.peek() {
        bail!("unexpected token after expression: {:?}", tok);
    }
    Ok(expr)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Int(i32),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
}

struct Lexer<'a> {
    s: &'a str,
    i: usize,
}
impl<'a> Lexer<'a> {
    /// Construct a lexer over a slice of source.
    fn new(s: &'a str) -> Self { Self { s, i: 0 } }
}
impl<'a> Iterator for Lexer<'a> {
    type Item = Tok;
    fn next(&mut self) -> Option<Self::Item> {
        let b = self.s.as_bytes();
        let n = b.len();
        while self.i < n && b[self.i].is_ascii_whitespace() { self.i += 1; }
        if self.i >= n { return None; }
        let c = b[self.i] as char;

        // number (allow leading digits; unary handled in parser)
        if c.is_ascii_digit() {
            let start = self.i;
            self.i += 1;
            while self.i < n && (b[self.i] as char).is_ascii_digit() { self.i += 1; }
            let s = &self.s[start..self.i];
            let v = s.parse::<i32>().ok()?;
            return Some(Tok::Int(v));
        }

        // ident
        if c.is_ascii_alphabetic() || c == '_' {
            let start = self.i;
            self.i += 1;
            while self.i < n {
                let ch = b[self.i] as char;
                if ch.is_ascii_alphanumeric() || ch == '_' { self.i += 1; } else { break; }
            }
            let name = self.s[start..self.i].to_string();
            return Some(Tok::Ident(name));
        }

        // single-char tokens
        self.i += 1;
        match c {
            '+' => Some(Tok::Plus),
            '-' => Some(Tok::Minus),
            '*' => Some(Tok::Star),
            '/' => Some(Tok::Slash),
            '(' => Some(Tok::LParen),
            ')' => Some(Tok::RParen),
            _ => None,
        }
    }
}

/// Pratt-style precedence parser (a top-down operator-precedence algorithm).
///
/// Each operator is assigned a binding power; recursive calls enforce precedence
/// by raising `min_bp` when stepping into tighter-binding operators. This keeps
/// the implementation compact compared with writing an explicit grammar, which
/// suits this example project.
fn parse_bp<I>(it: &mut std::iter::Peekable<I>, min_bp: u8) -> Result<Expr>
where
    I: Iterator<Item = Tok>,
{
    // prefix / atom
    let mut lhs = match it.next().ok_or_else(|| anyhow::anyhow!("expected expression"))? {
        Tok::Int(v) => Expr::Int(v),
        Tok::Ident(name) => Expr::Var(name),
        Tok::Minus => {
            // unary minus has high binding power
            let rhs = parse_bp(it, 9)?;
            Expr::UnaryNeg(Box::new(rhs))
        }
        Tok::LParen => {
            let e = parse_bp(it, 0)?;
            match it.next() {
                Some(Tok::RParen) => e,
                _ => anyhow::bail!("expected `)`"),
            }
        }
        t => anyhow::bail!("unexpected token: {:?}", t),
    };

    // infix loop
    loop {
        let op = match it.peek() {
            Some(Tok::Plus) => (5, 6, '+'),
            Some(Tok::Minus) => (5, 6, '-'),
            Some(Tok::Star) => (7, 8, '*'),
            Some(Tok::Slash) => (7, 8, '/'),
            _ => break,
        };
        if op.0 < min_bp {
            break;
        }
        let _ = it.next(); // consume operator
        let rhs = parse_bp(it, op.1)?; // right binding power
        lhs = match op.2 {
            '+' => Expr::Add(Box::new(lhs), Box::new(rhs)),
            '-' => Expr::Sub(Box::new(lhs), Box::new(rhs)),
            '*' => Expr::Mul(Box::new(lhs), Box::new(rhs)),
            '/' => Expr::Div(Box::new(lhs), Box::new(rhs)),
            _ => unreachable!(),
        };
    }

    Ok(lhs)
}

// Minimal escapes for our language's string literals: \n \t \" \\
/// Parse and unescape the limited string literal syntax Mini supports.
fn parse_string(mut s: &str) -> Result<String> {
    if !(s.starts_with('"') && s.ends_with('"')) { bail!("not a string literal"); }
    s = &s[1..s.len() - 1];

    let mut out = String::new();
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => bail!("unsupported escape \\{}", other),
                None => bail!("dangling backslash"),
            }
        } else {
            out.push(c);
        }
    }
    Ok(out)
}
