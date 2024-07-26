use super::types::*;
use crate::error::*;
use crate::lexer::token::*;
use std::collections::HashMap;

pub struct Environment {
    enclosing: Box<Option<Environment>>, // rc pointer
    values: HashMap<String, Type>,
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Environment {
        Environment {
            enclosing: Box::new(enclosing),
            values: HashMap::<String, Type>::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Type) {
        // var a = "before"
        // var a = "after"
        self.values.insert(name, value);
    }

    pub fn assign(&mut self, token: Token, value: Type) -> Result<(), Error> {
        match self.values.get(&token.lexeme) {
            Some(_) => {
                self.values.insert(token.lexeme, value);
                Ok(())
            }
            _ => {
                match &mut (*self.enclosing) {
                    Some(parent_environment) => parent_environment.assign(token, value),
                    _ => {
                        // the key does not exist in any environment
                        // variable was never declared
                        Err(Error::interpreter(
                            format!("Undefined variable {}", token.lexeme),
                            token.line,
                        ))
                    }
                }
            }
        }
    }

    pub fn get(&self, name: &Token) -> Result<Type, Error> {
        let line = name.line;
        match name.token_type {
            TokenType::Identifier => {
                if let Some(value) = self.values.get(&name.lexeme) {
                    Ok(value.clone())
                } else {
                    // Making it a runtime error
                    // Allowing to refer variables before decaluted as long as
                    // reference is not evaluated

                    match &(*self.enclosing) {
                        Some(parent_environment) => parent_environment.get(&name),
                        _ => Err(Error::interpreter(
                            format!("Undefined variable '{}'.", name.lexeme),
                            line,
                        )),
                    }
                }
            }
            _ => Err(Error::interpreter(
                "Attempt to get value of a non-identifier token.".to_string(),
                line,
            )),
        }
    }
}
