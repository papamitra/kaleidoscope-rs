pub(crate) enum Expr {
    Number(f64),
    Variable(String),
    Binary(char, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

struct Prototype(String, Vec<String>);
struct Function(Box<Prototype>, Box<Expr>);
