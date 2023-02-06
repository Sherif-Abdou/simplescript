use std::{collections::HashMap};

use super::{Statement, Variable, Scope};


#[derive(Default)]
pub struct Function {
  pub commands: Vec<Box<dyn Statement>>,
  pub variables: HashMap<String, Variable>,
  pub name: String
}

impl Scope for Function {
    fn commands(&self) -> &Vec<Box<dyn Statement>> {
      &self.commands
    }

    fn get_variable(&self, name: &str) -> Option<&Variable> {
      self.variables.get(name)
    }

    fn set_variable(&self, variable: Variable) {
        todo!()
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>> {
      &mut self.commands
    }
}