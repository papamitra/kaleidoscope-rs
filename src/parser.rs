use super::ast::Expr;
use super::ast::{Function, Prototype};
use super::token::Token;
use combine::error::ParseError;
use combine::parser::choice::or;
use combine::parser::repeat::chainl1;
pub(crate) use combine::parser::Parser;
use combine::stream::Stream;
use combine::{any, attempt, between, choice, many, optional, parser, satisfy_map, sep_by, token};

fn ident<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    satisfy_map(|t| match t {
        Token::Ident(id) => Some(id),
        _ => None,
    })
}

fn args<Input>() -> impl Parser<Input, Output = Vec<Expr>>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_by(expr(), token(Token::Kwd(',')))
}

fn call<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        ident(),
        between(token(Token::Kwd('(')), token(Token::Kwd(')')), args()),
    )
        .map(|(id, aa)| Expr::Call(id, aa))
}

fn primary_<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    use super::token::Token::*;
    let number = satisfy_map(|c| match c {
        Number(n) => Some(Expr::Number(n)),
        _ => None,
    });

    let paren = between(token(Kwd('(')), token(Kwd(')')), expr());

    let variable = ident().map(|id| Expr::Variable(id));

    choice((
        attempt(number),
        attempt(paren),
        attempt(call()),
        attempt(variable),
        attempt(parse_if()),
        attempt(parse_for()),
    ))
}

parser! {
    fn primary[Input]()(Input) -> Expr
        where [Input: Stream<Token=Token>]
    {
        primary_()
    }
}

fn parse_if<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    use super::token::Token::*;

    (token(If), expr(), token(Then), expr(), token(Else), expr())
        .map(|(_, c, _, t, _, e)| Expr::If(Box::new(c), Box::new(t), Box::new(e)))
}

fn parse_for<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    use super::token::Token::*;

    (
        token(For),
        ident(),
        token(Kwd('=')),
        expr(),
        token(Kwd(',')),
        expr(),
        optional((token(Kwd(',')), expr()).map(|(_, e)| e)),
        token(In),
        expr(),
    )
        .map(|(_, id, _, start, _, end, step, _, body)| {
            Expr::For(
                id,
                Box::new(start),
                Box::new(end),
                Box::new(step),
                Box::new(body),
            )
        })
}

fn expr<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let lt = token(Token::Kwd('<')).map(|_| |l, r| Expr::Binary('<', Box::new(l), Box::new(r)));
    or(chainl1(add(), lt), add())
}

fn add<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    or(
        chainl1(
            mul(),
            or(token(Token::Kwd('+')), token(Token::Kwd('-'))).map(|t| match t {
                Token::Kwd(c) => move |l, r| Expr::Binary(c, Box::new(l), Box::new(r)),
                _ => unreachable!(),
            }),
        ),
        mul(),
    )
}

fn mul<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    or(
        chainl1(
            primary(),
            token(Token::Kwd('*')).map(|_| |l, r| Expr::Binary('*', Box::new(l), Box::new(r))),
        ),
        primary(),
    )
}

fn prototype<Input>() -> impl Parser<Input, Output = Prototype>
where
    Input: Stream<Token = Token> + Clone,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    use super::token::Token::*;

    let args = many(ident());

    (ident(), between(token(Kwd('(')), token(Kwd(')')), args)).map(|(id, aa)| Prototype(id, aa))
}

pub(crate) fn definition<Input>() -> impl Parser<Input, Output = Function>
where
    Input: Stream<Token = Token> + Clone,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (token(Token::Def), prototype(), expr()).map(|(_, p, e)| Function(Box::new(p), Box::new(e)))
}

pub(crate) fn toplevel<Input>() -> impl Parser<Input, Output = Function>
where
    Input: Stream<Token = Token> + Clone,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    expr().map(|e| Function(Box::new(Prototype("".to_owned(), vec![])), Box::new(e)))
}

pub(crate) fn extern_parser<Input>() -> impl Parser<Input, Output = Prototype>
where
    Input: Stream<Token = Token> + Clone,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (token(Token::Extern), prototype()).map(|(_, p)| p)
}

#[cfg(test)]
mod test {

    use super::super::token::Token::*;
    use super::*;

