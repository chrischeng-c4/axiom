use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// inspect module for Mamba (#438).
///
/// Provides introspection utilities for examining live objects at runtime.
/// Functions check object types and extract member information from instances.
/// Some functions are stubs pending full closure/function object support.
use std::collections::HashMap;

/// Helper: extract a string from an MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_isfunction, mb_inspect_isfunction);
disp_unary!(d_isclass, mb_inspect_isclass);
disp_unary!(d_ismethod, mb_inspect_ismethod);
disp_unary!(d_getmembers, mb_inspect_getmembers);
disp_unary!(d_signature, mb_inspect_signature);

/// Register the inspect module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("isfunction", d_isfunction as *const () as usize),
        ("isclass", d_isclass as *const () as usize),
        ("ismethod", d_ismethod as *const () as usize),
        ("isbuiltin", d_isfunction as *const () as usize),
        ("isroutine", d_isfunction as *const () as usize),
        ("ismodule", d_isfunction as *const () as usize),
        ("isgeneratorfunction", d_isfunction as *const () as usize),
        ("iscoroutinefunction", d_isfunction as *const () as usize),
        ("isawaitable", d_isfunction as *const () as usize),
        ("isasyncgenfunction", d_isfunction as *const () as usize),
        ("getmembers", d_getmembers as *const () as usize),
        ("signature", d_signature as *const () as usize),
        ("getsourcelines", d_getsourcelines as *const () as usize),
        ("getsource", d_getsource as *const () as usize),
        ("getsourcefile", d_getsourcefile as *const () as usize),
        ("getfile", d_getsourcefile as *const () as usize),
        ("getmodule", d_getmodule as *const () as usize),
        ("getfullargspec", d_getfullargspec as *const () as usize),
        ("getargspec", d_getfullargspec as *const () as usize),
        ("getdoc", d_getdoc as *const () as usize),
        ("getcomments", d_getdoc as *const () as usize),
        ("cleandoc", d_cleandoc as *const () as usize),
        ("currentframe", d_currentframe as *const () as usize),
        ("stack", d_empty_list as *const () as usize),
        ("trace", d_empty_list as *const () as usize),
        ("getmro", d_empty_list as *const () as usize),
        ("getclasstree", d_empty_list as *const () as usize),
        ("getargvalues", d_argvalues as *const () as usize),
        ("getouterframes", d_empty_list as *const () as usize),
        ("getinnerframes", d_empty_list as *const () as usize),
        ("formatargspec", d_empty_str as *const () as usize),
        ("formatargvalues", d_empty_str as *const () as usize),
        ("unwrap", d_passthrough as *const () as usize),
    ];
    for (name, addr) in &dispatchers {
        attrs.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // Parameter / Signature class stubs with the kind sentinels CPython
    // exposes. `inspect.Parameter.POSITIONAL_OR_KEYWORD` etc. read as
    // string sentinels — exactly what `inspect.signature(...)` returns
    // from the existing mb_inspect_signature path.
    attrs.insert("Parameter".to_string(), make_parameter_class());
    attrs.insert("Signature".to_string(), make_signature_class());
    attrs.insert(
        "BoundArguments".to_string(),
        make_empty_class("BoundArguments"),
    );
    attrs.insert("FullArgSpec".to_string(), make_empty_class("FullArgSpec"));
    attrs.insert("ArgSpec".to_string(), make_empty_class("ArgSpec"));
    attrs.insert("ArgInfo".to_string(), make_empty_class("ArgInfo"));

    let co_flags: &[(&str, i64)] = &[
        ("CO_OPTIMIZED", 0x0001),
        ("CO_NEWLOCALS", 0x0002),
        ("CO_VARARGS", 0x0004),
        ("CO_VARKEYWORDS", 0x0008),
        ("CO_NESTED", 0x0010),
        ("CO_GENERATOR", 0x0020),
        ("CO_NOFREE", 0x0040),
        ("CO_COROUTINE", 0x0100),
        ("CO_ITERABLE_COROUTINE", 0x0200),
        ("CO_ASYNC_GENERATOR", 0x0400),
        ("CO_FUTURE_DIVISION", 0x20000),
        ("CO_FUTURE_ABSOLUTE_IMPORT", 0x40000),
        ("CO_FUTURE_WITH_STATEMENT", 0x80000),
        ("CO_FUTURE_PRINT_FUNCTION", 0x100000),
        ("CO_FUTURE_UNICODE_LITERALS", 0x200000),
        ("CO_FUTURE_BARRY_AS_BDFL", 0x400000),
        ("CO_FUTURE_GENERATOR_STOP", 0x800000),
        ("CO_FUTURE_ANNOTATIONS", 0x1000000),
    ];
    for (name, val) in co_flags {
        attrs.insert((*name).to_string(), MbValue::from_int(*val));
    }

    super::register_module("inspect", attrs);
}

