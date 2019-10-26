mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod token;
mod toplevel;

use llvm_sys::transforms::{instcombine, scalar};
use llvm_sys::{core, execution_engine, target, target_machine};
use std::ffi::CString;
use std::mem::{transmute, MaybeUninit};
use std::ptr::null_mut;

fn main() {
    let mut the_execution_engine =
        MaybeUninit::<execution_engine::LLVMExecutionEngineRef>::uninit();

    unsafe {
        // robust code should check that these calls complete successfully
        // each of these calls is necessary to setup an execution engine which compiles to native
        // code
        execution_engine::LLVMLinkInMCJIT();
        target::LLVM_InitializeNativeTarget();
        target::LLVM_InitializeNativeAsmPrinter();
        target::LLVM_InitializeNativeAsmParser();

        codegen::THE_MODULE.with(|the_module| {
            execution_engine::LLVMCreateExecutionEngineForModule(
                the_execution_engine.as_mut_ptr(),
                *the_module,
                null_mut::<*mut ::libc::c_char>(),
            );

            let the_execution_engine =
                transmute::<_, execution_engine::LLVMExecutionEngineRef>(the_execution_engine);

            // for debug
            //            let target_machine =
            //                execution_engine::LLVMGetExecutionEngineTargetMachine(the_execution_engine);
            // let triplet = target_machine::LLVMGetTargetMachineTriple(target_machine);
            // println!("triple: {}", CString::from_raw(triplet).to_str().unwrap());

            let data_layout =
                execution_engine::LLVMGetExecutionEngineTargetData(the_execution_engine);

            target::LLVMSetModuleDataLayout(*the_module, data_layout);

            let the_fpm = core::LLVMCreateFunctionPassManagerForModule(*the_module);

            instcombine::LLVMAddInstructionCombiningPass(the_fpm);

            scalar::LLVMAddReassociatePass(the_fpm);

            scalar::LLVMAddGVNPass(the_fpm);

            scalar::LLVMAddCFGSimplificationPass(the_fpm);

            core::LLVMInitializeFunctionPassManager(the_fpm);

            toplevel::main_loop(the_fpm, the_execution_engine);
        });
    }
}
