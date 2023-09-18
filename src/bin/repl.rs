use std::io::{self, Write};

use monkey::{
    ast::{Expression, Node},
    eval::Eval,
};

fn main() {
    loop {
        print!(">> ");
        std::io::stdout().flush().unwrap();
        let mut input = "".to_string();
        io::stdin().read_line(&mut input).unwrap();

        let l = monkey::lexer::Lexer::new(&input);
        let mut p = match monkey::parser::Parser::new(l) {
            Ok(p) => p,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let pro = match p.parse_program() {
            Ok(pro) => pro,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        let mut runtime = Eval::new();

        let out = match runtime.eval(Node::Expression(Expression::Program(pro))) {
            Ok(obj) => obj,
            Err(e) => {
                println!("{:?}", e);
                return;
            }
        };

        println!("{out}");
    }
}
