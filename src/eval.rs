use std::{collections::HashMap, sync::Arc};

use crate::{
    ast::{self, Expression, Ident, LetStatement, Literal, Node, ReturnStatement, Statement},
    object::{
        self, Builtin, Collection, Function, Integer, ObjectType, Reference, Str, Unit, Vector,
    },
    stack::Stack,
    token::{Operator, Token},
};

use error::{Error, Result};

pub mod error {
    use super::ops::Flow;

    pub type Result<T> = std::result::Result<Flow<T>, Error>;

    #[derive(Debug, Clone)]
    pub enum Error {
        Eval(String),
    }
}

pub mod ops {
    use std::fmt::{Display, Formatter, Result};

    pub enum Flow<T> {
        Continue(T),
        Break(T),
    }

    impl<T> std::ops::Deref for Flow<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            match self {
                Self::Continue(ref t) => t,
                Self::Break(ref t) => t,
            }
        }
    }

    impl<T> std::ops::DerefMut for Flow<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            match self {
                Self::Continue(ref mut t) => t,
                Self::Break(ref mut t) => t,
            }
        }
    }

    impl<T: Display> Display for Flow<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            f.write_fmt(format_args!("{}", self.as_ref().unwrap()))
        }
    }

    impl<T> Flow<T> {
        pub fn unwrap(self) -> T {
            match self {
                Self::Continue(t) => t,
                Self::Break(t) => t,
            }
        }

        pub fn is_break(&self) -> bool {
            matches!(self, Self::Break(_))
        }

        pub fn is_continue(&self) -> bool {
            matches!(self, Self::Continue(_))
        }

        pub fn as_ref(&self) -> Flow<&T> {
            match self {
                Self::Continue(ref t) => Flow::Continue(t),
                Self::Break(ref t) => Flow::Break(t),
            }
        }

        pub fn map<F, U>(self, f: F) -> Flow<U>
        where
            F: Fn(T) -> U,
        {
            match self {
                Self::Continue(t) => Flow::Continue(f(t)),
                Self::Break(t) => Flow::Break(f(t)),
            }
        }
    }
}

use ops::Flow;

#[derive(Debug)]
pub struct Eval {
    stack: Stack,
}

impl Eval {
    pub fn new() -> Self {
        Self {
            stack: Stack::new(),
        }
    }

    pub fn clear(&mut self) {
        self.stack = Stack::new();
    }

    pub fn eval(&mut self, node: Node) -> Result<Reference> {
        let ret = match node {
            Node::Statement(Statement::Expression(e)) => self.eval(Node::Expression(e))?,
            Node::Expression(Expression::Literal(Literal::Int(val))) => {
                Flow::Continue(Integer::erased(val))
            }
            Node::Expression(Expression::Literal(Literal::Bool(b))) => match b {
                ast::Bool::True => Flow::Continue(object::Bool::erased(true)),
                ast::Bool::False => Flow::Continue(object::Bool::erased(false)),
            },
            Node::Expression(Expression::Literal(Literal::String(str))) => {
                Flow::Continue(Str::erased(str))
            }
            Node::Expression(Expression::Ident(Ident { name })) => {
                let val = self
                    .stack
                    .get(&name)
                    .map(|a| a.clone())
                    .ok_or(Error::Eval(format!("Variable {} not found in scope", name)))?;

                Flow::Continue(val)
            }
            Node::Expression(Expression::Prefix { operator, operand }) => {
                self.eval_prefix(operator, *operand)?
            }
            Node::Expression(Expression::Infix { operator, lhs, rhs }) => {
                self.eval_infix(operator, *lhs, *rhs)?
            }
            Node::Expression(Expression::If {
                condition,
                consequence,
                alternative,
            }) => self.eval_if(*condition, *consequence, alternative.map(|b| *b))?,
            Node::Expression(Expression::Block { statements }) => {
                self.stack.push();
                let ret = self.eval_statements(statements)?;
                self.stack.pop();
                ret
            }
            Node::Expression(Expression::Program(pro)) => self.eval_statements(pro.statements)?,
            Node::Statement(Statement::Return(ReturnStatement { value })) => {
                let ret = self.eval(Node::Expression(value))?;
                Flow::Break(ret.unwrap())
            }
            Node::Statement(Statement::Let(LetStatement { name, value })) => self.eval_assign(
                Token::Operator(Operator::Assign),
                Expression::Ident(name),
                value,
            )?,
            Node::Expression(Expression::Invoked { invoked, args }) => {
                self.eval_invoke(*invoked, args)?
            }
            Node::Expression(Expression::Literal(Literal::Function {
                parameters,
                body,
                capture,
            })) => self.eval_function(parameters, *body, capture)?,
            Node::Expression(Expression::Literal(Literal::Collection { members })) => {
                Flow::Continue(Collection::erased(members.into_iter().try_fold(
                    HashMap::new(),
                    |mut members, (ident, exp)| {
                        self.eval(Node::Expression(exp)).map(|member| {
                            members.insert(ident, member.clone());
                            members
                        })
                    },
                )?))
            }
            Node::Expression(Expression::Literal(Literal::Vector { elements })) => Flow::Continue(
                Vector::erased(elements.into_iter().try_fold(vec![], |mut elements, exp| {
                    self.eval(Node::Expression(exp)).map(|reference| {
                        elements.push(reference.unwrap());
                        elements
                    })
                })?),
            ),
            Node::Expression(Expression::Indexed { indexee, index }) => {
                self.eval_index(*indexee, *index)?
            }
            _ => todo!(),
        };

        Ok(ret)
    }
    fn eval_function(
        &mut self,
        parameters: Vec<Ident>,
        body: Expression,
        capture: Vec<Ident>,
    ) -> Result<Reference> {
        let capture = capture
            .into_iter()
            .try_fold(HashMap::new(), |mut map, ident| {
                match self.stack.get(&ident.name) {
                    Some(value) => {
                        map.insert(ident, value.clone());
                        Ok(map)
                    }
                    None => Err(Error::Eval(format!(
                        "Attempting to capture unknown variable {} in function decleration.",
                        ident.name
                    ))),
                }
            })?;

        Ok(Flow::Continue(Function::erased(parameters, body, capture)))
    }

