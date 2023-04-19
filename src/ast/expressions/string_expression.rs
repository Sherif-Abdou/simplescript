use crate::ast::Statement;

use super::ExpressionStatement;

pub struct StringExpression {
    pub value: String,
}

impl Statement for StringExpression {
    fn visit<'a>(
        &'a self,
        data: &'a crate::ast::Compiler,
    ) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let bytes: Vec<_> = self
            .value
            .as_bytes()
            .iter()
            .map(|v| data.context.i8_type().const_int(*v as u64, false))
            .collect();
        // bytes.push(data.context.i8_type().const_zero());
        let array = data.context.i8_type().const_array(&bytes);

        return Some(Box::new(array));
    }
}

impl ExpressionStatement for StringExpression {
    fn data_type(
        &self,
        scope: &dyn crate::ast::Scope,
        data_types: &std::collections::HashMap<String, crate::ast::DataType>,
    ) -> Option<String> {
        Some(format!("[char:{}]", self.value.len()))
    }
}
