use crate::{lexing::{Lexer, Token}, ast::{Scope, Function, Expression, SetVariable}};
use std::{collections::VecDeque, error::Error, fmt::Display};

use super::{scope_stark::ScopeStack, expression_parser::ExpressionParser};

pub struct Parser {
  lexer: Lexer,
  current_token: Token,
  scope_stack: ScopeStack
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
      scope_stack: ScopeStack::default(),
      current_token: lexer.next(),
      lexer,
    }
  }

  pub fn parse(&mut self) -> ParsingResult<()> {
    while self.current_token != Token::EOF {
      if self.current_token == Token::Def {
        self.parse_function()?
      } else if let Token::Identifier(iden) = self.current_token.clone() {
        self.parse_set_variable(&iden)?;
      }
    }

    Ok(())
  }

  fn parse_expression(&mut self) -> ParsingResult<Expression> {
    let mut expr_parser = ExpressionParser::new();
    while self.current_token != Token::EOL {
      expr_parser.consume(self.current_token.clone())?;
      self.next();
    }

    Ok(expr_parser.build())
  }

  fn parse_set_variable(&mut self, iden: &str) -> ParsingResult<()> {
    if *self.next() != Token::Equal {
      return Err(Box::new(ParsingError::MissingToken));
    }
    self.next();
    let expr = self.parse_expression()?;
    let stmt = SetVariable::new(iden.to_string(), expr);
    self.scope_stack.commands_mut().push(Box::new(stmt));
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
}