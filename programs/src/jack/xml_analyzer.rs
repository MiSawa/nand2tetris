use crate::jack::token::{Identifier, Keyword, Symbol, Token};
use anyhow::{bail, ensure, Context, Result};
use std::collections::VecDeque;
use std::fmt::Display;
use std::iter::Fuse;

pub struct XMLAnalyzer<I: Iterator<Item = Result<Token>>> {
    token_stream: Fuse<I>,
    peeked: VecDeque<Result<Token>>,
    result_xml: String,
}

impl<I: Iterator<Item = Result<Token>>> XMLAnalyzer<I> {
    pub fn from(token_stream: I) -> Self {
        return Self {
            token_stream: token_stream.fuse(),
            peeked: VecDeque::new(),
            result_xml: String::new(),
        };
    }

    fn write_open<S: Display>(&mut self, tag: S) {
        self.result_xml.push_str(&format!("<{}>\n", tag));
    }
    fn write_close<S: Display>(&mut self, tag: S) {
        self.result_xml.push_str(&format!("</{}>\n", tag));
    }
    fn write_content_with_body<S: Display, T: Display>(&mut self, tag: S, body: T) {
        if body.to_string() == "&" {
            self.write_content_with_body(tag, "&amp;");
        } else if body.to_string() == "<" {
            self.write_content_with_body(tag, "&lt;");
        } else if body.to_string() == ">" {
            self.write_content_with_body(tag, "&gt;");
        } else {
            self.result_xml
                .push_str(&format!("<{}> {} </{}>\n", tag, body, tag));
        }
    }
    fn write_terminal(&mut self, token: &Token) {
        match token {
            Token::Keyword(k) => self.write_content_with_body("keyword", k.to_string()),
            Token::Symbol(s) => self.write_content_with_body("symbol", s.to_string()),
            Token::IntegerConstant(n) => self.write_content_with_body("integerConstant", n),
            Token::StringConstant(s) => self.write_content_with_body("stringConstant", s),
            Token::Identifier(s) => self.write_content_with_body("identifier", &s.0),
        }
    }

    fn peek_more(&mut self) -> bool {
        if let Some(token) = self.token_stream.next() {
            self.peeked.push_back(token);
            true
        } else {
            false
        }
    }
    fn peek_nth(&mut self, i: usize) -> Result<Option<&Token>> {
        while self.peeked.len() <= i {
            if !self.peek_more() {
                return Ok(None);
            }
        }
        match self.peeked[i].as_ref() {
            Ok(token) => Ok(Some(token)),
            _ => bail!("Unable to peek token"),
        }
    }
    fn peek_token(&mut self) -> Result<Option<&Token>> {
        self.peek_nth(0)
    }
    fn peek_keyword(&mut self) -> Result<Option<&Keyword>> {
        Ok(match self.peek_token()? {
            Some(Token::Keyword(keyword)) => Some(keyword),
            _ => None,
        })
    }
    fn peek_symbol(&mut self) -> Result<Option<&Symbol>> {
        Ok(match self.peek_token()? {
            Some(Token::Symbol(symbol)) => Some(symbol),
            _ => None,
        })
    }

    fn next_token(&mut self) -> Result<Token> {
        if let Some(peeked) = self.peeked.pop_front() {
            if let Ok(token) = &peeked {
                self.write_terminal(token);
            }
            return peeked;
        }
        match self.token_stream.next() {
            Some(Ok(token)) => {
                self.write_terminal(&token);
                Ok(token)
            }
            Some(Err(e)) => Err(e),
            None => bail!("Expected a token but reached to the end of the token stream"),
        }
    }
    fn next_keyword(&mut self) -> Result<Keyword> {
        match self.next_token()? {
            Token::Keyword(keyword) => Ok(keyword),
            token => bail!("Expected a keyword but was {:?}", token),
        }
    }
    fn next_identifier(&mut self) -> Result<Identifier> {
        match self.next_token()? {
            Token::Identifier(identifier) => Ok(identifier),
            token => bail!("Expected an identifier but was {:?}", token),
        }
    }
    fn next_symbol(&mut self) -> Result<Symbol> {
        match self.next_token()? {
            Token::Symbol(symbol) => Ok(symbol),
            token => bail!("Expected a symbol but was {:?}", token),
        }
    }

    fn next_non_void_type(&mut self) -> Result<Token> {
        let token = self.next_token()?;
        match token {
            Token::Keyword(Keyword::Int)
            | Token::Keyword(Keyword::Char)
            | Token::Keyword(Keyword::Boolean) => Ok(token),
            Token::Identifier(_) => Ok(token),
            _ => bail!("Expected a type but was {:?}", token),
        }
    }
    fn next_voidable_type(&mut self) -> Result<Token> {
        if Some(&Keyword::Void) == self.peek_keyword()? {
            self.next_token()
        } else {
            self.next_non_void_type()
        }
    }

