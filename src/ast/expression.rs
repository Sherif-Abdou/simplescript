use crate::ast::{DataType, datatype};
use crate::parsing::DataTypeParser;
use inkwell::{values::{
    AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum,
    FloatValue, IntValue, PointerValue, StructValue,
}, FloatPredicate, IntPredicate, AddressSpace};
use std::{borrow::Borrow, panic::Location};
use std::collections::HashMap;
use inkwell::types::BasicType;

use super::{statement::Statement, Compiler, DataTypeEnum, Scope};

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    expression_enum: ExpressionEnum,
    data_type: Option<DataType>,
}

impl Borrow<ExpressionEnum> for Expression {
    fn borrow(&self) -> &ExpressionEnum {
        &self.expression_enum
    }
}

impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        match &self.expression_enum {
            ExpressionEnum::Binary(left, right, binary_type) => {
                let parsed_left = left.as_ref().unwrap().visit(data)?.as_any_value_enum();
                let parsed_right = right.as_ref().unwrap().visit(data)?.as_any_value_enum();
                return Some(Self::binary_statement(
                    data,
                    binary_type,
                    parsed_left,
                    parsed_right,
                ));
            }
            ExpressionEnum::Unary(Some(interior), operation) => {
                return match operation {
                    UnaryExpressionType::Reference => {
                        Some(Box::new(interior.expression_location(data).unwrap()))
                    }
                    UnaryExpressionType::Dereference => {
                        let location = interior
                            .visit(data)
                            .unwrap()
                            .as_any_value_enum()
                            .into_pointer_value();
                        Some(Box::new(data.builder.build_load(location, "__tmp__")))
                    }
                }
            }
            ExpressionEnum::VariableRead(variable_name) => {
                return if let Some(param) = data.current_function_params.borrow().get(variable_name) {
                    Some(Box::new(param.as_basic_value_enum()))
                } else {
                    let load = data
                        .builder
                        .build_load(self.expression_location(data).unwrap(), variable_name);
                    Some(Box::new(load))
                }
            }
            ExpressionEnum::IntegerLiteral(ref literal) => {
                let t = data.context.i64_type();
                let value = t.const_int(literal.abs() as u64, true);

                return Some(Box::new(value));
            }
            ExpressionEnum::FloatLiteral(ref literal) => {
                let t = data.context.f64_type();
                let value = t.const_float(*literal);

                return Some(Box::new(value));
            }
            ExpressionEnum::StringLiteral(ref str) => {
                let bytes: Vec<_> = str
                    .as_bytes()
                    .iter()
                    .map(|v| data.context.i8_type().const_int(*v as u64, false))
                    .collect();
                // bytes.push(data.context.i8_type().const_zero());
                let array = data.context.i8_type().const_array(&bytes);

                return Some(Box::new(array));
            }
            ExpressionEnum::CharLiteral(c) => {
                let value = data.context.i8_type().const_int(*c as u64, false);

                return Some(Box::new(value));
            }
            ExpressionEnum::StructLiteral(_) => {
                let data_type = self.data_type.as_ref().unwrap();
                let t = data_type.produce_llvm_type(data.context).as_basic_type_enum();
                return Some(Box::new(t.const_zero()))
            }
            ExpressionEnum::Array(ref values) => {
                let expressions: Vec<Box<dyn AnyValue>> =
                    values.iter().map(|v| v.visit(data)).flatten().collect();
                if expressions.is_empty() {
                    return None;
                }
                let array_value_result: ArrayValue = match expressions[0].as_any_value_enum() {
                    AnyValueEnum::ArrayValue(ref v) => {
                        let mapped: Vec<ArrayValue> = expressions
                            .iter()
                            .map(|v| v.as_any_value_enum().into_array_value())
                            .collect();
                        let value = v.get_type().const_array(mapped.as_slice());
                        value
                    }
                    AnyValueEnum::IntValue(ref v) => {
                        let mapped: Vec<IntValue> = expressions
                            .iter()
                            .map(|v| v.as_any_value_enum().into_int_value())
                            .collect();
                        let value = v.get_type().const_array(mapped.as_slice());
                        value
                    }
                    AnyValueEnum::FloatValue(ref v) => {
                        let mapped: Vec<FloatValue> = expressions
                            .iter()
                            .map(|v| v.as_any_value_enum().into_float_value())
                            .collect();
                        let value = v.get_type().const_array(mapped.as_slice());
                        value
                    }
                    AnyValueEnum::PhiValue(_) => todo!(),
                    AnyValueEnum::FunctionValue(_) => todo!(),
                    AnyValueEnum::PointerValue(ref v) => {
                        let mapped: Vec<PointerValue> = expressions
                            .iter()
                            .map(|v| v.as_any_value_enum().into_pointer_value())
                            .collect();
                        let value = v.get_type().const_array(mapped.as_slice());
                        value
                    }
                    AnyValueEnum::StructValue(ref v) => {
                        let mapped: Vec<StructValue> = expressions
                            .iter()
                            .map(|v| v.as_any_value_enum().into_struct_value())
                            .collect();
                        let value = v.get_type().const_array(mapped.as_slice());
                        value
                    }
                    _ => panic!("Unexpected type")
                };
                return Some(Box::new(array_value_result));
            }
            ExpressionEnum::VariableExtract(_, _) => {
                let location = self.expression_location(data).unwrap();

                return Some(Box::new(data.builder.build_load(location, "__tmp__")));
            }
            ExpressionEnum::VariableNamedExtract(location, _) => {
                let location = self.expression_location(data).or_else(|| self.allocate_and_store(data, location))?;
                // dbg!(&data.current_function_params, &self.data_type);
                let dt = self.data_type.as_ref()?.produce_llvm_type(data.context).as_basic_type_enum();
                let adjusted_location = data.builder.build_pointer_cast(location, dt.ptr_type(AddressSpace::default()), "__tmp__");

                return Some(Box::new(data.builder.build_load(adjusted_location, "__tmp__")));
            }
            ExpressionEnum::FunctionCall(name, args) => {
                let function = data.function_table.borrow()[name];
                let params: Vec<BasicValueEnum> = args
                    .iter()
                    .map(|v| {
                        v.visit(data)
                            .unwrap()
                            .as_any_value_enum()
                            .try_into()
                            .unwrap()
                    })
                    .collect();
                let mapped: Vec<BasicMetadataValueEnum> = params.iter().map(|v| (*v).into()).collect();
                let call_output = data
                    .builder
                    .build_call(function, &mapped, "__tmp__")
                    .try_as_basic_value();
                if let Some(call_value) = call_output.left() {
                    let as_any = call_value.as_any_value_enum();
                    return Some(Box::new(as_any));
                }
            }
            ExpressionEnum::ExpressionCast(_, _) => return self.visit_cast(data),
            _ => (),
        }
        None
    }
}

