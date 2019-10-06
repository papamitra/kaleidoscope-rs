use super::token::Token;

use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_until, take_while},
    character::complete::{alpha1, alphanumeric0, anychar},
    combinator::map,
    error::{ParseError, VerboseError},
    number::complete::double,
    sequence::{pair, preceded},
    Err, IResult,
};

fn ws<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    let chars = " \t\r\n";

    take_while(move |c| chars.contains(c))(i)
}

fn lex_comment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    println!("lex_comment start: {}", i);
    if let ok @ Ok(_) = take_until("\n")(i) {
        take(1usize)(i)?;
        ok
    } else {
        // reached to EOF
        Ok(("", i))
    }
}

fn lex<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Token, E> {
    use Token::*;

    if i.len() == 0 {
        return Ok((i, EOF));
    }

    preceded(
        ws,
        alt((
            map(pair(alpha1, alphanumeric0), |x: (&str, &str)| {
                Ident(x.0.to_string() + &x.1.to_string())
            }),
            map(double, |v| Number(v)),
            preceded(tag("#"), preceded(lex_comment, lex)),
            map(anychar, |c| Kwd(c)),
        )),
    )(i)
}

#[test]
fn lex_test() {
    use Token::*;
    assert_eq!(lex::<VerboseError<&str>>("1"), Ok(("", Number(1.0))));
    assert_eq!(lex::<VerboseError<&str>>(" #hoge"), Ok(("", EOF)));
}
