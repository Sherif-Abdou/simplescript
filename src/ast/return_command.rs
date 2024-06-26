use inkwell::values::BasicValue;

use super::{Expression, Statement};


pub struct ReturnCommand {
    value: Expression,
}

impl ReturnCommand {
    pub fn new(value: Expression) -> Self {
        Self {
            value
        }
    }
}

impl Statement for ReturnCommand {
    fn visit<'a>(&'a self, data: &'a super::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        // dbg!(&self.value);
        let raw_visited = self.value.visit(data);
        let visited = raw_visited.unwrap().as_any_value_enum();
        let basic_value: &dyn BasicValue = (match visited {
            inkwell::values::AnyValueEnum::ArrayValue(ref a) => a,
            inkwell::values::AnyValueEnum::IntValue(ref a) => a,
            inkwell::values::AnyValueEnum::FloatValue(ref a) => a,
            inkwell::values::AnyValueEnum::PointerValue(ref a) => a,
            inkwell::values::AnyValueEnum::StructValue(ref a) => a,
            inkwell::values::AnyValueEnum::VectorValue(ref a) => a,
            _ => panic!()
        });

        data.builder.build_return(Some(basic_value));
        None
    }
}