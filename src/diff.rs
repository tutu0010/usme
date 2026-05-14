//! Symbolic differentiation rules for the AST.
//!
//! This module implements the core calculus logic of USME. It provides the ability
//! to recursively traverse an expression tree and calculate exact symbolic derivatives
//! using standard mathematical rules (Sum, Product, Chain, and General Power rules).

use crate::ast::Expr;

impl Expr {
    /// Computes the symbolic derivative of the expression with respect to a given variable.
    ///
    /// This method performs a recursive transformation of the Abstract Syntax Tree (AST).
    /// Whenever it encounters a mathematical operation, it applies the corresponding
    /// calculus rule, propagating the differentiation down to its children.
    ///
    /// # Note on Simplification
    /// The resulting expression is mathematically exact but often contains algebraically
    /// redundant terms (e.g., `* 1`, `+ 0`, or `x^1`). It is highly recommended to chain
    /// this method with [`simplify()`](crate::ast::Expr::simplify) to collapse the AST
    /// into a readable form.
    ///
    /// # Mathematical Rules Applied
    /// * **Constants**: $\frac{d}{dx}\[c\] = 0$
    /// * **Variables**: $\frac{d}{dx}\[x\] = 1$, $\frac{d}{dx}\[y\] = 0$
    /// * **Sum**: $(f + g)' = f' + g'$
    /// * **Product**: $(f \cdot g)' = f'g + fg'$
    /// * **Logarithm**: $\frac{d}{dx}[\ln(f)] = \frac{f'}{f}$
    /// * **Trigonometry**: $\frac{d}{dx}[\sin(f)] = \cos(f) \cdot f'$, $\frac{d}{dx}[\cos(f)] = -\sin(f) \cdot f'$
    /// * **General Power**: $(f^g)' = f^g \left(g'\ln(f) + \frac{g f'}{f}\right)$
    ///
    /// # Examples
    /// ```
    /// use usme::parser::parse;
    ///
    /// // Parse a simple linear expression: 3 * x
    /// let expr = parse("3 * x").unwrap();
    ///
    /// // Compute the derivative with respect to 'x' and simplify it
    /// let deriv = expr.diff("x").simplify();
    ///
    /// // The derivative of 3x is exactly 3
    /// assert_eq!(format!("{}", deriv), "3");
    /// ```
    pub fn diff(&self, var: &str) -> Expr {
        match self {
            // Constant Rule: d/dx(c) = 0
            Expr::Num(_) => Expr::num(0.0),

            // Variable Rule: d/dx(x) = 1, d/dx(y) = 0
            Expr::Var(s) => {
                if s == var {
                    Expr::num(1.0)
                } else {
                    Expr::num(0.0)
                }
            }

            // Sum Rule: (f + g)' = f' + g'
            Expr::Add(l, r) => l.diff(var) + r.diff(var),

            // Product Rule: (f * g)' = f' * g + f * g'
            Expr::Mul(l, r) => {
                let f_prime = l.diff(var);
                let g_prime = r.diff(var);
                // We must clone the original l and r because we are moving them
                // into a new expression tree while still keeping the originals.
                (f_prime * r.as_ref().clone()) + (l.as_ref().clone() * g_prime)
            }

            Expr::Pow(l, r) => {
                let f = l.as_ref().clone();
                let g = r.as_ref().clone();
                let f_prime = l.diff(var);
                let g_prime = r.diff(var);

                // General Power Rule: f^g * (g' * ln(f) + g * f' / f)
                // Note: / f is implemented as * f^-1
                let term1 = g_prime * f.clone().ln();
                let term2 = g.clone() * f_prime * f.clone().pow(Expr::num(-1.0));

                self.clone() * (term1 + term2)
            }

            // And add the Ln rule
            Expr::Ln(inner) => {
                let f = inner.as_ref().clone();
                let f_prime = inner.diff(var);
                // d/dx ln(f) = f' / f
                f_prime * f.pow(Expr::num(-1.0))
            }

            // Negation Rule: (-f)' = -(f')
            Expr::Neg(inner) => -inner.diff(var),

            // Chain Rule (Sin): sin(u)' = cos(u) * u'
            Expr::Sin(inner) => {
                let u = inner.as_ref().clone();
                let u_prime = inner.diff(var);
                Expr::Cos(Box::new(u)) * u_prime
            }

            // Chain Rule (Cos): cos(u)' = -sin(u) * u'
            Expr::Cos(inner) => {
                let u = inner.as_ref().clone();
                let u_prime = inner.diff(var);
                -(Expr::Sin(Box::new(u)) * u_prime)
            }
        }
    }
}
