use crate::ir::writer::IRWriter;
use crate::ir::{Arithmetic, FunctionCall, MemoryAccess, ProgramFlow, Segment};
use crate::jack::symbol_table::{Kind, SymbolTable, VariableType};
use crate::jack::token::{Identifier, Keyword, Symbol, Token};
use anyhow::{bail, ensure, Context, Result};
use std::collections::VecDeque;
use std::io::Write;
use std::iter::Fuse;

pub struct IRAnalyzer<I: Iterator<Item = Result<Token>>, W: Write> {
    token_stream: Fuse<I>,
    peeked: VecDeque<Result<Token>>,
    ir_writer: IRWriter<W>,
    symbol_table: SymbolTable,
    class_name: Option<Identifier>,
    next_label_id: usize,
}

impl<I: Iterator<Item = Result<Token>>, W: Write> IRAnalyzer<I, W> {
    pub fn from(token_stream: I, ir_writer: IRWriter<W>) -> Self {
        return Self {
            token_stream: token_stream.fuse(),
            peeked: VecDeque::new(),
            ir_writer,
            symbol_table: SymbolTable::new(),
            class_name: None,
            next_label_id: 0,
        };
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
            return peeked;
        }
        match self.token_stream.next() {
            Some(Ok(token)) => Ok(token),
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

    fn next_non_void_type(&mut self) -> Result<VariableType> {
        let token = self.next_token()?;
        match token {
            Token::Keyword(Keyword::Int) => Ok(VariableType::Int),
            Token::Keyword(Keyword::Char) => Ok(VariableType::Char),
            Token::Keyword(Keyword::Boolean) => Ok(VariableType::Boolean),
            Token::Identifier(ident) => Ok(VariableType::Object(ident)),
            _ => bail!("Expected a type but was {:?}", token),
        }
    }
    fn next_voidable_type(&mut self) -> Result<Option<VariableType>> {
        if Some(&Keyword::Void) == self.peek_keyword()? {
            self.next_token()?;
            Ok(None)
        } else {
            Ok(Some(self.next_non_void_type()?))
        }
    }

    fn generate_label(&mut self, prefix: &str) -> crate::ir::Symbol {
        let ret = format!("_{}_{}", prefix, self.next_label_id)
            .parse()
            .unwrap();
        self.next_label_id += 1;
        ret
    }
    fn get_class_name(&self) -> Result<&Identifier> {
        self.class_name
            .as_ref()
            .with_context(|| format!("Why not inside a class???"))
    }

    fn compile_invoke_function(&mut self, name: &str, n_args: u16) -> Result<()> {
        self.ir_writer.write_function_call(&FunctionCall::Invoke {
            name: name.parse()?,
            n_args,
        })
    }

    fn compile_class(&mut self) -> Result<()> {
        ensure!(Keyword::Class == self.next_keyword()?);
        self.symbol_table.start_new_class();
        let name = self.next_identifier()?;
        ensure!(self.class_name.replace(name).is_none());
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        while self.try_compile_class_var_dec()? {}
        while self.try_compile_subroutine()? {}
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        ensure!(None == self.peek_token()?);
        ensure!(self.class_name.take().is_some());
        Ok(())
    }
    fn try_compile_class_var_dec(&mut self) -> Result<bool> {
        match self.peek_keyword()? {
            Some(Keyword::Static) | Some(Keyword::Field) => {
                let modifier = self.next_keyword()?;
                let modifier = match modifier {
                    Keyword::Static => Kind::Static,
                    Keyword::Field => Kind::Field,
                    _ => bail!("??? got {:?}", modifier),
                };
                let t = self.next_non_void_type()?;
                let name = self.next_identifier()?;
                self.symbol_table
                    .define_class_variable(&modifier, &t, name)?;
                while let Some(Symbol::Comma) = self.peek_symbol()? {
                    ensure!(Symbol::Comma == self.next_symbol()?);
                    let name = self.next_identifier()?;
                    self.symbol_table
                        .define_class_variable(&modifier, &t, name)?;
                }
                ensure!(Symbol::Semicolon == self.next_symbol()?);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn try_compile_subroutine(&mut self) -> Result<bool> {
        match self.peek_keyword()? {
            Some(Keyword::Constructor) | Some(Keyword::Function) | Some(Keyword::Method) => {
                self.symbol_table.start_new_subroutine();

                let modifier = self.next_keyword()?;
                let _type = self.next_voidable_type();
                let name = self.next_identifier()?;
                self.ir_writer.comment(&format!(
                    "start function {}.{}",
                    self.get_class_name()?.0,
                    name.0
                ))?;
                ensure!(Symbol::OpenParen == self.next_symbol()?);
                self.compile_parameter_list()?;
                ensure!(Symbol::CloseParen == self.next_symbol()?);
                ensure!(Symbol::OpenBrace == self.next_symbol()?);
                while self.try_compile_var_dec()? {}

                let n_locals = self.symbol_table.get_count(Kind::Local);
                self.ir_writer.write_function_call(&FunctionCall::Declare {
                    name: format!("{}.{}", self.get_class_name()?.0, name.0).parse()?,
                    n_locals,
                })?;
                match modifier {
                    Keyword::Constructor => {
                        self.ir_writer.comment("Allocate memory")?;
                        self.alloc(self.symbol_table.get_count(Kind::Field))?;
                        self.ir_writer.comment("Set this pointer")?;
                        // set this to self
                        self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                            segment: Segment::Pointer,
                            index: 0,
                        })?;
                    }
                    Keyword::Function => {
                        // NoOp
                    }
                    Keyword::Method => {
                        self.symbol_table.shift_argument_variables_by_one();
                        self.ir_writer.comment("Set this pointer")?;
                        // set this to self
                        self.ir_writer.write_memory_access(&MemoryAccess::Push {
                            segment: Segment::Argument,
                            index: 0,
                        })?;
                        self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                            segment: Segment::Pointer,
                            index: 0,
                        })?;
                    }
                    _ => unreachable!(),
                }
                self.ir_writer.comment("Function body")?;
                self.compile_statements()?;
                ensure!(Symbol::CloseBrace == self.next_symbol()?);
                self.ir_writer.comment(&format!(
                    "end function {}.{}",
                    self.get_class_name()?.0,
                    name.0
                ))?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    fn compile_parameter_list(&mut self) -> Result<()> {
        if self.peek_symbol()?.is_none() {
            let t = self.next_non_void_type()?;
            let name = self.next_identifier()?;
            self.symbol_table.define_argument_variable(&t, name)?;
            while let Some(&Symbol::Comma) = self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                let t = self.next_non_void_type()?;
                let name = self.next_identifier()?;
                self.symbol_table.define_argument_variable(&t, name)?;
            }
        }
        Ok(())
    }
    fn try_compile_var_dec(&mut self) -> Result<bool> {
        if let Some(Keyword::Var) = self.peek_keyword()? {
            ensure!(Keyword::Var == self.next_keyword()?);
            let t = self.next_non_void_type()?;
            let name = self.next_identifier()?;
            self.symbol_table.define_local_variable(&t, name)?;
            while let Some(Symbol::Comma) = self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                let name = self.next_identifier()?;
                self.symbol_table.define_local_variable(&t, name)?;
            }
            ensure!(Symbol::Semicolon == self.next_symbol()?);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn compile_statements(&mut self) -> Result<()> {
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
        Ok(())
    }
    fn compile_do(&mut self) -> Result<()> {
        ensure!(Keyword::Do == self.next_keyword()?);
        self.compile_subroutine_call()?;
        self.ir_writer.write_memory_access(&MemoryAccess::Pop {
            segment: Segment::Temp,
            index: 0,
        })?;
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        Ok(())
    }

    fn kind_to_segment(kind: &Kind) -> Segment {
        match kind {
            Kind::Static => Segment::Static,
            Kind::Field => Segment::This,
            Kind::Argument => Segment::Argument,
            Kind::Local => Segment::Local,
        }
    }

    fn compile_let(&mut self) -> Result<()> {
        ensure!(Keyword::Let == self.next_keyword()?);
        let var = self.next_identifier()?;
        if let Some(Symbol::OpenBracket) = self.peek_symbol()? {
            // var[expr] = expr
            self.compile_push_variable(&var)?;
            ensure!(Symbol::OpenBracket == self.next_symbol()?);
            self.compile_expression()?;
            ensure!(Symbol::CloseBracket == self.next_symbol()?);
            self.ir_writer.write_arithmetic(&Arithmetic::Add)?;

            ensure!(Symbol::Equal == self.next_symbol()?);
            self.compile_expression()?;
            self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                segment: Segment::Temp,
                index: 0,
            })?;
            self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                segment: Segment::Pointer,
                index: 1,
            })?;
            self.ir_writer.write_memory_access(&MemoryAccess::Push {
                segment: Segment::Temp,
                index: 0,
            })?;
            self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                segment: Segment::That,
                index: 0,
            })?;
        } else {
            // var = expr
            ensure!(Symbol::Equal == self.next_symbol()?);
            let (kind, _type, id) = self
                .symbol_table
                .lookup(&var)
                .with_context(|| format!("Unknown variable `{}`", var.0))?;
            self.compile_expression()?;
            self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                segment: Self::kind_to_segment(&kind),
                index: id,
            })?;
        }
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        Ok(())
    }
    fn compile_while(&mut self) -> Result<()> {
        ensure!(Keyword::While == self.next_keyword()?);
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        let before_while = self.generate_label("before_while");
        let after_while = self.generate_label("after_while");
        self.ir_writer.write_program_flow(&ProgramFlow::Label {
            label: before_while.clone(),
        })?;
        self.compile_expression()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        self.ir_writer.write_arithmetic(&Arithmetic::Not)?;
        self.ir_writer.write_program_flow(&ProgramFlow::IfGoto {
            label: after_while.clone(),
        })?;
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        self.compile_statements()?;
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        self.ir_writer.write_program_flow(&ProgramFlow::Goto {
            label: before_while,
        })?;
        self.ir_writer
            .write_program_flow(&ProgramFlow::Label { label: after_while })?;
        Ok(())
    }
    fn compile_return(&mut self) -> Result<()> {
        self.write_open("returnStatement")?;
        ensure!(Keyword::Return == self.next_keyword()?);
        if let Some(Symbol::Semicolon) = self.peek_symbol()? {
            self.compile_push_constant(0)?;
        } else {
            self.compile_expression()?;
        }
        ensure!(Symbol::Semicolon == self.next_symbol()?);
        self.ir_writer.write_function_call(&FunctionCall::Return)?;
        self.write_close("returnStatement")?;
        Ok(())
    }
    fn compile_if(&mut self) -> Result<()> {
        ensure!(Keyword::If == self.next_keyword()?);
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        self.compile_expression()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        let end_if = self.generate_label("end_if");
        self.ir_writer.write_arithmetic(&Arithmetic::Not)?;
        self.ir_writer.write_program_flow(&ProgramFlow::IfGoto {
            label: end_if.clone(),
        })?;
        ensure!(Symbol::OpenBrace == self.next_symbol()?);
        self.compile_statements()?;
        ensure!(Symbol::CloseBrace == self.next_symbol()?);
        if let Some(Keyword::Else) = self.peek_keyword()? {
            ensure!(Keyword::Else == self.next_keyword()?);
            ensure!(Symbol::OpenBrace == self.next_symbol()?);
            let end_else = self.generate_label("end_else");
            self.ir_writer.write_program_flow(&ProgramFlow::Goto {
                label: end_else.clone(),
            })?;
            self.ir_writer.write_program_flow(&ProgramFlow::Label {
                label: end_if.clone(),
            })?;
            self.compile_statements()?;
            ensure!(Symbol::CloseBrace == self.next_symbol()?);
            self.ir_writer.write_program_flow(&ProgramFlow::Label {
                label: end_else.clone(),
            })?;
        } else {
            self.ir_writer.write_program_flow(&ProgramFlow::Label {
                label: end_if.clone(),
            })?;
        }
        Ok(())
    }
    fn compile_subroutine_call(&mut self) -> Result<()> {
        let ident = self.next_identifier()?;
        let (method_name, added_n_args) = if let Some(Symbol::Dot) = self.peek_symbol()? {
            ensure!(Symbol::Dot == self.next_symbol()?);
            // var.method() or Class.func()
            let var_or_class = ident;
            let func_or_method = self.next_identifier()?;
            if let Some((_kind, variable_type, _id)) = self.symbol_table.lookup(&var_or_class) {
                // var.method
                let var = var_or_class;
                self.compile_push_variable(&var)?;
                if let VariableType::Object(object_type) = variable_type {
                    (format!("{}.{}", object_type.0, func_or_method.0), 1)
                } else {
                    bail!(
                        "Primitive type {:?} doesn't have method {}",
                        variable_type,
                        func_or_method.0
                    );
                }
            } else {
                // Class.func()
                let class = var_or_class;
                (format!("{}.{}", class.0, func_or_method.0), 0)
            }
        } else {
            // this.method()
            self.ir_writer.write_memory_access(&MemoryAccess::Push {
                segment: Segment::Pointer,
                index: 0,
            })?;
            (format!("{}.{}", self.get_class_name()?.0, ident.0), 1)
        };
        ensure!(Symbol::OpenParen == self.next_symbol()?);
        let n_args = self.compile_expression_list()?;
        ensure!(Symbol::CloseParen == self.next_symbol()?);
        self.ir_writer.write_function_call(&FunctionCall::Invoke {
            name: method_name.parse()?,
            n_args: n_args + added_n_args,
        })?;
        Ok(())
    }
    fn compile_expression(&mut self) -> Result<()> {
        self.compile_term()?;
        let mut delayed: Vec<(Arithmetic, usize)> = Vec::new();
        loop {
            if let Some(symbol) = self.peek_symbol()? {
                match symbol {
                    Symbol::Plus
                    | Symbol::Dash
                    | Symbol::Ampersand
                    | Symbol::VerticalBar
                    | Symbol::LessThan
                    | Symbol::GreaterThan
                    | Symbol::Equal => {
                        let symbol = self.next_symbol()?;
                        let (op, precedence) = match symbol {
                            Symbol::Plus => (Arithmetic::Add, 3),
                            Symbol::Dash => (Arithmetic::Sub, 3),
                            Symbol::Ampersand => (Arithmetic::And, 2),
                            Symbol::VerticalBar => (Arithmetic::Or, 1),
                            Symbol::LessThan => (Arithmetic::Lt, 0),
                            Symbol::GreaterThan => (Arithmetic::Gt, 0),
                            Symbol::Equal => (Arithmetic::Eq, 0),
                            _ => unreachable!(),
                        };
                        while let Some((_, p)) = delayed.last() {
                            if &precedence <= p {
                                self.ir_writer.write_arithmetic(&delayed.pop().unwrap().0)?;
                            } else {
                                break;
                            }
                        }
                        self.compile_term()?;
                        delayed.push((op, precedence));
                    }
                    Symbol::Star => {
                        ensure!(Symbol::Star == self.next_symbol()?);
                        self.compile_term()?;
                        self.compile_invoke_function("Math.multiply", 2)?
                    }
                    Symbol::Slash => {
                        ensure!(Symbol::Slash == self.next_symbol()?);
                        self.compile_term()?;
                        self.compile_invoke_function("Math.divide", 2)?
                    }
                    _ => break,
                }
            }
        }
        while let Some((op, _)) = delayed.pop() {
            self.ir_writer.write_arithmetic(&op)?;
        }
        Ok(())
    }
    fn compile_push_constant(&mut self, v: i16) -> Result<()> {
        if v < 0 {
            self.compile_push_constant(!v)?;
            self.ir_writer.write_arithmetic(&Arithmetic::Not)
        } else {
            self.ir_writer.write_memory_access(&MemoryAccess::Push {
                segment: Segment::Constant,
                index: v as u16,
            })
        }
    }
    fn compile_push_string_constant(&mut self, s: &String) -> Result<()> {
        let v: Vec<_> = s.encode_utf16().collect();
        self.compile_push_constant(v.len() as i16)?;
        self.compile_invoke_function("String.new", 1)?;
        for c in v {
            self.compile_push_constant(c as i16)?;
            self.compile_invoke_function("String.appendChar", 2)?;
        }
        Ok(())
    }
    fn compile_push_variable(&mut self, variable: &Identifier) -> Result<()> {
        let (kind, _type, id) = self
            .symbol_table
            .lookup(variable)
            .with_context(|| format!("Unknown variable {}", variable.0))?;
        self.ir_writer.write_memory_access(&MemoryAccess::Push {
            segment: Self::kind_to_segment(&kind),
            index: id,
        })
    }
    fn alloc(&mut self, n: u16) -> Result<()> {
        self.compile_push_constant(n as i16)?;
        self.compile_invoke_function("Memory.alloc", 1)
    }

    fn compile_term(&mut self) -> Result<()> {
        if let Some(token) = self.peek_token()? {
            match token {
                Token::IntegerConstant(_) | Token::StringConstant(_) => {
                    let token = self.next_token()?;
                    match token {
                        Token::IntegerConstant(v) => self.compile_push_constant(v)?,
                        Token::StringConstant(s) => self.compile_push_string_constant(&s)?,
                        _ => unreachable!(),
                    }
                }
                Token::Keyword(_) => {
                    match self.next_keyword()? {
                        Keyword::True => self.compile_push_constant(-1)?,
                        // TODO: I have a feeling that it is better to set Null to -1...
                        Keyword::False | Keyword::Null => self.compile_push_constant(0)?,
                        Keyword::This => {
                            self.ir_writer.write_memory_access(&MemoryAccess::Push {
                                segment: Segment::Pointer,
                                index: 0,
                            })?
                        }
                        keyword => bail!("Unexpected keyword {:?}", keyword),
                    }
                }
                Token::Identifier(_) => {
                    let next_next_token = self.peek_nth(1)?;
                    if Some(&Token::Symbol(Symbol::OpenParen)) == next_next_token
                        || Some(&Token::Symbol(Symbol::Dot)) == next_next_token
                    {
                        // name '(' expression list ')'
                        // var '.' name '(' expression list ')'
                        // Function invocation.
                        self.compile_subroutine_call()?;
                    } else if Some(&Token::Symbol(Symbol::OpenBracket)) == next_next_token {
                        // var '[' expression ']'
                        // Array access
                        let var_name = self.next_identifier()?;
                        self.compile_push_variable(&var_name)?;
                        ensure!(Symbol::OpenBracket == self.next_symbol()?);
                        self.compile_expression()?;
                        ensure!(Symbol::CloseBracket == self.next_symbol()?);
                        self.ir_writer.write_arithmetic(&Arithmetic::Add)?;
                        self.ir_writer.write_memory_access(&MemoryAccess::Pop {
                            segment: Segment::Pointer,
                            index: 1,
                        })?;
                        self.ir_writer.write_memory_access(&MemoryAccess::Push {
                            segment: Segment::That,
                            index: 0,
                        })?;
                    } else {
                        // var
                        let var_name = self.next_identifier()?;
                        self.compile_push_variable(&var_name)?;
                    }
                }
                Token::Symbol(symbol) => {
                    match symbol {
                        Symbol::OpenParen => {
                            // '(' expression ')'
                            ensure!(Symbol::OpenParen == self.next_symbol()?);
                            self.compile_expression()?;
                            ensure!(Symbol::CloseParen == self.next_symbol()?);
                        }
                        Symbol::Dash => {
                            ensure!(Symbol::Dash == self.next_symbol()?);
                            self.compile_term()?;
                            self.ir_writer.write_arithmetic(&Arithmetic::Neg)?;
                        }
                        Symbol::Tilde => {
                            ensure!(Symbol::Tilde == self.next_symbol()?);
                            self.compile_term()?;
                            self.ir_writer.write_arithmetic(&Arithmetic::Not)?;
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
    fn compile_expression_list(&mut self) -> Result<u16> {
        self.write_open("expressionList")?;
        let mut n = 0;
        if Some(&Symbol::CloseParen) != self.peek_symbol()? {
            self.compile_expression()?;
            n += 1;
            while Some(&Symbol::Comma) == self.peek_symbol()? {
                ensure!(Symbol::Comma == self.next_symbol()?);
                self.compile_expression()?;
                n += 1;
            }
        }
        self.write_close("expressionList")?;
        Ok(n)
    }

    pub fn compile(&mut self) -> Result<()> {
        self.compile_class()
            .with_context(|| "Failed to compile...")?;
        self.ir_writer.flush()?;
        Ok(())
    }
    fn write_open(&mut self, tag: &str) -> Result<()> {
        self.ir_writer.comment(&format!("Begin {}", tag))
    }
    fn write_close(&mut self, tag: &str) -> Result<()> {
        self.ir_writer.comment(&format!("End {}", tag))
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
        let ret = vec![];
        let writer = IRWriter::new(ret);
        let mut analyzer = IRAnalyzer::from(token_iterator, writer);
        analyzer.compile()?;
        Ok(())
    }
}
