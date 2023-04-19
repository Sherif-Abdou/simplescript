use crate::ast::Statement;

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct StructExpression {
    pub name: String,
    data_type: Option<crate::ast::DataType>
}

impl StructExpression {
    pub fn new(name: String) -> Self {
        Self { name, data_type: None }
    }
}

impl Statement for StructExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let data_type = self.get_data_type().as_ref().unwrap().clone();
        let t = data_type.produce_llvm_type(data.context).as_basic_type_enum();
        return Some(Box::new(t.const_zero()))
    }
}

impl ExpressionStatement for StructExpression {
    fn set_data_type(&mut self, data_type: crate::ast::DataType) {
        self.data_type = Some(data_type);
    }
    fn get_data_type(&self) -> Option<&crate::ast::DataType> {
        self.data_type.as_ref()
    }
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        Some(self.name.clone())
    }
}