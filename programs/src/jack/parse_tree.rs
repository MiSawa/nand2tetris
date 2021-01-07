/*
use crate::jack::token::{Identifier, Symbol};

#[derive(Eq, PartialEq, Debug)]
pub struct ClassDeclaration {
    name: Identifier,
    variables: Vec<ClassVariableDeclaration>,
    subroutines: Vec<SubroutineDeclaration>,
}
#[derive(Eq, PartialEq, Debug)]
pub enum ClassVarModifier {
    Static,
    Field,
}
#[derive(Eq, PartialEq, Debug)]
pub struct ClassVariableDeclaration {
    modifier: ClassVarModifier,
    variable_type: VariableType,
    names: Vec<Identifier>,
}
#[derive(Eq, PartialEq, Debug)]
pub enum SubroutineModifier {
    Constructor,
    Function,
    Method,
}
#[derive(Eq, PartialEq, Debug)]
pub struct SubroutineDeclaration {
    modifier: SubroutineModifier,
    result_type: VoidOrType,
    name: Identifier,
    params: Vec<ParameterDeclaration>,
    body: SubroutineBody,
}
#[derive(Eq, PartialEq, Debug)]
pub struct ParameterDeclaration {
    param_type: VariableType,
    name: Identifier,
}
#[derive(Eq, PartialEq, Debug)]
pub struct SubroutineBody {
    variables: Vec<VariableDeclaration>,
    statements: Statements,
}
#[derive(Eq, PartialEq, Debug)]
pub struct VariableDeclaration {
    variable_type: VariableType,
    names: Vec<Identifier>,
}
#[derive(Eq, PartialEq, Debug)]
pub struct Statements {
    statements: Vec<Statement>,
}
#[derive(Eq, PartialEq, Debug)]
pub enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}
#[derive(Eq, PartialEq, Debug)]
pub struct LetStatement {
    name: Identifier,
    index: Option<Expression>,
    expression: Expression,
}
#[derive(Eq, PartialEq, Debug)]
pub struct IfStatement {
    condition: Expression,
    true_statements: Statements,
    false_statements: Statements,
}
#[derive(Eq, PartialEq, Debug)]
pub struct WhileStatement {
    condition: Expression,
    statements: Statements,
}
#[derive(Eq, PartialEq, Debug)]
pub struct DoStatement {
    call: SubroutineCall,
}
#[derive(Eq, PartialEq, Debug)]
pub struct ReturnStatement {
    expression: Option<Expression>,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Expression {
    terms: Vec<Term>,
    ops: Vec<BinaryOperator>,
}
#[derive(Eq, PartialEq, Debug)]
pub enum Term {
    IntegerConstant(i16),
    StringConstant(String),
    KeywordConstant(ConstantKeyword),
    Variable(Identifier),
    Indexing(Identifier, Expression),
    Call(SubroutineCall),
    Parenthesized(Expression),
    Unary(UnaryOperator, Box<Term>),
}

#[derive(Eq, PartialEq, Debug)]
pub enum SubroutineCall {
    Unqualified {
        name: Identifier,
        args: Vec<Expression>,
    },
    Qualified {
        qualification: Identifier,
        name: Identifier,
        args: Vec<Expression>,
    },
}

#[derive(Eq, PartialEq, Debug)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    Lt,
    Gt,
    Eq,
}
#[derive(Eq, PartialEq, Debug)]
pub enum UnaryOperator {
    Negate,
    Invert,
}
#[derive(Eq, PartialEq, Debug)]
pub enum ConstantKeyword {
    True,
    False,
    Null,
    This,
}

#[derive(Eq, PartialEq, Debug)]
pub enum VariableType {
    Int,
    Char,
    Boolean,
    Object(Identifier),
}
#[derive(Eq, PartialEq, Debug)]
pub enum VoidOrType {
    Void,
    NonVoid(VariableType),
}
*/
