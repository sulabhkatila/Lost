/*

Lox Type    Representation

nil         None
Boolean     bool
number      f64
string      String

*/

use std::fmt;

use crate::error::Error;

pub struct NativeFunction {}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
}

impl Function {
    pub fn call(&self) -> Result<Type, Error> {
        todo!()
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    String(String),
    Number(f32),
    Boolean(bool),
    Function(Box<Function>),
    Nil,
}

impl Type {
    pub fn value(&self) -> String {
        match self {
            Type::String(val) => val.to_string(),
            Type::Number(val) => val.to_string(),
            Type::Boolean(val) => val.to_string(),
            Type::Function(fun) => fun.to_string(),
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
            Type::Function(fun) => write!(f, "Function {}", fun.to_string()),
            Type::Nil => write!(f, "nil"),
        }
    }
}
