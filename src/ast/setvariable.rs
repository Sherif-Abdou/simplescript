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
        if !data.variable_table.borrow().contains_key(&self.name) {
            let allocation = data.builder.build_alloca(data.context.i64_type(), &self.name);
            data.variable_table.borrow_mut().insert(self.name.clone(), allocation);
        }
        let value = self.value.visit(data).unwrap();
        let borrowed = data.variable_table.borrow();
        let allocation = borrowed.get(&self.name).unwrap();
        let e = value.as_any_value_enum();
        let res = match e {
            AnyValueEnum::ArrayValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::IntValue(a) => data.builder.build_store(*allocation, a),
            AnyValueEnum::FloatValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::PointerValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::StructValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            AnyValueEnum::VectorValue(a) => data.builder.build_store(data.builder.build_alloca(data.context.i64_type(), &self.name), a),
            _ => unimplemented!()
        };

        Some(Box::new(res))
    }
}
