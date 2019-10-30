use super::codegen;
use super::lexer;
use super::parser;
use super::token::Token;
use combine::Parser;
use std::io::{stdin, stdout, Write};
use std::ptr::null_mut;

use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::prelude::*;

pub(crate) unsafe fn main_loop(
    c: &mut codegen::Context,
    the_fpm: LLVMPassManagerRef,
    the_execution_engine: LLVMExecutionEngineRef,
) {
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
                            match codegen::codegen_func(c, the_fpm, &e) {
                                Ok(v) => LLVMDumpValue(v),
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
                            match codegen::codegen_proto(c, &p) {
                                Ok(v) => LLVMDumpValue(v),
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
                            match codegen::codegen_func(c, the_fpm, &e) {
                                Ok(the_function) => {
                                    LLVMDumpValue(the_function);

                                    let result = LLVMRunFunction(
                                        the_execution_engine,
                                        the_function as *mut _,
                                        0,
                                        null_mut(),
                                    );
                                    println!("the_function: {:?}", the_function);
                                    println!("result: {:?}", result);

                                    println!(
                                        "Evaluated to {}",
                                        LLVMGenericValueToFloat(c.double_type, result) as f64
                                    );
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
