use std::{collections::HashSet};
use std::collections::HashMap;
use inkwell::values::AnyValue;
use crate::ast::{Compiler, Scope, Statement, Variable};

use super::DataType;

#[derive(Default)]
pub struct RootScope {
    pub commands: Vec<Box<dyn Statement>>,
    pub variables: HashMap<String, Variable>,
    pub functions: HashMap<String, Option<DataType>>,
    pub name: String,
}

impl Statement for RootScope {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        for command in &self.commands {
            command.visit(data);
        }

        None
    }
}

impl Scope for RootScope {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }

    fn set_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    fn commands(&self) -> &Vec<Box<dyn Statement>> {
        &self.commands
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>> {
        &mut self.commands
    }

    fn scope_type(&self) -> &'static str {
        "root"
    }

    fn contains_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    fn add_function(&mut self, name: &str, return_type: Option<DataType>) {
        self.functions.insert(name.to_owned(), return_type);
    }

    fn return_type_of(&self, name: &str) -> Option<DataType> {
        self.functions[name].clone()
    }
}