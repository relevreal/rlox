use crate::token::{Token, TokenKind};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error<'a> {
    Lexical(LexerError),
    Syntactic(ParserError<'a>),
    RunTime(String),
}

#[derive(Debug, Clone)]
pub struct LexerError {
    pub(crate) line: usize,
    pub(crate) message: String,
}

#[derive(Debug, Clone)]
pub struct ParserError<'a> {
    pub(crate) token: Token<'a>,
    pub(crate) message: String,
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lexical(l) => write!(f, "[line: {}] Lexical error: {}", l.line, l.message),
            Self::Syntactic(p) => {
                if p.token.kind == TokenKind::Eof {
                    write!(
                        f,
                        "[line: {}] Syntactic error: {} at end",
                        p.token.line, p.message
                    )
                } else {
                    write!(
                        f,
                        "[line: {}] Syntactic error: {} at '{}'",
                        p.token.line, p.message, p.token.lexeme
                    )
                }
            }
            Self::RunTime(s) => write!(f, "{}", s),
        }
    }
}

pub type LoxResult<'a, T> = Result<T, Error<'a>>;
