use crate::expr::Expr;
use crate::stmt::Stmt;
use crate::visitor::Visitor;

pub struct AstPrinter;
impl Visitor<String, ()> for AstPrinter {
    fn visit_expr(&mut self, e: &Expr) -> String {
        match e {
            Expr::Binary {
                ref left,
                operator,
                ref right,
            } => self.parenthesize(operator.lexeme, &[left, right]),
            Expr::Grouping(ref expr) => self.parenthesize("group", &[expr]),
            Expr::Literal(literal) => literal.to_string(),
            Expr::Unary {
                operator,
                ref right,
            } => self.parenthesize(operator.lexeme, &[right]),
            _ => todo!(),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        panic!()
    }
}

impl AstPrinter {
    pub fn print(&mut self, e: &Expr) -> String {
        self.visit_expr(e)
    }

    fn parenthesize(&mut self, name: &str, exprs: &[&Expr]) -> String {
        let mut s = String::from("(");
        s.push_str(name);
        for expr in exprs {
            let expr_str = self.visit_expr(expr);
            s.push(' ');
            s.push_str(&expr_str);
        }
        s.push(')');
        s
    }
}

#[cfg(test)]
mod tests {
    use crate::ast_printer::AstPrinter;
    use crate::expr::Expr;
    use crate::token::{Literal, Token, TokenKind};

    #[test]
    fn it_works() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenKind::Minus, "-", 0, 0),
                right: Box::new(Expr::Literal(Literal::Number(123.0))),
            }),
            operator: Token::new(TokenKind::Star, "*", 0, 0),
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        };
        let ast_str = AstPrinter.print(&expr);
        assert_eq!("(* (- 123) (group 45.67))", ast_str);
    }
}
