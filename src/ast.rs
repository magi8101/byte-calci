//! Abstract Syntax Tree - Parser output
//!
//! Represents the hierarchical structure of expressions
//! Extended with arrays and more operations

use std::fmt;

/// Unary operations (single operand)
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Negate,
    Factorial,
    // Trigonometric
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    // Hyperbolic
    Sinh,
    Cosh,
    Tanh,
    // Mathematical
    Sqrt,
    Cbrt,
    Log,        // log10
    Log2,       // log base 2
    Ln,         // natural log
    Exp,        // e^x
    Abs,
    Floor,
    Ceil,
    Round,
    Sign,
    // Conversion
    ToRad,
    ToDeg,
    // Array operations (take array, return scalar)
    Sum,
    Avg,
    Min,
    Max,
    Len,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Negate => write!(f, "-"),
            UnaryOp::Factorial => write!(f, "!"),
            UnaryOp::Sin => write!(f, "sin"),
            UnaryOp::Cos => write!(f, "cos"),
            UnaryOp::Tan => write!(f, "tan"),
            UnaryOp::Asin => write!(f, "asin"),
            UnaryOp::Acos => write!(f, "acos"),
            UnaryOp::Atan => write!(f, "atan"),
            UnaryOp::Sinh => write!(f, "sinh"),
            UnaryOp::Cosh => write!(f, "cosh"),
            UnaryOp::Tanh => write!(f, "tanh"),
            UnaryOp::Sqrt => write!(f, "sqrt"),
            UnaryOp::Cbrt => write!(f, "cbrt"),
            UnaryOp::Log => write!(f, "log"),
            UnaryOp::Log2 => write!(f, "log2"),
            UnaryOp::Ln => write!(f, "ln"),
            UnaryOp::Exp => write!(f, "exp"),
            UnaryOp::Abs => write!(f, "abs"),
            UnaryOp::Floor => write!(f, "floor"),
            UnaryOp::Ceil => write!(f, "ceil"),
            UnaryOp::Round => write!(f, "round"),
            UnaryOp::Sign => write!(f, "sign"),
            UnaryOp::ToRad => write!(f, "rad"),
            UnaryOp::ToDeg => write!(f, "deg"),
            UnaryOp::Sum => write!(f, "sum"),
            UnaryOp::Avg => write!(f, "avg"),
            UnaryOp::Min => write!(f, "min"),
            UnaryOp::Max => write!(f, "max"),
            UnaryOp::Len => write!(f, "len"),
        }
    }
}

/// Binary operations (two operands)
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    // Combinatorics
    Gcd,
    Lcm,
    Npr,        // Permutations
    Ncr,        // Combinations
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Power => write!(f, "^"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Gcd => write!(f, "gcd"),
            BinaryOp::Lcm => write!(f, "lcm"),
            BinaryOp::Npr => write!(f, "nPr"),
            BinaryOp::Ncr => write!(f, "nCr"),
        }
    }
}

/// Expression tree node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Numeric literal
    Number(f64),
    /// Array literal [1, 2, 3]
    Array(Vec<Expr>),
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// Postfix unary operation (like factorial)
    PostfixOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// Binary operation
    BinaryOp {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

impl Expr {
    pub fn number(value: f64) -> Self {
        Expr::Number(value)
    }

    pub fn array(elements: Vec<Expr>) -> Self {
        Expr::Array(elements)
    }

    pub fn unary(op: UnaryOp, operand: Expr) -> Self {
        Expr::UnaryOp {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn postfix(op: UnaryOp, operand: Expr) -> Self {
        Expr::PostfixOp {
            op,
            operand: Box::new(operand),
        }
    }

    pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Self {
        Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    // Convenience constructors
    pub fn negate(operand: Expr) -> Self {
        Self::unary(UnaryOp::Negate, operand)
    }

    pub fn factorial(operand: Expr) -> Self {
        Self::postfix(UnaryOp::Factorial, operand)
    }

    pub fn add(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Add, left, right)
    }

    pub fn subtract(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Subtract, left, right)
    }

    pub fn multiply(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Multiply, left, right)
    }

    pub fn divide(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Divide, left, right)
    }

    pub fn power(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Power, left, right)
    }

    pub fn modulo(left: Expr, right: Expr) -> Self {
        Self::binary(BinaryOp::Modulo, left, right)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => {
                if n.fract() == 0.0 && n.abs() < 1e10 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Expr::UnaryOp { op, operand } => {
                match op {
                    UnaryOp::Negate => write!(f, "(-{})", operand),
                    _ => write!(f, "{}({})", op, operand),
                }
            }
            Expr::PostfixOp { op, operand } => {
                write!(f, "({}{})", operand, op)
            }
            Expr::BinaryOp { op, left, right } => {
                match op {
                    BinaryOp::Gcd | BinaryOp::Lcm | BinaryOp::Npr | BinaryOp::Ncr => {
                        write!(f, "{}({}, {})", op, left, right)
                    }
                    _ => write!(f, "({} {} {})", left, op, right)
                }
            }
        }
    }
}
