mod ast;
mod codegen;
mod lexer;
mod parser;
mod token;
mod toplevel;
mod error;

fn main() {
    toplevel::main_loop();
}
