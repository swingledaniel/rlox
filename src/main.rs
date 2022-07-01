mod ast_display;
mod expr;
mod parser;
mod scanner;
mod token;
mod token_type;

use std::env;
use std::error::Error;
use std::fs;
use std::io::stdin;
use std::process;

use scanner::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        println!("Usage: rlox [script]");
    } else if args.len() == 2 {
        if let Err(error) = run_file(&args[1]) {
            println!("Error parsing file: {:?}", error);
        }
    } else {
        run_prompt();
    }
}

fn run_file(path: &str) -> Result<(), Box<dyn Error>> {
    let text: String = fs::read_to_string(path)?.parse()?;
    let had_error = run(&text);

    if had_error {
        process::exit(65);
    }

    Ok(())
}

fn run_prompt() {
    loop {
        print!("> ");
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Input invalid");
        if input.is_empty() {
            break;
        }
        run(&input);
    }
}

fn run(source: &str) -> bool {
    let scanner = Scanner::new(source);
    let (tokens, had_error) = scanner.scan_tokens();

    for token in tokens.iter() {
        println!("{:?}", token);
    }
    match parser::parse(tokens) {
        Ok(expr) => println!("{}", expr),
        Err((token, message)) => println!("{}, {}", token, message),
    };

    had_error
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, location, message);
}
