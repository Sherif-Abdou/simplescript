#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
  Identifier(String),
  Integer(i64),
  Def,
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
  Equal,
  Colon,
  Comma,
  EOL,
  EOF,
}