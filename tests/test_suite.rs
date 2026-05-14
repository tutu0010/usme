// =============================================================================
// USME — Test Suite
// =============================================================================
//
// Three layers of testing:
//
//   1. Golden Tests       — Known closed-form derivatives, verified symbolically
//                           and numerically. The ground truth layer.
//
//   2. Gradient Checks    — Symbolic derivative vs central finite difference.
//                           The correctness workhorse. Catches wrong rules,
//                           sign errors, missing chain rule applications, etc.
//
//   3. Property Tests     — Algebraic invariants that must hold for any valid
//                           symbolic engine: idempotency, linearity, numerical
//                           equivalence under simplification.
//
// Run with:
//   cargo test
//   cargo test -- --nocapture        (to see printed values)
//
// Dev dependencies required in Cargo.toml:
//
//   [dev-dependencies]
//   proptest = "1"
//
// =============================================================================

// Adjust these imports to match your actual module paths
use usme::ast::Expr;
use usme::parser::parse;

use std::collections::HashMap;

// ── Numerical helpers ────────────────────────────────────────────────────────

/// Step size for central finite difference. 1e-5 is a sweet spot:
/// small enough for accuracy, large enough to avoid f64 cancellation.
const H: f64 = 1e-5;

/// Relative tolerance for gradient checks. 1e-4 handles most smooth functions.
/// Tighten to 1e-5 if your simplifier is clean; loosen to 1e-3 near singularities.
const TOL: f64 = 1e-4;

/// Evaluate an expression at a single point x.
fn eval_at(expr: &Expr, x: f64) -> Option<f64> {
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), x);
    expr.eval(&vars).ok()
}

/// Central finite difference approximation of d/dx at point x.
/// (f(x+h) - f(x-h)) / 2h  — O(h²) error, much better than forward difference.
fn finite_diff(expr: &Expr, x: f64) -> Option<f64> {
    let f_plus = eval_at(expr, x + H)?;
    let f_minus = eval_at(expr, x - H)?;
    Some((f_plus - f_minus) / (2.0 * H))
}

/// Core gradient check: symbolic derivative evaluated at x should match
/// the finite difference of the original expression at x.
///
/// Uses relative error with an absolute fallback for near-zero values.
/// Returns false (rather than panicking) if evaluation fails — callers
/// can decide whether a failed eval is a test failure or a domain issue.
fn gradient_check(input: &str, x: f64) -> bool {
    let expr = parse(input).expect("parse failed");
    let deriv = expr.diff("x").simplify();

    let symbolic = match eval_at(&deriv, x) {
        Some(v) if v.is_finite() => v,
        _ => return false,
    };
    let numerical = match finite_diff(&expr, x) {
        Some(v) if v.is_finite() => v,
        _ => return false,
    };

    // Relative error, falling back to absolute for near-zero values
    let scale = numerical.abs().max(1.0);
    let error = (symbolic - numerical).abs() / scale;

    if error >= TOL {
        eprintln!(
            "[GRADIENT FAIL] expr='{}' at x={}\n  symbolic={:.8}  numerical={:.8}  error={:.2e}",
            input, x, symbolic, numerical, error
        );
    }

    error < TOL
}

// ── 1. GOLDEN TESTS ──────────────────────────────────────────────────────────
//
// These test known closed-form derivatives both:
//   (a) structurally — by checking numerical agreement at multiple points
//   (b) symbolically — by evaluating the *known* formula and comparing
//
// If a golden test fails, you have a broken differentiation rule.
// If a gradient check passes but a golden test fails, your simplifier
// is producing a mathematically correct but structurally messy result —
// which is also useful to know.

#[cfg(test)]
mod golden_tests {
    use super::*;

    /// Helper: check that diff(input) ≈ expected at several x values.
    fn check_derivative_matches(input: &str, expected: &str, xs: &[f64]) {
        let expr = parse(input).expect("parse failed");
        let expected = parse(expected).expect("expected parse failed");
        let deriv = expr.diff("x").simplify();

        for &x in xs {
            let got = eval_at(&deriv, x);
            let exp = eval_at(&expected, x);
            match (got, exp) {
                (Some(g), Some(e)) if g.is_finite() && e.is_finite() => {
                    let scale = e.abs().max(1.0);
                    let error = (g - e).abs() / scale;
                    assert!(
                        error < 1e-6,
                        "d/dx[{}] at x={}: got {:.8}, expected {:.8} (error {:.2e})",
                        input,
                        x,
                        g,
                        e,
                        error
                    );
                }
                _ => { /* skip non-finite points (e.g. 1/x at x=0) */ }
            }
        }
    }

