use super::expr::*;
use super::stmt::*;
use crate::error::*;
use crate::lexer::token::*;

use super::astprinter::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<Error>,
}

/*
    Production rules

    program     → statement* EOF ;

    statement   → exprStmt | printStmt ;

    exprStmt    → expression ";" ;
    printStmt   → "print" expression ";" ;

    expression  → equality ;
    equality    → comparison ( ( "!=" | "==" ) comparison )* ;
    comparison  → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term        → factor ( ( "-" | "+" ) factor )* ;
    factor      → unary ( ( "/" | "*" ) unary )* ;
    unary       → ( "!" | "-" ) unary
                | primary ;
    primary     → NUMBER | STRING | "true" | "false" | "nil"
                | "(" expression ")" ;
*/
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Box<Stmt>>, Error> {
        let mut statements: Vec<Box<Stmt>> = Vec::new();

        // program  → statement* EOF ;
        while !self.is_at_end() {
            let mut new_statement_box = Box::new(self.statement()?);
            // statements.push(self.statement()?);
            statements.push(new_statement_box);
        }
        Ok(statements)
    }

    // Synchronizing to avoid cacading errors
    pub fn synchronize(&mut self) {
        // Call upon encountering a ParseError
        // Parser will ignore all-tokens till and including ";"
        // or untill encountering start of new statement
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::SemiColon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    let _ = self.advance();
                }
            };
        }
    }

    // statement  → expression_statement  |  print_statement
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_next(vec![TokenType::Print]) {
            Ok(self.print_statement()?)
        } else {
            Ok(self.expression_statement()?)
        }
    }

    // print_statement  → "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?; // "print" will be self."advance"d by caller

        // Expression ends and now at `;`
        self.consume(TokenType::SemiColon, "Expected `;` at the end".to_string());
        Ok(Stmt::print(Box::new(expr)))
    }

    // expression_statement  → expression ;
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;

        // Expression ends and now at `;`
        self.consume(TokenType::SemiColon, "Expected `;` at the end".to_string());
        Ok(Stmt::expression(Box::new(expr)))
    }

    // expression  → equality ;
    fn expression(&mut self) -> Result<Expr, Error> {
        self.equality()
    }

    // equality  → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_next(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            expr = Expr::binary(expr, self.previous(), self.comparison()?);
        }
        Ok(expr)
    }

    // comparison  → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_next(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            expr = Expr::binary(expr, self.previous(), self.term()?)
        }
        Ok(expr)
    }

    // term  → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        if self.match_next(vec![TokenType::Minus, TokenType::Plus]) {
            expr = Expr::binary(expr, self.previous(), self.factor()?);
        }

        Ok(expr)
    }

    // factor  → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;
        if self.match_next(vec![TokenType::Slash, TokenType::Star]) {
            expr = Expr::binary(expr, self.previous(), self.unary()?);
        }

        Ok(expr)
    }

    // unary  → ( "!" | "-" ) unary  |  primary ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_next(vec![TokenType::Bang, TokenType::Minus]) {
            return Ok(Expr::unary(self.previous(), self.unary()?));
        }
        self.primary()
    }

    // primary  → NUMBER | STRING | "true" | "false" | "nil"  |  "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_next(vec![
            TokenType::Nil,
            TokenType::True,
            TokenType::False,
            TokenType::String,
            TokenType::Number,
        ]) {
            return Ok(Expr::literal(self.previous().clone()));
        }

        if self.match_next(vec![TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                "Expect ')' after expresion.".to_string(),
            )
            .unwrap();
            return Ok(Expr::grouping(expr));
        }

        Err(self.push_error("Unexpected Token".to_string()))
    }

    // Move forward if "current" matches the type else error
    fn consume(&mut self, token_type: TokenType, message: String) -> Result<Token, Error> {
        if self.check(token_type) {
            return Ok(self.advance());
        }
        Err(self.push_error(message))
    }

    // Check if the "current" token is among the specified token types
    fn match_next(&mut self, next_token_types: Vec<TokenType>) -> bool {
        for token_type in next_token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    // Add error to the list
    // Let main handle reporting
    fn push_error(&mut self, error_message: String) -> Error {
        let error = Error::parser(error_message, self.peek().line);
        self.errors.push(error.clone());
        error
    }

    // Check if the "current" token is of the specified token type
    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }
}
