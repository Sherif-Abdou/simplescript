use std::collections::HashMap;

use crate::{ast::{DataType, DataTypeEnum}, lexing::Token};

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
            build_type: None
        }
    }

    pub fn consume(&mut self, token: Token) -> bool {
        match token {
            Token::OpenSquare => {
                if self.build_type.is_none() {
                    self.build_type = Some(BuildType::Array);
                }
            },
            Token::Identifier(iden) => {
                // dbg!(&iden);
                self.internal_type = Some(self.data_types[&iden].clone());
            },
            Token::Colon => {

            },
            Token::Integer(size) => {
                let internal = (self.internal_type.as_ref().unwrap()).clone();
                let new_data_type = DataType {
                    symbol: format!("[{}:{}]", &internal.symbol, &size),
                    value: DataTypeEnum::Array(Box::new(internal), size as u64),
                };

                self.internal_type = Some(new_data_type);
            },
            Token::CloseSquare => return true,
            Token::EOL => return false,
            Token::Comma => return false,
            Token::Equal => return false,
            Token::CloseParenth => return false,
            _ => panic!("Unexpected token")
        }

        true
    }

    pub fn build(&mut self) -> DataType {
        let thing = self.internal_type.as_ref().unwrap().clone();

        thing
    }
}