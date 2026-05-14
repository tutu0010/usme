//! Defines the Abstract Syntax Tree (AST) and core data structures for USME.
//!
//! This module contains the [Expr](crate::ast::Expr) enum, which represents all mathematical
//! expressions in the engine. It handles the recursive memory layout, operator
//! precedence for formatting, and subtree substitution.

use std::fmt;

/// Represents a node in the mathematical Abstract Syntax Tree (AST).
///
/// Because expressions can be infinitely nested (e.g., `Add(Add(x, y), z)`),
/// recursive variants use `Box<Expr>` to allocate their children on the heap.
#[derive(Clone, PartialEq)]
pub enum Expr {
    /// A literal floating-point number (e.g., `5.0`).
    Num(f64),
    /// A named variable (e.g., `"x"` or `"theta"`).
    Var(String),

    /// Addition of two sub-expressions: $l + r$
    Add(Box<Expr>, Box<Expr>),
    /// Multiplication of two sub-expressions: $l \cdot r$
    Mul(Box<Expr>, Box<Expr>),
    /// Exponentiation: $l^r$. This is right-associative.
    Pow(Box<Expr>, Box<Expr>),
    /// The natural logarithm: $\ln(x)$
    Ln(Box<Expr>),
    /// Unary negation: $-x$
    Neg(Box<Expr>),
    /// The sine function: $\sin(x)$
    Sin(Box<Expr>),
    /// The cosine function: $\cos(x)$
    Cos(Box<Expr>),
}

impl Expr {
    /// Creates a new numerical constant expression.
    ///
    /// # Examples
    /// ```
    /// use usme::ast::Expr;
    /// let n = Expr::num(3.14);
    /// assert_eq!(format!("{}", n), "3.14");
    /// ```
    pub fn num(n: f64) -> Self {
        Expr::Num(n)
    }

    /// Creates a new variable expression.
    ///
    /// # Examples
    /// ```
    /// use usme::ast::Expr;
    /// let v = Expr::var("x");
    /// assert_eq!(format!("{}", v), "x");
    /// ```
    pub fn var(v: &str) -> Self {
        Expr::Var(v.to_string())
    }

    /// Wraps the current expression in a Sine function.
    pub fn sin(self) -> Self {
        Expr::Sin(Box::new(self))
    }

    /// Wraps the current expression in a Cosine function.
    pub fn cos(self) -> Self {
        Expr::Cos(Box::new(self))
    }

    /// Raises the current expression to the power of the given `exponent`.
    ///
    /// # Examples
    /// ```
    /// use usme::ast::Expr;
    /// let x_sq = Expr::var("x").pow(Expr::num(2.0));
    /// assert_eq!(format!("{}", x_sq), "x^2");
    /// ```
    pub fn pow(self, exponent: Expr) -> Self {
        Expr::Pow(Box::new(self), Box::new(exponent))
    }

    /// Wraps the current expression in a Natural Logarithm function.
    pub fn ln(self) -> Self {
        Expr::Ln(Box::new(self))
    }

    /// Internal helper to determine the mathematical "weight" of a node.
    /// Used by the formatter to decide when parentheses are necessary.
    fn precedence(&self) -> u8 {
        match self {
            Expr::Num(_) | Expr::Var(_) => 100,
            Expr::Ln(_) | Expr::Sin(_) | Expr::Cos(_) | Expr::Neg(_) => 4,
            Expr::Pow(_, _) => 3,
            Expr::Mul(_, _) => 2,
            Expr::Add(_, _) => 1,
        }
    }

    /// Recursively formats the AST into a mathematical string, inserting
    /// parentheses only when a child's precedence is strictly less than its parent's.
    fn format_with_precedence(&self, f: &mut fmt::Formatter<'_>, parent_prec: u8) -> fmt::Result {
        let my_prec = self.precedence();
        let use_parens: bool = my_prec < parent_prec;

        if use_parens {
            write!(f, "(")?;
        }

        match self {
            Expr::Num(n) => write!(f, "{}", n)?,
            Expr::Var(s) => write!(f, "{}", s)?,
            Expr::Neg(inner) => {
                write!(f, "-")?;
                inner.format_with_precedence(f, my_prec)?;
            }
            Expr::Add(l, r) => {
                l.format_with_precedence(f, my_prec)?;
                write!(f, " + ")?;
                r.format_with_precedence(f, my_prec)?;
            }
            Expr::Mul(l, r) => {
                l.format_with_precedence(f, my_prec)?;
                write!(f, " * ")?;
                r.format_with_precedence(f, my_prec)?;
            }
            Expr::Pow(l, r) => {
                // Power is tricky: right-associative!
                l.format_with_precedence(f, my_prec + 1)?;
                write!(f, "^")?;
                r.format_with_precedence(f, my_prec)?;
            }
            Expr::Ln(inner) => {
                write!(f, "ln(")?;
                inner.format_with_precedence(f, 0)?;
                write!(f, ")")?;
            }
            Expr::Sin(inner) => {
                write!(f, "sin(")?;
                inner.format_with_precedence(f, 0)?; // inside content starts fresh
                write!(f, ")")?;
            }
            Expr::Cos(inner) => {
                write!(f, "cos(")?;
                inner.format_with_precedence(f, 0)?;
                write!(f, ")")?;
            }
        }

        if use_parens {
            write!(f, ")")?;
        }
        Ok(())
    }

    /// Substitutes all instances of a specific variable with a replacement expression.
    ///
    /// This performs a deep clone of the replacement tree wherever the target
    /// variable is found. It is highly useful for calculating Total Derivatives
    /// or composing functions.
    ///
    /// # Examples
    /// ```
    /// use usme::ast::Expr;
    ///
    /// // f(x) = x^2
    /// let f = Expr::var("x").pow(Expr::num(2.0));
    ///
    /// // Replace "x" with "sin(y)"
    /// let replacement = Expr::var("y").sin();
    /// let composed = f.substitute("x", &replacement);
    ///
    /// assert_eq!(format!("{}", composed), "sin(y)^2");
    /// ```
    pub fn substitute(&self, var: &str, replacement: &Expr) -> Expr {
        match self {
            // The Base Case: If this is the variable we are looking for,
            // swap it for the replacement tree.
            Expr::Var(s) if s == var => replacement.clone(),

            // Recurse into all other variants
            Expr::Add(l, r) => Expr::Add(
                Box::new(l.substitute(var, replacement)),
                Box::new(r.substitute(var, replacement)),
            ),
            Expr::Mul(l, r) => Expr::Mul(
                Box::new(l.substitute(var, replacement)),
                Box::new(r.substitute(var, replacement)),
            ),
            Expr::Pow(l, r) => Expr::Pow(
                Box::new(l.substitute(var, replacement)),
                Box::new(r.substitute(var, replacement)),
            ),
            Expr::Neg(inner) => Expr::Neg(Box::new(inner.substitute(var, replacement))),
            Expr::Sin(inner) => Expr::Sin(Box::new(inner.substitute(var, replacement))),
            Expr::Cos(inner) => Expr::Cos(Box::new(inner.substitute(var, replacement))),
            Expr::Ln(inner) => Expr::Ln(Box::new(inner.substitute(var, replacement))),

            // If it's a number or a different variable, leave it alone
            other => other.clone(),
        }
    }
}

/// Allows the AST to be printed cleanly using `println!("{}", expr)`.
impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Precedence 0 ensures the outermost expression never gets wrapped in parentheses.
        self.format_with_precedence(f, 0)
    }
}
