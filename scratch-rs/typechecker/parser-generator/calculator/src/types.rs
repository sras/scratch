#[derive(Debug)]
pub struct Term(pub i32);

#[derive(Debug)]
pub enum Expr {
    TermExpr(Term),
    BinExpr(Box<Expr>, Box<Expr>)
}

