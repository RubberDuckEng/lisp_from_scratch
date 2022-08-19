use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

#[derive(Debug, PartialEq, Eq)]
enum Value {
    Cell(Box<Cell>),
    Symbol(String),
}

#[derive(Debug, PartialEq, Eq)]
struct Cell {
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
        for value in values {
            cell = Some(Cell::new(value, cell));
        }
        cell
    }
}

fn parse(input: &str) -> Result<Option<Value>> {
    let mut tokens = input.split_whitespace();
    let mut stack = Vec::new();
    while let Some(token) = tokens.next() {
        match token {
            "(" => {
                stack.push(Vec::new());
                break;
            }
            ")" => match stack.pop() {
                Some(values) => {
                    let value = Cell::from_vec(values);
                    match stack.last_mut() {
                        Some(top) => top.push(value),
                        None => return Ok(value),
                    }
                    break;
                }
                None => return Err(ReadlineError::Eof),
            },
            _ => {
                let value = Some(Value::Symbol(token.to_string()));
                match stack.last_mut() {
                    Some(top) => top.push(value),
                    None => return Ok(value),
                }
            }
        }
    }
    return Ok(None);
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

fn print_value(buffer: &mut String, value: &Option<Value>) {
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

fn main() -> Result<()> {
    // MVP: parse our prefix calculator
    // (+ 1 (+ 2 3))

    let mut rl = Editor::<()>::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parse(&line) {
                    Ok(value) => {
                        let mut buffer = String::new();
                        print_value(&mut buffer, &value);
                        println!("{}", buffer);
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_test() {
        let value = parse("a").unwrap();
        let a_symbol = Some(Value::Symbol("a".to_string()));
        assert_eq!(&value, &a_symbol);

        let value = parse("( a )").unwrap();
        assert_eq!(&value, &Some(Cell::new(a_symbol, None)));
    }

    // TODO: Test '(a)' or '()' without spaces.

    #[test]
    fn parse_and_print_test() {
        let value = parse("( a b c )").unwrap();
        // print!(value.left.unwrap());
        // print!(value.right.unwrap());
        let mut string = String::new();
        print_value(&mut string, &value);
        assert_eq!(string, "( a b c )");
    }
}
