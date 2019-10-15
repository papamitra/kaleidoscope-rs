use super::ast::Expr;
use super::ast::{Function, Prototype};
use super::token::Token;
use combine::error::ParseError;
use combine::parser::choice::or;
use combine::parser::repeat::chainl1;
pub(crate) use combine::parser::Parser;
use combine::stream::Stream;
use combine::{any, between, choice, many, parser, satisfy_map, token};

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

    choice((number, paren))
}

parser! {
    fn primary[Input]()(Input) -> Expr
        where [Input: Stream<Token=Token>]
    {
        primary_()
    }
}

fn expr<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let lt = token(Token::Kwd('<')).map(|_| |l, r| Expr::Binary('<', Box::new(l), Box::new(r)));
    chainl1(add(), lt)
}

fn add<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    chainl1(
        mul(),
        or(token(Token::Kwd('+')), token(Token::Kwd('-'))).map(|t| match t {
            Token::Kwd(c) => move |l, r| Expr::Binary(c, Box::new(l), Box::new(r)),
            _ => unreachable!(),
        }),
    )
}

fn mul<Input>() -> impl Parser<Input, Output = Expr>
where
    Input: Stream<Token = Token>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    chainl1(
        primary(),
        token(Token::Kwd('*')).map(|_| |l, r| Expr::Binary('*', Box::new(l), Box::new(r))),
    )
}

fn prototype<Input>() -> impl Parser<Input, Output = Prototype>
where
    Input: Stream<Token = Token> + Clone,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    use super::token::Token::*;

    let ident = satisfy_map(|t| match t {
        Ident(id) => Some(id),
        _ => None,
    });

    let args = many(ident.clone());

    (ident, between(token(Kwd('(')), token(Kwd(')')), args)).map(|(id, aa)| Prototype(id, aa))
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
        let tokens = vec![Token::Number(1.0)];
        assert_eq!(
            primary().parse(tokens.as_slice()).map(|x| x.0),
            Ok(Expr::Number(1.0))
        );
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
}
