use inkwell::values::{AnyValue, AnyValueEnum};

use super::{variable::Variable, statement::Statement};

pub enum Expression {
  Binary(Box<Expression>, Box<Expression>, BinaryExpressionType),
  Unary(Box<Expression>, UnaryExpressionType),
  VariableRead(String),
}

pub enum BinaryExpressionType {
  Addition,
  Subtraction,
  Multiplication,
  Division,
}

pub enum UnaryExpressionType {
  Reference,
  Dereference,
}

impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a super::statement::Compiler) -> Option<Box<dyn AnyValue + 'a>> {
      if let Expression::Binary(left, right, binary_type) = self {
        let parsed_left = left.visit(data)?.as_any_value_enum();
        let parsed_right = right.visit(data)?.as_any_value_enum();
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