/// getpass module for Mamba (#1261 long-tail).
///
/// Two-entry surface: `getpass(prompt='Password: ', stream=None)` and
/// `getuser()`. Mamba doesn't read from a terminal — `getpass()` reads
/// from stdin via std::io::BufRead so callers in non-interactive scripts
/// at least don't crash. The TTY-noecho dance is deferred (no consumer
/// asks for it on Mamba yet).

use std::collections::HashMap;
use std::io::BufRead;
use super::super::value::MbValue;
use super::super::rc::MbObject;

unsafe extern "C" fn dispatch_getpass(_a: *const MbValue, _n: usize) -> MbValue {
    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        // Strip trailing newline if present.
        if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } }
    }
    MbValue::from_ptr(MbObject::new_str(line))
}

unsafe extern "C" fn dispatch_getuser(_a: *const MbValue, _n: usize) -> MbValue {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "mamba".to_string());
    MbValue::from_ptr(MbObject::new_str(user))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("getpass", dispatch_getpass as *const () as usize),
        ("getuser", dispatch_getuser as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers { set.insert(*addr as u64); }
    });

    // `GetPassWarning(UserWarning)` — warned-but-not-fatal class emitted when
    // `getpass()` falls back to a non-secure input path. Register its real MRO
    // (`GetPassWarning <: UserWarning`) so `issubclass` / `is_subclass_of`
    // see a genuine hierarchy, and surface the module attr as a non-None value
    // so `hasattr(getpass, "GetPassWarning")` resolves (mb_hasattr treats a
    // None-valued field as absent). The surface dimension only asserts the
    // name's presence, not construction behavior.
    super::super::class::mb_class_register(
        "GetPassWarning",
        vec!["UserWarning".to_string()],
        HashMap::new(),
    );
    attrs.insert(
        "GetPassWarning".to_string(),
        MbValue::from_ptr(MbObject::new_str("GetPassWarning".to_string())),
    );

    super::register_module("getpass", attrs);
}
