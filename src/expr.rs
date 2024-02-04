use crate::token::{Literal, Token};

#[derive(Clone, PartialEq, Debug)]
pub enum Expr<'a> {
    Assign {
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Binary {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Call {
        callee: Box<Expr<'a>>,
        paren: Token<'a>,
        arguments: Vec<Box<Expr<'a>>>,
    },
    Get {
        object: Box<Expr<'a>>,
        name: Token<'a>,
    },
    Grouping(Box<Expr<'a>>),
    Literal(Literal),
    Logical {
        left: Box<Expr<'a>>,
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Set {
        object: Box<Expr<'a>>,
        name: Token<'a>,
        value: Box<Expr<'a>>,
    },
    Super {
        keyword: Token<'a>,
        method: Token<'a>,
    },
    This(Token<'a>),
    Unary {
        operator: Token<'a>,
        right: Box<Expr<'a>>,
    },
    Variable(Token<'a>),
}
