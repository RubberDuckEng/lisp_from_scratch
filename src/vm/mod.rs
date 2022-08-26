mod eval;
mod parser;
mod values;

pub use eval::*;
pub use parser::*;
pub use values::*;

#[derive(Debug)]
pub enum Error {
    ParseError,
    EvalError(String),
    TypeError,
    ArityError,
    NotFoundError(String),
}

#[cfg(test)]
mod test {
    use super::*;

    // FIXME: Test which are valid symbol characters.

    #[test]
    fn parse_test() {
        let value = parse("a").unwrap();
        let a_symbol = Value::from_name("a");
        assert_eq!(&value, &a_symbol);

        let value = parse("( a )").unwrap();
        assert_eq!(&value, &Cell::new(a_symbol, Value::nil()));

        let a_symbol = Value::from_name("a");
        let b_symbol = Value::from_name("b");
        let value = parse("( a b )").unwrap();
        assert_eq!(
            &value,
            &Cell::new(a_symbol, Cell::new(b_symbol, Value::nil()))
        );
    }

    #[test]
    fn parse_empty_test() {
        let value = parse("()").unwrap();
        assert_eq!(&value, &Value::nil());
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
