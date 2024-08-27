use std::{
    env, fs,
    io::{self, Write},
};

use interpreter::Interpreter;
use lost::{
    interpreter::*,
    lexer::lexer::*,
    parser::{astprinter::AstPrinter, parser::*},
};

fn main() {
    let argv: Vec<String> = env::args().collect();

    match argv.len() {
        1 => {
            // Run Repl
            // > ...
            run_prompt();
        }
        2 => run_file(&argv[1]),
        _ => {
            eprintln!("Usage: {} [script]", argv[0]);
        }
    }
}

fn run_file(filepath: &String) {
    // Get the source code from the file
    let source_code = match fs::read_to_string(filepath) {
        Ok(file) => file,
        _ => {
            eprintln!("`{filepath}` does not exist");
            return;
        }
    };

    // Start interpreting
    run(source_code)
}

fn run_prompt() {
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut new_input = String::new();
        match io::stdin().read_line(&mut new_input) {
            Err(_) => continue,
            Ok(_) => {
                run(new_input);
            }
        };
    }
}

fn run(code: String) {
    let mut lexer: Lexer = Lexer::new(code);
    lexer.scan();

    let tokens = lexer.tokens;

    let mut parser = Parser::new(tokens);
    parser.parse();

    let parser_errors = parser.get_errors();

    if parser_errors.len() > 0 {
        for parser_error in parser_errors {
            parser_error.report()
        }

        return;
    }

    let statements = parser.get_parsed_statements();
    let ast_printer = AstPrinter;
    let mut interpreter = Interpreter::new(None);

    if let Err(interpreter_err) = interpreter.interpret(statements) {
        interpreter_err.report();
    }
}
