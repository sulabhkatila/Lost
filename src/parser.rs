use crate::error::*;
use crate::node::*;
use crate::token::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<Error>,
}

/*
    Production rules

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

    pub fn parse(&mut self) -> Result<Expr, Error> {
        self.expression()
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
        if self.match_next(vec![TokenType::Nil]) {
            return Ok(Expr::literal(LiteralType::StringType("".to_string())));
        }
        if self.match_next(vec![TokenType::True]) {
            return Ok(Expr::literal(LiteralType::StringType("true".to_string())));
        }
        if self.match_next(vec![TokenType::False]) {
            return Ok(Expr::literal(LiteralType::StringType("false".to_string())));
        }

        if self.match_next(vec![TokenType::String, TokenType::Number]) {
            return Ok(Expr::literal(match self.previous().literal {
                Some(val) => match val {
                    LiteralType::StringType(string_val) => LiteralType::StringType(string_val),
                    LiteralType::NumberType(number_val) => LiteralType::NumberType(number_val),
                },
                _ => {
                    // Error
                    return Err(self.push_error("Unexpected Token".to_string()));
                }
            }));
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
        let error = Error::new(ErrorType::ParseError, error_message, self.peek().line);
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