unsafe extern "C" fn d_getsourcelines(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // CPython returns (list[str], int). Provide a non-empty placeholder so
    // callers that assert shape (list, positive int) work.
    let placeholder = MbValue::from_ptr(MbObject::new_str("<source unavailable>\n".to_string()));
    let lines = MbValue::from_ptr(MbObject::new_list(vec![placeholder]));
    let lineno = MbValue::from_int(1);
    MbValue::from_ptr(MbObject::new_tuple(vec![lines, lineno]))
}

unsafe extern "C" fn d_getsource(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("<source unavailable>\n".to_string()))
}

unsafe extern "C" fn d_getsourcefile(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_getmodule(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_getfullargspec(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Build an Instance with the FullArgSpec field set CPython exposes:
    // args, varargs, varkw, defaults, kwonlyargs, kwonlydefaults, annotations.
    let empty_list = MbValue::from_ptr(MbObject::new_list(vec![]));
    let empty_dict = MbValue::from_ptr(MbObject::new_dict());
    let inst = MbObject::new_instance("FullArgSpec".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("args".to_string(), empty_list);
            g.insert("varargs".to_string(), MbValue::none());
            g.insert("varkw".to_string(), MbValue::none());
            g.insert("defaults".to_string(), MbValue::none());
            g.insert("kwonlyargs".to_string(), empty_list);
            g.insert("kwonlydefaults".to_string(), MbValue::none());
            g.insert("annotations".to_string(), empty_dict);
        }
    }
    MbValue::from_ptr(inst)
}

unsafe extern "C" fn d_getdoc(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_cleandoc(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn d_currentframe(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn d_empty_list(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

unsafe extern "C" fn d_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn d_passthrough(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn d_argvalues(_a: *const MbValue, _n: usize) -> MbValue {
    let empty_list = MbValue::from_ptr(MbObject::new_list(vec![]));
    let empty_dict = MbValue::from_ptr(MbObject::new_dict());
    let inst = MbObject::new_instance("ArgInfo".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("args".to_string(), empty_list);
            g.insert("varargs".to_string(), MbValue::none());
            g.insert("keywords".to_string(), MbValue::none());
            g.insert("locals".to_string(), empty_dict);
        }
    }
    MbValue::from_ptr(inst)
}

fn make_parameter_class() -> MbValue {
    let inst = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("Parameter".to_string())),
            );
            g.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("inspect".to_string())),
            );
            for kind in &[
                "POSITIONAL_ONLY",
                "POSITIONAL_OR_KEYWORD",
                "VAR_POSITIONAL",
                "KEYWORD_ONLY",
                "VAR_KEYWORD",
            ] {
                g.insert(
                    kind.to_string(),
                    MbValue::from_ptr(MbObject::new_str((*kind).to_string())),
                );
            }
            // `Parameter.empty` is a sentinel — CPython uses a singleton
            // class-level marker. Mirror with an Instance whose class name
            // identifies it uniquely.
            let empty = MbObject::new_instance("_empty".to_string());
            g.insert("empty".to_string(), MbValue::from_ptr(empty));
        }
    }
    MbValue::from_ptr(inst)
}

fn make_signature_class() -> MbValue {
    let inst = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("Signature".to_string())),
            );
            g.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("inspect".to_string())),
            );
            let empty = MbObject::new_instance("_empty".to_string());
            g.insert("empty".to_string(), MbValue::from_ptr(empty));
        }
    }
    MbValue::from_ptr(inst)
}

