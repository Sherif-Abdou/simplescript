use super::Token;

pub struct Lexer {
  raw_text: String,
}

impl Lexer {
  pub fn new(raw_text: String) -> Self {
    Self {
      raw_text
    }
  }

  fn empty(&self) -> bool {
    self.raw_text.is_empty()
  }

  pub fn next(&mut self) -> Token {
    if self.empty() {
      return Token::EOF;
    }
    let mut current_string = String::new();
    let mut current: char = self.peek();
    while current.is_whitespace() && current != '\n' && !self.empty() {
      self.pop();
      current = self.peek();
    }
    if current == '\n' {
      self.pop();
      return Token::EOL;
    }

    let sc_token = match current {
      '+' => Some(Token::Plus),
      '-' => Some(Token::Minus),
      '*' => Some(Token::Star),
      '/' => Some(Token::Slash),
      '(' => Some(Token::OpenParenth),
      ')' => Some(Token::CloseParenth),
      '{' => Some(Token::OpenCurly),
      '}' => Some(Token::ClosedCurly),
      '=' => Some(Token::Equal),
      _ => None
    };

    if let Some(token) = sc_token {
      self.pop();
      return token;
    }

    if current.is_numeric() {
      while current.is_numeric() && !self.empty() {
        current_string.push(self.pop());
        current = self.peek();
      }
      return Token::Integer(current_string.parse().unwrap());
    }

    if current.is_alphabetic() {
      while current.is_alphanumeric() && !self.empty() {
        current_string.push(self.pop());
        current = self.peek();
      }
      return match current_string.as_str() {
          "def" => Token::Def,
          _ => Token::Identifier(current_string)
      }
    }
    return Token::EOL;
  }

  fn pop(&mut self) -> char {
    self.raw_text.remove(0)
  }

  fn peek(&self) -> char {
    self.raw_text.chars().next().unwrap()
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use Token::*;
  #[test]
  fn test_parser() {
    let raw = "def hello() {\n2 + 3\n}".to_string();

    let mut lexer = Lexer::new(raw);
    let expected_tokens = &[Token::Def, Token::Identifier("hello".into()),
      OpenParenth, CloseParenth, OpenCurly, EOL, Integer(2), Plus, Integer(3), EOL, ClosedCurly];

    for expected in expected_tokens {
      assert_eq!(lexer.next(), *expected);
    }
  }
}