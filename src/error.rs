use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum ErrorType {
    LexError,
    ParseError,
    InterpretError,
}

#[derive(Debug, Clone)]
pub enum Error {
    LexError(String, usize),
    ParseError(String, usize),
    InterpretError(String, usize),
}

impl Error {
    pub fn lexer(message: String, line: usize) -> Error {
        Error::LexError(message, line)
    }

    pub fn parser(message: String, line: usize) -> Error {
        Error::ParseError(message, line)
    }

    pub fn interpreter(message: String, line: usize) -> Error {
        Error::InterpretError(message, line)
    }

    pub fn report(&self) {
        match self {
            Error::LexError(message, line) => {
                writeln!(io::stderr(), "LexError: {} at line {}", message, line)
            }
            Error::ParseError(message, line) => {
                writeln!(io::stderr(), "ParseError: {} at line {}", message, line)
            }
            Error::InterpretError(message, line) => {
                writeln!(io::stderr(), "InterpretError; {} at line {}", message, line)
            }
        };
    }
}
