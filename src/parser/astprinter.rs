use super::expr::*;
use crate::lexer::token::*;

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(&mut self, expr: &mut Expr) -> String {
        expr.accept(self)
    }
}

impl Visitor<String> for AstPrinter {
    fn visit_binary(
        &mut self,
        left_expr: &mut Box<Expr>,
        operator: &Token,
        right_expr: &mut Box<Expr>,
    ) -> String {
        format!(
            "({} {} {})",
            left_expr.accept(self),
            operator.lexeme,
            right_expr.accept(self)
        )
    }

    fn visit_call(
        &mut self,
        callee: &mut Box<Expr>,
        closing_paren: &Token,
        arguments: &mut Box<Vec<Expr>>,
    ) -> String {
        let mut comma_seperated_arguments = String::new();
        for argument in (*arguments).iter_mut() {
            if comma_seperated_arguments.len() != 0 {
                comma_seperated_arguments += ", ";
            }
            comma_seperated_arguments += &argument.accept(self)
        }
        format!("{}({})", callee.accept(self), comma_seperated_arguments)
    }

    fn visit_get(&mut self, expr: &mut Box<Expr>, name: &Token) -> String {
        String::from("I dont care")
    }

    fn visit_set(&mut self, expr: &mut Box<Expr>, name: &Token, value: &mut Box<Expr>) -> String {
        String::from("I dont care again")
    }

    fn visit_grouping(&mut self, grouping_expr: &mut Box<Expr>) -> String {
        format!("({})", grouping_expr.accept(self))
    }

    fn visit_unary(&mut self, operator: &Token, unary_expr: &mut Box<Expr>) -> String {
        format!("({} {})", operator.lexeme, unary_expr.accept(self))
    }

    fn visit_literal(&mut self, token: &Token) -> String {
        match token.token_type {
            TokenType::String
            | TokenType::Number
            | TokenType::True
            | TokenType::False
            | TokenType::Nil => token.lexeme.clone(),
            _ => "(NOT IMPLEMENTED)".to_string(),
        }
    }

    fn visit_variable(&mut self, variable: &Token) -> String {
        match variable.token_type {
            TokenType::Identifier => variable.lexeme.clone(),
            _ => "(NOT IMPLEMENTED)".to_string(),
        }
    }

    fn visit_assign(&mut self, variable: &Token, expr: &mut Box<Expr>) -> String {
        format!("{} {}", variable.lexeme, expr.accept(self))
    }

    fn visit_logical(
        &mut self,
        left_expr: &mut Box<Expr>,
        logical_and_or: &mut Token,
        right_expr: &mut Box<Expr>,
    ) -> String {
        format!(
            "{} {} {}",
            left_expr.accept(self),
            logical_and_or.lexeme,
            right_expr.accept(self)
        )
    }
}
