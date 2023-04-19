use inkwell::{AddressSpace, values::PointerValue};
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;

use crate::ast::{Expression, Statement, DataTypeEnum, Compiler};

use super::ExpressionStatement;

pub struct VariableNamedExtractExpression {
    pub location: Box<Expression>,
    pub named_location: String,
}

impl VariableNamedExtractExpression {
    fn allocate_and_store<'a>(&'a self, data: &'a Compiler, expression: &Expression) -> Option<PointerValue<'a>> {
        let produced_type = expression.data_type.as_ref();
        let dt = produced_type?.produce_llvm_type(data.context).as_basic_type_enum();
        let allocated_space = data.builder.build_alloca(dt, "__tmp__");

        let visited: BasicValueEnum = expression.visit(data).unwrap().as_any_value_enum().try_into().unwrap();

        data.builder.build_store(allocated_space, visited);

        Some(allocated_space)
    }
}
impl Statement for VariableNamedExtractExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let location = self.expression_location(data).or_else(|| self.allocate_and_store(data, &self.location))?;
        let dt = self.get_data_type()?.produce_llvm_type(data.context).as_basic_type_enum();
        let adjusted_location = data.builder.build_pointer_cast(location, dt.ptr_type(AddressSpace::default()), "__tmp__");

        return Some(Box::new(data.builder.build_load(adjusted_location, "__tmp__")));
    }
}

impl ExpressionStatement for VariableNamedExtractExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        self.location.attach_data_types(scope, data_types);
    }

    fn expression_location<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<inkwell::values::PointerValue<'a>> {
        let dt = self.location.data_type.clone()?;
        let location = self.location.expression_location(data)?;
        if let DataTypeEnum::Struct(_, name_map) = &dt.value {
            return Some(
                data.builder.build_struct_gep(location, name_map[&self.named_location] as u32, "__tmp__").unwrap()
            );
        }

        None
    }

    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        let data_type = self.location.data_type.clone()?;
        if let DataTypeEnum::Struct(ref slots, ref name_map) = data_type.value {
            let interior_type = slots[name_map[&self.named_location] as usize].as_ref();
            return Some(interior_type.produce_string());
        }
        None
    }
}
