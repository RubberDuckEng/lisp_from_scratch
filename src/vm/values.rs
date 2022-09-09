use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use crate::vm::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Nil,
    Cell(Cell),
    Symbol(String),
    Quoted(Arc<Value>),
    Function(Func),
    SpecialForm(SpecialForm),
}

impl Value {
    #[cfg(test)]
    pub fn from_name(name: &str) -> Arc<Value> {
        Arc::new(Value::Symbol(name.to_string()))
    }

    pub fn nil() -> Arc<Value> {
        Arc::new(Value::Nil)
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            _ => true,
        }
    }

    pub fn to_args(self: &Arc<Self>) -> Result<Vec<Arc<Value>>, Error> {
        let mut args = Vec::new();
        let mut current = self.clone();
        loop {
            match current.deref() {
                Value::Nil => break,
                Value::Cell(cell) => {
                    args.push(cell.left.clone());
                    current = cell.right.clone();
                }
                _ => return Err(Error::TypeError),
            }
        }
        return Ok(args);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cell {
    pub left: Arc<Value>,
    pub right: Arc<Value>,
}

impl Cell {
    pub fn new(left: Arc<Value>, right: Arc<Value>) -> Arc<Value> {
        Arc::new(Value::Cell(Cell { left, right }))
    }

    // '(), the empty list, is the same as nil, which is the same as None.
    pub fn empty_list() -> Arc<Value> {
        Value::nil()
    }

    pub fn from_vec(values: Vec<Arc<Value>>) -> Arc<Value> {
        let mut cell = Self::empty_list();
        for value in values.into_iter().rev() {
            cell = Cell::new(value, cell);
        }
        cell
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Macro {
    scope: Arc<Scope>,
    formals: Vec<String>,
    body: Arc<Value>,
}

impl Macro {
    pub fn new(scope: Arc<Scope>, formals: Vec<String>, body: Arc<Value>) -> Arc<Value> {
        Arc::new(Value::SpecialForm(SpecialForm {
            name: "#macro".to_string(),
            arity: formals.len(),
            body: SpecialFormBody::Macro(Macro {
                scope,
                formals,
                body,
            }),
        }))
    }

    pub fn call(
        &self,
        dynamic_scope: &Arc<Scope>,
        args: &[Arc<Value>],
    ) -> Result<Arc<Value>, Error> {
        let bindings = HashMap::from_iter(
            self.formals
                .iter()
                .zip(args)
                .map(|(name, value)| (name.clone(), value.clone())),
        );
        let lexical_scope = self.scope.new_child(bindings);
        // We need to evaluate the body in the context of the macro's scope.
        let code = eval(&lexical_scope, &self.body)?;
        // We need to evaluate the code produced the macro in the context of where the macro is expanded.
        eval(dynamic_scope, &code)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Lambda {
    scope: Arc<Scope>,
    formals: Vec<String>,
    body: Arc<Value>,
}

impl Lambda {
    pub fn new(scope: Arc<Scope>, formals: Vec<String>, body: Arc<Value>) -> Arc<Value> {
        Arc::new(Value::Function(Func {
            name: "#lambda".to_string(),
            arity: None, // Lambda does arity checking separately.
            body: FuncBody::Lambda(Lambda {
                scope,
                formals,
                body,
            }),
        }))
    }

    pub fn call(&self, args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
        const SPLAT_MARKER: &str = "...";
        let mut args_iter = args.iter();
        let pairs = self
            .formals
            .iter()
            .map(|name| {
                if name.starts_with(SPLAT_MARKER) {
                    let name = name[SPLAT_MARKER.len()..].to_string();
                    let mut values = Vec::new();
                    while let Some(arg) = args_iter.next() {
                        values.push(arg.clone());
                    }
                    Ok((name, Cell::from_vec(values)))
                } else {
                    let arg = args_iter.next().ok_or(Error::ArityError)?;
                    Ok((name.clone(), arg.clone()))
                }
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let bindings = HashMap::from_iter(pairs.into_iter());
        let scope = self.scope.new_child(bindings);
        eval(&scope, &self.body)
    }
}

pub enum FuncBody {
    Native(fn(&[Arc<Value>]) -> Result<Arc<Value>, Error>),
    Lambda(Lambda),
}

impl std::fmt::Debug for FuncBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("FuncBody").field("fn", &"#code").finish()
    }
}

impl std::cmp::PartialEq for FuncBody {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl std::cmp::Eq for FuncBody {}

#[derive(Debug, PartialEq, Eq)]
pub struct Func {
    pub name: String,
    pub arity: Option<usize>,
    pub body: FuncBody,
}

impl Func {
    pub fn new(name: String, body: FuncBody) -> Arc<Value> {
        Arc::new(Value::Function(Self {
            name,
            arity: None,
            body,
        }))
    }

    pub fn new_with_arity(name: String, body: FuncBody, arity: usize) -> Arc<Value> {
        Arc::new(Value::Function(Self {
            name,
            arity: Some(arity),
            body,
        }))
    }

    pub fn from_native(
        name: &'static str,
        native: fn(&[Arc<Value>]) -> Result<Arc<Value>, Error>,
    ) -> Arc<Value> {
        Self::new(name.to_string(), FuncBody::Native(native))
    }

    pub fn from_native_with_arity(
        name: &'static str,
        native: fn(&[Arc<Value>]) -> Result<Arc<Value>, Error>,
        arity: usize,
    ) -> Arc<Value> {
        Self::new_with_arity(name.to_string(), FuncBody::Native(native), arity)
    }

    pub fn call(&self, args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
        match &self.body {
            FuncBody::Native(function) => {
                if let Some(arity) = self.arity {
                    if args.len() != arity {
                        return Err(Error::ArityError);
                    }
                }
                function(args)
            }
            FuncBody::Lambda(lambda) => lambda.call(args),
        }
    }
}

pub type NativeSpecialForm = fn(&Arc<Scope>, &[Arc<Value>]) -> Result<Arc<Value>, Error>;

pub enum SpecialFormBody {
    Native(NativeSpecialForm),
    Macro(Macro),
}

pub struct SpecialForm {
    pub name: String,
    pub arity: usize,
    pub body: SpecialFormBody,
}

impl std::fmt::Debug for SpecialForm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("SpecialForm").field("fn", &"#code").finish()
    }
}

impl std::cmp::PartialEq for SpecialForm {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl std::cmp::Eq for SpecialForm {}

impl SpecialForm {
    pub fn new(name: String, arity: usize, body: NativeSpecialForm) -> Arc<Value> {
        Arc::new(Value::SpecialForm(SpecialForm {
            name,
            arity,
            body: SpecialFormBody::Native(body),
        }))
    }

    pub fn from_native(name: &'static str, arity: usize, native: NativeSpecialForm) -> Arc<Value> {
        Self::new(name.to_string(), arity, native)
    }

    pub fn call(&self, scope: &Arc<Scope>, args: &[Arc<Value>]) -> Result<Arc<Value>, Error> {
        // TODO: Handle arity like for lambda.
        if args.len() != self.arity {
            return Err(Error::ArityError);
        }
        match &self.body {
            SpecialFormBody::Native(function) => function(scope, args),
            SpecialFormBody::Macro(macro_) => macro_.call(scope, args),
        }
    }
}

fn print_list(buffer: &mut String, cell: &Arc<Value>) {
    buffer.push_str("(");
    let mut first = true;
    let mut maybe_current = cell;
    while let Value::Cell(cell) = maybe_current.deref() {
        if first {
            first = false;
        } else {
            buffer.push_str(" ");
        }
        print_value(buffer, &cell.left);
        maybe_current = &cell.right;
    }
    match maybe_current.deref() {
        Value::Nil => {}
        _ => {
            buffer.push_str(" . ");
            print_value(buffer, maybe_current);
        }
    }
    buffer.push_str(")");
}

fn print_value(buffer: &mut String, value: &Arc<Value>) {
    match value.deref() {
        Value::Cell(_) => {
            print_list(buffer, value);
        }
        Value::Symbol(name) => {
            buffer.push_str(name);
        }
        Value::Quoted(value) => {
            buffer.push('\'');
            print_value(buffer, value);
        }
        Value::Function(_) => {
            buffer.push_str("#func");
        }
        Value::SpecialForm(_) => {
            buffer.push_str("#special_form");
        }
        Value::Nil => {
            buffer.push_str("nil");
        }
    }
}

pub fn to_string(value: &Arc<Value>) -> String {
    let mut buffer = String::new();
    print_value(&mut buffer, value);
    buffer
}
