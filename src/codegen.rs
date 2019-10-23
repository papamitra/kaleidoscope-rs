use llvm_sys::prelude::*;
use llvm_sys::{core, execution_engine, target, LLVMRealPredicate};
use std::collections::HashMap;
use std::convert::TryInto;

use super::ast::{Expr, Prototype};
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

    static NAMED_VALUES:HashMap<String, LLVMValueRef> = HashMap::new();
}

fn codegen_expr(e: &Expr) -> Result<LLVMValueRef, Error> {
    match e {
        Expr::Number(n) => unsafe { DOUBLE_TYPE.with(|dt| Ok(core::LLVMConstReal(*dt, *n))) },
        Expr::Variable(name) => NAMED_VALUES.with(|named_values| match named_values.get(name) {
            Some(v) => Ok(v.clone()),
            None => Err(Error::from(ErrorKind::Codegen(format!(
                "unknown variable name: {}",
                name
            )))),
        }),
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
        Expr::Call(callee, args) => unsafe {
            THE_MODULE.with(|the_module| {
                let func = core::LLVMGetNamedFunction(*the_module, callee.as_ptr() as *const _);
                if core::LLVMIsNull(func) == 1 {
                    return Err(Error::from(ErrorKind::Codegen(format!(
                        "unknown function: {}",
                        callee
                    ))));
                }

                let param_cnt = core::LLVMCountParams(func);
                if param_cnt as usize != args.len() {
                    return Err(Error::from(ErrorKind::Codegen(format!(
                        "incorrect # arguments passed",
                    ))));
                }

                let mut args = args
                    .into_iter()
                    .map(|e| codegen_expr(e))
                    .collect::<Result<Vec<_>, _>>()?;

                BUILDER.with(|builder| {
                    Ok(core::LLVMBuildCall(
                        *builder,
                        func,
                        args.as_mut_ptr(),
                        param_cnt,
                        b"calltmp\0".as_ptr() as *const _,
                    ))
                })
            })
        },
        _ => unimplemented!(),
    }
}

unsafe fn codegen_proto(Prototype(name, args): &Prototype) -> Result<LLVMValueRef, Error> {
    THE_MODULE.with(|the_module| {
        let func = core::LLVMGetNamedFunction(*the_module, name.as_ptr() as *const _);
        let func = if core::LLVMIsNull(func) == 1 {
            DOUBLE_TYPE.with(|double_type| {
                let mut doubles = vec![double_type.clone(); args.len()];
                let ft = core::LLVMFunctionType(
                    *double_type,
                    doubles.as_mut_ptr(),
                    args.len().try_into().unwrap(),
                    0, /* isvararg is false*/
                );
                core::LLVMAddFunction(*the_module, name.as_ptr() as *const _, ft)
            })
        } else {
            //TODO: If `func` already has a body, reject this.

            //TODO: If `func` took a different number of arguments, reject.

            func
        };

        //TODO: Set names for all arguments.

        Ok(func)
    })
}
