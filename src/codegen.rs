use llvm_sys::analysis::*;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMRealPredicate;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::ffi::CString;

use super::ast::{Expr, Function, Prototype};
use super::error::{Error, ErrorKind};

pub(crate) struct Context {
    context: LLVMContextRef,
    pub(crate) the_module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    pub(crate) double_type: LLVMTypeRef,
    named_values: HashMap<String, LLVMValueRef>,
}

impl Context {
    pub(crate) fn new() -> Self {
        let context = unsafe { LLVMContextCreate() };
        let the_module = unsafe {
            LLVMModuleCreateWithNameInContext(b"my cool jit\0".as_ptr() as *const _, context)
        };
        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        let double_type = unsafe { LLVMDoubleTypeInContext(context) };
        let named_values = HashMap::new();

        Context {
            context,
            the_module,
            builder,
            double_type,
            named_values,
        }
    }
}

unsafe fn codegen_expr(c: &mut Context, e: &Expr) -> Result<LLVMValueRef, Error> {
    match e {
        Expr::Number(n) => Ok(LLVMConstReal(c.double_type, *n)),
        Expr::Variable(name) => match c.named_values.get(name) {
            Some(v) => Ok(v.clone()),
            None => Err(Error::from(ErrorKind::Codegen(format!(
                "unknown variable name: {}",
                name
            )))),
        },
        Expr::Binary(op, lhs, rhs) => {
            let lhs_val = codegen_expr(c, lhs)?;
            let rhs_val = codegen_expr(c, rhs)?;
            match op {
                '+' => Ok(LLVMBuildFAdd(
                    c.builder,
                    lhs_val,
                    rhs_val,
                    b"addtmp\0".as_ptr() as *const _,
                )),
                '-' => Ok(LLVMBuildFSub(
                    c.builder,
                    lhs_val,
                    rhs_val,
                    b"subtmp\0".as_ptr() as *const _,
                )),

                '*' => Ok(LLVMBuildFMul(
                    c.builder,
                    lhs_val,
                    rhs_val,
                    b"multmp\0".as_ptr() as *const _,
                )),
                '<' => {
                    let i = LLVMBuildFCmp(
                        c.builder,
                        LLVMRealPredicate::LLVMRealULT,
                        lhs_val,
                        rhs_val,
                        b"cmptmp\0".as_ptr() as *const _,
                    );
                    Ok(LLVMBuildUIToFP(
                        c.builder,
                        i,
                        c.double_type,
                        b"booltmp\0".as_ptr() as *const _,
                    ))
                }
                _ => Err(Error::from(ErrorKind::Codegen("op '<' failed".to_owned()))),
            }
        }
        Expr::Call(callee, args) => {
            let func =
                LLVMGetNamedFunction(c.the_module, CString::new(callee.clone()).unwrap().as_ptr());
            if LLVMIsNull(func) == 1 {
                return Err(Error::from(ErrorKind::Codegen(format!(
                    "unknown function: {}",
                    callee
                ))));
            }

            let param_cnt = LLVMCountParams(func);
            if param_cnt as usize != args.len() {
                return Err(Error::from(ErrorKind::Codegen(format!(
                    "incorrect # arguments passed",
                ))));
            }

            let mut args = args
                .into_iter()
                .map(|e| codegen_expr(c, e))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(LLVMBuildCall(
                c.builder,
                func,
                args.as_mut_ptr(),
                param_cnt,
                b"calltmp\0".as_ptr() as *const _,
            ))
        }
        _ => unimplemented!(),
    }
}

pub(crate) unsafe fn codegen_proto(
    c: &mut Context,
    Prototype(name, args): &Prototype,
) -> Result<LLVMValueRef, Error> {
    let func = LLVMGetNamedFunction(c.the_module, CString::new(name.clone()).unwrap().as_ptr());
    let func = if LLVMIsNull(func) == 0 {
        let mut doubles = vec![c.double_type; args.len()];
        let ft = LLVMFunctionType(
            c.double_type,
            doubles.as_mut_ptr(),
            args.len() as u32,
            0, /* isvararg is false*/
        );
        LLVMAddFunction(
            c.the_module,
            CString::new(name.clone()).unwrap().as_ptr(),
            ft,
        )
    } else {
        //TODO: If `func` already has a body, reject this.

        //TODO: If `func` took a different number of arguments, reject.
        func
    };

    // Set names for all arguments.
    let pcnt = LLVMCountParams(func) as usize;
    for i in 0..pcnt {
        let arg = LLVMGetParam(func, i as u32);
        LLVMSetValueName2(
            arg,
            CString::new(args[i].clone()).unwrap().as_ptr(),
            args[i].len(),
        );
        c.named_values.insert(args[i].clone(), arg);
    }

    Ok(func)
}

pub(crate) unsafe fn codegen_func(
    c: &mut Context,
    the_fpm: LLVMPassManagerRef,
    Function(proto, body): &Function,
) -> Result<LLVMValueRef, Error> {
    c.named_values.clear();

    let the_function = codegen_proto(c, proto)?;
    let ret = {
        let bb =
            LLVMAppendBasicBlockInContext(c.context, the_function, b"entry\0".as_ptr() as *const _);
        LLVMPositionBuilderAtEnd(c.builder, bb);

        let ret_val = codegen_expr(c, body)?;
        let _ = LLVMBuildRet(c.builder, ret_val);

        //Validate the generated code, checking for consistency.
        LLVMVerifyFunction(
            the_function,
            LLVMVerifierFailureAction::LLVMAbortProcessAction,
        );

        LLVMRunFunctionPassManager(the_fpm, the_function);

        Ok(the_function)
    };

    if let Err(_) = ret {
        println!("codegen_func errro:{:?}", ret);
        LLVMDeleteFunction(the_function);
    }

    ret
}
