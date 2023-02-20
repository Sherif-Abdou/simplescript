use inkwell::AddressSpace;
use inkwell::values::{AnyValue, BasicValue, BasicValueEnum};
use crate::ast::{Compiler, Expression, Statement};

pub struct InsertVariable {
    location: Expression,
    value: Expression,
}

impl InsertVariable {
    pub fn new(location: Expression, value: Expression) -> Self {
        Self {
            location,
            value
        }
    }
}

impl Statement for InsertVariable {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        if let Expression::VariableExtract(ref name, ref slot) = self.location {
            let to_be_stored: BasicValueEnum = self.value.visit(data)?.as_any_value_enum().try_into().unwrap();
            let ptr = data.variable_table.borrow()[name];
            let slot_value = slot.visit(data).unwrap().as_any_value_enum().into_int_value();
            unsafe {
                let new_location = data.builder.build_gep(ptr, &[data.context.i64_type().const_zero(), slot_value], "__tmp__");
                let type_changed = data.builder.build_pointer_cast(new_location, data.context.i64_type().ptr_type(AddressSpace::default()), "__tmp");

                let stored = data.builder.build_store(type_changed, to_be_stored);
                return Some(Box::new(stored));
            }
        }
        None
    }
}