    fn eval_index(&mut self, indexee: Expression, index: Expression) -> Result<Reference> {
        let index = self.eval(Node::Expression(index))?.unwrap();
        let indexee = self.eval(Node::Expression(indexee))?.unwrap();

        let obj = indexee
            .v_table()
            .get("idx")
            .ok_or(Error::Eval("Object does not support indexing".into()))?(Some(
            index,
        ))
        .ok_or(Error::Eval(
            "Indexing not supported with this object.".into(),
        ))?;

        Ok(Flow::Continue(obj))
    }

    fn eval_invoke(&mut self, invoked: Expression, args: Vec<Expression>) -> Result<Reference> {
        let function = self.eval(Node::Expression(invoked))?.unwrap();

        let args: Vec<Reference> =
            args.into_iter().try_fold(vec![], |mut args, arg| {
                match self.eval(Node::Expression(arg)) {
                    Err(e) => Err(e),
                    Ok(arg) => {
                        args.push(arg.unwrap());
                        Ok(args)
                    }
                }
            })?;

        if matches!(function.r#type(), ObjectType::Builtin) {
            let builtin = unsafe { function.get_mut::<Builtin>() };
            return builtin.call(args);
        }

        if !matches!(function.r#type(), ObjectType::Function) {
            return Err(Error::Eval(format!(
                "Inovking non-function types is not supported",
            )));
        }

        let function = unsafe { function.get_mut::<Function>() };

        if function.parameters.len() != args.len() {
            return Err(Error::Eval(format!(
                "Incorrect number of arguments passed for invocation",
            )));
        }

        self.stack.push_frame();

        for (ident, arg) in function.parameters.iter().zip(args.into_iter()) {
            self.stack.add(ident.name.clone(), arg);
        }

        for (ident, captured) in &function.capture {
            self.stack.add(ident.name.clone(), captured.clone());
        }

        let ret = self.eval(Node::Expression(function.body.clone()));

        self.stack.pop_frame();

        ret
    }

    fn eval_statements(&mut self, statements: Vec<Statement>) -> Result<Reference> {
        let mut ret = Flow::Continue(Unit::erased());
        for st in statements {
            ret = match self.eval(Node::Statement(st))? {
                f @ Flow::Continue(_) => f,
                f @ Flow::Break(_) => {
                    return Ok(f);
                }
            };
        }

        Ok(ret)
    }

    fn eval_prefix(&mut self, operator: Token, operand: Expression) -> Result<Reference> {
        let mut operand = self.eval(Node::Expression(operand))?;
        if operand.is_break() {
            return Ok(operand);
        };

        let err = Error::Eval(format!(
            "Unsupported operator {:?} for operand type {}",
            operator, operand
        ));

        match operator {
            Token::Operator(Operator::Bang) => {
                operand = Flow::Continue(
                    (operand.v_table().get("inv").ok_or(err.clone())?)(None).ok_or(err.clone())?,
                );
            }
            Token::Operator(Operator::Minus) => {
                operand = Flow::Continue(
                    (operand.v_table().get("neg").ok_or(err.clone())?)(None).ok_or(err.clone())?,
                );
            }
            _ => unsafe { core::hint::unreachable_unchecked() },
        }

        Ok(operand)
    }

