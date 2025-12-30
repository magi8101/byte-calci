//! Code Generator - Compiles AST to bytecode
//!
//! Traverses the AST in post-order to generate stack-based bytecode.
//! The generated code follows these conventions:
//!   - Operands are pushed before operations
//!   - Binary ops: left operand pushed first, then right
//!   - Result of each operation remains on stack
//!   - Arrays: elements pushed in order, then PUSH_ARRAY with count

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::bytecode::{Chunk, OpCode};

pub struct CodeGenerator {
    chunk: Chunk,
    current_line: usize,
}

impl CodeGenerator {
    pub fn new() -> Self {
        CodeGenerator {
            chunk: Chunk::new(),
            current_line: 1,
        }
    }

    pub fn compile(mut self, expr: &Expr) -> Chunk {
        self.generate(expr);
        self.chunk.write_op(OpCode::Halt, self.current_line);
        self.chunk
    }

    fn generate(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(value) => {
                self.chunk.write_push(*value, self.current_line);
            }
            Expr::Array(elements) => {
                // Push all elements onto stack
                for element in elements {
                    self.generate(element);
                }
                // Write PUSH_ARRAY with element count
                self.chunk.write_op(OpCode::PushArray, self.current_line);
                let count_bytes = (elements.len() as u64).to_le_bytes();
                for byte in count_bytes {
                    self.chunk.write_byte(byte, self.current_line);
                }
            }
            Expr::UnaryOp { op, operand } => {
                // Generate operand first (post-order)
                self.generate(operand);

                // Then apply operation
                let opcode = match op {
                    UnaryOp::Negate => OpCode::Neg,
                    UnaryOp::Factorial => OpCode::Factorial,
                    UnaryOp::Sin => OpCode::Sin,
                    UnaryOp::Cos => OpCode::Cos,
                    UnaryOp::Tan => OpCode::Tan,
                    UnaryOp::Asin => OpCode::Asin,
                    UnaryOp::Acos => OpCode::Acos,
                    UnaryOp::Atan => OpCode::Atan,
                    UnaryOp::Sinh => OpCode::Sinh,
                    UnaryOp::Cosh => OpCode::Cosh,
                    UnaryOp::Tanh => OpCode::Tanh,
                    UnaryOp::Sqrt => OpCode::Sqrt,
                    UnaryOp::Cbrt => OpCode::Cbrt,
                    UnaryOp::Log => OpCode::Log,
                    UnaryOp::Log2 => OpCode::Log2,
                    UnaryOp::Ln => OpCode::Ln,
                    UnaryOp::Exp => OpCode::Exp,
                    UnaryOp::Abs => OpCode::Abs,
                    UnaryOp::Floor => OpCode::Floor,
                    UnaryOp::Ceil => OpCode::Ceil,
                    UnaryOp::Round => OpCode::Round,
                    UnaryOp::Sign => OpCode::Sign,
                    UnaryOp::ToRad => OpCode::ToRad,
                    UnaryOp::ToDeg => OpCode::ToDeg,
                    UnaryOp::Sum => OpCode::Sum,
                    UnaryOp::Avg => OpCode::Avg,
                    UnaryOp::Min => OpCode::Min,
                    UnaryOp::Max => OpCode::Max,
                    UnaryOp::Len => OpCode::Len,
                };
                self.chunk.write_op(opcode, self.current_line);
            }
            Expr::BinaryOp { op, left, right } => {
                // Generate left operand first
                self.generate(left);
                // Then right operand
                self.generate(right);

                // Apply binary operation
                let opcode = match op {
                    BinaryOp::Add => OpCode::Add,
                    BinaryOp::Subtract => OpCode::Sub,
                    BinaryOp::Multiply => OpCode::Mul,
                    BinaryOp::Divide => OpCode::Div,
                    BinaryOp::Power => OpCode::Pow,
                    BinaryOp::Modulo => OpCode::Mod,
                    BinaryOp::Gcd => OpCode::Gcd,
                    BinaryOp::Lcm => OpCode::Lcm,
                    BinaryOp::Npr => OpCode::Npr,
                    BinaryOp::Ncr => OpCode::Ncr,
                };
                self.chunk.write_op(opcode, self.current_line);
            }
            Expr::PostfixOp { op, operand } => {
                // Generate operand first
                self.generate(operand);
                
                // Apply postfix operation (factorial only for now)
                let opcode = match op {
                    UnaryOp::Factorial => OpCode::Factorial,
                    // Other unary ops shouldn't be used as postfix
                    _ => OpCode::Factorial,
                };
                self.chunk.write_op(opcode, self.current_line);
            }
        }
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::OpCode;

    #[test]
    fn test_compile_number() {
        let expr = Expr::number(42.0);
        let chunk = CodeGenerator::new().compile(&expr);

        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.read_f64(1), 42.0);
        assert_eq!(chunk.code()[9], OpCode::Halt as u8);
    }

    #[test]
    fn test_compile_addition() {
        let expr = Expr::add(Expr::number(1.0), Expr::number(2.0));
        let chunk = CodeGenerator::new().compile(&expr);

        // PUSH 1.0, PUSH 2.0, ADD, HALT
        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.read_f64(1), 1.0);
        assert_eq!(chunk.code()[9], OpCode::Push as u8);
        assert_eq!(chunk.read_f64(10), 2.0);
        assert_eq!(chunk.code()[18], OpCode::Add as u8);
        assert_eq!(chunk.code()[19], OpCode::Halt as u8);
    }

    #[test]
    fn test_compile_sin() {
        let expr = Expr::unary(UnaryOp::Sin, Expr::number(90.0));
        let chunk = CodeGenerator::new().compile(&expr);

        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.read_f64(1), 90.0);
        assert_eq!(chunk.code()[9], OpCode::Sin as u8);
        assert_eq!(chunk.code()[10], OpCode::Halt as u8);
    }

    #[test]
    fn test_compile_array() {
        let expr = Expr::array(vec![
            Expr::number(1.0),
            Expr::number(2.0),
            Expr::number(3.0),
        ]);
        let chunk = CodeGenerator::new().compile(&expr);

        // PUSH 1.0, PUSH 2.0, PUSH 3.0, PUSH_ARRAY 3, HALT
        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.code()[9], OpCode::Push as u8);
        assert_eq!(chunk.code()[18], OpCode::Push as u8);
        assert_eq!(chunk.code()[27], OpCode::PushArray as u8);
        // Count should be 3
        let count_bytes: [u8; 8] = chunk.code()[28..36].try_into().unwrap();
        assert_eq!(u64::from_le_bytes(count_bytes), 3);
    }

    #[test]
    fn test_compile_factorial() {
        let expr = Expr::factorial(Expr::number(5.0));
        let chunk = CodeGenerator::new().compile(&expr);

        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.read_f64(1), 5.0);
        assert_eq!(chunk.code()[9], OpCode::Factorial as u8);
        assert_eq!(chunk.code()[10], OpCode::Halt as u8);
    }

    #[test]
    fn test_compile_modulo() {
        let expr = Expr::modulo(Expr::number(10.0), Expr::number(3.0));
        let chunk = CodeGenerator::new().compile(&expr);

        assert_eq!(chunk.code()[0], OpCode::Push as u8);
        assert_eq!(chunk.code()[9], OpCode::Push as u8);
        assert_eq!(chunk.code()[18], OpCode::Mod as u8);
    }
}
