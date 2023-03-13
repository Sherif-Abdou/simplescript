use crate::ast::Expression;
use std::{collections::HashMap};

use super::{DataType, Scope, Statement, Variable};

pub struct IfCondition {
    pub commands: Vec<Box<dyn Statement>>,
    pub variables: HashMap<String, Variable>,
    condition: Expression,
}

impl IfCondition {
    pub fn new(condition: Expression) -> Self {
        Self {
            commands: Vec::new(),
            variables: HashMap::new(),
            condition,
        }
    }
}

impl Scope for IfCondition {
    fn get_variable(&self, name: &str) -> Option<&Variable> {
        self.variables.get(name)
    }

    fn set_variable(&mut self, variable: Variable) {
        self.variables.insert(variable.name.clone(), variable);
    }

    fn commands(&self) -> &Vec<Box<dyn Statement>> {
        &self.commands
    }

    fn commands_mut(&mut self) -> &mut Vec<Box<dyn Statement>> {
        &mut self.commands
    }

    fn contains_function(&self, name: &str) -> bool {
        false
    }

    fn return_type_of(&self, name: &str) -> Option<DataType> {
        todo!()
    }

    fn add_function(&mut self, name: &str, return_type: Option<DataType>) {}

    fn scope_type(&self) -> &'static str {
        "function"
    }

    fn wrap_up_parsing(&mut self, parser: &mut crate::parsing::Parser) {}
}

impl Statement for IfCondition {
    fn visit<'a>(
        &'a self,
        data: &'a super::Compiler,
    ) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let current_block = data.builder.get_insert_block().unwrap();
        let condition_block = data.context.insert_basic_block_after(current_block, "0");
        let after_block = data.context.insert_basic_block_after(condition_block, "0");

        data.builder.build_conditional_branch(
            self.condition
                .visit(data)?
                .as_any_value_enum()
                .try_into()
                .unwrap(),
            condition_block,
            after_block,
        );
        // Inside if condition
        data.builder.position_at_end(condition_block);
        for command in &self.commands {
            command.visit(data);
        }
        data.builder.build_unconditional_branch(after_block);

        data.builder.position_at_end(after_block);

        for name in self.variables.keys() {
            data.variable_table.borrow_mut().remove(name);
            data.variable_type.borrow_mut().remove(name);
        }
        None
    }
}