impl From<ExpressionEnum> for Expression {
    fn from(expression_enum: ExpressionEnum) -> Self {
        Self {
            expression_enum,
            data_type: None,
        }
    }
}

impl From<Expression> for ExpressionEnum {
    fn from(expr: Expression) -> Self {
        expr.expression_enum
    }
}

impl Expression {
    pub fn attach_data_types(&mut self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) {
        match &mut self.expression_enum {
            ExpressionEnum::Binary(Some(l), Some(r), _) => {
                l.attach_data_types(scope, data_types);
                r.attach_data_types(scope, data_types);
            }
            ExpressionEnum::Unary(Some(interior), _) => {
                interior.attach_data_types(scope, data_types);
            }
            ExpressionEnum::ExpressionCast(interior, _) => {
                interior.attach_data_types(scope, data_types);
            }
            ExpressionEnum::Array(values) => {
                for value in values {
                    value.attach_data_types(scope, data_types);
                }
            }
            ExpressionEnum::VariableExtract(location, slot) => {
                location.attach_data_types(scope, data_types);
                slot.attach_data_types(scope, data_types);
            },
            ExpressionEnum::VariableNamedExtract(location, _) => {
                location.attach_data_types(scope, data_types);
            }
            ExpressionEnum::FunctionCall(_, params) => {
                for param in params {
                    param.attach_data_types(scope, data_types);
                }
            }
            _ => {}
        };

        let expression_type = self.expression_type(scope, data_types);
        if expression_type.is_none() {
            // dbg!(&self.expression_enum);
        }
        self.data_type = expression_type;
    }

