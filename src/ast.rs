use std::collections::HashMap;

use crate::{error::Error, token::Token};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Statement(Statement),
    Expression(Expression),
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(Expression),
    Empty,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub errors: Vec<Error>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub name: Ident,
    pub value: Expression,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub value: Expression,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Program(Program),
    Ident(Ident),
    Literal(Literal),
    Infix {
        operator: Token,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Prefix {
        operator: Token,
        operand: Box<Expression>,
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
    Indexed {
        indexee: Box<Expression>,
        index: Box<Expression>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    Int(i32),
    String(String),
    Bool(Bool),
    Function {
        parameters: Vec<Ident>,
        body: Box<Expression>,
        capture: Vec<Ident>,
    },
    Collection {
        members: HashMap<Ident, Expression>,
    },
    Vector {
        elements: Vec<Expression>,
    },
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bool {
    True,
    False,
}
