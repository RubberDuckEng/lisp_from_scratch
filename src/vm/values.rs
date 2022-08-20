#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Cell(Box<Cell>),
    Symbol(String),
}

impl Value {
    #[cfg(test)]
    pub fn from_str(value: &str) -> Option<Value> {
        Some(Value::Symbol(value.to_string()))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cell {
    pub left: Option<Value>,  // The value associated with this cell.
    pub right: Option<Value>, // The pointer to the next cell.
}

impl Cell {
    pub fn new(left: Option<Value>, right: Option<Value>) -> Value {
        Value::Cell(Box::new(Cell { left, right }))
    }

    // '(), the empty list, is the same as nil, which is the same as None.
    pub fn empty_list() -> Option<Value> {
        None
    }

    pub fn from_vec(values: Vec<Option<Value>>) -> Option<Value> {
        let mut cell: Option<Value> = Self::empty_list();
        for value in values.into_iter().rev() {
            cell = Some(Cell::new(value, cell));
        }
        cell
    }
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

pub fn to_string(value: &Option<Value>) -> String {
    let mut buffer = String::new();
    print_value(&mut buffer, value);
    buffer
}