    #[test]
    fn test_parser_token() {
        assert_eq!(any().parse(vec![Def].as_slice()).map(|x| x.0), Ok(Def));

        assert_eq!(
            token(Ident("hoge".to_owned()))
                .parse(vec![Ident("hoge".to_owned())].as_slice())
                .map(|x| x.0),
            Ok(Ident("hoge".to_owned()))
        );
    }

    #[test]
    fn test_primary() {
        {
            let tokens = vec![Token::Number(1.0)];
            assert_eq!(
                primary().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Number(1.0))
            );
        }

        {
            let tokens = vec![Token::Ident("y".to_owned())];
            assert_eq!(
                primary().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Variable("y".to_owned()))
            );
        }
    }

    #[test]
    fn test_expr() {
        {
            let tokens = vec![Number(1.0), Kwd('+'), Number(2.0)];
            assert_eq!(
                expr().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Binary(
                    '+',
                    Box::new(Expr::Number(1.0)),
                    Box::new(Expr::Number(2.0))
                ))
            );
        }

        {
            let tokens = vec![Number(1.0), Kwd('+'), Number(2.0), Kwd('*'), Number(3.0)];
            assert_eq!(
                expr().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Binary(
                    '+',
                    Box::new(Expr::Number(1.0)),
                    Box::new(Expr::Binary(
                        '*',
                        Box::new(Expr::Number(2.0)),
                        Box::new(Expr::Number(3.0))
                    ))
                ))
            );
        }

        {
            let tokens = vec![
                Kwd('('),
                Number(1.0),
                Kwd('+'),
                Number(2.0),
                Kwd(')'),
                Kwd('*'),
                Number(3.0),
            ];
            assert_eq!(
                expr().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Binary(
                    '*',
                    Box::new(Expr::Binary(
                        '+',
                        Box::new(Expr::Number(1.0)),
                        Box::new(Expr::Number(2.))
                    )),
                    Box::new(Expr::Number(3.0))
                ))
            );
        }

        {
            let tokens = vec![Ident("y".to_owned())];
            assert_eq!(
                expr().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::Variable("y".to_owned()))
            );
        }
    }

    #[test]
    fn test_prototype() {
        {
            let tokens = vec![Ident("f".to_owned()), Kwd('('), Kwd(')')];
            assert_eq!(
                prototype().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Prototype("f".to_owned(), vec![]))
            );
        }
    }

    fn lex_tokens(s: &str) -> Vec<Token> {
        let mut buf = s;
        let mut tokens = Vec::new();
        loop {
            match super::super::lexer::lex().parse(buf) {
                Ok((Some(token), rest)) => {
                    buf = rest;
                    tokens.push(token);
                }
                Ok(_) => break,
                e => {
                    println!("error: {:?}", e);
                    e.unwrap();
                }
            }
        }

        tokens
    }

    #[test]
    fn test_call() {
        let tokens = lex_tokens("foo(y, 4.0)");
        assert_eq!(
            call().parse(tokens.as_slice()).map(|x| x.0),
            Ok(Expr::Call(
                "foo".to_owned(),
                vec![Expr::Variable("y".to_owned()), Expr::Number(4.0)]
            ))
        );
    }

    #[test]
    fn test_args() {
        let tokens = lex_tokens("y, 4.0");
        assert_eq!(
            args().parse(tokens.as_slice()).map(|x| x.0),
            Ok(vec![Expr::Variable("y".to_owned()), Expr::Number(4.0)])
        );
    }

    #[test]
    fn test_for() {
        {
            let tokens = lex_tokens("for i=1, 3 in 3");
            assert_eq!(
                parse_for().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::For(
                    "i".to_owned(),
                    Box::new(Expr::Number(1.0)),
                    Box::new(Expr::Number(3.0)),
                    Box::new(None),
                    Box::new(Expr::Number(3.0))
                ))
            );
        }

        {
            let tokens = lex_tokens("for i=1, 3,2 in 3");
            assert_eq!(
                parse_for().parse(tokens.as_slice()).map(|x| x.0),
                Ok(Expr::For(
                    "i".to_owned(),
                    Box::new(Expr::Number(1.0)),
                    Box::new(Expr::Number(3.0)),
                    Box::new(Some(Expr::Number(2.0))),
                    Box::new(Expr::Number(3.0))
                ))
            );
        }
    }
}
