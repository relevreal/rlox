use crate::error::{Error, LexerError, LoxResult};
use crate::token::{Literal, Token, TokenKind};
use std::hint::unreachable_unchecked;
use std::iter::{once, Peekable};
use std::str::Chars;
// use unicode_segmentation::UnicodeSegmentation;

pub const EOF_CHAR: char = '\0';
pub const EOF_STR: &str = "\0";

fn keywords(identifier: &str) -> Option<TokenKind> {
    match identifier {
        "and" => Some(TokenKind::And),
        "class" => Some(TokenKind::Class),
        "else" => Some(TokenKind::Else),
        "false" => Some(TokenKind::False),
        "for" => Some(TokenKind::For),
        "fun" => Some(TokenKind::Fun),
        "if" => Some(TokenKind::If),
        "nil" => Some(TokenKind::Nil),
        "or" => Some(TokenKind::Or),
        "print" => Some(TokenKind::Print),
        "return" => Some(TokenKind::Return),
        "super" => Some(TokenKind::Super),
        "this" => Some(TokenKind::This),
        "true" => Some(TokenKind::True),
        "var" => Some(TokenKind::Var),
        "while" => Some(TokenKind::While),
        _ => None,
    }
}

struct StringData<'a> {
    lexeme: &'a str,
    literal: &'a str,
    start: usize,
}

