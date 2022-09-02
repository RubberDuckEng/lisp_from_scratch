use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::vm::values::*;
use crate::vm::*;

fn cons(args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    Ok(Cell::new(args[0].clone(), args[1].clone()))
}

fn car(args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    match args[0].deref() {
        Value::Cell(cell) => Ok(cell.left.clone()),
        _ => Err(Error::TypeError),
    }
}

fn cdr(args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    match args[0].deref() {
        Value::Cell(cell) => Ok(cell.right.clone()),
        _ => Err(Error::TypeError),
    }
}

fn quote(args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    Ok(args[0].clone())
}

pub struct Scope {
    bindings: HashMap<String, Arc<Value>>,
}

impl Scope {
    pub fn builtin() -> Scope {
        let mut scope = Scope {
            bindings: HashMap::new(),
        };
        scope.bind_native_function("cons", 2, cons);
        scope.bind_native_function("car", 1, car);
        scope.bind_native_function("cdr", 1, cdr);
        scope.bind_special_form("quote", 1, quote);
        scope
    }

    pub fn bind_native_function(
        &mut self,
        name: &'static str,
        arity: usize,
        native: fn(&[Arc<Value>]) -> Result<Arc<Value>, Error>,
    ) {
        self.bind(name, Func::from_native(name, arity, native));
    }

    pub fn bind_special_form(
        &mut self,
        name: &'static str,
        arity: usize,
        native: fn(&[Arc<Value>]) -> Result<Arc<Value>, Error>,
    ) {
        self.bind(name, SpecialForm::from_native(name, arity, native));
    }

    pub fn bind(&mut self, name: &str, value: Arc<Value>) {
        self.bindings.insert(name.to_string(), value);
    }

    pub fn lookup(&self, name: &str) -> Result<Arc<Value>, Error> {
        if let Some(value) = self.bindings.get(name) {
            Ok(value.clone())
        } else {
            Err(Error::NotFoundError(name.to_string()))
        }
    }
}

pub fn eval(scope: &Scope, value: &Arc<Value>) -> Result<Arc<Value>, Error> {
    match value.deref() {
        Value::Nil | Value::Function(_) | Value::SpecialForm(_) => Ok(value.clone()),
        Value::Symbol(name) => scope.lookup(name),
        Value::Quoted(value) => Ok(value.clone()),
        Value::Cell(_) => {
            let args = value.to_args()?;
            let op = eval(scope, &args[0])?;
            match op.deref() {
                Value::Function(function) => {
                    let evaluated: Vec<Arc<Value>> = args
                        .iter()
                        .skip(1)
                        .map(|value| eval(scope, value))
                        .collect::<Result<Vec<Arc<Value>>, Error>>()?;
                    function.call(&evaluated)
                }
                Value::SpecialForm(special_form) => special_form.call(&args[1..]),
                _ => Err(Error::EvalError(format!(
                    "Not a function: {}",
                    to_string(&op)
                ))),
            }
        }
    }
}
