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

unsafe extern "C" fn pipes_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn pipes_clone(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("pipes.Template".to_string()))
}

unsafe extern "C" fn pipes_quote(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    args.first().copied().unwrap_or_else(|| new_str(""))
}

// `pipes.Template()` constructor. Registered under the CLASS_REGISTRY key
// "pipes.Template" (NOT the bare "Template") — `string.Template` registers
// its own, unrelated method table under the bare "Template" key (see
// string_constants_mod.rs), and `mb_class_register` does a plain HashMap
// insert that replaces whatever was there. With both modules keyed on the
// same bare name, whichever `register()` runs last in `register_stdlib()`
// silently clobbered the other's methods, so e.g. `string.Template(...)
// .substitute(...)` resolved through pipes' no-op method table instead of
// its own (getattr/call on the instance both consult CLASS_REGISTRY by
// class_name, and a colliding key means the wrong table wins). Giving pipes
// its own real constructor (mirroring `string_constants_mod::dispatch_
// template`) instead of the generic type-stub auto-construction path also
// means `pipes.Template()` never accidentally routes through whichever
// class the "Template" key happens to hold.
unsafe extern "C" fn dispatch_pipes_template(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("pipes.Template".to_string()))
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
    super::super::class::mb_class_register("pipes.Template", vec!["object".to_string()], map);
}

pub fn register() {
    register_template_class();

    let quote_addr = pipes_quote as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(quote_addr as u64);
    });

    let template_addr = dispatch_pipes_template as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(template_addr as u64);
    });
    super::super::module::register_native_type_name(
        template_addr as u64,
        "pipes.Template".to_string(),
    );

    let mut attrs = HashMap::new();
    attrs.insert("Template".to_string(), MbValue::from_func(template_addr));
    attrs.insert("quote".to_string(), MbValue::from_func(quote_addr));
    super::register_module("pipes", attrs);
}
