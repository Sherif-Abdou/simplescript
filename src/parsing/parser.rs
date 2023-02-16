use crate::{lexing::{Lexer, Token}, ast::{Scope, Function, Expression, SetVariable, Statement, ReturnCommand, Variable, DataType}};
use std::{collections::{VecDeque, HashMap}, error::Error, fmt::Display, cell::RefCell};

use super::{scope_stack::ScopeStack, expression_parser::ExpressionParser};

pub struct Parser {
  lexer: RefCell<Lexer>,
  current_token: RefCell<Token>,
  scope_stack: ScopeStack,
  data_types: HashMap<String, DataType>
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
    let mut data_types = HashMap::new();
    data_types.insert("i64".to_string(), DataType {
        symbol: "i64".to_string(),
        value: crate::ast::DataTypeEnum::Primitive,
    });
    Self {
      scope_stack: ScopeStack::default(),
      current_token: RefCell::new(lexer.next()),
      lexer: RefCell::new(lexer),
      data_types,
    }
  }

  pub fn parse(&mut self) -> ParsingResult<Box<dyn Scope>> {
    while self.current_token() != Token::EOF {
      if self.current_token() == Token::Def {
        self.parse_function()?
      } else if let Token::Identifier(iden) = self.current_token().clone() {
        self.parse_set_variable(&iden)?;
      } else if self.current_token() == Token::Return {
        self.parse_return()?;
      }
      self.next();
    }

    Ok(self.scope_stack.pop_front().unwrap())
  }

  fn parse_return(&mut self) -> ParsingResult<()> {
    if self.current_token() != Token::Return {
      return Err(Box::new(ParsingError::MissingToken));
    }
    self.next();
    // // dbg!("Did return");
    let value = self.parse_expression()?;
    let command = ReturnCommand::new(value);
    self.scope_stack.commands_mut().push(Box::new(command));
    Ok(())
  }

  fn parse_expression(&mut self) -> ParsingResult<Expression> {
    let mut expr_parser = ExpressionParser::with_scope_stack(&self.scope_stack);
    while self.current_token() != Token::EOL {
      expr_parser.consume(self.current_token().clone())?;
      self.next();
    }

    Ok(expr_parser.build())
  }

  fn parse_set_variable(&mut self, iden: &str) -> ParsingResult<()> {
    if self.next() != Token::Equal {
      return Err(Box::new(ParsingError::MissingToken));
    }
    self.next();
    let expr = self.parse_expression()?;
    let stmt = SetVariable::new(iden.to_string(), self.data_types["i64"].clone(), expr);
    if self.scope_stack.get_variable(iden).is_none() {
      let variable = Variable {
        name: iden.to_string(),
        data_type: self.data_types["i64"].clone(),
      };
      // dbg!("setting variable");
      self.scope_stack.set_variable(variable);
    }
    self.scope_stack.commands_mut().push(Box::new(stmt));
    Ok(())
  }

  fn parse_function(&mut self) -> ParsingResult<()> {
    if self.current_token() != Token::Def {
      return Err(Box::new(ParsingError::MissingToken));
    }

    let mut func_name = String::new();

    {
      let Token::Identifier(fn_name) = self.next().clone() else {
        return Err(Box::new(ParsingError::MissingToken))
      };

      func_name = fn_name.clone();
    }
    
    if Token::OpenParenth != self.next() {
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

  fn next(&self) -> Token {
    let a = RefCell::new(self.lexer.borrow_mut().next());
    self.current_token.swap(&a);
    self.current_token()
  }

  fn current_token(&self) -> Token {
    return self.current_token.borrow().clone();
  }
}