use crate::lox_callable::LoxCallable;
use std::fmt;
use std::fmt::{Display, Formatter};

macro_rules! enum_str {
    (
        pub enum $name:ident {
            $( $variant:ident, )*
        }
    ) => {
        #[derive(Copy, Clone, PartialEq, Debug)]
        pub enum $name {
            $( $variant ),*
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    $(
                        $name::$variant => write!(f, stringify!($variant)),
                    )*
                }
            }
        }
    };
}

enum_str! {
    pub enum TokenKind {
        // Single-character tokens
        LeftParen,
        RightParen,
        LeftBrace,
        RightBrace,
        Comma,
        Dot,
        Minus,
        Plus,
        Semicolon,
        Slash,
        Star,

        // One or two character tokens
        Bang,
        BangEqual,
        Equal,
        EqualEqual,
        Greater,
        GreaterEqual,
        Less,
        LessEqual,

        // Literals
        Identifier,
        String,
        Number,

        // Keywords
        And,
        Class,
        Else,
        False,
        Fun,
        For,
        If,
        Nil,
        Or,
        Print,
        Return,
        Super,
        This,
        True,
        Var,
        While,

        Comment,

        Eof,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
    Callable(Box<dyn LoxCallable>),
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(num) => write!(f, "{}", num),
            Self::String(s) => write!(f, "{}", s),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Nil => write!(f, "nil"),
            Self::Callable(c) => write!(f, "function"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub literal: Option<Literal>,
    pub start: usize,
    pub line: usize,
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenKind, lexeme: &'a str, start: usize, line: usize) -> Self {
        Self {
            kind,
            lexeme,
            literal: None,
            start,
            line,
        }
    }

    pub fn new_full(
        kind: TokenKind,
        lexeme: &'a str,
        literal: Literal,
        start: usize,
        line: usize,
    ) -> Self {
        Self {
            kind,
            lexeme,
            literal: Some(literal),
            start,
            line,
        }
    }
}

impl<'a> fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Token({}, {}, {}:{}-{})",
            self.kind,
            self.lexeme,
            self.line,
            self.start,
            self.start + self.lexeme.len() - 1,
        )
    }
}
