//! Minimal `pipes` module surface for CPython 3.12 compatibility.
//!
//! `pipes` is deprecated in CPython 3.12, but still importable. Mamba keeps a
//! small shell so type-wall fixtures and legacy probes can exercise the public
//! surface without pulling in shell-pipeline behavior.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn make_type_obj(name: &str, module: &str) -> MbValue {
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut map = fields.write().unwrap();
            map.insert("__name__".to_string(), new_str(name));
            map.insert("__qualname__".to_string(), new_str(name));
            map.insert("__module__".to_string(), new_str(module));
        }
    }
    MbValue::from_ptr(obj)
}

unsafe extern "C" fn pipes_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn pipes_clone(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("Template".to_string()))
}

unsafe extern "C" fn pipes_quote(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    args.first().copied().unwrap_or_else(|| new_str(""))
}

fn register_template_class() {
    let methods: [(&str, usize); 8] = [
        ("__init__", pipes_noop as *const () as usize),
        ("append", pipes_noop as *const () as usize),
        ("clone", pipes_clone as *const () as usize),
        ("copy", pipes_noop as *const () as usize),
        ("debug", pipes_noop as *const () as usize),
        ("open", pipes_noop as *const () as usize),
        ("prepend", pipes_noop as *const () as usize),
        ("reset", pipes_noop as *const () as usize),
    ];
    let mut map = HashMap::new();
    for (name, addr) in methods {
        super::super::module::register_variadic_func(addr as u64);
        map.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("Template", vec!["object".to_string()], map);
}

pub fn register() {
    register_template_class();

    let quote_addr = pipes_quote as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(quote_addr as u64);
    });

    let mut attrs = HashMap::new();
    attrs.insert("Template".to_string(), make_type_obj("Template", "pipes"));
    attrs.insert("quote".to_string(), MbValue::from_func(quote_addr));
    super::register_module("pipes", attrs);
}
