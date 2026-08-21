//! Minimal `cmd` module surface.
//!
//! This does not implement the interactive command loop. It provides a real
//! `Cmd` type object and method table so CPython-derived type-wall fixtures can
//! construct bare receivers with `object.__new__(Cmd)` and grade method
//! signatures.

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

unsafe extern "C" fn cmd_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn cmd_identity_line(_self_v: MbValue, args: MbValue) -> MbValue {
    super::super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn cmd_empty_list(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn cmd_parse_line(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::none(),
        MbValue::none(),
        MbValue::none(),
    ]))
}

fn register_cmd_class() {
    let methods: [(&str, usize); 13] = [
        ("__init__", cmd_noop as *const () as usize),
        ("cmdloop", cmd_noop as *const () as usize),
        ("columnize", cmd_noop as *const () as usize),
        ("complete", cmd_noop as *const () as usize),
        ("completenames", cmd_empty_list as *const () as usize),
        ("default", cmd_noop as *const () as usize),
        ("do_help", cmd_noop as *const () as usize),
        ("onecmd", cmd_noop as *const () as usize),
        ("parseline", cmd_parse_line as *const () as usize),
        ("postcmd", cmd_identity_line as *const () as usize),
        ("precmd", cmd_identity_line as *const () as usize),
        ("print_topics", cmd_noop as *const () as usize),
        ("emptyline", cmd_noop as *const () as usize),
    ];
    let mut map = HashMap::new();
    for (name, addr) in methods {
        super::super::module::register_variadic_func(addr as u64);
        map.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("Cmd", vec!["object".to_string()], map);
}

pub fn register() {
    register_cmd_class();
    let mut attrs = HashMap::new();
    attrs.insert("Cmd".to_string(), make_type_obj("Cmd", "cmd"));
    super::register_module("cmd", attrs);
}
