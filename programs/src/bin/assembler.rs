use anyhow::{anyhow, Context, Result};

use nand2tetris::assembly::assembler::Assembler;
use nand2tetris::assembly::parser::Parser;
use std::io::{BufReader, Write};

const ASM_EXT: &str = ".asm";
const HACK_EXT: &str = ".hack";

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next()
        .with_context(|| "First arg should be the program name...")?;
    let asm_path = args
        .next()
        .with_context(|| "This program expects an argument but non was given")?;
    if args.next().is_some() {
        return Result::Err(anyhow!("This program expects at most one argument"));
    }

    if !asm_path.ends_with(ASM_EXT) {
        return Result::Err(anyhow!("Input file must be suffixed by {}", ASM_EXT));
    }
    let file_prefix = asm_path.trim_end_matches(ASM_EXT);
    let hack_path = file_prefix.to_owned() + HACK_EXT;

    let asm = std::fs::File::open(&asm_path)
        .map(BufReader::new)
        .with_context(|| format!("Unable to open file {}", asm_path))?;
    let parsed = Parser::parse(asm).with_context(|| format!("Unable to parse {}", asm_path))?;
    let assembled = Assembler::assemble(parsed)?;

    let mut hack = std::fs::File::create(&hack_path)
        .with_context(|| format!("Unable to open file {}", hack_path))?;
    for line in assembled {
        writeln!(hack, "{:016b}", line)?;
    }
    Result::Ok(())
}
