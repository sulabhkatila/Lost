use super::expr::*;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>),
}

impl Stmt {
    pub fn expression(expr: Box<Expr>) -> Stmt {
        Stmt::Expression(expr)
    }

    pub fn print(expr: Box<Expr>) -> Stmt {
        Stmt::Print(expr)
    }
}

pub trait Visitable<T> {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T;
}

impl<T> Visitable<T> for Stmt {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Stmt::Expression(expr) => visitor.visit_expression(expr),
            Stmt::Print(expr) => visitor.visit_print(expr),
        }
    }
}

// Any Visitor class to Stmt must implement Visitor trait
pub trait Visitor<T> {
    fn visit_expression(&mut self, expr: &Box<Expr>) -> T;
    fn visit_print(&mut self, expr: &Box<Expr>) -> T;
}
