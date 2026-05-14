<div align="center">

# USME
### Universal Symbolic Math Engine

*A lightweight symbolic expression engine written in Rust.*

</div>

---

USME parses mathematical expressions into Abstract Syntax Trees (ASTs), performs symbolic differentiation, recursively simplifies expressions through fixed-point reduction, supports symbolic substitution, and evaluates results numerically.

Rather than attempting to become a full-scale Computer Algebra System (CAS), USME focuses on a smaller and more deliberate scope:

- Symbolic transformation
- Differentiation
- Simplification
- Expression evaluation
- AST-based mathematical representation

The project prioritizes **architectural clarity**, **correctness**, and **educational value** over feature breadth.

---

## Table of Contents

- [Features](#features)
- [Architecture](#architecture)
- [Example](#example)
- [Internal Representation](#internal-representation)
- [Current Limitations](#current-limitations)
- [Design Philosophy](#design-philosophy)
- [Educational Value](#educational-value)
- [Why Rust?](#why-rust)
- [Installation](#installation)
- [Build](#build)
- [Project Status](#project-status)
- [License](#license)

---

## Features

### Recursive Descent Parsing

USME includes a hand-written tokenizer and recursive descent parser capable of transforming human-readable mathematical expressions into structured ASTs.

The parser correctly handles:

- Operator precedence
- Associativity
- Nested expressions
- Unary operations
- Function calls

```text
sin(x^2) / (x + cos(x))
```

is transformed into a recursive symbolic expression tree.

---

### Symbolic Differentiation

USME supports symbolic differentiation using recursive transformation rules.

| Rule | Description |
|------|-------------|
| Sum Rule | $\frac{d}{dx}[f + g] = f' + g'$ |
| Product Rule | $\frac{d}{dx}[fg] = f'g + fg'$ |
| Quotient Rule | $\frac{d}{dx}\left[\frac{f}{g}\right] = \frac{f'g - fg'}{g^2}$ |
| Chain Rule | $\frac{d}{dx}[f(g(x))] = f'(g(x)) \cdot g'(x)$ |
| General Power Rule | See below |

**Supported symbolic functions:** `sin(x)` · `cos(x)` · `ln(x)`

---

### General Power Rule

USME supports symbolic differentiation of arbitrary exponentiation $f(x)^{g(x)}$ using:

$$\frac{d}{dx}\left[f(x)^{g(x)}\right] = f(x)^{g(x)} \left( g'(x)\ln(f(x)) + \frac{g(x)f'(x)}{f(x)} \right)$$

This allows symbolic differentiation of expressions such as:

```text
(x^2 + 1)^(sin(x))
```

which require both logarithmic differentiation and recursive chain rule application.

---

### Fixed-Point Recursive Simplification

USME implements a recursive rewrite-based simplification engine that runs repeatedly until no further reductions are possible.

```text
x * 1        →  x
x + 0        →  x
(x * x) / x  →  x
```

The simplifier operates **bottom-up** across the AST and applies reductions recursively until the expression stabilizes. This fixed-point strategy allows nested redundancies to collapse naturally over multiple passes.

---

### Variable Substitution

USME supports symbolic subtree substitution, enabling:

- Function composition
- Total derivative construction
- Symbolic injection of expressions into larger expressions

```text
f(x) = x^2 + 1
g(x) = sin(x)

f(g(x))  →  sin(x)^2 + 1
```

---

### Numerical Evaluation

Expressions can be evaluated numerically using runtime variable bindings through a `HashMap<String, f64>` context.

```rust
let mut vars = HashMap::new();
vars.insert("x".to_string(), 1.5);

expr.eval(&vars);
```

---

### Operator Overloading

USME implements standard Rust arithmetic traits — `Add`, `Sub`, `Mul`, `Div` — allowing expressions to be composed naturally within Rust code.

---

### Intelligent Formatting

Expressions are rendered back into readable mathematical strings with minimal parentheses. Formatting decisions are made dynamically using precedence-aware traversal to preserve correctness while avoiding unnecessary visual clutter.

---

## Architecture

USME follows a modular symbolic transformation pipeline:

```
Input String
    ↓
Tokenizer
    ↓
Recursive Descent Parser
    ↓
Abstract Syntax Tree (AST)
    ↓
Symbolic Transformations
    ↓
Recursive Simplification
    ↓
Evaluation / Rendering
```

---

## Example

```rust
use usme::parser::parse;
use std::collections::HashMap;

fn main() {
    let input = "sin(x^2) / (x + cos(x))";

    // Parse expression into AST
    let expr = parse(input).unwrap();

    // Differentiate symbolically
    let derivative = expr.diff("x");

    // Simplify derivative
    let simplified = derivative.simplify();

    println!("Expression : {}", expr);
    println!("Derivative : {}", simplified);

    // Numerical evaluation
    let mut vars = HashMap::new();
    vars.insert("x".to_string(), 1.5);

    let result = expr.eval(&vars).unwrap();

    println!("Value at x=1.5 : {}", result);
}
```

**Output:**

```text
Expression : sin(x^2)/(x + cos(x))

Derivative : (2*x*cos(x^2)*(x + cos(x)) - sin(x^2)*(1 - sin(x))) / (x + cos(x))^2
```

---

## Internal Representation

Expressions are represented internally as recursive enums:

```rust
Expr::Binary(
    Box::new(lhs),
    Operator::Mul,
    Box::new(rhs),
)
```

This recursive structure enables symbolic traversal, transformation passes, differentiation, recursive simplification, and precedence-aware formatting — all through Rust's pattern matching system.

---

## Current Limitations

<details>
<summary><b>Click to expand</b></summary>

USME intentionally focuses on symbolic transformation rather than full symbolic algebra. The following capabilities are outside the engine's current scope.

### No Canonicalization

USME does not normalize expressions into a canonical form. `x + y` and `y + x` are treated as structurally different expressions.

### No Like-Term Collection

The simplifier does not combine algebraically equivalent terms.

```text
x + x  →  (not simplified to 2*x)
```

### Expression Explosion

USME does not implement Common Subexpression Elimination (CSE), DAG-based node reuse, or symbolic factoring. Deeply nested symbolic differentiation can produce rapidly growing ASTs.

### No Symbolic Integration

USME is derivative-focused and does not implement integration tables, heuristic symbolic integration, or Risch-style algorithms.

### No Algebraic Expansion or Factoring

The engine cannot expand polynomial products or factor symbolic expressions.

```text
(x + 1)(x + 1)  →  (not expanded to x^2 + 2x + 1)
```

### No Equation Solving

USME is an expression engine, not a symbolic solver. It cannot isolate variables, solve equations, or compute symbolic roots.

### Floating-Point Precision Limits

Numerical evaluation relies on IEEE-754 `f64` arithmetic. Floating-point rounding errors exist; arbitrary precision and exact rational arithmetic are unsupported.

### No Trigonometric Identity Simplification

The simplifier does not recognize transcendental identities such as $\sin^2(x) + \cos^2(x) = 1$.

</details>

---

## Design Philosophy

USME was designed primarily as a symbolic transformation engine, a systems programming exercise, and an exploration of recursive computation. The project intentionally prioritizes:

- Architectural transparency
- Explicit transformation logic
- Recursive AST manipulation
- Readability of implementation

over maximal symbolic capability.

---

## Educational Value

USME touches concepts from:

| Domain | Topics |
|--------|--------|
| Compiler Construction | Tokenization, recursive descent parsing, AST design |
| Symbolic Algebra | Differentiation rules, rewrite systems, fixed-point reduction |
| Systems Programming | Recursive data structures, memory safety, trait-based dispatch |
| Numerical Methods | Expression evaluation, finite-difference approximation |

---

## Why Rust?

Rust maps naturally onto symbolic systems due to algebraic enums, exhaustive pattern matching, recursive data modeling, memory safety, and strong type guarantees.

USME was partially developed to explore how Rust's type system interacts with recursive symbolic computation problems.

---

## Installation

Add to `Cargo.toml`:

```toml
[dependencies]
usme = "0.1.0"
```

---

## Build

```bash
cargo build --release
```

Run the test suite (43 tests across golden, gradient, property, and proptest layers):

```bash
cargo test
```

---

## Project Status

USME is a completed educational and research project. The architecture is intentionally scoped and complete in its current form. Future experimentation may continue, but the core symbolic transformation pipeline is stable.

---

## License

Licensed under either of:

- [Apache License 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

---

## Author

**Austin T J** — Mechatronics Engineering Student

---

## Contributions

Bug reports, discussions, and small improvements are welcome.
