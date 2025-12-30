//! Bytecode Calculator
//!
//! A calculator with a full compilation pipeline:
//!   User Input -> Tokenizer -> Parser -> CodeGenerator -> Bytecode
//!                                                             |
//!                                                         Assembly
//!                                                             |
//!                                                     Virtual Machine
//!                                                             |
//!                                                       Disassembler
//!
//! Example:
//!   Input:    "sin(90) + 2^3"
//!   Bytecode:
//!     0x00: PUSH 90.0
//!     0x09: SIN
//!     0x0A: PUSH 2.0
//!     0x13: PUSH 3.0
//!     0x1C: POW
//!     0x1D: ADD
//!     0x1E: HALT
//!   Result: 9.0

pub mod ast;
pub mod bytecode;
pub mod codegen;
pub mod disassembler;
pub mod gc;
pub mod gui;
pub mod memory;
pub mod parser;
pub mod tokenizer;
pub mod vm;

pub use ast::{BinaryOp, Expr, UnaryOp};
pub use bytecode::{Chunk, OpCode};
pub use codegen::CodeGenerator;
pub use disassembler::Disassembler;
pub use gc::GarbageCollector;
pub use gui::CalculatorApp;
pub use memory::MemoryManager;
pub use parser::Parser;
pub use tokenizer::Tokenizer;
pub use vm::VirtualMachine;

/// Evaluate an expression string and return the result
pub fn evaluate(input: &str) -> Result<f64, String> {
    // Tokenize
    let mut tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    // Compile
    let chunk = CodeGenerator::new().compile(&ast);

    // Execute
    let mut vm = VirtualMachine::new();
    vm.execute(&chunk).map_err(|e| e.to_string())
}

/// Compile and disassemble an expression
pub fn disassemble(input: &str) -> Result<String, String> {
    // Tokenize
    let mut tokenizer = Tokenizer::new(input);
    let tokens = tokenizer.tokenize().map_err(|e| e.to_string())?;

    // Parse
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().map_err(|e| e.to_string())?;

    // Compile
    let chunk = CodeGenerator::new().compile(&ast);

    // Disassemble
    Ok(Disassembler::format_with_hex(&chunk))
}
