use crate::ast::{BinaryExpressionType, Expression, ExpressionEnum, Scope, UnaryExpressionType};
use crate::lexing::Token;
use crate::parsing::sub_expression_parser::SubExpressionParser;
use crate::parsing::ParsingResult;
use std::cell::Cell;
use std::iter::Skip;
use std::ops::Index;
use std::slice::Iter;

#[derive(Clone, PartialEq, Debug)]
enum Slot {
    Expression(ExpressionEnum),
    Token(Token),
    None,
}

#[derive(Clone, Debug)]
struct SlotList<'a> {
    internal_slots: &'a Vec<Slot>,
    start: Cell<usize>,
    end: Cell<usize>,
}

impl<'a> SlotList<'a> {
    fn shift_by(&self, n: usize) {
        self.start.set(self.start.get() + n)
    }

    fn pop(&self) -> &Slot {
        let reference = &self[0];
        self.shift_by(1);

        reference
    }

    fn set_end_to(&self, new_end: usize) {
        self.end.set(self.start.get() + new_end)
    }

    fn len(&self) -> usize {
        self.end.get() - self.start.get()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> Iter<'_, Slot> {
        self.internal_slots[self.start.get()..self.end.get()].iter()
    }
}

impl<'a> Index<usize> for SlotList<'a> {
    type Output = Slot;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.end.get() - self.start.get() {
            return &Slot::None;
        }
        self.internal_slots.get(index + self.start.get()).unwrap_or(&Slot::None)
    }
}

#[derive(Default)]
struct ExpressionParser<'a> {
    slots: Vec<Slot>,
    subparser: Option<Box<ExpressionParser<'a>>>,
    scope: Option<&'a dyn Scope>
}

impl<'a> ExpressionParser<'a> {
    fn set_scope(mut self, scope: Option<&'a dyn Scope>) -> Self {
        self.scope = scope;
        self
    }
    fn look_for(slot: Slot, slots: &SlotList) -> Option<usize> {
        slots.iter().position(|s| slot.eq(s))
    }
    fn check_for_prefix(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        // Look for unary operations
        let unary_type = match slots[0] {
            Slot::Token(Token::Ampersand) => Some(UnaryExpressionType::Reference),
            Slot::Token(Token::Star) => Some(UnaryExpressionType::Dereference),
            _ => None,
        };
        if let Some(unary_type) = unary_type {
            slots.shift_by(1);
            return self.parse_local(slots)
                .map(|v| ExpressionEnum::Unary(Some(Box::new(v.into())), unary_type));
        }
        None
    }

    fn parse_local(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        let prefix_check = self.check_for_prefix(slots);
        if prefix_check.is_some()  {
            return prefix_check;
        }
        let middle = match &slots[0] {
            Slot::Expression(expr) => Some(expr.clone()),
            Slot::Token(Token::Identifier(iden)) => todo!(),
            _ => return None,
        };

        middle
            .and_then(|expression| self.check_for_postfix(expression, slots))
    }

    fn parse_identifier(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        let Slot::Token(Token::Identifier(identifier)) = slots.pop() else {
            return None;
        };

        let Some(scope) = self.scope else {
            return None;
        };

        if scope.get_variable(identifier).is_some() {
            return Some(
                ExpressionEnum::VariableRead(identifier.to_string())
            );
        }
        if scope.contains_function(identifier) {
            assert_eq!(*slots.pop(), Slot::Token(Token::OpenParenth));
        }
        None
    }

    /// Slot list should not include brackets
    fn parse_comma_list(&self, slots: &SlotList) -> Option<Vec<ExpressionEnum>> {
        let comma_position = slots.iter().position(|v| *v == Slot::Token(Token::Comma));
        if let Some(position) = comma_position {
            let first_item_slots = slots.clone();
            first_item_slots.set_end_to(position);
            let remaining_slots = slots.clone();
            remaining_slots.shift_by(position + 1);
            let first_item = self.parse(&first_item_slots)?;
            let mut remaining_items = self.parse_comma_list(&remaining_slots)?;
            remaining_items.insert(0, first_item);
            return Some(remaining_items);
        }
        self.parse(slots).map(|v| vec![v])
    }