struct NumberData<'a> {
    data: f64,
    literal: &'a str,
    start: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    length: usize,
    cursor: usize,
    start: usize,
    line: usize,
    saw_eof: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            length: source.len(),
            cursor: 0,
            start: 0,
            line: 0,
            saw_eof: false,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.chars.clone().next().is_none()
    }

    pub fn advance_char(&mut self) -> Option<char> {
        self.cursor += 1;
        self.chars.next()
    }

    fn peek_first(&mut self) -> char {
        self.chars.peek().copied().unwrap_or(EOF_CHAR)
    }

    fn peek_second(&self) -> char {
        let mut chars = self.chars.clone();
        chars.next();
        chars.next().unwrap_or(EOF_CHAR)
    }

    fn advance_if(&mut self, char: char) -> bool {
        match self.chars.peek() {
            Some(c) if *c == char => {
                self.advance_char();
                true
            }
            _ => false,
        }
    }

    fn eat_while(&mut self, mut predicate: impl FnMut(char) -> bool) {
        while predicate(self.peek_first()) && !self.is_eof() {
            self.advance_char();
        }
    }

    fn string(&mut self) -> StringData<'a> {
        // Len without quotes
        let mut offset: usize = 1;
        while self.peek_first() != '"' && !self.is_eof() {
            if self.peek_first() == '\n' {
                self.line += 1
            }
            self.advance_char();
            offset += 1;
        }
        // The closing "
        self.advance_char();
        offset += 1;

        // Add check if eof without closing the string
        let lexeme = &self.source[self.start..(self.start + offset)];
        let literal = &lexeme[1..(lexeme.len() - 1)];

        StringData {
            lexeme,
            literal,
            start: self.start,
        }
    }

    fn number(&mut self, first_digit: char) -> NumberData<'a> {
        let mut offset: usize = 1;
        let chars = once(first_digit).chain(self.chars.clone());
        while self.peek_first().is_ascii_digit() {
            offset += 1;
            self.advance_char();
        }
        // Look for fractional part
        let mut is_float = false;
        if self.peek_first() == '.' && self.peek_second().is_ascii_digit() {
            is_float = true;
            // Consume the '.'
            self.advance_char();
            offset += 1;

            while self.peek_first().is_ascii_digit() {
                offset += 1;
                self.advance_char();
            }
        }

        let mut str_num: String = chars.take(offset).collect();
        // Need to handle integer case
        if !is_float {
            str_num.push_str(".0");
        }

        let num: f64 = str_num.parse().unwrap();
        let literal = &self.source[self.start..(self.start + offset)];
        NumberData {
            data: num,
            literal,
            start: self.start,
        }
    }

    fn identifier(&mut self) -> StringData<'a> {
        // Take into account already consumed byte
        let start = self.cursor - 1;
        let mut offset: usize = 1;
        while self.peek_first().is_alphanumeric() {
            self.advance_char();
            offset += 1;
        }
        let identifier = &self.source[start..(start + offset)];
        StringData {
            lexeme: identifier,
            literal: identifier,
            start,
        }
    }

    fn new_token(&self, token_kind: TokenKind) -> Token<'a> {
        Token::new(
            token_kind,
            &self.source[self.start..self.cursor],
            self.start,
            self.line,
        )
    }

    pub fn advance_token(&mut self) -> LoxResult<'a, Token<'a>> {
        let mut first_char: Option<char>;
        loop {
            first_char = self.advance_char();
            let c = match first_char {
                Some(c) => c,
                None => {
                    self.saw_eof = true;
                    return Ok(Token::new(
                        TokenKind::Eof,
                        EOF_STR,
                        self.length,
                        self.line + 1,
                    ));
                }
            };
            match c {
                ' ' | '\r' | '\t' => continue,

                '\n' => {
                    self.line += 1;
                    continue;
                }
                _ => break,
            }
        }
        let first_char = first_char.unwrap();
        self.start = self.cursor - 1;

        let token = match first_char {
            '(' => self.new_token(TokenKind::LeftParen),
            ')' => self.new_token(TokenKind::RightParen),
            '{' => self.new_token(TokenKind::LeftBrace),
            '}' => self.new_token(TokenKind::RightBrace),
            ',' => self.new_token(TokenKind::Comma),
            '.' => self.new_token(TokenKind::Dot),
            '-' => self.new_token(TokenKind::Minus),
            '+' => self.new_token(TokenKind::Plus),
            ';' => self.new_token(TokenKind::Semicolon),
            '*' => self.new_token(TokenKind::Star),

            '!' => {
                if self.advance_if('=') {
                    self.new_token(TokenKind::BangEqual)
                } else {
                    self.new_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.advance_if('=') {
                    self.new_token(TokenKind::EqualEqual)
                } else {
                    self.new_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.advance_if('=') {
                    self.new_token(TokenKind::LessEqual)
                } else {
                    self.new_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.advance_if('=') {
                    self.new_token(TokenKind::GreaterEqual)
                } else {
                    self.new_token(TokenKind::Greater)
                }
            }

            '/' => {
                if self.advance_if('/') {
                    // Comment goes until the end of the line
                    self.eat_while(|c| c != '\n');
                    self.new_token(TokenKind::Comment)
                } else {
                    self.new_token(TokenKind::Slash)
                }
            }

            '"' => {
                let line = self.line;
                let s = self.string();
                Token::new_full(
                    TokenKind::String,
                    s.lexeme,
                    Literal::String(s.literal.to_string()),
                    s.start,
                    line,
                )
            }

            c if c.is_ascii_digit() => {
                let line = self.line;
                let num = self.number(c);
                Token::new_full(
                    TokenKind::Number,
                    num.literal,
                    Literal::Number(num.data),
                    num.start,
                    line,
                )
            }

            c if is_alpha(c) => {
                let line = self.line;
                let identifier = self.identifier();
                match keywords(identifier.lexeme) {
                    Some(TokenKind::True) => Token::new_full(
                        TokenKind::True,
                        "true",
                        Literal::Bool(true),
                        identifier.start,
                        line,
                    ),
                    Some(TokenKind::False) => Token::new_full(
                        TokenKind::True,
                        "false",
                        Literal::Bool(false),
                        identifier.start,
                        line,
                    ),
                    Some(TokenKind::Nil) => Token::new_full(
                        TokenKind::True,
                        "nil",
                        Literal::Nil,
                        identifier.start,
                        line,
                    ),
                    Some(keyword) => Token::new_full(
                        keyword,
                        identifier.lexeme,
                        Literal::String(identifier.literal.to_string()),
                        identifier.start,
                        line,
                    ),
                    None => Token::new_full(
                        TokenKind::Identifier,
                        identifier.lexeme,
                        Literal::String(identifier.literal.to_string()),
                        identifier.start,
                        line,
                    ),
                }
            }

            c => {
                return Err(Error::Lexical(LexerError {
                    line: self.line,
                    message: format!("Unexpected char: {}", c),
                }))
            }
        };
        Ok(token)
    }

    pub fn tokenize(&mut self) -> LoxResult<'a, Vec<Token<'a>>> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let token = self.advance_token()?;
            if token.kind == TokenKind::Eof {
                tokens.push(token);
                break;
            }
            if token.kind == TokenKind::Comment {
                continue;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

// impl<'a> Iterator for Lexer<'a> {
//     type Item = LoxResult<Token<'a>>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.saw_eof {
//             return None;
//         }
//         let token = self.advance_token();
//         Some(token)
//     }
// }

fn is_alpha(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_ascii_uppercase() || c == '_'
}
