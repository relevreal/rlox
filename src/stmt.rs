use crate::expr::Expr;
use crate::token::Token;

#[derive(Clone, PartialEq, Debug)]
pub enum Stmt<'a> {
    Print(Expr<'a>),
    Expression(Expr<'a>),
    Var(Token<'a>, Expr<'a>),
    Block(Vec<Stmt<'a>>),
    If {
        condition: Expr<'a>,
        then_branch: Box<Stmt<'a>>,
        else_branch: Option<Box<Stmt<'a>>>,
    },
    While {
        condition: Box<Expr<'a>>,
        body: Box<Stmt<'a>>,
    },
    Function {
        name: Token<'a>,
        params: Vec<Token<'a>>,
        body: Vec<Stmt<'a>>,
    },
}
