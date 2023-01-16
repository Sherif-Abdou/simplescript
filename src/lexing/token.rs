#[derive(Debug, PartialEq, Eq)]
pub enum Token {
  Identifier(String),
  Integer(i64),
  Def,
  OpenCurly,
  ClosedCurly,
  OpenParenth,
  CloseParenth,
  Plus,
  Minus,
  Star,
  Slash,
  Equal,
  EOL,
  EOF,
}