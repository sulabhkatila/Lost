use crate::token::*;

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
    Literal(LiteralType),
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

    pub fn literal(literalval: LiteralType) -> Expr {
        Expr::Literal(literalval)
    }
}