    fn eval_access(
        &mut self,
        _operator: Token,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Reference> {
        let collection = match lhs {
            c @ Expression::Literal(Literal::Collection { .. }) => {
                self.eval(Node::Expression(c))?.unwrap()
            }
            Expression::Ident(Ident { name }) => {
                let c = self.stack.get(&name).ok_or(Error::Eval(format!(
                    "Cannot find {} in the current scope.",
                    name
                )))?;

                c
            }
            _ => {
                return Err(Error::Eval(format!(
                    "Accessing non-collection types is not supported."
                )))
            }
        };

        if !matches!(collection.r#type(), ObjectType::Collection) {
            return Err(Error::Eval(format!(
                "Accessing non-collection types is not supported",
            )));
        }

        let members = unsafe { collection.get_mut::<Collection>().members.clone() };

        let ident = match rhs {
            Expression::Ident(i) => i,
            _ => {
                let rhs = self.eval(Node::Expression(rhs))?;
                return Err(Error::Eval(format!(
                    "Exprected identifier as accessor {} instead",
                    rhs
                )));
            }
        };

        members
            .get(&ident)
            .map(|mem| Flow::Continue(mem.clone()))
            .ok_or(Error::Eval(format!(
                "Collection does not contain the member {}.",
                ident.name
            )))
    }

    fn eval_access_assign(
        &mut self,
        _operator: Token,
        collection: Expression,
        accessor: Expression,
        rhs: Expression,
    ) -> Result<Reference> {
        let collection = self.eval(Node::Expression(collection))?.unwrap();
        let ident = match accessor {
            Expression::Ident(i) => i,
            _ => {
                return Err(Error::Eval(
                    "Collection can only be accessed via an ident.".into(),
                ))
            }
        };

        if !matches!(collection.r#type(), ObjectType::Collection) {
            return Err(Error::Eval("Only collections can be accessed.".into()));
        }

        let rhs = self.eval(Node::Expression(rhs))?;
        if rhs.is_break() {
            return Ok(Flow::Break(rhs.unwrap()));
        };

        let collection = unsafe { collection.get_mut::<Collection>() };

        let mut map = (*collection.members).clone();

        map.insert(ident, rhs.clone());

        collection.members = Arc::new(map);

        Ok(rhs)
    }

    fn eval_assign(
        &mut self,
        operator: Token,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Reference> {
        let ident = match lhs {
            Expression::Ident(Ident { name }) => name,
            Expression::Infix {
                operator: Token::Operator(Operator::Dot),
                lhs: collection,
                rhs: accessor,
                ..
            } => return self.eval_access_assign(operator, *collection, *accessor, rhs),
            _ => {
                let lhs = self.eval(Node::Expression(lhs))?;
                return Err(Error::Eval(format!(
                    "Exprected identifier in assignment got {} instead",
                    lhs
                )));
            }
        };
        let rhs = self.eval(Node::Expression(rhs))?;
        if rhs.is_break() {
            return Ok(Flow::Break(rhs.unwrap()));
        };

        let err = Error::Eval(format!(
            "Unsupported operator {:?} for operand types {} and {}",
            operator, ident, rhs
        ));

        let rhs = match operator {
            Token::Operator(op @ Operator::MinusEqual | op @ Operator::PlusEqual) => {
                let op = match op {
                    Operator::PlusEqual => "add_lhs",
                    Operator::MinusEqual => "sub_lhs",
                    _ => unsafe { core::hint::unreachable_unchecked() },
                };

                let lhs = self.stack.get(&ident).ok_or(Error::Eval(format!(
                    "Identifier {} not found in scope",
                    ident
                )))?;

                let op = lhs.v_table().get(op).ok_or(err.clone())?;

                op(Some(rhs.unwrap()))
                    .map(|op| Flow::Continue(op))
                    .ok_or(err)?
            }
            _ => rhs,
        };

        self.stack
            .assign(ident, rhs.as_ref().map(|t| t.clone()).unwrap());

        Ok(rhs)
    }

    fn eval_infix(
        &mut self,
        operator: Token,
        lhs: Expression,
        rhs: Expression,
    ) -> Result<Reference> {
        if matches!(
            operator,
            Token::Operator(Operator::Assign)
                | Token::Operator(Operator::PlusEqual)
                | Token::Operator(Operator::MinusEqual)
        ) {
            return self.eval_assign(operator, lhs, rhs);
        }

        if matches!(operator, Token::Operator(Operator::Dot)) {
            return self.eval_access(operator, lhs, rhs);
        }

        let lhs = self.eval(Node::Expression(lhs))?;
        if lhs.is_break() {
            return Ok(lhs);
        }
        let rhs = self.eval(Node::Expression(rhs))?;
        if rhs.is_break() {
            return Ok(rhs);
        }
        let err = Error::Eval(format!(
            "Unsupported operator {:?} for operand types {} and {}",
            operator, lhs, rhs
        ));

        let op = match operator {
            Token::Operator(Operator::Minus) => "sub_lhs",
            Token::Operator(Operator::MinusEqual) => "sub_lhs",
            Token::Operator(Operator::Plus) => "add_lhs",
            Token::Operator(Operator::PlusEqual) => "add_lhs",
            Token::Operator(Operator::Multiply) => "mul_lhs",
            Token::Operator(Operator::Divide) => "div_lhs",
            Token::Operator(Operator::Equal) => "eq_lhs",
            Token::Operator(Operator::NotEqual) => "neq_lhs",
            Token::Operator(Operator::Less) => "le_lhs",
            Token::Operator(Operator::LessOrEqual) => "leq_lhs",
            Token::Operator(Operator::Greater) => "ge_lhs",
            Token::Operator(Operator::GreaterOrEqual) => "geq_lhs",
            Token::Operator(Operator::Ampersand) => "ins_lhs",
            Token::Operator(Operator::Pipe) => "uni_lhs",
            _ => Err(Error::Eval("Infix operator is not supported".into()))?,
        };

        let sub = lhs.v_table().get(op).ok_or(err.clone())?;

        sub(Some(rhs.unwrap()))
            .map(|op| Flow::Continue(op))
            .ok_or(err)
    }

    fn eval_if(
        &mut self,
        condition: Expression,
        consequence: Expression,
        alternative: Option<Expression>,
    ) -> Result<Reference> {
        let cond = self.eval(Node::Expression(condition))?;
        let err = Error::Eval(format!(
            "Condition type of {} is not fit for conditions.",
            cond
        ));
        if cond.is_break() {
            return Ok(cond);
        }
        let cond_fn = cond.v_table().get("truthy").ok_or(err)?;

        if cond_fn(None).is_some() {
            return self.eval(Node::Expression(consequence));
        }

        if let Some(alt) = alternative {
            return self.eval(Node::Expression(alt));
        }

        Ok(Flow::Continue(Unit::erased()))
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;

    #[test]
    fn test_integer_comp() {
        let input = r#"
            5;
            10;
            4 * (10 + 2);
            "#;

        let mut p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap()
            .statements
            .into_iter();

        let mut r = Eval::new();

        let mut e = r.eval(Node::Statement(p.next().unwrap()));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "5");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "10");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "48");
        }

