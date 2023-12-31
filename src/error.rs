pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ParseError,
    LetStatement(String),
    Unsupported(String),
    Args(String),
    Block(String),
    IfError(String),
    FunctionError(String),
    Collection(String),
}
