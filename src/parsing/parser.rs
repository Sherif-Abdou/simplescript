use crate::ast::{Expression, RootScope};
use crate::parsing::ParsingError::MissingToken;
use crate::{
    ast::{
        DataType, ExpressionEnum, Function, IfCondition, InsertVariable, ReturnCommand, Scope,
        SetVariable, Variable,
    },
    lexing::{Lexer, Token},
};
use std::borrow::Borrow;
use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display};
use crate::parsing::sub_expression_parser::SubExpressionParser;

use super::{
    data_type_parser::DataTypeParser, expression_parser::ExpressionParser, scope_stack::ScopeStack,
};

pub struct Parser {
    lexer: RefCell<Lexer>,
    current_token: RefCell<Token>,
    scope_stack: ScopeStack,
    pub data_types: HashMap<String, DataType>,
}

pub type ParsingResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum ParsingError {
    MissingToken,
}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}

impl Error for ParsingError {}

impl Parser {
    pub fn new(raw: String) -> Self {
        let mut lexer = Lexer::new(raw);
        let mut data_types = HashMap::new();
        data_types.insert(
            "i64".to_string(),
            DataType {
                symbol: "i64".to_string(),
                value: crate::ast::DataTypeEnum::Primitive,
            },
        );
        data_types.insert(
            "f64".to_string(),
            DataType {
                symbol: "f64".to_string(),
                value: crate::ast::DataTypeEnum::Primitive,
            },
        );
        data_types.insert(
            "char".to_string(),
            DataType {
                symbol: "char".to_string(),
                value: crate::ast::DataTypeEnum::Primitive,
            },
        );
        let mut scope_stack = ScopeStack::default();
        scope_stack.push_front(Box::new(RootScope::default()));
        Self {
            scope_stack,
            current_token: RefCell::new(lexer.next()),
            lexer: RefCell::new(lexer),
            data_types,
        }
    }

    pub fn parse(&mut self) -> ParsingResult<Box<dyn Scope>> {
        while self.current_token() != Token::EOF {
            if self.current_token() == Token::Def {
                self.parse_function()?
            } else if self.current_token() == Token::Return {
                self.parse_return()?;
            } else if self.current_token() == Token::If {
                self.parse_if_statement()?;
            } else if let Token::Identifier(_) = self.current_token() {
                let expression = self
                    .parse_expression_choice(false)
                    .expect("Couldn't parse expected expression");
                if let ExpressionEnum::VariableRead(ref iden) = expression.borrow() {
                    self.parse_set_variable(iden)?;
                } else {
                    self.parse_insert_value(expression)?;
                }
            } else if self.current_token() == Token::Star {
                let expression = self
                    .parse_expression_choice(false)
                    .expect("Couldn't parse expected expression");
                // dbg!(&expression);
                self.parse_insert_value(expression)?;
            } else if Token::ClosedCurly == self.current_token() {
                let mut thing = self.scope_stack.pop_front().unwrap();
                thing.wrap_up_parsing(self);
                self.scope_stack
                    .peek_front_mut()
                    .unwrap()
                    .commands_mut()
                    .push(thing);
            }
            self.next();
        }

        Ok(self.scope_stack.pop_front().unwrap())
    }

    fn parse_if_statement(&mut self) -> ParsingResult<()> {
        if self.current_token() != Token::If {
            return Err(Box::new(MissingToken));
        }
        let mut _token = self.next();
        let condition = self.parse_expression()?;
        if self.current_token() != Token::OpenCurly {
            return Err(Box::new(MissingToken));
        }

        let condition = IfCondition::new(condition);
        self.scope_stack.push_front(Box::new(condition));
        Ok(())
    }

    fn parse_return(&mut self) -> ParsingResult<()> {
        if self.current_token() != Token::Return {
            return Err(Box::new(MissingToken));
        }
        self.next();
        // // dbg!("Did return");
        let value = self.parse_expression()?;
        let command = ReturnCommand::new(value);
        self.scope_stack.commands_mut().push(Box::new(command));
        Ok(())
    }

    fn parse_expression(&mut self) -> ParsingResult<Expression> {
        self.parse_expression_choice(true)
    }

    fn parse_expression_choice(&mut self, checked: bool) -> ParsingResult<Expression> {
        let mut expr_parser = ExpressionParser::with_scope_stack(&self.scope_stack);
        expr_parser.check_stack = checked;
        expr_parser.data_types = Some(&self.data_types);
        while expr_parser.consume(self.current_token())? {
            self.next();
        }

        let mut built_expression: Expression = expr_parser.build().unwrap().into();
        if checked {
            built_expression.attach_data_types(&self.scope_stack, &self.data_types);
        }
        Ok(built_expression)
    }

