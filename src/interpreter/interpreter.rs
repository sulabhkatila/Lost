use std::{
    borrow::Borrow,
    cell::{Ref, RefCell},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{environment::*, types::*};

use crate::{
    error::*,
    lexer::token::*,
    parser::{
        expr::{Visitor as ExpressionVisitor, *},
        stmt::{Visitable as StatementVisitable, Visitor as StatementVisitor, *},
    },
};

pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new(enclosing: Option<Environment>) -> Interpreter {
        let mut globals = Environment::new(None);

        // Native Functions
        fn clock() {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            let milli_secs = since_the_epoch.as_secs() * 1000
                + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
            println!("{}", milli_secs)
        }

        globals.define(
            "clock".to_string(),
            Type::NativeFunction(Box::new(NativeFunction::new("clock".to_string(), clock))),
        );

        Interpreter {
            globals: Rc::new(RefCell::new(globals)),
            environment: Rc::new(RefCell::new(Environment::new(match enclosing {
                Some(parent_environment) => Some(Rc::new(RefCell::new(parent_environment))),
                None => None,
            }))),
        }
    }

    pub fn interpret(&mut self, expr_vec: &mut Vec<Box<Stmt>>) -> Result<(), Error> {
        for expr in expr_vec {
            let _ = self.execute(expr)?;
        }
        Ok(())
    }

    fn execute(&mut self, stmt: &mut Stmt) -> Result<(), Error> {
        stmt.accept(self)?;
        Ok(())
    }

    fn evaluate(&mut self, expr: &Box<Expr>) -> Result<Type, Error> {
        expr.clone().accept(self)
    }

    // Returns the number value if `value` is of type `Type::Number`, otherwise returns an `Error`.
    pub fn get_number_or_return_error(&self, value: Type, line: usize) -> Result<f32, Error> {
        match value {
            Type::Number(val) => Ok(val),
            _ => Err(Error::InterpretError(
                format!("Expected Number, got {}", value),
                line,
            )),
        }
    }

    // Compares equality between two Type values.
    // Returns true if both values are of the same type and have the same value.
    // Returns false if the types are different or the values do not match.
    pub fn is_equal(&self, left_expr: Type, right_expr: Type) -> bool {
        match left_expr {
            Type::Nil => match right_expr {
                Type::Nil => true,
                _ => false,
            },
            Type::Boolean(left_val) => match right_expr {
                Type::Boolean(right_val) => left_val == right_val,
                _ => false,
            },
            Type::Number(left_val) => match right_expr {
                Type::Number(right_val) => left_val == right_val,
                _ => false,
            },
            Type::String(left_val) => match right_expr {
                Type::String(right_val) => left_val == right_val,
                _ => false,
            },
            Type::Function(fun) => todo!(),
            Type::NativeFunction(fun) => todo!(),
        }
    }

    // Determines the truthiness of a Type value.
    // Returns true for non-empty strings, non-zero numbers, and true booleans.
    // Returns false for zero numbers, false booleans, and Nil values.
    pub fn is_truthly(&self, value: &Type) -> bool {
        match value {
            Type::String(_) => true,
            Type::Number(val) => {
                if *val == 0.0 {
                    false
                } else {
                    true
                }
            }
            Type::Boolean(val) => *val,
            Type::Function(fun) => todo!(),
            Type::NativeFunction(fun) => todo!(),
            Type::Nil => false,
        }
    }

    pub fn execute_block(
        &mut self,
        statements: &mut Box<Vec<Stmt>>,
        environment: Rc<RefCell<Environment>>,
    ) -> Result<(), Error> {
        let temp = Rc::clone(&self.environment);

        self.environment = environment;

        for statement in (*statements).as_mut().iter_mut() {
            match self.execute(statement) {
                Err(error) => {
                    self.environment = temp;
                    return Err(error);
                }
                Ok(_) => {}
            };
        }

        self.environment = temp;
        Ok(())
    }
}

