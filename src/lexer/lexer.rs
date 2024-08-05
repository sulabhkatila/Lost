use super::token::*;
use crate::error::*;

use std::collections::HashMap;

pub struct Lexer<'lexer> {
    pub source_code: Vec<char>,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub errors: Vec<Error>,
    keywords: HashMap<&'lexer str, TokenType>,
}

impl<'lexer> Lexer<'lexer> {
    pub fn new(source_code: String) -> Lexer<'lexer> {
        Lexer {
            source_code: source_code.chars().collect(),
            tokens: Vec::new(),
            start: 0,   // Starts at the 0th character
            current: 0, // Current == Start in the beginning
            line: 1,    // Begin at line number 1
            errors: Vec::new(),
            keywords: HashMap::from([
                ("and", TokenType::And),
                ("class", TokenType::Class),
                ("else", TokenType::Else),
                ("false", TokenType::False),
                ("for", TokenType::For),
                ("fun", TokenType::Fun),
                ("if", TokenType::If),
                ("nil", TokenType::Nil),
                ("or", TokenType::Or),
                ("print", TokenType::Print),
                ("return", TokenType::Return),
                ("super", TokenType::Super),
                ("this", TokenType::This),
                ("true", TokenType::True),
                ("var", TokenType::Var),
                ("while", TokenType::While),
            ]),
        }
    }

    pub fn scan(&mut self) {
        // Keep scanning for Tokens untill the end of file
        while !self.is_at_end() {
            // start holds the start of the current lexeme being scanned
            // current tells the scan_token the position in the lexeme
            self.start = self.current;
            self.scan_token();
        }

        // Add the final Token, denoting the end of file
        self.tokens.push(Token::new(
            TokenType::EOF,
            String::from(""),
            None,
            self.line,
        ));
    }

    fn scan_token(&mut self) {
        let c = self.advance(); // Get current char and move current index
        match c {
            // New line
            '\n' => self.line += 1,

            // Single Character tokens
            '(' => self.add_token(TokenType::LeftParen, None),
            ')' => self.add_token(TokenType::RightParen, None),
            '{' => self.add_token(TokenType::LeftBrace, None),
            '}' => self.add_token(TokenType::RightBrace, None),
            '.' => self.add_token(TokenType::Dot, None),
            ',' => self.add_token(TokenType::Comma, None),
            '+' => self.add_token(TokenType::Plus, None),
            '-' => self.add_token(TokenType::Minus, None),
            '*' => self.add_token(TokenType::Star, None),
            ';' => self.add_token(TokenType::SemiColon, None),

            // Single or Double Character tokens
            '!' => {
                // '!=' or just '='
                let is_bang_equal = self.match_next('=');

                if is_bang_equal {
                    self.add_token(TokenType::BangEqual, None);
                } else {
                    self.add_token(TokenType::Bang, None);
                }
            }
            '=' => {
                // '==' or '='
                let is_equal_equal = self.match_next('=');

                if is_equal_equal {
                    self.add_token(TokenType::EqualEqual, None);
                } else {
                    self.add_token(TokenType::Equal, None);
                }
            }
            '<' => {
                // '<=' or '<'
                let is_less_equal = self.match_next('=');

                if is_less_equal {
                    self.add_token(TokenType::LessEqual, None);
                } else {
                    self.add_token(TokenType::Less, None);
                }
            }
            '>' => {
                // '>=' or '<'
                let is_greater_equal = self.match_next('=');

                if is_greater_equal {
                    self.add_token(TokenType::GreaterEqual, None);
                } else {
                    self.add_token(TokenType::Greater, None);
                }
            }

            // Longer tokens
            '/' => {
                // '//' (comment) or '/' (division)
                if self.match_next('/') {
                    // Ignore everything till the end of line
                    let mut next_char = self.peek();
                    while next_char != '\n' && next_char != '\0' {
                        let _ = self.advance();
                        next_char = self.peek();
                    }
                } else {
                    self.add_token(TokenType::Slash, None)
                }
            }

            // Stirng literals
            '"' => self.string_literal(),

            c => {
                if c.is_digit(10) {
                    // Numeric literals
                    self.number_literal();
                } else if Self::is_alpha(c) {
                    // Identifier (user defined and Keywords)
                    self.identifier();
                } else {
                    // Invalid character
                    // Add the error to the list, main will report
                    self.errors
                        .push(Error::lexer("Unexpected Token".to_string(), self.line));
                }
            },
        }
    }

    fn identifier(&mut self) {
        // Assume it is only called when is_alpha is true for first char
        while Self::is_alphanumeric(self.peek()) {
            self.advance();
        }

        let identifier_text: String = self.source_code[self.start..self.current].iter().collect();
        match self.keywords.get(&identifier_text.as_str()) {
            Some(val) => self.add_token(val.clone(), None),
            None => self.add_token(TokenType::Identifier, None),
        }
    }

    fn string_literal(&mut self) {
        // Get the complete literal
        let mut next_char = self.peek();
        while next_char != '"' && next_char != '\0' {
            if next_char == '\n' {
                self.line += 1;
            }
            let _ = self.advance();
            next_char = self.peek();
        }

        if next_char == '\0' {
            // The string literal was not terminated
            self.errors
                .push(Error::lexer("Unterminated String".to_string(), self.line));
        }

        // Consume the closing quote "
        let _ = self.advance();

        // Remove the surrounding quotes ->"..."<-
        let string_literal: String = self.source_code[self.start + 1..self.current - 1]
            .iter()
            .collect();
        self.add_token(
            TokenType::String,
            Some(LiteralType::StringType(string_literal)),
        );
    }

    fn number_literal(&mut self) {
        while self.peek().is_digit(10) {
            self.advance();
        }

        // Decimals
        if self.peek() == '.' && self.peek_next().is_digit(10) {
            // Consume the "."
            self.advance();
            while self.peek().is_digit(10) {
                self.advance();
            }
        }

        let num_literal: f32 = self.source_code[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap();
        self.add_token(
            TokenType::Number,
            Some(LiteralType::NumberType(num_literal)),
        )
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<LiteralType>) {
        let text: String = self.source_code[self.start..self.current].iter().collect();
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line))
    }

    fn is_alpha(c: char) -> bool {
        // abc..z + ABC..Z + _
        c.is_ascii_alphabetic() || c == '_'
    }

    fn is_alphanumeric(c: char) -> bool {
        // abc..z + ABC..Z + _ + 0..9
        Self::is_alpha(c) || c.is_digit(10)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source_code.len()
    }

    fn match_next(&mut self, expected_next: char) -> bool {
        // There is no next character if already at end
        if self.is_at_end() {
            return false;
        }

        if self.source_code[self.current] != expected_next {
            return false;
        }
        let _ = self.advance(); // Move current, it is the part of this token
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0'; // The End
        }
        self.source_code[self.current]
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source_code.len() {
            return '\0'; // The End
        }
        self.source_code[self.current + 1]
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source_code[self.current - 1]
    }
}
