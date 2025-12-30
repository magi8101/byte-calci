//! Tokenizer - Converts input string into tokens
//! 
//! Input: "sin(90) + 2^3"
//! Output: [Func(Sin), LParen, Num(90), RParen, Op(Add), Num(2), Op(Pow), Num(3)]
//!
//! Extended features:
//!   - Arrays: [1, 2, 3]
//!   - Modulo: 10 % 3
//!   - Factorial: 5!
//!   - More functions: exp, sinh, cosh, tanh, round, sign, min, max, sum, avg, len, gcd, lcm
//!   - Permutations/Combinations: nPr(5,2), nCr(5,2)

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    // Basic operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    Modulo,
    Factorial,
    // Brackets
    LParen,
    RParen,
    LBracket,
    RBracket,
    Comma,
    // Trigonometric functions
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    // Hyperbolic functions
    Sinh,
    Cosh,
    Tanh,
    // Mathematical functions
    Sqrt,
    Cbrt,       // Cube root
    Log,        // log10
    Log2,       // log base 2
    Ln,         // natural log
    Exp,        // e^x
    Abs,
    Floor,
    Ceil,
    Round,
    Sign,
    // Array functions
    Sum,
    Avg,
    Min,
    Max,
    Len,
    // Combinatorics
    Gcd,
    Lcm,
    Npr,        // Permutations
    Ncr,        // Combinations
    // Conversion
    ToRad,      // Degrees to radians
    ToDeg,      // Radians to degrees
    // Constants
    Pi,
    E,
    Tau,        // 2*pi
    Phi,        // Golden ratio
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Power => write!(f, "^"),
            Token::Modulo => write!(f, "%"),
            Token::Factorial => write!(f, "!"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Sin => write!(f, "sin"),
            Token::Cos => write!(f, "cos"),
            Token::Tan => write!(f, "tan"),
            Token::Asin => write!(f, "asin"),
            Token::Acos => write!(f, "acos"),
            Token::Atan => write!(f, "atan"),
            Token::Sinh => write!(f, "sinh"),
            Token::Cosh => write!(f, "cosh"),
            Token::Tanh => write!(f, "tanh"),
            Token::Sqrt => write!(f, "sqrt"),
            Token::Cbrt => write!(f, "cbrt"),
            Token::Log => write!(f, "log"),
            Token::Log2 => write!(f, "log2"),
            Token::Ln => write!(f, "ln"),
            Token::Exp => write!(f, "exp"),
            Token::Abs => write!(f, "abs"),
            Token::Floor => write!(f, "floor"),
            Token::Ceil => write!(f, "ceil"),
            Token::Round => write!(f, "round"),
            Token::Sign => write!(f, "sign"),
            Token::Sum => write!(f, "sum"),
            Token::Avg => write!(f, "avg"),
            Token::Min => write!(f, "min"),
            Token::Max => write!(f, "max"),
            Token::Len => write!(f, "len"),
            Token::Gcd => write!(f, "gcd"),
            Token::Lcm => write!(f, "lcm"),
            Token::Npr => write!(f, "nPr"),
            Token::Ncr => write!(f, "nCr"),
            Token::ToRad => write!(f, "rad"),
            Token::ToDeg => write!(f, "deg"),
            Token::Pi => write!(f, "pi"),
            Token::E => write!(f, "e"),
            Token::Tau => write!(f, "tau"),
            Token::Phi => write!(f, "phi"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenizerError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for TokenizerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tokenizer error at position {}: {}", self.position, self.message)
    }
}

pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Tokenizer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        self.position += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<f64, TokenizerError> {
        let start = self.position;
        let mut has_dot = false;
        let mut has_e = false;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot && !has_e {
                has_dot = true;
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !has_e {
                has_e = true;
                self.advance();
                // Handle optional sign after e
                if let Some(next) = self.peek() {
                    if next == '+' || next == '-' {
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        num_str.parse::<f64>().map_err(|_| TokenizerError {
            message: format!("Invalid number: {}", num_str),
            position: start,
        })
    }

    fn read_identifier(&mut self) -> String {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.position].iter().collect()
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizerError> {
        let mut tokens = Vec::new();

        while self.position < self.input.len() {
            self.skip_whitespace();

            if self.position >= self.input.len() {
                break;
            }

            let ch = self.peek().unwrap();

            let token = if ch.is_ascii_digit() || (ch == '.' && self.input.get(self.position + 1).map_or(false, |c| c.is_ascii_digit())) {
                Token::Number(self.read_number()?)
            } else if ch.is_alphabetic() {
                let ident = self.read_identifier().to_lowercase();
                match ident.as_str() {
                    // Trig functions
                    "sin" => Token::Sin,
                    "cos" => Token::Cos,
                    "tan" => Token::Tan,
                    "asin" | "arcsin" => Token::Asin,
                    "acos" | "arccos" => Token::Acos,
                    "atan" | "arctan" => Token::Atan,
                    // Hyperbolic
                    "sinh" => Token::Sinh,
                    "cosh" => Token::Cosh,
                    "tanh" => Token::Tanh,
                    // Math functions
                    "sqrt" => Token::Sqrt,
                    "cbrt" => Token::Cbrt,
                    "log" | "log10" => Token::Log,
                    "log2" => Token::Log2,
                    "ln" => Token::Ln,
                    "exp" => Token::Exp,
                    "abs" => Token::Abs,
                    "floor" => Token::Floor,
                    "ceil" => Token::Ceil,
                    "round" => Token::Round,
                    "sign" | "sgn" => Token::Sign,
                    // Array functions
                    "sum" => Token::Sum,
                    "avg" | "mean" | "average" => Token::Avg,
                    "min" => Token::Min,
                    "max" => Token::Max,
                    "len" | "length" | "count" => Token::Len,
                    // Combinatorics
                    "gcd" => Token::Gcd,
                    "lcm" => Token::Lcm,
                    "npr" | "perm" => Token::Npr,
                    "ncr" | "comb" | "choose" => Token::Ncr,
                    // Conversion
                    "rad" | "torad" => Token::ToRad,
                    "deg" | "todeg" => Token::ToDeg,
                    // Constants
                    "pi" => Token::Pi,
                    "e" => Token::E,
                    "tau" => Token::Tau,
                    "phi" | "golden" => Token::Phi,
                    _ => return Err(TokenizerError {
                        message: format!("Unknown identifier: {}", ident),
                        position: self.position - ident.len(),
                    }),
                }
            } else {
                self.advance();
                // Check for ** (power operator)
                if ch == '*' && self.peek() == Some('*') {
                    self.advance();
                    Token::Power
                } else {
                    match ch {
                        '+' => Token::Plus,
                        '-' => Token::Minus,
                        '*' | '×' => Token::Multiply,
                        '/' | '÷' => Token::Divide,
                        '^' => Token::Power,
                        '%' => Token::Modulo,
                        '!' => Token::Factorial,
                        '(' => Token::LParen,
                        ')' => Token::RParen,
                        '[' => Token::LBracket,
                        ']' => Token::RBracket,
                        ',' => Token::Comma,
                        'π' => Token::Pi,
                        'τ' => Token::Tau,
                        'φ' => Token::Phi,
                        _ => return Err(TokenizerError {
                            message: format!("Unexpected character: {}", ch),
                            position: self.position - 1,
                        }),
                    }
                }
            };

            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenize() {
        let mut tokenizer = Tokenizer::new("sin(90) + 2^3");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![
            Token::Sin,
            Token::LParen,
            Token::Number(90.0),
            Token::RParen,
            Token::Plus,
            Token::Number(2.0),
            Token::Power,
            Token::Number(3.0),
        ]);
    }

    #[test]
    fn test_array_tokenize() {
        let mut tokenizer = Tokenizer::new("sum([1, 2, 3])");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![
            Token::Sum,
            Token::LParen,
            Token::LBracket,
            Token::Number(1.0),
            Token::Comma,
            Token::Number(2.0),
            Token::Comma,
            Token::Number(3.0),
            Token::RBracket,
            Token::RParen,
        ]);
    }

    #[test]
    fn test_factorial() {
        let mut tokenizer = Tokenizer::new("5!");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(5.0), Token::Factorial]);
    }

    #[test]
    fn test_scientific_notation() {
        let mut tokenizer = Tokenizer::new("1.5e10 + 2E-3");
        let tokens = tokenizer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(1.5e10));
        assert_eq!(tokens[2], Token::Number(2e-3));
    }
}
