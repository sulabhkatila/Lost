use crate::error::*;
use crate::token::*;

pub struct Lexer {
    pub source_code: String,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: usize,
    pub errors: Option<Vec<Error>>,
}

impl Lexer {
    pub fn new(source_code: String) -> Lexer {
        Lexer {
            source_code: source_code,
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
        while self.current < self.source_code.len() {
            // start holds the start of the current lexeme being scanned
            // current tells the scan_token the position in the lexeme
            self.start = self.current;
            self.scan_token();
        }

        // Add the final Token, denoting the end of file
        tokens.push(Token::new(TokenType::EOF, String::from(""), None, 10));
        tokens
    }

    fn scan_token(&mut self) -> Result<(), i32>{
        let source_code: Vec<char> = self.source_code.chars().collect();

        let c = source_code[self.current];
        self.current += 1;
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

            _ => {
                Error::report(ErrorType::CompileTimeError, "Unexpected Token".to_string(), self.line);
                return Err(1);
            },
        }
        Ok(())
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<String>) {
        let text = &self.source_code[self.start..=self.current];
        self.tokens
            .push(Token::new(token_type, text.to_string(), literal, self.line))
    }
}
