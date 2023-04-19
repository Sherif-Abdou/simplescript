use inkwell::values::{AnyValueEnum, ArrayValue, AnyValue, IntValue, FloatValue, PointerValue, StructValue};

use crate::ast::{Expression, Statement};

use super::ExpressionStatement;

struct ArrayExpression {
    pub values: Vec<Expression>,
}

impl Statement for ArrayExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let expressions: Vec<Box<dyn AnyValue>> =
            self.values.iter().map(|v| v.visit(data)).flatten().collect();
        if expressions.is_empty() {
            return None;
        }
        let array_value_result: ArrayValue = match expressions[0].as_any_value_enum() {
            AnyValueEnum::ArrayValue(ref v) => {
                let mapped: Vec<ArrayValue> = expressions
                    .iter()
                    .map(|v| v.as_any_value_enum().into_array_value())
                    .collect();
                let value = v.get_type().const_array(mapped.as_slice());
                value
            }
            AnyValueEnum::IntValue(ref v) => {
                let mapped: Vec<IntValue> = expressions
                    .iter()
                    .map(|v| v.as_any_value_enum().into_int_value())
                    .collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                value
            }
            AnyValueEnum::FloatValue(ref v) => {
                let mapped: Vec<FloatValue> = expressions
                    .iter()
                    .map(|v| v.as_any_value_enum().into_float_value())
                    .collect();
                let value = v.get_type().const_array(mapped.as_slice());
                value
            }
            AnyValueEnum::PhiValue(_) => todo!(),
            AnyValueEnum::FunctionValue(_) => todo!(),
            AnyValueEnum::PointerValue(ref v) => {
                let mapped: Vec<PointerValue> = expressions
                    .iter()
                    .map(|v| v.as_any_value_enum().into_pointer_value())
                    .collect();
                let value = v.get_type().const_array(mapped.as_slice());
                value
            }
            AnyValueEnum::StructValue(ref v) => {
                let mapped: Vec<StructValue> = expressions
                    .iter()
                    .map(|v| v.as_any_value_enum().into_struct_value())
                    .collect();
                let value = v.get_type().const_array(mapped.as_slice());
                value
            }
            _ => panic!("Unexpected type")
        };
        return Some(Box::new(array_value_result));
    }
}

impl ExpressionStatement for ArrayExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        for value in &mut self.values {
            value.attach_data_types(scope, data_types);
        }
    }
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some(format!(
            "[{}:{}]",
            self.values[0].data_type.as_ref()?.symbol,
            self.values.len()
        ))
    }
}
