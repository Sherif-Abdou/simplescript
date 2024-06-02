use crate::ast::Statement;

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct StringExpression {
    pub value: String,
}

impl StringExpression {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Statement for StringExpression {
    fn visit<'a>(
        &'a self,
        data: &'a crate::ast::Compiler,
    ) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let mut bytes: Vec<_> = self
            .value
            .as_bytes()
            .iter()
            .map(|v| data.context.i8_type().const_int(*v as u64, false))
            .collect();
        bytes.push(data.context.i8_type().const_zero());
        let array = data.context.i8_type().const_array(&bytes);
        let allocated = data.builder.build_alloca(array.get_type(), "__strstore__").unwrap();

        data.builder.build_store(allocated, array).unwrap();
        return Some(Box::new(allocated));
    }
}

impl ExpressionStatement for StringExpression {
    fn data_type(
        &self,
        scope: &dyn crate::ast::Scope,
        data_types: &std::collections::HashMap<String, crate::ast::DataType>,
    ) -> Option<String> {
        Some("&char".to_string())
    }
}
