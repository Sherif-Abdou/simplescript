use inkwell::{values::{AnyValue, AnyValueEnum, ArrayValue, IntValue, FloatValue, PointerValue, StructValue}, types::BasicType, AddressSpace};
use inkwell::types::AnyTypeEnum::IntType;

use super::{statement::Statement, DataType, Scope, DataTypeEnum, Compiler};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Binary(Option<Box<Expression>>, Option<Box<Expression>>, BinaryExpressionType),
    Unary(Option<Box<Expression>>, UnaryExpressionType),
    Array(Vec<Expression>),
    VariableRead(String),
    VariableExtract(String, Box<Expression>),
    IntegerLiteral(i64),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BinaryExpressionType {
    Addition,
    Subtraction,
    Multiplication,
    Division,
}

impl BinaryExpressionType {
    // Higher precidence operations are computed first
    pub fn precidence(&self) -> i64 {
        match self {
            BinaryExpressionType::Addition => 1,
            BinaryExpressionType::Subtraction => 1,
            BinaryExpressionType::Multiplication => 2,
            BinaryExpressionType::Division => 2,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UnaryExpressionType {
    Reference,
    Dereference,
}

impl UnaryExpressionType {
    pub fn precidence(&self) -> i64 {
        match self {
            UnaryExpressionType::Reference => 10,
            UnaryExpressionType::Dereference => 10,
        }
    }
}

impl Expression {
    pub fn precidence(&self) -> i64 {
        match self {
            Expression::Binary(_, _, t) => t.precidence(),
            Expression::Unary(_, t) => t.precidence(),
            Expression::VariableRead(_) => 100,
            Expression::IntegerLiteral(_) => 100,
            Expression::Array(_) => 200,
            Expression::VariableExtract(_, _) => 100,
        }
    }

    pub fn data_type(&self, scope: &dyn Scope) -> Option<String> {
        match self {
            Expression::Binary(l, r, _) => {
                if l.as_ref().unwrap().data_type(scope) == r.as_ref().unwrap().data_type(scope) {
                    return l.as_ref().unwrap().data_type(scope).clone();
                }
                return None;
            }
            Expression::Unary(Some(interior), dt) => {
                let thing = match dt {
                    UnaryExpressionType::Reference => format!("&{}", interior.data_type(scope).unwrap()),
                    UnaryExpressionType::Dereference => interior.data_type(scope).unwrap()[1..].to_string(),
                };
                return Some(thing);
            },
            Expression::VariableRead(v) => return Some(scope.get_variable(v).unwrap().data_type.symbol.clone()),
            Expression::IntegerLiteral(_) => return Some("i64".to_string()),
            Expression::Array(ref list) => {
                // dbg!("is array");
                return Some(format!("[{}:{}]", list[0].data_type(scope)?, list.len()));
            }
            Expression::VariableExtract(ref name, _) => {
                let data_type = &scope.get_variable(name).unwrap().data_type;
                // For arrays, ignoring structs right now
                if let DataTypeEnum::Array(ref a, _) = data_type.value {
                    return Some(a.symbol.clone());
                } else {
                    unimplemented!()
                }
            },
            _ => unimplemented!()
        };
        None
    }

    pub fn is_binary(&self) -> bool {
        if let Expression::Binary(_, _, _) = self {
            return true;
        }
        return false;
    }

    pub fn binary_get_left(&self) -> &Option<Box<Expression>> {
        if let Expression::Binary(l, r, t) = self {
            return l;
        }
        panic!()
    }

    pub fn binary_get_right(&self) -> &Option<Box<Expression>> {
        if let Expression::Binary(l, r, t) = self {
            return r;
        }
        panic!()
    }

    pub fn binary_set_left(self, expr: Option<Expression>) -> Expression {
        let front = self;
        let Expression::Binary(_, r, t) = front else {
            panic!("Critical Expression Parsing Error");
        };

        let new_expression = Expression::Binary(expr.map(Box::new), r, t);

        new_expression
    }

    pub fn binary_set_right(self, expr: Option<Expression>) -> Expression {
        let front = self;
        let Expression::Binary(l, _, t) = front else {
            panic!("Critical Expression Parsing Error");
        };

        let new_expression = Expression::Binary(l, expr.map(Box::new), t);

        new_expression
    }

    pub fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> {
        if let Expression::VariableExtract(ref name, ref slot) = self {
            // dbg!("Doing thing for extract");
            let ptr = data.variable_table.borrow()[name];
            let slot_value = slot.visit(data).unwrap().as_any_value_enum().into_int_value();
            unsafe {
                let new_location = data.builder.build_gep(ptr, &[data.context.i64_type().const_zero(), slot_value], "__tmp__");
                let type_changed = data.builder.build_pointer_cast(new_location, data.context.i64_type().ptr_type(AddressSpace::default()), "__tmp__");

                return Some(type_changed);
            }
        }

        if let Expression::VariableRead(variable_name) = self {
            let ptr = data.variable_table.borrow()[variable_name];
            return Some(ptr);
        }

        if let Expression::Unary(Some(ref interior), UnaryExpressionType::Dereference) = self {
            let dereference = data.builder.build_load(interior.expression_location(data).unwrap(), "__tmp__");
            let as_ptr_type = dereference.into_pointer_value();

            return Some(as_ptr_type);
        }

        None
    }
}

impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a super::statement::Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        if let Expression::Binary(left, right, binary_type) = self {
            let parsed_left = left.as_ref().unwrap().visit(data)?.as_any_value_enum();
            let parsed_right = right.as_ref().unwrap().visit(data)?.as_any_value_enum();
            if let (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) = (parsed_left, parsed_right) {
                let value = match binary_type {
                    BinaryExpressionType::Addition => data.builder.build_int_add(int_left, int_right, "__tmp__"),
                    BinaryExpressionType::Subtraction => data.builder.build_int_sub(int_left, int_right, "__tmp__"),
                    BinaryExpressionType::Multiplication => data.builder.build_int_mul(int_left, int_right, "__tmp__"),
                    BinaryExpressionType::Division => data.builder.build_int_signed_div(int_left, int_right, "__tmp__"),
                };


                return Some(Box::new(value));
            }
        }

        if let Expression::Unary(Some(interior), operation) = self {
            match operation {
                UnaryExpressionType::Reference => {
                    return Some(Box::new(interior.expression_location(data).unwrap()));
                },
                UnaryExpressionType::Dereference => {
                    let location = interior.visit(data).unwrap().as_any_value_enum().into_pointer_value();
                    return Some(Box::new(data.builder.build_load(location, "__tmp__")));
                },
            } 
        };


        if let Expression::VariableRead(variable_name) = self {
            let load = data.builder.build_load(self.expression_location(data).unwrap(), variable_name);
            return Some(Box::new(load));
        }

        if let Expression::IntegerLiteral(ref literal) = self {
            let t = data.context.i64_type();
            let value = t.const_int(literal.abs() as u64, false);

            return Some(Box::new(value));
        }
        if let Expression::Array(ref values) = self {
            let expressions: Vec<Box<dyn AnyValue>> = values.iter().map(|v| v.visit(data)).flatten().collect();
            if expressions.is_empty() {
                return None;
            }
            let thing: ArrayValue = match expressions[0].as_any_value_enum() {
                AnyValueEnum::ArrayValue(ref v) => {
                    let mapped: Vec<ArrayValue> = expressions.iter().map(|v| v.as_any_value_enum().into_array_value()).collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                    value
                }
                AnyValueEnum::IntValue(ref v) => {
                    let mapped: Vec<IntValue> = expressions.iter().map(|v| v.as_any_value_enum().into_int_value()).collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                    value
                }
                AnyValueEnum::FloatValue(ref v) => {
                    let mapped: Vec<FloatValue> = expressions.iter().map(|v| v.as_any_value_enum().into_float_value()).collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                    value
                }
                AnyValueEnum::PhiValue(_) => todo!(),
                AnyValueEnum::FunctionValue(_) => todo!(),
                AnyValueEnum::PointerValue(ref v) => {
                    let mapped: Vec<PointerValue> = expressions.iter().map(|v| v.as_any_value_enum().into_pointer_value()).collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                    value
                }
                AnyValueEnum::StructValue(ref v) => {
                    let mapped: Vec<StructValue> = expressions.iter().map(|v| v.as_any_value_enum().into_struct_value()).collect();
                    let value = v.get_type().const_array(mapped.as_slice());
                    value
                }
                AnyValueEnum::VectorValue(ref v) => todo!(),
                AnyValueEnum::InstructionValue(_) => todo!(),
                AnyValueEnum::MetadataValue(_) => todo!(),
            };
            return Some(Box::new(thing));
        }

        if let Expression::VariableExtract(_, _) = self {
            // dbg!("Doing thing for extract");
            let location = self.expression_location(data).unwrap();

            return Some(Box::new(data.builder.build_load(location, "__tmp__")));
        }
        None
    }
}
