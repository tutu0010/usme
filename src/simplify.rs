//! Implements the fixed-point iterative reduction engine for algebraic cleanup.
//!
//! Symbolic differentiation often produces mathematically correct but structurally
//! redundant trees (e.g., `x * 1 + 0`). This module provides the logic to recursively
//! pattern-match and eliminate these redundancies.
//!
//! The simplifier uses a **Fixed-Point Iteration** strategy. Because simplifying one
//! part of a tree might expose a new simplification opportunity higher up, the engine
//! continuously applies its rule set until the AST reaches a stable state where no
//! further reductions are possible.

use crate::ast::Expr;

impl Expr {
    /// Reduces the expression to its simplest algebraic form.
    ///
    /// This method performs a fixed-point iteration. It repeatedly applies algebraic
    /// reduction rules (like constant folding, zero rules, and identity rules) until
    /// the output of a simplification pass is identical to its input.
    ///
    /// # Mathematical Rules Applied
    /// * **Constants**: `2 + 3 -> 5`, `2 * 3 -> 6`, `2^3 -> 8`
    /// * **Identities**: `x + 0 -> x`, `x * 1 -> x`, `x^1 -> x`
    /// * **Annihilation**: `x * 0 -> 0`, `x^0 -> 1`
    /// * **Double Negation**: `-(-x) -> x`
    /// * **Logarithmic**: `ln(1) -> 0`
    ///
    /// # Limitations
    /// The simplifier does not currently support polynomial expansion, factoring,
    /// or term collection (e.g., `x + x` will not simplify to `2*x`).
    ///
    /// # Examples
    /// ```
    /// use usme::parser::parse;
    ///
    /// // A messy expression often created by the product/chain rules
    /// let messy = parse("(x * 1) + (y * 0)").unwrap();
    ///
    /// // The simplifier reduces it to just 'x'
    /// let clean = messy.simplify();
    /// assert_eq!(format!("{}", clean), "x");
    /// ```
    pub fn simplify(&self) -> Expr {
        let mut current = self.clone();
        loop {
            let next = current.simplify_once();
            if next == current {
                return next;
            }
            current = next;
        }
    }

    fn simplify_once(&self) -> Expr {
        match self {
            // First, recursively simplify children (Bottom-Up)
            Expr::Add(l, r) => {
                let l = l.simplify_once();
                let r = r.simplify_once();

                match (l, r) {
                    // Constant Folding: 2 + 3 = 5
                    (Expr::Num(a), Expr::Num(b)) => Expr::num(a + b),
                    // Neutral Element: x + 0 = x
                    (l, Expr::Num(n)) if n == 0.0 => l,
                    (Expr::Num(n), r) if n == 0.0 => r,
                    (l, r) => Expr::Add(Box::new(l), Box::new(r)),
                }
            }

            Expr::Mul(l, r) => {
                let l = l.simplify_once();
                let r = r.simplify_once();

                match (l, r) {
                    // Constant Folding: 2 * 3 = 6
                    (Expr::Num(a), Expr::Num(b)) => Expr::num(a * b),
                    // Zero Rule: x * 0 = 0
                    (_, Expr::Num(n)) if n == 0.0 => Expr::num(0.0),
                    (Expr::Num(n), _) if n == 0.0 => Expr::num(0.0),
                    // Neutral Element: x * 1 = x
                    (l, Expr::Num(n)) if n == 1.0 => l,
                    (Expr::Num(n), r) if n == 1.0 => r,
                    (l, r) => Expr::Mul(Box::new(l), Box::new(r)),
                }
            }

            Expr::Pow(l, r) => {
                let l = l.simplify_once();
                let r = r.simplify_once();

                match (l, r) {
                    // x^0 = 1
                    (_, Expr::Num(n)) if n == 0.0 => Expr::num(1.0),
                    // x^1 = x
                    (l, Expr::Num(n)) if n == 1.0 => l,
                    // 1^x = 1
                    (Expr::Num(n), _) if n == 1.0 => Expr::num(1.0),
                    // Constant Folding
                    (Expr::Num(a), Expr::Num(b)) => Expr::num(a.powf(b)),
                    (l, r) => Expr::Pow(Box::new(l), Box::new(r)),
                }
            }

            Expr::Ln(inner) => {
                let inner = inner.simplify_once();
                match inner {
                    Expr::Num(n) if n == 1.0 => Expr::num(0.0), // ln(1) = 0
                    other => Expr::Ln(Box::new(other)),
                }
            }

            Expr::Neg(inner) => {
                let inner = inner.simplify_once();
                match inner {
                    // Double Negation: -(-x) = x
                    Expr::Neg(inner_inner) => *inner_inner,
                    // -0 = 0
                    Expr::Num(n) if n == 0.0 => Expr::num(0.0),
                    Expr::Num(n) => Expr::num(-n),
                    other => Expr::Neg(Box::new(other)),
                }
            }

            // For Sin/Cos, just simplify the inner part
            Expr::Sin(inner) => Expr::Sin(Box::new(inner.simplify_once())),
            Expr::Cos(inner) => Expr::Cos(Box::new(inner.simplify_once())),

            // Atoms (Num/Var) stay as they are
            other => other.clone(),
        }
    }
}
