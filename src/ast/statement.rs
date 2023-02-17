use inkwell::{module::Module, context::Context, builder::Builder, values::{AnyValue, BasicValue, PointerValue}};
use std::{collections::HashMap, cell::RefCell};

use super::DataType;

pub struct Compiler<'ctx> {
  pub context: &'ctx Context,
  pub module: Module<'ctx>,
  pub builder: Builder<'ctx>,
  pub variable_table: RefCell<HashMap<String, PointerValue<'ctx>>>,
  pub data_types: HashMap<String, DataType>
}

pub trait Statement {
  fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>>;
}
