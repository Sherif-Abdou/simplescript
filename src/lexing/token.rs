#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Identifier(String),
    String(String),
    Char(u8),
    Integer(i64),
    Float(f64),
    Def,
    As,
    Struct,
    While,
    Extern,
    Variadic,
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
    Dot,
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
    Arrow,
    EOL,
    EOF,
}