    pub fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> {
        match self.expression_enum {
            ExpressionEnum::VariableExtract(ref location, ref slot) => {

                // let ptr = data.variable_table.borrow()[name];
                let mut ptr: PointerValue = location.expression_location(data)?;
                let slot_value = slot
                    .visit(data)
                    .unwrap()
                    .as_any_value_enum()
                    .into_int_value();
                unsafe {
                    let zero = data.context.i64_type().const_zero();
                    let indices: Vec<IntValue> = match location.data_type {
                        Some(ref v) => {
                            match v.value {
                                DataTypeEnum::Array(_,_) => vec![zero, slot_value],
                                DataTypeEnum::Pointer(_) => {
                                    ptr = data.builder.build_load(ptr, "__tmp__").into_pointer_value();
                                    vec![slot_value]
                                },
                                _ => vec![slot_value],
                            }
                        },
                        None => vec![slot_value],
                    };
                    let new_location = data.builder.build_gep(
                        ptr,
                        &indices,
                        "__tmp__",
                    );

                    return Some(new_location);
                }
            }
            ExpressionEnum::VariableNamedExtract(ref location, ref name) => {
                let dt = location.data_type.clone()?;
                let location = location.expression_location(data)?;

                if let DataTypeEnum::Struct(_, name_map) = &dt.value {
                    return Some(
                        data.builder.build_struct_gep(location, name_map[name] as u32, "__tmp__").unwrap()
                    );
                }
            }
            ExpressionEnum::VariableRead(ref variable_name) => {
                if let Some(p) = data.current_function_params.borrow().get(variable_name) {
                    return Some(p.into_pointer_value());
                }
                let ptr = data.variable_table.borrow()[variable_name];
                return Some(ptr);
            }
            ExpressionEnum::Unary(Some(ref interior), UnaryExpressionType::Dereference) => {
                let dereference = data
                    .builder
                    .build_load(interior.expression_location(data)?, "__tmp__");
                if dereference.is_pointer_value() {
                    let as_ptr_type = dereference.into_pointer_value();

                    return Some(as_ptr_type);
                } else {
                    return interior.expression_location(data);
                }
            }
            _ => (),
        }

        None
    }

