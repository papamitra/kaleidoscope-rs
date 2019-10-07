use super::ast::Expr;
use super::token::Token;

use combine::error::{Consumed, ParseError};
use combine::parser::function::parser;
use combine::parser::item::any;
use combine::stream::state::State;
use combine::stream::{buffered, IteratorStream, ReadStream};
use combine::{Parser, Stream};

fn number<Input>() -> impl Parser<Input = Input, Output = f64>
where
    Input: Stream<Item = Token>,
    Input::Error: ParseError<Input::Item, Input::Range, Input::Position>,
{
    parser(|input: &mut Input| {
        let (c, consumed) = any().parse_lazy(input).into_result()?;
        match c {
            Token::Number(n) => Ok((n, consumed)),
            _ => Err(Consumed::Empty(
                Input::Error::empty(input.position()).into(),
            )),
        }
    })
}

#[test]
fn test() {
    let stream = buffered::Stream::new(
        State::new(IteratorStream::new(vec![Token::Number(1.0)].into_iter())),
        1,
    );
    assert_eq!(number().parse(stream).map(|t| t.0), Ok(1.0f64));
}
