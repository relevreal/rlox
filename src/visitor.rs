use crate::expr::Expr;
use crate::stmt::Stmt;

pub trait Visitor<T, G> {
    fn visit_expr(&mut self, expr: &Expr) -> T;
    fn visit_stmt(&mut self, stmt: &Stmt) -> G;
}
