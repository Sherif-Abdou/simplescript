use inkwell::{values::{BasicValueEnum, AnyValue}, AddressSpace, types::BasicType};

use crate::ast::{Expression, Statement, ExpressionEnum};

use super::ExpressionStatement;


#[derive(Clone, PartialEq, Debug)]
pub struct ExpressionCastExpression {
    pub expression: Box<Expression>,
    pub cast_type: String,
}

impl ExpressionCastExpression {
    pub fn new(expression: Box<Expression>, cast_type: String) -> Self {
        Self {
            expression,
            cast_type,
        }
    }
}

impl Statement for ExpressionCastExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let compiled = self.expression.visit(data)?.as_any_value_enum();
        if compiled.is_int_value() {
            let integer = compiled.into_int_value();

            let result: Box<dyn AnyValue> = match self.cast_type.as_str() {
                "f64" => Box::new(data.builder.build_signed_int_to_float(
                    integer,
                    data.context.f64_type(),
                    "__tmp__",
                )),
                "i64" => Box::new(data.builder.build_int_cast(
                    integer,
                    data.context.i64_type(),
                    "__tmp__",
                )),
                "char" => Box::new(data.builder.build_int_cast(
                    integer,
                    data.context.i8_type(),
                    "__tmp__",
                )),
                _ => unimplemented!(),
            };

            return Some(result);
        }
        if compiled.is_pointer_value() && self.cast_type.starts_with('&') {
            let element_type = compiled.into_array_value().get_type().get_element_type();
            let pointer_type = element_type.ptr_type(AddressSpace::default());
            let basic_value: BasicValueEnum = compiled.try_into().unwrap();

            let result: Box<dyn AnyValue> = Box::new(
                data.builder.build_bitcast(
                    basic_value,
                    pointer_type,
                    "__tmp__"
                )
            );

            return Some(result);
        }
        None
    }
}

impl ExpressionStatement for ExpressionCastExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        self.expression.attach_data_types(scope, data_types);
    }

    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some(self.cast_type.clone())
    }
}