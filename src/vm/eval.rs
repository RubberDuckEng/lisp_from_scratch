pub use crate::vm::values::*;
pub use crate::vm::*;

fn car(value: Option<Value>) -> Result<Option<Value>, Error> {
    match value {
        Some(Value::Cell(cell)) => Ok(cell.left),
        _ => Err(Error::TypeError),
    }
}

fn cdr(value: Option<Value>) -> Result<Option<Value>, Error> {
    match value {
        Some(Value::Cell(cell)) => Ok(cell.right),
        _ => Err(Error::TypeError),
    }
}

pub fn eval(value: Option<Value>) -> Result<Option<Value>, Error> {
    if let Some(value) = value {
        match value {
            Value::Cell(cell) => {
                if let Some(Value::Symbol(function_name)) = cell.left {
                    if function_name == "car" {
                        return car(cell.right);
                    } else if function_name == "cdr" {
                        return cdr(cell.right);
                    }
                    return Err(Error::EvalError(format!(
                        "Unknown function: {}",
                        function_name
                    )));
                }
                Err(Error::EvalError("Unimplemented".to_string()))
            }
            Value::Symbol(string) => Err(Error::EvalError(format!("Unbound symbol: {}", string))),
            Value::Quoted(value) => Ok(*value),
        }
    } else {
        Ok(None)
    }
}
