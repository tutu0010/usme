use std::collections::HashMap;
use usme::parser::parse;

fn main() {
    // 1. Define a complex input string
    let input = "(2 - 4^4) * (((x) * y) ^ x)";

    let x_value = 1.5;
    let y_value = 15.75;

    // 2. Parse the string into an Expression AST
    let expr = parse(input).expect("Failed to parse expression");

    // 3. Calculate the symbolic derivative with respect to 'x'
    // This applies the product, chain, and power rules automatically.
    let raw_diff = expr.diff("x");

    // 4. Simplify the resulting derivative
    let clean_diff = raw_diff.simplify();

    println!("Original:   {}", expr);
    println!("Derivative: {}", clean_diff);

    // 5. Numerical Evaluation
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), x_value);
    vars.insert("y".to_string(), y_value);
    if let Ok(result) = expr.eval(&vars) {
        println!("Value at x={} and y={}: {}", x_value, y_value, result);
    }
}
