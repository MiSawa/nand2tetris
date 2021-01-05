use crate::assembly::{Condition, Dest, Instruction, Operation, UnresolvedInstruction};
use crate::regex;
use anyhow::{anyhow, ensure, Context, Result};
use enumset::EnumSet;
use once_cell;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::io::BufRead;

pub struct Parser();

impl Parser {
    fn check_symbol(s: &str) -> bool {
        let symbol_pattern = regex!("^[a-zA-Z_.$:][0-9a-zA-Z_.$:]*$");
        symbol_pattern.is_match(s)
    }

    fn check_nat(s: &str) -> bool {
        let nat_pattern = regex!("^[0-9]+$");
        nat_pattern.is_match(s)
    }

    fn clean_line(mut line: String) -> String {
        if let Some(i) = line.find("//") {
            line.truncate(i);
            return Self::clean_line(line);
        }
        if line.contains(' ') {
            // This will remove spaces from symbols or whatnot as well,
            // though the specification seems to require this.
            return Self::clean_line(line.replace(' ', ""));
        }
        line
    }

    fn parse_a_instruction(arg: &str) -> Result<UnresolvedInstruction> {
        if Self::check_nat(arg) {
            if arg.len() > "32768".len() {
                return Err(anyhow!("Invalid number `{}`, may be too large", arg));
            }

            let immediate: u16 = arg
                .parse()
                .with_context(|| format!("Failed to parse number `{}`", arg))?;
            if immediate >= (1 << 15) {
                Err(anyhow!(
                    "Invalid number `{}`: exceeds 15 bit width",
                    immediate
                ))
            } else {
                Ok(Instruction::Address { value: immediate }.into())
            }
        } else if Self::check_symbol(arg) {
            Ok(UnresolvedInstruction::Address {
                symbol: arg.to_owned(),
            })
        } else {
            Err(anyhow!("Unable to parse A-instruction @{}", arg))
        }
    }

    fn parse_dest(arg: &str) -> Result<EnumSet<Dest>> {
        let mut ret = EnumSet::empty();
        for c in arg.chars() {
            let loc = match c {
                'A' => Dest::A,
                'M' => Dest::M,
                'D' => Dest::D,
                _ => return Err(anyhow!("Failed to parse dest `{}`", arg)),
            };
            if !ret.insert(loc) {
                return Err(anyhow!(
                    "Destination `{}` is specified multiple times in `{}`",
                    c,
                    arg
                ));
            }
        }
        Ok(ret)
    }

    fn parse_operation(arg: &str) -> Result<EnumSet<Operation>> {
        static TABLE: OnceCell<HashMap<&str, EnumSet<Operation>>> = OnceCell::new();
        #[cfg_attr(rustfmt, rustfmt_skip)]
        let table = TABLE.get_or_init(|| {
            let mut table = HashMap::new();
            table.insert("0",   EnumSet::from_u8(0b101010));
            table.insert("1",   EnumSet::from_u8(0b111111));
            table.insert("-1",  EnumSet::from_u8(0b111010));
            table.insert("D",   EnumSet::from_u8(0b001100));
            table.insert("A",   EnumSet::from_u8(0b110000));
            table.insert("!D",  EnumSet::from_u8(0b001101));
            table.insert("!A",  EnumSet::from_u8(0b110001));
            table.insert("-D",  EnumSet::from_u8(0b001111));
            table.insert("-A",  EnumSet::from_u8(0b110011));
            table.insert("D+1", EnumSet::from_u8(0b011111));
            table.insert("A+1", EnumSet::from_u8(0b110111));
            table.insert("1+D", EnumSet::from_u8(0b011111));
            table.insert("1+A", EnumSet::from_u8(0b110111));
            table.insert("D-1", EnumSet::from_u8(0b001110));
            table.insert("A-1", EnumSet::from_u8(0b110010));
            table.insert("D+A", EnumSet::from_u8(0b000010));
            table.insert("A+D", EnumSet::from_u8(0b000010));
            table.insert("D-A", EnumSet::from_u8(0b010011));
            table.insert("A-D", EnumSet::from_u8(0b000111));
            table.insert("D&A", EnumSet::from_u8(0b000000));
            table.insert("A&D", EnumSet::from_u8(0b000000));
            table.insert("D|A", EnumSet::from_u8(0b010101));
            table.insert("A|D", EnumSet::from_u8(0b010101));
            table
        });
        let non_addressing = arg.replace('M', "A");
        let ret = table
            .get(non_addressing.as_str())
            .with_context(|| format!("Unknown operation `{}`", arg))?;
        let mut ret = (*ret).clone();
        if arg.contains('M') {
            ret.insert(Operation::A);
        }
        Ok(ret)
    }

    fn parse_jump(arg: &str) -> Result<EnumSet<Condition>> {
        ensure!(arg.len() == 3);
        ensure!(arg.starts_with('J'));
        match &arg[1..3] {
            "MP" => return Ok(EnumSet::all()),
            "EQ" => return Ok(EnumSet::only(Condition::EQ)),
            "NE" => return Ok(EnumSet::only(Condition::EQ).complement()),
            _ => {}
        }
        let mut ret = EnumSet::new();
        ret.insert(match arg.bytes().nth(1).unwrap() {
            b'G' => Condition::GT,
            b'L' => Condition::LT,
            _ => return Err(anyhow!("Unable to parse jump `{}`", arg)),
        });
        match arg.bytes().nth(2).unwrap() {
            b'E' => {
                ret.insert(Condition::EQ);
            }
            b'T' => {}
            _ => return Err(anyhow!("Unable to parse jump `{}`", arg)),
        };
        Ok(ret)
    }

    fn parse_c_instruction(line: String) -> Result<Instruction> {
        let c_instruction_pattern =
            regex!("^(?:([AMD]{1,3})=|)([!-01+AMD&|]*)(?:|;(J[GL][TE]|JNE|JEQ|JMP))$");
        if let Some(captures) = c_instruction_pattern.captures(&line) {
            let dest = captures
                .get(1)
                .map(|m| m.as_str())
                .map_or(Ok(EnumSet::empty()), Self::parse_dest)?;
            let operation = captures
                .get(2)
                .with_context(|| format!("Failed to parse comp part of a C-instruction `{}`", line))
                .map(|m| m.as_str())
                .map(Self::parse_operation)??;
            let jump = captures
                .get(3)
                .map(|m| m.as_str())
                .map_or(Ok(EnumSet::empty()), Self::parse_jump)?;
            Ok(Instruction::Compute {
                operation,
                dest,
                jump,
            })
        } else {
            Err(anyhow!("Failed to parse a C-instruction `{}`", line))
        }
    }

    fn parse_line(line: String) -> Result<Option<UnresolvedInstruction>> {
        let line = Self::clean_line(line);
        return if line.is_empty() {
            // Empty line (may be comment)
            Ok(None)
        } else if line.starts_with('(') && line.ends_with(')') {
            // (symbol)
            let symbol = &line[1..(line.len() - 1)];
            if Self::check_symbol(symbol) {
                Ok(Some(UnresolvedInstruction::Label {
                    symbol: symbol.to_owned(),
                }))
            } else {
                Err(anyhow!("Invalid symbol `{}`", symbol))
            }
        } else if line.starts_with('@') {
            Self::parse_a_instruction(&line[1..]).map(Some)
        } else {
            Self::parse_c_instruction(line)
                .map(UnresolvedInstruction::from)
                .map(Some)
        };
    }

    pub fn parse<R: BufRead>(input: R) -> Result<Vec<UnresolvedInstruction>> {
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
