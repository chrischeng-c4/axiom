use super::super::rc::MbObject;
use super::super::value::MbValue;
/// fileinput module for Mamba (#1261 long-tail).
///
/// Iterates over lines from a list of files. Surface-only shim — returns
/// an empty list for `input()` calls when no files are provided. The
/// per-file metadata helpers (`filename()`, `lineno()`, `isfirstline()`)
/// each return a fixed sentinel so the chained-API protocol resolves
/// without crashing.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_input(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn dispatch_filename(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("".to_string()))
}

unsafe extern "C" fn dispatch_lineno(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_filelineno(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_isfirstline(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

unsafe extern "C" fn dispatch_isstdin(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

unsafe extern "C" fn dispatch_nextfile(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_close(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_fileinput_class(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("input", dispatch_input as *const () as usize),
        ("filename", dispatch_filename as *const () as usize),
        ("lineno", dispatch_lineno as *const () as usize),
        ("filelineno", dispatch_filelineno as *const () as usize),
        ("isfirstline", dispatch_isfirstline as *const () as usize),
        ("isstdin", dispatch_isstdin as *const () as usize),
        ("nextfile", dispatch_nextfile as *const () as usize),
        ("close", dispatch_close as *const () as usize),
        ("FileInput", dispatch_fileinput_class as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("fileinput", attrs);
}
