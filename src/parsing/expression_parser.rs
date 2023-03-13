use std::collections::{HashMap, VecDeque};


use crate::{
    ast::{DataType, ExpressionEnum, Scope, UnaryExpressionType},
    lexing::Token,
};
use crate::ast::ExpressionEnum::VariableRead;

use crate::parsing::sub_expression_parser::SubExpressionParser;

use super::{
    expression_cast_parser::ExpressionCastParser, function_call_parser::FunctionCallParser,
    parser::ParsingResult, scope_stack::ScopeStack,
};

enum WaitingUnaryTypes {
    Reference,
    Dereference,
    Negation,
}

pub struct ExpressionParser<'a> {
    // top_expression: Option<Expression>,
    expression_stack: VecDeque<ExpressionEnum>,
    scope_stack: Option<&'a ScopeStack>,
    parser_stack: VecDeque<Box<dyn SubExpressionParser<'a> + 'a>>,
    waiting_variable_name: Option<String>,
    waiting_unary_operation: Option<WaitingUnaryTypes>,
    waiting_data_type_parser: Option<Box<ExpressionCastParser<'a>>>,
    pub data_types: Option<&'a HashMap<String, DataType>>,
    was_last_binary: bool,
    pub check_stack: bool,
}

impl<'a> ExpressionParser<'a> {
    pub fn new() -> Self {
        Self {
            expression_stack: VecDeque::new(),
            scope_stack: None,
            parser_stack: VecDeque::new(),
            waiting_variable_name: None,
            waiting_unary_operation: None,
            waiting_data_type_parser: None,
            data_types: None,
            was_last_binary: false,
            check_stack: true,
        }
    }

    pub fn with_scope_stack(stack: &'a ScopeStack) -> Self {
        let mut new = Self::new();
        new.scope_stack = Some(stack);
        new
    }

    fn check_variable(&mut self, token: &Token) {
        if self.waiting_variable_name.is_some() {
            match token {
                Token::OpenSquare => {}
                _ => {
                    self.append_expr(ExpressionEnum::VariableRead(
                        self.waiting_variable_name.as_ref().unwrap().clone(),
                    ));
                    self.waiting_variable_name = None;
                }
            }
        }
    }



    fn append_expr(&mut self, expression: ExpressionEnum) {
        self.was_last_binary = expression.is_binary();
        if self.waiting_unary_operation.is_some() {
            let new_expression = match self.waiting_unary_operation.as_ref().unwrap() {
                WaitingUnaryTypes::Reference => ExpressionEnum::Unary(
                    Some(Box::new(expression.into())),
                    UnaryExpressionType::Reference,
                ),
                WaitingUnaryTypes::Dereference => ExpressionEnum::Unary(
                    Some(Box::new(expression.into())),
                    UnaryExpressionType::Dereference,
                ),
                WaitingUnaryTypes::Negation => todo!(),
            };
            self.waiting_unary_operation = None;
            return self.append_expr(new_expression);
        }

        if self.expression_stack.is_empty() {
            self.expression_stack.push_front(expression);
            return;
        }

        if self.front().is_binary() {
            if !self.binary_left() {
                self.binary_set_left(Some(expression));
            } else if !self.binary_right() {
                self.binary_set_right(Some(expression));
            } else if expression.is_binary() {
                let new_expr_precedence = expression.precedence();
                let top_expr_precedence = self.front().precedence();

                // Ex: 3+2*5
                if new_expr_precedence >= top_expr_precedence {
                    let tmp_right = self.front().binary_get_right().clone();
                    self.binary_set_right(None);
                    let new_expr = expression.binary_set_left(tmp_right.map(|v| (*v).into()));
                    self.expression_stack.push_front(new_expr);
                } else {
                    // Ex: 2*5+3
                    let new_outer_expr =
                        expression.binary_set_left(self.expression_stack.pop_front());
                    self.expression_stack.push_front(new_outer_expr);
                }
            }
        } else if expression.is_binary() {
            let v = expression.binary_set_left(self.expression_stack.pop_front());
            self.expression_stack.push_front(v);
        }
    }

    fn front(&self) -> &ExpressionEnum {
        &self.expression_stack[0]
    }

