use crate::ast::Expression;
use inkwell::values::BasicValue;

use super::{Statement};

pub struct ReturnCommand {
    value: Option<Expression>,
}

impl ReturnCommand {
    pub fn new(value: Option<Expression>) -> Self {
        Self { value }
    }
}

impl Statement for ReturnCommand {
    fn visit<'a>(
        &'a self,
        data: &'a super::Compiler,
    ) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        if self.value.is_none() {
            data.builder.build_return(None);
            return None;
        }
        let raw_visited = self.value.as_ref()?.visit(data);
        let visited = raw_visited.unwrap().as_any_value_enum();
        let basic_value: &dyn BasicValue = match visited {
            inkwell::values::AnyValueEnum::ArrayValue(ref a) => a,
            inkwell::values::AnyValueEnum::IntValue(ref a) => a,
            inkwell::values::AnyValueEnum::FloatValue(ref a) => a,
            inkwell::values::AnyValueEnum::PointerValue(ref a) => a,
            inkwell::values::AnyValueEnum::StructValue(ref a) => a,
            inkwell::values::AnyValueEnum::VectorValue(ref a) => a,
            _ => panic!(),
        };

        data.builder.build_return(Some(basic_value));
        None
    }
}
