use crate::ast::Statement;

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct IntegerExpression {
    pub value: i64,
}

impl IntegerExpression {
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

impl Statement for IntegerExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        Some(Box::new(data.context.i64_type().const_int(self.value as u64, true)))
    }
}

impl ExpressionStatement for IntegerExpression {
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some("i64".to_string())
    }
}