use std::borrow::Borrow;

use inkwell::{AddressSpace, values::PointerValue};
use inkwell::types::BasicType;
use inkwell::values::BasicValueEnum;

use crate::ast::{Expression, Statement, DataTypeEnum, Compiler, DataType};
use crate::parsing::DataTypeParser;

use super::ExpressionStatement;

#[derive(Clone, PartialEq, Debug)]
pub struct VariableNamedExtractExpression {
    pub location: Box<Expression>,
    pub named_location: String,
    data_type: Option<DataType>,
}

impl VariableNamedExtractExpression {
    pub fn new(location: Expression, named_location: String) -> Self {
        Self {
            location: Box::new(location),
            named_location,
            data_type: None,
        }
    }

    fn allocate_and_store<'a>(&'a self, data: &'a Compiler, expression: &Expression) -> Option<PointerValue<'a>> {
        let produced_type = expression.data_type.as_ref();
        let dt = produced_type?.produce_llvm_type(data.context).as_basic_type_enum();
        let allocated_space = data.builder.build_alloca(dt, "__tmp__");

        let visited: BasicValueEnum = expression.visit(data).unwrap().as_any_value_enum().try_into().unwrap();

        data.builder.build_store(allocated_space.as_ref().unwrap().clone(), visited);

        Some(allocated_space.unwrap())
    }
}
impl Statement for VariableNamedExtractExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let location = self.expression_location(data).or_else(|| self.allocate_and_store(data, &self.location))?;
        let dt = self.get_data_type()?.produce_llvm_type(data.context).as_basic_type_enum();
        let adjusted_location = data.builder.build_pointer_cast(location, dt.ptr_type(AddressSpace::default()), "__tmp__");

        return Some(Box::new(data.builder.build_load(adjusted_location.unwrap(), "__tmp__").unwrap()));
    }
}

impl ExpressionStatement for VariableNamedExtractExpression {
    fn get_data_type(&self) -> Option<&crate::ast::DataType> {
        self.data_type.as_ref()
    }

    fn set_data_type(&mut self, data_type: DataType) {
        self.data_type = Some(data_type);
    }

    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        self.location.attach_data_types(scope, data_types);
    }

    fn expression_location<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<inkwell::values::PointerValue<'a>> {
        let dt = self.location.data_type.clone()?;
        let location = self.location.expression_location(data)?;
        if let DataTypeEnum::Struct(types, name_map) = &dt.value {
            let node = 
                data.builder.build_struct_gep(location, name_map[&self.named_location] as u32, "__tmp__");
            return match node {
                Ok(v) => Some(v),
                Err(inkwell::builder::BuilderError::GEPPointee) => {
                    let new_location = data.builder.build_load(location, "__tmp__").unwrap();

                    let attempted_type = types[name_map[&self.named_location] as usize].clone();
                    if let DataTypeEnum::Pointer(ref v) = attempted_type.value {
                        if let DataTypeEnum::Placeholder(ref k) = v.value {
                            let parsed = DataTypeParser::new(&data.data_types).parse_string(attempted_type.symbol.clone());

                            let t = parsed.produce_llvm_type(data.context);

                            let v =data.builder.build_pointer_cast(location, t.as_basic_type_enum().into_pointer_type(), "__casted__").unwrap();
                            let node = 
                                data.builder.build_struct_gep(v, name_map[&self.named_location] as u32, "__tmp__");
                            return Some(node.unwrap());
                        }
                    }


                    let node = 
                        data.builder.build_struct_gep(new_location.into_pointer_value(), name_map[&self.named_location] as u32, "__tmp__");
                    return Some(node.unwrap());
                }
                _ => unreachable!(),
            };
        } else if let DataTypeEnum::Placeholder(ref v) = dt.value {
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
