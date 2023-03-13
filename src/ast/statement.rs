use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    values::{AnyValue, BasicValueEnum, FunctionValue, PointerValue},
};
use std::any::Any;
use std::{cell::RefCell, collections::HashMap};

use super::DataType;

pub struct Compiler<'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub variable_table: RefCell<HashMap<String, PointerValue<'ctx>>>,
    pub variable_type: RefCell<HashMap<String, DataType>>,
    pub function_table: RefCell<HashMap<String, FunctionValue<'ctx>>>,
    pub current_function_params: RefCell<HashMap<String, BasicValueEnum<'ctx>>>,
    pub data_types: HashMap<String, DataType>,
}

pub trait Statement: Any {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>>;
}
