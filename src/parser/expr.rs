use crate::lexer::token::*;

#[derive(Debug, Clone)]
pub enum Expr {
    // AST nodes

    //         Expr::Biinary
    //         /     |     \
    //      Some    Some    Some
    //      Expr    Token   Expr
    //      ...             ...
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Box<Vec<Expr>>),
    Grouping(Box<Expr>),
    Unary(Token, Box<Expr>),
    Literal(Token),
    Logical(Box<Expr>, Token, Box<Expr>),
    Variable(Token),
    Assign(Token, Box<Expr>),
}

// Will change the Taking of owned variables and then converting it to Box
// take box right away
impl Expr {
    pub fn binary(left_expr: Expr, operator: Token, right_expr: Expr) -> Expr {
        Expr::Binary(Box::new(left_expr), operator, Box::new(right_expr))
    }

    pub fn call(callee: Expr, closing_paren: Token, arguments: Vec<Expr>) -> Expr {
        Expr::Call(Box::new(callee), closing_paren, Box::new(arguments))
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

    pub fn logical(left_expr: Expr, logical_and_or: Token, right_expr: Expr) -> Expr {
        Expr::Logical(Box::new(left_expr), logical_and_or, Box::new(right_expr))
    }

    pub fn variable(variable_name: Token) -> Expr {
        Expr::Variable(variable_name)
    }
}

pub trait Visitable<T> {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T;
}

impl<T> Visitable<T> for Expr {
    fn accept(&mut self, visitor: &mut impl Visitor<T>) -> T {
        match self {
            Expr::Binary(left, operator, right) => visitor.visit_binary(left, operator, right),
            Expr::Call(callee, closing_paren, arguments) => {
                visitor.visit_call(callee, closing_paren, arguments)
            }
            Expr::Grouping(expr) => visitor.visit_grouping(expr),
            Expr::Unary(operator, right) => visitor.visit_unary(operator, right),
            Expr::Literal(lit) => visitor.visit_literal(lit),
            Expr::Logical(left_expr, logical_and_or, right_expr) => {
                visitor.visit_logical(left_expr, logical_and_or, right_expr)
            }
            Expr::Variable(variable) => visitor.visit_variable(variable),
            Expr::Assign(token, expr) => visitor.visit_assign(token, expr),
        }
    }
}

// Any Visitor class to Expr must implement Visitor trait
pub trait Visitor<T> {
    fn visit_binary(
        &mut self,
        left_expr: &mut Box<Expr>,
        operator: &Token,
        right_expr: &mut Box<Expr>,
    ) -> T;
    fn visit_call(
        &mut self,
        callee: &mut Box<Expr>,
        closing_paren: &Token,
        arguments: &mut Box<Vec<Expr>>,
    ) -> T;
    fn visit_grouping(&mut self, grouping_expr: &mut Box<Expr>) -> T;
    fn visit_unary(&mut self, operator: &Token, unary_expr: &mut Box<Expr>) -> T;
    fn visit_literal(&mut self, lit: &Token) -> T;
    fn visit_logical(
        &mut self,
        left_expr: &mut Box<Expr>,
        logical_and_or: &mut Token,
        right_expr: &mut Box<Expr>,
    ) -> T;
    fn visit_variable(&mut self, variable: &Token) -> T;
    fn visit_assign(&mut self, variable: &Token, expr: &mut Box<Expr>) -> T;
}
