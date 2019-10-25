mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod token;
mod toplevel;

use llvm_sys::execution_engine;
use std::mem::MaybeUninit;
use std::ptr::null_mut;

fn main() {
    let mut eengine_ref = MaybeUninit::<execution_engine::LLVMExecutionEngineRef>::uninit();

    unsafe {
        codegen::THE_MODULE.with(|the_module| {
            execution_engine::LLVMCreateExecutionEngineForModule(
                eengine_ref.as_mut_ptr(),
                *the_module,
                null_mut::<*mut ::libc::c_char>(),
            )
        });
    }
    toplevel::main_loop();
}
