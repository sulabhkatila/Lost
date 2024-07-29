use super::expr::*;
use crate::lexer::token::*;

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Box<Vec<Stmt>>),
    Expression(Box<Expr>),
    IfElse(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>), // Condition, Then_branch, Else_branch
    Print(Box<Expr>),
    Var(Token, Option<Box<Expr>>),
}

impl Stmt {
    pub fn block(statements: Box<Vec<Stmt>>) -> Stmt {
        Stmt::Block(statements)
    }

    pub fn expression(expr: Box<Expr>) -> Stmt {
        Stmt::Expression(expr)
    }

    pub fn ifelse(
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) -> Stmt {
        Stmt::IfElse(condition, then_branch, else_branch)
    }

    pub fn print(expr: Box<Expr>) -> Stmt {
        Stmt::Print(expr)
    }

    pub fn var(variable_name: Token, expr: Option<Box<Expr>>) -> Stmt {
        Stmt::Var(variable_name, expr)
    }
}

pub trait Visitable<T> {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T;
}

impl<T> Visitable<T> for Stmt {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Stmt::Block(statements) => visitor.visit_block(statements),
            Stmt::Expression(expr) => visitor.visit_expression(expr),
            Stmt::IfElse(condition, then_branch, else_branch) => {
                visitor.visit_ifelse(condition, then_branch, else_branch)
            }
            Stmt::Print(expr) => visitor.visit_print(expr),
            Stmt::Var(token, expr) => visitor.visit_var(&token, &expr),
        }
    }
}

// Any Visitor class to Stmt must implement Visitor trait
pub trait Visitor<T> {
    fn visit_block(&mut self, statements: &mut Box<Vec<Stmt>>) -> T;
    fn visit_expression(&mut self, expr: &Box<Expr>) -> T;
    fn visit_ifelse(
        &mut self,
        condition: &Box<Expr>,
        then_branch: &Box<Stmt>,
        else_branch: &Option<Box<Stmt>>,
    ) -> T;
    fn visit_print(&mut self, expr: &Box<Expr>) -> T;
    fn visit_var(&mut self, token: &Token, expr: &Option<Box<Expr>>) -> T;
}
