use crate::ir::{Arithmetic, Command, FunctionCall, MemoryAccess, ProgramFlow, Segment, Symbol};
use anyhow::{anyhow, ensure, Context, Result};
use std::fmt::Display;

#[derive(Debug)]
struct TranslationContext {
    class: Symbol,
    function: Option<Symbol>,
}

pub struct Translator {
    translated_code: Vec<String>,
    next_label_id: usize,
}

impl Translator {
    pub fn new() -> Self {
        let mut ret = Translator {
            translated_code: Vec::new(),
            next_label_id: 0,
        };
        ret.add_comment("Setup SP");
        ret.add_assembly(&["@256", "D=A", "@SP", "M=D"]);
        let mut context = TranslationContext {
            class: Symbol("$ENTRY_POINT$".to_owned()),
            function: Some(Symbol("$ENTRY_POINT$".to_owned())),
        };
        ret.add_comment("Call Sys.init");
        ret.add_function_call(
            &mut context,
            &FunctionCall::Invoke {
                name: Symbol("Sys.init".to_owned()),
                n_args: 0,
            },
        )
        .unwrap();
        ret
    }

    fn generate_label(&mut self) -> String {
        let label = format!("$$L{}", self.next_label_id);
        self.next_label_id += 1;
        label
    }

    fn add_assembly<S: ToString>(&mut self, asm: &[S]) {
        for s in asm {
            self.translated_code.push(s.to_string())
        }
    }

    fn add_comment<S: Display>(&mut self, s: S) {
        self.add_assembly(&[format!("// {}", s)]);
    }

    fn add_label<S: Display>(&mut self, label: &S) {
        self.add_assembly(&[format!("({})", label)]);
    }

