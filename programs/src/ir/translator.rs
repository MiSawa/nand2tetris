use crate::ir::{Arithmetic, Command, MemoryAccess, Segment};

pub struct Translator {
    translated_code: Vec<String>,
    next_label_id: usize,
}
impl Translator {
    fn prelude() -> Vec<&'static str> {
        vec![
            "@256",
            "D=A",
            "@SP",
            "M=D",
        ]
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

    fn add_memory_access(&mut self, memory_access: &MemoryAccess) {
        match memory_access {
            MemoryAccess::Push { segment, index } => match segment {
                Segment::Argument => todo!(),
                Segment::Local => todo!(),
                Segment::Static => todo!(),
                Segment::Constant => {
                    self.add_assembly(&[format!("@{}", index)]);
                    self.add_assembly(&["D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
                }
                Segment::This => todo!(),
                Segment::That => todo!(),
                Segment::Pointer => todo!(),
                Segment::Temp => todo!(),
            },
            MemoryAccess::Pop { segment, index } => match segment {
                Segment::Argument => todo!(),
                Segment::Local => todo!(),
                Segment::Static => todo!(),
                Segment::Constant => todo!(),
                Segment::This => todo!(),
                Segment::That => todo!(),
                Segment::Pointer => todo!(),
                Segment::Temp => todo!(),
            },
        }
    }

    fn add_command(&mut self, command: &Command) {
        match command {
            Command::Arithmetic(arithmetic) => self.add_arithmetic(arithmetic),
            Command::MemoryAccess(memory_access) => self.add_memory_access(memory_access),
            Command::ProgramFlow(_program_flow) => todo!(),
            Command::FunctionCall(_function_call) => todo!(),
        }
    }

    pub fn add_commands(&mut self, commands: &Vec<Command>) {
        for command in commands {
            self.add_command(command);
        }
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
        let translated = Translator::add_commands(&commands);
        println!("{:?}", translated);
        Ok(())
    }
}
