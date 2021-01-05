use crate::common::trim_line_comment;
use crate::ir::{Arithmetic, Command, FunctionCall, MemoryAccess};
use anyhow::{anyhow, ensure, Context, Result};
use std::collections::VecDeque;
use std::io::BufRead;

pub struct Parser();
impl Parser {
    fn parse_line(original_line: String) -> Result<Vec<Command>> {
        let line = trim_line_comment(original_line);
        let mut tokens: VecDeque<_> = line.split(' ').filter(|x| !x.is_empty()).collect();
        let commands = if let Some(command) = tokens.pop_front() {
            match command {
                "add" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Add.into()]
                }
                "sub" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Sub.into()]
                }
                "neg" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Neg.into()]
                }
                "eq" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Eq.into()]
                }
                "gt" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Gt.into()]
                }
                "lt" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Lt.into()]
                }
                "and" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::And.into()]
                }
                "or" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Or.into()]
                }
                "not" => {
                    ensure!(tokens.is_empty());
                    vec![Arithmetic::Not.into()]
                }
                "push" => {
                    ensure!(tokens.len() == 2);
                    let segment = tokens[0].parse()?;
                    let index = tokens[1].parse()?;
                    vec![MemoryAccess::Push { segment, index }.into()]
                }
                "pop" => {
                    ensure!(tokens.len() == 2);
                    let segment = tokens[0].parse()?;
                    let index = tokens[1].parse()?;
                    vec![MemoryAccess::Pop { segment, index }.into()]
                }
                "label" => {
                    ensure!(tokens.len() == 1);
                    todo!()
                }
                "goto" => {
                    ensure!(tokens.len() == 1);
                    todo!()
                }
                "if-goto" => {
                    ensure!(tokens.len() == 1);
                    todo!()
                }
                "function" => {
                    ensure!(tokens.len() == 2);
                    todo!()
                }
                "call" => {
                    ensure!(tokens.len() == 2);
                    todo!()
                }
                "return" => {
                    ensure!(tokens.is_empty());
                    vec![FunctionCall::Return.into()]
                }
                unknown_command => return Err(anyhow!("Unknown command {}", unknown_command)),
            }
        } else {
            vec![]
        };
        return Ok(commands);
    }

    pub fn parse<R: BufRead>(input: R) -> Result<Vec<Command>> {
        let lines = input
            .lines()
            .enumerate()
            .map(|(i, res)| res.map(|line| (i, line)).with_context(|| "IO failure"))
            .collect::<Result<Vec<_>>>()?;
        let parsed = lines
            .into_iter()
            .map(|(i, line)| (i, line.clone(), Self::parse_line(line)))
            .map(|(i, line, res)| {
                res.with_context(|| format!("Failed to parse on L:{} `{}`", i + 1, line))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(parsed.into_iter().flat_map(|x| x).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ir::*;

    #[test]
    fn test() -> Result<()> {
        let ret = Parser::parse(
            r#"
        push constant 10
        push constant 2
        add
        push constant 3
        sub
        "#
            .as_bytes(),
        )?;
        assert_eq!(
            ret,
            vec![
                Command::MemoryAccess(MemoryAccess::Push {
                    segment: Segment::Constant,
                    index: 10
                }),
                Command::MemoryAccess(MemoryAccess::Push {
                    segment: Segment::Constant,
                    index: 2
                }),
                Command::Arithmetic(Arithmetic::Add),
                Command::MemoryAccess(MemoryAccess::Push {
                    segment: Segment::Constant,
                    index: 3
                }),
                Command::Arithmetic(Arithmetic::Sub),
            ]
        );
        Ok(())
    }
}
