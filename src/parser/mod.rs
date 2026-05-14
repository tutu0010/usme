//! Recursive Descent Parser for the USME engine.
//!
//! This module converts sequential tokens into a hierarchical Abstract Syntax Tree (AST).
//! It enforces mathematical order of operations (PEMDAS) by nesting parsing functions
//! from lowest precedence (Addition) to highest precedence (Parentheses/Atoms).

pub mod tokenizer;
use crate::ast::Expr;
use std::iter::Peekable;
use std::slice::Iter;
use tokenizer::{tokenize, Token};

/// Parses a mathematical string expression into an Abstract Syntax Tree (AST).
///
/// This is the primary entry point for converting human-readable input into
/// manipulable symbolic expressions.
///
/// # Errors
/// Returns a `String` containing an error message if the input contains
/// invalid characters, mismatched parentheses, or trailing tokens that cannot
/// be parsed into a single expression.
///
/// # Examples
/// ```
/// use usme::parser::parse;
///
/// let expr = parse("x^2 + 3 * y").expect("Failed to parse");
/// assert_eq!(format!("{}", expr), "x^2 + 3 * y");
/// ```
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    let mut it = tokens.iter().peekable();
    let expr = parse_expression(&mut it)?;

    if it.peek().is_none() {
        Ok(expr)
    } else {
        Err("Trailing tokens after expression".to_string())
    }
}

/// Parses lowest-precedence binary operators: Addition (`+`) and Subtraction (`-`).
///
/// Subtraction is internally converted to addition of a negated expression: `a - b` -> `a + (-b)`.
fn parse_expression(it: &mut Peekable<Iter<Token>>) -> Result<Expr, String> {
    let mut expr = parse_term(it)?;
    while let Some(t) = it.peek() {
        match t {
            Token::Plus => {
                it.next();
                let rhs = parse_term(it)?;
                expr = expr + rhs;
            }
            Token::Minus => {
                it.next();
                let rhs = parse_term(it)?;
                expr = expr - rhs;
            }
            _ => break,
        }
    }
    Ok(expr)
}

/// Parses medium-precedence binary operators: Multiplication (`*`) and Division (`/`).
///
/// Division is internally converted to multiplication by a power of negative one: `a / b` -> `a * b^(-1)`.
fn parse_term(it: &mut Peekable<Iter<Token>>) -> Result<Expr, String> {
    let mut expr = parse_power(it)?;
    while let Some(t) = it.peek() {
        match t {
            Token::Star => {
                it.next();
                let rhs = parse_power(it)?;
                expr = expr * rhs;
            }
            Token::Slash => {
                it.next();
                let rhs = parse_power(it)?;
                expr = expr / rhs;
            }
            _ => break,
        }
    }
    Ok(expr)
}

/// Parses exponentiation operators (`^`).
///
/// Note that exponentiation is **right-associative**.
/// `a^b^c` is parsed as `a^(b^c)`, not `(a^b)^c`.
fn parse_power(it: &mut Peekable<Iter<Token>>) -> Result<Expr, String> {
    let lhs = parse_unary(it)?;
    if let Some(Token::Caret) = it.peek() {
        it.next();
        // Power is right-associative, so we recurse into parse_power, not parse_unary
        let rhs = parse_power(it)?;
        Ok(Expr::Pow(Box::new(lhs), Box::new(rhs)))
    } else {
        Ok(lhs)
    }
}

/// Parses unary operators and functions, such as negation (`-`), `sin`, `cos`, and `ln`.
fn parse_unary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, String> {
    match it.peek() {
        Some(Token::Minus) => {
            it.next();
            Ok(-parse_unary(it)?)
        }
        Some(Token::Sin) => {
            it.next();
            consume(it, Token::LParen)?;
            let inner = parse_expression(it)?;
            consume(it, Token::RParen)?;
            Ok(inner.sin())
        }
        Some(Token::Cos) => {
            it.next();
            consume(it, Token::LParen)?;
            let inner = parse_expression(it)?;
            consume(it, Token::RParen)?;
            Ok(inner.cos())
        }
        Some(Token::Ln) => {
            it.next();
            consume(it, Token::LParen)?;
            let inner = parse_expression(it)?;
            consume(it, Token::RParen)?;
            Ok(inner.ln())
        }
        _ => parse_primary(it),
    }
}

/// Parses primary expressions (Atoms): numerical constants, variables, and parenthesized sub-expressions.
fn parse_primary(it: &mut Peekable<Iter<Token>>) -> Result<Expr, String> {
    match it.next() {
        Some(Token::Num(n)) => Ok(Expr::num(*n)),
        Some(Token::Var(s)) => Ok(Expr::var(s)),
        Some(Token::LParen) => {
            let expr = parse_expression(it)?;
            consume(it, Token::RParen)?;
            Ok(expr)
        }
        Some(t) => Err(format!("Unexpected token: {:?}", t)),
        None => Err("Unexpected end of input".to_string()),
    }
}

/// Helper function to advance the iterator only if the next token matches the expected one.
fn consume(it: &mut Peekable<Iter<Token>>, expected: Token) -> Result<(), String> {
    match it.next() {
        Some(t) if t == &expected => Ok(()),
        Some(t) => Err(format!("Expected {:?}, found {:?}", expected, t)),
        None => Err(format!("Expected {:?}, found end of input", expected)),
    }
}
