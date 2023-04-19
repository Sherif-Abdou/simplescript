use inkwell::values::{AnyValue, BasicMetadataValueEnum, BasicValueEnum};

use crate::ast::{Expression, Statement};

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct FunctionCallExpression {
    pub name: String,
    pub arguments: Vec<Expression>
}

impl FunctionCallExpression {
    pub fn new(name: String, arguments: Vec<Expression>) -> Self {
        Self {
            name,
            arguments,
        }
    }
}

impl Statement for FunctionCallExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let function = data.function_table.borrow()[&self.name];
        let params: Vec<BasicValueEnum> = self.arguments
            .iter()
            .map(|v| {
                v.visit(data)
                    .unwrap()
                    .as_any_value_enum()
                    .try_into()
                    .unwrap()
            })
            .collect();
        let mapped: Vec<BasicMetadataValueEnum> = params.iter().map(|v| (*v).into()).collect();
        let call_output = data
            .builder
            .build_call(function, &mapped, "__tmp__")
            .try_as_basic_value();
        if let Some(call_value) = call_output.left() {
            let as_any = call_value.as_any_value_enum();
            return Some(Box::new(as_any));
        }
        None
    }
}

impl ExpressionStatement for FunctionCallExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        for argument in &mut self.arguments {
            argument.attach_data_types(scope, data_types);
        }
    }
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        let result = scope.return_type_of(&self.name)?.produce_string();
        Some(result)
    }
}
