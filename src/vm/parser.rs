use crate::vm::values::*;
use crate::vm::Error;

pub fn parse(input: &str) -> Result<Option<Value>, Error> {
    let mut tokens = input.split_whitespace();
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

    while let Some(token) = tokens.next() {
        match token {
            "(" => {
                stack.push(Vec::new());
            }
            ")" => match stack.pop() {
                Some(values) => {
                    save_value(&mut stack, Cell::from_vec(values))?;
                }
                None => return Err(Error::ParseError),
            },
            _ => {
                save_value(&mut stack, Some(Value::Symbol(token.to_string())))?;
            }
        }
    }
    return only(only(stack)?);
}
