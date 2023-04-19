use crate::ast::Statement;

use super::ExpressionStatement;

pub struct FloatExpression {
    pub value: f64,
}

impl Statement for FloatExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        Some(Box::new(data.context.f64_type().const_float(self.value)))
    }
}

impl ExpressionStatement for FloatExpression {
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
      return Some("f64".to_string());
    }
}