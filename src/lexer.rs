use crate::error::Result;
use crate::token::{Keyword, Operator, Token};

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    c: char,
}

#[allow(dead_code)]
impl Lexer {
    pub fn new(input: impl AsRef<str>) -> Self {
        let mut l = Self {
            input: input.as_ref().chars().collect(),
            ..Default::default()
        };

        l.read_char();

        l
    }

    fn read_single_token(&mut self) -> Token {
        match self.c {
            '+' => Token::Operator(Operator::Plus),
            '-' => Token::Operator(Operator::Minus),
            '/' => Token::Operator(Operator::Divide),
            '*' => Token::Operator(Operator::Multiply),
            '<' => Token::Operator(Operator::Less),
            '>' => Token::Operator(Operator::Greater),
            '=' => Token::Operator(Operator::Assign),
            '.' => Token::Operator(Operator::Dot),
            '!' => Token::Operator(Operator::Bang),
            '?' => Token::Operator(Operator::Hook),
            '&' => Token::Operator(Operator::Ampersand),
            '|' => Token::Operator(Operator::Pipe),
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ';' => Token::Semicolon,
            ',' => Token::Comma,
            _ => Token::Illegal,
        }
    }

    fn peek_next(&self) -> char {
        if self.read_position >= self.input.len() {
            return '\0';
        }

        self.input[self.read_position]
    }

    fn read_double_token(&mut self) -> Token {
        if self.position + 1 > self.input.len() {
            return self.read_single_token();
        }

        let token = match self.peek_next() {
            '=' => match self.c {
                '!' => Token::Operator(Operator::NotEqual),
                '=' => Token::Operator(Operator::Equal),
                '>' => Token::Operator(Operator::GreaterOrEqual),
                '<' => Token::Operator(Operator::LessOrEqual),
                '+' => Token::Operator(Operator::PlusEqual),
                '-' => Token::Operator(Operator::MinusEqual),
                _ => return self.read_single_token(),
            },
            '>' if self.c == '-' => Token::Operator(Operator::RightArrow),
            '-' if self.c == '<' => Token::Operator(Operator::LeftArrow),
            '&' if self.c == '&' => Token::Operator(Operator::And),
            '|' if self.c == '|' => Token::Operator(Operator::Or),
            _ => return self.read_single_token(),
        };

        self.read_char();

        token
    }

    pub fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let token = match self.c {
            '=' | '!' | '-' | '+' | '&' | '|' | '<' | '>' => self.read_double_token(),
            '/' | '*' | '.' | '?' | '{' | '}' | '(' | ')' | '[' | ']' | ';' | ',' => {
                self.read_single_token()
            }
            '"' => return Ok(Token::Str(self.read_string())),
            '\0' => Token::EOF,
            // Parse idents and keywords.
            // Needs an early return as `read_ident` calls `read_char`.
            _ if self.is_letter() => {
                let ident = self.read_ident();

                if let Ok(keyword) = Keyword::try_from(&ident) {
                    return Ok(Token::Keyword(keyword));
                }

                return Ok(Token::Ident(ident));
            }
            // Parse integer literals.
            // Needs early return for the same reason.
            _ if self.is_integer() => {
                return Ok(Token::Int(self.read_integer()));
            }
            _ => Token::Illegal,
        };

        self.read_char();