        let p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap();

        r.clear();

        let e = r.eval(Node::Expression(Expression::Program(p)));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "48");
        }
    }

    #[test]
    fn test_boolean_comp() {
        let input = r#"
            true == true;
            4 < 10;
            (5 >= 8) == true;
            false == (3 > 20);
            "#;

        let mut p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap()
            .statements
            .into_iter();

        let mut r = Eval::new();

        let mut e = r.eval(Node::Statement(p.next().unwrap()));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "true");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "true");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "false");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "true");
        }

        let p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap();

        r.clear();

        let e = r.eval(Node::Expression(Expression::Program(p)));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "true");
        }
    }

    #[test]
    fn test_if() {
        let input = r#"
            if 1 > 10 { 5 } else { 10 * 8 };
            if 4 == 3 { 4 };
            "#;

        let mut p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap()
            .statements
            .into_iter();

        let mut r = Eval::new();

        let mut e = r.eval(Node::Statement(p.next().unwrap()));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "80");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "null");
        }

        let p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap();

        r.clear();

        let e = r.eval(Node::Expression(Expression::Program(p)));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "null");
        }
    }

    #[test]
    fn test_let() {
        let input = r#"
            let a = 4;
            a;
            "#;

        let mut p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap()
            .statements
            .into_iter();

        let mut r = Eval::new();

        let mut e = r.eval(Node::Statement(p.next().unwrap()));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "4");

            e = r.eval(Node::Statement(p.next().unwrap()));

            assert_eq!(format!("{}", e.unwrap_unchecked()), "4");
        }

        let p = Parser::new(Lexer::new(input))
            .unwrap()
            .parse_program()
            .unwrap();

        r.clear();

        let e = r.eval(Node::Expression(Expression::Program(p)));

        unsafe {
            assert_eq!(format!("{}", e.unwrap_unchecked()), "4");
        }
    }
}
