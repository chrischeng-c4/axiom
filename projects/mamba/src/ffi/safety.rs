/// FFI safety utilities for panic-safe wrappers and Result mapping (#267, #269).

/// Convention for how a Rust function signals errors to C (#267).
#[derive(Debug, Clone, PartialEq)]
pub enum ResultConvention {
    /// Returns an error code: 0 = success, non-zero = error
    ErrorCode,
    /// Returns a nullable pointer: null = error
    NullablePointer,
    /// Sets a thread-local error flag after call
    ThreadLocalFlag,
}

/// Descriptor for generating a safe FFI wrapper (#267, #269).
#[derive(Debug, Clone)]
pub struct SafeWrapper {
    pub fn_name: String,
    pub params: Vec<(String, String)>,
    pub return_type: String,
    pub result_convention: ResultConvention,
}

/// Generate a panic-safe `catch_unwind` wrapper as Rust source code (#269).
///
/// Wraps the inner function call in `std::panic::catch_unwind` to prevent
/// Rust panics from unwinding across the FFI boundary (which is UB).
pub fn generate_panic_wrapper(wrapper: &SafeWrapper) -> String {
    let mut out = String::new();

    let param_list: String = wrapper.params.iter()
        .map(|(name, ty)| format!("{name}: {ty}"))
        .collect::<Vec<_>>()
        .join(", ");

    let arg_list: String = wrapper.params.iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let safe_name = format!("{}_safe", wrapper.fn_name);

    out.push_str(&format!(
        "#[no_mangle]\npub extern \"C\" fn {safe_name}({param_list}) -> {} {{\n",
        error_return_type(&wrapper.return_type, &wrapper.result_convention),
    ));

    out.push_str("    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {\n");
    out.push_str(&format!("        {}({arg_list})\n", wrapper.fn_name));
    out.push_str("    })) {\n");

    match wrapper.result_convention {
        ResultConvention::ErrorCode => {
            out.push_str("        Ok(Ok(_)) => 0,\n");
            out.push_str("        Ok(Err(_)) => 1,  // Rust error\n");
            out.push_str("        Err(_) => 2,       // Rust panic\n");
        }
        ResultConvention::NullablePointer => {
            out.push_str("        Ok(Ok(val)) => val,\n");
            out.push_str("        Ok(Err(_)) => std::ptr::null_mut(),\n");
            out.push_str("        Err(_) => std::ptr::null_mut(),\n");
        }
        ResultConvention::ThreadLocalFlag => {
            out.push_str("        Ok(Ok(val)) => { LAST_ERROR.with(|e| *e.borrow_mut() = None); val }\n");
            out.push_str("        Ok(Err(err)) => { LAST_ERROR.with(|e| *e.borrow_mut() = Some(err.to_string())); Default::default() }\n");
            out.push_str("        Err(panic) => {\n");
            out.push_str("            let msg = panic.downcast_ref::<String>()\n");
            out.push_str("                .map(|s| s.clone())\n");
            out.push_str("                .unwrap_or_else(|| \"unknown panic\".into());\n");
            out.push_str("            LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg));\n");
            out.push_str("            Default::default()\n");
            out.push_str("        }\n");
        }
    }

    out.push_str("    }\n}\n");
    out
}

/// Generate the Result → error code mapping stub (#267).
///
/// For functions that return `Result<T, E>`, generates a wrapper that:
/// 1. Calls the inner function
/// 2. Maps Ok(v) → stores v in out-param, returns 0
/// 3. Maps Err(e) → stores error string, returns error code
pub fn generate_result_check(fn_name: &str, ret_type: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "/// Error-checked wrapper for `{fn_name}` (#267).\n"
    ));
    out.push_str(&format!(
        "#[no_mangle]\npub extern \"C\" fn {fn_name}_checked(\
         out: *mut {ret_type}) -> i32 {{\n"
    ));
    out.push_str(&format!("    match {fn_name}() {{\n"));
    out.push_str(&format!(
        "        Ok(val) => {{ unsafe {{ *out = val }}; 0 }}\n"
    ));
    out.push_str("        Err(_) => 1,\n");
    out.push_str("    }\n}\n");
    out
}

/// Generate thread-local error storage for ThreadLocalFlag convention.
pub fn generate_error_storage() -> &'static str {
    "thread_local! {\n\
     \x20   static LAST_ERROR: std::cell::RefCell<Option<String>> = \
     std::cell::RefCell::new(None);\n\
     }\n\n\
     #[no_mangle]\n\
     pub extern \"C\" fn mamba_last_error() -> *const std::ffi::c_char {\n\
     \x20   LAST_ERROR.with(|e| {\n\
     \x20       match &*e.borrow() {\n\
     \x20           Some(s) => s.as_ptr() as *const std::ffi::c_char,\n\
     \x20           None => std::ptr::null(),\n\
     \x20       }\n\
     \x20   })\n\
     }\n"
}

