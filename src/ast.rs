use crate::{error::Error, token::Token};

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Statement(Statement),
    Expression(Expression),
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Program(Program),
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(Expression),
    Empty,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub errors: Vec<Error>,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Ident,
    pub value: Expression,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct ReturnStatement {
    pub value: Expression,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Expression {
    Ident(Ident),
    Literal(Literal),
    Infix {
        operator: Token,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Block {
        statements: Vec<Statement>,
    },
    If {
        condition: Box<Expression>,
        consequence: Box<Expression>,
        alternative: Option<Box<Expression>>,
    },
    Invoked {
        invoked: Box<Expression>,
        args: Vec<Expression>,
    },
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Literal {
    Int(i32),
    String(String),
    Bool(Bool),
    Function {
        parameters: Vec<Ident>,
        body: Box<Expression>,
    },
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub enum Bool {
    True,
    False,
}