    const XS: &[f64] = &[-2.0, -0.5, 0.1, 0.5, 1.0, 1.5, 2.0, 3.0];

    // ── Polynomial / Power ───────────────────────────────────────────────────

    #[test]
    fn d_constant_is_zero() {
        let expr = parse("5").unwrap();
        let deriv = expr.diff("x").simplify();
        for &x in XS {
            let v = eval_at(&deriv, x).unwrap_or(f64::NAN);
            assert!(v.abs() < 1e-10, "d/dx[5] should be 0 at x={}, got {}", x, v);
        }
    }

    #[test]
    fn d_x_is_one() {
        let expr = parse("x").unwrap();
        let deriv = expr.diff("x").simplify();
        for &x in XS {
            let v = eval_at(&deriv, x).unwrap_or(f64::NAN);
            assert!(
                (v - 1.0).abs() < 1e-10,
                "d/dx[x] should be 1 at x={}, got {}",
                x,
                v
            );
        }
    }

    #[test]
    fn d_x_squared() {
        // d/dx[x^2] = 2x
        check_derivative_matches("x^2", "2*x", XS);
    }

    #[test]
    fn d_x_cubed() {
        // d/dx[x^3] = 3*x^2
        check_derivative_matches("x^3", "3*x^2", XS);
    }

    #[test]
    fn d_polynomial() {
        // d/dx[x^3 + 2*x^2 + x + 1] = 3*x^2 + 4*x + 1
        check_derivative_matches("x^3 + 2*x^2 + x + 1", "3*x^2 + 4*x + 1", XS);
    }

    // ── Trigonometric ────────────────────────────────────────────────────────

    #[test]
    fn d_sin_x() {
        // d/dx[sin(x)] = cos(x)
        check_derivative_matches("sin(x)", "cos(x)", XS);
    }

    #[test]
    fn d_cos_x() {
        // d/dx[cos(x)] = -sin(x)
        check_derivative_matches("cos(x)", "-1*sin(x)", XS);
    }

    #[test]
    fn d_sin_x_squared() {
        // d/dx[sin(x^2)] = 2x*cos(x^2)  — chain rule
        check_derivative_matches("sin(x^2)", "2*x*cos(x^2)", XS);
    }

    // ── Logarithm ────────────────────────────────────────────────────────────

    #[test]
    fn d_ln_x() {
        // d/dx[ln(x)] = 1/x  — only test positive x
        let xs_pos = &[0.1f64, 0.5, 1.0, 1.5, 2.0, 3.0];
        check_derivative_matches("ln(x)", "1/x", xs_pos);
    }

    #[test]
    fn d_ln_x_squared() {
        // d/dx[ln(x^2)] = 2/x
        let xs_pos = &[0.1f64, 0.5, 1.0, 1.5, 2.0, 3.0];
        check_derivative_matches("ln(x^2)", "2/x", xs_pos);
    }

    // ── Product Rule ─────────────────────────────────────────────────────────

    #[test]
    fn d_x_sin_x() {
        // d/dx[x*sin(x)] = sin(x) + x*cos(x)
        check_derivative_matches("x*sin(x)", "sin(x) + x*cos(x)", XS);
    }

    #[test]
    fn d_x_squared_cos_x() {
        // d/dx[x^2*cos(x)] = 2*x*cos(x) - x^2*sin(x)
        check_derivative_matches("x^2*cos(x)", "2*x*cos(x) - x^2*sin(x)", XS);
    }

    // ── Quotient Rule ────────────────────────────────────────────────────────

    #[test]
    fn d_sin_over_x() {
        // d/dx[sin(x)/x] = (x*cos(x) - sin(x)) / x^2
        let xs_nonzero = &[-2.0f64, -0.5, 0.5, 1.0, 1.5, 2.0];
        check_derivative_matches("sin(x)/x", "(x*cos(x) - sin(x)) / (x^2)", xs_nonzero);
    }

