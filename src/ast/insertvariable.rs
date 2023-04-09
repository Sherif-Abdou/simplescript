use crate::ast::{Compiler, Expression, ExpressionEnum, Statement};
use inkwell::values::{AnyValue, BasicValueEnum};

pub struct InsertVariable {
    location: Expression,
    value: Expression,
}

impl InsertVariable {
    pub fn new(location: Expression, value: Expression) -> Self {
        Self { location, value }
    }
}

impl Statement for InsertVariable {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        let to_be_stored: BasicValueEnum = self
            .value
            .visit(data)?
            .as_any_value_enum()
            .try_into()
            .unwrap();
        // dbg!(&self.location);
        let ptr = self.location.expression_location(data).unwrap();
        let stored = data.builder.build_store(ptr, to_be_stored);
        Some(Box::new(stored))
    }
}
