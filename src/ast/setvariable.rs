use inkwell::values::AnyValueEnum;

use super::{Expression, Statement};

pub struct SetVariable {
  name: String,
  value: Expression,
}

impl SetVariable {
  pub fn new(name: String, value: Expression) -> Self {
    Self {
      name,
      value,
    }
  }
}

impl Statement for SetVariable {
    fn visit<'a>(&'a self, data: &'a super::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        let value = self.value.visit(data).unwrap();
        let e = value.as_any_value_enum();
        let res = match e {
            AnyValueEnum::ArrayValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::IntValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::FloatValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::PointerValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::StructValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::VectorValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            _ => unimplemented!()
        };

        Some(Box::new(res))
    }
}