use crate::lexer::token::*;

#[derive(Debug)]
pub enum Expr {
    // AST nodes

    //         Expr::Biinary
    //         /     |     \
    //      Some    Some    Some
    //      Expr    Token   Expr
    //      ...             ...
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Token),
}

impl Expr {
    pub fn binary(left_expr: Expr, operator: Token, right_expr: Expr) -> Expr {
        Expr::Binary(Box::new(left_expr), operator, Box::new(right_expr))
    }

    pub fn grouping(expr: Expr) -> Expr {
        Expr::Grouping(Box::new(expr))
    }

    pub fn unary(operator: Token, right_expr: Expr) -> Expr {
        Expr::Unary(operator, Box::new(right_expr))
    }

    pub fn literal(literalval: Token) -> Expr {
        Expr::Literal(literalval)
    }
}

pub trait Visitable<T> {
    fn accept(&self, visitor: &impl Visitor<T>) -> T;
}

impl<T> Visitable<T> for Expr {
    fn accept(&self, visitor: &impl Visitor<T>) -> T {
        match self {
            Expr::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expr::Literal(lit) => visitor.visit_literal(lit),
        }
    }
}

// Any Visitor class to Expr must implement Visitor trait
pub trait Visitor<T> {
    fn visit_binary(&self, left_expr: &Box<Expr>, operator: &Token, right_expr: &Box<Expr>) -> T;
    fn visit_grouping(&self, grouping_expr: &Box<Expr>) -> T;
    fn visit_unary(&self, operator: &Token, unary_expr: &Box<Expr>) -> T;
    fn visit_literal(&self, lit: &Token) -> T;
}
