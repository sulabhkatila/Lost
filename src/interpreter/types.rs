/*

Lox Type    Representation

nil         None
Boolean     bool
number      f64
string      String

*/

use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{error::Error, lexer::token::Token, parser::stmt::Stmt};

use super::{
    environment::{self, Environment},
    interpreter::Interpreter,
};

pub trait Callable {
    fn arity(&self) -> usize;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Option<Vec<Type>>,
    ) -> Result<Type, Error>;
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub arity: usize,
    pub declaration: Rc<RefCell<Stmt>>, // Function statement
    pub closure: Rc<RefCell<Environment>>,
}

impl Function {
    pub fn new(name: Token, arity: usize, declaration: Rc<RefCell<Stmt>>, closure: Rc<RefCell<Environment>>) -> Function {
        let declaration_borrowed = declaration.borrow();
        Function {
            name,
            arity,
            declaration: match &*declaration_borrowed {
                Stmt::Function(_, _, _) => Rc::clone(&declaration),
                _ => panic!("Tried to create a funciton with non funciton body"),
            },
            closure,
        }
    }
}

impl Callable for Function {
    fn arity(&self) -> usize {
        self.arity
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Option<Vec<Type>>,
    ) -> Result<Type, Error> {
        // let mut environment = Environment::new(Some(Rc::clone(&interpreter.globals)));
        let mut environment = Environment::new(Some(Rc::clone(&self.closure)));
        let arguments = arguments.unwrap_or_else(|| Vec::<Type>::new());

        let (name, parameters, body) = match &mut *self.declaration.borrow_mut() {
            Stmt::Function(name, parameters, body) => {
                (name.clone(), parameters.clone(), body.clone())
            }
            _ => {
                return Err(Error::interpreter(
                    "Calling a non-callable".to_string(),
                    self.name.line,
                ))
            }
        };

        for i in 0..parameters.len() {
            environment.define(parameters[i].lexeme.clone(), arguments[i].clone());
        }

        match interpreter.execute_block(&mut body.clone(), Rc::new(RefCell::new(environment)))? {
            Some(return_value) => Ok(return_value),
            None => Ok(Type::Nil),
        }
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        self.name.to_string()
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

    fn call(
        &self,
        interpreter: &mut Interpreter,
        arguments: Option<Vec<Type>>,
    ) -> Result<Type, Error> {
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
            Type::String(val) => write!(f, "{}", val),
            Type::Number(val) => write!(f, "{}", val),
            Type::Boolean(val) => write!(f, "{}", val),
            Type::Function(fun) => write!(f, "Function <{}>", fun.to_string()),
            Type::NativeFunction(fun) => write!(f, "Native Function <{}>", fun.to_string()),
            Type::Nil => write!(f, "nil"),
        }
    }
}
