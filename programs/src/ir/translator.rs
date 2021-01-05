use crate::ir::{Arithmetic, Command, MemoryAccess, Segment};
use anyhow::{anyhow, ensure, Result};

pub struct Translator {
    translated_code: Vec<String>,
    next_label_id: usize,
}
impl Translator {
    fn prelude() -> Vec<&'static str> {
        vec!["@256", "D=A", "@SP", "M=D"]
    }
    pub fn new() -> Self {
        let mut ret = Translator {
            translated_code: Vec::new(),
            next_label_id: 0,
        };
        ret.add_assembly(&Self::prelude());
        ret
    }

    fn generate_label(&mut self) -> String {
        let label = format!("L{}", self.next_label_id);
        self.next_label_id += 1;
        label
    }

    fn add_assembly<S: ToString>(&mut self, asm: &[S]) {
        for s in asm {
            self.translated_code.push(s.to_string())
        }
    }

    fn add_comparison_arithmetic(&mut self, condition: &str) {
        let one = self.generate_label();
        let end = self.generate_label();
        self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "D=M-D"]);
        self.add_assembly(&[format!("@{}", one)]);
        self.add_assembly(&[format!("D;J{}", condition)]);
        self.add_assembly(&[format!("@{}", end)]);
        self.add_assembly(&["D=0;JMP"]);
        self.add_assembly(&[format!("({})", one)]);
        self.add_assembly(&["D=-1"]);
        self.add_assembly(&[format!("({})", end)]);
        self.add_assembly(&["@SP", "A=M-1", "M=D"]);
    }

    fn add_arithmetic(&mut self, arithmetic: &Arithmetic) {
        match arithmetic {
            Arithmetic::Add => self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "M=M+D"]),
            Arithmetic::Sub => self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "M=M-D"]),
            Arithmetic::Neg => self.add_assembly(&["@SP", "A=M-1", "M=-M"]),
            Arithmetic::Eq => self.add_comparison_arithmetic("EQ"),
            Arithmetic::Gt => self.add_comparison_arithmetic("GT"),
            Arithmetic::Lt => self.add_comparison_arithmetic("LT"),
            Arithmetic::And => self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "M=M&D"]),
            Arithmetic::Or => self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "M=M|D"]),
            Arithmetic::Not => self.add_assembly(&["@SP", "A=M-1", "M=!M"]),
        }
    }

    fn add_push_const<C: ToString>(&mut self, ptr: C) {
        self.add_assembly(&[format!("@{}", ptr.to_string())]);
        self.add_assembly(&["D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
    }

    fn add_push(&mut self, symbol: &str, index: u16) {
        self.add_assembly(&[format!("@{}", index)]);
        self.add_assembly(&["D=A"]);
        self.add_assembly(&[format!("@{}", symbol)]);
        self.add_assembly(&["A=M+D", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
    }

    fn add_pop_const<C: ToString>(&mut self, ptr: C) {
        self.add_assembly(&["@SP", "AM=M-1", "D=M"]);
        self.add_assembly(&[format!("@{}", ptr.to_string())]);
        self.add_assembly(&["M=D"]);
    }

    fn add_pop(&mut self, symbol: &str, index: u16) {
        self.add_assembly(&[format!("@{}", index)]);
        self.add_assembly(&["D=A"]);
        self.add_assembly(&[format!("@{}", symbol)]);
        self.add_assembly(&["M=M+D"]);

        self.add_assembly(&["@SP", "AM=M-1", "D=M"]);
        self.add_assembly(&[format!("@{}", symbol)]);
        self.add_assembly(&["A=M", "M=D"]);

        self.add_assembly(&[format!("@{}", index)]);
        self.add_assembly(&["D=A"]);
        self.add_assembly(&[format!("@{}", symbol)]);
        self.add_assembly(&["M=M-D"]);
    }

    fn add_memory_access(&mut self, class: &str, memory_access: &MemoryAccess) -> Result<()> {
        match memory_access {
            MemoryAccess::Push { segment, index } => match segment {
                Segment::Argument => self.add_push("ARG", *index),
                Segment::Local => self.add_push("LCL", *index),
                Segment::Static => self.add_push_const(format!("{}.{}", class, index)),
                Segment::Constant => {
                    self.add_assembly(&[format!("@{}", index)]);
                    self.add_assembly(&["D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
                }
                Segment::This => self.add_push("THIS", *index),
                Segment::That => self.add_push("THAT", *index),
                Segment::Pointer => {
                    ensure!(index < &2);
                    self.add_push_const(3 + index);
                }
                Segment::Temp => {
                    ensure!(index < &8);
                    self.add_push_const(5 + index);
                }
            },
            MemoryAccess::Pop { segment, index } => match segment {
                Segment::Argument => self.add_pop("ARG", *index),
                Segment::Local => self.add_pop("LCL", *index),
                Segment::Static => self.add_pop_const(format!("{}.{}", class, index)),
                Segment::Constant => return Err(anyhow!("Unable to pop to {:?}", memory_access)),
                Segment::This => self.add_pop("THIS", *index),
                Segment::That => self.add_pop("THAT", *index),
                Segment::Pointer => {
                    ensure!(index < &2);
                    self.add_pop_const(3 + index);
                }
                Segment::Temp => {
                    ensure!(index < &8);
                    self.add_pop_const(5 + index);
                }
            },
        }
        Ok(())
    }

    fn add_command(&mut self, class: &str, command: &Command) -> Result<()> {
        match command {
            Command::Arithmetic(arithmetic) => self.add_arithmetic(arithmetic),
            Command::MemoryAccess(memory_access) => self.add_memory_access(class, memory_access)?,
            Command::ProgramFlow(_program_flow) => todo!(),
            Command::FunctionCall(_function_call) => todo!(),
        }
        Ok(())
    }

    pub fn add_commands(&mut self, class: &str, commands: &Vec<Command>) -> Result<()> {
        self.add_assembly(&[format!("// -- Class: {} --", class)]);
        for command in commands {
            self.add_assembly(&[format!("// {:?}", command)]);
            self.add_command(class, command)?;
        }
        Ok(())
    }

    pub fn get_assembly(self) -> Vec<String> {
        self.translated_code
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ir::parser::Parser;
    use anyhow::Result;

    #[test]
    fn test() -> Result<()> {
        let commands = Parser::parse(
            r#"
        push constant 10
        push constant 2
        add
        push constant 3
        sub
        "#
            .as_bytes(),
        )?;
        let mut translator = Translator::new();
        translator.add_commands("test", &commands)?;
        let _translated = translator.get_assembly();
        // println!("{:?}", translated);
        Ok(())
    }
}
