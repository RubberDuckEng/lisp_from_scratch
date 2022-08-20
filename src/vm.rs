#[derive(Debug)]
pub enum Error {
    ParseError,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Cell(Box<Cell>),
    Symbol(String),
}

impl Value {
    #[cfg(test)]
    fn from_str(value: &str) -> Option<Value> {
        Some(Value::Symbol(value.to_string()))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cell {
    left: Option<Value>,  // The value associated with this cell.
    right: Option<Value>, // The pointer to the next cell.
}

impl Cell {
    fn new(left: Option<Value>, right: Option<Value>) -> Value {
        Value::Cell(Box::new(Cell { left, right }))
    }

    // '(), the empty list, is the same as nil, which is the same as None.
    fn empty_list() -> Option<Value> {
        None
    }

    fn from_vec(values: Vec<Option<Value>>) -> Option<Value> {
        let mut cell: Option<Value> = Self::empty_list();
        for value in values.into_iter().rev() {
            cell = Some(Cell::new(value, cell));
        }
        cell
    }
}

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

fn print_list(buffer: &mut String, cell: &Option<Value>) {
    buffer.push_str("(");
    let mut first = true;
    let mut maybe_current = cell;
    while let Some(Value::Cell(cell)) = maybe_current {
        if first {
            first = false;
        } else {
            buffer.push_str(" ");
        }
        print_value(buffer, &cell.left);
        maybe_current = &cell.right;
    }
    buffer.push_str(")");
}

pub fn print_value(buffer: &mut String, value: &Option<Value>) {
    match value {
        Some(Value::Cell(_)) => {
            print_list(buffer, value);
        }
        Some(Value::Symbol(symbol)) => {
            buffer.push_str(symbol);
        }
        None => {
            buffer.push_str("nil");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test() {
        let value = parse("a").unwrap();
        let a_symbol = Value::from_str("a");
        assert_eq!(&value, &a_symbol);

        let value = parse("( a )").unwrap();
        assert_eq!(&value, &Some(Cell::new(a_symbol, None)));

        let a_symbol = Value::from_str("a");
        let b_symbol = Value::from_str("b");
        let value = parse("( a b )").unwrap();
        assert_eq!(
            &value,
            &Some(Cell::new(a_symbol, Some(Cell::new(b_symbol, None))))
        );
    }

    // TODO: Test '(a)' or '()' without spaces.

    // Test 'a b' should be an error.

    #[test]
    fn parse_and_print_test() {
        let value = parse("( a b c )").unwrap();
        let mut string = String::new();
        print_value(&mut string, &value);
        assert_eq!(string, "(a b c)");
    }
}