    fn parse(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        let operations = vec![
            Token::DoubleEqual,
            Token::NotEqual,
            Token::Greater,
            Token::GreaterEqual,
            Token::Lesser,
            Token::LesserEqual,
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
        ];
        for operation in &operations {
            if let Some(index) = Self::look_for(Slot::Token(operation.clone()), slots) {
                let left_list = slots.clone();
                left_list.set_end_to(index);
                let left = self.parse(&left_list);

                let right_list = slots.clone();
                right_list.shift_by(index+1);
                let right = self.parse(&right_list);

                return Some(
                    ExpressionEnum::Binary(
                        left.map(|v| v.into()).map(Box::new),
                        right.map(|v| v.into()).map(Box::new),
                            Self::binary_token_to_type(operation.clone()),
                    )
                )
            }
        }
        self.parse_local(slots)
    }

    fn binary_token_to_type(token: Token) -> BinaryExpressionType {
        use BinaryExpressionType::*;
        match token {
            Token::DoubleEqual => Equal ,
            Token::NotEqual => NotEqual,
            Token::Greater => Greater,
            Token::GreaterEqual => GreaterEqual,
            Token::Lesser => Less,
            Token::LesserEqual => LessEqual,
            Token::Plus => Addition,
            Token::Minus => Subtraction,
            Token::Star => Multiplication,
            Token::Slash => Division,
            _ => panic!()
        }
    }

    fn check_for_postfix(
        &self,
        old_expression: ExpressionEnum,
        slots: &SlotList,
    ) -> Option<ExpressionEnum> {
        // Matches a function call
        match (&slots[0], &slots[1], &slots[2]) {
            (Slot::Token(Token::OpenSquare), Slot::Expression(inside), Slot::Token(Token::CloseParenth)) => {
                slots.shift_by(2);
                self.check_for_postfix(ExpressionEnum::VariableExtract(
                    Box::new(old_expression.into()),
                    Box::new(inside.clone().into()),
                ), slots)
            }
            _ => Some(old_expression),
        }
    }
}

impl<'a> SubExpressionParser<'a> for ExpressionParser<'a> {
    fn consume(&mut self, token: Token) -> ParsingResult<bool> {
        if let Some(ref mut subparser) = self.subparser {
            let can_continue = subparser.consume(token.clone())?;
            if !can_continue {
                let built = subparser.build();
                self.slots
                    .push(built.map(Slot::Expression).unwrap_or(Slot::None));
                self.slots.push(Slot::Token(token.clone()));
                self.subparser = None;
                return Ok(true);
            }
        }

        match token {
            Token::String(str) => self
                .slots
                .push(Slot::Expression(ExpressionEnum::StringLiteral(str))),
            Token::Char(chr) => self
                .slots
                .push(Slot::Expression(ExpressionEnum::CharLiteral(chr))),
            Token::Integer(integer) => self
                .slots
                .push(Slot::Expression(ExpressionEnum::IntegerLiteral(integer))),
            Token::Float(float) => self
                .slots
                .push(Slot::Expression(ExpressionEnum::FloatLiteral(float))),
            Token::OpenSquare | Token::OpenParenth => {
                self.subparser = Some(Box::new(ExpressionParser::default().set_scope(self.scope)))
            }
            // Weird case with comma, need to work with that
            Token::Comma
            | Token::EOF
            | Token::EOL
            | Token::ClosedCurly
            | Token::CloseSquare
            | Token::CloseParenth
            | Token::OpenCurly => return Ok(false),
            token => self.slots.push(Slot::Token(token)),
        };

        Ok(true)
    }

    fn build(&mut self) -> Option<ExpressionEnum> {
        todo!()
    }
}
