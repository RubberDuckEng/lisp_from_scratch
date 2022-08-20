mod parser;
mod values;

pub use parser::*;
pub use values::*;

#[derive(Debug)]
pub enum Error {
    ParseError,
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
