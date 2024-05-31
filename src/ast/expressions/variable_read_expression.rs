use inkwell::values::BasicValue;

use crate::ast::Statement;

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct VariableReadExpression {
    pub name: String
}

impl VariableReadExpression {
    pub fn new(name: String) -> Self {
        Self {
            name
        }
    }
}

impl Statement for VariableReadExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        return if let Some(param) = data.current_function_params.borrow().get(&self.name) {
            Some(Box::new(param.as_basic_value_enum()))
        } else {
            let load = data
                .builder
                .build_load(self.expression_location(data).unwrap(), &self.name);
            Some(Box::new(load.unwrap()))
        }
    }
}

impl ExpressionStatement for VariableReadExpression {
    fn expression_location<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<inkwell::values::PointerValue<'a>> {
        if let Some(p) = data.current_function_params.borrow().get(&self.name) {
            return Some(p.into_pointer_value());
        }
        let ptr = data.variable_table.borrow()[&self.name];
        return Some(ptr);
    }
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some(scope.get_variable(&self.name)?.data_type.symbol.clone())
    }
}
