use crate::regex;
use anyhow::{anyhow, ensure, Error, Result};
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq)]
pub enum Keyword {
    Class,
    Constructor,
    Function,
    Method,
    Field,
    Static,
    Var,
    Int,
    Char,
    Boolean,
    Void,
    True,
    False,
    Null,
    This,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
}
#[derive(Debug, Eq, PartialEq)]
pub enum Symbol {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    Dot,
    Comma,
    Semicolon,
    Plus,
    Dash,
    Star,
    Slash,
    Ampersand,
    VerticalBar,
    LessThan,
    GreaterThan,
    Equal,
    Tilde,
}
#[derive(Debug, Eq, PartialEq)]
pub struct Identifier(pub String);

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
    Keyword(Keyword),
    Symbol(Symbol),
    IntegerConstant(i16),
    StringConstant(String),
    Identifier(Identifier),
}

impl From<Keyword> for Token {
    fn from(keyword: Keyword) -> Self {
        Token::Keyword(keyword)
    }
}
impl From<Symbol> for Token {
    fn from(symbol: Symbol) -> Self {
        Token::Symbol(symbol)
    }
}
impl From<Identifier> for Token {
    fn from(identifier: Identifier) -> Self {
        Token::Identifier(identifier)
    }
}

impl FromStr for Keyword {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keyword = match s {
            "class" => Keyword::Class,
            "constructor" => Keyword::Constructor,
            "function" => Keyword::Function,
            "method" => Keyword::Method,
            "field" => Keyword::Field,
            "static" => Keyword::Static,
            "var" => Keyword::Var,
            "int" => Keyword::Int,
            "char" => Keyword::Char,
            "boolean" => Keyword::Boolean,
            "void" => Keyword::Void,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "null" => Keyword::Null,
            "this" => Keyword::This,
            "let" => Keyword::Let,
            "do" => Keyword::Do,
            "if" => Keyword::If,
            "else" => Keyword::Else,
            "while" => Keyword::While,
            "return" => Keyword::Return,
            _ => return Err(anyhow!("Unknown keyword {}", s)),
        };
        Ok(keyword)
    }
}
impl FromStr for Symbol {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sym = match s {
            "{" => Symbol::OpenBrace,
            "}" => Symbol::CloseBrace,
            "(" => Symbol::OpenParen,
            ")" => Symbol::CloseParen,
            "[" => Symbol::OpenBracket,
            "]" => Symbol::CloseBracket,
            "." => Symbol::Dot,
            "," => Symbol::Comma,
            ";" => Symbol::Semicolon,
            "+" => Symbol::Plus,
            "-" => Symbol::Dash,
            "*" => Symbol::Star,
            "/" => Symbol::Slash,
            "&" => Symbol::Ampersand,
            "|" => Symbol::VerticalBar,
            "<" => Symbol::LessThan,
            ">" => Symbol::GreaterThan,
            "=" => Symbol::Equal,
            "~" => Symbol::Tilde,
            _ => {
                return Err(anyhow!("Unknown symbol {}", s));
            }
        };
        Ok(sym)
    }
}
impl FromStr for Identifier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let identifier_pattern = regex!("^[a-zA-Z_][0-9a-zA-Z_]*$");
        ensure!(
            identifier_pattern.is_match(s),
            "String `{}` is invalid as an identifier",
            s
        );
        Ok(Identifier(s.to_owned()))
    }
}

impl ToString for Keyword {
    fn to_string(&self) -> String {
        match self {
            Keyword::Class => "class",
            Keyword::Constructor => "constructor",
            Keyword::Function => "function",
            Keyword::Method => "method",
            Keyword::Field => "field",
            Keyword::Static => "static",
            Keyword::Var => "var",
            Keyword::Int => "int",
            Keyword::Char => "char",
            Keyword::Boolean => "boolean",
            Keyword::Void => "void",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Null => "null",
            Keyword::This => "this",
            Keyword::Let => "let",
            Keyword::Do => "do",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::While => "while",
            Keyword::Return => "return",
        }
        .to_owned()
    }
}
impl ToString for Symbol {
    fn to_string(&self) -> String {
        let c = match self {
            Symbol::OpenBrace => '{',
            Symbol::CloseBrace => '}',
            Symbol::OpenParen => '(',
            Symbol::CloseParen => ')',
            Symbol::OpenBracket => '[',
            Symbol::CloseBracket => ']',
            Symbol::Dot => '.',
            Symbol::Comma => ',',
            Symbol::Semicolon => ';',
            Symbol::Plus => '+',
            Symbol::Dash => '-',
            Symbol::Star => '*',
            Symbol::Slash => '/',
            Symbol::Ampersand => '&',
            Symbol::VerticalBar => '|',
            Symbol::LessThan => '<',
            Symbol::GreaterThan => '>',
            Symbol::Equal => '=',
            Symbol::Tilde => '~',
        };
        c.to_string()
    }
}
impl ToString for Identifier {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
