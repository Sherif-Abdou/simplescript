use super::{variable::Variable, Statement};


pub trait Scope: Statement {
    fn get_variable(&self, name: &str) -> Option<&Variable>;
    fn set_variable(&mut self, variable: Variable);
    fn commands(&self) -> &Vec<Box<dyn Statement>>;
    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>>;
    fn contains_function(&self, name: &str) -> bool;
    fn add_function(&mut self, name: &str);
    fn scope_type(&self) -> &'static str;
}