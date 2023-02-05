use inkwell::values::{AnyValue, AnyValueEnum};

use super::{variable::Variable, statement::Statement};

#[derive(Clone)]
pub enum Expression {
  Binary(Option<Box<Expression>>, Option<Box<Expression>>, BinaryExpressionType),
  Unary(Option<Box<Expression>>, UnaryExpressionType),
  VariableRead(String),
  IntegerLiteral(i64),
}

#[derive(Clone)]
pub enum BinaryExpressionType {
  Addition,
  Subtraction,
  Multiplication,
  Division,
}

impl BinaryExpressionType {
  // Higher precidence operations are computed first
  pub fn precidence(&self) -> i64 {
    match self {
        BinaryExpressionType::Addition => 1,
        BinaryExpressionType::Subtraction => 1,
        BinaryExpressionType::Multiplication => 2,
        BinaryExpressionType::Division => 2,
    }
  }
}

#[derive(Clone)]
pub enum UnaryExpressionType {
  Reference,
  Dereference,
}

impl UnaryExpressionType {
  pub fn precidence(&self) -> i64 {
    match self {
        UnaryExpressionType::Reference => 10,
        UnaryExpressionType::Dereference => 10,
    }
  }
}

impl Expression {
  pub fn precidence(&self) -> i64 {
    match self {
        Expression::Binary(_, _, t) => t.precidence(),
        Expression::Unary(_, t) => t.precidence(),
        Expression::VariableRead(_) => 100,
        Expression::IntegerLiteral(_) => 100,
    }
  }

  pub fn is_binary(&self) -> bool {
    if let Expression::Binary(_, _, _) = self {
      return true;
    }
    return false;
  }

  pub fn binary_get_left(&self) -> &Option<Box<Expression>> {
    if let Expression::Binary(l, r, t) = self {
      return l;
    }
    panic!()
  }

  pub fn binary_get_right(&self) -> &Option<Box<Expression>> {
    if let Expression::Binary(l, r, t) = self {
      return r;
    }
    panic!()
  }

  pub fn binary_set_left(self, expr: Option<Expression>) -> Expression {
    let front = self;
    let Expression::Binary(_, r, t) = front else {
      panic!("Critical Expression Parsing Error");
    };

    let new_expression = Expression::Binary(expr.map(Box::new), r, t);

    new_expression
  }

  pub fn binary_set_right(self, expr: Option<Expression>) -> Expression {
    let front = self;
    let Expression::Binary(l, _, t) = front else {
      panic!("Critical Expression Parsing Error");
    };

    let new_expression = Expression::Binary(l, expr.map(Box::new), t);

    new_expression
  }
}
impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a super::statement::Compiler) -> Option<Box<dyn AnyValue + 'a>> {
      if let Expression::Binary(left, right, binary_type) = self {
        let parsed_left = left.as_ref().unwrap().visit(data)?.as_any_value_enum();
        let parsed_right = right.as_ref().unwrap().visit(data)?.as_any_value_enum();
        if let (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) = (parsed_left, parsed_right) {
          let value = match binary_type {
            BinaryExpressionType::Addition => data.builder.build_int_add(int_left, int_right, "__tmp__"),
            BinaryExpressionType::Subtraction => data.builder.build_int_sub(int_left, int_right, "__tmp__"),
            BinaryExpressionType::Multiplication => data.builder.build_int_mul(int_left, int_right, "__tmp__"),
            BinaryExpressionType::Division => data.builder.build_int_signed_div(int_left, int_right, "__tmp__"),
          };
          

          return Some(Box::new(value));
        }
      }

      if let Expression::VariableRead(variable_name) = self {
        // data.builder.build_load(ptr, name)
      }
      None
    }
}