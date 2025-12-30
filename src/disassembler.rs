//! Disassembler - Converts bytecode back to human-readable format
//!
//! Useful for debugging and displaying the compiled bytecode to users.

use crate::bytecode::{Chunk, OpCode};
use std::fmt::Write;

/// Disassembled instruction
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    pub offset: usize,
    pub opcode: OpCode,
    pub operand: Option<f64>,
    pub array_count: Option<u64>,
    pub text: String,
}

/// Disassembler for bytecode chunks
pub struct Disassembler;

impl Disassembler {
    /// Disassemble an entire chunk
    pub fn disassemble(chunk: &Chunk) -> Vec<DisassembledInstruction> {
        let mut instructions = Vec::new();
        let mut offset = 0;

        while offset < chunk.len() {
            if let Some((instruction, new_offset)) = Self::disassemble_instruction(chunk, offset) {
                instructions.push(instruction);
                offset = new_offset;
            } else {
                break;
            }
        }

        instructions
    }

    /// Disassemble a single instruction at the given offset
    pub fn disassemble_instruction(
        chunk: &Chunk,
        offset: usize,
    ) -> Option<(DisassembledInstruction, usize)> {
        if offset >= chunk.len() {
            return None;
        }

        let byte = chunk.code()[offset];
        let opcode = OpCode::from_byte(byte)?;

        let (operand, array_count, text, new_offset) = match opcode {
            OpCode::Push => {
                let value = chunk.read_f64(offset + 1);
                let text = format!("0x{:04X}: {} {}", offset, opcode.name(), value);
                (Some(value), None, text, offset + 9)
            }
            OpCode::PushArray => {
                let count_bytes: [u8; 8] = chunk.code()[offset + 1..offset + 9]
                    .try_into()
                    .expect("Invalid count bytes");
                let count = u64::from_le_bytes(count_bytes);
                let text = format!("0x{:04X}: {} count={}", offset, opcode.name(), count);
                (None, Some(count), text, offset + 9)
            }
            _ => {
                let text = format!("0x{:04X}: {}", offset, opcode.name());
                (None, None, text, offset + 1)
            }
        };

        Some((
            DisassembledInstruction {
                offset,
                opcode,
                operand,
                array_count,
                text,
            },
            new_offset,
        ))
    }

    /// Format disassembly as a string
    pub fn format(chunk: &Chunk) -> String {
        let mut output = String::new();
        let instructions = Self::disassemble(chunk);

        writeln!(output, "=== Bytecode Disassembly ===").unwrap();
        writeln!(output, "Size: {} bytes", chunk.len()).unwrap();
        writeln!(output).unwrap();

        for instr in instructions {
            writeln!(output, "  {}", instr.text).unwrap();
        }

        output
    }

    /// Format disassembly with hex dump
    pub fn format_with_hex(chunk: &Chunk) -> String {
        let mut output = String::new();
        let instructions = Self::disassemble(chunk);

        writeln!(output, "=== Bytecode Disassembly ===").unwrap();
        writeln!(output, "Size: {} bytes", chunk.len()).unwrap();
        writeln!(output).unwrap();
        writeln!(output, "Offset  Hex                      Instruction").unwrap();
        writeln!(output, "------  -----------------------  -----------").unwrap();

        for instr in instructions {
            let size = Self::instruction_size(&instr);
            let hex_bytes = Self::format_hex_bytes(chunk, instr.offset, size);
            writeln!(
                output,
                "0x{:04X}  {:24} {}",
                instr.offset,
                hex_bytes,
                Self::format_instruction(&instr)
            )
            .unwrap();
        }

        output
    }

    /// Get the size of an instruction
    fn instruction_size(instr: &DisassembledInstruction) -> usize {
        match instr.opcode {
            OpCode::Push => 9,
            OpCode::PushArray => 9, // opcode + count
            _ => 1,
        }
    }

    /// Format hex bytes for an instruction
    fn format_hex_bytes(chunk: &Chunk, offset: usize, size: usize) -> String {
        let mut hex = String::new();
        let max_show = 8.min(size);
        for i in 0..max_show {
            if offset + i < chunk.len() {
                write!(hex, "{:02X} ", chunk.code()[offset + i]).unwrap();
            }
        }
        if size > 8 {
            hex.push_str("...");
        }
        hex
    }

    /// Format instruction text
    fn format_instruction(instr: &DisassembledInstruction) -> String {
        match (&instr.operand, &instr.array_count) {
            (Some(value), _) => format!("{} {}", instr.opcode.name(), value),
            (_, Some(count)) => format!("{} count={}", instr.opcode.name(), count),
            _ => instr.opcode.name().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::codegen::CodeGenerator;

    #[test]
    fn test_disassemble_simple() {
        let expr = Expr::add(Expr::number(1.0), Expr::number(2.0));
        let chunk = CodeGenerator::new().compile(&expr);
        let instructions = Disassembler::disassemble(&chunk);

        assert_eq!(instructions.len(), 4); // PUSH, PUSH, ADD, HALT
        assert_eq!(instructions[0].opcode, OpCode::Push);
        assert_eq!(instructions[1].opcode, OpCode::Push);
        assert_eq!(instructions[2].opcode, OpCode::Add);
        assert_eq!(instructions[3].opcode, OpCode::Halt);
    }

    #[test]
    fn test_format_output() {
        let expr = Expr::number(42.0);
        let chunk = CodeGenerator::new().compile(&expr);
        let output = Disassembler::format(&chunk);

        assert!(output.contains("PUSH"));
        assert!(output.contains("42"));
        assert!(output.contains("HALT"));
    }
}
