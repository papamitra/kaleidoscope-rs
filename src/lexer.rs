use super::token::Token;

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{alpha1, alphanumeric0},
    combinator::map,
    error::ParseError,
    number::complete::double,
    sequence::{pair, preceded},
    Err, IResult,
};

fn ws<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

fn lex<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token, E> {
    use Token::*;

    alt((
        preceded(ws, lex),
        map(pair(alpha1, alphanumeric0), |x: (&str, &str)| {
            Ident(x.0.to_string() + &x.1.to_string())
        }),
        map(double, |v| Number(v)),
    ))(i)
}
