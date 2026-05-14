/// Represents a discrete lexical token extracted from a mathematical string.
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// A literal floating point number (e.g., `3.14`).
    Num(f64),
    /// A named variable (e.g., `x`, `theta`).
    Var(String),
    /// Addition operator (`+`).
    Plus,
    /// Subtraction or Negation operator (`-`).
    Minus,
    /// Multiplication operator (`*`).
    Star,
    /// Division operator (`/`).
    Slash,
    /// Exponentiation operator (`^`).
    Caret,
    /// Opening parenthesis `(`.
    LParen,
    /// Closing parenthesis `)`.
    RParen,
    /// The Sine function keyword `sin`.
    Sin,
    /// The Cosine function keyword `cos`.
    Cos,
    /// The Natural Logarithm keyword `ln`.
    Ln,
}

/// Converts a raw mathematical string into a sequential list of tokens.
///
/// This function performs lexical analysis, discarding whitespace and combining
/// characters into numerical constants or multi-character identifiers.
///
/// # Errors
/// Returns an error if the string contains unrecognized characters (like `%` or `$`)
/// or if a number is improperly formatted (like `1.2.3`).
///
/// # Examples
/// ```
/// use usme::parser::tokenizer::{tokenize, Token};
///
/// let tokens = tokenize("x + 1").unwrap();
/// assert_eq!(tokens, vec![
///     Token::Var("x".to_string()),
///     Token::Plus,
///     Token::Num(1.0)
/// ]);
/// ```
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Ignore whitespace entirely
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            // Parse numerical constants, including decimals
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) || c == '.' {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                let n = num_str.parse::<f64>().map_err(|_| "Invalid number")?;
                tokens.push(Token::Num(n));
            }
            // Parse variable names and reserved mathematical functions
            'a'..='z' | 'A'..='Z' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() {
                        ident.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "sin" => tokens.push(Token::Sin),
                    "cos" => tokens.push(Token::Cos),
                    "ln" => tokens.push(Token::Ln),
                    _ => tokens.push(Token::Var(ident)),
                }
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}
