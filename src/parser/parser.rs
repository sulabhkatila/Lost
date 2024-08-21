use super::{expr::*, stmt::*};
use std::io::{self, Write};

use crate::{error::*, lexer::token::*};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    statements: Vec<Box<Stmt>>,
    errors: Vec<Box<Error>>,
}

/*
    Production rules

    program     -> declaration* EOF ;

    declaration -> fun_declaration | var_declaration | statement ;

    fun_declaration    -> "fun" function ;
    function           -> IDENTIFIER "(" parameters? ")" block ;
    parameters         -> IDENTIFIER ( "," IDENTIFIER )* ;

    var_declaration    -> "var" IDENTIFIER ( "=" expression )? ";" ;
    statement          -> expression_statement | for_statement | while_statement
                        | if_statement | print_statement | return_statement | block ;

    for_statement      -> "for" "(" ( var_declaration | expression_statement | ";" )
                        expression? ";"
                        expression? ")" statement ;
    while_statement    -> "while" "(" expression ")" statement ;
    if_statement       -> "if" "(" expression ")" statement ("else" statement)? ;
    return_statement   -> "return" expression? ;
    block              -> "{" declaration* "}" ;

    expression_statement    -> expression ";" ;
    print_statement         -> "print" expression ";" ;

    expression  -> assignment ;
    assignment  -> IDENTIFIER "=" assignment | logic_or ;
    logic_or    -> logic_and ( "or" logic_and )* ;
    logic_and   -> equality ( "and" equality )* ;
    equality    -> comparison ( ( "!=" | "==" ) comparison )* ;
    comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    term        -> factor ( ( "-" | "+" ) factor )* ;
    factor      -> unary ( ( "/" | "*" ) unary )* ;
    unary       -> ( "!" | "-" ) unary
                | call ;
    call        -> primary ( "(" arguments? ")" )* ;
    arguments   -> expression ( "," expression )* ;
    primary     -> NUMBER | STRING | IDENTIFIER | "true" | "false"
                | "nil" | "(" expression ")";
*/

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            statements: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn get_parsed_statements(&mut self) -> &mut Vec<Box<Stmt>> {
        &mut self.statements
    }

    pub fn get_errors(&mut self) -> &Vec<Box<Error>> {
        &self.errors
    }

    pub fn parse(&mut self) {
        // program  -> statement* EOF ;
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => self.statements.push(Box::new(statement)),
                Err(_) => self.synchronize(),
            }
        }
    }

    // Synchronizing to avoid cacading errors
    pub fn synchronize(&mut self) {
        // Call upon encountering a ParseError
        // Parser will ignore all-tokens till and including ";"
        // or untill encountering start of new statement

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

    // declaration -> fun_declaration | var_declaration | statement ;
    // just a special statement
    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.match_next(vec![TokenType::Fun]) {
            Ok(self.fun_declaration()?)
        } else if self.match_next(vec![TokenType::Var]) {
            Ok(self.var_declaration()?)
        } else {
            Ok(self.statement()?)
        }
    }

    // fun_declaration -> "fun" IDENTIFIER "(" parameters ")" block ;
    fn fun_declaration(&mut self) -> Result<Stmt, Error> {
        self.function(String::from("function"))
    }

    fn function(&mut self, callable_type: String) -> Result<Stmt, Error> {
        let name = self.consume(
            TokenType::Identifier,
            format!("Expected a {} name", callable_type),
        )?;
        let _ = self.consume(
            TokenType::LeftParen,
            "Expected `(` after function name in declaration".to_string(),
        );

        let mut parameters = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if parameters.len() == 255 {
                    self.push_error("Too many parameters: 255 parameters allowed".to_string());
                }

                parameters.push(self.consume(
                    TokenType::Identifier,
                    "Expected a parameter name".to_string(),
                )?);
                if !self.match_next(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let _ = self.consume(
            TokenType::RightParen,
            "Expected a `)` after parameters".to_string(),
        );

        let _ = self.consume(
            TokenType::LeftBrace,
            "Expected `{` in function declaration and define function block".to_string(),
        );
        let body = self.block()?;

        Ok(Stmt::function(name, Box::new(parameters), Box::new(body)))
    }

    // var_declaration -> "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, Error> {
        let variable_name = self.consume(
            TokenType::Identifier,
            "Expected a variable name".to_string(),
        )?;

        let mut initializer: Option<Box<Expr>> = None;
        if self.match_next(vec![TokenType::Equal]) {
            initializer = Some(Box::new(self.expression()?));
        }

        self.consume(TokenType::SemiColon, "Expected `;` in the end".to_string())?;
        Ok(Stmt::var(variable_name, initializer))
    }

    // statement  -> expression_statement | for_statement | while_statement | if_statement
    //              | print_statement | return_statement | block ;
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_next(vec![TokenType::For]) {
            self.for_statement()
        } else if self.match_next(vec![TokenType::While]) {
            self.while_statement()
        } else if self.match_next(vec![TokenType::If]) {
            self.if_statement()
        } else if self.match_next(vec![TokenType::Print]) {
            self.print_statement()
        } else if self.match_next(vec![TokenType::Return]) {
            self.return_statement()
        } else if self.match_next(vec![TokenType::LeftBrace]) {
            Ok(Stmt::block(Box::new(self.block()?)))
        } else {
            self.expression_statement()
        }
    }

    // for_statement  -> "for" "(" ( var_declaration | expression_statement | ";" )
    //                    expression? ";"
    //                    expression? ")" statement ;
    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected `(` after `for`".to_string())?;

        // for (var i = 0 ; i < 1 ; i = i + 1) {......}
        //      ^^^         ^^^      ^^^           ^^^
        // initializer   condition   incrementer   loop_body

        let mut initializer: Option<Stmt> = None;
        if self.match_next(vec![TokenType::SemiColon]) {
            initializer = None
        } else if (self.match_next(vec![TokenType::Var])) {
            initializer = Some(self.var_declaration()?)
        } else {
            initializer = Some(self.expression_statement()?)
        }

        let mut condition: Option<Expr> = None;
        if !self.check(TokenType::SemiColon) {
            condition = Some(self.expression()?)
        }
        self.consume(
            TokenType::SemiColon,
            "Expected `;` after loop condition".to_string(),
        )?;

        let mut incrementer: Option<Expr> = None;
        if !self.check(TokenType::SemiColon) {
            incrementer = Some(self.expression()?)
        }
        self.consume(
            TokenType::RightParen,
            "Expected `)` after for clauses".to_string(),
        )?;

        let mut loop_body = self.statement()?;

        // Desugar for loop into while loop
        //
        // this:
        // for (var i = 0; i < 1; i = i + 1) {...}
        //
        // to:
        // var i = 0;
        // while (i < 1) {
        // ...
        // i = i + 1
        // }

        if let Some(incrementer_) = incrementer {
            loop_body = Stmt::block(Box::new(vec![
                loop_body,
                Stmt::expression(Box::new(incrementer_)),
            ]));
        }

        if let None = condition {
            condition = Some(Expr::literal(Token::new(
                TokenType::True,
                "true".to_string(),
                None,
                1, // Line doesn't matter
            )))
        }
        loop_body = Stmt::whileloop(Box::new(condition.unwrap()), Box::new(loop_body));

        if let Some(initializer_) = initializer {
            loop_body = Stmt::block(Box::new(vec![initializer_, loop_body]))
        }

        Ok(loop_body)
    }

    // while_statement  -> "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected `(` after while".to_string())?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected `)` after condition".to_string(),
        )?;

        let loop_body = self.statement()?;

        Ok(Stmt::WhileLoop(Box::new(condition), Box::new(loop_body)))
    }

    // if_statement  -> "if" "(" expression ")" statement ("else" statement)? ;
    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected `(` after if".to_string())?;
        let condition = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "Expected `)` after condition".to_string(),
        )?;

        let then_branch = self.statement()?;

        if self.match_next(vec![TokenType::Else]) {
            let else_branch = self.statement()?;
            return Ok(Stmt::IfElse(
                Box::new(condition),
                Box::new(then_branch),
                Some(Box::new(else_branch)),
            ));
        }

        Ok(Stmt::ifelse(
            Box::new(condition),
            Box::new(then_branch),
            None,
        ))
    }

    // return_statement  -> "return" expression? ;
    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let return_keyword = self.previous();
        let mut return_value = Expr::literal(Token::new(
            TokenType::Nil,
            "nil".to_string(),
            None,
            return_keyword.line,
        ));

        if !self.check(TokenType::SemiColon) {
            return_value = self.expression()?
        }

        let _ = self.consume(
            TokenType::SemiColon,
            "Expected a `;` in the end of a statement".to_string(),
        )?;

        Ok(Stmt::ret(return_keyword, Box::new(return_value)))
    }

    // block  -> "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::<Stmt>::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(
            TokenType::RightBrace,
            "Expected `}` at the end of block".to_string(),
        )?;
        Ok(statements)
    }

    // print_statement  -> "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?; // "print" will be self."advance"d by caller

        // Expression ends and now at `;`
        self.consume(TokenType::SemiColon, "Expected `;` at the end".to_string())?;
        Ok(Stmt::print(Box::new(expr)))
    }

    // expression_statement  -> expression ;
    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;

        // Expression ends and now at `;`
        self.consume(TokenType::SemiColon, "Expected `;` at the end".to_string())?;
        Ok(Stmt::expression(Box::new(expr)))
    }

    // expression  -> assignment ;
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    // assignment  -> IDENTIFIER "=" assignment | logic_or ;
    fn assignment(&mut self) -> Result<Expr, Error> {
        let left_side_identifier = self.logic_or()?;

        if self.match_next(vec![TokenType::Equal]) {
            let equals = self.previous();
            let right_side_expr = self.assignment()?;

            match left_side_identifier {
                Expr::Variable(token) => return Ok(Expr::Assign(token, Box::new(right_side_expr))),
                _ => {
                    return Err(self.push_error("Invalid assignment target".to_string()));
                }
            }
        }
        Ok(left_side_identifier)
    }

    // logic_or  -> logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> Result<Expr, Error> {
        let left_expr = self.logic_and()?;

        if self.match_next(vec![TokenType::Or]) {
            let logical_or = self.previous();
            let right_expr = self.logic_and()?;
            return Ok(Expr::logical(left_expr, logical_or, right_expr));
        }

        Ok(left_expr)
    }

    // logic_and  -> equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Result<Expr, Error> {
        let left_expr = self.equality()?;

        if self.match_next(vec![TokenType::And]) {
            let logical_and = self.previous();
            let right_expr = self.equality()?;
            return Ok(Expr::logical(left_expr, logical_and, right_expr));
        }

        Ok(left_expr)
    }

    // equality  -> comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_next(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            expr = Expr::binary(expr, self.previous(), self.comparison()?);
        }
        Ok(expr)
    }

    // comparison  -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
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

    // term  -> factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;
        if self.match_next(vec![TokenType::Minus, TokenType::Plus]) {
            expr = Expr::binary(expr, self.previous(), self.factor()?);
        }

        Ok(expr)
    }

    // factor  -> unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;
        if self.match_next(vec![TokenType::Slash, TokenType::Star]) {
            expr = Expr::binary(expr, self.previous(), self.unary()?);
        }

        Ok(expr)
    }

    // unary  -> ( "!" | "-" ) unary  |  call ;
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_next(vec![TokenType::Bang, TokenType::Minus]) {
            return Ok(Expr::unary(self.previous(), self.unary()?));
        }
        self.call()
    }

    // call  -> primary ( "(" arguments? ")" )* ;
    fn call(&mut self) -> Result<Expr, Error> {
        let mut expression = self.primary()?;

        loop {
            if self.match_next(vec![TokenType::LeftParen]) {
                expression = self.finish_call(expression)?;
            } else {
                break;
            }
        }
        Ok(expression)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = Vec::new();
        let mut error = None;

        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    let _ = self.push_error(
                        "Too many arguments. (A function can have at max 255 arguments)"
                            .to_string(),
                    );
                }
                arguments.push(self.expression()?);
                if !self.match_next(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let closing_paren = self.consume(
            TokenType::RightParen,
            "Expected `)` after arguments".to_string(),
        )?;

        if let Some(err) = error {
            return Err(err);
        }
        Ok(Expr::call(callee, closing_paren, arguments))
    }

    // primary  -> NUMBER | STRING | IDENTIFIER | "true" | "false"
    //           | "nil"  |  "(" expression ")";
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

        if self.match_next(vec![TokenType::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
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
        if self.check(token_type.clone()) {
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
        let error = Error::parser(error_message, self.previous().line);
        self.errors.push(Box::new(error.clone()));
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
