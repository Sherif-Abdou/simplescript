use inkwell::{module::Module, context::Context, builder::Builder, values::{AnyValue, PointerValue, FunctionValue, BasicValueEnum}};
use std::{collections::HashMap, cell::RefCell};
use std::any::Any;

use super::DataType;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variable_table: RefCell<HashMap<String, PointerValue<'ctx>>>,
    pub function_table: RefCell<HashMap<String, FunctionValue<'ctx>>>,
    pub current_function_params: RefCell<HashMap<String, BasicValueEnum<'ctx>>>,
    pub data_types: HashMap<String, DataType>,
}

pub trait Statement: Any {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>>;
}
