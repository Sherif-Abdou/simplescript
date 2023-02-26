use inkwell::{module::Module, context::Context, builder::Builder, values::{AnyValue, BasicValue, PointerValue, FunctionValue}};
use std::{collections::HashMap, cell::RefCell};
use std::any::Any;

use super::DataType;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variable_table: RefCell<HashMap<String, PointerValue<'ctx>>>,
    pub function_table: RefCell<HashMap<String, FunctionValue<'ctx>>>,
    pub data_types: HashMap<String, DataType>,
}

pub trait Statement: Any {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>>;
}