    fn parse_set_variable(&mut self, iden: &str) -> ParsingResult<()> {
        let val = self.current_token();
        if val == Token::Colon {
            // let data_type_iden = self.next();
            // if let Token::Identifier(ref data_iden) = data_type_iden {
            //     let variable = Variable {
            //         name: iden.to_string(),
            //         data_type: self.data_types[data_iden].clone(),
            //     };
            //     // dbg!("setting variable");
            //     self.scope_stack.set_variable(variable);
            //     val = self.next()
            // } else {
            //     panic!("Missing data type");
            // }
            let mut data_type_parser = DataTypeParser::new(&self.data_types);
            while data_type_parser.consume(self.next()) {}
            let data_type = data_type_parser.build();
            let variable = Variable {
                name: iden.to_string(),
                data_type,
            };
            // dbg!("setting variable");
            self.scope_stack.set_variable(variable);
        }
        if self.current_token() != Token::Equal {
            dbg!("Missing equal");
            dbg!(&self.current_token());
            return Err(Box::new(MissingToken));
        }
        self.next();
        let expr = self.parse_expression()?;
        if self.scope_stack.get_variable(iden).is_none() {
            let variable = Variable {
                name: iden.to_string(),
                data_type: self.expression_type(&expr),
            };
            // dbg!("setting variable");
            self.scope_stack.set_variable(variable);
        }
        let data_type = self
            .scope_stack
            .get_variable(iden)
            .expect("Missing variable")
            .data_type
            .clone();
        let stmt = SetVariable::new(iden.to_string(), data_type, expr);
        self.scope_stack.commands_mut().push(Box::new(stmt));
        Ok(())
    }

    fn parse_insert_value(&mut self, location: Expression) -> ParsingResult<()> {
        if self.current_token() != Token::Equal {
            return Err(Box::new(MissingToken));
        }
        self.next();

        let expr = self.parse_expression()?;
        let stmt = InsertVariable::new(location, expr);

        self.scope_stack.commands_mut().push(Box::new(stmt));
        Ok(())
    }

    fn expression_type(&mut self, expr: &Expression) -> DataType {
        // let mut data_type_parser = DataTypeParser::new(&self.data_types);
        // let thing = expr.data_type(&self.scope_stack).unwrap();

        // let data_type = data_type_parser.parse_string(thing);
        // data_type
        let borrowed: &ExpressionEnum = expr.borrow();
        borrowed
            .expression_type(&self.scope_stack, &self.data_types)
            .unwrap()
    }

    fn parse_function(&mut self) -> ParsingResult<()> {
        if self.current_token() != Token::Def {
            return Err(Box::new(MissingToken));
        }

        let mut func_name = String::new();

        {
            let Token::Identifier(fn_name) = self.next() else {
                return Err(Box::new(MissingToken));
            };

            func_name = fn_name;
        }

        if Token::OpenParenth != self.next() {
            return Err(Box::new(MissingToken));
        };
        let mut next = self.next();
        let mut params = Vec::new();
        while next != Token::CloseParenth {
            let Token::Identifier(iden) = next.clone() else {
                return Err(Box::new(MissingToken));
            };
            next = self.next();
            let Token::Colon = next.clone() else {
                return Err(Box::new(MissingToken));
            };
            next = self.next();
            let mut dt_parser = DataTypeParser::new(&self.data_types);
            while dt_parser.consume(next.clone()) {
                next = self.next();
            }
            let dt = dt_parser.build();
            params.push((iden, dt));
            if next == Token::Comma {
                next = self.next();
            }
        }

        next = self.next();
        let mut return_type = None;
        if next == Token::Colon {
            next = self.next();
            let mut data_type_parser = DataTypeParser::new(&self.data_types);
            while data_type_parser.consume(next.clone()) {
                next = self.next();
            }
            return_type = Some(data_type_parser.build());
        }
        let Token::OpenCurly = next else {
            return Err(Box::new(MissingToken));
        };

        let mut function = Function::new(return_type.clone());
        for (name, dt) in &params {
            function.variables.insert(
                name.clone(),
                Variable {
                    name: name.clone(),
                    data_type: dt.clone(),
                },
            );
        }
        function.params = params;
        self.scope_stack
            .add_function(&func_name, return_type);
        function.name = func_name.to_string();
        self.scope_stack.push_front(Box::new(function));

        Ok(())
    }

    fn next(&self) -> Token {
        let a = RefCell::new(self.lexer.borrow_mut().next());
        self.current_token.swap(&a);
        self.current_token()
    }

    fn current_token(&self) -> Token {
        return self.current_token.borrow().clone();
    }
}
