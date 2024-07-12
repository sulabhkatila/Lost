use std::io::{self, Write};

#[derive(Debug)]
pub enum ErrorType {
    CompileTimeError,
    RunTimeError,
}

pub struct Error;
impl Error {
    pub fn report(error_type: ErrorType, message: String, line: usize) {
        writeln!(io::stderr(), "{:?}\n{} at line {}", error_type, message, line).unwrap();
    }
}