mod ast;
mod lexer;
mod parser;
mod token;
mod toplevel;

fn main() {
    toplevel::main_loop();
}
