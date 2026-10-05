//! Hand-rolled parser for Mini source: statement scanning plus a Pratt expression parser.

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;

use crate::ast::{BinOp, Expr, Program, Stmt, UnaryOp};

/// Words with a special meaning, which can't be used as variable names.
const KEYWORDS: [&str; 7] = ["let", "print", "true", "false", "and", "or", "not"];

/// Decimals keep exactly this many digits after the point.
pub const DECIMAL_PLACES: usize = 6;
/// A decimal is stored as a whole number of millionths: `2.5` is `2_500_000`.
pub const DECIMAL_SCALE: i64 = 1_000_000;

/// Entry point for turning source code into an AST.
pub struct Parser;

impl Parser {
    /// Parse a complete Mini program from raw source text.
    ///
    /// This handles line-oriented statements (`let`, `print`) and delegates to the
    /// Pratt parser for expressions.
    pub fn parse(src: &str) -> Result<Program> {
        let let_re = Regex::new(r#"^let\s+([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+);\s*$"#).unwrap();
        let print_re = Regex::new(r#"^print\s+([A-Za-z_][A-Za-z0-9_]*)\s*;\s*$"#).unwrap();

        let mut stmts = Vec::new();

        for (lineno, raw) in src.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if let Some(caps) = let_re.captures(line) {
                let name = caps[1].to_string();
                let rhs = caps[2].trim();
                if KEYWORDS.contains(&name.as_str()) {
                    bail!(
                        "line {}: `{}` is a keyword and can't be a variable name",
                        lineno + 1,
                        name
                    );
                }

                // string literal?
                if rhs.starts_with('"') && rhs.ends_with('"') {
                    let s = parse_string(rhs)
                        .with_context(|| format!("line {} string literal", lineno + 1))?;
                    stmts.push(Stmt::Let {
                        name,
                        expr: Expr::Str(s),
                    });
                    continue;
                }

                // otherwise: an expression (number, decimal or boolean)
                let expr = parse_expr(rhs)
                    .with_context(|| format!("line {}: bad expression `{}`", lineno + 1, rhs))?;
                stmts.push(Stmt::Let { name, expr });
                continue;
            }

            if let Some(caps) = print_re.captures(line) {
                stmts.push(Stmt::Print {
                    name: caps[1].to_string(),
                });
                continue;
            }

            bail!("line {}: unrecognized syntax", lineno + 1);
        }

        Ok(Program { stmts })
    }
}

// =============== expression parser ==================
//
// Grammar (Pratt parser / precedence climbing). A higher binding power binds tighter:
//   infix left:  'or'                        binding power: 1
//   infix left:  'and'                       binding power: 3
//   prefix:      'not'                       binding power: 5
//   infix left:  '==' '!=' '<' '<=' '>' '>=' binding power: 7
//   infix left:  '+' '-'                     binding power: 9
//   infix left:  '*' '/'                     binding power: 11
//   prefix:      '-'                         binding power: 13
// atoms: INT, DECIMAL, 'true', 'false', IDENT, '(' expr ')'

/// Parse an expression into an AST node, rejecting trailing tokens.
fn parse_expr(s: &str) -> Result<Expr> {
    let mut it = tokenize(s)?.into_iter().peekable();
    let expr = parse_bp(&mut it, 0)?;
    // ensure no trailing tokens
    if let Some(tok) = it.peek() {
        bail!("unexpected token after expression: {:?}", tok);
    }
    Ok(expr)
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    MinMagnitude, // 2147483648 is valid only immediately after unary minus
    Int(i32),
    Dec(i64),
    Ident(String),
    True,
    False,
    And,
    Or,
    Not,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    LParen,
    RParen,
}

/// Split an expression into tokens. Any character Mini doesn't know is an error,
/// so `2 % 3` is rejected instead of being silently read as `2`.
fn tokenize(s: &str) -> Result<Vec<Tok>> {
    let chars: Vec<char> = s.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c.is_ascii_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() {
            // a number, or a decimal if a '.' and more digits follow (unary minus is handled by the parser)
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < chars.len() && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                tokens.push(Tok::Dec(parse_decimal(&text)?));
            } else {
                let text: String = chars[start..i].iter().collect();
                let v = text
                    .parse::<u32>()
                    .map_err(|_| anyhow!("number `{text}` is too large"))?;
                if v == i32::MAX as u32 + 1 {
                    tokens.push(Tok::MinMagnitude);
                } else {
                    tokens.push(Tok::Int(
                        i32::try_from(v).map_err(|_| anyhow!("number `{text}` is too large"))?,
                    ));
                }
            }
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            tokens.push(match word.as_str() {
                "true" => Tok::True,
                "false" => Tok::False,
                "and" => Tok::And,
                "or" => Tok::Or,
                "not" => Tok::Not,
                _ => Tok::Ident(word),
            });
        } else if let Some(tok) = two_char_token(c, next) {
            tokens.push(tok);
            i += 2;
        } else {
            // single-char tokens
            tokens.push(match c {
                '+' => Tok::Plus,
                '-' => Tok::Minus,
                '*' => Tok::Star,
                '/' => Tok::Slash,
                '<' => Tok::Less,
                '>' => Tok::Greater,
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                '=' => bail!("unexpected `=`: to compare two values, write `==`"),
                _ => bail!("unexpected character `{c}`"),
            });
            i += 1;
        }
    }
    const MAX_EXPRESSION_TOKENS: usize = 256;
    if tokens.len() > MAX_EXPRESSION_TOKENS {
        bail!("expression exceeds {MAX_EXPRESSION_TOKENS} tokens; split it into several variables");
    }
    Ok(tokens)
}

