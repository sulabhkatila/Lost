use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::types::Type;
use crate::{error::*, lexer::token::Token};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>, // Parent Environment
    values: HashMap<String, Type>,               // Current Scope
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Environment {
        Environment {
            enclosing,
            values: HashMap::<String, Type>::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Type) {
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, variable_token: &Token, value: Type) -> Result<(), Error> {
        match self.values.get(variable_token.lexeme.as_str()) {
            Some(_) => {
                self.values.insert(variable_token.lexeme.clone(), value);
                Ok(())
            }
            None => match &self.enclosing {
                Some(parent_environment) => parent_environment
                    .borrow_mut()
                    .assign(variable_token, value),
                None => Err(Error::interpreter(
                    format!("Undefined Variable {}", variable_token.lexeme),
                    variable_token.line,
                )),
            },
        }
    }

    pub fn get(&self, variable_token: &Token) -> Result<Type, Error> {
        match self.values.get(variable_token.lexeme.as_str()) {
            Some(value) => Ok((*value).clone()),
            None => match &self.enclosing {
                Some(parent_environment) => parent_environment.borrow().get(variable_token),
                None => Err(Error::interpreter(
                    format!("Undefined Variable {}", variable_token.lexeme.as_str()),
                    variable_token.line,
                )),
            },
        }
    }
}
