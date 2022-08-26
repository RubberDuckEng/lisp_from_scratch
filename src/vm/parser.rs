use std::sync::Arc;

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
    peeked: Option<Token>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &str) -> Tokenizer {
        Tokenizer {
            input,
            position: 0,
            buffer: String::new(),
            peeked: None,
        }
    }

    fn take_buffer(&mut self) -> String {
        let mut string = String::new();
        std::mem::swap(&mut string, &mut self.buffer);
        return string;
    }

    fn peek(&mut self) -> &Option<Token> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }
        return &self.peeked;
    }

    fn next(&mut self) -> Option<Token> {
        if let Some(token) = self.peeked.take() {
            return Some(token);
        }
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

fn parse_value(tokenizer: &mut Tokenizer) -> Result<Arc<Value>, Error> {
    if let Some(token) = tokenizer.next() {
        match token {
            Token::OpenParen => {
                let mut values = Vec::new();
                while *tokenizer.peek() != Some(Token::CloseParen) {
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
                Ok(Arc::new(Value::Quoted(value)))
            }
            Token::Symbol(name) => Ok(Arc::new(Value::Symbol(name))),
        }
    } else {
        Err(Error::ParseError)
    }
}

pub fn parse(input: &str) -> Result<Arc<Value>, Error> {
    let mut tokenizer = Tokenizer::new(input);
    let value = parse_value(&mut tokenizer)?;
    if tokenizer.next().is_some() {
        return Err(Error::ParseError);
    }
    return Ok(value);
}
