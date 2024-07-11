use crate::token::*;

pub struct Lexer {
    source_code: String,
}

impl Lexer {
    pub fn new(source_code: String) -> Lexer {
        Lexer {
            source_code: source_code,
        }
    }

    pub fn scan_code(&self) -> Vec<Token> {
        let mut tokens: Vec<Token> = Vec::new();
        for c in self.source_code.chars() {
            print!("{c}");
        }
        tokens
    }
}
