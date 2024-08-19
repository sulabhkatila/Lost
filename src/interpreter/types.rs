/*

Lox Type    Representation

nil         None
Boolean     bool
number      f64
string      String

*/

use std::{collections::HashMap, fmt};

use crate::{error::Error, lexer::token::Token};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(&self, arguments: Option<Vec<Type>>) -> Result<Type, Error>;
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,
}

impl Function {
    pub fn new(name: String, parameters: Option<Vec<Token>>) -> Function {
        Function {
            name,
            arity: match parameters {
                Some(parameters) => parameters.len(),
                None => 0,
            },
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, arguments: Option<Vec<Type>>) -> Result<Type, Error> {
        todo!()
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Clone)]
pub struct NativeFunction {
    pub name: String,
    pub arity: usize,
    to_call: fn(), // Currently only no parameters
                   // and no return value native functions
}

impl NativeFunction {
    pub fn new(name: String, to_call: fn()) -> NativeFunction {
        NativeFunction {
            name,
            arity: 0,
            to_call,
        }
    }
}

impl Callable for NativeFunction {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(&self, arguments: Option<Vec<Type>>) -> Result<Type, Error> {
        let res = (self.to_call)();

        Ok(Type::Nil) // Native Functions will reutrn nothing for now
    }
}

impl ToString for NativeFunction {
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
    NativeFunction(Box<NativeFunction>),
    Nil,
}

impl Type {
    pub fn value(&self) -> String {
        match self {
            Type::String(val) => val.to_string(),
            Type::Number(val) => val.to_string(),
            Type::Boolean(val) => val.to_string(),
            Type::Function(fun) => fun.to_string(),
            Type::NativeFunction(fun) => fun.to_string(),
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
            Type::NativeFunction(fun) => write!(f, "Native Function {}", fun.to_string()),
            Type::Nil => write!(f, "nil"),
        }
    }
}
