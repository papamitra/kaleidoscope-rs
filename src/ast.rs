#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Number(f64),
    Variable(String),
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug, PartialEq)]
pub(crate) struct Prototype(pub(crate) String, pub(crate) Vec<String>);

#[derive(Debug)]
pub(crate) struct Function(pub(crate) Box<Prototype>, pub(crate) Box<Expr>);
