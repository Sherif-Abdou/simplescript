use std::collections::HashMap;

use super::{variable::Variable, Statement};


pub trait Scope {
  fn variables(&self) -> &HashMap<String, Variable>;
  fn commands(&self) -> &Vec<dyn Statement>;

  fn variables_mut(&mut self) -> &mut HashMap<String, Variable>;
  fn commands_mut(&mut self) -> &mut Vec<dyn Statement>;
}