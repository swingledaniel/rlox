#![feature(is_some_with, let_chains)]

mod ast_display;
mod callable;
mod class;
mod environment;
mod expr;
mod instance;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
mod token;
mod token_type;
mod utils;

use std::env;
use std::error::Error;
use std::fs;
use std::io::{stdin, stdout, Write};
use std::process;

use environment::Environment;
use interpreter::interpret;
use scanner::Scanner;
use utils::Soo;

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
    let mut environment = Environment::new();

    let (had_error, had_runtime_error) = run(&text, &mut environment);

    if had_error {
        process::exit(65);
    }
    if had_runtime_error {
        process::exit(70);
    }

    Ok(())
}

fn run_prompt() {
    let mut environment = Environment::new();
    loop {
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Input invalid");
        if input.is_empty() {
            break;
        }
        run(&input, &mut environment);
    }
}

fn run(source: &str, environment: &mut Environment) -> (bool, bool) {
    let scanner = Scanner::new(source);
    let (tokens, had_error) = scanner.scan_tokens();

    if had_error {
        return (had_error, false);
    }

    match parser::parse(tokens) {
        Ok(mut statements) => {
            let mut had_error = false;
            if resolver::resolve_statements(
                &mut statements,
                environment,
                &mut Vec::new(),
                &mut Vec::new(),
                &mut had_error,
            )
            .is_err()
                || had_error
            {
                (true, false)
            } else {
                (false, interpret(statements, environment))
            }
        }
        Err(_errors) => {
            println!("Parse errors encountered.");
            (true, false)
        }
    }
}

fn error(line: usize, message: &Soo) {
    report(line, "", message);
}

fn report(line: usize, location: &str, message: &Soo) {
    println!("[line {}] Error{}: {}", line, location, message);
}

fn runtime_error(line: usize, message: &mut Soo) {
    println!("{}\n[line {}]", message, line);
}
