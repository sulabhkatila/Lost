use crate::error::*;
use crate::token::*;

pub struct Lexer {
    pub source_code: Vec<char>,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub errors: Option<Vec<Error>>,
}

impl Lexer {
    pub fn new(source_code: String) -> Lexer {
        Lexer {
            source_code: source_code.chars().collect(),
            tokens: Vec::new(),
            start: 0,   // Starts at the 0th character
            current: 0, // Current == Start in the beginning
            line: 1,    // Begin at line number 1
            errors: None,
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

            _ => {
                Error::report(
                    ErrorType::CompileTimeError,
                    "Unexpected Token".to_string(),
                    self.line,
                );
                return Err(1);
            }
        }
        Ok(())
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

    fn add_token(&mut self, token_type: TokenType, literal: Option<String>) {
        let text: String = self.source_code[self.start..=self.current].iter().collect();
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line))
    }

    fn is_at_end(&self) -> bool {
        self.current < self.source_code.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source_code[self.current - 1]
    }
}
