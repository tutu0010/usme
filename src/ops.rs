//! Provides operator overloading (`+`, `-`, `*`, `/`) for ergonomic AST construction.
//!
//! This module implements the `std::ops` traits for the [Expr](crate::ast::Expr) enum, allowing 
//! developers to construct complex mathematical trees using native Rust syntax 
//! rather than manual builder methods or verbose enum variant construction.
//!
//! # Ergonomics and Ownership
//! Because an `Expr` represents an allocated tree on the heap, it does not 
//! implement `Copy`. To make math feel natural without forcing the user to 
//! explicitly `.clone()` variables constantly, operators are implemented for 
//! both owned expressions (`Expr`) and references (`&Expr`).
//! 
//! # Compositional Rules
//! To keep the AST minimal, subtraction and division are not standalone variants.
//! They are automatically composed:
//! * **Subtraction**: $x - y$ becomes $x + (-y)$
//! * **Division**: $x / y$ becomes $x \cdot y^{-1}$

use crate::ast::Expr;
use std::ops::{Add, Div, Mul, Neg, Sub};

// --- Addition ---

/// Implements `+` for owned expressions. Consumes both operands.
impl Add for Expr {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Expr::Add(Box::new(self), Box::new(rhs))
    }
}

/// Implements `+` for referenced expressions. Performs a deep clone of the tree.
impl Add for &Expr {
    type Output = Expr;
    fn add(self, rhs: Self) -> Expr {
        self.clone() + rhs.clone()
    }
}

// --- Multiplication ---

/// Implements `*` for owned expressions. Consumes both operands.
impl Mul for Expr {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Expr::Mul(Box::new(self), Box::new(rhs))
    }
}

/// Implements `*` for referenced expressions. Performs a deep clone of the tree.
impl Mul for &Expr {
    type Output = Expr;
    fn mul(self, rhs: Self) -> Expr {
        self.clone() * rhs.clone()
    }
}

// --- Negation ---

/// Implements unary `-` for an owned expression. Consumes the operand.
impl Neg for Expr {
    type Output = Self;
    fn neg(self) -> Self {
        Expr::Neg(Box::new(self))
    }
}

/// Implements unary `-` for a referenced expression. Performs a deep clone.
impl Neg for &Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        -(self.clone())
    }
}

// --- Subtraction ---

/// Implements `-` for owned expressions.
///
/// Under the hood, this creates an `Add` node where the right-hand side 
/// is wrapped in a `Neg` node.
impl Sub for Expr {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

/// Implements `-` for referenced expressions. Performs a deep clone.
impl Sub for &Expr {
    type Output = Expr;
    fn sub(self, rhs: Self) -> Expr {
        self.clone() - rhs.clone()
    }
}

// --- Division ---

/// Implements `/` for owned expressions.
///
/// Under the hood, this creates a `Mul` node where the right-hand side 
/// is wrapped in a `Pow` node with an exponent of `-1.0`.
impl Div for Expr {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        self * Expr::Pow(Box::new(rhs), Box::new(Expr::Num(-1.0)))
    }
}

/// Implements `/` for referenced expressions. Performs a deep clone.
impl Div for &Expr {
    type Output = Expr;
    fn div(self, rhs: Self) -> Expr {
        self.clone() / rhs.clone()
    }
}