#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Token {
    Def,
    Extern,
    If,
    Then,
    Else,
    For,
    In,
    Ident(String),
    Number(f64),
    Kwd(char),
    Eof,
}
