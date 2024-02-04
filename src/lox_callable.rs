use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::token::Literal;
use std::fmt::Debug;

pub trait LoxCallable: Debug + Clone + PartialEq {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, arguments: &[Box<Expr>]) -> Literal;
}