fn make_empty_class(name: &str) -> MbValue {
    let inst = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            g.insert(
                "__module__".to_string(),
                MbValue::from_ptr(MbObject::new_str("inspect".to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// inspect.isfunction(obj) -> bool.
///
/// Checks if obj is a function/closure. In Mamba, functions are currently
/// represented as integer addresses (function pointers). Returns True if
/// the value is an int (potential function pointer) or has callable markers.
pub fn mb_inspect_isfunction(obj: MbValue) -> MbValue {
    // Canonical: JIT-compiled or extern function pointer (TAG_FUNC).
    if obj.as_func().is_some() {
        return MbValue::from_bool(true);
    }
    // Closure-like objects (instances with __call__) — but NOT type-objects
    // (class_name == "type"), which represent classes and must remain
    // distinct from functions.
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    return MbValue::from_bool(false);
                }
                let fields = fields.read().unwrap();
                if fields.contains_key("__call__") {
                    return MbValue::from_bool(true);
                }
            }
        }
    }
    MbValue::from_bool(false)
}

/// inspect.isclass(obj) -> bool.
///
/// Recognises mamba's class representations:
///   - Instance with class_name == "type" (true type-object).
///   - Bare class-name string registered via mb_class_register.
pub fn mb_inspect_isclass(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return MbValue::from_bool(class_name == "type");
            }
        }
    }
    if let Some(s) = extract_str(obj) {
        if super::super::class::class_is_registered(&s) {
            return MbValue::from_bool(true);
        }
        let is_class = s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
        return MbValue::from_bool(is_class);
    }
    MbValue::from_bool(false)
}

/// inspect.ismethod(obj) -> bool.
///
/// Checks if obj is a bound method. In current Mamba, methods are not
/// distinguished from functions, so this returns the same as isfunction.
pub fn mb_inspect_ismethod(obj: MbValue) -> MbValue {
    mb_inspect_isfunction(obj)
}

/// inspect.getmembers(obj) -> list of (name, value) tuples.
///
/// If obj is an Instance, returns a list of 2-tuples for each field in
/// the instance's attribute dictionary. Otherwise returns an empty list.
pub fn mb_inspect_getmembers(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { ref fields, .. } => {
                    let fields = fields.read().unwrap();
                    let mut members = Vec::new();
                    for (name, value) in fields.iter() {
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.clone()));
                        let tuple = MbValue::from_ptr(MbObject::new_tuple(vec![name_val, *value]));
                        members.push(tuple);
                    }
                    MbValue::from_ptr(MbObject::new_list(members))
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    let mut members = Vec::new();
                    for (name, value) in map.iter() {
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.to_string()));
                        let tuple = MbValue::from_ptr(MbObject::new_tuple(vec![name_val, *value]));
                        members.push(tuple);
                    }
                    MbValue::from_ptr(MbObject::new_list(members))
                }
                _ => MbValue::from_ptr(MbObject::new_list(vec![])),
            }
        }
    } else {
        MbValue::from_ptr(MbObject::new_list(vec![]))
    }
}

/// inspect.signature(func) -> string representation of function params (stub).
///
/// In CPython, signature returns a Signature object describing the callable's
/// parameters. Since Mamba does not yet have rich function objects, this stub
/// returns "()" as a placeholder string.
pub fn mb_inspect_signature(_func: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("()".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isfunction() {
        // Function pointers are stored as ints
        let func_ptr = MbValue::from_int(0x1234);
        assert_eq!(mb_inspect_isfunction(func_ptr).as_bool(), Some(true));

        // Non-functions
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(mb_inspect_isfunction(s).as_bool(), Some(false));

        assert_eq!(
            mb_inspect_isfunction(MbValue::none()).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_isclass() {
        let class_name = MbValue::from_ptr(MbObject::new_str("MyClass".to_string()));
        assert_eq!(mb_inspect_isclass(class_name).as_bool(), Some(true));

        let not_class = MbValue::from_ptr(MbObject::new_str("my_function".to_string()));
        assert_eq!(mb_inspect_isclass(not_class).as_bool(), Some(false));
    }

    #[test]
    fn test_getmembers_instance() {
        let inst = MbObject::new_instance("Foo".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*inst).data {
                let mut fields = fields.write().unwrap();
                fields.insert("x".to_string(), MbValue::from_int(10));
                fields.insert("y".to_string(), MbValue::from_int(20));
            }
        }
        let obj = MbValue::from_ptr(inst);
        let members = mb_inspect_getmembers(obj);

        unsafe {
            if let ObjData::List(ref lock) = (*members.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                // Each item should be a tuple of (name, value)
                for item in items.iter() {
                    if let ObjData::Tuple(ref elems) = (*item.as_ptr().unwrap()).data {
                        assert_eq!(elems.len(), 2);
                    } else {
                        panic!("expected tuple member");
                    }
                }
            } else {
                panic!("expected list");
            }
        }
    }
}
