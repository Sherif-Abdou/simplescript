use super::{ExpressionEnum, Expression, BinaryExpressionType, UnaryExpressionType, expressions::{BinaryExpression, UnaryExpression, FunctionCallExpression, ArrayExpression, VariableReadExpression, VariableExtractExpression, VariableNamedExtractExpression, IntegerExpression, FloatExpression, StringExpression, StructExpression, CharExpression, ExpressionCastExpression}};


type OperationExpressionContainer = Option<Box<Expression>>;

#[derive(Default)]
pub struct ExpressionBuilder {
    internal_enum: Option<ExpressionEnum>
}

impl ExpressionBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn build(self) -> Expression {
        self.internal_enum.unwrap().into()
    }

    pub fn binary(mut self, left: OperationExpressionContainer, right: OperationExpressionContainer, binary_type: BinaryExpressionType) -> Self {
        self.internal_enum = Some(ExpressionEnum::Binary(BinaryExpression::new(left, right, binary_type)));
        self
    }

    pub fn unary(mut self, internal: OperationExpressionContainer, unary_type: UnaryExpressionType) -> Self {
        self.internal_enum = Some(ExpressionEnum::Unary(UnaryExpression::new(internal, unary_type)));
        self
    }

    pub fn function_call(mut self, name: impl Into<String>, arguments: Vec<Expression>) -> Self {
        self.internal_enum = Some(ExpressionEnum::FunctionCall(FunctionCallExpression::new(name.into(), arguments)));
        self
    }

    pub fn array_literal(mut self, elements: Vec<Expression>) -> Self {
        self.internal_enum = Some(ExpressionEnum::Array(ArrayExpression::new(elements)));
        self
    }

    pub fn variable_read(mut self, string: impl Into<String>) -> Self {
        self.internal_enum = Some(ExpressionEnum::VariableRead(VariableReadExpression::new(string.into())));
        self
    }

    pub fn variable_extract(mut self, location: Box<Expression>, index: Box<Expression>) -> Self {
        self.internal_enum = Some(ExpressionEnum::VariableExtract(VariableExtractExpression::new(location, index)));
        self
    }

    pub fn variable_named_extract(mut self, location: Box<Expression>, label: impl Into<String>) -> Self {
        self.internal_enum = Some(ExpressionEnum::VariableNamedExtract(VariableNamedExtractExpression::new(*location, label.into())));
        self
    }

    pub fn integer_literal(mut self, integer: i64) -> Self {
        self.internal_enum = Some(ExpressionEnum::IntegerLiteral(IntegerExpression::new(integer)));
        self
    }

    pub fn float_literal(mut self, float: f64) -> Self {
        self.internal_enum = Some(ExpressionEnum::FloatLiteral(FloatExpression::new(float)));
        self
    }

    pub fn string_literal(mut self, string: impl Into<String>) -> Self {
        self.internal_enum = Some(ExpressionEnum::StringLiteral(StringExpression::new(string.into())));
        self
    }

    pub fn struct_literal(mut self, name: impl Into<String>) -> Self {
        self.internal_enum = Some(ExpressionEnum::StructLiteral(StructExpression::new(name.into())));
        self
    }

    pub fn char_literal(mut self, c: u8) -> Self {
        self.internal_enum = Some(ExpressionEnum::CharLiteral(CharExpression::new(c)));
        self
    }

    pub fn expression_cast(mut self, value: Box<Expression>, result_type: impl Into<String>) -> Self {
        self.internal_enum = Some(ExpressionEnum::ExpressionCast(ExpressionCastExpression::new(value, result_type.into())));
        self
    }
}
