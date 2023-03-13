use crate::ast::DataType;
use crate::parsing::DataTypeParser;
use inkwell::{
    values::{
        AnyValue, AnyValueEnum, ArrayValue, BasicMetadataValueEnum, BasicValue, BasicValueEnum,
        FloatValue, IntValue, PointerValue, StructValue,
    }, FloatPredicate, IntPredicate,
};
use std::borrow::Borrow;
use std::collections::HashMap;

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
        if let ExpressionEnum::Binary(left, right, binary_type) = &self.expression_enum {
            let parsed_left = left.as_ref().unwrap().visit(data)?.as_any_value_enum();
            let parsed_right = right.as_ref().unwrap().visit(data)?.as_any_value_enum();
            return Some(Self::binary_statement(
                data,
                binary_type,
                parsed_left,
                parsed_right,
            ));
        }

        if let ExpressionEnum::Unary(Some(interior), operation) = &self.expression_enum {
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
        };

        if let ExpressionEnum::VariableRead(variable_name) = &self.expression_enum {
            return if let Some(param) = data.current_function_params.borrow().get(variable_name) {
                Some(Box::new(param.as_basic_value_enum()))
            } else {
                let load = data
                    .builder
                    .build_load(self.expression_location(data).unwrap(), variable_name);
                Some(Box::new(load))
            }
        }

        if let ExpressionEnum::IntegerLiteral(ref literal) = &self.expression_enum {
            let t = data.context.i64_type();
            let value = t.const_int(literal.abs() as u64, true);

            return Some(Box::new(value));
        }

        if let ExpressionEnum::FloatLiteral(ref literal) = &self.expression_enum {
            let t = data.context.f64_type();
            let value = t.const_float(*literal);

            return Some(Box::new(value));
        }

        if let ExpressionEnum::StringLiteral(ref str) = &self.expression_enum {
            let bytes: Vec<_> = str
                .as_bytes()
                .iter()
                .map(|v| data.context.i8_type().const_int(*v as u64, false))
                .collect();
            let array = data.context.i8_type().const_array(&bytes);

            return Some(Box::new(array));
        }

        if let ExpressionEnum::CharLiteral(c) = &self.expression_enum {
            let value = data.context.i8_type().const_int(*c as u64, false);

            return Some(Box::new(value));
        }

        if let ExpressionEnum::Array(ref values) = &self.expression_enum {
            let expressions: Vec<Box<dyn AnyValue>> =
                values.iter().map(|v| v.visit(data)).flatten().collect();
            if expressions.is_empty() {
                return None;
            }
            let thing: ArrayValue = match expressions[0].as_any_value_enum() {
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
                AnyValueEnum::VectorValue(ref v) => todo!(),
                AnyValueEnum::InstructionValue(_) => todo!(),
                AnyValueEnum::MetadataValue(_) => todo!(),
            };
            return Some(Box::new(thing));
        }

        if let ExpressionEnum::VariableExtract(_, _) = &self.expression_enum {
            let location = self.expression_location(data).unwrap();

            return Some(Box::new(data.builder.build_load(location, "__tmp__")));
        }

        if let ExpressionEnum::FunctionCall(name, args) = &self.expression_enum {
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

        if let ExpressionEnum::ExpressionCast(_, _) = &self.expression_enum {
            return self.visit_cast(data);
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
        let expression_type = self.expression_enum.expression_type(scope, data_types);
        self.data_type = expression_type;

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
            ExpressionEnum::VariableExtract(_, slot) => {
                slot.attach_data_types(scope, data_types);
            }
            ExpressionEnum::FunctionCall(_, params) => {
                for param in params {
                    param.attach_data_types(scope, data_types);
                }
            }
            _ => {}
        };
    }

    pub fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> {
        if let ExpressionEnum::VariableExtract(ref location, ref slot) = self.expression_enum {

            // let ptr = data.variable_table.borrow()[name];
            let ptr: PointerValue = location.expression_location(data)?;
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

        if let ExpressionEnum::VariableRead(ref variable_name) = self.expression_enum {
            if let Some(p) = data.current_function_params.borrow().get(variable_name) {
                dbg!("converting p hopefully", p.is_pointer_value());
                return Some(p.into_pointer_value());
            }
            let ptr = data.variable_table.borrow()[variable_name];
            return Some(ptr);
        }

        if let ExpressionEnum::Unary(Some(ref interior), UnaryExpressionType::Dereference) =
            self.expression_enum
        {
            let dereference = data
                .builder
                .build_load(interior.expression_location(data).unwrap(), "__tmp__");
            dbg!(&dereference);
            let as_ptr_type = dereference.into_pointer_value();

            return Some(as_ptr_type);
        }

        None
    }

    fn binary_statement<'a>(
        data: &'a Compiler,
        binary_type: &'a BinaryExpressionType,
        parsed_left: AnyValueEnum<'a>,
        parsed_right: AnyValueEnum<'a>,
    ) -> Box<AnyValueEnum<'a>> {
        if let (AnyValueEnum::IntValue(int_left), AnyValueEnum::IntValue(int_right)) =
            (parsed_left, parsed_right)
        {
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
        if let (AnyValueEnum::FloatValue(int_left), AnyValueEnum::FloatValue(int_right)) =
            (parsed_left, parsed_right)
        {
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
        dbg!(&parsed_left, &parsed_right);
        unimplemented!()
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
    IntegerLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
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

    pub fn data_type(
        &self,
        scope: &dyn Scope,
        _data_types: &HashMap<String, DataType>,
    ) -> Option<String> {
        return match self {
            ExpressionEnum::Binary(l, r, _) => {
                if l.as_ref().unwrap().data_type == r.as_ref().unwrap().data_type {
                    return l
                        .as_ref()
                        .or(r.as_ref())
                        .unwrap()
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
                            .map(|v| v.symbol.clone())
                            .unwrap()
                    ),
                    UnaryExpressionType::Dereference => {
                        interior.data_type.as_ref().unwrap().symbol[1..].to_string()
                    }
                };
                Some(thing)
            }
            ExpressionEnum::VariableRead(v) => {
                Some(scope.get_variable(v).unwrap().data_type.symbol.clone())
            }
            ExpressionEnum::IntegerLiteral(_) => Some("i64".to_string()),
            ExpressionEnum::FloatLiteral(_) => Some("f64".to_string()),
            ExpressionEnum::StringLiteral(ref s) => Some(format!("[char:{}]", s.len())),
            ExpressionEnum::CharLiteral(_) => Some("char".to_string()),
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
                        Some(interior.symbol.clone())
                    },
                    _ => {
                        None
                    }
                }
            }
            ExpressionEnum::FunctionCall(name, _) => {
                let result = scope.return_type_of(name).unwrap().produce_string();
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

        if let ExpressionEnum::FunctionCall(_, _) = self {}
        None
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
