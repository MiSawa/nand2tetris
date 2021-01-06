use anyhow::{anyhow, Context, Result};

use nand2tetris::ir::parser::Parser;
use nand2tetris::ir::translator::Translator;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const ASM_EXT: &str = ".asm";
const VM_EXT: &str = ".vm";

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next()
        .with_context(|| "First arg should be the program name...")?;
    let input_path = args
        .next()
        .with_context(|| "This program expects an argument but non was given")?;
    if args.next().is_some() {
        return Result::Err(anyhow!("This program expects at most one argument"));
    }

    let (output_path, vm_files) = if fs::metadata(&input_path)
        .with_context(|| format!("Unable to read {}", input_path))?
        .is_dir()
    {
        let files = fs::read_dir(&input_path)
            .with_context(|| format!("Unable to read {}", input_path))?
            .map(|r| {
                r.with_context(|| format!("Unable to read an entry in {}", input_path))
                    .map(|e| {
                        let vm_name = e
                            .path()
                            .file_name()
                            .map::<&Path, _>(|p| p.as_ref())
                            .and_then(|p| p.to_str())
                            .map_or(false, |p| p.ends_with(VM_EXT));
                        if vm_name && e.path().is_file() {
                            Some(e.path().to_path_buf())
                        } else {
                            None
                        }
                    })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|x| x)
            .collect::<Vec<PathBuf>>();
        let mut output_path = PathBuf::from(input_path);
        let dirname = output_path
            .file_name()
            .with_context(|| {
                format!(
                    "Unable to get file name for {}",
                    output_path.to_string_lossy()
                )
            })?
            .to_owned();
        output_path.push(dirname);
        output_path.set_extension(&ASM_EXT[1..]);
        (output_path, files)
    } else {
        if !input_path.ends_with(VM_EXT) {
            return Result::Err(anyhow!("Input file must be suffixed by {}", VM_EXT));
        }
        let output_path = PathBuf::from(&input_path).with_extension(&ASM_EXT[1..]);
        (output_path, vec![PathBuf::from(&input_path)])
    };

    let mut translator = Translator::new();
    for path in &vm_files {
        let vm = File::open(path)
            .map(BufReader::new)
            .with_context(|| format!("Unable to open file {}", path.to_string_lossy()))?;
        let parsed = Parser::parse(vm)?;
        let class = path
            .file_name()
            .with_context(|| anyhow!("Unable to get file name from {}", path.to_string_lossy()))?
            .to_str()
            .with_context(|| anyhow!("Unable to stringify file name {}", path.to_string_lossy()))?
            .trim_end_matches(VM_EXT);
        translator.add_commands(class, &parsed)?;
    }
    let ret = translator.get_assembly();
    let mut output_file = File::create(&output_path)
        .with_context(|| format!("Unable to open file {}", output_path.to_string_lossy()))
        .map(BufWriter::new)?;
    for op in ret {
        writeln!(output_file, "{}", op)?;
    }
    Result::Ok(())
}