impl ExpressionVisitor<Result<Type, Error>> for Interpreter {
    fn visit_binary(
        &mut self,
        left_expr: &mut Box<Expr>,
        operator: &Token,
        right_expr: &mut Box<Expr>,
    ) -> Result<Type, Error> {
        let left_value = self.evaluate(left_expr)?;
        let right_value = self.evaluate(right_expr)?;

        let line = operator.line;
        match operator.token_type {
            // Arithmetic operations
            // left_number  - | / | *  right_number
            TokenType::Minus => {
                let left = self.get_number_or_return_error(left_value, line)?;
                let right = self.get_number_or_return_error(right_value, line)?;

                return Ok(Type::Number(left - right));
            }
            TokenType::Slash => {
                let right = self.get_number_or_return_error(right_value, line)?;
                if right == 0.0 {
                    return Err(Error::InterpretError("Division by Zero".to_string(), line));
                }
                Ok(Type::Number(
                    self.get_number_or_return_error(left_value, line)? / right,
                ))
            }
            TokenType::Star => Ok(Type::Number(
                self.get_number_or_return_error(left_value, line)?
                    * self.get_number_or_return_error(right_value, line)?,
            )),

            // Arithmetic operation or String concatnation
            // left_number + right_number
            // left_string + right_string
            TokenType::Plus => {
                match self.get_number_or_return_error(left_value.clone(), line) {
                    Ok(left_number) => {
                        // Left is a number, so right has to be a number for '+' to be valid
                        let right_number =
                            self.get_number_or_return_error(right_value.clone(), line)?;
                        Ok(Type::Number(left_number + right_number))
                    }
                    _ => match self.get_number_or_return_error(right_value.clone(), line) {
                        // Left is a String,
                        // so right needs to be a String
                        Ok(_) => {
                            return Err(Error::interpreter(
                                format!("Expected String, got {}", right_value),
                                line,
                            ))
                        }
                        _ => {
                            return Ok(Type::String(format!(
                                "{}{}",
                                left_value.value(),
                                right_value.value()
                            )));
                        }
                    },
                }
            }

            // Comparison operations
            // left_number  > | >= | < | <= | == | !=  right_number
            TokenType::Greater => Ok(Type::Boolean(
                self.get_number_or_return_error(left_value, line)?
                    > self.get_number_or_return_error(right_value, line)?,
            )),
            TokenType::GreaterEqual => Ok(Type::Boolean(
                self.get_number_or_return_error(left_value, line)?
                    >= self.get_number_or_return_error(right_value, line)?,
            )),
            TokenType::Less => Ok(Type::Boolean(
                self.get_number_or_return_error(left_value, line)?
                    < self.get_number_or_return_error(right_value, line)?,
            )),
            TokenType::LessEqual => Ok(Type::Boolean(
                self.get_number_or_return_error(left_value, line)?
                    <= self.get_number_or_return_error(right_value, line)?,
            )),

            // Comparing Equality
            // left_value_of_X_type  == | !=  right_value_of_X_type
            TokenType::EqualEqual => Ok(Type::Boolean(self.is_equal(left_value, right_value))),
            TokenType::BangEqual => Ok(Type::Boolean(!self.is_equal(left_value, right_value))),

            _ => {
                return Err(Error::interpreter(
                    format!("Unexpected Operator, got {}", operator),
                    line,
                ));
            }
        }
    }

    fn visit_grouping(&mut self, grouping_expr: &mut Box<Expr>) -> Result<Type, Error> {
        self.evaluate(grouping_expr)
    }

    fn visit_unary(&mut self, operator: &Token, unary_expr: &mut Box<Expr>) -> Result<Type, Error> {
        let right = self.evaluate(unary_expr)?;

        let line = operator.line;
        match operator.token_type {
            TokenType::Minus => {
                return Ok(Type::Number(match right {
                    Type::Number(val) => -val,
                    _ => {
                        return Err(Error::interpreter(
                            format!("Expected Number, got {}", right),
                            line,
                        ))
                    }
                }))
            }
            TokenType::Bang => return Ok(Type::Boolean(!self.is_truthly(&right))),
            _ => {
                return Err(Error::interpreter(
                    format!("Expected `!` or `-`, got {}", operator),
                    line,
                ))
            }
        }
    }

    fn visit_literal(&mut self, lit: &Token) -> Result<Type, Error> {
        let line = lit.line;
        match lit.token_type {
            // String and Number literals
            TokenType::String => Ok(Type::String(match lit.literal.clone() {
                Some(val) => match val {
                    LiteralType::StringType(string_val) => string_val,
                    LiteralType::NumberType(number_val) => {
                        return Err(Error::interpreter(
                            format!("Expected String, got Number: `{}`", number_val),
                            line,
                        ));
                    }
                },
                None => {
                    return Err(Error::interpreter(
                        format!("Expected String, got None"),
                        line,
                    ))
                }
            })),
            TokenType::Number => Ok(Type::Number(match lit.literal.clone() {
                Some(val) => match val {
                    LiteralType::NumberType(number_val) => number_val,
                    LiteralType::StringType(string_val) => {
                        return Err(Error::interpreter(
                            format!("Expected String, got String: `{}`", string_val),
                            line,
                        ));
                    }
                },
                None => {
                    return Err(Error::interpreter(
                        format!("Expected String, got None"),
                        line,
                    ))
                }
            })),

            // Booleans
            TokenType::True => Ok(Type::Boolean(true)),
            TokenType::False => Ok(Type::Boolean(false)),

            // Nil
            TokenType::Nil => Ok(Type::Nil),

            _ => Err(Error::interpreter(
                format!("Unexpected! unreachable code reached"),
                line,
            )),
        }
    }

