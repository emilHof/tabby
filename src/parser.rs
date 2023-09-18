use crate::{
    ast::{Bool, Expression, Ident, LetStatement, Literal, Program, ReturnStatement, Statement},
    error::{Error, Result},
    lexer::Lexer,
    token::{Keyword, Operator, Token},
};

pub struct Parser {
    lexer: Lexer,
    cur: Token,
    peek: Token,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Result<Self> {
        Ok(Self {
            cur: lexer.next_token()?,
            peek: lexer.next_token()?,
            lexer,
            errors: vec![],
        })
    }

    pub fn next_token(&mut self) -> Result<()> {
        std::mem::swap(&mut self.cur, &mut self.peek);
        self.peek = self.lexer.next_token()?;
        Ok(())
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut statements = vec![];

        while self.cur != Token::EOF {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(e) => self.errors.push(e),
            };

            self.next_token()?;
        }

        Ok(Program {
            statements,
            errors: self.errors.drain(..).collect(),
        })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.cur {
            Token::Keyword(Keyword::Let) => Ok(Statement::Let(self.parse_let()?)),
            Token::Keyword(Keyword::Return) => Ok(Statement::Return(self.parse_return()?)),
            Token::Semicolon => Ok(Statement::Empty),
            _ => {
                let expression = self.parse_expression(Precedence::Lowest)?;
                if matches!(self.peek, Token::Semicolon) {
                    self.next_token()?;
                }
                Ok(Statement::Expression(expression))
            }
        }
    }

    fn parse_return(&mut self) -> Result<ReturnStatement> {
        self.next_token()?;

        let value = self.parse_expression(Precedence::Lowest)?;

        self.expect_peek(
            |t| matches!(t, Token::Semicolon),
            Error::LetStatement("Expected semicolon at the end of statment".into()),
        )?;

        Ok(ReturnStatement { value })
    }

    fn parse_let(&mut self) -> Result<LetStatement> {
        self.expect_peek(
            |t| matches!(t, Token::Ident(_)),
            Error::LetStatement("Expected identifier after `let`".into()),
        )?;

        let name = match &self.cur {
            Token::Ident(name) => Ident { name: name.clone() },
            _ => unsafe { core::hint::unreachable_unchecked() },
        };

        self.expect_peek(
            |t| matches!(t, Token::Operator(Operator::Assign)),
            Error::LetStatement("Expected assignment operator after identifier".into()),
        )?;

        self.next_token()?;
        let value = self.parse_expression(Precedence::Lowest)?;

        self.expect_peek(
            |t| matches!(t, Token::Semicolon),
            Error::LetStatement("Expected semicolon at the end of statment".into()),
        )?;

        Ok(LetStatement { name, value })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut lhs = match self.cur {
            Token::LParen => self.parse_grouped()?,
            Token::LBrace => self.parse_block()?,
            Token::Ident(_) => self.parse_ident()?,
            Token::Int(_) => self.parse_int()?,
            Token::Keyword(Keyword::If) => self.parse_if()?,
            Token::Keyword(Keyword::True | Keyword::False) => self.parse_bool()?,
            Token::Keyword(Keyword::Function) => self.parse_function()?,
            Token::Operator(Operator::Bang | Operator::Minus) => self.parse_prefix()?,
            Token::Semicolon
            | Token::Operator(_)
            | Token::Keyword(_)
            | Token::EOF
            | Token::Comman
            | Token::RParen
            | Token::RBrace
            | Token::Illegal => todo!(),
        };

        while !matches!(self.peek, Token::Semicolon) && precedence < self.peek_precedence() {
            lhs = match self.peek {
                Token::Operator(Operator::Assign)
                | Token::Operator(Operator::Plus)
                | Token::Operator(Operator::Minus)
                | Token::Operator(Operator::Divide)
                | Token::Operator(Operator::Multiply)
                | Token::Operator(Operator::Equal)
                | Token::Operator(Operator::NotEqual)
                | Token::Operator(Operator::Less)
                | Token::Operator(Operator::LessOrEqual)
                | Token::Operator(Operator::Greater)
                | Token::Operator(Operator::GreaterOrEqual) => {
                    self.next_token()?;
                    self.parse_infix_operator(lhs)?
                }
                Token::LParen => {
                    self.next_token()?;
                    self.parse_invoke(lhs)?
                }
                Token::Semicolon
                | Token::Operator(_)
                | Token::Keyword(_)
                | Token::EOF
                | Token::Comman
                | Token::Ident(_)
                | Token::Int(_)
                | Token::RParen
                | Token::LBrace
                | Token::RBrace
                | Token::Illegal => break,
            };
        }

        return Ok(lhs);
    }

    fn parse_prefix(&mut self) -> Result<Expression> {
        let operator = self.cur.clone();
        self.next_token()?;
        let operand = Box::new(self.parse_expression(Precedence::Prefix)?);

        Ok(Expression::Prefix { operator, operand })
    }

    fn parse_invoke(&mut self, lhs: Expression) -> Result<Expression> {
        let mut args = vec![];

        while !matches!(self.peek, Token::RParen) && !matches!(self.cur, Token::EOF) {
            self.next_token()?;
            args.push(self.parse_expression(Precedence::Lowest)?);
            if matches!(self.peek, Token::Comman) {
                self.next_token()?;
            }
        }

        self.expect_peek(
            |t| matches!(t, Token::RParen),
            Error::FunctionError("Expected closing parentheses at function invocation".into()),
        )?;

        Ok(Expression::Invoked {
            invoked: Box::new(lhs),
            args,
        })
    }

    fn parse_function(&mut self) -> Result<Expression> {
        self.expect_peek(
            |t| matches!(t, Token::LParen),
            Error::FunctionError("Expected parentheses after `fn` keyword".into()),
        )?;

        let mut parameters = vec![];

        self.next_token()?;
        while let Token::Ident(name) = &self.cur {
            parameters.push(Ident { name: name.clone() });
            if matches!(self.peek, Token::Comman) {
                self.next_token()?;
            }
            self.next_token()?;
        }

        if !matches!(self.cur, Token::RParen) {
            return Err(Error::FunctionError(
                "Expected closing parentheses at function declaration".into(),
            ));
        }

        self.expect_peek(
            |t| matches!(t, Token::LBrace),
            Error::FunctionError("Expected function body".into()),
        )?;

        let body = Box::new(self.parse_block()?);

        Ok(Expression::Literal(Literal::Function { parameters, body }))
    }

    fn parse_if(&mut self) -> Result<Expression> {
        self.next_token()?;
        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);
        self.expect_peek(
            |t| matches!(t, Token::LBrace),
            Error::IfError("Expected expression block after condition".into()),
        )?;

        let consequence = Box::new(self.parse_block()?);

        if Token::Keyword(Keyword::Else) != self.peek {
            return Ok(Expression::If {
                condition,
                consequence,
                alternative: None,
            });
        }

        self.next_token()?;

        self.expect_peek(
            |t| matches!(t, Token::LBrace),
            Error::IfError("Expected expression block after else".into()),
        )?;

        let alternative = Some(Box::new(self.parse_block()?));

        Ok(Expression::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_block(&mut self) -> Result<Expression> {
        self.next_token()?;
        let mut statements = vec![];

        while self.cur != Token::RBrace && self.cur != Token::EOF {
            match self.parse_statement() {
                Ok(statement) => statements.push(statement),
                Err(e) => self.errors.push(e),
            };

            self.next_token()?;
        }

        Ok(Expression::Block { statements })
    }

    fn parse_grouped(&mut self) -> Result<Expression> {
        self.next_token()?;

        let expression = self.parse_expression(Precedence::Lowest)?;

        self.expect_peek(|t| matches!(t, Token::RParen), Error::ParseError)?;

        Ok(expression)
    }

    fn parse_infix_operator(&mut self, lhs: Expression) -> Result<Expression> {
        let precedence = self.cur_precedence();
        let operator = self.cur.clone();
        self.next_token()?;
        let lhs = Box::new(lhs);
        let rhs = Box::new(self.parse_expression(precedence)?);

        Ok(Expression::Infix { operator, lhs, rhs })
    }

    fn parse_int(&mut self) -> Result<Expression> {
        let Token::Int(value) = &self.cur else {
            unsafe { core::hint::unreachable_unchecked() }
        };

        let int = Expression::Literal(Literal::Int(*value));
        Ok(int)
    }

    fn parse_ident(&mut self) -> Result<Expression> {
        let Token::Ident(name) = &self.cur else {
            unsafe { core::hint::unreachable_unchecked() }
        };

        let ident = Expression::Ident(Ident { name: name.clone() });
        Ok(ident)
    }

    fn parse_bool(&mut self) -> Result<Expression> {
        let bool = if matches!(self.cur, Token::Keyword(Keyword::True)) {
            Bool::True
        } else {
            Bool::False
        };

        Ok(Expression::Literal(Literal::Bool(bool)))
    }

    fn expect_cur(&mut self, f: impl Fn(&Token) -> bool, e: Error) -> Result<()> {
        if !f(&self.cur) {
            return Err(e);
        }

        Ok(())
    }

    fn expect_peek(&mut self, f: impl Fn(&Token) -> bool, e: Error) -> Result<()> {
        if !f(&self.peek) {
            return Err(e);
        }

        self.next_token()?;

        Ok(())
    }

    fn peek_precedence(&self) -> Precedence {
        Self::precendence(&self.peek)
    }

    fn cur_precedence(&self) -> Precedence {
        Self::precendence(&self.cur)
    }

    fn precendence(t: &Token) -> Precedence {
        match t {
            Token::LParen => Precedence::Invoke,
            Token::Operator(Operator::Divide) | Token::Operator(Operator::Multiply) => {
                Precedence::Product
            }
            Token::Operator(Operator::Plus) | Token::Operator(Operator::Minus) => Precedence::Sum,
            Token::Operator(Operator::Assign) => Precedence::Assign,
            Token::Operator(Operator::Equal) | Token::Operator(Operator::NotEqual) => {
                Precedence::Equals
            }
            Token::Operator(Operator::Less)
            | Token::Operator(Operator::LessOrEqual)
            | Token::Operator(Operator::Greater)
            | Token::Operator(Operator::GreaterOrEqual) => Precedence::LessGreater,
            Token::Semicolon
            | Token::Operator(_)
            | Token::Keyword(_)
            | Token::EOF
            | Token::Ident(_)
            | Token::Int(_)
            | Token::Comman
            | Token::LBrace
            | Token::RParen
            | Token::Illegal
            | Token::RBrace => Precedence::Lowest,
        }
    }
}

