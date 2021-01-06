use std::collections::VecDeque;
use std::io::BufRead;

use anyhow::{bail, ensure, Context, Result};

use crate::jack::token::Token::{self, IntegerConstant, StringConstant};
use crate::jack::token::{Identifier, Keyword, Symbol};

pub struct TokenIterator<R: BufRead> {
    reader: R,
    current_line: VecDeque<char>,
    buffer: String,
    line_no: usize,
    finished: bool,
}

impl<R: BufRead> TokenIterator<R> {
    pub fn from(reader: R) -> Self {
        Self {
            reader,
            current_line: VecDeque::new(),
            buffer: String::new(),
            line_no: 0,
            finished: false,
        }
    }

    fn fill_if_needed(&mut self) -> Result<()> {
        if self.current_line.is_empty() {
            self.reader
                .read_line(&mut self.buffer)
                .with_context(|| format!("Unable to read line {}", self.line_no))?;
            self.current_line.extend(self.buffer.chars());
            self.buffer.clear();
            self.line_no += 1;
        }
        Ok(())
    }

    fn peek_char(&mut self) -> Result<Option<&char>> {
        self.fill_if_needed()?;
        Ok(self.current_line.front())
    }

    fn next_char(&mut self) -> Result<Option<char>> {
        self.fill_if_needed()?;
        Ok(self.current_line.pop_front())
    }

    fn read_while<F>(&mut self, f: F) -> Result<String>
    where
        F: Fn(&char) -> bool,
    {
        let mut res = String::new();
        while let Some(c) = self.peek_char()? {
            if f(c) {
                res.push(self.next_char()?.unwrap());
            } else {
                break;
            }
        }
        return Ok(res);
    }

    fn discard_while<F>(&mut self, f: F) -> Result<()>
    where
        F: Fn(&char) -> bool,
    {
        while let Some(c) = self.peek_char()? {
            if f(c) {
                self.next_char()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn try_read_token(&mut self) -> Result<Option<Token>> {
        if let Some(&c) = self.peek_char()? {
            if c == '/' {
                ensure!(Some('/') == self.next_char()?);
                let is_comment = self
                    .peek_char()?
                    .map_or(false, |&next| next == '/' || next == '*');
                if is_comment {
                    if let Some('/') = self.peek_char()? {
                        self.read_while(|&c| c != '\n')?;
                        self.next_char()?;
                        return Ok(None);
                    } else {
                        ensure!(Some('*') == self.next_char()?);
                        loop {
                            self.discard_while(|&c| c != '*')?;
                            if Some(&'*') == self.peek_char()? {
                                ensure!(Some('*') == self.next_char()?);
                                if self.peek_char()?.map_or(false, |&c| c == '/') {
                                    ensure!(Some('/') == self.next_char()?);
                                    return Ok(None);
                                }
                            } else {
                                bail!("Comment must be closed");
                            }
                        }
                    }
                } else {
                    return Ok(Some(Symbol::Slash.into()));
                }
            } else if c.is_whitespace() {
                ensure!(self.next_char()?.unwrap().is_whitespace());
                return Ok(None);
            } else if c == '"' {
                // should be stringConstant
                ensure!(Some('"') == self.next_char()?);
                let str = self.read_while(|&c| c != '"')?;
                ensure!(Some('"') == self.next_char()?);
                return Ok(Some(StringConstant(str).into()));
            } else if c.is_ascii_digit() {
                // should be integerConstant
                let n_str = self.read_while(|c| c.is_ascii_digit())?;
                let n = n_str
                    .parse()
                    .with_context(|| format!("Failed to parse integer constant `{}`", n_str))?;
                return Ok(IntegerConstant(n).into());
            } else if !c.is_alphabetic() && c != '_' {
                // should be symbol
                let symbol: Symbol = self.next_char()?.unwrap().to_string().parse()?;
                return Ok(Some(symbol.into()));
            } else {
                // identifier or keyword
                let word = self.read_while(|&c| c.is_alphanumeric() || c == '_')?;
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

    fn next_token(&mut self) -> Result<Option<Token>> {
        if self.finished {
            return Ok(None);
        }
        self.finished = true;
        while self.peek_char()?.is_some() {
            let token = self
                .try_read_token()
                .with_context(|| format!("Failed to tokenize at line {}", self.line_no));
            if let ret @ Ok(Some(_)) = token {
                self.finished = false;
                return ret;
            }
        }
        Ok(None)
    }
}

impl<R: BufRead> Iterator for TokenIterator<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(option_token) => option_token.map(Result::Ok),
            Err(e) => Option::Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn test() -> Result<()> {
        let input = r#"
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
        .as_bytes();
        let token_iterator = TokenIterator::from(input);
        let _tokens: Vec<_> = token_iterator.collect();
        Ok(())
    }
}
