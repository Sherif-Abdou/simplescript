use std::collections::VecDeque;

use crate::{lexing::Token, ast::{Expression, Scope}};

use super::{parser::{ParsingResult}, scope_stack::ScopeStack};

pub struct ExpressionParser<'a> {
    // top_expression: Option<Expression>,
    expression_stack: VecDeque<Expression>,
    scope_stack: Option<&'a ScopeStack>,
    parser_stack: VecDeque<ExpressionParser<'a>>,
    waiting_variable_name: Option<String>,
    pub check_stack: bool,
}

impl<'a> ExpressionParser<'a> {
    pub fn new() -> Self {
        Self {
            expression_stack: VecDeque::new(),
            scope_stack: None,
            parser_stack: VecDeque::new(),
            waiting_variable_name: None,
            check_stack: true,
        }
    }

    pub fn with_scope_stack(stack: &'a ScopeStack) -> Self {
        let mut new = Self::new();
        new.scope_stack = Some(stack);
        return new;
    }

    pub fn consume(&mut self, token: Token) -> ParsingResult<bool> {
        // dbg!(&self.expression_stack);
        if !self.parser_stack.is_empty() {
            let can_continue = self.parser_stack.front_mut().unwrap().consume(token.clone())?;
            if can_continue { return Ok(true); }
            let sub_expression = self.parser_stack.pop_front().unwrap().build();
            // dbg!(&self.expression_stack, &sub_expression);

            match token {
                Token::CloseSquare => {
                    if let Some(Expression::Array(_)) = self.expression_stack.front() {
                        let Some(Expression::Array(mut arr)) = self.expression_stack.pop_front() else {
                            panic!();
                        };
                        arr.push(sub_expression);
                        self.expression_stack.push_front(Expression::Array(arr));
                        return Ok(false);
                    } else if let Some(ref name) = self.waiting_variable_name {
                        // dbg!(&self.expression_stack, &sub_expression);
                        let new_value = Expression::VariableExtract(name.clone(), Box::new(sub_expression));
                        self.waiting_variable_name = None;
                        // dbg!(&self.expression_stack);
                        self.append_expr(new_value);
                        // dbg!(&self.expression_stack);
                        return Ok(true);
                    }
                }
                Token::Comma => {
                    if let Expression::Array(mut arr) = self.expression_stack.pop_front().unwrap() {
                        arr.push(sub_expression);
                        self.expression_stack.push_front(Expression::Array(arr));
                        self.parser_stack.push_front(ExpressionParser::with_scope_stack(&self.scope_stack.unwrap()));
                    }
                }
                _ => panic!("Unexpected token in array literal")
            }
            return Ok(true);
        }
        self.check_variable(&token);

        match token {
            Token::Integer(v) => {
                let mini_expr = Expression::IntegerLiteral(v);
                self.append_expr(mini_expr);
            }
            Token::Plus => self.append_expr(Expression::Binary(None, None, crate::ast::BinaryExpressionType::Addition)),
            Token::Minus => self.append_expr(Expression::Binary(None, None, crate::ast::BinaryExpressionType::Subtraction)),
            Token::Star => self.append_expr(Expression::Binary(None, None, crate::ast::BinaryExpressionType::Multiplication)),
            Token::Slash => self.append_expr(Expression::Binary(None, None, crate::ast::BinaryExpressionType::Division)),
            Token::OpenSquare => {
//        dbg!("Open Square reached", &self.expression_stack);
                if self.expression_stack.is_empty() && self.waiting_variable_name.is_none() {
                    let new_parser = ExpressionParser::with_scope_stack(&self.scope_stack.unwrap());
                    self.parser_stack.push_front(new_parser);
                    self.append_expr(Expression::Array(Vec::new()));
                } else {
                    // dbg!(&self.expression_stack);
                    let new_parser = ExpressionParser::with_scope_stack(&self.scope_stack.unwrap());
                    self.parser_stack.push_front(new_parser);
                }
            }
            Token::Identifier(name) => {
                // dbg!("Looking for variable");
                if !self.check_stack {
                    self.waiting_variable_name = Some(name.clone());
                    return Ok(true);
                }
                if let Some(stack) = self.scope_stack {
                    if stack.get_variable(&name).is_some() {
                        // dbg!("Found it");
                        self.waiting_variable_name = Some(name.clone());
//            self.append_expr(Expression::VariableRead(name.clone()));
                        return Ok(true);
                    }
                }
            }
            Token::EOL => return Ok(false),
            Token::Comma => return Ok(false),
            Token::Colon => return Ok(false),
            Token::CloseSquare => return Ok(false),
            Token::Equal => return Ok(false),
            _ => panic!("Didn't expect {:?}", token)
        };

        Ok(true)
    }

