use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};

enum Value {
    Cell(Box<Cell>),
    Symbol(String),
}

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
    let mut values = Vec::new();
    while let Some(token) = tokens.next() {
        match token {
            "(" => {
                let mut sub_values = Vec::new();
                while let Some(token) = tokens.next() {
                    if token == ")" {
                        break;
                    } else {
                        // TODO: Convert to use a stack rather than using the
                        // runtime stack.
                        sub_values.push(parse(token)?);
                    }
                }
                values.push(Cell::from_vec(sub_values));
            }
            ")" => {
                return Err(ReadlineError::Eof);
            }
            _ => {
                values.push(Some(Value::Symbol(token.to_string())));
            }
        }
    }
    Ok(Cell::from_vec(values))
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
    fn base_test() {
        let value = parse("(a b c)").unwrap();
        let mut string = String::new();
        print_value(&mut string, &value);
        assert_eq!(string, "(a b c)");
    }
}
