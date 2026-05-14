//! # USME — Universal Symbolic Math Engine
//!
//! USME is a lightweight symbolic expression engine focused on
//! differentiation and simplification of mathematical trees.
//!
//! It is designed to be a transparent, systems-level approach to symbolic
//! computation, leveraging Rust's algebraic data types and pattern matching.
//!
//! ## Core Features
//! * **Parsing**: String to AST via Recursive Descent.
//! * **Differentiation**: Symbolic rules for polynomials, trig, logs, and the general power rule.
//! * **Simplification**: Fixed-point reduction of algebraic redundancies.
//! * **Evaluation**: Fast numerical substitution using HashMaps.
//!
//! ## Quick Start
//! ```rust
//! use std::collections::Ha`shMap;
//!
//! // 1. Parse an expression
//! let expr = parse("x^2 * sin(x)").unwrap();
//!
//! // 2. Differentiate symbolically
//! let deriv = expr.diff("x");
//!
//! // 3. Simplify the resulting tree
//! let clean = deriv.simplify();
//!
//! // 4. Evaluate numerically
//! let mut vars = HashMap::new();
//! vars.insert("x".to_string(), std::f64::consts::PI / 2.0);
//! let val = clean.eval(&vars).unwrap();
//! ```

/// Defines the Abstract Syntax Tree (AST) via the `Expr` enum and its base methods.
pub mod ast;

/// Implements the symbolic differentiation rules (Sum, Product, Chain, Power).
pub mod diff;

/// Handles the numerical evaluation of expressions at specific data points.
pub mod eval;

/// Provides operator overloading (`+`, `-`, `*`, `/`) for ergonomic AST construction.
pub mod ops;

/// Contains the Lexer (Tokenizer) and Recursive Descent Parser.
pub mod parser;

/// Implements the fixed-point iterative reduction engine for algebraic cleanup.
pub mod simplify;