#[derive(Debug)]
pub enum Precedence {
    Lowest,
    Assign,      // x = ...
    Equals,      // x == y, x != y
    LessGreater, // x < y, x > y
    Sum,         // x + y, x - y
    Product,     // x * y, x / y
    Prefix,      // !x, -x
    Invoke,      // foo(x, y)
}

impl Precedence {
    fn int(&self) -> i32 {
        match self {
            Self::Lowest => 0,
            Self::Assign => 1,
            Self::Equals => 2,
            Self::LessGreater => 3,
            Self::Sum => 4,
            Self::Product => 5,
            Self::Prefix => 6,
            Self::Invoke => 7,
        }
    }
}

impl PartialEq for Precedence {
    fn eq(&self, other: &Self) -> bool {
        self.int().eq(&other.int())
    }
}

impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.int().cmp(&other.int()))
    }
}

#[cfg(test)]
mod test {
    use std::vec;

    use crate::ast::{Expression, Ident, LetStatement, Literal, Statement};

    use super::*;

    #[test]
    fn test_statements() {
        let input = r#"flix;
        50;
        let x = 0;
        let y = 1 + 2;
        let yes = !false;
        x = 3;
        let a = b = c = false;
        x = 1 * (2 + 3);
        x = {
            let y = 1 + 2;
            y
        };
        x = if x == y {
            let z = 1 + 2;
            z
        } else {
            y
        };
        let add = fn(a, b) {
            a + b
        };
        let hello = fn() {};
        let boo = fn(a) {};
        let moo = fn(a) {a}(1);
        let moo = boo(1 + 2);
        "#;

        let expected = vec![
            Statement::Expression(Expression::Ident(Ident {
                name: "flix".into(),
            })),
            Statement::Expression(Expression::Literal(Literal::Int(50))),
            Statement::Let(LetStatement {
                name: Ident { name: "x".into() },
                value: Expression::Literal(Literal::Int(0)),
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "y".into() },
                value: Expression::Infix {
                    operator: Token::Operator(Operator::Plus),
                    lhs: Box::new(Expression::Literal(Literal::Int(1))),
                    rhs: Box::new(Expression::Literal(Literal::Int(2))),
                },
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "yes".into() },
                value: Expression::Prefix {
                    operator: Token::Operator(Operator::Bang),
                    operand: Box::new(Expression::Literal(Literal::Bool(Bool::False))),
                },
            }),
            Statement::Expression(Expression::Infix {
                operator: Token::Operator(Operator::Assign),
                lhs: Box::new(Expression::Ident(Ident { name: "x".into() })),
                rhs: Box::new(Expression::Literal(Literal::Int(3))),
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "a".into() },
                value: Expression::Infix {
                    operator: Token::Operator(Operator::Assign),
                    lhs: Box::new(Expression::Infix {
                        operator: Token::Operator(Operator::Assign),
                        lhs: Box::new(Expression::Ident(Ident { name: "b".into() })),
                        rhs: Box::new(Expression::Ident(Ident { name: "c".into() })),
                    }),
                    rhs: Box::new(Expression::Literal(Literal::Bool(Bool::False))),
                },
            }),
            Statement::Expression(Expression::Infix {
                operator: Token::Operator(Operator::Assign),
                lhs: Box::new(Expression::Ident(Ident { name: "x".into() })),
                rhs: Box::new(Expression::Infix {
                    operator: Token::Operator(Operator::Multiply),
                    lhs: Box::new(Expression::Literal(Literal::Int(1))),
                    rhs: Box::new(Expression::Infix {
                        operator: Token::Operator(Operator::Plus),
                        lhs: Box::new(Expression::Literal(Literal::Int(2))),
                        rhs: Box::new(Expression::Literal(Literal::Int(3))),
                    }),
                }),
            }),
            Statement::Expression(Expression::Infix {
                operator: Token::Operator(Operator::Assign),
                lhs: Box::new(Expression::Ident(Ident { name: "x".into() })),
                rhs: Box::new(Expression::Block {
                    statements: vec![
                        Statement::Let(LetStatement {
                            name: Ident { name: "y".into() },
                            value: Expression::Infix {
                                operator: Token::Operator(Operator::Plus),
                                lhs: Box::new(Expression::Literal(Literal::Int(1))),
                                rhs: Box::new(Expression::Literal(Literal::Int(2))),
                            },
                        }),
                        Statement::Expression(Expression::Ident(Ident { name: "y".into() })),
                    ],
                }),
            }),
            Statement::Expression(Expression::Infix {
                operator: Token::Operator(Operator::Assign),
                lhs: Box::new(Expression::Ident(Ident { name: "x".into() })),
                rhs: Box::new(Expression::If {
                    condition: Box::new(Expression::Infix {
                        operator: Token::Operator(Operator::Equal),
                        lhs: Box::new(Expression::Ident(Ident { name: "x".into() })),
                        rhs: Box::new(Expression::Ident(Ident { name: "y".into() })),
                    }),
                    consequence: Box::new(Expression::Block {
                        statements: vec![
                            Statement::Let(LetStatement {
                                name: Ident { name: "z".into() },
                                value: Expression::Infix {
                                    operator: Token::Operator(Operator::Plus),
                                    lhs: Box::new(Expression::Literal(Literal::Int(1))),
                                    rhs: Box::new(Expression::Literal(Literal::Int(2))),
                                },
                            }),
                            Statement::Expression(Expression::Ident(Ident { name: "z".into() })),
                        ],
                    }),
                    alternative: Some(Box::new(Expression::Block {
                        statements: vec![Statement::Expression(Expression::Ident(Ident {
                            name: "y".into(),
                        }))],
                    })),
                }),
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "add".into() },
                value: Expression::Literal(Literal::Function {
                    parameters: vec![Ident { name: "a".into() }, Ident { name: "b".into() }],
                    body: Box::new(Expression::Block {
                        statements: vec![Statement::Expression(Expression::Infix {
                            operator: Token::Operator(Operator::Plus),
                            lhs: Box::new(Expression::Ident(Ident { name: "a".into() })),
                            rhs: Box::new(Expression::Ident(Ident { name: "b".into() })),
                        })],
                    }),
                }),
            }),
            Statement::Let(LetStatement {
                name: Ident {
                    name: "hello".into(),
                },
                value: Expression::Literal(Literal::Function {
                    parameters: vec![],
                    body: Box::new(Expression::Block { statements: vec![] }),
                }),
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "boo".into() },
                value: Expression::Literal(Literal::Function {
                    parameters: vec![Ident { name: "a".into() }],
                    body: Box::new(Expression::Block { statements: vec![] }),
                }),
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "moo".into() },
                value: Expression::Invoked {
                    invoked: Box::new(Expression::Literal(Literal::Function {
                        parameters: vec![Ident { name: "a".into() }],
                        body: Box::new(Expression::Block {
                            statements: vec![Statement::Expression(Expression::Ident(Ident {
                                name: "a".into(),
                            }))],
                        }),
                    })),
                    args: vec![Expression::Literal(Literal::Int(1))],
                },
            }),
            Statement::Let(LetStatement {
                name: Ident { name: "moo".into() },
                value: Expression::Invoked {
                    invoked: Box::new(Expression::Ident(Ident { name: "boo".into() })),
                    args: vec![Expression::Infix {
                        operator: Token::Operator(Operator::Plus),
                        lhs: Box::new(Expression::Literal(Literal::Int(1))),
                        rhs: Box::new(Expression::Literal(Literal::Int(2))),
                    }],
                },
            }),
        ];

        let errors = vec![];

        let lexer = Lexer::new(input);
        let mut sut = Parser::new(lexer).unwrap();
        let program = sut.parse_program();

        assert!(program.is_ok());

        let program = program.unwrap();

        assert_eq!(program.statements, expected);
        assert_eq!(program.errors, errors);
    }
}
