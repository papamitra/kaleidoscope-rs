use llvm_sys::prelude::*;
use llvm_sys::{core, execution_engine, target, LLVMRealPredicate};

use super::ast::Expr;
use super::error::{Error, ErrorKind};

thread_local! {
    static CONTEXT: LLVMContextRef = unsafe {core::LLVMContextCreate()};
    static THE_MODULE: LLVMModuleRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMModuleCreateWithNameInContext("my cool jit".as_ptr() as *const i8, *c))
    };
    static BUILDER: LLVMBuilderRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMCreateBuilderInContext(*c))
    };

    static DOUBLE_TYPE:LLVMTypeRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMDoubleTypeInContext(*c))
    };
}

fn codegen_expr(e: &Expr) -> Result<LLVMValueRef, Error> {
    match e {
        Expr::Number(n) => Ok(unsafe { DOUBLE_TYPE.with(|dt| core::LLVMConstReal(*dt, *n)) }),
        Expr::Binary(op, lhs, rhs) => {
            let lhs_val = codegen_expr(lhs)?;
            let rhs_val = codegen_expr(rhs)?;
            BUILDER.with(|builder| match op {
                '+' => unsafe {
                    Ok(core::LLVMBuildAdd(
                        *builder,
                        lhs_val,
                        rhs_val,
                        b"addtmp\0".as_ptr() as *const _,
                    ))
                },
                '-' => unsafe {
                    Ok(core::LLVMBuildSub(
                        *builder,
                        lhs_val,
                        rhs_val,
                        b"subtmp\0".as_ptr() as *const _,
                    ))
                },
                '*' => unsafe {
                    Ok(core::LLVMBuildAdd(
                        *builder,
                        lhs_val,
                        rhs_val,
                        b"multmp\0".as_ptr() as *const _,
                    ))
                },
                '<' => unsafe {
                    let i = core::LLVMBuildFCmp(
                        *builder,
                        LLVMRealPredicate::LLVMRealULT,
                        lhs_val,
                        rhs_val,
                        b"cmptmp\0".as_ptr() as *const _,
                    );
                    Ok(DOUBLE_TYPE.with(|dt| {
                        core::LLVMBuildUIToFP(*builder, i, *dt, b"booltmp\0".as_ptr() as *const _)
                    }))
                },
                _ => Err(Error::from(ErrorKind::Codegen("op '<' failed".to_owned()))),
            })
        }
        _ => unimplemented!(),
    }
}