    fn binary_left(&self) -> bool {
        if let ExpressionEnum::Binary(l, _, _) = self.front() {
            return l.is_some();
        }
        false
    }

    fn binary_set_left(&mut self, expression: Option<ExpressionEnum>) {
        let v = self
            .expression_stack
            .pop_front()
            .unwrap()
            .binary_set_left(expression);
        self.expression_stack.push_front(v);
    }

    fn binary_set_right(&mut self, expression: Option<ExpressionEnum>) {
        let v = self
            .expression_stack
            .pop_front()
            .unwrap()
            .binary_set_right(expression);
        self.expression_stack.push_front(v);
    }

    fn binary_right(&self) -> bool {
        if let ExpressionEnum::Binary(_, r, _) = self.front() {
            return r.is_some();
        }
        false
    }
}

impl<'a> SubExpressionParser<'a> for ExpressionParser<'a> {
    fn consume(&mut self, token: Token) -> ParsingResult<bool> {
        if let Some(ref mut parser) = self.waiting_data_type_parser {
            if !parser.consume(token)? {
                let res = parser.build();
                self.waiting_data_type_parser = None;
                self.append_expr(res);
            }
            return Ok(true);
        }
        if !self.parser_stack.is_empty() {
            let can_continue = self
                .parser_stack
                .front_mut()
                .unwrap()
                .consume(token.clone())?;
            if can_continue {
                return Ok(true);
            }
            let sub_expression = self.parser_stack.pop_front().unwrap().build();

            match token {
                Token::CloseSquare => {
                    if let Some(ExpressionEnum::Array(_)) = self.expression_stack.front() {
                        let Some(ExpressionEnum::Array(mut arr)) = self.expression_stack.pop_front() else {
                            panic!();
                        };
                        if let Some(..) = sub_expression {
                            arr.push(sub_expression.unwrap().into());
                        }
                        self.expression_stack.push_front(ExpressionEnum::Array(arr));
                        return Ok(false);
                    } else if let Some(ref name) = self.waiting_variable_name {
                        let new_value = ExpressionEnum::VariableExtract(
                            Box::new(VariableRead(name.to_string()).into()),
                            Box::new(sub_expression.unwrap().into()),
                        );
                        self.waiting_variable_name = None;
                        self.append_expr(new_value);
                        return Ok(true);
                    }
                }
                Token::Comma => {
                    if let ExpressionEnum::Array(mut arr) =
                        self.expression_stack.pop_front().unwrap()
                    {
                        arr.push(sub_expression.unwrap().into());
                        self.expression_stack.push_front(ExpressionEnum::Array(arr));
                        self.parser_stack
                            .push_front(Box::new(ExpressionParser::with_scope_stack(
                                self.scope_stack.unwrap(),
                            )));
                    }
                }
                _ => {
                    if let Some(new_expression) = sub_expression {
                        self.append_expr(new_expression);
                        self.was_last_binary = false;
                    }
                }
            }
            return Ok(true);
        }
        self.check_variable(&token);

        // dbg!(&token);
        let unary_mode = self.was_last_binary || self.expression_stack.is_empty();

        match token {
            Token::Integer(v) => {
                let mini_expr = ExpressionEnum::IntegerLiteral(v);
                self.append_expr(mini_expr);
            }
            Token::Float(v) => {
                self.append_expr(ExpressionEnum::FloatLiteral(v));
            }
            Token::String(v) => {
                self.append_expr(ExpressionEnum::StringLiteral(v));
            }
            Token::Char(v) => {
                self.append_expr(ExpressionEnum::CharLiteral(v));
            }
            Token::Plus => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Addition,
            )),
            Token::Lesser => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Less,
            )),
            Token::LesserEqual => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::LessEqual,
            )),
            Token::Greater => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Greater,
            )),
            Token::GreaterEqual => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::GreaterEqual,
            )),
            Token::NotEqual => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::NotEqual,
            )),
            Token::DoubleEqual => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Equal,
            )),
            Token::Minus if !unary_mode => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Subtraction,
            )),
            Token::Star if !unary_mode => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Multiplication,
            )),
            Token::Star if unary_mode => {
                self.waiting_unary_operation = Some(WaitingUnaryTypes::Dereference)
            }
            Token::Ampersand if unary_mode => {
                self.waiting_unary_operation = Some(WaitingUnaryTypes::Reference)
            }
            Token::Minus if unary_mode => {
                self.waiting_unary_operation = Some(WaitingUnaryTypes::Negation)
            }
            Token::Slash => self.append_expr(ExpressionEnum::Binary(
                None,
                None,
                crate::ast::BinaryExpressionType::Division,
            )),
            Token::OpenSquare => {
                if self.expression_stack.is_empty() && self.waiting_variable_name.is_none() {
                    let new_parser = ExpressionParser::with_scope_stack(self.scope_stack.unwrap());
                    self.parser_stack.push_front(Box::new(new_parser));
                    self.append_expr(ExpressionEnum::Array(Vec::new()));
                } else {
                    let new_parser = ExpressionParser::with_scope_stack(self.scope_stack.unwrap());
                    self.parser_stack.push_front(Box::new(new_parser));
                }
            }
            Token::Identifier(ref name) => {
                if !self.check_stack {
                    self.waiting_variable_name = Some(name.clone());
                    return Ok(true);
                }
                if let Some(stack) = self.scope_stack {
                    if stack.get_variable(name).is_some() {
                        self.waiting_variable_name = Some(name.clone());
                        //            self.append_expr(Expression::VariableRead(name.clone()));
                        return Ok(true);
                    } else if stack.contains_function(name) {
                        let mut function_parser = Box::new(FunctionCallParser::new(stack));
                        function_parser.consume(Token::Identifier(name.clone()))?;
                        self.parser_stack.push_front(function_parser);
                    } else {
                        self.waiting_data_type_parser = Some(Box::new(ExpressionCastParser::new(
                            self.scope_stack.unwrap(),
                            self.data_types.unwrap(),
                        )));
                        self.waiting_data_type_parser
                            .as_mut()
                            .unwrap()
                            .consume(token)?;
                    }
                } else {
                }
            }
            Token::OpenParenth => {
                self.parser_stack.push_front(
                    self.scope_stack
                        .map(ExpressionParser::with_scope_stack)
                        .map(Box::new)
                        .unwrap_or_else(|| Box::new(ExpressionParser::new())),
                );
            }
            Token::EOL => return Ok(false),
            Token::Comma => return Ok(false),
            Token::Colon => return Ok(false),
            Token::CloseParenth => return Ok(false),
            Token::CloseSquare => return Ok(false),
            Token::ClosedCurly => return Ok(false),
            Token::OpenCurly => return Ok(false),
            Token::Equal => return Ok(false),
            _ => panic!("Didn't expect {:?}", token),
        };

        Ok(true)
    }

    fn build(&mut self) -> Option<ExpressionEnum> {
        // dbg!(&self.expression_stack);
        if let ExpressionEnum::Array(_) = self.expression_stack.front()? {
            return Some(self.expression_stack.front()?.clone());
        }
        let mut current = self.expression_stack.pop_front();
        while !self.expression_stack.is_empty() {
            if self.front().is_binary() && !self.binary_right() {
                self.binary_set_right(current);
            }
            current = self.expression_stack.pop_front();
        }

        Some(current.unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_parse_number() {
        let number = Token::Integer(24);

        let mut expression_parser = ExpressionParser::new();

        expression_parser.consume(number).expect("Some error");
        let expr = expression_parser.build().unwrap();

    }

    #[test]
    fn can_parse_one_operation() {
        let values = [Token::Integer(24), Token::Plus, Token::Integer(7)];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build().unwrap();
    }

    #[test]
    fn can_parse_multiple_operations() {
        let values = [
            Token::Integer(24),
            Token::Plus,
            Token::Integer(7),
            Token::Star,
            Token::Integer(3),
        ];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build();
        println!("{:?}", expr);
    }

    #[test]
    fn can_parse_multiple_operations_2() {
        let values = [
            Token::Integer(24),
            Token::Slash,
            Token::Integer(7),
            Token::Plus,
            Token::Integer(3),
        ];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build();
        println!("{:?}", expr);
    }
}
