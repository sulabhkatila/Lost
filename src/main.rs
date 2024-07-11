use std::env;
use std::fs;
use std::io::{self, Write};

use lost::lexer::*;

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
    let source_code = match fs::read_to_string(filepath) {
        Ok(file) => file,
        _ => {
            writeln!(io::stderr(), "`{filepath}` does not exist").unwrap();
            return;
        }
    };

    run(source_code)
}

fn run_prompt() {
    let mut input = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut new_input = String::new();
        match io::stdin().read_line(&mut new_input) {
            Err(_) => continue,
            Ok(_) => {
                input.extend(new_input.chars());
                run(new_input);
            }
        };
    }
}

fn run(code: String) {
    let mut lexer: Lexer = Lexer::new(code);
    lexer.scan_code();
}
