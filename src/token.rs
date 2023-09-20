#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Illegal,
    EOF,
    Keyword(Keyword),
    Ident(String),
    Int(i32),
    Str(String),
    Operator(Operator),
    Comma,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    Assign,
    Plus,
    Minus,
    Divide,
    Multiply,
    PlusEqual,
    MinusEqual,
    Dot,
    Bang,
    Hook,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    Equal,
    NotEqual,
    And,
    Or,
    Ampersand,
    Pipe,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Function,
    Def,
    If,
    Else,
    True,
    False,
    Return,
}

impl Keyword {
    fn match_ident(ident: impl AsRef<str>) -> Result<Self, ()> {
        match ident.as_ref() {
            "let" => Ok(Self::Let),
            "fn" => Ok(Self::Function),
            "def" => Ok(Self::Def),
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            "if" => Ok(Self::If),
            "else" => Ok(Self::Else),
            "return" => Ok(Self::Return),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for Keyword {
    type Error = ();
    fn try_from(ident: &str) -> Result<Self, Self::Error> {
        Self::match_ident(ident)
    }
}

impl TryFrom<&String> for Keyword {
    type Error = ();
    fn try_from(ident: &String) -> Result<Self, Self::Error> {
        Self::match_ident(ident)
    }
}
