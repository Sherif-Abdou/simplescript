use super::Token;

pub struct Lexer {
    raw_text: String,
}

impl Lexer {
    pub fn new(raw_text: String) -> Self {
        Self { raw_text }
    }

    fn empty(&self) -> bool {
        self.raw_text.is_empty()
    }

    pub fn next(&mut self) -> Token {
        if self.empty() {
            return Token::EOF;
        }
        let mut current_string = String::new();
        let mut current: char = self.peek().unwrap();
        while current.is_whitespace() && current != '\n' && !self.empty() {
            self.pop();
            if self.peek().is_none() {
                break;
            }
            current = self.peek().unwrap();
        }
        if current == '\n' {
            self.pop();
            return Token::EOL;
        }

        let sc_token = match current {
            '+' => Some(Token::Plus),
            '-' => Some(Token::Minus),
            '*' => Some(Token::Star),
            '/' => Some(Token::Slash),
            '(' => Some(Token::OpenParenth),
            ')' => Some(Token::CloseParenth),
            '{' => Some(Token::OpenCurly),
            '}' => Some(Token::ClosedCurly),
            '&' => Some(Token::Ampersand),
            '=' if self.peek_next() != Some('=') => Some(Token::Equal),
            '=' if self.peek_next() == Some('=') => {
                self.pop();
                Some(Token::DoubleEqual)
            }
            '<' if self.peek_next() != Some('=') => Some(Token::Lesser),
            '<' if self.peek_next() == Some('=') => {
                self.pop();
                Some(Token::LesserEqual)
            }
            '>' if self.peek_next() != Some('=') => Some(Token::Greater),
            '>' if self.peek_next() == Some('=') => {
                self.pop();
                Some(Token::GreaterEqual)
            }
            '!' if self.peek_next() == Some('=') => {
                self.pop();
                Some(Token::NotEqual)
            }
            ':' => Some(Token::Colon),
            '[' => Some(Token::OpenSquare),
            ']' => Some(Token::CloseSquare),
            ',' => Some(Token::Comma),
            _ => None,
        };

        if current == '"' {
            let mut string = String::new();
            let mut escaped = false;
            self.pop();
            while self.peek() != Some('"') {
                if !escaped {
                    if self.peek() == Some('\\') {
                        escaped = true;
                        self.pop();
                    } else {
                        string.push(self.pop());
                    }
                } else {
                    match self.peek().unwrap() {
                        '\\' => string.push(self.pop()),
                        '"' => string.push(self.pop()),
                        'n' => {
                            string.push('\n');
                            self.pop();
                        }
                        _ => panic!("Invalid Escaped Character"),
                    };
                    escaped = false;
                }
            }
            self.pop();
            return Token::String(string);
        }
        if current == '\'' {
            let mut string = String::new();
            let mut escaped = false;
            self.pop();
            while self.peek() != Some('\'') {
                if !escaped {
                    if self.peek() == Some('\\') {
                        escaped = true;
                        self.pop();
                    } else {
                        string.push(self.pop());
                    }
                } else {
                    match self.peek().unwrap() {
                        '\\' => string.push(self.pop()),
                        '"' => string.push(self.pop()),
                        'n' => {
                            string.push('\n');
                            self.pop();
                        }
                        _ => panic!("Invalid Escaped Character"),
                    };
                    escaped = false;
                }
            }
            assert!(string.len() == 1);
            self.pop();
            return Token::Char(string.as_bytes()[0].clone());
        }
        if let Some(token) = sc_token {
            self.pop();
            return token;
        }

        if current.is_numeric() {
            while (current.is_numeric() || current == '.') && !self.empty() {
                current_string.push(self.pop());
                if self.peek().is_none() {
                    break;
                }
                current = self.peek().unwrap();
            }
            if current_string.contains(".") {
                return Token::Float(current_string.parse().unwrap());
            }
            return Token::Integer(current_string.parse().unwrap());
        }

        if current.is_alphabetic() {
            while current.is_alphanumeric() && !self.empty() {
                current_string.push(self.pop());
                if self.peek().is_none() {
                    break;
                }
                current = self.peek().unwrap();
            }
            return match current_string.as_str() {
                "def" => Token::Def,
                "if" => Token::If,
                "as" => Token::As,
                "else" => Token::Else,
                "return" => Token::Return,
                _ => Token::Identifier(current_string),
            };
        }
        return Token::EOL;
    }

    fn pop(&mut self) -> char {
        self.raw_text.remove(0)
    }

    fn peek(&self) -> Option<char> {
        self.raw_text.chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        self.raw_text.chars().skip(1).next()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use Token::*;

    #[test]
    fn test_parser() {
        let raw = "def hello() {\n2 + 3\n}".to_string();

        let mut lexer = Lexer::new(raw);
        let expected_tokens = &[
            Token::Def,
            Token::Identifier("hello".into()),
            OpenParenth,
            CloseParenth,
            OpenCurly,
            EOL,
            Integer(2),
            Plus,
            Integer(3),
            EOL,
            ClosedCurly,
        ];

        for expected in expected_tokens {
            assert_eq!(lexer.next(), *expected);
        }
    }
}
