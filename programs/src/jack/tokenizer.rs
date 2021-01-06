use crate::jack::token::Token::{self, IntegerConstant, StringConstant};
use crate::jack::token::{Identifier, Keyword, Symbol};
use anyhow::{anyhow, ensure, Context, Result};
use std::cell::Cell;
use std::io::BufRead;
use std::iter::Peekable;

pub struct Tokenizer {}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {}
    }

    fn take_token<I, F>(it: &mut Peekable<I>, f: F) -> String
    where
        I: Iterator<Item = char>,
        F: Fn(&char) -> bool,
    {
        let mut token = String::new();
        while let Some(c) = it.peek() {
            if f(c) {
                token.push(it.next().unwrap());
            } else {
                break;
            }
        }
        return token;
    }

    fn read_token<I: Iterator<Item = char>>(
        &mut self,
        it: &mut Peekable<I>,
    ) -> Result<Option<Token>> {
        if let Some(&c) = it.peek() {
            if c == '/' {
                ensure!(Some('/') == it.next());
                let is_comment = it.peek().map_or(false, |&next| next == '/' || next == '*');
                if is_comment {
                    if let Some('/') = it.peek() {
                        it.find(|&c| c == '\n');
                        return Ok(None);
                    } else {
                        ensure!(Some('*') == it.next());
                        loop {
                            if let Some(_) = it.find(|&c| c == '*') {
                                if it.peek().map_or(false, |&c| c == '/') {
                                    ensure!(Some('/') == it.next());
                                    return Ok(None);
                                }
                            } else {
                                return Err(anyhow!("Comment must be closed"));
                            }
                        }
                    }
                } else {
                    return Ok(Some(Symbol::Slash.into()));
                }
            } else if c.is_whitespace() {
                it.next();
                return Ok(None);
            } else if c == '"' {
                // should be stringConstant
                ensure!(Some('"') == it.next());
                let str = it.take_while(|&c| c != '"').collect();
                return Ok(Some(StringConstant(str).into()));
            } else if c.is_ascii_digit() {
                // should be integerConstant
                let n_str = Self::take_token(it, |c| c.is_ascii_digit());
                let n = n_str
                    .parse()
                    .with_context(|| format!("Failed to parse integer constant `{}`", n_str))?;
                return Ok(IntegerConstant(n).into());
            } else if !c.is_alphabetic() && c != '_' {
                // should be symbol
                let symbol: Symbol = it.take(1).collect::<String>().parse()?;
                return Ok(Some(symbol.into()));
            } else {
                // identifier or keyword
                let word = Self::take_token(it, |&c| c.is_alphanumeric() || c == '_');
                if let Ok(keyword) = word.parse::<Keyword>() {
                    return Ok(Some(keyword.into()));
                }
                return word
                    .parse::<Identifier>()
                    .with_context(|| {
                        format!(
                            "Word `{}` is not a keyword but can't be an identifier",
                            word
                        )
                    })
                    .map(|identifier| Some(identifier.into()));
            }
        }
        Ok(None)
    }

    pub fn tokenize<R: BufRead>(&mut self, input: R) -> Result<Vec<Token>> {
        let mut ret = Vec::new();
        let lines = input
            .lines()
            .collect::<std::io::Result<Vec<_>>>()
            .with_context(|| format!("Failed to read input"))?;
        let chars = lines
            .into_iter()
            .flat_map(|line| {
                line.chars()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .chain(std::iter::once('\n'))
            })
            .collect::<Vec<_>>();
        let eol_count = Cell::new(0usize);
        let mut it = chars
            .into_iter()
            .inspect(|&c| {
                if c == '\n' {
                    eol_count.replace(eol_count.get() + 1);
                }
            })
            .peekable();
        while it.peek().is_some() {
            let line = eol_count.get() + 1;
            dbg!(it.peek().unwrap());
            let option_token = self
                .read_token(&mut it)
                .with_context(|| format!("Unable to tokenize line {}", line))?;
            if let Some(token) = option_token {
                ret.push(token);
            }
        }
        Ok(ret)
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    #[test]
    fn test() -> Result<()> {
        let mut tokenizer = Tokenizer::new();
        let tokens = tokenizer.tokenize(
            r#"
            class Main {
              // foo
              function main() {
                /* ba
                r */
                do Output.print("Hello, world!");
                do Output.printInt(-15 / 3);
                return;
              }
            }
            "#
            .as_bytes(),
        )?;
        Ok(())
    }
}