    // ── General Power Rule (the fancy one) ───────────────────────────────────

    #[test]
    fn d_x_to_the_x() {
        // d/dx[x^x] = x^x * (ln(x) + 1)
        let xs_pos = &[0.5f64, 1.0, 1.5, 2.0, 3.0];
        check_derivative_matches("x^x", "x^x * (ln(x) + 1)", xs_pos);
    }

    #[test]
    fn d_x_squared_plus_1_to_sin_x() {
        // d/dx[(x^2+1)^sin(x)] — no clean closed form, just gradient check
        let xs_pos = &[0.5f64, 1.0, 1.5, 2.0];
        for &x in xs_pos {
            assert!(
                gradient_check("(x^2 + 1)^sin(x)", x),
                "gradient check failed for (x^2+1)^sin(x) at x={}",
                x
            );
        }
    }
}

// ── 2. GRADIENT CHECKS ───────────────────────────────────────────────────────
//
// No closed-form reference needed. Just: does the symbolic derivative
// numerically agree with (f(x+h) - f(x-h)) / 2h ?
//
// This is the widest-coverage layer. Add any expression you're unsure about.

#[cfg(test)]
mod gradient_checks {
    use super::*;

    /// Run gradient check at several representative x values.
    /// Skips NaN/Inf points (e.g. expressions undefined at x=0).
    fn check_at_standard_points(input: &str) {
        let xs = [-2.0_f64, -1.0, -0.3, 0.3, 0.7, 1.0, 1.5, 2.5];
        let mut passed = 0;
        let mut skipped = 0;

        for &x in &xs {
            if gradient_check(input, x) {
                passed += 1;
            } else {
                // Check if it was a domain issue or an actual failure
                let expr = parse(input).unwrap();
                let f_val = eval_at(&expr, x);
                if f_val.map(|v| v.is_finite()).unwrap_or(false) {
                    // Function is defined here — this is a real failure
                    panic!(
                        "gradient_check failed for '{}' at x={} (function IS defined here)",
                        input, x
                    );
                } else {
                    skipped += 1;
                }
            }
        }

        assert!(
            passed >= 3,
            "gradient_check for '{}' passed at only {}/{} valid points (skipped {})",
            input,
            passed,
            xs.len(),
            skipped
        );
    }

    // ── Basic ────────────────────────────────────────────────────────────────

    #[test]
    fn gc_x() {
        check_at_standard_points("x");
    }
    #[test]
    fn gc_x2() {
        check_at_standard_points("x^2");
    }
    #[test]
    fn gc_x3() {
        check_at_standard_points("x^3");
    }
    #[test]
    fn gc_sin() {
        check_at_standard_points("sin(x)");
    }
    #[test]
    fn gc_cos() {
        check_at_standard_points("cos(x)");
    }
    #[test]
    fn gc_ln() {
        check_at_standard_points("ln(x)");
    } // only pos x will pass

    // ── Composed ─────────────────────────────────────────────────────────────

    #[test]
    fn gc_sin_x2() {
        check_at_standard_points("sin(x^2)");
    }
    #[test]
    fn gc_cos_sin_x() {
        check_at_standard_points("cos(sin(x))");
    }
    #[test]
    fn gc_ln_x2_plus_1() {
        check_at_standard_points("ln(x^2 + 1)");
    }
    #[test]
    fn gc_sin_ln_abs() {
        check_at_standard_points("sin(ln(x^2 + 1))");
    }

    // ── Product ──────────────────────────────────────────────────────────────

    #[test]
    fn gc_x_sin_x() {
        check_at_standard_points("x*sin(x)");
    }
    #[test]
    fn gc_x2_cos_x() {
        check_at_standard_points("x^2*cos(x)");
    }
    #[test]
    fn gc_sin_x_cos_x() {
        check_at_standard_points("sin(x)*cos(x)");
    }

    // ── Quotient ─────────────────────────────────────────────────────────────

    #[test]
    fn gc_sin_over_cos() {
        check_at_standard_points("sin(x)/cos(x)");
    } // tan(x)
    #[test]
    fn gc_x_over_x2_plus1() {
        check_at_standard_points("x / (x^2 + 1)");
    }
    #[test]
    fn gc_1_over_x2() {
        check_at_standard_points("1 / (x^2 + 1)");
    }

