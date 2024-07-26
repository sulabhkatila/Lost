use super::environment::*;
use super::environment::*;
use super::types::*;
use crate::error::*;
use crate::lexer::token::*;
use crate::parser::expr::Visitor as ExpressionVisitor;
use crate::parser::expr::*;
use crate::parser::stmt::Visitable as StatementVisitable;
use crate::parser::stmt::Visitor as StatementVisitor;
use crate::parser::stmt::*;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new(enclosing: Option<Environment>) -> Interpreter {
        Interpreter {
            environment: Environment::new(enclosing),
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
        }
    }

    // Determines the truthiness of a Type value.
    // Returns true for non-empty strings, non-zero numbers, and true booleans.
    // Returns false for zero numbers, false booleans, and Nil values.
    pub fn is_truthly(&self, value: Type) -> bool {
        match value {
            Type::String(_) => true,
            Type::Number(val) => {
                if val == 0.0 {
                    false
                } else {
                    true
                }
            }
            Type::Boolean(val) => val,
            Type::Nil => false,
        }
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
            TokenType::Bang => return Ok(Type::Boolean(!self.is_truthly(right))),
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
        self.environment.get(variable)
    }

    fn visit_assign(&mut self, variable: &Token, expr: &mut Box<Expr>) -> Result<Type, Error> {
        let value = self.evaluate(expr)?;
        let _ = self.environment.assign(variable.clone(), value.clone())?;
        Ok(value)
    }
}

impl StatementVisitor<Result<(), Error>> for Interpreter {
    fn visit_expression(&mut self, expr: &Box<Expr>) -> Result<(), Error> {
        let value = self.evaluate(expr)?;
        println!("{}", value);

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
                self.environment.define(token.lexeme.clone(), val);
            }
            _ => self.environment.define(token.lexeme.clone(), Type::Nil),
        }
        Ok(())
    }
}
