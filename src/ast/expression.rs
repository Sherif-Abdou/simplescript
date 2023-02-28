use std::any::Any;
use std::collections::HashMap;
use inkwell::{values::{AnyValue, AnyValueEnum, ArrayValue, IntValue, FloatValue, PointerValue, StructValue, BasicValue, BasicValueEnum, BasicMetadataValueEnum}, types::BasicType, AddressSpace, IntPredicate, FloatPredicate};
use crate::ast::DataType;
use crate::parsing::DataTypeParser;


use super::{statement::Statement, Scope, DataTypeEnum, Compiler};

#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
    Binary(Option<Box<Expression>>, Option<Box<Expression>>, BinaryExpressionType),
    Unary(Option<Box<Expression>>, UnaryExpressionType),
    FunctionCall(String, Vec<Box<Expression>>),
    Array(Vec<Expression>),
    VariableRead(String),
    VariableExtract(String, Box<Expression>),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(u8),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BinaryExpressionType {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl BinaryExpressionType {
    // Higher precidence operations are computed first
    pub fn precidence(&self) -> i64 {
        match self {
            BinaryExpressionType::Addition => 1,
            BinaryExpressionType::Subtraction => 1,
            BinaryExpressionType::Multiplication => 2,
            BinaryExpressionType::Division => 2,
            _ => 0,
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
            Expression::FloatLiteral(_) => 100,
            Expression::Array(_) => 200,
            Expression::VariableExtract(_, _) => 100,
            Expression::FunctionCall(_, _) => 100,
            _ => 100,
        }
    }

    pub fn data_type(&self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) -> Option<String> {
        match self {
            Expression::Binary(l, r, _) => {
                if l.as_ref().unwrap().data_type(scope, data_types) == r.as_ref().unwrap().data_type(scope, data_types) {
                    return l.as_ref().or(r.as_ref()).unwrap().data_type(scope, data_types).clone();
                }
                return None;
            }
            Expression::Unary(Some(interior), dt) => {
                let thing = match dt {
                    UnaryExpressionType::Reference => format!("&{}", interior.data_type(scope, data_types).unwrap()),
                    UnaryExpressionType::Dereference => interior.data_type(scope, data_types).unwrap()[1..].to_string(),
                };
                return Some(thing);
            },
            Expression::VariableRead(v) => {
                return Some(scope.get_variable(v).unwrap().data_type.symbol.clone())
            },
            Expression::IntegerLiteral(_) => return Some("i64".to_string()),
            Expression::FloatLiteral(_) => return Some("f64".to_string()),
            Expression::StringLiteral(ref s) => return Some(format!("[char:{}]", s.len())),
            Expression::CharLiteral(_) => return Some("char".to_string()),
            Expression::Array(ref list) => {
                // dbg!("is array");
                return Some(format!("[{}:{}]", list[0].data_type(scope, data_types)?, list.len()));
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
            Expression::FunctionCall(name, _) => {
                let result = scope.return_type_of(name).unwrap().produce_string();
                return Some(result);
            },
            _ => unimplemented!()
        };
        None
    }

    pub fn expression_type(&self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) -> Option<DataType> {
        let dt_opt = self.data_type(scope, data_types);
        if let Some(dt) = dt_opt {
            let mut data_type_parser = DataTypeParser::new(data_types);

            let data_type = data_type_parser.parse_string(dt);
            return Some(data_type);
        }

        if let Expression::FunctionCall(name, params) = self {
        }

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
                // let type_changed = data.builder.build_pointer_cast(new_location, data.context.i64_type().ptr_type(AddressSpace::default()), "__tmp__");

                return Some(new_location);
            }
        }

        if let Expression::VariableRead(variable_name) = self {
            if let Some(p) = data.current_function_params.borrow().get(variable_name) {
                dbg!("converting p hopefully", p.is_pointer_value());
                return Some(p.into_pointer_value());
            }
            let ptr = data.variable_table.borrow()[variable_name];
            return Some(ptr);
        }

        if let Expression::Unary(Some(ref interior), UnaryExpressionType::Dereference) = self {
            let dereference = data.builder.build_load(interior.expression_location(data).unwrap(), "__tmp__");
            dbg!(&dereference);
            let as_ptr_type = dereference.into_pointer_value();

            return Some(as_ptr_type);
        }

        None
    }

    fn binary_statement<'a>(data: &'a Compiler, binary_type: &'a BinaryExpressionType, parsed_left: AnyValueEnum<'a>, parsed_right: AnyValueEnum<'a>) -> Box<AnyValueEnum<'a>> {
        if let (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) = (parsed_left, parsed_right) {
            let value = match binary_type {
                BinaryExpressionType::Addition => data.builder.build_int_add(int_left, int_right, "__tmp__"),
                BinaryExpressionType::Subtraction => data.builder.build_int_sub(int_left, int_right, "__tmp__"),
                BinaryExpressionType::Multiplication => data.builder.build_int_mul(int_left, int_right, "__tmp__"),
                BinaryExpressionType::Division => data.builder.build_int_signed_div(int_left, int_right, "__tmp__"),
                _ => {
                    let predicate = match binary_type {
                        BinaryExpressionType::Equal => IntPredicate::EQ,
                        BinaryExpressionType::NotEqual => IntPredicate::NE,
                        BinaryExpressionType::Less => IntPredicate::SLT,
                        BinaryExpressionType::LessEqual => IntPredicate::SLE,
                        BinaryExpressionType::Greater => IntPredicate::SGE,
                        BinaryExpressionType::GreaterEqual => IntPredicate::SGT,
                        _ => unreachable!()
                    };
                    data.builder.build_int_compare(predicate, int_left, int_right, "__tmp__")
                }
            };

            return Box::new(value.as_any_value_enum());
        }
        if let (AnyValueEnum::FloatValue(int_left), AnyValueEnum::FloatValue(int_right)) = (parsed_left, parsed_right) {
            let value: Box<dyn AnyValue> = match binary_type {
                BinaryExpressionType::Addition => Box::new(data.builder.build_float_add(int_left, int_right, "__tmp__")),
                BinaryExpressionType::Subtraction => Box::new(data.builder.build_float_sub(int_left, int_right, "__tmp__")),
                BinaryExpressionType::Multiplication => Box::new(data.builder.build_float_mul(int_left, int_right, "__tmp__")),
                BinaryExpressionType::Division => Box::new(data.builder.build_float_div(int_left, int_right, "__tmp__")),
                _ => {
                    let predicate = match binary_type {
                        BinaryExpressionType::Equal => FloatPredicate::OEQ,
                        BinaryExpressionType::NotEqual => FloatPredicate::ONE,
                        BinaryExpressionType::Less => FloatPredicate::OLT,
                        BinaryExpressionType::LessEqual => FloatPredicate::OLE,
                        BinaryExpressionType::Greater => FloatPredicate::OGE,
                        BinaryExpressionType::GreaterEqual => FloatPredicate::OGT,
                        _ => unreachable!()
                    };
                    Box::new(data.builder.build_float_compare(predicate, int_left, int_right, "__tmp__"))
                }
            };

            return Box::new(value.as_any_value_enum());
        }
        unimplemented!()
    }
}

impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a super::statement::Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        if let Expression::Binary(left, right, binary_type) = self {
            let parsed_left = left.as_ref().unwrap().visit(data)?.as_any_value_enum();
            let parsed_right = right.as_ref().unwrap().visit(data)?.as_any_value_enum();
            return Some(Self::binary_statement(data, binary_type, parsed_left, parsed_right));
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
            if let Some(param) = data.current_function_params.borrow().get(variable_name) {
                return Some(Box::new(param.as_basic_value_enum()));
            } else {
                let load = data.builder.build_load(self.expression_location(data).unwrap(), variable_name);
                return Some(Box::new(load));
            }
        }

        if let Expression::IntegerLiteral(ref literal) = self {
            let t = data.context.i64_type();
            let value = t.const_int(literal.abs() as u64, true);

            return Some(Box::new(value));
        }

        if let Expression::FloatLiteral(ref literal) = self {
            let t = data.context.f64_type();
            let value = t.const_float(*literal);

            return Some(Box::new(value));
        }

        if let Expression::StringLiteral(ref str) = self {
            let bytes: Vec<_> = str.as_bytes().iter().map(|v| data.context.i8_type().const_int(*v as u64, false)).collect();
            let array = data.context.i8_type().const_array(&bytes);

            return Some(Box::new(array));
        }

        if let Expression::CharLiteral(c) = self {
            let value = data.context.i8_type().const_int(*c as u64, false);

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

        if let Expression::FunctionCall(name, args) = self {
            let function = data.function_table.borrow()[name];
            let params: Vec<BasicValueEnum> = args.iter().map(|v| v.visit(data).unwrap().as_any_value_enum().try_into().unwrap()).collect();
            let mapped: Vec<BasicMetadataValueEnum> = params.iter().map(|v| (*v).into()).collect();
            let call_output = data.builder.build_call(function, &mapped, "__tmp__").try_as_basic_value();
            if let Some(call_value) = call_output.left() {
                let as_any = call_value.as_any_value_enum();
                return Some(Box::new(as_any));
            }
        }
        None
    }
}
