use crate::ast::{BinaryExpressionType, Expression, ExpressionEnum, Scope, UnaryExpressionType};
use crate::lexing::Token;
use crate::parsing::sub_expression_parser::SubExpressionParser;
use crate::parsing::ParsingResult;
use std::cell::Cell;
use std::collections::VecDeque;
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
    fn new(slots: &'a Vec<Slot>) -> Self {
        Self {
            internal_slots: slots,
            start: Cell::new(0),
            end: Cell::new(slots.len()),
        }
    }

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
    raw: Vec<Token>,
    slots: Vec<Slot>,
    consuming_token_stack: VecDeque<Token>,
    subparser: Option<Box<ExpressionParser<'a>>>,
    scope: Option<&'a dyn Scope>
}

impl<'a> ExpressionParser<'a> {
    fn set_scope(mut self, scope: Option<&'a dyn Scope>) -> Self {
        self.scope = scope;
        self
    }

    /// Looks for slot within the same layer
    fn look_for(slot: Slot, slots: &SlotList) -> Option<usize> {
        let mut layer : usize = 0;
        for (i, current_slot) in slots.iter().enumerate() {
            match current_slot {
                Slot::Token(Token::OpenParenth)
                | Slot::Token(Token::OpenSquare) => layer += 1,
                Slot::Token(Token::CloseParenth)
                | Slot::Token(Token::CloseSquare) => layer -= 1,
                _ => {
                    if *current_slot == slot {
                        return Some(i as usize);
                    }
                }
            };
        }
        None
        // slots.iter().position(|s| slot.eq(s))
    }

    fn handle_parentheses(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        let close_position = self.find_close_parenth(slots);
        if close_position == -1 {return None;}

        let subslots = slots.clone();
        subslots.set_end_to(close_position as usize);
        subslots.shift_by(1);

        slots.shift_by((close_position + 1) as usize);
        self.parse(&subslots)
    }

    fn find_close_parenth(&self, slots: &SlotList) -> i32 {
        let mut layer = 0;
        let mut close_position = -1;
        for (position, slot) in slots.iter().enumerate() {
            match *slot {
                Slot::Token(Token::OpenParenth) => layer += 1,
                Slot::Token(Token::CloseParenth) if layer > 1 => layer -= 1,
                Slot::Token(Token::CloseParenth) if layer == 1 => close_position = position as i32,
                _ => {}
            }
        }
        close_position
    }

    fn handle_square_brackets(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        let close_position = self.find_close_square(slots);
        if close_position == -1 { return None;}

        let subslots = slots.clone();
        subslots.set_end_to(close_position as usize);
        subslots.shift_by(1);

        slots.shift_by((close_position + 1) as usize);
        let things = self.parse_comma_list(&subslots)?;
        Some(ExpressionEnum::Array(things.into_iter().map(|v| v.into()).collect()))
    }

    fn find_close_square(&self, slots: &SlotList) -> i32 {
        let mut layer = 0;
        let mut close_position = -1;
        for (position, slot) in slots.iter().enumerate() {
            match *slot {
                Slot::Token(Token::OpenSquare) => layer += 1,
                Slot::Token(Token::CloseSquare) if layer > 1 => layer -= 1,
                Slot::Token(Token::CloseSquare) if layer == 1 => close_position = position as i32,
                _ => {}
            }
        }
        close_position
    }

    fn check_for_prefix(&self, slots: &SlotList) -> Option<ExpressionEnum> {
        // Look for unary operations
        let unary_type = match slots[0] {
            Slot::Token(Token::Ampersand) => Some(UnaryExpressionType::Reference),
            Slot::Token(Token::Star) => Some(UnaryExpressionType::Dereference),
            Slot::Token(Token::OpenParenth) => return self.handle_parentheses(slots),
            Slot::Token(Token::OpenSquare) => return self.handle_square_brackets(slots),
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
            assert_eq!(slots[0], Slot::Token(Token::OpenParenth));
            let close_position = self.find_close_parenth(slots);
            let subsection = slots.clone();
            slots.set_end_to(close_position as usize);
            slots.shift_by(1);
            let arguments = self.parse_comma_list(&subsection)?;
            let expression = ExpressionEnum::FunctionCall(identifier.clone(),
                arguments.iter().map(|v| (v.clone()).into()).collect());
            return Some(
                expression
            );
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
        self.consume_token(token.clone());
        match &token {
            | Token::EOF
            | Token::EOL
            | Token::OpenCurly => self.consuming_token_stack.push_front(token.clone()),

            Token::ClosedCurly
            | Token::CloseSquare
            | Token::CloseParenth => {
                if self.consuming_token_stack.is_empty() {
                    return Ok(false)
                } else {
                    self.consuming_token_stack.pop_front();
                }
            }
            Token::Comma if self.consuming_token_stack.is_empty() => return Ok(false),
            _ => return Ok(true)
        }

        Ok(true)
    }

    fn build(&mut self) -> Option<ExpressionEnum> {
        let slotlist = SlotList::new(&self.slots);

        self.parse(&slotlist)
    }
}

impl<'a> ExpressionParser<'a> {
    fn consume_token(&mut self, token: Token) {
        if let Some(ref mut subparser) = self.subparser {
            let can_continue = subparser.consume(token.clone());
            if let Ok(false) = can_continue {
                let built = subparser.build();
                self.slots
                    .push(built.map(Slot::Expression).unwrap_or(Slot::None));
                self.slots.push(Slot::Token(token.clone()));
                self.subparser = None;
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
                self.slots.push(Slot::Token(token));
                self.subparser = Some(Box::new(ExpressionParser::default().set_scope(self.scope)))
            }
            // Weird case with comma, need to work with that
            Token::Comma
            | Token::EOF
            | Token::EOL
            | Token::ClosedCurly
            // | Token::CloseSquare
            // | Token::CloseParenth
            | Token::OpenCurly => {},
            token => self.slots.push(Slot::Token(token)),
        };
    }
}

#[cfg(test)]
mod test {
    use crate::lexing::Token;
    use crate::parsing::new_expression_parser::ExpressionParser;
    use crate::parsing::sub_expression_parser::SubExpressionParser;

    #[test]
    fn test_basic_binary() {
        let tokens = vec![Token::Integer(2), Token::Plus, Token::Integer(3)];
        print_parsed(tokens);
    }

    #[test]
    fn test_advanced_binary() {
        let tokens = vec![Token::Integer(2), Token::Plus, Token::Integer(3), Token::Slash, Token::Integer(78)];
        print_parsed(tokens);
    }

    #[test]
    fn test_array() {
        let tokens = vec![Token::OpenSquare, Token::Integer(3), Token::Comma, Token::Float(2.7), Token::CloseSquare];
        print_parsed(tokens);
    }

    fn print_parsed(tokens: Vec<Token>) {
        let mut parser = ExpressionParser::default();

        for token in tokens {
            parser.consume(token).unwrap();
        }

        let expr = parser.build();

        println!("{:?}", expr);
    }
}
