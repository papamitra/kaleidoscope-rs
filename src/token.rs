#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Def,
    Extern,
    Ident(String),
    Number(f64),
    Kwd(char),
    Eof,
}
