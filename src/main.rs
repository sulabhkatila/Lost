use std::env;
use std::fs;
use std::io::{self, Write};

use interpreter::Interpreter;
use lost::interpreter::*;
use lost::lexer::lexer::*;
use lost::parser::astprinter::AstPrinter;
use lost::parser::parser::*;
use lost::parser::stmt::*;

fn main() {
    let argv: Vec<String> = env::args().collect();
    if argv.len() > 2 {
        writeln!(io::stderr(), "Usage: {} [script]", argv[0]).unwrap();
        return;
    } else if argv.len() == 2 {
        // Run code from the given file
        run_file(&argv[1]);
    } else {
        // Run REPL
        // > ...
        run_prompt();
    }
}

fn run_file(filepath: &String) {
    // Get the source code from the file
    let source_code = match fs::read_to_string(filepath) {
        Ok(file) => file,
        _ => {
            writeln!(io::stderr(), "`{filepath}` does not exist").unwrap();
            return; // Quit if no file
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

    let mut parser = Parser::new(lexer.tokens);
    let parsed = parser.parse();

    let ast_printer = AstPrinter;
    match parsed {
        Ok(mut val_vec) => {
            let mut interpreter = Interpreter::new(None);
            let interpreter_res = interpreter.interpret(&mut val_vec);
            match interpreter_res {
                Ok(_) => {},
                Err(error) => {println!("{:#?}", error)},
            }
        }
        _ => println!("Error on parsing"),
    }
}
