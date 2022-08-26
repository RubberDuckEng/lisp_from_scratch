use crate::vm::values::*;
use crate::vm::Error;

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Symbol(String),
    OpenParen,
    CloseParen,
    QuoteMark,
}

struct Tokenizer<'a> {
    input: &'a str,
    position: usize,
    buffer: String,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer {
            input,
            position: 0,
            buffer: String::new(),
        }
    }

    fn take_buffer(&mut self) -> String {
        let mut string = String::new();
        std::mem::swap(&mut string, &mut self.buffer);
        return string;
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len()
            && self
                .input
                .chars()
                .nth(self.position)
                .unwrap()
                .is_whitespace()
        {
            self.position += 1;
        }
    }

    // TODO: It's gross that this modifies the parser state.
    // We need to think more about how to terminate the list parsing.
    fn next_is_close_paren(&mut self) -> bool {
        self.skip_whitespace();
        if self.position >= self.input.len() {
            return false;
        }
        return self.input.chars().nth(self.position) == Some(')');
    }

    fn next(&mut self) -> Option<Token> {
        while let Some(ch) = self.input.chars().nth(self.position) {
            self.position += 1;
            match ch {
                '(' => {
                    if self.buffer.is_empty() {
                        return Some(Token::OpenParen);
                    } else {
                        self.position -= 1;
                        return Some(Token::Symbol(self.take_buffer()));
                    }
                }
                ')' => {
                    if self.buffer.is_empty() {
                        return Some(Token::CloseParen);
                    } else {
                        self.position -= 1;
                        return Some(Token::Symbol(self.take_buffer()));
                    }
                }
                '\'' => {
                    if self.buffer.is_empty() {
                        return Some(Token::QuoteMark);
                    } else {
                        self.position -= 1;
                        return Some(Token::Symbol(self.take_buffer()));
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    if !self.buffer.is_empty() {
                        return Some(Token::Symbol(self.take_buffer()));
                    }
                }
                _ => self.buffer.push(ch),
            };
        }
        if !self.buffer.is_empty() {
            return Some(Token::Symbol(self.take_buffer()));
        }
        return None;
    }
}

fn parse_value(tokenizer: &mut Tokenizer) -> Result<Option<Value>, Error> {
    if let Some(token) = tokenizer.next() {
        // Option<Vec<Value>> maybe_values;
        match token {
            Token::OpenParen => {
                let mut values = Vec::new();
                while !tokenizer.next_is_close_paren() {
                    let value = parse_value(tokenizer)?;
                    values.push(value);
                }
                let close_paren = tokenizer.next().unwrap();
                assert_eq!(close_paren, Token::CloseParen);
                return Ok(Cell::from_vec(values));
            }
            Token::CloseParen => Err(Error::ParseError),
            Token::QuoteMark => {
                let value = parse_value(tokenizer)?;
                Ok(Some(Value::Quoted(Box::new(value))))
            }
            Token::Symbol(symbol) => Ok(Some(Value::Symbol(symbol))),
        }
    } else {
        Err(Error::ParseError)
    }
}

pub fn parse(input: &str) -> Result<Option<Value>, Error> {
    let mut tokenizer = Tokenizer::new(input);
    let value = parse_value(&mut tokenizer)?;
    if tokenizer.next().is_some() {
        return Err(Error::ParseError);
    }
    return Ok(value);
}
