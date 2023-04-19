use crate::ast::Statement;

use super::ExpressionStatement;

struct CharExpression {
    value: u8,
}

impl Statement for CharExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
      return Some(Box::new(data.context.i8_type().const_int(self.value as u64, false)));
    }
}

impl ExpressionStatement for CharExpression {
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some("char".to_string())
    }
}