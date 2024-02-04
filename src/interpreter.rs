use crate::environment::Environment;
use crate::error::{Error, LoxResult};
use crate::expr::Expr;
use crate::lox_callable::LoxCallable;
use crate::stmt::Stmt;
use crate::stmt::Stmt::Print;
use crate::token::{Literal, TokenKind};
use crate::visitor::Visitor;
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// TODO: swap stdout for generic writer, good for tests, maybe also for other reasons?
pub struct Interpreter {
    globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
}

// Need to change literal to token, for error handling???
impl<'a> Visitor<LoxResult<'a, Literal>, LoxResult<'a, ()>> for Interpreter {
    fn visit_expr(&mut self, e: &Expr) -> LoxResult<'a, Literal> {
        match e {
            Expr::Literal(literal) => Ok(literal.clone()),
            Expr::Grouping(ref e) => self.visit_expr(e),
            Expr::Unary {
                ref right,
                operator,
            } => {
                let right = self.visit_expr(right)?;

                match (right, operator.kind) {
                    (Literal::Number(number), TokenKind::Minus) => Ok(Literal::Number(-number)),
                    (r, TokenKind::Bang) => Ok(Literal::Bool(!is_truthy(&r))),
                    _ => unreachable!(),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;
                let right = self.visit_expr(right)?;

                // Maybe change to two levels of match operation.kind => literal
                match (left, right, operator.kind) {
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Minus) => {
                        Ok(Literal::Number(n1 - n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Plus) => {
                        Ok(Literal::Number(n1 + n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Slash) => {
                        Ok(Literal::Number(n1 / n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Star) => {
                        Ok(Literal::Number(n1 * n2))
                    }
                    (Literal::String(s1), Literal::String(s2), TokenKind::Plus) => {
                        Ok(Literal::String(s1 + &s2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Greater) => {
                        Ok(Literal::Bool(n1 > n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::GreaterEqual) => {
                        Ok(Literal::Bool(n1 >= n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::Less) => {
                        Ok(Literal::Bool(n1 < n2))
                    }
                    (Literal::Number(n1), Literal::Number(n2), TokenKind::LessEqual) => {
                        Ok(Literal::Bool(n1 <= n2))
                    }
                    (r, l, TokenKind::Equal) => Ok(Literal::Bool(r == l)),
                    (l, r, TokenKind::Plus) => Err(Error::RunTime(format!(
                        "({} + {}), both should be a number",
                        l, r
                    ))),
                    _ => unreachable!(),
                }
            }
            Expr::Variable(token) => self.environment.borrow().get(token),
            Expr::Assign { name, value } => {
                let value = self.visit_expr(value)?;
                self.environment
                    .borrow_mut()
                    .assign(name.lexeme.to_string(), value.clone())?;
                Ok(value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.visit_expr(left)?;
                if operator.kind == TokenKind::Or {
                    if is_truthy(&left) {
                        return Ok(left);
                    }
                } else if !is_truthy(&left) {
                    return Ok(left);
                }
                self.visit_expr(right)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.visit_expr(callee)?;

                let mut args = vec![];
                for arg in arguments {
                    args.push(self.visit_expr(arg)?);
                }

                match callee {
                    Literal::Callable(c) => {
                        let function = c;
                        if arguments.len() != function.arity() {
                            return Err(Error::RunTime(format!(
                                "Expected {} arguments but got {}.",
                                function.arity(),
                                arguments.len()
                            )));
                        }
                        Ok(function.call(&mut self, arguments))
                    }
                    _ => {
                        return Err(Error::RunTime(
                            "Can only call function and classes".to_string(),
                        ));
                    }
                }
            }
            _ => todo!(),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> LoxResult<'a, ()> {
        match stmt {
            Stmt::Expression(expr) => {
                self.visit_expr(expr).unwrap();
            }
            Stmt::Print(expr) => {
                let value = self.visit_expr(expr).unwrap();
                println!("{}", value);
            }
            Stmt::Var(name, initializer) => {
                let value = self.visit_expr(initializer)?;
                self.environment
                    .borrow_mut()
                    .define(name.lexeme.to_string(), value);
            }
            Stmt::Block(stmts) => {
                let environment =
                    Rc::new(RefCell::new(Environment::new(Rc::clone(&self.environment))));
                self.execute_block(stmts, environment);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.visit_expr(condition)?;
                if is_truthy(&cond) {
                    self.visit_stmt(then_branch)?;
                } else if let Some(else_stmt) = else_branch {
                    self.visit_stmt(else_stmt)?;
                }
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.visit_expr(condition)?) {
                    self.visit_stmt(body)?;
                }
            }
        }
        Ok(())
    }
}

fn is_truthy(literal: &Literal) -> bool {
    match literal {
        Literal::Nil => false,
        Literal::Bool(b) => *b,
        _ => true,
    }
}

#[derive(Clone, PartialEq, Debug)]
struct LoxFunction<'a> {
    declaration: Stmt<'a>,
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        todo!()
    }

    fn call(&self, interpreter: &mut Interpreter, arguments: &[Box<Expr>]) -> Literal {
        let environment = Rc::clone(&interpreter.globals);
        for
        todo!()
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Clock;

impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _arguments: &[Box<Expr>]) -> Literal {
        let time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_millis(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };
        Literal::Number((time / 1000) as f64)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Rc::new(RefCell::new(Environment::new_global()));
        let environment = Rc::clone(&globals);
        globals
            .borrow_mut()
            .define("clock".to_string(), Literal::Callable(Box::new(Clock)));
        Self {
            globals,
            environment,
        }
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for stmt in statements.iter() {
            self.visit_stmt(stmt).unwrap_or_else(|err| {
                eprintln!("{}", err);
            });
        }
    }

    fn execute_block(&mut self, statements: &[Stmt], environment: Rc<RefCell<Environment>>) {
        let mut previous = environment;
        std::mem::swap(&mut self.environment, &mut previous);

        for statement in statements.iter() {
            self.visit_stmt(statement).unwrap();
        }

        std::mem::swap(&mut self.environment, &mut previous);
    }
}
