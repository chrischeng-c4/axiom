use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use crate::source::FileId;
/// compileall module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `compileall` entry points (`compile_dir`, `compile_file`,
/// `compile_path`, `main`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit
/// CPython's module-dict probe chain for read-only compileall
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn extract_str_v(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    }
}

fn kwargs_of(a: &[MbValue]) -> Option<MbValue> {
    let last = *a.last()?;
    last.as_ptr().and_then(|p| unsafe {
        if matches!((*p).data, ObjData::Dict(_)) {
            Some(last)
        } else {
            None
        }
    })
}

fn kwarg(a: &[MbValue], key: &str) -> Option<MbValue> {
    let kw = kwargs_of(a)?;
    let ptr = kw.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            return lock.read().unwrap().get(key).copied();
        }
    }
    None
}

/// True iff the source at `path` parses; missing files are vacuous successes.
fn compile_one(path: &str) -> bool {
    let Ok(src) = std::fs::read_to_string(path) else {
        return true; // nothing to compile, nothing to fail
    };
    crate::parser::parse(&src, FileId::default()).is_ok()
}

unsafe extern "C" fn dispatch_compile_dir(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    if let Some(w) = kwarg(a, "workers").and_then(|v| v.as_int()) {
        if w < 0 {
            return raise_exc("ValueError", "workers must be greater or equal to 0");
        }
    }
    let ddir_given = kwarg(a, "ddir").map(|v| !v.is_none()).unwrap_or(false);
    let strip_or_prepend = kwarg(a, "stripdir").map(|v| !v.is_none()).unwrap_or(false)
        || kwarg(a, "prependdir")
            .map(|v| !v.is_none())
            .unwrap_or(false);
    if ddir_given && strip_or_prepend {
        return raise_exc(
            "ValueError",
            "Destination dir (ddir) cannot be used in combination with stripdir or prependdir",
        );
    }
    let dir = a
        .first()
        .copied()
        .and_then(extract_str_v)
        .unwrap_or_default();
    let mut ok = true;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("py") {
                ok &= compile_one(&p.to_string_lossy());
            }
        }
    }
    MbValue::from_bool(ok)
}

unsafe extern "C" fn dispatch_compile_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let ddir_given = kwarg(a, "ddir").map(|v| !v.is_none()).unwrap_or(false);
    let strip_or_prepend = kwarg(a, "stripdir").map(|v| !v.is_none()).unwrap_or(false)
        || kwarg(a, "prependdir")
            .map(|v| !v.is_none())
            .unwrap_or(false);
    if ddir_given && strip_or_prepend {
        return raise_exc(
            "ValueError",
            "Destination dir (ddir) cannot be used in combination with stripdir or prependdir",
        );
    }
    let path = a
        .first()
        .copied()
        .and_then(extract_str_v)
        .unwrap_or_default();
    MbValue::from_bool(compile_one(&path))
}

unsafe extern "C" fn dispatch_compile_path(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_bool(true)
}

unsafe extern "C" fn dispatch_main(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the compileall module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_cd = dispatch_compile_dir as *const () as usize;
    attrs.insert("compile_dir".into(), MbValue::from_func(addr_cd));

    let addr_cf = dispatch_compile_file as *const () as usize;
    attrs.insert("compile_file".into(), MbValue::from_func(addr_cf));

    let addr_cp = dispatch_compile_path as *const () as usize;
    attrs.insert("compile_path".into(), MbValue::from_func(addr_cp));

    let addr_m = dispatch_main as *const () as usize;
    attrs.insert("main".into(), MbValue::from_func(addr_m));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_cd as u64);
        set.insert(addr_cf as u64);
        set.insert(addr_cp as u64);
        set.insert(addr_m as u64);
    });

    super::register_module("compileall", attrs);
}
