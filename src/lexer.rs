use super::token::Token;
use combine::easy;
use combine::error::{ParseError, StreamError, UnexpectedParse};
use combine::parser::char::{alpha_num, digit, newline, space, spaces};
use combine::parser::choice::or;
use combine::parser::repeat::{chainl1, take_until};
use combine::parser::{EasyParser, Parser};
use combine::stream::{Stream, StreamErrorFor};
use combine::{
    any, between, choice, eof, many, many1, none_of, not_followed_by, parser, satisfy_map,
    skip_many, skip_many1, token,
};

fn number<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(choice((digit(), token('.'))))
        .and_then(|ns: String| {
            ns.parse::<f64>().map_err(|e| {
                <Input::Error as combine::error::ParseError<char, Input::Range, Input::Position>>
                                                         ::StreamError::other(e)
            })
        })
        .map(|n| Token::Number(n))
}

fn ident<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(alpha_num()).map(|s: String| match s.as_ref() {
        "def" => Token::Def,
        "extern" => Token::Extern,
        id => Token::Ident(id.to_string()),
    })
}

fn comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    skip_many1((
        token('#'),
        take_until::<Vec<_>, _, _>(or(newline().map(|_| ()), eof())),
    ))
}

fn lex_<Input>() -> impl Parser<Input, Output = Option<Token>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    skip_many(or(space(), newline())).with(choice((
        number().map(|x| Some(x)),
        ident().map(|x| Some(x)),
        comment().with(lex()),
        eof().map(|_| None),
    )))
}

parser! {
    pub(crate) fn lex[Input]()(Input) -> Option<Token>
        where [Input: Stream<Token=char>]
    {
        lex_()
    }
}

#[cfg(test)]
mod test {
    use super::super::token::Token::*;
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(number().easy_parse("1.0").map(|x| x.0), Ok(Number(1.0)));
    }

    #[test]
    fn test_ident() {
        assert_eq!(
            ident().easy_parse("test").map(|x| x.0),
            Ok(Ident("test".to_owned()))
        );

        assert_eq!(ident().easy_parse("def").map(|x| x.0), Ok(Def));
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment().easy_parse("#hoge").map(|x| x.0), Ok(()));
    }

    #[test]
    fn test_lex() {
        assert_eq!(
            lex()
                .easy_parse(
                    r#"#comment
1.0
"#
                )
                .map(|x| x.0),
            Ok(Some(Number(1.0)))
        );
    }
}
