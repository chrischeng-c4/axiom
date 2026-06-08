/// readline module for Mamba (#1261 long-tail).
///
/// Mamba isn't an interactive REPL, so the GNU-readline line-editing
/// surface (parse_and_bind, set_completer, insert_text, ...) stays
/// no-op. But the history-management surface — `add_history`,
/// `get_history_length`, `read_history_file`, `write_history_file` —
/// is useful even outside a real REPL, because plenty of CPython
/// scripts use it as a generic "remember this line for later" sink
/// backed by a file. We back it with a process-wide in-memory list
/// plus straight file I/O.
///
/// Replaces the long_tail readline stub that returned `""` / `0` /
/// `None` for everything — every history-aware script was silently
/// broken.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// Process-wide history list (`item_index >= 1` per CPython convention).
fn history() -> &'static Mutex<Vec<String>> {
    static HISTORY: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    HISTORY.get_or_init(|| Mutex::new(Vec::new()))
}

/// Configurable max history length (-1 = unlimited per CPython).
fn history_length_setting() -> &'static Mutex<i64> {
    static LEN: OnceLock<Mutex<i64>> = OnceLock::new();
    LEN.get_or_init(|| Mutex::new(-1))
}

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => std::str::from_utf8(b).ok().map(str::to_string),
        _ => None,
    }
}

fn trim_to_max(items: &mut Vec<String>, max: i64) {
    if max < 0 { return; }
    let max = max as usize;
    if items.len() > max {
        let drop = items.len() - max;
        items.drain(..drop);
    }
}

unsafe extern "C" fn dispatch_add_history(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(line) = args.first().copied().and_then(|v| as_str(v)) else {
        return MbValue::none();
    };
    let max = *history_length_setting().lock().unwrap_or_else(|e| { history_length_setting().clear_poison(); e.into_inner() });
    let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    h.push(line);
    trim_to_max(&mut h, max);
    MbValue::none()
}

unsafe extern "C" fn dispatch_clear_history(_a: *const MbValue, _n: usize) -> MbValue {
    history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).clear();
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_history_length(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython's `get_history_length` returns the *configured* max,
    // not the current item count. Item count goes through
    // `get_current_history_length`.
    MbValue::from_int(*history_length_setting().lock().unwrap_or_else(|e| { history_length_setting().clear_poison(); e.into_inner() }))
}

unsafe extern "C" fn dispatch_set_history_length(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(n) = args.first().and_then(|v| v.as_int()) else {
        return MbValue::none();
    };
    *history_length_setting().lock().unwrap_or_else(|e| { history_length_setting().clear_poison(); e.into_inner() }) = n;
    if n >= 0 {
        let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
        trim_to_max(&mut h, n);
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_current_history_length(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).len() as i64)
}