    fn binary_statement<'a>(
        data: &'a Compiler,
        binary_type: &'a BinaryExpressionType,
        parsed_left: AnyValueEnum<'a>,
        parsed_right: AnyValueEnum<'a>,
    ) -> Box<AnyValueEnum<'a>> {
        match (parsed_left, parsed_right) {
            (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) => {
                let value = match binary_type {
                    BinaryExpressionType::Addition => {
                        data.builder.build_int_add(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Subtraction => {
                        data.builder.build_int_sub(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Multiplication => {
                        data.builder.build_int_mul(int_left, int_right, "__tmp__")
                    }
                    BinaryExpressionType::Division => data
                        .builder
                        .build_int_signed_div(int_left, int_right, "__tmp__"),
                    _ => {
                        let predicate = match binary_type {
                            BinaryExpressionType::Equal => IntPredicate::EQ,
                            BinaryExpressionType::NotEqual => IntPredicate::NE,
                            BinaryExpressionType::Less => IntPredicate::SLT,
                            BinaryExpressionType::LessEqual => IntPredicate::SLE,
                            BinaryExpressionType::Greater => IntPredicate::SGE,
                            BinaryExpressionType::GreaterEqual => IntPredicate::SGT,
                            _ => unreachable!(),
                        };
                        data.builder
                            .build_int_compare(predicate, int_left, int_right, "__tmp__")
                    }
                };

                return Box::new(value.as_any_value_enum());
            }
            (AnyValueEnum::FloatValue(int_left), AnyValueEnum::FloatValue(int_right)) => {
                let value: Box<dyn AnyValue> = match binary_type {
                    BinaryExpressionType::Addition => {
                        Box::new(data.builder.build_float_add(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Subtraction => {
                        Box::new(data.builder.build_float_sub(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Multiplication => {
                        Box::new(data.builder.build_float_mul(int_left, int_right, "__tmp__"))
                    }
                    BinaryExpressionType::Division => {
                        Box::new(data.builder.build_float_div(int_left, int_right, "__tmp__"))
                    }
                    _ => {
                        let predicate = match binary_type {
                            BinaryExpressionType::Equal => FloatPredicate::OEQ,
                            BinaryExpressionType::NotEqual => FloatPredicate::ONE,
                            BinaryExpressionType::Less => FloatPredicate::OLT,
                            BinaryExpressionType::LessEqual => FloatPredicate::OLE,
                            BinaryExpressionType::Greater => FloatPredicate::OGE,
                            BinaryExpressionType::GreaterEqual => FloatPredicate::OGT,
                            _ => unreachable!(),
                        };
                        Box::new(
                            data.builder
                                .build_float_compare(predicate, int_left, int_right, "__tmp__"),
                        )
                    }
                };

                return Box::new(value.as_any_value_enum());
            }
            _ => (),
        }
        unimplemented!()
    }

    pub fn data_type(
        &self,
        scope: &dyn Scope,
        _data_types: &HashMap<String, DataType>,
    ) -> Option<String> {
        return match &self.expression_enum {
            ExpressionEnum::Binary(l, r, _) => {
                if l.as_ref()?.data_type == r.as_ref()?.data_type {
                    return l
                        .as_ref()
                        .or(r.as_ref())?
                        .data_type
                        .as_ref()
                        .map(|v| v.symbol.clone());
                }
                None
            }
            ExpressionEnum::Unary(Some(interior), dt) => {
                let thing = match dt {
                    UnaryExpressionType::Reference => format!(
                        "&{}",
                        interior
                            .data_type
                            .as_ref()
                            .map(|v| v.symbol.clone())?
                    ),
                    UnaryExpressionType::Dereference => {
                        interior.data_type.as_ref()?.symbol[1..].to_string()
                    }
                };
                Some(thing)
            }
            ExpressionEnum::VariableRead(ref v) => {
                Some(scope.get_variable(v)?.data_type.symbol.clone())
            }
            ExpressionEnum::IntegerLiteral(_) => Some("i64".to_string()),
            ExpressionEnum::FloatLiteral(_) => Some("f64".to_string()),
            ExpressionEnum::StringLiteral(ref s) => Some(format!("[char:{}]", s.len())),
            ExpressionEnum::CharLiteral(_) => Some("char".to_string()),
            ExpressionEnum::StructLiteral(ref s) => Some(s.to_string()),
            ExpressionEnum::Array(ref list) => {
                Some(format!(
                    "[{}:{}]",
                    list[0].data_type.as_ref()?.symbol,
                    list.len()
                ))
            }
            ExpressionEnum::VariableExtract(ref location, _) => {
                let data_type = location.data_type.clone()?;
                // For arrays, ignoring structs right now
                match data_type.value {
                    DataTypeEnum::Array(ref a, _) => {
                        Some(a.symbol.clone())
                    },
                    DataTypeEnum::Pointer(ref interior) => {
                        Some(format!("{}", interior.symbol.clone()))
                    },
                    _ => {
                        None
                    }
                }
            },
            ExpressionEnum::VariableNamedExtract(ref location, ref name) => {
                let data_type = location.data_type.clone()?;
                if let DataTypeEnum::Struct(ref slots, ref name_map) = data_type.value {
                    let interior_type = slots[name_map[name] as usize].as_ref();

                    return Some(interior_type.produce_string());
                }
                // if let DataTypeEnum::Pointer(ref interior) = data_type.value {
                //     if let DataTypeEnum::Struct(ref slots, ref name_map) = interior.value {
                //         let interior_type = slots[name_map[name] as usize].as_ref();
                //
                //         return Some(interior_type.produce_string());
                //     }
                // }
                None
            }
            ExpressionEnum::FunctionCall(ref name, _) => {
                let result = scope.return_type_of(name)?.produce_string();
                Some(result)
            }
            ExpressionEnum::ExpressionCast(_, res) => Some(res.clone()),
            _ => None,
        };
    }

    pub fn expression_type(
        &self,
        scope: &dyn Scope,
        data_types: &HashMap<String, DataType>,
    ) -> Option<DataType> {
        let dt_opt = self.data_type(scope, data_types);
        if let Some(dt) = dt_opt {
            let mut data_type_parser = DataTypeParser::new(data_types);

            let data_type = data_type_parser.parse_string(dt);
            return Some(data_type);
        }

        None
    }

    fn allocate_and_store<'a>(&'a self, data: &'a Compiler, expression: &Expression) -> Option<PointerValue<'a>> {
        let produced_type = expression.data_type.as_ref();
        let dt = produced_type?.produce_llvm_type(data.context).as_basic_type_enum();
        let allocated_space = data.builder.build_alloca(dt, "__tmp__");

        let visited: BasicValueEnum = expression.visit(data).unwrap().as_any_value_enum().try_into().unwrap();

        data.builder.build_store(allocated_space, visited);

        Some(allocated_space)
    }

    fn visit_cast<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        let ExpressionEnum::ExpressionCast(interior, resultant) = &self.expression_enum else {
            return None;
        };
        let compiled = interior.visit(data)?.as_any_value_enum();
        if compiled.is_int_value() {
            let integer = compiled.into_int_value();

            let result: Box<dyn AnyValue> = match resultant.as_str() {
                "f64" => Box::new(data.builder.build_signed_int_to_float(
                    integer,
                    data.context.f64_type(),
                    "__tmp__",
                )),
                "i64" => Box::new(data.builder.build_int_cast(
                    integer,
                    data.context.i64_type(),
                    "__tmp__",
                )),
                "char" => Box::new(data.builder.build_int_cast(
                    integer,
                    data.context.i8_type(),
                    "__tmp__",
                )),
                _ => unimplemented!(),
            };

            return Some(result);
        }
        if compiled.is_pointer_value() && resultant.starts_with('&') {
            let element_type = compiled.into_array_value().get_type().get_element_type();
            let pointer_type = element_type.ptr_type(AddressSpace::default());
            let basic_value: BasicValueEnum = compiled.try_into().unwrap();

            let result: Box<dyn AnyValue> = Box::new(
                data.builder.build_bitcast(
                    basic_value,
                    pointer_type,
                    "__tmp__"
                )
            );

            return Some(result);
        }
        None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum ExpressionEnum {
    Binary(
        Option<Box<Expression>>,
        Option<Box<Expression>>,
        BinaryExpressionType,
    ),
    Unary(Option<Box<Expression>>, UnaryExpressionType),
    FunctionCall(String, Vec<Expression>),
    Array(Vec<Expression>),
    VariableRead(String),
    VariableExtract(Box<Expression>, Box<Expression>),
    VariableNamedExtract(Box<Expression>, String),
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    StructLiteral(String),
    CharLiteral(u8),
    ExpressionCast(Box<Expression>, String),
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
    // Higher precedence operations are computed first
    pub fn precedence(&self) -> i64 {
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
    pub fn precedence(&self) -> i64 {
        match self {
            UnaryExpressionType::Reference => 10,
            UnaryExpressionType::Dereference => 10,
        }
    }
}

impl ExpressionEnum {
    pub fn precedence(&self) -> i64 {
        match self {
            ExpressionEnum::Binary(_, _, t) => t.precedence(),
            ExpressionEnum::Unary(_, t) => t.precedence(),
            ExpressionEnum::VariableRead(_) => 100,
            ExpressionEnum::IntegerLiteral(_) => 100,
            ExpressionEnum::FloatLiteral(_) => 100,
            ExpressionEnum::Array(_) => 200,
            ExpressionEnum::VariableExtract(_, _) => 100,
            ExpressionEnum::FunctionCall(_, _) => 100,
            _ => 100,
        }
    }

        pub fn is_binary(&self) -> bool {
        if let ExpressionEnum::Binary(_, _, _) = self {
            return true;
        }
        return false;
    }

    pub fn binary_get_left(&self) -> &Option<Box<Expression>> {
        if let ExpressionEnum::Binary(l, _, _) = self {
            return l;
        }
        panic!()
    }

    pub fn binary_get_right(&self) -> &Option<Box<Expression>> {
        if let ExpressionEnum::Binary(_, r, _) = self {
            return r;
        }
        panic!()
    }

    pub fn binary_set_left(self, expr: Option<ExpressionEnum>) -> ExpressionEnum {
        let front = self;
        let ExpressionEnum::Binary(_, r, t) = front else {
            panic!("Critical Expression Parsing Error");
        };

        let new_expression = ExpressionEnum::Binary(expr.map(|v| Box::new(v.into())), r, t);

        new_expression
    }

    pub fn binary_set_right(self, expr: Option<ExpressionEnum>) -> ExpressionEnum {
        let front = self;
        let ExpressionEnum::Binary(l, _, t) = front else {
            panic!("Critical Expression Parsing Error");
        };

        let new_expression = ExpressionEnum::Binary(l, expr.map(|v| Box::new(v.into())), t);

        new_expression
    }
}
