mod parser;
mod values;

pub use parser::*;
pub use values::*;

#[derive(Debug)]
pub enum Error {
    ParseError,
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

    #[test]
    fn parse_empty_test() {
        let value = parse("()").unwrap();
        assert_eq!(&value, &None);
    }

    #[test]
    fn parse_error_test() {
        let result = parse("a b");
        assert!(result.is_err());
    }

    #[test]
    fn parse_and_print_test() {
        let value = parse("(a)").unwrap();
        assert_eq!(to_string(&value), "(a)");

        let value = parse("( a b c )").unwrap();
        assert_eq!(to_string(&value), "(a b c)");

        let value = parse("(a b c)").unwrap();
        assert_eq!(to_string(&value), "(a b c)");

        let value = parse("(a(b c))").unwrap();
        assert_eq!(to_string(&value), "(a (b c))");
    }
}