unsafe extern "C" fn dispatch_get_history_item(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(idx) = args.first().and_then(|v| v.as_int()) else {
        return MbValue::none();
    };
    // CPython readline indices are 1-based; 0 returns None.
    if idx < 1 { return MbValue::none(); }
    let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    match h.get((idx - 1) as usize) {
        Some(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        None => MbValue::none(),
    }
}

unsafe extern "C" fn dispatch_remove_history_item(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(idx) = args.first().and_then(|v| v.as_int()) else {
        return MbValue::none();
    };
    // CPython remove_history_item is 0-based (yes, asymmetric with get).
    if idx < 0 { return MbValue::none(); }
    let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    if (idx as usize) < h.len() {
        h.remove(idx as usize);
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_replace_history_item(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(idx) = args.first().and_then(|v| v.as_int()) else {
        return MbValue::none();
    };
    let Some(line) = args.get(1).copied().and_then(|v| as_str(v)) else {
        return MbValue::none();
    };
    if idx < 0 { return MbValue::none(); }
    let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    if let Some(slot) = h.get_mut(idx as usize) {
        *slot = line;
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_read_history_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let path = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.history", h)))
        .unwrap_or_default();
    if let Ok(content) = std::fs::read_to_string(&path) {
        let max = *history_length_setting().lock().unwrap_or_else(|e| { history_length_setting().clear_poison(); e.into_inner() });
        let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
        for line in content.lines() {
            h.push(line.to_string());
        }
        trim_to_max(&mut h, max);
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_write_history_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(path) = args.first().copied().and_then(|v| as_str(v)) else {
        return MbValue::none();
    };
    let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    let body = h.join("\n");
    let _ = std::fs::write(&path, format!("{}\n", body));
    MbValue::none()
}

unsafe extern "C" fn dispatch_append_history_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(nelements) = args.first().and_then(|v| v.as_int()) else {
        return MbValue::none();
    };
    let Some(path) = args.get(1).copied().and_then(|v| as_str(v)) else {
        return MbValue::none();
    };
    if nelements < 1 { return MbValue::none(); }
    let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
    let take = (nelements as usize).min(h.len());
    let tail: Vec<&str> = h.iter().rev().take(take).map(|s| s.as_str()).collect();
    let tail: Vec<&str> = tail.into_iter().rev().collect();
    let body = tail.join("\n");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        use std::io::Write;
        let _ = writeln!(f, "{}", body);
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue { MbValue::none() }
unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}
unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        // Real history surface.
        ("add_history",                dispatch_add_history as *const () as usize),
        ("clear_history",              dispatch_clear_history as *const () as usize),
        ("get_history_length",         dispatch_get_history_length as *const () as usize),
        ("set_history_length",         dispatch_set_history_length as *const () as usize),
        ("get_current_history_length", dispatch_get_current_history_length as *const () as usize),
        ("get_history_item",           dispatch_get_history_item as *const () as usize),
        ("remove_history_item",        dispatch_remove_history_item as *const () as usize),
        ("replace_history_item",       dispatch_replace_history_item as *const () as usize),
        ("read_history_file",          dispatch_read_history_file as *const () as usize),
        ("write_history_file",         dispatch_write_history_file as *const () as usize),
        ("append_history_file",        dispatch_append_history_file as *const () as usize),
        // No-op line-editing surface (Mamba isn't an interactive REPL).
        ("parse_and_bind",         dispatch_noop as *const () as usize),
        ("get_line_buffer",        dispatch_empty_str as *const () as usize),
        ("insert_text",            dispatch_noop as *const () as usize),
        ("read_init_file",         dispatch_noop as *const () as usize),
        ("redisplay",              dispatch_noop as *const () as usize),
        ("set_startup_hook",       dispatch_noop as *const () as usize),
        ("set_pre_input_hook",     dispatch_noop as *const () as usize),
        ("set_completer",          dispatch_noop as *const () as usize),
        ("get_completer",          dispatch_noop as *const () as usize),
        ("get_completion_type",    dispatch_int_zero as *const () as usize),
        ("get_begidx",             dispatch_int_zero as *const () as usize),
        ("get_endidx",             dispatch_int_zero as *const () as usize),
        ("set_completer_delims",   dispatch_noop as *const () as usize),
        ("get_completer_delims",   dispatch_empty_str as *const () as usize),
        ("set_auto_history",       dispatch_noop as *const () as usize),
    ];
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::register_module("readline", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    // Serialize tests that share the process-global history list.
    fn test_lock() -> &'static Mutex<()> {
        static L: OnceLock<Mutex<()>> = OnceLock::new();
        L.get_or_init(|| Mutex::new(()))
    }

    fn test_guard() -> MutexGuard<'static, ()> {
        // Clear poisoning if a prior test panicked.
        test_lock().lock().unwrap_or_else(|e| {
            test_lock().clear_poison();
            e.into_inner()
        })
    }

    fn reset_history() {
        let mut h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() });
        h.clear();
        drop(h);
        let mut l = history_length_setting().lock().unwrap_or_else(|e| { history_length_setting().clear_poison(); e.into_inner() });
        *l = -1;
    }

    #[test]
    fn add_and_count() {
        let _g = test_guard();
        reset_history();
        unsafe {
            let a = MbValue::from_ptr(MbObject::new_str("first".to_string()));
            dispatch_add_history(&a as *const _, 1);
            let b = MbValue::from_ptr(MbObject::new_str("second".to_string()));
            dispatch_add_history(&b as *const _, 1);
            let n = dispatch_get_current_history_length(std::ptr::null(), 0);
            assert_eq!(n.as_int(), Some(2));
        }
    }

    #[test]
    fn get_history_item_one_based() {
        let _g = test_guard();
        reset_history();
        unsafe {
            let a = MbValue::from_ptr(MbObject::new_str("alpha".to_string()));
            dispatch_add_history(&a as *const _, 1);
            let one = MbValue::from_int(1);
            let result = dispatch_get_history_item(&one as *const _, 1);
            let s = result.as_ptr().and_then(|p| match &(*p).data {
                ObjData::Str(s) => Some(s.clone()),
                _ => None,
            });
            assert_eq!(s.as_deref(), Some("alpha"));
            // Zero returns None per CPython.
            let zero = MbValue::from_int(0);
            let none_res = dispatch_get_history_item(&zero as *const _, 1);
            assert!(none_res.as_ptr().is_none());
        }
    }

    #[test]
    fn remove_history_item_zero_based() {
        let _g = test_guard();
        reset_history();
        unsafe {
            for s in ["a", "b", "c"] {
                let v = MbValue::from_ptr(MbObject::new_str(s.into()));
                dispatch_add_history(&v as *const _, 1);
            }
            let one = MbValue::from_int(1);
            dispatch_remove_history_item(&one as *const _, 1);
            let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).clone();
            assert_eq!(h, vec!["a", "c"]);
        }
    }

    #[test]
    fn replace_history_item_works() {
        let _g = test_guard();
        reset_history();
        unsafe {
            for s in ["a", "b", "c"] {
                let v = MbValue::from_ptr(MbObject::new_str(s.into()));
                dispatch_add_history(&v as *const _, 1);
            }
            let idx = MbValue::from_int(1);
            let new_val = MbValue::from_ptr(MbObject::new_str("B".into()));
            let args = [idx, new_val];
            dispatch_replace_history_item(args.as_ptr(), 2);
            let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).clone();
            assert_eq!(h, vec!["a", "B", "c"]);
        }
    }

    #[test]
    fn clear_history_empties() {
        let _g = test_guard();
        reset_history();
        unsafe {
            for s in ["a", "b"] {
                let v = MbValue::from_ptr(MbObject::new_str(s.into()));
                dispatch_add_history(&v as *const _, 1);
            }
            dispatch_clear_history(std::ptr::null(), 0);
            assert_eq!(history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).len(), 0);
        }
    }

    #[test]
    fn set_history_length_trims() {
        let _g = test_guard();
        reset_history();
        unsafe {
            for s in ["a", "b", "c", "d"] {
                let v = MbValue::from_ptr(MbObject::new_str(s.into()));
                dispatch_add_history(&v as *const _, 1);
            }
            let two = MbValue::from_int(2);
            dispatch_set_history_length(&two as *const _, 1);
            // Should drop the oldest two ("a", "b") and keep ("c", "d").
            let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).clone();
            assert_eq!(h, vec!["c", "d"]);
            let cfg = dispatch_get_history_length(std::ptr::null(), 0);
            assert_eq!(cfg.as_int(), Some(2));
        }
    }

    #[test]
    fn write_and_read_roundtrip() {
        let _g = test_guard();
        reset_history();
        let tmp = std::env::temp_dir().join("mamba_readline_test_history");
        unsafe {
            for s in ["alpha", "beta", "gamma"] {
                let v = MbValue::from_ptr(MbObject::new_str(s.into()));
                dispatch_add_history(&v as *const _, 1);
            }
            let path = MbValue::from_ptr(MbObject::new_str(tmp.to_string_lossy().to_string()));
            dispatch_write_history_file(&path as *const _, 1);
            // Wipe in-memory and read back.
            dispatch_clear_history(std::ptr::null(), 0);
            dispatch_read_history_file(&path as *const _, 1);
            let h = history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).clone();
            assert_eq!(h, vec!["alpha", "beta", "gamma"]);
        }
        let _ = std::fs::remove_file(tmp);
    }

    #[test]
    fn read_history_file_missing_is_safe() {
        let _g = test_guard();
        reset_history();
        unsafe {
            let path = MbValue::from_ptr(MbObject::new_str(
                "/nonexistent/path/that/should/not/exist".into(),
            ));
            dispatch_read_history_file(&path as *const _, 1);
            assert_eq!(history().lock().unwrap_or_else(|e| { history().clear_poison(); e.into_inner() }).len(), 0);
        }
    }
}
