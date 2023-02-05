use crate::{lexing::{Lexer, Token}, ast::{Scope, Function}};
use std::{collections::VecDeque, error::Error, fmt::Display};

pub struct Parser {
  lexer: Lexer,
  current_token: Token,
  scope_stack: VecDeque<Box<dyn Scope>>
}

pub type ParsingResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum ParsingError {
  MissingToken
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "Error")
    }
}

impl Error for ParsingError {}

impl Parser {
  pub fn new(raw: String) -> Self {
    let mut lexer = Lexer::new(raw);
    Self {
      scope_stack: VecDeque::new(),
      current_token: lexer.next(),
      lexer,
    }
  }

  pub fn parse(&mut self) -> ParsingResult<()> {
    while self.current_token != Token::EOF {
      if self.current_token == Token::Def {
        self.parse_function()?
      }
    }

    Ok(())
  }


  fn parse_function(&mut self) -> ParsingResult<()> {
    if self.current_token != Token::Def {
      return Err(Box::new(ParsingError::MissingToken));
    }

    let mut func_name = String::new();

    {
      let Token::Identifier(fn_name) = self.next().clone() else {
        return Err(Box::new(ParsingError::MissingToken))
      };

      func_name = fn_name.clone();
    }
    
    if Token::OpenParenth != *self.next() {
      return Err(Box::new(ParsingError::MissingToken))
    };

    let Token::CloseParenth = self.next() else {
      return Err(Box::new(ParsingError::MissingToken))
    };

    let Token::OpenCurly = self.next() else {
      return Err(Box::new(ParsingError::MissingToken));
    };

    let mut function = Function::default();
    function.name = func_name.to_string();
    self.scope_stack.push_front(Box::new(function));

    Ok(())
  }

  fn next(&mut self) -> &Token {
    self.current_token = self.lexer.next();
    &self.current_token
  }
  
  fn top_scope(&mut self) -> &mut dyn Scope {
    return &mut *self.scope_stack[0];
  }
}