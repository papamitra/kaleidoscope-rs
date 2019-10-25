use super::codegen;
use super::lexer;
use super::parser;
use super::token::Token;
use combine::Parser;
use std::io::{stdin, stdout, Write};

use llvm_sys::{core, execution_engine};

pub(crate) fn main_loop() {
    'outer: loop {
        print!("Ready> ");
        stdout().flush().unwrap();
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
                    Ok((e, rest)) => {
                        println!("parse a function definition.");
                        unsafe {
                            match codegen::codegen_func(&e) {
                                Ok(v) => core::LLVMDumpValue(v),
                                Err(e) => println!("error: {}", e),
                            }
                        };
                        ts = rest;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
                Token::Extern => match parser::extern_parser().parse(ts) {
                    Ok((p, rest)) => {
                        println!("parsed an extern.");
                        unsafe {
                            match codegen::codegen_proto(&p) {
                                Ok(v) => core::LLVMDumpValue(v),
                                Err(e) => println!("error: {}", e),
                            }
                        };
                        ts = rest;
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break;
                    }
                },
                _ => match parser::toplevel().parse(ts) {
                    Ok((e, rest)) => {
                        println!("parse a top-level expr");
                        unsafe {
                            match codegen::codegen_func(&e) {
                                Ok(the_function) => {
                                    core::LLVMDumpValue(the_function);
                                }
                                Err(e) => println!("error: {}", e),
                            }
                        };
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
