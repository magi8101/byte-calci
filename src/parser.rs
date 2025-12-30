//! Parser - Converts tokens into AST using recursive descent with Pratt parsing
//!
//! Grammar (Extended):
//!   expression  -> term (('+' | '-') term)*
//!   term        -> factor (('*' | '/' | '%') factor)*
//!   factor      -> base ('^' factor)?          // right associative
//!   base        -> unary | primary
//!   unary       -> ('-' unary) | postfix
//!   postfix     -> function_call ('!')*
//!   function    -> FUNC '(' expression ')' | FUNC '(' expression ',' expression ')'
//!   primary     -> NUMBER | '(' expression ')' | CONSTANT | array
//!   array       -> '[' (expression (',' expression)*)? ']'

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::tokenizer::Token;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        if token.is_some() {
            self.position += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn expect(&mut self, expected: &Token) -> Result<(), ParseError> {
        match self.peek() {
            Some(token) if token == expected => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(ParseError {
                message: format!("Expected {:?}, found {:?}", expected, token),
                position: self.position,
            }),
            None => Err(ParseError {
                message: format!("Expected {:?}, found end of input", expected),
                position: self.position,
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.expression()?;
        if !self.is_at_end() {
            return Err(ParseError {
                message: format!("Unexpected token: {:?}", self.peek()),
                position: self.position,
            });
        }
        Ok(expr)
    }

    // expression -> term (('+' | '-') term)*
    fn expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.term()?;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token::Plus => {
                    self.advance();
                    let right = self.term()?;
                    left = Expr::add(left, right);
                }
                Token::Minus => {
                    self.advance();
                    let right = self.term()?;
                    left = Expr::subtract(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // term -> factor (('*' | '/' | '%') factor)*
    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.factor()?;

        while let Some(token) = self.peek().cloned() {
            match token {
                Token::Multiply => {
                    self.advance();
                    let right = self.factor()?;
                    left = Expr::multiply(left, right);
                }
                Token::Divide => {
                    self.advance();
                    let right = self.factor()?;
                    left = Expr::divide(left, right);
                }
                Token::Modulo => {
                    self.advance();
                    let right = self.factor()?;
                    left = Expr::modulo(left, right);
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // factor -> base ('^' factor)?  (right associative)
    fn factor(&mut self) -> Result<Expr, ParseError> {
        let base = self.unary()?;

        if let Some(Token::Power) = self.peek() {
            self.advance();
            let exponent = self.factor()?;
            return Ok(Expr::power(base, exponent));
        }

        Ok(base)
    }

    // unary -> ('-' unary) | postfix
    fn unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(Token::Minus) = self.peek() {
            self.advance();
            let operand = self.unary()?;
            return Ok(Expr::negate(operand));
        }

        self.postfix()
    }

    // postfix -> function_call ('!')*
    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.function_call()?;

        // Handle postfix factorial
        while let Some(Token::Factorial) = self.peek() {
            self.advance();
            expr = Expr::factorial(expr);
        }

        Ok(expr)
    }

    // function_call -> FUNC '(' args ')' | primary
    fn function_call(&mut self) -> Result<Expr, ParseError> {
        let token = match self.peek().cloned() {
            Some(t) => t,
            None => {
                return Err(ParseError {
                    message: "Unexpected end of input".to_string(),
                    position: self.position,
                })
            }
        };

        // Unary functions
        let unary_op = match &token {
            Token::Sin => Some(UnaryOp::Sin),
            Token::Cos => Some(UnaryOp::Cos),
            Token::Tan => Some(UnaryOp::Tan),
            Token::Asin => Some(UnaryOp::Asin),
            Token::Acos => Some(UnaryOp::Acos),
            Token::Atan => Some(UnaryOp::Atan),
            Token::Sinh => Some(UnaryOp::Sinh),
            Token::Cosh => Some(UnaryOp::Cosh),
            Token::Tanh => Some(UnaryOp::Tanh),
            Token::Sqrt => Some(UnaryOp::Sqrt),
            Token::Cbrt => Some(UnaryOp::Cbrt),
            Token::Log => Some(UnaryOp::Log),
            Token::Log2 => Some(UnaryOp::Log2),
            Token::Ln => Some(UnaryOp::Ln),
            Token::Exp => Some(UnaryOp::Exp),
            Token::Abs => Some(UnaryOp::Abs),
            Token::Floor => Some(UnaryOp::Floor),
            Token::Ceil => Some(UnaryOp::Ceil),
            Token::Round => Some(UnaryOp::Round),
            Token::Sign => Some(UnaryOp::Sign),
            Token::ToRad => Some(UnaryOp::ToRad),
            Token::ToDeg => Some(UnaryOp::ToDeg),
            Token::Sum => Some(UnaryOp::Sum),
            Token::Avg => Some(UnaryOp::Avg),
            Token::Min => Some(UnaryOp::Min),
            Token::Max => Some(UnaryOp::Max),
            Token::Len => Some(UnaryOp::Len),
            _ => None,
        };

        if let Some(op) = unary_op {
            self.advance();
            self.expect(&Token::LParen)?;
            let arg = self.expression()?;
            self.expect(&Token::RParen)?;
            return Ok(Expr::unary(op, arg));
        }

        // Binary functions (gcd, lcm, nPr, nCr)
        let binary_op = match &token {
            Token::Gcd => Some(BinaryOp::Gcd),
            Token::Lcm => Some(BinaryOp::Lcm),
            Token::Npr => Some(BinaryOp::Npr),
            Token::Ncr => Some(BinaryOp::Ncr),
            _ => None,
        };

        if let Some(op) = binary_op {
            self.advance();
            self.expect(&Token::LParen)?;
            let arg1 = self.expression()?;
            self.expect(&Token::Comma)?;
            let arg2 = self.expression()?;
            self.expect(&Token::RParen)?;
            return Ok(Expr::binary(op, arg1, arg2));
        }

        self.primary()
    }

    // primary -> NUMBER | '(' expression ')' | CONSTANT | array
    fn primary(&mut self) -> Result<Expr, ParseError> {
        let token = match self.peek().cloned() {
            Some(t) => t,
            None => {
                return Err(ParseError {
                    message: "Unexpected end of input".to_string(),
                    position: self.position,
                })
            }
        };

        match token {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::number(n))
            }
            Token::Pi => {
                self.advance();
                Ok(Expr::number(std::f64::consts::PI))
            }
            Token::E => {
                self.advance();
                Ok(Expr::number(std::f64::consts::E))
            }
            Token::Tau => {
                self.advance();
                Ok(Expr::number(std::f64::consts::TAU))
            }
            Token::Phi => {
                self.advance();
                // Golden ratio: (1 + sqrt(5)) / 2
                Ok(Expr::number(1.618033988749895))
            }
            Token::LParen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::LBracket => {
                self.parse_array()
            }
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?}", token),
                position: self.position,
            }),
        }
    }

    // array -> '[' (expression (',' expression)*)? ']'
    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Token::LBracket)?;
        
        let mut elements = Vec::new();

        // Check for empty array
        if let Some(Token::RBracket) = self.peek() {
            self.advance();
            return Ok(Expr::array(elements));
        }

        // Parse first element
        elements.push(self.expression()?);

        // Parse remaining elements
        while let Some(Token::Comma) = self.peek() {
            self.advance();
            elements.push(self.expression()?);
        }

        self.expect(&Token::RBracket)?;
        Ok(Expr::array(elements))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::Tokenizer;

    fn parse(input: &str) -> Result<Expr, ParseError> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize().expect("Tokenization failed");
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_simple_number() {
        let expr = parse("42").unwrap();
        assert_eq!(expr, Expr::number(42.0));
    }

    #[test]
    fn test_addition() {
        let expr = parse("1 + 2").unwrap();
        assert_eq!(expr, Expr::add(Expr::number(1.0), Expr::number(2.0)));
    }

    #[test]
    fn test_precedence() {
        let expr = parse("1 + 2 * 3").unwrap();
        assert_eq!(
            expr,
            Expr::add(
                Expr::number(1.0),
                Expr::multiply(Expr::number(2.0), Expr::number(3.0))
            )
        );
    }

    #[test]
    fn test_function() {
        let expr = parse("sin(90)").unwrap();
        assert_eq!(expr, Expr::unary(UnaryOp::Sin, Expr::number(90.0)));
    }

    #[test]
    fn test_power_right_associative() {
        let expr = parse("2^3^2").unwrap();
        assert_eq!(
            expr,
            Expr::power(
                Expr::number(2.0),
                Expr::power(Expr::number(3.0), Expr::number(2.0))
            )
        );
    }

    #[test]
    fn test_factorial() {
        let expr = parse("5!").unwrap();
        assert_eq!(expr, Expr::factorial(Expr::number(5.0)));
    }

    #[test]
    fn test_array() {
        let expr = parse("[1, 2, 3]").unwrap();
        assert_eq!(
            expr,
            Expr::array(vec![
                Expr::number(1.0),
                Expr::number(2.0),
                Expr::number(3.0),
            ])
        );
    }

    #[test]
    fn test_sum_array() {
        let expr = parse("sum([1, 2, 3])").unwrap();
        assert_eq!(
            expr,
            Expr::unary(
                UnaryOp::Sum,
                Expr::array(vec![
                    Expr::number(1.0),
                    Expr::number(2.0),
                    Expr::number(3.0),
                ])
            )
        );
    }

    #[test]
    fn test_gcd() {
        let expr = parse("gcd(12, 8)").unwrap();
        assert_eq!(
            expr,
            Expr::binary(BinaryOp::Gcd, Expr::number(12.0), Expr::number(8.0))
        );
    }

    #[test]
    fn test_modulo() {
        let expr = parse("10 % 3").unwrap();
        assert_eq!(
            expr,
            Expr::modulo(Expr::number(10.0), Expr::number(3.0))
        );
    }
}