    // ── General power ────────────────────────────────────────────────────────

    #[test]
    fn gc_x_to_x() {
        // x^x is only real-valued for x > 0; derivative contains ln(x)
        let xs_pos = [0.5f64, 1.0, 1.5, 2.0, 2.5];
        for &x in &xs_pos {
            assert!(gradient_check("x^x", x), "gc_x_to_x failed at x={}", x);
        }
    }

    #[test]
    fn gc_sin_x_to_cos_x() {
        check_at_standard_points("sin(x^2+1)^cos(x)");
    }

    // --- The example from the README ──────────────────────────────────────────

    #[test]
    fn gc_readme_example() {
        // sin(x^2) / (x + cos(x))  --- avoid x ≈ -cos(x) singularities
        let xs = [0.5_f64, 1.0, 1.5, 2.0, 2.5];
        for &x in &xs {
            assert!(
                gradient_check("sin(x^2) / (x + cos(x))", x),
                "README example gradient check failed at x={}",
                x
            );
        }
    }
}

// ── 3. PROPERTY TESTS ────────────────────────────────────────────────────────
//
// Algebraic invariants that must hold regardless of the specific expression.
// These don't need proptest — a well-chosen fixed suite of varied expressions
// is sufficient and easier to debug than randomized failures.

#[cfg(test)]
mod property_tests {
    use super::*;

    /// All test expressions. Chosen to cover: polynomials, trig, logs,
    /// products, quotients, nested compositions, and power-rule cases.
    const EXPRS: &[&str] = &[
        "x",
        "x^2",
        "x^3 + 2*x",
        "sin(x)",
        "cos(x)",
        "ln(x^2 + 1)",
        "x*sin(x)",
        "x^2*cos(x)",
        "sin(x)*cos(x)",
        "sin(x^2) / (x^2 + 1)",
        "x / (x^2 + 1)",
        "cos(sin(x))",
        "sin(ln(x^2 + 1))",
    ];

    const XS: &[f64] = &[-1.5, -0.5, 0.5, 1.0, 1.5, 2.0];

    // fn eval_at_all(expr: &Expr, xs: &[f64]) -> Vec<Option<f64>> {
    //     xs.iter().map(|&x| eval_at(expr, x)).collect()
    // }

    // ── Property 1: Simplification Idempotency ───────────────────────────────
    //
    // simplify(simplify(e)) ≡ simplify(e)  numerically
    //
    // If this fails, your simplifier is not converging to a fixed point.

