use std::collections::VecDeque;

use crate::{lexing::Token, ast::Expression};

use super::parser::{ParsingResult};

pub struct ExpressionParser<'a> {
  top_expression: Option<Expression>,
  expression_stack: VecDeque<&'a mut Expression>
}

impl<'a> ExpressionParser<'a> {
  pub fn new() -> Self {
    Self {
      expression_stack: VecDeque::new(),
      top_expression: None,
    }
  }

  pub fn consume(&mut self, token: Token) -> ParsingResult<()> {
    match token {
      Token::Integer(v) => {
        let mini_expr = Expression::IntegerLiteral(v);
      },
      _ => unimplemented!()
    };

    Ok(())
  }

  fn append_expr(&mut self, expression: Expression) {
    if self.expression_stack.is_empty() {
      self.top_expression = Some(expression);
      self.expression_stack.push_front(&mut self.top_expression.unwrap());
      return
    }

    if self.front().is_binary() {
      if !self.binary_left() {
        self.binary_set_left(Some(expression));
      } else if !self.binary_right() {
        self.binary_set_right(Some(expression));
      } else if expression.is_binary() {
        let new_expr_precidence = expression.precidence();
        let top_expr_precidence = self.front().precidence();

        // Ex: 3+2*5
        if new_expr_precidence >= top_expr_precidence {
          let old_right = self.front().binary_get_right().clone();
          let new_expression = expression.binary_set_left(Some(*old_right.unwrap()));
          self.binary_set_right(Some(new_expression));
        } else { // Ex: 2*5+3
        }
      }
   }
  }

  fn front(&self) -> &Expression {
    return &self.expression_stack[0];
  }

  fn binary_left(&self) -> bool {
    if let Expression::Binary(l, r, t) = self.front() {
      return l.is_some();
    }
    false
  }

  fn binary_set_left(&mut self, expression: Option<Expression>) {
    let v = self.expression_stack.pop_front().unwrap().binary_set_left(expression);
    self.expression_stack.push_front(v);
  }

  fn binary_set_right(&mut self, expression: Option<Expression>) {
    let v = self.expression_stack.pop_front().unwrap().binary_set_right(expression);
    self.expression_stack.push_front(v);
  }
  

  fn binary_right(&self) -> bool {
    if let Expression::Binary(l, r, t) = self.front() {
      return r.is_some();
    }
    false
  }
}