    fn compile_class(&mut self) -> Result<()> {
        self.write_open("class");
        ensure!(Keyword::Class == self.next_keyword()?);
        let _name = self.next_identifier()?;
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        while self.try_compile_class_var_dec()? {}
        while self.try_compile_subroutine()? {}
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        ensure!(None == self.peek_token()?);
        self.write_close("class");
        Ok(())
    }
    fn try_compile_class_var_dec(&mut self) -> Result<bool> {
        match self.peek_keyword()? {
            Some(Keyword::Static) | Some(Keyword::Field) => {
                self.write_open("classVarDec");
                let _modifier = self.next_keyword()?;
                let _type = self.next_non_void_type();

                let _name = self.next_identifier()?;
                while let Some(Symbol::Comma) = self.peek_symbol()? {
                    ensure!(Symbol::Comma == self.next_symbol()?);
                    let _name = self.next_identifier()?;
                }
                ensure!(Symbol::Semicolon == self.next_symbol()?);
                self.write_close("classVarDec");
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn try_compile_subroutine(&mut self) -> Result<bool> {
        match self.peek_keyword()? {
            Some(Keyword::Constructor) | Some(Keyword::Function) | Some(Keyword::Method) => {
                self.write_open("subroutineDec");
                let _modifier = self.next_keyword()?;
                let _type = self.next_voidable_type();
                let _name = self.next_identifier()?;
                ensure!(Symbol::OpenParen == self.next_symbol()?);
                self.compile_parameter_list()?;
                ensure!(Symbol::CloseParen == self.next_symbol()?);

                self.write_open("subroutineBody");
                ensure!(Symbol::OpenBrace == self.next_symbol()?);
                while self.try_compile_var_dec()? {}
                self.compile_statements()?;
                ensure!(Symbol::CloseBrace == self.next_symbol()?);
                self.write_close("subroutineBody");

                self.write_close("subroutineDec");
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn compile_parameter_list(&mut self) -> Result<()> {
        self.write_open("parameterList");
        if self.peek_symbol()?.is_none() {
            let _type = self.next_non_void_type()?;
            let _name = self.next_identifier()?;
            while let Some(&Symbol::Comma) = self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                let _type = self.next_non_void_type()?;
                let _name = self.next_identifier()?;
            }
        }
        self.write_close("parameterList");
        Ok(())
    }
    fn try_compile_var_dec(&mut self) -> Result<bool> {
        if let Some(Keyword::Var) = self.peek_keyword()? {
            self.write_open("varDec");
            ensure!(Keyword::Var == self.next_keyword()?);
            let _type = self.next_non_void_type()?;
            let _name = self.next_identifier()?;
            while let Some(Symbol::Comma) = self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                let _name = self.next_identifier()?;
            }
            ensure!(Symbol::Semicolon == self.next_symbol()?);
            self.write_close("varDec");
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn compile_statements(&mut self) -> Result<()> {
        self.write_open("statements");
        while let Some(keyword) = self.peek_keyword()? {
            match keyword {
                Keyword::Let => self.compile_let()?,
                Keyword::If => self.compile_if()?,
                Keyword::While => self.compile_while()?,
                Keyword::Do => self.compile_do()?,
                Keyword::Return => self.compile_return()?,
                _ => break,
            };
        }
        self.write_close("statements");
        Ok(())
    }
    fn compile_do(&mut self) -> Result<()> {
        self.write_open("doStatement");
        ensure!(Keyword::Do == self.next_keyword()?);
        self.compile_subroutine_call()?;
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        self.write_close("doStatement");
        Ok(())
    }
    fn compile_let(&mut self) -> Result<()> {
        self.write_open("letStatement");
        ensure!(Keyword::Let == self.next_keyword()?);
        let _var = self.next_identifier()?;
        if let Some(Symbol::OpenBracket) = self.peek_symbol()? {
            ensure!(Symbol::OpenBracket == self.next_symbol()?);
            let _expression = self.compile_expression()?;
            ensure!(Symbol::CloseBracket == self.next_symbol()?);
        }
        ensure!(Symbol::Equal == self.next_symbol()?);
        self.compile_expression()?;
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        self.write_close("letStatement");
        Ok(())
    }
    fn compile_while(&mut self) -> Result<()> {
        self.write_open("whileStatement");
        ensure!(Keyword::While == self.next_keyword()?);
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        self.compile_expression()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        self.compile_statements()?;
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        self.write_close("whileStatement");
        Ok(())
    }
    fn compile_return(&mut self) -> Result<()> {
        self.write_open("returnStatement");
        ensure!(Keyword::Return == self.next_keyword()?);
        if let Some(Symbol::Semicolon) = self.peek_symbol()? {
        } else {
            self.compile_expression()?;
        }
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        self.write_close("returnStatement");
        Ok(())
    }
    fn compile_if(&mut self) -> Result<()> {
        self.write_open("ifStatement");
        ensure!(Keyword::If == self.next_keyword()?);
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        self.compile_expression()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        self.compile_statements()?;
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        if let Some(Keyword::Else) = self.peek_keyword()? {
            ensure!(Keyword::Else == self.next_keyword()?);
            ensure!(Symbol::OpenBrace == self.next_symbol()?);
            self.compile_statements()?;
            ensure!(Symbol::CloseBrace == self.next_symbol()?);
        }
        self.write_close("ifStatement");
        Ok(())
    }
    fn compile_subroutine_call(&mut self) -> Result<()> {
        self.next_identifier()?;
        if let Some(Symbol::Dot) = self.peek_symbol()? {
            ensure!(Symbol::Dot == self.next_symbol()?);
            self.next_identifier()?;
        }
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        self.compile_expression_list()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        Ok(())
    }
    fn compile_expression(&mut self) -> Result<()> {
        self.write_open("expression");
        self.compile_term()?;
        loop {
            if let Some(symbol) = self.peek_symbol()? {
                match symbol {
                    Symbol::Plus
                    | Symbol::Dash
                    | Symbol::Star
                    | Symbol::Slash
                    | Symbol::Ampersand
                    | Symbol::VerticalBar
                    | Symbol::LessThan
                    | Symbol::GreaterThan
                    | Symbol::Equal => {
                        self.next_symbol()?;
                        let _ = self.compile_term()?;
                    }
                    _ => break,
                }
            }
        }
        self.write_close("expression");
        Ok(())
    }
    fn compile_term(&mut self) -> Result<()> {
        if let Some(token) = self.peek_token()? {
            match token {
                Token::IntegerConstant(_) | Token::StringConstant(_) => {
                    self.write_open("term");
                    self.next_token()?;
                    self.write_close("term");
                }
                Token::Keyword(keyword) => match keyword {
                    Keyword::True | Keyword::False | Keyword::Null | Keyword::This => {
                        self.write_open("term");
                        self.next_keyword()?;
                        self.write_close("term");
                    }
                    _ => bail!("Unexpected keyword {:?}", keyword),
                },
                Token::Identifier(_) => {
                    let next_next_token = self.peek_nth(1)?;
                    if Some(&Token::Symbol(Symbol::OpenParen)) == next_next_token
                        || Some(&Token::Symbol(Symbol::Dot)) == next_next_token
                    {
                        // name '(' expression list ')'
                        // var '.' name '(' expression list ')'
                        self.write_open("term");
                        self.compile_subroutine_call()?;
                        self.write_close("term");
                    } else if Some(&Token::Symbol(Symbol::OpenBracket)) == next_next_token {
                        // var '[' expression ']'
                        self.write_open("term");
                        self.next_identifier()?;
                        ensure!(Symbol::OpenBracket == self.next_symbol()?);
                        self.compile_expression()?;
                        ensure!(Symbol::CloseBracket == self.next_symbol()?);
                        self.write_close("term");
                    } else {
                        // var
                        self.write_open("term");
                        self.next_identifier()?;
                        self.write_close("term");
                    }
                }
                Token::Symbol(symbol) => {
                    match symbol {
                        Symbol::OpenParen => {
                            // '(' expression ')'
                            self.write_open("term");
                            ensure!(Symbol::OpenParen == self.next_symbol()?);
                            self.compile_expression()?;
                            ensure!(Symbol::CloseParen == self.next_symbol()?);
                            self.write_close("term");
                        }
                        Symbol::Dash | Symbol::Tilde => {
                            self.write_open("term");
                            self.next_symbol()?;
                            self.compile_term()?;
                            self.write_close("term");
                        }
                        _ => bail!("Unexpected symbol {:?}", symbol),
                    }
                }
            }
            Ok(())
        } else {
            bail!("Unexpected end of token stream")
        }
    }
    fn compile_expression_list(&mut self) -> Result<()> {
        self.write_open("expressionList");
        if Some(&Symbol::CloseParen) != self.peek_symbol()? {
            self.compile_expression()?;
            while Some(&Symbol::Comma) == self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                self.compile_expression()?;
            }
        }
        self.write_close("expressionList");
        Ok(())
    }

    pub fn compile(&mut self) -> Result<String> {
        self.result_xml.clear();
        self.compile_class()
            .with_context(|| self.result_xml.clone())?;
        Ok(std::mem::take(&mut self.result_xml))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::jack::tokenizer::TokenIterator;
    use anyhow::Result;

    #[test]
    fn test() -> Result<()> {
        let input = r#"
            class Main {
              // foo
              function void main() {
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
        let mut analyzer = XMLAnalyzer::from(token_iterator);
        let ret = analyzer.compile()?;
        println!("{}", ret);
        Ok(())
    }
}
