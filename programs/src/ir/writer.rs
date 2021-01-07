use crate::ir::{Arithmetic, Command, FunctionCall, MemoryAccess, ProgramFlow};
use anyhow::{Context, Result};
use std::io::Write;
pub struct IRWriter<W: Write> {
    writer: W,
}

impl<W: Write> IRWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
    fn writeln(&mut self, s: &str) -> Result<()> {
        self.writer
            .write(s.as_bytes())
            .with_context(|| format!("Unable to write {}", s))?;
        self.writer
            .write("\n".as_bytes())
            .with_context(|| "Unable to write \\n")?;
        Ok(())
    }
    pub fn comment(&mut self, comment: &str) -> Result<()> {
        self.writeln(&format!("// {}", comment))
    }
    pub fn write_arithmetic(&mut self, arithmetic: &Arithmetic) -> Result<()> {
        self.writeln(&arithmetic.to_string())
    }
    pub fn write_memory_access(&mut self, memory_access: &MemoryAccess) -> Result<()> {
        self.writeln(&memory_access.to_string())
    }
    pub fn write_program_flow(&mut self, program_flow: &ProgramFlow) -> Result<()> {
        self.writeln(&program_flow.to_string())
    }
    pub fn write_function_call(&mut self, function_call: &FunctionCall) -> Result<()> {
        self.writeln(&function_call.to_string())
    }

    pub fn write(&mut self, command: &Command) -> Result<()> {
        match command {
            Command::Arithmetic(arithmetic) => self.write_arithmetic(arithmetic),
            Command::MemoryAccess(memory_access) => self.write_memory_access(memory_access),
            Command::ProgramFlow(program_flow) => self.write_program_flow(program_flow),
            Command::FunctionCall(function_call) => self.write_function_call(function_call),
        }
    }
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().with_context(|| "Failed to flush")
    }
}
