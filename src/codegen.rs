use llvm_sys::prelude::*;
use llvm_sys::{analysis, core, LLVMRealPredicate};
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;

use super::ast::{Expr, Function, Prototype};
use super::error::{Error, ErrorKind};

thread_local! {
    pub(crate) static CONTEXT: LLVMContextRef = unsafe {core::LLVMContextCreate()};
    pub(crate) static THE_MODULE: LLVMModuleRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMModuleCreateWithNameInContext(b"my cool jit\0".as_ptr() as *const _, *c))
    };
    static BUILDER: LLVMBuilderRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMCreateBuilderInContext(*c))
    };

    pub(crate) static DOUBLE_TYPE:LLVMTypeRef = unsafe {
        CONTEXT.with(|c|
        core::LLVMDoubleTypeInContext(*c))
    };

    static NAMED_VALUES:RefCell<HashMap<String, LLVMValueRef>> = RefCell::new(HashMap::new());
}

unsafe fn codegen_expr(e: &Expr) -> Result<LLVMValueRef, Error> {
    match e {
        Expr::Number(n) => DOUBLE_TYPE.with(|dt| Ok(core::LLVMConstReal(*dt, *n))),
        Expr::Variable(name) => {
            NAMED_VALUES.with(|named_values| match named_values.borrow().get(name) {
                Some(v) => Ok(v.clone()),
                None => Err(Error::from(ErrorKind::Codegen(format!(
                    "unknown variable name: {}",
                    name
                )))),
            })
        }
        Expr::Binary(op, lhs, rhs) => {
            let lhs_val = codegen_expr(lhs)?;
            let rhs_val = codegen_expr(rhs)?;
            BUILDER.with(|builder| match op {
                '+' => Ok(core::LLVMBuildFAdd(
                    *builder,
                    lhs_val,
                    rhs_val,
                    b"addtmp\0".as_ptr() as *const _,
                )),
                '-' => Ok(core::LLVMBuildFSub(
                    *builder,
                    lhs_val,
                    rhs_val,
                    b"subtmp\0".as_ptr() as *const _,
                )),

                '*' => Ok(core::LLVMBuildFMul(
                    *builder,
                    lhs_val,
                    rhs_val,
                    b"multmp\0".as_ptr() as *const _,
                )),
                '<' => {
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
                }
                _ => Err(Error::from(ErrorKind::Codegen("op '<' failed".to_owned()))),
            })
        }
        Expr::Call(callee, args) => THE_MODULE.with(|the_module| {
            let func = core::LLVMGetNamedFunction(
                *the_module,
                CString::new(callee.clone()).unwrap().as_ptr(),
            );
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
        }),
    }
}

pub(crate) unsafe fn codegen_proto(
    Prototype(name, args): &Prototype,
) -> Result<LLVMValueRef, Error> {
    THE_MODULE.with(|the_module| {
        let func =
            core::LLVMGetNamedFunction(*the_module, CString::new(name.clone()).unwrap().as_ptr());
        let func = if core::LLVMIsNull(func) == 0 {
            DOUBLE_TYPE.with(|double_type| {
                let mut doubles = vec![*double_type; args.len()];
                let ft = core::LLVMFunctionType(
                    *double_type,
                    doubles.as_mut_ptr(),
                    args.len() as u32,
                    0, /* isvararg is false*/
                );
                core::LLVMAddFunction(
                    *the_module,
                    CString::new(name.clone()).unwrap().as_ptr(),
                    ft,
                )
            })
        } else {
            //TODO: If `func` already has a body, reject this.

            //TODO: If `func` took a different number of arguments, reject.
            func
        };

        // Set names for all arguments.
        let pcnt = core::LLVMCountParams(func) as usize;
        for i in 0..pcnt {
            let arg = core::LLVMGetParam(func, i as u32);
            core::LLVMSetValueName2(
                arg,
                CString::new(args[i].clone()).unwrap().as_ptr(),
                args[i].len(),
            );
            NAMED_VALUES
                .with(|named_values| named_values.borrow_mut().insert(args[i].clone(), arg));
        }

        Ok(func)
    })
}

pub(crate) unsafe fn codegen_func(
    the_fpm: LLVMPassManagerRef,
    Function(proto, body): &Function,
) -> Result<LLVMValueRef, Error> {
    NAMED_VALUES.with(|named_values| named_values.borrow_mut().clear());

    let the_function = codegen_proto(proto)?;
    let ret = CONTEXT.with(|context| {
        BUILDER.with(|builder| {
            let bb = core::LLVMAppendBasicBlockInContext(
                *context,
                the_function,
                b"entry\0".as_ptr() as *const _,
            );
            core::LLVMPositionBuilderAtEnd(*builder, bb);

            let ret_val = codegen_expr(body)?;
            let _ = core::LLVMBuildRet(*builder, ret_val);

            //Validate the generated code, checking for consistency.
            analysis::LLVMVerifyFunction(
                the_function,
                analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction,
            );

            core::LLVMRunFunctionPassManager(the_fpm, the_function);

            Ok(the_function)
        })
    });

    if let Err(_) = ret {
        println!("codegen_func errro:{:?}", ret);
        core::LLVMDeleteFunction(the_function);
    }

    ret
}
