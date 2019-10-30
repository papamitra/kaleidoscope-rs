mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod token;
mod toplevel;

use llvm_sys::core::*;
use llvm_sys::execution_engine::*;
use llvm_sys::target::*;
use llvm_sys::transforms::{instcombine, scalar};
use std::ffi::CString;
use std::mem::{transmute, MaybeUninit};
use std::ptr::null_mut;

fn main() {
    let mut the_execution_engine = MaybeUninit::<LLVMExecutionEngineRef>::uninit();

    unsafe {
        // robust code should check that these calls complete successfully
        // each of these calls is necessary to setup an execution engine which compiles to native
        // code
        LLVMLinkInMCJIT();
        LLVM_InitializeNativeTarget();
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();

        let mut c = codegen::Context::new();

        LLVMCreateExecutionEngineForModule(
            the_execution_engine.as_mut_ptr(),
            c.the_module,
            null_mut::<*mut ::libc::c_char>(),
        );

        let the_execution_engine = transmute::<_, LLVMExecutionEngineRef>(the_execution_engine);

        // for debug
        //            let target_machine =
        //                execution_engine::LLVMGetExecutionEngineTargetMachine(the_execution_engine);
        // let triplet = target_machine::LLVMGetTargetMachineTriple(target_machine);
        // println!("triple: {}", CString::from_raw(triplet).to_str().unwrap());

        let data_layout = LLVMGetExecutionEngineTargetData(the_execution_engine);

        LLVMSetModuleDataLayout(c.the_module, data_layout);

        let the_fpm = LLVMCreateFunctionPassManagerForModule(c.the_module);

        instcombine::LLVMAddInstructionCombiningPass(the_fpm);

        scalar::LLVMAddReassociatePass(the_fpm);

        scalar::LLVMAddGVNPass(the_fpm);

        scalar::LLVMAddCFGSimplificationPass(the_fpm);

        LLVMInitializeFunctionPassManager(the_fpm);

        toplevel::main_loop(&mut c, the_fpm, the_execution_engine);
    }
}
