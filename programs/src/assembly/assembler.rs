use crate::assembly::symbol_table::SymbolTable;
use crate::assembly::{Instruction, UnresolvedInstruction};
use anyhow::{anyhow, Result};

pub struct Assembler();
impl Assembler {
    pub fn assemble_resolved(instruction: &Instruction) -> u16 {
        match instruction {
            &Instruction::Address { value } => value & (!(1u16 << 15)),
            &Instruction::Compute {
                operation,
                dest,
                jump,
            } => {
                let mut ret = 0b1110_0000_0000_0000u16;
                ret |= operation.as_u16() << 6;
                ret |= dest.as_u16() << 3;
                ret |= jump.as_u16();
                ret
            }
        }
    }

    pub fn assemble_resolved_all(instructions: &Vec<Instruction>) -> Vec<u16> {
        instructions.iter().map(Self::assemble_resolved).collect()
    }

    pub fn assemble(instructions: Vec<UnresolvedInstruction>) -> Result<Vec<u16>> {
        let mut table = SymbolTable::new();
        let mut jmp_line = 0u16;
        for instruction in &instructions {
            match instruction {
                UnresolvedInstruction::Label { symbol } => {
                    if !table.register(&symbol, jmp_line) {
                        return Err(anyhow!(
                            "Symbol {} is either a reserved symbol or declared more than once",
                            symbol
                        ));
                    }
                }
                _ => {
                    jmp_line += 1;
                }
            }
        }
        let mut resolved_instructions = Vec::with_capacity(instructions.len());
        for instruction in instructions {
            let resolved = match instruction {
                UnresolvedInstruction::Resolved(instruction) => instruction,
                UnresolvedInstruction::Address { symbol } => Instruction::Address {
                    value: table.get_or_auto_register(&symbol),
                },
                UnresolvedInstruction::Label { .. } => {
                    continue;
                }
            };
            resolved_instructions.push(resolved);
        }
        Ok(Self::assemble_resolved_all(&resolved_instructions))
    }
}
