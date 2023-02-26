use std::{collections::{HashMap, HashSet}, cell::RefCell};

use inkwell::types::{BasicType, BasicTypeEnum, BasicMetadataTypeEnum};

use super::{Statement, Variable, Scope, DataType};


#[derive(Default)]
pub struct Function {
    pub params: Vec<(String, DataType)>,
    pub commands: Vec<Box<dyn Statement>>,
    pub variables: HashMap<String, Variable>,
    pub functions: HashSet<String>,
    pub name: String,
}

impl Scope for Function {
    fn commands(&self) -> &Vec<Box<dyn Statement>> {
        &self.commands
    }

    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }

    fn set_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>> {
        &mut self.commands
    }

    fn scope_type(&self) -> &'static str {
        "function"
    }

    fn contains_function(&self, name: &str) -> bool {
        self.functions.contains(name)
    }

    fn add_function(&mut self, name: &str) {
        self.functions.insert(name.to_owned());
    }
}

impl Statement for Function {
    fn visit<'a>(&'a self, data: &'a super::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let param_types: Vec<BasicMetadataTypeEnum> = self.params.iter().map(|(n, dt)| dt.produce_llvm_type(data.context).as_basic_type_enum().into()).collect();
        let fn_value = data.module.add_function(&self.name, data.context.i64_type().fn_type(&param_types, false), None);
        let values = fn_value.get_params();
        let mut param_map = HashMap::new();
        for i in 0..values.len() {
            let name = self.params[i].0.clone();
            let value = values[i];
            param_map.insert(name.clone(), value);
        }
        let rfcell = RefCell::new(param_map);
        data.current_function_params.swap(&rfcell);
        let block = data.context.append_basic_block(fn_value, "entry");
        data.builder.position_at_end(block);
        for command in &self.commands {
            command.visit(data);
        }
        for name in self.variables.keys() {
            data.variable_table.borrow_mut().remove(name);
        }
        data.function_table.borrow_mut().insert(self.name.clone(), fn_value);
        return Some(Box::new(fn_value));
    }
}