use crate::error::*;
use crate::token::*;

pub struct Lexer {
    pub source_code: Vec<char>,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub errors: Vec<Error>,
}

impl Lexer {
    pub fn new(source_code: String) -> Lexer {
        Lexer {
            source_code: source_code.chars().collect(),
            tokens: Vec::new(),
            start: 0,   // Starts at the 0th character
            current: 0, // Current == Start in the beginning
            line: 1,    // Begin at line number 1
            errors: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();

        // Keep scanning for Tokens untill the end of file
        while self.is_at_end() {
            // start holds the start of the current lexeme being scanned
            // current tells the scan_token the position in the lexeme
            self.start = self.current;
            self.scan_token();
        }

        // Add the final Token, denoting the end of file
        tokens.push(Token::new(TokenType::EOF, String::from(""), None, 10));
        tokens
    }

    fn scan_token(&mut self) -> Result<(), i32> {
        let c = self.advance(); // Get current char and move current index
        match c {
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
                }
            }

            // Stirng literals
            '"' => self.string_literal(),

            c => {
                if c.is_digit(10) {
                    self.number_literal();
                } else {
                    self.errors.push(Error::new(
                        ErrorType::CompileTimeError,
                        "Unexpected Token".to_string(),
                        self.line,
                    ));
                }
            }
        }
        Ok(())
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
            self.errors.push(Error::new(
                ErrorType::CompileTimeError,
                "Unterminated String".to_string(),
                self.line,
            ));
        }

        // Consume the closing quote "
        let _ = self.advance();

        // Remove the surrounding quotes ->"..."<-
        let string_literal: String = self.source_code[self.start + 1..self.current]
            .iter()
            .collect();
        self.add_token(TokenType::String, Some(string_literal));
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
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<String>) {
        let text: String = self.source_code[self.start..=self.current].iter().collect();
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line))
    }

    fn is_at_end(&self) -> bool {
        self.current < self.source_code.len()
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
