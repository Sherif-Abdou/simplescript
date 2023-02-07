use std::collections::HashMap;

use super::{variable::Variable, Statement};


pub trait Scope: Statement {
  fn get_variable(&self, name: &str) -> Option<&Variable>;
  fn set_variable(&self, variable: Variable);
  fn commands(&self) -> &Vec<Box<dyn Statement>>;
  fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>>;
}