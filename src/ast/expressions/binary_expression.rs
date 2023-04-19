use inkwell::{IntPredicate, FloatPredicate};
use inkwell::values::{AnyValueEnum, AnyValue};

use crate::ast::{BinaryExpressionType, Statement, Compiler};

use super::super::Expression;
use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct BinaryExpression {
    left: Option<Box<Expression>>,
    right: Option<Box<Expression>>,
    binary_type: BinaryExpressionType
}

impl BinaryExpression {
    pub fn new(left: Option<Box<Expression>>, right: Option<Box<Expression>>, binary_type: BinaryExpressionType) -> Self {
        Self {
            left,
            right,
            binary_type
        }
    }
}

impl BinaryExpression {
    fn binary_statement<'a>(
        data: &'a Compiler,
        binary_type: &'a BinaryExpressionType,
        parsed_left: AnyValueEnum<'a>,
        parsed_right: AnyValueEnum<'a>,
    ) -> Box<AnyValueEnum<'a>> {
        match (parsed_left, parsed_right) {
            (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) => {
                let value = match binary_type {
                    BinaryExpressionType::Addition => {
                        data.builder.build_int_add(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Subtraction => {
                        data.builder.build_int_sub(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Multiplication => {
                        data.builder.build_int_mul(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Division => data
                        .builder
                        .build_int_signed_div(int_left, int_right, "__tmp__"),
                    _ => {
                        let predicate = match binary_type {
                            BinaryExpressionType::Equal => IntPredicate::EQ,
                            BinaryExpressionType::NotEqual => IntPredicate::NE,
                            BinaryExpressionType::Less => IntPredicate::SLT,
                            BinaryExpressionType::LessEqual => IntPredicate::SLE,
                            BinaryExpressionType::Greater => IntPredicate::SGE,
                            BinaryExpressionType::GreaterEqual => IntPredicate::SGT,
                            _ => unreachable!(),
                        };
                        data.builder
                            .build_int_compare(predicate, int_left, int_right, "__tmp__")
                    }
                };

                return Box::new(value.as_any_value_enum());
            }
            (AnyValueEnum::FloatValue(int_left), AnyValueEnum::FloatValue(int_right)) => {
                let value: Box<dyn AnyValue> = match binary_type {
                    BinaryExpressionType::Addition => {
                        Box::new(data.builder.build_float_add(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Subtraction => {
                        Box::new(data.builder.build_float_sub(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Multiplication => {
                        Box::new(data.builder.build_float_mul(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Division => {
                        Box::new(data.builder.build_float_div(int_left, int_right, "__tmp__"))
                    }
                    _ => {
                        let predicate = match binary_type {
                            BinaryExpressionType::Equal => FloatPredicate::OEQ,
                            BinaryExpressionType::NotEqual => FloatPredicate::ONE,
                            BinaryExpressionType::Less => FloatPredicate::OLT,
                            BinaryExpressionType::LessEqual => FloatPredicate::OLE,
                            BinaryExpressionType::Greater => FloatPredicate::OGE,
                            BinaryExpressionType::GreaterEqual => FloatPredicate::OGT,
                            _ => unreachable!(),
                        };
                        Box::new(
                            data.builder
                                .build_float_compare(predicate, int_left, int_right, "__tmp__"),
                        )
                    }
                };

                return Box::new(value.as_any_value_enum());
            }
            _ => (),
        }
        unimplemented!()
    }
}

impl ExpressionStatement for BinaryExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        if let Some(ref mut left) = self.left {
            left.attach_data_types(scope, data_types);
        }
        if let Some(ref mut right) = self.right {
            right.attach_data_types(scope, data_types);
        }
    }

    fn expression_location<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<inkwell::values::PointerValue<'a>> {
        None
    }

    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
                if self.left.as_ref()?.data_type == self.right.as_ref()?.data_type {
                    return self.left
                        .as_ref()
                        .or(self.right.as_ref())?
                        .data_type
                        .as_ref()
                        .map(|v| v.symbol.clone());
                }
                None
    }
}

impl Statement for BinaryExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let parsed_left = self.left.as_ref().unwrap().visit(data)?.as_any_value_enum();
        let parsed_right = self.right.as_ref().unwrap().visit(data)?.as_any_value_enum();
        return Some(Self::binary_statement(
            data,
            &self.binary_type,
            parsed_left,
            parsed_right,
        ));
    }
}
