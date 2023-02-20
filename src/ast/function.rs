use std::{collections::HashMap};

use super::{Statement, Variable, Scope};


#[derive(Default)]
pub struct Function {
    pub commands: Vec<Box<dyn Statement>>,
    pub variables: HashMap<String, Variable>,
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
}

impl Statement for Function {
    fn visit<'a>(&'a self, data: &'a super::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let fn_value = data.module.add_function(&self.name, data.context.i64_type().fn_type(&[], false), None);
        let block = data.context.append_basic_block(fn_value, "entry");
        data.builder.position_at_end(block);
        for command in &self.commands {
            command.visit(data);
        }
        return Some(Box::new(fn_value));
    }
}