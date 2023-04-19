use crate::ast::{Expression, UnaryExpressionType, Statement};

use super::ExpressionStatement;

pub struct UnaryExpression {
    pub interior: Option<Box<Expression>>,
    unary_type: UnaryExpressionType,
}

impl Statement for UnaryExpression {
    fn visit<'a>(&'a self, data: &'a crate::ast::Compiler) -> Option<Box<dyn inkwell::values::AnyValue + 'a>> {
        return match self.unary_type {
            UnaryExpressionType::Reference => {
                let thing = Box::new(self.interior.as_ref()?.expression_location(data).unwrap());
                Some(thing)
            }
            UnaryExpressionType::Dereference => {
                let location = self.interior.as_ref()?
                    .visit(data)
                    .unwrap()
                    .as_any_value_enum()
                    .into_pointer_value();
                Some(Box::new(data.builder.build_load(location, "__tmp__")))
            }
        }
    }
}

impl ExpressionStatement for UnaryExpression {
    fn attach_data_types(&mut self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) {
        if let Some(ref mut interior) = self.interior {
            interior.attach_data_types(scope, data_types);
        }
    }
    fn data_type(&self, scope: &dyn crate::ast::Scope, data_types: &std::collections::HashMap<String, crate::ast::DataType>) -> Option<String> {
        let thing = match self.unary_type {
            UnaryExpressionType::Reference => format!(
                "&{}",
                self.interior.as_ref()?
                    .data_type
                    .as_ref()
                    .map(|v| v.symbol.clone())?
            ),
            UnaryExpressionType::Dereference => {
                self.interior.as_ref()?.data_type.as_ref()?.symbol[1..].to_string()
            }
        };
        Some(thing)
    }
}
