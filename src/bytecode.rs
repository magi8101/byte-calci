//! Bytecode - Instruction set for the virtual machine
//!
//! Format:
//!   - Each instruction is 1 byte opcode
//!   - PUSH instruction followed by 8 bytes for f64 value
//!   - PUSH_ARRAY followed by 8 bytes for count, then count * 8 bytes for values
//!   - All other instructions are single byte
//!
//! Example bytecode for "sin(90) + 2^3":
//!   0x00: PUSH 90.0     (9 bytes: opcode + f64)
//!   0x09: SIN           (1 byte)
//!   0x0A: PUSH 2.0      (9 bytes)
//!   0x13: PUSH 3.0      (9 bytes)
//!   0x1C: POW           (1 byte)
//!   0x1D: ADD           (1 byte)
//!   0x1E: HALT          (1 byte)

use std::fmt;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Stack operations
    Push = 0x01,      // Push constant onto stack (followed by 8 bytes f64)
    Pop = 0x02,       // Pop value from stack
    Dup = 0x03,       // Duplicate top of stack
    PushArray = 0x04, // Push array (followed by u64 count, then count * f64 values)

    // Arithmetic operations
    Add = 0x10,       // Pop two, push sum
    Sub = 0x11,       // Pop two, push difference (second - first)
    Mul = 0x12,       // Pop two, push product
    Div = 0x13,       // Pop two, push quotient (second / first)
    Pow = 0x14,       // Pop two, push power (second ^ first)
    Neg = 0x15,       // Negate top of stack
    Mod = 0x16,       // Pop two, push modulo (second % first)
    Factorial = 0x17, // Pop one, push factorial

    // Trigonometric functions (radians)
    Sin = 0x20,
    Cos = 0x21,
    Tan = 0x22,
    Asin = 0x23,
    Acos = 0x24,
    Atan = 0x25,
    Sinh = 0x26,      // Hyperbolic sine
    Cosh = 0x27,      // Hyperbolic cosine
    Tanh = 0x28,      // Hyperbolic tangent

    // Mathematical functions
    Sqrt = 0x30,
    Log = 0x31,       // log10
    Ln = 0x32,        // natural log
    Abs = 0x33,
    Floor = 0x34,
    Ceil = 0x35,
    Cbrt = 0x36,      // Cube root
    Log2 = 0x37,      // Log base 2
    Exp = 0x38,       // e^x
    Round = 0x39,     // Round to nearest
    Sign = 0x3A,      // Sign function (-1, 0, 1)
    ToRad = 0x3B,     // Degrees to radians
    ToDeg = 0x3C,     // Radians to degrees

    // Array operations
    Sum = 0x40,       // Sum of array
    Avg = 0x41,       // Average of array
    Min = 0x42,       // Minimum of array
    Max = 0x43,       // Maximum of array
    Len = 0x44,       // Length of array

    // Binary functions (2-argument)
    Gcd = 0x50,       // Greatest common divisor
    Lcm = 0x51,       // Least common multiple
    Npr = 0x52,       // Permutations nPr
    Ncr = 0x53,       // Combinations nCr

    // Control
    Halt = 0xFF,
}

