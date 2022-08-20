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
