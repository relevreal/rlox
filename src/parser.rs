use crate::error::{Error, LoxResult, ParserError};
use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::stmt::Stmt::Print;
use crate::token::{Literal, Token, TokenKind};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        Self { tokens, current: 0 }
    }

    // pub fn parse(&mut self) -> Option<Expr<'a>> {
    //     self.expression().ok()
    // }
    pub fn parse(&mut self) -> Vec<Stmt<'a>> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_eof() {
            statements.push(self.declaration());
        }
        statements
    }

    fn declaration(&mut self) -> Stmt<'a> {
        if self.match_(&[TokenKind::Fun]) {
            self.function("function")
        } else if self.match_(&[TokenKind::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn function(&mut self, kind: &str) -> Stmt<'a> {
        let name = self
            .consume(TokenKind::Identifier, &format!("Expect {} name.", kind))
            .unwrap();
        self.consume(
            TokenKind::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )
        .unwrap();
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            params.push(
                self.consume(TokenKind::Identifier, "Expect parameter name.")
                    .unwrap(),
            );
            while self.match_(&[TokenKind::Comma]) {
                if params.len() >= 255 {
                    eprintln!("Can't have more than 255 parameters.");
                }
                params.push(
                    self.consume(TokenKind::Identifier, "Expect parameter name.")
                        .unwrap(),
                );
            }
        }
        self.consume(TokenKind::RightParen, "Expect ')' after parameters")
            .unwrap();
        let body = self.block();
        Stmt::Function { name, params, body }
    }

    fn var_declaration(&mut self) -> Stmt<'a> {
        let name = self
            .consume(TokenKind::Identifier, "Expect variable name")
            .unwrap();

        let mut initializer = Expr::Literal(Literal::Nil);
        if self.match_(&[TokenKind::Equal]) {
            initializer = self.expression().unwrap();
        }
        self.consume(
            TokenKind::Semicolon,
            "Expect ';' after variable declaration.",
        )
        .unwrap();

        Stmt::Var(name, initializer)
    }

    fn statement(&mut self) -> Stmt<'a> {
        if self.match_(&[TokenKind::For]) {
            return self.for_statement();
        }
        if self.match_(&[TokenKind::If]) {
            return self.if_statement();
        }
        if self.match_(&[TokenKind::Print]) {
            return self.print_statement();
        }
        if self.match_(&[TokenKind::While]) {
            return self.while_statement();
        }
        if self.match_(&[TokenKind::LeftBrace]) {
            return Stmt::Block(self.block());
        }
        self.expression_statement()
    }

    fn for_statement(&mut self) -> Stmt<'a> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'for'.")
            .unwrap();
        let initializer: Option<Stmt>;
        if self.match_(&[TokenKind::Semicolon]) {
            initializer = None;
        } else if self.match_(&[TokenKind::Var]) {
            initializer = Some(self.var_declaration());
        } else {
            initializer = Some(self.expression_statement());
        }

        let mut condition: Option<Expr> = None;
        if !self.check(&TokenKind::Semicolon) {
            condition = Some(self.expression().unwrap());
        }
        self.consume(TokenKind::Semicolon, "Expect ';' after loop condition.")
            .unwrap();

        let mut increment: Option<Expr> = None;
        if !self.check(&TokenKind::RightParen) {
            increment = Some(self.expression().unwrap());
        }
        self.consume(TokenKind::RightParen, "Expect ')' after loop condition.")
            .unwrap();

        let mut body = self.statement();

        if increment.is_some() {
            body = Stmt::Block(vec![body, Stmt::Expression(increment.unwrap())]);
        }

        if condition.is_none() {
            condition = Some(Expr::Literal(Literal::Bool(true)));
        }

        body = Stmt::While {
            condition: Box::new(condition.unwrap()),
            body: Box::new(body),
        };

        if initializer.is_some() {
            body = Stmt::Block(vec![initializer.unwrap(), body]);
        }

        body
    }

    fn while_statement(&mut self) -> Stmt<'a> {
        self.consume(TokenKind::LeftParen, "Ex[ect '(' after 'whiie'")
            .unwrap();
        let condition = self.expression().unwrap();
        self.consume(TokenKind::RightParen, "Ex[ect ')' after condition")
            .unwrap();
        let body = self.statement();
        Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }
    }

    fn if_statement(&mut self) -> Stmt<'a> {
        self.consume(TokenKind::LeftParen, "Expect '(' after 'if'.")
            .unwrap();

        let condition = self.expression().unwrap();

        self.consume(TokenKind::RightParen, "Expect ')' after if condition.")
            .unwrap();

        let then_branch = Box::new(self.statement());
        let mut else_branch = None;
        if self.match_(&[TokenKind::Else]) {
            else_branch = Some(Box::new(self.statement()));
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
        }
    }

    fn block(&mut self) -> Vec<Stmt<'a>> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_eof() {
            statements.push(self.declaration());
        }

        self.consume(TokenKind::RightBrace, "Expect '}' after block.")
            .unwrap();
        statements
    }

    fn print_statement(&mut self) -> Stmt<'a> {
        let value = self.expression().unwrap();
        self.consume(TokenKind::Semicolon, "Expect ';' after value.")
            .unwrap();
        Stmt::Print(value)
    }

    fn expression_statement(&mut self) -> Stmt<'a> {
        let expr = self.expression().unwrap();
        self.consume(TokenKind::Semicolon, "Expect ';' after expression")
            .unwrap();
        Stmt::Expression(expr)
    }

    fn expression(&mut self) -> LoxResult<'a, Expr<'a>> {
        self.assignment()
    }

    fn assignment(&mut self) -> LoxResult<'a, Expr<'a>> {
        let expr = self.or()?;

        if self.match_(&[TokenKind::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    })
                }
                _ => eprintln!("{}. Invalid assignment target.", equals),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut left = self.and()?;
        while self.match_(&[TokenKind::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            left = Expr::Logical {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn and(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut left = self.equality()?;
        while self.match_(&[TokenKind::And]) {
            let operator = self.previous();
            let right = self.equality()?;
            left = Expr::Logical {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn comparison(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut expr = self.term()?;
        while self.match_(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut expr = self.factor()?;
        while self.match_(&[TokenKind::Minus, TokenKind::Plus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut expr = self.unary()?;
        while self.match_(&[TokenKind::Slash, TokenKind::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> LoxResult<'a, Expr<'a>> {
        if self.match_(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    fn call(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut expr = self.primary()?;

        loop {
            if self.match_(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr<'a>) -> LoxResult<'a, Expr<'a>> {
        let mut arguments = vec![];
        if !self.check(&TokenKind::RightParen) {
            arguments.push(Box::new(self.expression()?));
            while self.match_(&[TokenKind::Comma]) {
                arguments.push(Box::new(self.expression()?));
            }
            if arguments.len() >= 255 {
                eprintln!(
                    "{}, Can't have more than 255 arguments.",
                    self.peek().unwrap()
                );
            }
        }

        let paren = self.consume(TokenKind::RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
    }

    fn primary(&mut self) -> LoxResult<'a, Expr<'a>> {
        if self.match_(&[
            TokenKind::False,
            TokenKind::True,
            TokenKind::Nil,
            TokenKind::Number,
            TokenKind::String,
        ]) {
            let expr = Expr::Literal(self.previous().literal.unwrap());
            return Ok(expr);
        }

        if self.match_(&[TokenKind::Identifier]) {
            return Ok(Expr::Variable(self.previous()));
        }

        if self.match_(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expect ')' after expression.")?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }

        Err(Error::Syntactic(ParserError {
            token: self.peek().cloned().unwrap(),
            message: "Expected expression".to_string(),
        }))
    }

    fn consume(&mut self, token_kind: TokenKind, message: &str) -> LoxResult<'a, Token<'a>> {
        if self.check(&token_kind) {
            return Ok(self.advance());
        }

        Err(Error::Syntactic(ParserError {
            token: self.tokens.get(self.current).cloned().unwrap(),
            message: String::from(message),
        }))
    }

    fn previous(&mut self) -> Token<'a> {
        self.tokens.get(self.current - 1).cloned().unwrap()
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek(), Some(t) if t.kind == TokenKind::Eof)
    }

    fn peek(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.current)
    }

    fn equality(&mut self) -> LoxResult<'a, Expr<'a>> {
        let mut expr = self.comparison()?;
        while self.match_(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn match_(&mut self, token_kinds: &[TokenKind]) -> bool {
        for token_kind in token_kinds {
            if self.check(token_kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&mut self, token_kind: &TokenKind) -> bool {
        if self.is_eof() {
            return false;
        }
        matches!(self.peek(), Some(t) if t.kind == *token_kind)
    }

    fn advance(&mut self) -> Token<'a> {
        if !self.is_eof() {
            self.current += 1;
        }
        self.previous()
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_eof() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }

            if let Some(t) = self.peek() {
                match t.kind {
                    TokenKind::Class
                    | TokenKind::Fun
                    | TokenKind::Var
                    | TokenKind::For
                    | TokenKind::If
                    | TokenKind::While
                    | TokenKind::Print
                    | TokenKind::Return => return,
                    _ => (),
                }
            }

            self.advance();
        }
    }
}
