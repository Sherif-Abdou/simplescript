#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    String(String),
    Char(u8),
    Integer(i64),
    Float(f64),
    Def,
    As,
    Return,
    OpenCurly,
    ClosedCurly,
    OpenSquare,
    CloseSquare,
    OpenParenth,
    CloseParenth,
    Plus,
    Minus,
    Star,
    Slash,
    Ampersand,
    Greater,
    Lesser,
    GreaterEqual,
    LesserEqual,
    DoubleEqual,
    NotEqual,
    If,
    Else,
    Equal,
    Colon,
    Comma,
    EOL,
    EOF,
}