/*

Lox Type    Representation

nil         None
Boolean     bool
number      f64
string      String

*/

use std::fmt;

#[derive(Clone)]
pub enum Type {
    String(String),
    Number(f32),
    Boolean(bool),
    Nil,
}

impl Type {
    pub fn value(&self) -> String {
        match self {
            Type::String(val) => val.to_string(),
            Type::Number(val) => val.to_string(),
            Type::Boolean(val) => val.to_string(),
            Type::Nil => "nil".to_string(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::String(val) => write!(f, "String {}", val),
            Type::Number(val) => write!(f, "Number {}", val),
            Type::Boolean(val) => write!(f, "Boolean {}", val),
            Type::Nil => write!(f, "nil"),
        }
    }
}
