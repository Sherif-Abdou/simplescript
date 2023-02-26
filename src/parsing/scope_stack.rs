use std::{collections::VecDeque};

use crate::ast::{Scope, Statement};


#[derive(Default)]
pub struct ScopeStack {
    scope_stack: VecDeque<Box<dyn Scope>>,
}

impl ScopeStack {
    pub fn push_front(&mut self, scope: Box<dyn Scope>) {
        self.scope_stack.push_front(scope);
    }

    pub fn pop_front(&mut self) -> Option<Box<dyn Scope>> {
        self.scope_stack.pop_front()
    }

    pub fn peek_front(&mut self) -> Option<&Box<dyn Scope>> {
        self.scope_stack.front()
    }

    pub fn peek_front_mut(&mut self) -> Option<&mut Box<dyn Scope>> {
        self.scope_stack.front_mut()
    }
}

impl Statement for ScopeStack {
    fn visit<'a>(&self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        None
    }
}

impl Scope for ScopeStack {
    fn get_variable(&self, name: &str) -> Option<&crate::ast::Variable> {
        for scope in &self.scope_stack {
            if scope.get_variable(name).is_some() {
                return scope.get_variable(name);
            }
        }
        None
    }

    fn set_variable(&mut self, variable: crate::ast::Variable) {
        self.scope_stack[0].set_variable(variable);
    }

    fn commands(&self) -> &Vec<Box<dyn crate::ast::Statement>> {
        self.scope_stack.front().unwrap().commands()
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn crate::ast::Statement>> {
        let front_scope = self.scope_stack.front_mut().unwrap();
        front_scope.commands_mut()
    }

    fn scope_type(&self) -> &'static str {
        "scope_stack"
    }

    fn contains_function(&self, name: &str) -> bool {
        for scope in &self.scope_stack {
            if scope.contains_function(name) {
                return true;
            }
        }
        false
    }

    fn add_function(&mut self, name: &str) {
        self.scope_stack.front_mut().unwrap().add_function(name);
    }
}