    #[test]
    fn simplify_is_idempotent() {
        for &input in EXPRS {
            let expr = parse(input).unwrap();
            let once = expr.simplify();
            let twice = once.simplify();

            for &x in XS {
                let v1 = eval_at(&once, x);
                let v2 = eval_at(&twice, x);
                match (v1, v2) {
                    (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
                        let scale = a.abs().max(1.0);
                        let err = (a - b).abs() / scale;
                        assert!(
                            err < 1e-10,
                            "idempotency failed for '{}' at x={}: once={:.8}, twice={:.8}",
                            input,
                            x,
                            a,
                            b
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Property 2: Simplification Preserves Numerical Value ─────────────────
    //
    // eval(e) ≡ eval(simplify(e))  at all x
    //
    // If this fails, your simplifier is changing the expression's meaning.
    // This is a soundness property — it must never be violated.

    #[test]
    fn simplify_preserves_value() {
        for &input in EXPRS {
            let expr = parse(input).unwrap();
            let simplified = expr.simplify();

            for &x in XS {
                let original = eval_at(&expr, x);
                let simplified_v = eval_at(&simplified, x);

                match (original, simplified_v) {
                    (Some(a), Some(b)) if a.is_finite() && b.is_finite() => {
                        let scale = a.abs().max(1.0);
                        let err = (a - b).abs() / scale;
                        assert!(
                            err < 1e-10,
                            "simplify changed value of '{}' at x={}: before={:.8}, after={:.8}",
                            input,
                            x,
                            a,
                            b
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Property 3: Linearity of Differentiation ─────────────────────────────
    //
    // d/dx[f + g] ≡ d/dx[f] + d/dx[g]  (Additivity)
    // d/dx[c*f]   ≡ c * d/dx[f]         (Homogeneity)
    //
    // These follow directly from the sum rule. If they fail, your sum rule
    // or your eval is broken.

    #[test]
    fn differentiation_is_additive() {
        let pairs = [
            ("sin(x)", "cos(x)"),
            ("x^2", "x^3"),
            ("ln(x^2 + 1)", "x*sin(x)"),
        ];

        let xs = [0.5_f64, 1.0, 1.5, 2.0];

        for (f_str, g_str) in &pairs {
            let f = parse(f_str).unwrap();
            let g = parse(g_str).unwrap();

            // Build f + g by parsing "f_str + g_str"
            let sum_str = format!("({}) + ({})", f_str, g_str);
            let sum = parse(&sum_str).unwrap();

            let df = f.diff("x").simplify();
            let dg = g.diff("x").simplify();
            let dsum = sum.diff("x").simplify();

            for &x in &xs {
                let lhs = eval_at(&dsum, x);
                let rhs = eval_at(&df, x).zip(eval_at(&dg, x)).map(|(a, b)| a + b);

                match (lhs, rhs) {
                    (Some(l), Some(r)) if l.is_finite() && r.is_finite() => {
                        let err = (l - r).abs() / r.abs().max(1.0);
                        assert!(
                            err < 1e-6,
                            "linearity failed: d/dx[{} + {}] at x={}: lhs={:.8}, rhs={:.8}",
                            f_str,
                            g_str,
                            x,
                            l,
                            r
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    #[test]
    fn differentiation_is_homogeneous() {
        // d/dx[3 * f(x)] should equal 3 * d/dx[f(x)]
        let fs = ["sin(x)", "x^2", "x*cos(x)", "ln(x^2 + 1)"];
        let c = 3.0_f64;
        let xs = [0.5_f64, 1.0, 1.5, 2.0];

        for f_str in &fs {
            let scaled_str = format!("3 * ({})", f_str);
            let scaled = parse(&scaled_str).unwrap();
            let f = parse(f_str).unwrap();

            let d_scaled = scaled.diff("x").simplify();
            let c_df = f.diff("x").simplify();

            for &x in &xs {
                let lhs = eval_at(&d_scaled, x);
                let rhs = eval_at(&c_df, x).map(|v| c * v);

                match (lhs, rhs) {
                    (Some(l), Some(r)) if l.is_finite() && r.is_finite() => {
                        let err = (l - r).abs() / r.abs().max(1.0);
                        assert!(
                            err < 1e-6,
                            "homogeneity failed: d/dx[3*{}] at x={}: got {:.8}, expected {:.8}",
                            f_str,
                            x,
                            l,
                            r
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    // ── Property 4: Chain Rule Consistency ───────────────────────────────────
    //
    // For f(g(x)), the derivative should agree with finite-difference.
    // This is already covered by gradient_checks but included here explicitly
    // because chain rule failures are the most common CAS bug.

    #[test]
    fn chain_rule_nested_compositions() {
        let nested = [
            "sin(cos(x))",
            "cos(sin(x))",
            "ln(sin(x^2) + 2)", // +2 to keep argument positive
            "sin(x^2 + cos(x))",
        ];
        for &expr in &nested {
            for &x in XS {
                assert!(
                    gradient_check(expr, x),
                    "chain rule gradient check failed for '{}' at x={}",
                    expr,
                    x
                );
            }
        }
    }

    // ── Property 5: Zero Derivative for Constants ─────────────────────────────

    #[test]
    fn derivative_of_constants_is_zero() {
        let constants = ["1", "0", "3.14", "100", "-7"];
        for &c in &constants {
            let expr = parse(c).unwrap();
            let deriv = expr.diff("x").simplify();
            for &x in XS {
                if let Some(v) = eval_at(&deriv, x) {
                    assert!(
                        v.abs() < 1e-10,
                        "d/dx[{}] should be 0 at x={}, got {}",
                        c,
                        x,
                        v
                    );
                }
            }
        }
    }
}

// ── 4. PROPTEST (randomized property tests) ───────────────────────────────────
//
// Randomly generated expressions tested for gradient correctness and
// simplification soundness. Finds edge cases your hand-written tests miss.
//
// Uses the `proptest` crate (add to [dev-dependencies]).
//
// The expression generator is deliberately small: just enough to produce
// varied but valid inputs. Extend it as USME's supported function set grows.

#[cfg(test)]
mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random, syntactically valid expression string.
    /// Depth-limited to avoid explosion during differentiation.
    fn arb_expr(depth: u32) -> impl Strategy<Value = String> {
        let leaf = prop_oneof![
            Just("x".to_string()),
            (1i32..=5).prop_map(|n| n.to_string()),
        ];

        if depth == 0 {
            return leaf.boxed();
        }

        //let inner = arb_expr(depth - 1);

        prop_oneof![
            Just("x".to_string()),
            (1i32..=5).prop_map(|n| n.to_string()),
            arb_expr(depth - 1).prop_map(|e| format!("sin({})", e)),
            arb_expr(depth - 1).prop_map(|e| format!("cos({})", e)),
            arb_expr(depth - 1).prop_map(|e| format!("ln(({})^2 + 1)", e)),
            (arb_expr(depth - 1), arb_expr(depth - 1))
                .prop_map(|(a, b)| format!("({}) + ({})", a, b)),
            (arb_expr(depth - 1), arb_expr(depth - 1))
                .prop_map(|(a, b)| format!("({}) * ({})", a, b)),
            (arb_expr(depth - 1), (2u32..=3).prop_map(|n| n.to_string()))
                .prop_map(|(b, e)| format!("({})^{}", b, e)),
        ]
        .boxed()
    }

    proptest! {
        /// Gradient check on randomly generated expressions.
        /// Tests that the symbolic derivative numerically agrees with
        /// finite difference at a random evaluation point.
        #[test]
        fn prop_gradient_check(
            expr_str in arb_expr(2),
            x in -2.0f64..2.0f64,
        ) {
            // Skip if x is very close to 0 (potential domain issues)
            prop_assume!(x.abs() > 0.05);

            // Parse must succeed — if it doesn't, our generator is broken
            let expr = parse(&expr_str);
            prop_assume!(expr.is_ok());
            let expr = expr.unwrap();

            // Skip if function is not finite at this point
            let f_val = eval_at(&expr, x);
            prop_assume!(f_val.map(|v| v.is_finite()).unwrap_or(false));

            // The actual check
            prop_assert!(
                gradient_check(&expr_str, x),
                "gradient check failed for '{}' at x={}",
                expr_str, x
            );
        }

        /// Simplification must preserve numerical value.
        /// For any expression, eval before and after simplify must agree.
        #[test]
        fn prop_simplify_preserves_value(
            expr_str in arb_expr(2),
            x in -2.0f64..2.0f64,
        ) {
            prop_assume!(x.abs() > 0.05);

            let expr = parse(&expr_str);
            prop_assume!(expr.is_ok());
            let expr = expr.unwrap();

            let before = eval_at(&expr, x);
            prop_assume!(before.map(|v| v.is_finite()).unwrap_or(false));

            let after = eval_at(&expr.simplify(), x);
            prop_assume!(after.map(|v| v.is_finite()).unwrap_or(false));

            let (b, a) = (before.unwrap(), after.unwrap());
            let err = (b - a).abs() / b.abs().max(1.0);

            prop_assert!(
                err < 1e-9,
                "simplify changed value of '{}' at x={}: before={:.8}, after={:.8}",
                expr_str, x, b, a
            );
        }

        /// Simplification must be idempotent.
        #[test]
        fn prop_simplify_idempotent(
            expr_str in arb_expr(2),
            x in -2.0f64..2.0f64,
        ) {
            prop_assume!(x.abs() > 0.05);

            let expr = parse(&expr_str);
            prop_assume!(expr.is_ok());

            let once  = expr.unwrap().simplify();
            let twice = once.simplify();

            let v1 = eval_at(&once, x);
            let v2 = eval_at(&twice, x);

            prop_assume!(
                v1.map(|v| v.is_finite()).unwrap_or(false) &&
                v2.map(|v| v.is_finite()).unwrap_or(false)
            );

            let (a, b) = (v1.unwrap(), v2.unwrap());
            let err = (a - b).abs() / a.abs().max(1.0);

            prop_assert!(
                err < 1e-10,
                "idempotency failed for '{}' at x={}: once={:.8}, twice={:.8}",
                expr_str, x, a, b
            );
        }
    }
}
