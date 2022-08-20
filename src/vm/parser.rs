use crate::vm::values::*;
use crate::vm::Error;

enum Token {
    Symbol(String),
    OpenParen,
    CloseParen,
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

pub fn parse(input: &str) -> Result<Option<Value>, Error> {
    let mut tokenizer = Tokenizer::new(input);
    let mut stack = Vec::new();
    stack.push(Vec::new());

    fn save_value(stack: &mut Vec<Vec<Option<Value>>>, value: Option<Value>) -> Result<(), Error> {
        match stack.last_mut() {
            Some(top) => top.push(value),
            None => return Err(Error::ParseError),
        }
        Ok(())
    }

    fn only<T>(vec: Vec<T>) -> Result<T, Error> {
        match vec.len() {
            1 => Ok(vec.into_iter().next().unwrap()),
            _ => Err(Error::ParseError),
        }
    }

    while let Some(token) = tokenizer.next() {
        match token {
            Token::OpenParen => {
                stack.push(Vec::new());
            }
            Token::CloseParen => match stack.pop() {
                Some(values) => {
                    save_value(&mut stack, Cell::from_vec(values))?;
                }
                None => return Err(Error::ParseError),
            },
            Token::Symbol(symbol) => {
                save_value(&mut stack, Some(Value::Symbol(symbol)))?;
            }
        }
    }
    return only(only(stack)?);
}
