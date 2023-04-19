use crate::ast::{DataType, datatype};
use crate::parsing::DataTypeParser;
use inkwell::{values::{
    AnyValue, BasicValueEnum, PointerValue,
}};
use std::borrow::BorrowMut;
use std::{borrow::Borrow};
use std::collections::HashMap;
use inkwell::types::BasicType;

use super::expressions::{ExpressionStatement, BinaryExpression, UnaryExpression, FunctionCallExpression, ArrayExpression, VariableReadExpression, VariableExtractExpression, VariableNamedExtractExpression, IntegerExpression, ExpressionCastExpression, CharExpression, StringExpression, FloatExpression, StructExpression};
use super::{statement::Statement, Compiler, DataTypeEnum, Scope};

#[derive(Clone, PartialEq, Debug)]
pub struct Expression {
    pub expression_enum: ExpressionEnum,
    pub data_type: Option<DataType>,
}

impl Borrow<ExpressionEnum> for Expression {
    fn borrow(&self) -> &ExpressionEnum {
        &self.expression_enum
    }
}

impl ExpressionStatement for Expression {
    fn attach_data_types(&mut self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) {
        Self::attach_data_types(self, scope, data_types)
    }

    fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> {
        Self::expression_location(self, data)
    }

    fn data_type(&self, scope: &dyn Scope, data_types: &HashMap<String, DataType>) -> Option<String> {
        Self::data_type(self, scope, data_types)
    }
}

impl Statement for Expression {
    fn visit<'a>(&'a self, data: &'a Compiler) -> Option<Box<dyn AnyValue + 'a>> {
        let expression_statement: &dyn ExpressionStatement = self.expression_enum.borrow();
        expression_statement.visit(data)
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
        BorrowMut::<dyn ExpressionStatement>::borrow_mut(&mut self.expression_enum).attach_data_types(scope, data_types);
        let expression_type = self.expression_type(scope, data_types);
        if let Some(dt)= expression_type.clone() {
            BorrowMut::<dyn ExpressionStatement>::borrow_mut(&mut self.expression_enum).set_data_type(dt);
        }
        if expression_type.is_none() {
            // dbg!(&self.expression_enum);
        }
        self.data_type = expression_type;
    }

    pub fn expression_location<'a>(&'a self, data: &'a Compiler) -> Option<PointerValue<'a>> {
        let expression_statement: &dyn ExpressionStatement = self.expression_enum.borrow();

        expression_statement.expression_location(data)
    }



    pub fn data_type(
        &self,
        scope: &dyn Scope,
        _data_types: &HashMap<String, DataType>,
    ) -> Option<String> {
        let expression_statement: &dyn ExpressionStatement = self.expression_enum.borrow();

        expression_statement.data_type(scope, _data_types)
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

}

#[derive(Clone, Debug)]
pub enum ExpressionEnum {
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    FunctionCall(FunctionCallExpression),
    Array(ArrayExpression),
    VariableRead(VariableReadExpression),
    VariableExtract(VariableExtractExpression),
    VariableNamedExtract(VariableNamedExtractExpression),
    IntegerLiteral(IntegerExpression),
    FloatLiteral(FloatExpression),
    StringLiteral(StringExpression),
    CharLiteral(CharExpression),
    StructLiteral(StructExpression),
    ExpressionCast(ExpressionCastExpression),
}

impl PartialEq for ExpressionEnum {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Binary(l0), Self::Binary(r0)) => l0 == r0,
            (Self::Unary(l0), Self::Unary(r0)) => l0 == r0,
            (Self::FunctionCall(l0), Self::FunctionCall(r0)) => l0 == r0,
            (Self::Array(l0), Self::Array(r0)) => l0 == r0,
            (Self::VariableRead(l0), Self::VariableRead(r0)) => l0 == r0,
            (Self::VariableExtract(l0), Self::VariableExtract(r0)) => l0 == r0,
            (Self::VariableNamedExtract(l0), Self::VariableNamedExtract(r0)) => l0 == r0,
            (Self::IntegerLiteral(l0), Self::IntegerLiteral(r0)) => l0 == r0,
            (Self::FloatLiteral(l0), Self::FloatLiteral(r0)) => l0 == r0,
            (Self::StringLiteral(l0), Self::StringLiteral(r0)) => l0 == r0,
            (Self::CharLiteral(l0), Self::CharLiteral(r0)) => l0 == r0,
            (Self::ExpressionCast(l0), Self::ExpressionCast(r0)) => l0 == r0,
            (Self::StructLiteral(l0), Self::StructLiteral(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Borrow<dyn ExpressionStatement> for ExpressionEnum {
    fn borrow(&self) -> &dyn ExpressionStatement {
        match self {
            ExpressionEnum::Binary(v) => v,
            ExpressionEnum::Unary(v) => v,
            ExpressionEnum::FunctionCall(v) => v,
            ExpressionEnum::Array(v) => v,
            ExpressionEnum::VariableRead(v) => v,
            ExpressionEnum::VariableExtract(v) => v,
            ExpressionEnum::VariableNamedExtract(v) => v,
            ExpressionEnum::IntegerLiteral(v) => v,
            ExpressionEnum::FloatLiteral(v) => v,
            ExpressionEnum::StringLiteral(v) => v,
            ExpressionEnum::CharLiteral(v) => v,
            ExpressionEnum::ExpressionCast(v) => v,
            ExpressionEnum::StructLiteral(v) => v,
        }
    }
}
impl BorrowMut<dyn ExpressionStatement> for ExpressionEnum {
    fn borrow_mut(&mut self) -> &mut dyn ExpressionStatement {
        match self {
            ExpressionEnum::Binary(v) => v,
            ExpressionEnum::Unary(v) => v,
            ExpressionEnum::FunctionCall(v) => v,
            ExpressionEnum::Array(v) => v,
            ExpressionEnum::VariableRead(v) => v,
            ExpressionEnum::VariableExtract(v) => v,
            ExpressionEnum::VariableNamedExtract(v) => v,
            ExpressionEnum::IntegerLiteral(v) => v,
            ExpressionEnum::FloatLiteral(v) => v,
            ExpressionEnum::StringLiteral(v) => v,
            ExpressionEnum::CharLiteral(v) => v,
            ExpressionEnum::ExpressionCast(v) => v,
            ExpressionEnum::StructLiteral(v) => v,
        }
    }
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UnaryExpressionType {
    Reference,
    Dereference,
}
