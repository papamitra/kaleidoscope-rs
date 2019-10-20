use super::lexer;
use super::parser;
use super::token::Token;
use combine::Parser;
use std::io::{stdin, stdout, Write};

pub(crate) fn main_loop() {
    'outer: loop {
        print!("Ready> ");
        stdout().flush();
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        let mut buf = line.as_str();
        let mut tokens = Vec::new();
        loop {
            match lexer::lex().parse(buf) {
                Ok((Some(token), rest)) => {
                    buf = rest;
                    tokens.push(token);
                }
                Ok(_) => break,
                Err(e) => {
                    println!("error: {}", e);
                    continue 'outer;
                }
            }
        }

        let mut ts = tokens.as_slice();

        while ts.len() > 0 {
            match ts[0] {
                Token::Kwd(';') => ts = &ts[1..],
                Token::Def => match parser::definition().parse(ts) {
                    Ok((_, rest)) => {
                        println!("parse definition");
                        ts = rest;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
                Token::Extern => match parser::extern_parser().parse(ts) {
                    Ok((_, rest)) => {
                        println!("parse extern");
                        ts = rest;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
                _ => match parser::toplevel().parse(ts) {
                    Ok((_, rest)) => {
                        println!("parse toplevel");
                        ts = rest;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
            }
        }
    }
}
