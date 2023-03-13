use std::collections::HashMap;

use crate::{
    ast::{DataType, ExpressionEnum},
    lexing::Token,
};
use crate::parsing::sub_expression_parser::SubExpressionParser;

use super::{
    expression_parser::ExpressionParser, scope_stack::ScopeStack, DataTypeParser, ParsingResult,
};

enum State {
    ParsingDataType,
    ParsingExpression,
}

pub struct ExpressionCastParser<'a> {
    current_data_type: DataTypeParser<'a>,
    to_be_casted: ExpressionParser<'a>,
    data_types: &'a HashMap<String, DataType>,
    scope: &'a ScopeStack,
    state: State,
}

impl<'a> ExpressionCastParser<'a> {
    pub fn new(scope: &'a ScopeStack, data_types: &'a HashMap<String, DataType>) -> Self {
        let mut to_be_casted = ExpressionParser::with_scope_stack(scope);
        to_be_casted.data_types = Some(data_types);
        let current_data_type = DataTypeParser::new(data_types);
        Self {
            state: State::ParsingDataType,
            current_data_type,
            to_be_casted,
            data_types,
            scope,
        }
    }

    pub fn consume(&mut self, token: Token) -> ParsingResult<bool> {
        match self.state {
            State::ParsingDataType => {
                let can_continue = self.current_data_type.consume(token.clone());
                if !can_continue {
                    assert_eq!(token, Token::OpenParenth);
                    self.state = State::ParsingExpression;
                }

                Ok(true)
            }
            State::ParsingExpression => {
                let can_continue = self.to_be_casted.consume(token.clone())?;

                if !can_continue {
                    assert_eq!(token, Token::CloseParenth);
                    return Ok(false);
                }

                Ok(true)
            }
        }
    }

    pub fn build(&mut self) -> ExpressionEnum {
        let expr = self.to_be_casted.build().unwrap();
        let dt = self.current_data_type.build();

        ExpressionEnum::ExpressionCast(Box::new(expr.into()), dt.produce_string())
    }
}
