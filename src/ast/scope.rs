use std::collections::HashMap;

use super::{variable::Variable, Statement};


pub struct Scope {
  pub variables: HashMap<String, Variable>,
  pub commands: Vec<Box<dyn Statement>>
}