/// The operators written with two characters.
fn two_char_token(c: char, next: Option<char>) -> Option<Tok> {
    match (c, next?) {
        ('=', '=') => Some(Tok::EqEq),
        ('!', '=') => Some(Tok::NotEq),
        ('<', '=') => Some(Tok::LessEq),
        ('>', '=') => Some(Tok::GreaterEq),
        _ => None,
    }
}

/// Turn decimal text like `2.5` into millionths (`2_500_000`), exactly, with no floating point.
fn parse_decimal(text: &str) -> Result<i64> {
    let (whole, frac) = text.split_once('.').unwrap();
    if frac.len() > DECIMAL_PLACES {
        bail!("decimal `{text}` has more than {DECIMAL_PLACES} digits after the point");
    }
    // pad the fraction to 6 digits: "5" → "500000"
    let frac = format!("{frac:0<DECIMAL_PLACES$}");
    let too_large = || anyhow!("decimal `{text}` is too large");
    let whole: i64 = whole.parse().map_err(|_| too_large())?;
    let frac: i64 = frac.parse().unwrap();
    whole
        .checked_mul(DECIMAL_SCALE)
        .and_then(|w| w.checked_add(frac))
        .ok_or_else(too_large)
}

/// Pratt-style precedence parser (a top-down operator-precedence algorithm).
///
/// Each operator is assigned a binding power; recursive calls enforce precedence
/// by raising `min_bp` when stepping into tighter-binding operators. This keeps
/// the implementation compact compared with writing an explicit grammar, which
/// suits this example project.
fn parse_bp(it: &mut std::iter::Peekable<std::vec::IntoIter<Tok>>, min_bp: u8) -> Result<Expr> {
    // prefix / atom
    let mut lhs = match it.next().ok_or_else(|| anyhow!("expected expression"))? {
        Tok::Int(v) => Expr::Int(v),
        Tok::Dec(v) => Expr::Dec(v),
        Tok::True => Expr::Bool(true),
        Tok::False => Expr::Bool(false),
        Tok::Ident(name) => Expr::Var(name),
        Tok::Minus if matches!(it.peek(), Some(Tok::MinMagnitude)) => {
            it.next();
            Expr::Int(i32::MIN)
        }
        Tok::Minus => Expr::Unary(UnaryOp::Neg, Box::new(parse_bp(it, 13)?)),
        // `not` binds looser than comparisons: `not a == b` means `not (a == b)`
        Tok::Not => Expr::Unary(UnaryOp::Not, Box::new(parse_bp(it, 5)?)),
        Tok::LParen => {
            let e = parse_bp(it, 0)?;
            match it.next() {
                Some(Tok::RParen) => e,
                _ => bail!("expected `)`"),
            }
        }
        t => bail!("unexpected token: {:?}", t),
    };

    // infix loop
    loop {
        // (left binding power, right binding power, operator)
        let (l_bp, r_bp, op) = match it.peek() {
            Some(Tok::Or) => (1, 2, BinOp::Or),
            Some(Tok::And) => (3, 4, BinOp::And),
            Some(Tok::EqEq) => (7, 8, BinOp::Eq),
            Some(Tok::NotEq) => (7, 8, BinOp::Ne),
            Some(Tok::Less) => (7, 8, BinOp::Lt),
            Some(Tok::LessEq) => (7, 8, BinOp::Le),
            Some(Tok::Greater) => (7, 8, BinOp::Gt),
            Some(Tok::GreaterEq) => (7, 8, BinOp::Ge),
            Some(Tok::Plus) => (9, 10, BinOp::Add),
            Some(Tok::Minus) => (9, 10, BinOp::Sub),
            Some(Tok::Star) => (11, 12, BinOp::Mul),
            Some(Tok::Slash) => (11, 12, BinOp::Div),
            _ => break,
        };
        if l_bp < min_bp {
            break;
        }
        it.next(); // consume operator
        let rhs = parse_bp(it, r_bp)?;
        lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
    }

    Ok(lhs)
}

