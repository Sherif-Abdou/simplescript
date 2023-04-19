use inkwell::values::{IntValue, PointerValue};

use crate::ast::{Expression, Statement, DataTypeEnum};

use super::ExpressionStatement;

pub struct VariableExtractExpression {
    pub location: Box<Expression>,
    pub slot: Box<Expression>,
}

impl Statement for VariableExtractExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        todo!()
    }
}

impl ExpressionStatement for VariableExtractExpression {
    fn expression_location<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<inkwell::values::PointerValue<'a>> {
        // let ptr = data.variable_table.borrow()[name];
        let mut ptr: PointerValue = self.location.expression_location(data)?;
        let slot_value = self.slot
            .visit(data)
            .unwrap()
            .as_any_value_enum()
            .into_int_value();
        unsafe {
            let zero = data.context.i64_type().const_zero();
            let indices: Vec<IntValue> = match self.location.data_type {
                Some(ref v) => {
                    match v.value {
                        DataTypeEnum::Array(_,_) => vec![zero, slot_value],
                        DataTypeEnum::Pointer(_) => {
                            ptr = data.builder.build_load(ptr, "__tmp__").into_pointer_value();
                            vec![slot_value]
                        },
                        _ => vec![slot_value],
                    }
                },
                None => vec![slot_value],
            };
            let new_location = data.builder.build_gep(
                ptr,
                &indices,
                "__tmp__",
            );

            return Some(new_location);
        }
    }

    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        let data_type = self.location.data_type.clone()?;
        // For arrays, ignoring structs right now
        match data_type.value {
            DataTypeEnum::Array(ref a, _) => {
                Some(a.symbol.clone())
            },
            DataTypeEnum::Pointer(ref interior) => {
                Some(format!("{}", interior.symbol.clone()))
            },
            _ => {
                None
            }
        }  
    }

    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        self.slot.attach_data_types(scope, data_types);
        self.location.attach_data_types(scope, data_types);
    }
}
