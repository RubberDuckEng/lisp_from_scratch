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

fn quote(_scope: &Arc<Scope>, args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    Ok(args[0].clone())
}

fn lambda(scope: &Arc<Scope>, args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
    let formals: Vec<String> = args[0]
        .to_args()?
        .iter()
        .map(|value| -> Result<String, Error> {
            match value.as_ref() {
                Value::Symbol(symbol) => Ok(symbol.clone()),
                _ => Err(Error::TypeError),
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let body = args[1].clone();
    Ok(Lambda::new(scope.clone(), formals, body))
}

#[derive(Debug, PartialEq, Eq)]
pub struct Scope {
    bindings: HashMap<String, Arc<Value>>,
    parent: Option<Arc<Scope>>,
}

impl Scope {
    pub fn builtin() -> Arc<Scope> {
        let mut scope = Scope {
            bindings: HashMap::new(),
            parent: None,
        };
        scope.bind_native_function("cons", 2, cons);
        scope.bind_native_function("car", 1, car);
        scope.bind_native_function("cdr", 1, cdr);
        scope.bind_special_form("quote", 1, quote);
        scope.bind_special_form("lambda", 2, lambda);
        Arc::new(scope)
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
        native: NativeSpecialForm,
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
            if let Some(parent) = self.parent.as_ref() {
                parent.lookup(name)
            } else {
                Err(Error::NotFoundError(name.to_string()))
            }
        }
    }

    pub fn new_child(self: &Arc<Self>, bindings: HashMap<String, Arc<Value>>) -> Arc<Scope> {
        Arc::new(Scope {
            bindings,
            parent: Some(self.clone()),
        })
    }
}

pub fn eval(scope: &Arc<Scope>, value: &Arc<Value>) -> Result<Arc<Value>, Error> {
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
                Value::SpecialForm(special_form) => special_form.call(scope, &args[1..]),
                _ => Err(Error::EvalError(format!(
                    "Not a function: {}",
                    to_string(&op)
                ))),
            }
        }
    }
}