// Minimal escapes for our language's string literals: \n \t \" \\
/// Parse and unescape the limited string literal syntax Mini supports.
fn parse_string(mut s: &str) -> Result<String> {
    if s.len() < 2 || !(s.starts_with('"') && s.ends_with('"')) {
        bail!("not a string literal");
    }
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
        } else if c == '"' {
            bail!("a `\"` inside text must be written as `\\\"`");
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expr(src: &str) -> Expr {
        parse_expr(src).unwrap()
    }

    #[test]
    fn unknown_characters_are_errors() {
        // Before the fix, `%` and `;` silently ended the expression.
        for src in [
            "let a = 2 % 3;",
            "let x = 5; let y = 6;",
            "let a = 2147483648;",
        ] {
            assert!(Parser::parse(src).is_err(), "{src} should be rejected");
        }
    }

    #[test]
    fn decimals_are_exact_millionths() {
        assert_eq!(parse_decimal("2.5").unwrap(), 2_500_000);
        assert_eq!(parse_decimal("0.1").unwrap(), 100_000);
        assert_eq!(parse_decimal("3.141592").unwrap(), 3_141_592);
        assert!(parse_decimal("1.1234567").is_err()); // 7 places
        assert!(parse_decimal("99999999999999.0").is_err()); // too large
    }

    #[test]
    fn precedence_of_logic_and_comparisons() {
        // a or b and c  →  a or (b and c)
        assert!(matches!(
            expr("a or b and c"),
            Expr::Binary(BinOp::Or, _, _)
        ));
        // not a == b  →  not (a == b)
        assert!(matches!(expr("not a == b"), Expr::Unary(UnaryOp::Not, _)));
        // 1 + 2 < 4  →  (1 + 2) < 4
        assert!(matches!(expr("1 + 2 < 4"), Expr::Binary(BinOp::Lt, _, _)));
    }

    #[test]
    fn keywords_are_not_variable_names() {
        assert!(Parser::parse("let and = 1;").is_err());
        assert!(Parser::parse("let x = 1 = 2;").is_err());
    }
    #[test]
    fn malformed_quote_and_integer_boundaries_are_checked() {
        assert!(Parser::parse("let x = \";").is_err());
        assert!(Parser::parse("let x = 2147483648;").is_err());
        assert!(Parser::parse("let x = -2147483648;").is_ok());
        assert!(Parser::parse(&format!("let x = {}1;", "(".repeat(300))).is_err());
    }
}
