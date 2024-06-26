use crate::parsing::Parser;

use super::{variable::Variable, Statement, DataType};


pub trait Scope: Statement {
    fn get_variable(&self, name: &str) -> Option<&Variable>;
    fn set_variable(&mut self, variable: Variable);
    fn commands(&self) -> &Vec<Box<dyn Statement>>;
    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>>;
    fn contains_function(&self, name: &str) -> bool;
    fn return_type_of(&self, name: &str) -> Option<DataType>;
    fn add_function(&mut self, name: &str, return_type: Option<DataType>);
    fn scope_type(&self) -> &'static str;
    fn wrap_up_parsing(&mut self, parser: &mut Parser) {}
}