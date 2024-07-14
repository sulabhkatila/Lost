use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum ErrorType {
    LexError,
    ParseError,
}

#[derive(Debug, Clone)]
pub struct Error {
    error_type: ErrorType,
    message: String,
    line: usize,
}

impl Error {
    pub fn new(error_type: ErrorType, message: String, line: usize) -> Error {
        Error {
            error_type,
            message,
            line,
        }
    }

    pub fn report(&self) {
        writeln!(
            io::stderr(),
            "{:?}\n{} at line {}",
            self.error_type,
            self.message,
            self.line
        )
        .unwrap();
    }
}