    fn visit_variable(&mut self, variable: &Token) -> Result<Type, Error> {
        (*self.environment).borrow().get(variable)
    }

    fn visit_assign(&mut self, variable: &Token, expr: &mut Box<Expr>) -> Result<Type, Error> {
        let value = self.evaluate(expr)?;
        let _ = (*self.environment)
            .borrow_mut()
            .assign(variable, value.clone())?;
        Ok(value)
    }

    fn visit_logical(
        &mut self,
        left_expr: &mut Box<Expr>,
        logical_and_or: &mut Token,
        right_expr: &mut Box<Expr>,
    ) -> Result<Type, Error> {
        let left_value = self.evaluate(&left_expr)?;

        match logical_and_or.token_type {
            TokenType::Or => {
                if self.is_truthly(&left_value) {
                    return Ok(left_value);
                }
            }
            TokenType::And => {
                if !self.is_truthly(&left_value) {
                    return Ok(left_value);
                }
            }
            _ => {
                println!("{:#?}", &logical_and_or);
                unreachable!()
            }
        }

        self.evaluate(&right_expr)
    }

    fn visit_call(
        &mut self,
        callee: &mut Box<Expr>,
        closing_paren: &Token,
        arguments: &mut Box<Vec<Expr>>,
    ) -> Result<Type, Error> {
        let callee = self.evaluate(callee)?;

        let mut evaluated_arguments = Vec::new();
        for argument in &mut (**arguments) {
            evaluated_arguments.push(self.evaluate(&Box::new(argument.clone())));
        }

        match callee {
            Type::Function(to_call) => {
                if to_call.arity != evaluated_arguments.len() {
                    return Err(Error::interpreter(
                        "Number of arguments does not match number of parameters".to_string(),
                        closing_paren.line,
                    ));
                }
                to_call.call(None)
            }
            Type::NativeFunction(to_call) => {
                if to_call.arity != evaluated_arguments.len() {
                    return Err(Error::interpreter(
                        "Number of arguments does not match number of parameters".to_string(),
                        closing_paren.line,
                    ));
                }
                to_call.call(None)
            }
            _ => Err(Error::interpreter(
                "Not a function".to_string(),
                closing_paren.line,
            )),
        }
    }
}

impl StatementVisitor<Result<(), Error>> for Interpreter {
    fn visit_block(&mut self, statements: &mut Box<Vec<Stmt>>) -> Result<(), Error> {
        self.execute_block(statements, Rc::clone(&self.environment))?;
        Ok(())
    }

    fn visit_expression(&mut self, expr: &Box<Expr>) -> Result<(), Error> {
        let _ = self.evaluate(expr)?;

        Ok(())
    }

    fn visit_print(&mut self, expr: &Box<Expr>) -> Result<(), Error> {
        let value = self.evaluate(expr)?;
        println!("{}", value);

        Ok(())
    }

    fn visit_var(&mut self, token: &Token, expr: &Option<Box<Expr>>) -> Result<(), Error> {
        // token is the variable
        // expr is the value for the variable // initializer
        match expr {
            Some(val) => {
                let val = self.evaluate(val)?;
                (*self.environment)
                    .borrow_mut()
                    .define(token.lexeme.clone(), val.clone());
            }
            _ => (*self.environment)
                .borrow_mut()
                .define(token.lexeme.clone(), Type::Nil),
        }
        Ok(())
    }

    fn visit_ifelse(
        &mut self,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> Result<(), Error> {
        let condition_evaluated = self.evaluate(condition)?;
        if self.is_truthly(&condition_evaluated) {
            let mut then_branch = then_branch.clone();
            self.execute(&mut then_branch)
        } else {
            match else_branch {
                Some(else_branch) => self.execute(&mut (**else_branch).clone()),
                _ => return Ok(()),
            }
        }
    }

    fn visit_whileloop(
        &mut self,
        condition: &Box<Expr>,
        statement: &mut Box<Stmt>,
    ) -> Result<(), Error> {
        let mut evaluated_condition = self.evaluate(condition)?;

        while self.is_truthly(&evaluated_condition) {
            self.execute(&mut *statement)?;

            evaluated_condition = self.evaluate(condition)?;
        }

        Ok(())
    }

    fn visit_function(
        &mut self,
        name: &Token,
        parameters: &Box<Vec<Token>>,
        body: &mut Box<Vec<Stmt>>,
    ) -> Result<(), Error> {
        todo!()
    }
}
