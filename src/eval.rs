use crate::{
    ast::{self, Expression, Literal, Node, Statement},
    object::{self, Integer, Null, Object},
    token::{Operator, Token},
};

use error::{Error, Result};

pub mod error {
    pub type Result<T> = std::result::Result<T, Error>;

    #[derive(Debug, Clone)]
    pub enum Error {
        Eval(String),
    }
}

pub struct Eval {}

impl Eval {
    pub fn eval(node: Node) -> Result<Box<dyn Object>> {
        let ret = match node {
            Node::Statement(Statement::Expression(e)) => Eval::eval(Node::Expression(e))?,
            Node::Expression(Expression::Literal(Literal::Int(val))) => Integer::erased(val),
            Node::Expression(Expression::Literal(Literal::Bool(b))) => match b {
                ast::Bool::True => object::Bool::erased(true),
                ast::Bool::False => object::Bool::erased(false),
            },
            Node::Expression(Expression::Prefix { operator, operand }) => {
                Self::eval_prefix(operator, *operand)?
            }
            Node::Expression(Expression::Infix { operator, lhs, rhs }) => {
                Self::eval_infix(operator, *lhs, *rhs)?
            }
            Node::Expression(Expression::Program(pro)) => Self::eval_statements(pro.statements)?,
            _ => todo!(),
        };

        Ok(ret)
    }

    fn eval_statements(statements: Vec<Statement>) -> Result<Box<dyn Object>> {
        statements
            .into_iter()
            .try_fold(Null::erased(), |_, st| Self::eval(Node::Statement(st)))
    }

    fn eval_prefix(operator: Token, operand: Expression) -> Result<Box<dyn Object>> {
        let mut operand = Self::eval(Node::Expression(operand))?;
        match operator {
            Token::Operator(Operator::Bang) => {
                operand = (operand.v_table().inverte)();
            }
            Token::Operator(Operator::Minus) => {
                operand = (operand.v_table().negate)();
            }
            _ => unsafe { core::hint::unreachable_unchecked() },
        }

        Ok(operand)
    }

    fn eval_infix(operator: Token, lhs: Expression, rhs: Expression) -> Result<Box<dyn Object>> {
        let lhs = Self::eval(Node::Expression(lhs))?;
        let rhs = Self::eval(Node::Expression(rhs))?;
        let unsup_error = Error::Eval("Unsupported operator for operand types".into());

        let op = match operator {
            Token::Operator(Operator::Minus) => "sub_lhs",
            Token::Operator(Operator::Plus) => "add_lhs",
            Token::Operator(Operator::Multiply) => "mul_lhs",
            Token::Operator(Operator::Divide) => "div_lhs",
            _ => Err(Error::Eval("Infix operator is not supported".into()))?,
        };

        let sub = lhs.v_table().get(op).ok_or(unsup_error.clone())?;

        sub(Some(rhs)).ok_or(unsup_error)
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test_integer_lit() {
        let input = r#"
            5;
            10;
            "#;

        let mut p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap()
            .statements
            .into_iter();

        let mut e = Eval::eval(Node::Statement(p.next().unwrap()));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "5");

            e = Eval::eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "10");
        }

        let p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap();

        let e = Eval::eval(Node::Expression(Expression::Program(p)));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "10");
        }
    }
}