        Ok(token)
    }

    fn read_string(&mut self) -> String {
        self.read_char();
        let start_position = self.position;

        while self.c != '"' && self.c != '\0' {
            self.read_char();
        }

        let ret = self.input[start_position..self.position].iter().collect();

        self.read_char();

        ret
    }

    fn is_whitespace(&self) -> bool {
        match self.c {
            ' ' | '\t' | '\n' | '\r' => true,
            _ => false,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.is_whitespace() {
            self.read_char();
        }
    }

    fn read_ident(&mut self) -> String {
        let start_position = self.position;

        while self.is_letter() {
            self.read_char()
        }

        self.input[start_position..self.position].iter().collect()
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.c = '\0';
        } else {
            self.c = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn is_letter(&self) -> bool {
        ('a' <= self.c && self.c <= 'z') || ('A' <= self.c && self.c <= 'Z') || self.c == '_'
    }

    fn read_integer(&mut self) -> i32 {
        let starting_position = self.position;

        while self.is_integer() {
            self.read_char();
        }

        // Ohhh boy.. this is unsound as heck, huh?
        self.input[starting_position..self.position]
            .iter()
            .collect::<String>()
            .parse::<i32>()
            .unwrap()
    }

    fn is_integer(&self) -> bool {
        '0' <= self.c && self.c <= '9'
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token().ok()
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::Lexer, token::Operator};

    #[test]
    fn test_next_token() {
        use crate::token::Keyword;
        use crate::token::Token;

        let input = r#"let five = 5;
        let ten = 10;

        let add = fn(x, y) {
            x + y;
        };

        let result = add(five, ten);

        if result != 15 || five < 7 && ten > 11 {
            return false;
        } else {
            return true;
        }
        !-/*5;
        let a = "hello there";
        a -= "2";
        "#;

        let tests = vec![
            Token::Keyword(Keyword::Let),
            Token::Ident("five".into()),
            Token::Operator(Operator::Assign),
            Token::Int(5),
            Token::Semicolon,
            Token::Keyword(Keyword::Let),
            Token::Ident("ten".into()),
            Token::Operator(Operator::Assign),
            Token::Int(10),
            Token::Semicolon,
            Token::Keyword(Keyword::Let),
            Token::Ident("add".into()),
            Token::Operator(Operator::Assign),
            Token::Keyword(Keyword::Function),
            Token::LParen,
            Token::Ident("x".into()),
            Token::Comma,
            Token::Ident("y".into()),
            Token::RParen,
            Token::LBrace,
            Token::Ident("x".into()),
            Token::Operator(Operator::Plus),
            Token::Ident("y".into()),
            Token::Semicolon,
            Token::RBrace,
            Token::Semicolon,
            Token::Keyword(Keyword::Let),
            Token::Ident("result".into()),
            Token::Operator(Operator::Assign),
            Token::Ident("add".into()),
            Token::LParen,
            Token::Ident("five".into()),
            Token::Comma,
            Token::Ident("ten".into()),
            Token::RParen,
            Token::Semicolon,
            Token::Keyword(Keyword::If),
            Token::Ident("result".into()),
            Token::Operator(Operator::NotEqual),
            Token::Int(15),
            Token::Operator(Operator::Or),
            Token::Ident("five".into()),
            Token::Operator(Operator::Less),
            Token::Int(7),
            Token::Operator(Operator::And),
            Token::Ident("ten".into()),
            Token::Operator(Operator::Greater),
            Token::Int(11),
            Token::LBrace,
            Token::Keyword(Keyword::Return),
            Token::Keyword(Keyword::False),
            Token::Semicolon,
            Token::RBrace,
            Token::Keyword(Keyword::Else),
            Token::LBrace,
            Token::Keyword(Keyword::Return),
            Token::Keyword(Keyword::True),
            Token::Semicolon,
            Token::RBrace,
            Token::Operator(Operator::Bang),
            Token::Operator(Operator::Minus),
            Token::Operator(Operator::Divide),
            Token::Operator(Operator::Multiply),
            Token::Int(5),
            Token::Semicolon,
            Token::Keyword(Keyword::Let),
            Token::Ident("a".into()),
            Token::Operator(Operator::Assign),
            Token::Str("hello there".into()),
            Token::Semicolon,
            Token::Ident("a".into()),
            Token::Operator(Operator::MinusEqual),
            Token::Str("2".into()),
            Token::Semicolon,
            Token::EOF,
        ];

        let mut sut = Lexer::new(input);

        for tc in tests {
            assert_eq!(sut.next_token().unwrap(), tc);
        }
    }
}
