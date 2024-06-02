use std::{
    cell::RefCell,
    collections::{HashMap},
};

use inkwell::types::{BasicMetadataTypeEnum};

use super::{DataType, Scope, Statement, Variable};

pub struct Function {
    pub params: Vec<(String, DataType)>,
    pub return_type: Option<DataType>,
    pub commands: Vec<Box<dyn Statement>>,
    pub variables: HashMap<String, Variable>,
    pub functions: HashMap<String, Option<DataType>>,
    pub is_extern: bool,
    pub name: String,
    pub is_variadic: bool, 
}

impl Function {
    pub fn new(return_type: Option<DataType>) -> Self {
        Self {
            params: vec![],
            return_type,
            commands: vec![],
            variables: Default::default(),
            functions: Default::default(),
            is_extern: false,
            name: "".to_string(),
            is_variadic: false,
        }
    }
}

impl Scope for Function {
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
        self.functions.contains_key(name)
    }

    fn return_type_of(&self, name: &str) -> Option<DataType> {
        self.functions[name].clone()
    }

    fn add_function(&mut self, name: &str, return_type: Option<DataType>) {
        self.functions.insert(name.to_owned(), return_type);
    }

    fn scope_type(&self) -> &'static str {
        "function"
    }
}

impl Statement for Function {
    fn visit<'a>(
        &'a self,
        data: &'a super::Compiler,
    ) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let param_types: Vec<BasicMetadataTypeEnum> = self
            .params
            .iter()
            .map(|(n, dt)| {
                dt.produce_llvm_type(data.context)
                    .as_basic_type_enum()
                    .into()
            })
            .collect();
        let fn_type = match self.return_type {
            Some(ref dt) => dt
                .produce_llvm_type(&data.context)
                .fn_type(&param_types, self.is_variadic),
            None => data.context.void_type().fn_type(&param_types, self.is_variadic),
        };
        let fn_value = data.module.add_function(&self.name, fn_type, None);
        data.function_table
            .borrow_mut()
            .insert(self.name.clone(), fn_value);
        if !self.is_extern {
            let values = fn_value.get_params();
            let mut param_map = HashMap::new();
            for i in 0..values.len() {
                let name = self.params[i].0.clone();
                let value = values[i];
                param_map.insert(name.clone(), value);
            }
            let refcell = RefCell::new(param_map);
            data.current_function_params.swap(&refcell);
            let block = data.context.append_basic_block(fn_value, "entry");
            data.builder.position_at_end(block);

            for command in &self.commands {
                command.visit(data);
            }
            for name in self.variables.keys() {
                data.variable_table.borrow_mut().remove(name);
                data.variable_type.borrow_mut().remove(name);
            }
        }

        Some(Box::new(fn_value))
    }
}
