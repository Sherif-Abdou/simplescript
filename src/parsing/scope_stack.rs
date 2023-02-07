use std::collections::VecDeque;

use crate::ast::{Scope, Statement};

#[derive(Default)]
pub struct ScopeStack {
  scope_stack: VecDeque<Box<dyn Scope>>
}

impl ScopeStack {
  pub fn push_front(&mut self, scope: Box<dyn Scope>) {
    self.scope_stack.push_front(scope);
  }

  pub fn pop_front(&mut self) -> Option<Box<dyn Scope>> {
    self.scope_stack.pop_front()
  }

}

impl Statement for ScopeStack {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        None
    }
}

impl Scope for ScopeStack {
    fn commands(&self) -> &Vec<Box<dyn crate::ast::Statement>> {
        self.scope_stack.front().unwrap().commands()
    }

    fn get_variable(&self, name: &str) -> Option<&crate::ast::Variable> {
        for scope in &self.scope_stack {
          if scope.get_variable(name).is_some() {
            return scope.get_variable(name);
          }
        }
        None
    }

    fn set_variable(&self, variable: crate::ast::Variable) {
        self.scope_stack[0].set_variable(variable);
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn crate::ast::Statement>> {
        let front_scope = self.scope_stack.front_mut().unwrap();
        front_scope.commands_mut()
    }
}
