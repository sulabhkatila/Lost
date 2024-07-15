use std::fmt::format;

use super::node::*;
use crate::lexer::token::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&self, expr: Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(
        &self,
        left_expr: &Box<Expr>,
        operator: &Token,
        right_expr: &Box<Expr>,
    ) -> String {
        format!(
            "({} {} {})",
            left_expr.accept(self),
            operator.lexeme,
            right_expr.accept(self)
        )
    }

    fn visit_grouping(&self, grouping_expr: &Box<Expr>) -> String {
        format!("({})", grouping_expr.accept(self))
    }

    fn visit_unary(&self, operator: &Token, unary_expr: &Box<Expr>) -> String {
        format!("({} {})", operator.lexeme, unary_expr.accept(self))
    }

    fn visit_literal(&self, token: &Token) -> String {
        match token.token_type {
            TokenType::String
            | TokenType::Number
            | TokenType::True
            | TokenType::False
            | TokenType::Nil => token.lexeme.clone(),
            _ => "(NOT IMPLEMENTED)".to_string(),
        }
    }
}
