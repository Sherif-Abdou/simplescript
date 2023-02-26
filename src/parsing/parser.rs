use crate::{lexing::{Lexer, Token}, ast::{Scope, Function, Expression, SetVariable, InsertVariable, ReturnCommand, Variable, DataType}};
use std::{collections::{VecDeque, HashMap}, error::Error, fmt::Display, cell::RefCell, ops::IndexMut};
use inkwell::values::InstructionOpcode::InsertValue;
use regex::Regex;
use std::any::{Any, TypeId};
use crate::ast::{RootScope, Statement};
use crate::parsing::ParsingError::MissingToken;

use super::{scope_stack::ScopeStack, expression_parser::ExpressionParser, data_type_parser::DataTypeParser};

const ARRAY_REGEX: &'static str = r"\[(.*):(\d+)\]";

pub struct Parser {
    lexer: RefCell<Lexer>,
    current_token: RefCell<Token>,
    scope_stack: ScopeStack,
    pub data_types: HashMap<String, DataType>,
}

pub type ParsingResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum ParsingError {
    MissingToken
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
        data_types.insert("i64".to_string(), DataType {
            symbol: "i64".to_string(),
            value: crate::ast::DataTypeEnum::Primitive,
        });
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
            } else if let Token::Identifier(ref name) = self.current_token() {
                let expression = self.parse_expression_choice(false).expect("Couldn't parse expected expression");
                if let Expression::VariableRead(ref iden) = expression {
                    self.parse_set_variable(iden)?;
                } else {
                    self.parse_insert_value(expression)?;
                }
            } else if self.current_token() == Token::Star {
                let expression = self.parse_expression_choice(false).expect("Couldn't parse expected expression");
                // dbg!(&expression);
                self.parse_insert_value(expression)?;
            } else if Token::ClosedCurly == self.current_token() {
                let thing = self.scope_stack.pop_front().unwrap();
                self.scope_stack.peek_front_mut().unwrap().commands_mut().push(thing);
            }
            self.next();
        }

        Ok(self.scope_stack.pop_front().unwrap())
    }

    fn parse_return(&mut self) -> ParsingResult<()> {
        if self.current_token() != Token::Return {
            return Err(Box::new(ParsingError::MissingToken));
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
        while expr_parser.consume(self.current_token())? {
            self.next();
        }

        Ok(expr_parser.build().unwrap())
    }

    fn parse_set_variable(&mut self, iden: &str) -> ParsingResult<()> {
        let mut val = self.current_token();
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
            while data_type_parser.consume(self.next()) {
            }
            let data_type = data_type_parser.build();
            let variable = Variable {
                name: iden.to_string(),
                data_type
            };
            // dbg!("setting variable");
            self.scope_stack.set_variable(variable);
        }
        if self.current_token() != Token::Equal {
            dbg!("Missing equal");
            dbg!(&self.current_token());
            return Err(Box::new(ParsingError::MissingToken));
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
        let data_type = self.scope_stack.get_variable(iden).expect("Missing variable").data_type.clone();
        let stmt = SetVariable::new(iden.to_string(), data_type, expr);
        self.scope_stack.commands_mut().push(Box::new(stmt));
        Ok(())
    }

    fn parse_insert_value(&mut self, location: Expression) -> ParsingResult<()> {
        if self.current_token() != Token::Equal {return Err(Box::new(MissingToken))}
        self.next();

        let expr = self.parse_expression()?;
        let stmt = InsertVariable::new(location, expr);

        self.scope_stack.commands_mut().push(Box::new(stmt));
        Ok(())
    }

    fn expression_type(&mut self, expr: &Expression) -> DataType {
        // let symbol = expr.data_type(&self.scope_stack).expect("Can't infer data type").to_string();
        // let re = Regex::new(ARRAY_REGEX).unwrap();
        // if let Some(captures) = re.captures(&symbol) {
        //     let interior_type = captures.get(1).unwrap().as_str();
        //     let count: u64 = captures.get(2).unwrap().as_str().parse().unwrap();
        //     let data_type = DataType {
        //         symbol: symbol.clone(),
        //         value: crate::ast::DataTypeEnum::Array(Box::new(self.data_types[interior_type].clone()), count),
        //     };
        //     self.data_types.insert(symbol.clone(), data_type.clone());
        //     return data_type;
        // }
        let mut data_type_parser = DataTypeParser::new(&self.data_types);
        let thing = expr.data_type(&self.scope_stack).unwrap();
        // dbg!(&thing);
        let data_type = data_type_parser.parse_string(thing);
        data_type
    }

    fn parse_function(&mut self) -> ParsingResult<()> {
        if self.current_token() != Token::Def {
            return Err(Box::new(ParsingError::MissingToken));
        }

        let mut func_name = String::new();

        {
            let Token::Identifier(fn_name) = self.next().clone() else {
                return Err(Box::new(ParsingError::MissingToken));
            };

            func_name = fn_name.clone();
        }

        if Token::OpenParenth != self.next() {
            return Err(Box::new(ParsingError::MissingToken));
        };

        let Token::CloseParenth = self.next() else {
            return Err(Box::new(ParsingError::MissingToken));
        };

        let Token::OpenCurly = self.next() else {
            return Err(Box::new(ParsingError::MissingToken));
        };

        let mut function = Function::default();
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