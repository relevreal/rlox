use crate::ast_printer::AstPrinter;
use crate::error::{Error, LoxResult};
use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::token::Token;
use std::fs;
use std::io;
use std::io::Write;

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(&mut self, file_path: &str) -> LoxResult<()> {
        let source = fs::read_to_string(file_path).expect("should have been able to read the file");
        self.run(&source).unwrap();
        Ok(())
    }

    pub fn run_prompt(&mut self) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let mut line = String::new();

        loop {
            print!(">>> ");
            stdout.flush().unwrap();
            match stdin.read_line(&mut line) {
                Ok(0) => {
                    println!("0 bytes read, exiting...");
                    break;
                }
                Ok(_) => {
                    // Removes '\n'
                    line.pop();
                    if line == "exit()" {
                        break;
                    }
                    self.run(&line).unwrap();
                    line.clear();
                }
                Err(error) => panic!("something went wrong: {}", error),
            }
        }
    }

    fn run<'a>(&mut self, source: &'a str) -> LoxResult<'a, ()> {
        let mut lexer = Lexer::new(source);
        let tokens: Vec<Token> = lexer.tokenize()?;
        // for token in tokens {
        //     println!("{}", token);
        // }
        // let mut ast_printer = AstPrinter;
        let mut parser = Parser::new(tokens);
        let statements = parser.parse();
        // match expr {
        //     Some(e) => println!("{}", ast_printer.print(&e)),
        //     None => println!("Error when parsing tokens"),
        // }
        self.interpreter.interpret(statements);
        // match expr {
        //     Some(e) => {
        //         self.interpreter.interpret(statements);
        //     }
        //     None => println!("Error when parsing tokens"),
        // }

        Ok(())
    }
}