fn error_return_type(ret_type: &str, convention: &ResultConvention) -> String {
    match convention {
        ResultConvention::ErrorCode => "i32".into(),
        ResultConvention::NullablePointer => format!("*mut {ret_type}"),
        ResultConvention::ThreadLocalFlag => ret_type.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panic_wrapper_error_code() {
        let wrapper = SafeWrapper {
            fn_name: "compute".into(),
            params: vec![("x".into(), "i64".into())],
            return_type: "i64".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("catch_unwind"));
        assert!(code.contains("compute_safe"));
        assert!(code.contains("-> i32"));
        assert!(code.contains("Ok(Ok(_)) => 0"));
        assert!(code.contains("Err(_) => 2"));
    }

    #[test]
    fn test_panic_wrapper_nullable() {
        let wrapper = SafeWrapper {
            fn_name: "get_data".into(),
            params: vec![],
            return_type: "Data".into(),
            result_convention: ResultConvention::NullablePointer,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("-> *mut Data"));
        assert!(code.contains("std::ptr::null_mut()"));
    }

    #[test]
    fn test_result_check() {
        let code = generate_result_check("compute", "i64");
        assert!(code.contains("compute_checked"));
        assert!(code.contains("out: *mut i64"));
        assert!(code.contains("Ok(val)"));
        assert!(code.contains("Err(_) => 1"));
    }

    #[test]
    fn test_error_storage() {
        let code = generate_error_storage();
        assert!(code.contains("LAST_ERROR"));
        assert!(code.contains("mamba_last_error"));
        assert!(code.contains("*const std::ffi::c_char"));
    }

    // --- Additional tests ---

    #[test]
    fn test_result_convention_eq() {
        assert_eq!(ResultConvention::ErrorCode, ResultConvention::ErrorCode);
        assert_ne!(ResultConvention::ErrorCode, ResultConvention::NullablePointer);
        assert_ne!(
            ResultConvention::NullablePointer,
            ResultConvention::ThreadLocalFlag
        );
    }

    #[test]
    fn test_error_return_type_error_code() {
        let ret = error_return_type("i64", &ResultConvention::ErrorCode);
        assert_eq!(ret, "i32");
    }

    #[test]
    fn test_error_return_type_nullable_pointer() {
        let ret = error_return_type("Data", &ResultConvention::NullablePointer);
        assert_eq!(ret, "*mut Data");
    }

    #[test]
    fn test_error_return_type_thread_local() {
        let ret = error_return_type("i64", &ResultConvention::ThreadLocalFlag);
        assert_eq!(ret, "i64");
    }

    #[test]
    fn test_panic_wrapper_thread_local() {
        let wrapper = SafeWrapper {
            fn_name: "process".into(),
            params: vec![("buf".into(), "*const u8".into())],
            return_type: "i32".into(),
            result_convention: ResultConvention::ThreadLocalFlag,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("process_safe"));
        assert!(code.contains("-> i32"));
        assert!(code.contains("LAST_ERROR"));
        assert!(code.contains("Default::default()"));
        assert!(code.contains("unknown panic"));
    }

    #[test]
    fn test_panic_wrapper_no_params() {
        let wrapper = SafeWrapper {
            fn_name: "noop".into(),
            params: vec![],
            return_type: "()".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("noop_safe()"));
        assert!(code.contains("noop()"));
    }

    #[test]
    fn test_panic_wrapper_multiple_params() {
        let wrapper = SafeWrapper {
            fn_name: "sum".into(),
            params: vec![
                ("a".into(), "i32".into()),
                ("b".into(), "i32".into()),
                ("c".into(), "i32".into()),
            ],
            return_type: "i32".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("a: i32, b: i32, c: i32"));
        assert!(code.contains("sum(a, b, c)"));
    }

    #[test]
    fn test_panic_wrapper_has_no_mangle() {
        let wrapper = SafeWrapper {
            fn_name: "f".into(),
            params: vec![],
            return_type: "i32".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("#[no_mangle]"));
        assert!(code.contains("pub extern \"C\""));
    }

    #[test]
    fn test_panic_wrapper_error_code_has_rust_error_branch() {
        let wrapper = SafeWrapper {
            fn_name: "f".into(),
            params: vec![],
            return_type: "i32".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("Ok(Err(_)) => 1"));
    }

    #[test]
    fn test_panic_wrapper_nullable_ok_val() {
        let wrapper = SafeWrapper {
            fn_name: "alloc".into(),
            params: vec![("sz".into(), "usize".into())],
            return_type: "u8".into(),
            result_convention: ResultConvention::NullablePointer,
        };
        let code = generate_panic_wrapper(&wrapper);
        assert!(code.contains("Ok(Ok(val)) => val"));
        assert!(code.contains("Ok(Err(_)) => std::ptr::null_mut()"));
    }

    #[test]
    fn test_result_check_output_format() {
        let code = generate_result_check("do_work", "f64");
        assert!(code.contains("/// Error-checked wrapper for `do_work`"));
        assert!(code.contains("#[no_mangle]"));
        assert!(code.contains("pub extern \"C\" fn do_work_checked"));
        assert!(code.contains("out: *mut f64) -> i32"));
    }

    #[test]
    fn test_error_storage_thread_local_macro() {
        let code = generate_error_storage();
        assert!(code.contains("thread_local!"));
        assert!(code.contains("RefCell<Option<String>>"));
    }

    #[test]
    fn test_error_storage_null_when_no_error() {
        let code = generate_error_storage();
        assert!(code.contains("None => std::ptr::null()"));
    }

    #[test]
    fn test_safe_wrapper_clone() {
        let wrapper = SafeWrapper {
            fn_name: "test".into(),
            params: vec![],
            return_type: "i32".into(),
            result_convention: ResultConvention::ErrorCode,
        };
        let cloned = wrapper.clone();
        assert_eq!(cloned.fn_name, "test");
        assert_eq!(cloned.result_convention, ResultConvention::ErrorCode);
    }

    #[test]
    fn test_result_convention_clone() {
        let rc = ResultConvention::ThreadLocalFlag;
        let cloned = rc.clone();
        assert_eq!(cloned, ResultConvention::ThreadLocalFlag);
    }
}
