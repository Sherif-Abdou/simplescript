use crate::ast::ExpressionEnum;
use crate::lexing::Token;
use crate::parsing::ParsingResult;

pub trait SubExpressionParser<'a> {
    fn consume(&mut self, token: Token) -> ParsingResult<bool>;
    fn build(&mut self) -> Option<ExpressionEnum>;
}