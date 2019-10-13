use super::token::Token;
use combine::easy;
use combine::error::{ParseError, UnexpectedParse};
use combine::parser::char::{digit, spaces};
use combine::parser::choice::or;
use combine::parser::repeat::chainl1;
use combine::parser::Parser;
use combine::stream::{Stream, StreamErrorFor};
use combine::{any, between, choice, many1, parser, satisfy_map, token};

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

#[cfg(test)]
mod test {
    #[test]
    fn test_number() {}
}
