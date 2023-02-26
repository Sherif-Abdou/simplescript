use crate::{lexing::Token, ast::Expression};

use super::{expression_parser::ExpressionParser, ParsingResult, scope_stack::ScopeStack};

pub struct FunctionCallParser<'a> {
  sub_parser: Option<ExpressionParser<'a>>,
  arguments: Vec<Box<Expression>>,
  scope_stack: &'a ScopeStack,
  name: String,
}

impl<'a> FunctionCallParser<'a> {
  pub fn new(scope_stack: &'a ScopeStack) -> Self {
    Self {
      arguments: Vec::new(),
      name: "".to_owned(),
      sub_parser: None,
      scope_stack
    }
  }

  pub fn consume(&mut self, token: Token) -> ParsingResult<bool> {
    if let Some(ref mut parser) = self.sub_parser {
      let can_continue = parser.consume(token.clone())?;
      if !can_continue {
        let maybe_built = parser.build();
        if let Some(built) = maybe_built {
          self.arguments.push(Box::new(built));
        }
      }
      if token != Token::CloseParenth {
        return Ok(true)
      }
    }

    match token {
      Token::Identifier(name) => self.name = name,
      Token::OpenParenth => self.sub_parser = Some(ExpressionParser::with_scope_stack(&self.scope_stack)),
      Token::CloseParenth => {
        self.sub_parser = None;
        return Ok(false);
      }
      _ => {}
    }

    Ok(true)
  }

  pub fn build(&mut self) -> Expression {
      let function_call = Expression::FunctionCall(self.name.to_owned(), self.arguments.clone());
      return function_call;
  }
}