    fn add_comparison_arithmetic(&mut self, condition: &str) {
        let one = self.generate_label();
        let end = self.generate_label();
        self.add_assembly(&["@SP", "AM=M-1", "D=M", "A=A-1", "D=M-D"]);
        self.add_assembly(&[format!("@{}", one)]);
        self.add_assembly(&[format!("D;J{}", condition)]);
        self.add_assembly(&[format!("@{}", end)]);
        self.add_assembly(&["D=0;JMP"]);
        self.add_label(&one);
        self.add_assembly(&["D=-1"]);
        self.add_label(&end);
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

    fn add_push_const<C: ToString>(&mut self, c: C) {
        self.add_assembly(&[format!("@{}", c.to_string())]);
        self.add_assembly(&["D=A", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
    }

    fn add_push_const_ref<C: ToString>(&mut self, ptr: C) {
        self.add_assembly(&[format!("@{}", ptr.to_string())]);
        self.add_assembly(&["D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
    }

    fn add_push(&mut self, symbol: &str, index: u16) {
        self.add_assembly(&[format!("@{}", index)]);
        self.add_assembly(&["D=A"]);
        self.add_assembly(&[format!("@{}", symbol)]);
        self.add_assembly(&["A=M+D", "D=M", "@SP", "A=M", "M=D", "@SP", "M=M+1"]);
    }

    fn add_pop_const_ref<C: ToString>(&mut self, ptr: C) {
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

    fn add_memory_access(
        &mut self,
        context: &TranslationContext,
        memory_access: &MemoryAccess,
    ) -> Result<()> {
        match memory_access {
            MemoryAccess::Push { segment, index } => match segment {
                Segment::Argument => self.add_push("ARG", *index),
                Segment::Local => self.add_push("LCL", *index),
                Segment::Static => {
                    self.add_push_const_ref(format!("{}.{}", context.class.0, index))
                }
                Segment::Constant => self.add_push_const(index),
                Segment::This => self.add_push("THIS", *index),
                Segment::That => self.add_push("THAT", *index),
                Segment::Pointer => {
                    ensure!(index < &2);
                    self.add_push_const_ref(3 + index);
                }
                Segment::Temp => {
                    ensure!(index < &8);
                    self.add_push_const_ref(5 + index);
                }
            },
            MemoryAccess::Pop { segment, index } => match segment {
                Segment::Argument => self.add_pop("ARG", *index),
                Segment::Local => self.add_pop("LCL", *index),
                Segment::Static => self.add_pop_const_ref(format!("{}.{}", context.class.0, index)),
                Segment::Constant => return Err(anyhow!("Unable to pop to {:?}", memory_access)),
                Segment::This => self.add_pop("THIS", *index),
                Segment::That => self.add_pop("THAT", *index),
                Segment::Pointer => {
                    ensure!(index < &2);
                    self.add_pop_const_ref(3 + index);
                }
                Segment::Temp => {
                    ensure!(index < &8);
                    self.add_pop_const_ref(5 + index);
                }
            },
        }
        Ok(())
    }

    fn add_program_flow(
        &mut self,
        context: &TranslationContext,
        program_flow: &ProgramFlow,
    ) -> Result<()> {
        let function = &context
            .function
            .as_ref()
            .with_context(|| "Expected a function")?
            .0;
        match program_flow {
            ProgramFlow::Label { label } => {
                self.add_label(&format!("{}${}", function, &label.0));
            }
            ProgramFlow::Goto { label } => {
                self.add_assembly(&[format!("@{}${}", function, &label.0)]);
                self.add_assembly(&["0;JMP"]);
            }
            ProgramFlow::IfGoto { label } => {
                self.add_assembly(&["@SP", "AM=M-1", "D=M"]);
                self.add_assembly(&[format!("@{}${}", function, &label.0)]);
                self.add_assembly(&["D;JNE"]);
            }
        }
        Ok(())
    }

    fn add_function_call(
        &mut self,
        context: &mut TranslationContext,
        function_call: &FunctionCall,
    ) -> Result<()> {
        match function_call {
            FunctionCall::Declare { name, n_locals } => {
                self.add_label(&name.0);
                self.add_comment("Push locals");
                for _ in 0..*n_locals {
                    self.add_push_const(0);
                }
                context.function = Some(name.clone());
                self.add_comment("Function body");
            }
            FunctionCall::Invoke { name, n_args } => {
                let return_address = self.generate_label();
                self.add_comment("  Push return address");
                self.add_push_const(&return_address);
                self.add_comment("  Push LCL");
                self.add_push_const_ref("LCL");
                self.add_comment("  Push ARG");
                self.add_push_const_ref("ARG");
                self.add_comment("  Push THIS");
                self.add_push_const_ref("THIS");
                self.add_comment("  Push THAT");
                self.add_push_const_ref("THAT");

                // ARG = SP - n - 5
                self.add_comment("  Set ARG = SP - n - 5");
                self.add_assembly(&["@SP", "D=M"]);
                self.add_assembly(&[format!("@{}", n_args)]);
                self.add_assembly(&["D=D-A", "@5", "D=D-A", "@ARG", "M=D"]);

                // LCL = SP
                self.add_comment("  Set LCL = SP");
                self.add_assembly(&["@SP", "D=M", "@LCL", "M=D"]);

                // GOTO func
                self.add_comment(format!("  Goto func {}", name.0));
                self.add_assembly(&[format!("@{}", name.0)]);
                self.add_assembly(&["0;JMP"]);
                self.add_label(&return_address);
            }
            FunctionCall::Return => {
                // Result (R15) = pop()
                self.add_comment("  Pop result to R15");
                self.add_pop_const_ref("R15");
                // The previous SP (R14) = ARG
                self.add_comment("  Temporarily store the previous SP to R14");
                self.add_assembly(&["@ARG", "D=M", "@R14", "M=D"]);

                // SP = LCL
                self.add_comment("  Set SP to LCL");
                self.add_assembly(&["@LCL", "D=M", "@SP", "M=D"]);

                self.add_comment("  Pop THAT");
                self.add_pop_const_ref("THAT");
                self.add_comment("  Pop THIS");
                self.add_pop_const_ref("THIS");
                self.add_comment("  Pop ARG");
                self.add_pop_const_ref("ARG");
                self.add_comment("  Pop LCL");
                self.add_pop_const_ref("LCL");

                // Return address (R13) = pop()
                self.add_comment("  Pop return address to R13");
                self.add_pop_const_ref("R13");
                // Pop args
                self.add_comment("  Set SP to ARG");
                self.add_assembly(&["@R14", "D=M", "@SP", "M=D"]);
                // Push result
                self.add_comment("  Push result again");
                self.add_push_const_ref("R15");
                // GOTO ret
                self.add_comment("  Goto return address");
                self.add_assembly(&["@R13", "A=M", "0;JMP"]);
            }
        }
        Ok(())
    }

    fn add_command(&mut self, context: &mut TranslationContext, command: &Command) -> Result<()> {
        match command {
            Command::Arithmetic(arithmetic) => self.add_arithmetic(arithmetic),
            Command::MemoryAccess(memory_access) => {
                self.add_memory_access(context, memory_access)?
            }
            Command::ProgramFlow(program_flow) => self.add_program_flow(context, program_flow)?,
            Command::FunctionCall(function_call) => {
                self.add_function_call(context, function_call)?
            }
        }
        Ok(())
    }

    pub fn add_commands(&mut self, class: &str, commands: &Vec<Command>) -> Result<()> {
        let class = class
            .parse()
            .with_context(|| format!("Class name `{}` is invalid", class))?;
        let mut context = TranslationContext {
            class,
            function: None,
        };
        self.add_comment(format!("-- Class: {} --", context.class.0));
        for command in commands {
            self.add_comment(format!("{:?}", command));
            self.add_command(&mut context, command).with_context(|| {
                format!(
                    "Unable to translate: Command: {:?}, Context: {:?}",
                    command, context
                )
            })?;
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
