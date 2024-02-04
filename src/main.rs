use rlox::lox::Lox;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: jlox [script]");
        return ExitCode::from(64);
    }
    let mut lox = Lox::new();
    if args.len() == 2 {
        println!("Running file: {}", args[1]);
        lox.run_file(&args[1]).unwrap();
    } else {
        println!("Running prompt");
        lox.run_prompt();
    }

    ExitCode::SUCCESS
}
