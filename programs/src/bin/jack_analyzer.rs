use anyhow::{anyhow, Context, Result};

use nand2tetris::jack::tokenizer::TokenIterator;
use nand2tetris::jack::xml_analyzer::XMLAnalyzer;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

const JACK_EXT: &str = ".jack";
const XML_EXT: &str = ".xml";

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

    let jack_files = if fs::metadata(&input_path)
        .with_context(|| format!("Unable to read {}", input_path))?
        .is_dir()
    {
        fs::read_dir(&input_path)
            .with_context(|| format!("Unable to read {}", input_path))?
            .map(|r| {
                r.with_context(|| format!("Unable to read an entry in {}", input_path))
                    .map(|e| {
                        let jack_name = e
                            .path()
                            .file_name()
                            .map::<&Path, _>(|p| p.as_ref())
                            .and_then(|p| p.to_str())
                            .map_or(false, |p| p.ends_with(JACK_EXT));
                        if jack_name && e.path().is_file() {
                            Some(e.path().to_path_buf())
                        } else {
                            None
                        }
                    })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|x| x)
            .collect::<Vec<PathBuf>>()
    } else {
        if !input_path.ends_with(JACK_EXT) {
            return Result::Err(anyhow!("Input file must be suffixed by {}", JACK_EXT));
        }
        vec![PathBuf::from(&input_path)]
    };

    for jack_file in jack_files {
        let jack = File::open(&jack_file)
            .map(BufReader::new)
            .with_context(|| format!("Unable to open file {}", jack_file.to_string_lossy()))?;
        let token_iterator = TokenIterator::from(jack);
        let mut analyzer = XMLAnalyzer::from(token_iterator);
        let result = analyzer
            .compile()
            .with_context(|| format!("Unable to analyze {}", jack_file.to_string_lossy()))?;
        let output_file = jack_file.with_extension(&XML_EXT[1..]);
        let mut xml = File::create(&output_file)
            .map(BufWriter::new)
            .with_context(|| format!("Unable to open file {}", output_file.to_string_lossy()))?;
        xml.write(result.as_bytes()).with_context(|| {
            format!("Failed to write to file {}", output_file.to_string_lossy())
        })?;
    }
    Result::Ok(())
}
