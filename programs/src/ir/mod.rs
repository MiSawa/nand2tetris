use crate::regex;
use anyhow::{anyhow, ensure};
use std::str::FromStr;

pub mod parser;
pub mod translator;

type Word = u16;

#[derive(Debug, Eq, PartialEq)]
pub enum Arithmetic {
    // u16 or i16
    Add,
    Sub,
    Neg,
    // Comparison
    Eq,
    Gt,
    Lt,
    // Bitwise operations
    And,
    Or,
    Not,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Segment {
    Argument, // per function invocation
    Local,    // per function invocation, initialized with 0
    Static,   // per vm file
    Constant, // emulated constant, visible from all over the world
    This,
    That,
    Pointer, // base address for this/that segment
    Temp,    // has 8 elements
}

#[derive(Debug, Eq, PartialEq)]
pub enum MemoryAccess {
    Push { segment: Segment, index: Word },
    Pop { segment: Segment, index: Word },
}

#[derive(Debug, Eq, PartialEq)]
pub struct Symbol(String);

#[derive(Debug, Eq, PartialEq)]
pub enum ProgramFlow {
    Label { label: Symbol },
    Goto { label: Symbol },
    IfGoto { label: Symbol },
}

#[derive(Debug, Eq, PartialEq)]
pub enum FunctionCall {
    Declare { name: Symbol, n_locals: Word },
    Invoke { name: Symbol, n_args: Word },
    Return,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Arithmetic(Arithmetic),
    MemoryAccess(MemoryAccess),
    ProgramFlow(ProgramFlow),
    FunctionCall(FunctionCall),
}

impl From<Arithmetic> for Command {
    fn from(value: Arithmetic) -> Self {
        Command::Arithmetic(value)
    }
}
impl From<MemoryAccess> for Command {
    fn from(value: MemoryAccess) -> Self {
        Command::MemoryAccess(value)
    }
}
impl From<ProgramFlow> for Command {
    fn from(value: ProgramFlow) -> Self {
        Command::ProgramFlow(value)
    }
}
impl From<FunctionCall> for Command {
    fn from(value: FunctionCall) -> Self {
        Command::FunctionCall(value)
    }
}

impl FromStr for Symbol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let label_pattern = regex!("^[a-zA-Z_.:][0-9a-zA-Z_.:]*$");
        ensure!(
            label_pattern.is_match(s),
            "String `{}` is invalid as a label",
            s
        );
        Ok(Symbol(s.to_owned()))
    }
}

impl FromStr for Segment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Segment::*;
        let ret = match s {
            "argument" => Argument,
            "local" => Local,
            "static" => Static,
            "constant" => Constant,
            "this" => This,
            "that" => That,
            "pointer" => Pointer,
            "temp" => Temp,
            _ => return Err(anyhow!("Unknown segment {}", s)),
        };
        Ok(ret)
    }
}

impl ToString for Segment {
    fn to_string(&self) -> String {
        match self {
            Segment::Argument => "argument",
            Segment::Local => "local",
            Segment::Static => "static",
            Segment::Constant => "constant",
            Segment::This => "this",
            Segment::That => "that",
            Segment::Pointer => "pointer",
            Segment::Temp => "temp",
        }
        .to_owned()
    }
}

impl ToString for MemoryAccess {
    fn to_string(&self) -> String {
        match self {
            MemoryAccess::Push { segment, index } => {
                format!("push {} {}", segment.to_string(), index)
            }
            MemoryAccess::Pop { segment, index } => {
                format!("pop {} {}", segment.to_string(), index)
            }
        }
    }
}
