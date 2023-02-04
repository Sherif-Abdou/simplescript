use std::{collections::HashMap};

use super::{Statement, Variable, Scope};


#[derive(Default)]
pub struct Function {
  pub commands: Vec<Box<dyn Statement>>,
  pub variables: HashMap<String, Variable>,
  pub name: String
}

impl Scope for Function {
    fn variables(&self) -> &HashMap<String, Variable> {
      &self.variables
    }

    fn commands(&self) -> &Vec<Box<dyn Statement>> {
      &self.commands
    }

    fn variables_mut(&mut self) -> &mut HashMap<String, Variable> {
      &mut self.variables
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>> {
      &mut self.commands
    }
}