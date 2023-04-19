use std::{collections::HashMap, fmt::Debug};

use inkwell::values::PointerValue;

use crate::ast::{Statement, Scope, DataType, Compiler};

pub trait ExpressionStatement: Statement + Debug {
    fn attach_data_types(&mut self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) {}
    
    fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> { None }

    fn data_type(&self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) -> Option<String>;
    
    fn get_data_type(&self) -> Option<&DataType> { None }

    fn set_data_type(&mut self, data_type: DataType) { }
}
