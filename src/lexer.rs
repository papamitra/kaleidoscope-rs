use super::token::Token;
use combine::easy;
use combine::error::{ParseError, UnexpectedParse};
use combine::parser::char::{alpha_num, digit, newline, spaces};
use combine::parser::choice::or;
use combine::parser::repeat::chainl1;
use combine::parser::{EasyParser, Parser};
use combine::stream::{Stream, StreamErrorFor};
use combine::{any, between, choice, eof, many, many1, parser, satisfy_map, skip_many1, token};

fn number<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char, Error = easy::ParseError<Input>>,
    Input::Range: PartialEq,
    Input::Error: ParseError<
        Input::Token,
        Input::Range,
        Input::Position,
        StreamError = easy::Error<Input::Token, Input::Range>,
    >,
{
    spaces()
        .with(many1(choice((digit(), token('.')))))
        .and_then(|ns: String| {
            ns.parse::<f64>()
                .map_err(|e| easy::Error::Expected(easy::Info::Static("float")))
        })
        .map(|n| Token::Number(n))
}

fn ident<Input>() -> impl Parser<Input, Output = Token>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(many1(alpha_num()))
        .map(|s: String| match s.as_ref() {
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
    skip_many1(spaces().with(between(
        token('#'),
        or(newline().map(|_| ()), eof()),
        many::<Vec<_>, _, _>(any()),
    )))
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
    }

    #[test]
    fn test_comment() {
        assert_eq!(comment().easy_parse("   #hoge").map(|x| x.0), Ok(()));
    }
}