    fn check_variable(&mut self, token: &Token) {
        if self.waiting_variable_name.is_some() {
            match token {
                Token::OpenSquare => {}
                _ => {
                    self.append_expr(Expression::VariableRead(self.waiting_variable_name.as_ref().unwrap().clone()));
                    self.waiting_variable_name = None;
                }
            }
        }
    }

    pub fn build(&mut self) -> Expression {
        // dbg!(&self.expression_stack);
        if let Expression::Array(_) = self.expression_stack.front().unwrap() {
            return self.expression_stack.front().unwrap().clone();
        }
        let mut current = self.expression_stack.pop_front();
        while !self.expression_stack.is_empty() {
            if self.front().is_binary() && !self.binary_right() {
                self.binary_set_right(current);
            }
            current = self.expression_stack.pop_front();
        }

        return current.unwrap();
    }

    fn append_expr(&mut self, expression: Expression) {
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
                let new_expr_precidence = expression.precidence();
                let top_expr_precidence = self.front().precidence();

                // Ex: 3+2*5
                if new_expr_precidence >= top_expr_precidence {
                    let tmp_right = self.front().binary_get_right().clone();
                    self.binary_set_right(None);
                    let new_expr = expression.binary_set_left(tmp_right.map(|v| *v));
                    self.expression_stack.push_front(new_expr);
                } else { // Ex: 2*5+3
                    let new_outer_expr = expression.binary_set_left(self.expression_stack.pop_front());
                    self.expression_stack.push_front(new_outer_expr);
                }
            }
        } else if expression.is_binary() {
            let v = expression.binary_set_left(self.expression_stack.pop_front());
            self.expression_stack.push_front(v);
        }
    }

    fn front(&self) -> &Expression {
        return &self.expression_stack[0];
    }

    fn binary_left(&self) -> bool {
        if let Expression::Binary(l, r, t) = self.front() {
            return l.is_some();
        }
        false
    }

    fn binary_set_left(&mut self, expression: Option<Expression>) {
        let v = self.expression_stack.pop_front().unwrap().binary_set_left(expression);
        self.expression_stack.push_front(v);
    }

    fn binary_set_right(&mut self, expression: Option<Expression>) {
        let v = self.expression_stack.pop_front().unwrap().binary_set_right(expression);
        self.expression_stack.push_front(v);
    }


    fn binary_right(&self) -> bool {
        if let Expression::Binary(l, r, t) = self.front() {
            return r.is_some();
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::Expression::*;
    use crate::ast::BinaryExpressionType::*;

    #[test]
    fn can_parse_number() {
        let number = Token::Integer(24);

        let mut expression_parser = ExpressionParser::new();

        expression_parser.consume(number).expect("Some error");
        let expr = expression_parser.build();

        assert_eq!(expr, Expression::IntegerLiteral(24));
    }

    #[test]
    fn can_parse_one_operation() {
        let values = [Token::Integer(24), Token::Plus, Token::Integer(7)];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build();
        assert_eq!(expr, Binary(Some(Box::new(IntegerLiteral(24))), Some(Box::new(IntegerLiteral(7))), Addition))
    }

    #[test]
    fn can_parse_multiple_operations() {
        let values = [Token::Integer(24), Token::Plus, Token::Integer(7), Token::Star, Token::Integer(3)];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build();
        println!("{:?}", expr);
    }

    #[test]
    fn can_parse_multiple_operations_2() {
        let values = [Token::Integer(24), Token::Slash, Token::Integer(7), Token::Plus, Token::Integer(3)];

        let mut expression_parser = ExpressionParser::new();
        for value in values {
            expression_parser.consume(value).expect("Some Error");
        }

        let expr = expression_parser.build();
        println!("{:?}", expr);
    }
}