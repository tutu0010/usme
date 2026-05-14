//! Numerical evaluation of the Abstract Syntax Tree.
//!
//! This module provides the capability to compute a concrete floating-point
//! result from a symbolic expression tree. It relies on a provided context
//! (a `HashMap`) to substitute numerical values for named variables.

use crate::ast::Expr;
use std::collections::HashMap;
use std::fmt;

/// Errors that can occur during the numerical evaluation of an expression.
#[derive(Debug)]
pub enum EvalError {
    /// Returned when the expression contains a variable name that was not
    /// provided in the evaluation context map.
    UndefinedVariable(String),
    /// Represents generic mathematical errors (e.g., domain errors),
    /// reserved for future expansion.
    MathError(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::UndefinedVariable(v) => write!(f, "Variable '{}' not found", v),
            EvalError::MathError(m) => write!(f, "Math Error: {}", m),
        }
    }
}

impl Expr {
    /// Recursively evaluates the expression into a concrete floating-point number.
    ///
    /// The evaluation traverses the AST from the bottom up, replacing any `Expr::Var`
    /// nodes with their corresponding values from the `vars` map. Operations are
    /// performed using standard Rust `f64` arithmetic.
    ///
    /// # Errors
    /// This function returns an `EvalError::UndefinedVariable` if it encounters a
    /// variable in the AST that does not exist as a key in the `vars` HashMap.
    ///
    /// # Examples
    /// ```
    /// use usme::parser::parse;
    /// use std::collections::HashMap;
    ///
    /// // Parse an expression with variables
    /// let expr = parse("x^2 + y").unwrap();
    ///
    /// // Provide the numerical context
    /// let mut vars = HashMap::new();
    /// vars.insert("x".to_string(), 3.0);
    /// vars.insert("y".to_string(), 10.0);
    ///
    /// // Evaluate the expression
    /// let result = expr.eval(&vars).unwrap();
    /// assert_eq!(result, 19.0);
    /// ```
    pub fn eval(&self, vars: &HashMap<String, f64>) -> Result<f64, EvalError> {
        match self {
            // Base cases
            Expr::Num(n) => Ok(*n),

            Expr::Var(s) => vars
                .get(s)
                .copied()
                .ok_or_else(|| EvalError::UndefinedVariable(s.clone())),

            // Binary operations
            Expr::Add(l, r) => Ok(l.eval(vars)? + r.eval(vars)?),

            Expr::Mul(l, r) => Ok(l.eval(vars)? * r.eval(vars)?),

            Expr::Pow(l, r) => {
                let base = l.eval(vars)?;
                let exp = r.eval(vars)?;
                Ok(base.powf(exp))
            }

            // Unary functions
            Expr::Ln(inner) => Ok(inner.eval(vars)?.ln()),

            Expr::Neg(inner) => Ok(-inner.eval(vars)?),

            Expr::Sin(inner) => Ok(inner.eval(vars)?.sin()),

            Expr::Cos(inner) => Ok(inner.eval(vars)?.cos()),
        }
    }
}
