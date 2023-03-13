use std::{collections::HashMap};

use crate::{
    ast::{DataType, DataTypeEnum},
    lexing::{Lexer, Token},
};

enum BuildType {
    Array,
    Simple,
    Reference,
}
pub struct DataTypeParser<'a> {
    data_types: &'a HashMap<String, DataType>,
    internal_type: Option<DataType>,
    build_type: Option<BuildType>,
}

impl<'a> DataTypeParser<'a> {
    pub fn new(data_types: &'a HashMap<String, DataType>) -> Self {
        Self {
            data_types,
            internal_type: None,
            build_type: None,
        }
    }

    pub fn parse_string(&mut self, string: String) -> DataType {
        let mut lexer = Lexer::new(string);
        let mut token = lexer.next();
        while self.consume(token) {
            token = lexer.next();
        }

        self.build()
    }

    pub fn consume(&mut self, token: Token) -> bool {
        match token {
            Token::OpenSquare => {
                if self.build_type.is_none() {
                    self.build_type = Some(BuildType::Array);
                }
            }
            Token::Identifier(iden) => {
                self.internal_type = Some(self.data_types[&iden].clone());
                if let Some(BuildType::Reference) = &self.build_type {
                    self.internal_type = Some(DataType {
                        symbol: format!("&{}", self.internal_type.as_ref().unwrap().symbol),
                        value: DataTypeEnum::Pointer(Box::new(
                            self.internal_type.as_ref().unwrap().clone(),
                        )),
                    });
                }
            }
            Token::Colon => {}
            Token::Integer(size) => {
                let internal = (self.internal_type.as_ref().unwrap()).clone();
                let new_data_type = DataType {
                    symbol: format!("[{}:{}]", &internal.symbol, &size),
                    value: DataTypeEnum::Array(Box::new(internal), size as u64),
                };

                self.internal_type = Some(new_data_type);
            }
            Token::Ampersand => self.build_type = Some(BuildType::Reference),
            Token::CloseSquare => return true,
            Token::EOL => return false,
            Token::EOF => return false,
            Token::Comma => return false,
            Token::Equal => return false,
            Token::CloseParenth => return false,
            Token::OpenCurly => return false,
            _ => return false,
        }

        true
    }

    pub fn build(&mut self) -> DataType {
        let thing = self.internal_type.as_ref().unwrap().clone();
        thing
    }
}