impl OpCode {
    pub fn from_byte(byte: u8) -> Option<OpCode> {
        match byte {
            0x01 => Some(OpCode::Push),
            0x02 => Some(OpCode::Pop),
            0x03 => Some(OpCode::Dup),
            0x04 => Some(OpCode::PushArray),
            0x10 => Some(OpCode::Add),
            0x11 => Some(OpCode::Sub),
            0x12 => Some(OpCode::Mul),
            0x13 => Some(OpCode::Div),
            0x14 => Some(OpCode::Pow),
            0x15 => Some(OpCode::Neg),
            0x16 => Some(OpCode::Mod),
            0x17 => Some(OpCode::Factorial),
            0x20 => Some(OpCode::Sin),
            0x21 => Some(OpCode::Cos),
            0x22 => Some(OpCode::Tan),
            0x23 => Some(OpCode::Asin),
            0x24 => Some(OpCode::Acos),
            0x25 => Some(OpCode::Atan),
            0x26 => Some(OpCode::Sinh),
            0x27 => Some(OpCode::Cosh),
            0x28 => Some(OpCode::Tanh),
            0x30 => Some(OpCode::Sqrt),
            0x31 => Some(OpCode::Log),
            0x32 => Some(OpCode::Ln),
            0x33 => Some(OpCode::Abs),
            0x34 => Some(OpCode::Floor),
            0x35 => Some(OpCode::Ceil),
            0x36 => Some(OpCode::Cbrt),
            0x37 => Some(OpCode::Log2),
            0x38 => Some(OpCode::Exp),
            0x39 => Some(OpCode::Round),
            0x3A => Some(OpCode::Sign),
            0x3B => Some(OpCode::ToRad),
            0x3C => Some(OpCode::ToDeg),
            0x40 => Some(OpCode::Sum),
            0x41 => Some(OpCode::Avg),
            0x42 => Some(OpCode::Min),
            0x43 => Some(OpCode::Max),
            0x44 => Some(OpCode::Len),
            0x50 => Some(OpCode::Gcd),
            0x51 => Some(OpCode::Lcm),
            0x52 => Some(OpCode::Npr),
            0x53 => Some(OpCode::Ncr),
            0xFF => Some(OpCode::Halt),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            OpCode::Push => "PUSH",
            OpCode::Pop => "POP",
            OpCode::Dup => "DUP",
            OpCode::PushArray => "PUSH_ARR",
            OpCode::Add => "ADD",
            OpCode::Sub => "SUB",
            OpCode::Mul => "MUL",
            OpCode::Div => "DIV",
            OpCode::Pow => "POW",
            OpCode::Neg => "NEG",
            OpCode::Mod => "MOD",
            OpCode::Factorial => "FACT",
            OpCode::Sin => "SIN",
            OpCode::Cos => "COS",
            OpCode::Tan => "TAN",
            OpCode::Asin => "ASIN",
            OpCode::Acos => "ACOS",
            OpCode::Atan => "ATAN",
            OpCode::Sinh => "SINH",
            OpCode::Cosh => "COSH",
            OpCode::Tanh => "TANH",
            OpCode::Sqrt => "SQRT",
            OpCode::Log => "LOG",
            OpCode::Ln => "LN",
            OpCode::Abs => "ABS",
            OpCode::Floor => "FLOOR",
            OpCode::Ceil => "CEIL",
            OpCode::Cbrt => "CBRT",
            OpCode::Log2 => "LOG2",
            OpCode::Exp => "EXP",
            OpCode::Round => "ROUND",
            OpCode::Sign => "SIGN",
            OpCode::ToRad => "TORAD",
            OpCode::ToDeg => "TODEG",
            OpCode::Sum => "SUM",
            OpCode::Avg => "AVG",
            OpCode::Min => "MIN",
            OpCode::Max => "MAX",
            OpCode::Len => "LEN",
            OpCode::Gcd => "GCD",
            OpCode::Lcm => "LCM",
            OpCode::Npr => "NPR",
            OpCode::Ncr => "NCR",
            OpCode::Halt => "HALT",
        }
    }

    /// Returns true if this opcode is followed by an operand
    pub fn has_operand(&self) -> bool {
        matches!(self, OpCode::Push | OpCode::PushArray)
    }

    /// Size in bytes of instruction including operand (only for fixed-size operands)
    pub fn size(&self) -> usize {
        match self {
            OpCode::Push => 9, // 1 byte opcode + 8 bytes f64
            // PushArray has variable size, returns minimum
            OpCode::PushArray => 9, // 1 byte opcode + 8 bytes count (values follow)
            _ => 1,
        }
    }
}

impl fmt::Display for OpCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Chunk of bytecode with associated data
#[derive(Debug, Clone)]
pub struct Chunk {
    code: Vec<u8>,
    /// Source line numbers for debugging (maps bytecode offset to source line)
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Write a single byte
    pub fn write_byte(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    /// Write an opcode
    pub fn write_op(&mut self, op: OpCode, line: usize) {
        self.write_byte(op as u8, line);
    }

    /// Write a PUSH instruction with f64 constant
    pub fn write_push(&mut self, value: f64, line: usize) {
        self.write_op(OpCode::Push, line);
        let bytes = value.to_le_bytes();
        for byte in bytes {
            self.write_byte(byte, line);
        }
    }

    /// Get the bytecode
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// Get source line for bytecode offset
    pub fn line(&self, offset: usize) -> usize {
        self.lines.get(offset).copied().unwrap_or(0)
    }

    /// Get length of bytecode
    pub fn len(&self) -> usize {
        self.code.len()
    }

    /// Check if chunk is empty
    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    /// Read f64 from bytecode at offset (after PUSH opcode)
    pub fn read_f64(&self, offset: usize) -> f64 {
        let bytes: [u8; 8] = self.code[offset..offset + 8]
            .try_into()
            .expect("Invalid f64 bytes");
        f64::from_le_bytes(bytes)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
