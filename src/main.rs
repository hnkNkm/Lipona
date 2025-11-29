mod ast;
mod interpreter;
mod parser;
mod stdlib;

use std::env;
use std::fs;
use std::process;

use interpreter::Interpreter;
use parser::parse;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: lipona <file.lipo>");
        eprintln!("       lipona -e '<code>'");
        process::exit(1);
    }

    let code = if args[1] == "-e" {
        if args.len() < 3 {
            eprintln!("Error: -e requires code argument");
            process::exit(1);
        }
        args[2].clone()
    } else {
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("pakala: cannot read file '{filename}': {e}");
                process::exit(1);
            }
        }
    };

    match run(&code) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}

fn run(code: &str) -> Result<(), String> {
    // Parse
    let program = parse(code).map_err(|e| e.to_string())?;

    // Interpret
    let mut interpreter = Interpreter::new();
    interpreter.run(&program).map_err(|e| e.to_string())?;

    Ok(())
}
