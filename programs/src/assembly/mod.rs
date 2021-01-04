use enumset::EnumSet;

pub mod assembler;
pub mod parser;
mod symbol_table;

pub enum Location {}

#[derive(EnumSetType, Debug)]
pub enum Dest {
    A = 2,
    D = 1,
    M = 0,
}

#[derive(EnumSetType, Debug)]
pub enum Condition {
    LT = 2,
    EQ = 1,
    GT = 0,
}

#[derive(EnumSetType, Debug)]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub enum Operation {
    A  = 6,
    ZY = 5,
    NY = 4,
    ZX = 3,
    NX = 2,
    F  = 1,
    NO = 0,
}

#[derive(Debug)]
pub enum Instruction {
    Address {
        value: u16,
    },
    Compute {
        operation: EnumSet<Operation>,
        dest: EnumSet<Dest>,
        jump: EnumSet<Condition>,
    },
}

#[derive(Debug)]
pub enum UnresolvedInstruction {
    Resolved(Instruction),
    Address { symbol: String },
    Label { symbol: String },
}

impl From<Instruction> for UnresolvedInstruction {
    fn from(v: Instruction) -> Self {
        UnresolvedInstruction::Resolved(v)
    }
}
