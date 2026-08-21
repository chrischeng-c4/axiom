use super::super::rc::MbObject;
use super::super::value::MbValue;
/// getpass module for Mamba (#1261 long-tail).
///
/// Two-entry surface: `getpass(prompt='Password: ', stream=None)` and
/// `getuser()`. Mamba doesn't read from a terminal — `getpass()` reads
/// from stdin via std::io::BufRead so callers in non-interactive scripts
/// at least don't crash. The TTY-noecho dance is deferred (no consumer
/// asks for it on Mamba yet).
use std::collections::HashMap;
use std::io::BufRead;

unsafe extern "C" fn dispatch_getpass(_a: *const MbValue, _n: usize) -> MbValue {
    let stdin = std::io::stdin();
    let mut line = String::new();
    if stdin.lock().read_line(&mut line).is_ok() {
        // Strip trailing newline if present.
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
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

/// De-registered in favor of the vendored `py_src/getpass.py` port (#868
/// round 6): registering a native module here would pre-seed `getpass` into
/// `MODULES` and permanently shadow the vendored source (see
/// `vendor_lib.rs` precedence doc). `register()` is kept as a no-op call
/// site (invoked from `stdlib::register_stdlib()`) so the migration didn't
/// need to touch that call list. The dispatch functions/helpers above are
/// dead code kept for reference; nothing calls them anymore.
pub fn register() {
    // Intentionally empty: vendor_lib::register() (called earlier in
    // stdlib::register_stdlib()) already materializes py_src/getpass.py into
    // the shared vendored search-path directory.
}
