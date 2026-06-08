use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// logging module for Mamba (#400).
///
/// Provides: debug, info, warning, error, critical, getLogger, basicConfig
/// Outputs structured log lines to stderr with level prefix.
use std::collections::HashMap;

unsafe fn dispatch_unary(
    args_ptr: *const MbValue,
    nargs: usize,
    f: fn(MbValue) -> MbValue,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    f(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_debug(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_debug) }
}
unsafe extern "C" fn dispatch_info(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_info) }
}
unsafe extern "C" fn dispatch_warning(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_warning) }
}
unsafe extern "C" fn dispatch_error(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_error) }
}
unsafe extern "C" fn dispatch_critical(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_critical) }
}
unsafe extern "C" fn dispatch_getlogger(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    unsafe { dispatch_unary(args_ptr, nargs, mb_logging_getlogger) }
}
unsafe extern "C" fn dispatch_basicconfig(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // basicConfig(level=N) — keyword arg arrives as a trailing dict.
    let mut level = a.get(0).copied().unwrap_or_else(MbValue::none);
    for v in a.iter() {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let key = super::super::dict_ops::DictKey::Str("level".to_string());
                    if let Some(found) = g.get(&key) {
                        level = *found;
                        break;
                    }
                }
            }
        }
    }
    mb_logging_basicconfig(level)
}

/// Register the logging module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Log level constants
    attrs.insert("DEBUG".into(), MbValue::from_int(10));
    attrs.insert("INFO".into(), MbValue::from_int(20));
    attrs.insert("WARNING".into(), MbValue::from_int(30));
    attrs.insert("ERROR".into(), MbValue::from_int(40));
    attrs.insert("CRITICAL".into(), MbValue::from_int(50));

    let dispatchers: Vec<(&str, usize)> = vec![
        ("debug", dispatch_debug as *const () as usize),
        ("info", dispatch_info as *const () as usize),
        ("warning", dispatch_warning as *const () as usize),
        ("error", dispatch_error as *const () as usize),
        ("critical", dispatch_critical as *const () as usize),
        ("getLogger", dispatch_getlogger as *const () as usize),
        ("basicConfig", dispatch_basicconfig as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("logging", attrs);
}

// ── Helpers ──

fn extract_str(val: MbValue) -> String {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return s.clone();
            }
        }
    }
    if let Some(i) = val.as_int() {
        return format!("{i}");
    }
    if let Some(f) = val.as_float() {
        return format!("{f}");
    }
    if let Some(b) = val.as_bool() {
        return if b { "True" } else { "False" }.to_string();
    }
    if val.is_none() {
        return "None".to_string();
    }
    String::new()
}

use std::cell::Cell;
thread_local! {
    static LOG_LEVEL: Cell<i64> = const { Cell::new(30) }; // WARNING
}

fn log_at_level(level: &str, level_num: i64, msg: MbValue) {
    LOG_LEVEL.with(|l| {
        if level_num >= l.get() {
            eprintln!("{level}:{}", extract_str(msg));
        }
    });
}

// ── Runtime functions ──

pub fn mb_logging_debug(msg: MbValue) -> MbValue {
    log_at_level("DEBUG", 10, msg);
    MbValue::none()
}

pub fn mb_logging_info(msg: MbValue) -> MbValue {
    log_at_level("INFO", 20, msg);
    MbValue::none()
}

pub fn mb_logging_warning(msg: MbValue) -> MbValue {
    log_at_level("WARNING", 30, msg);
    MbValue::none()
}

pub fn mb_logging_error(msg: MbValue) -> MbValue {
    log_at_level("ERROR", 40, msg);
    MbValue::none()
}

pub fn mb_logging_critical(msg: MbValue) -> MbValue {
    log_at_level("CRITICAL", 50, msg);
    MbValue::none()
}

/// logging.getLogger(name) -> returns a dict with name field (stub)
pub fn mb_logging_getlogger(name: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    let n = if name.is_none() {
        "root".to_string()
    } else {
        extract_str(name)
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("name".into(), MbValue::from_ptr(MbObject::new_str(n)));
        }
    }
    MbValue::from_ptr(dict)
}

/// logging.basicConfig(level=...) -> set global log level
pub fn mb_logging_basicconfig(level: MbValue) -> MbValue {
    if let Some(l) = level.as_int() {
        LOG_LEVEL.with(|s| s.set(l));
    }
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_field(dict: MbValue, key: &str) -> Option<String> {
        dict.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                map.get(key).and_then(|v| v.as_ptr()).and_then(|p| {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
    }

    // --- extract_str ---
    #[test]
    fn test_extract_str_str_value() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(extract_str(s), "hello");
    }

    #[test]
    fn test_extract_str_int() {
        assert_eq!(extract_str(MbValue::from_int(42)), "42");
    }

    #[test]
    fn test_extract_str_float() {
        let s = extract_str(MbValue::from_float(3.14));
        assert!(s.starts_with("3.14"), "got: {s}");
    }

    #[test]
    fn test_extract_str_bool_true() {
        assert_eq!(extract_str(MbValue::from_bool(true)), "True");
    }

    #[test]
    fn test_extract_str_bool_false() {
        assert_eq!(extract_str(MbValue::from_bool(false)), "False");
    }

    #[test]
    fn test_extract_str_none() {
        assert_eq!(extract_str(MbValue::none()), "None");
    }

    #[test]
    fn test_extract_str_other_ptr() {
        // A non-Str pointer (list) returns empty string
        let v = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(extract_str(v), "");
    }

    // --- log level filter ---
    #[test]
    fn test_log_level_filter() {
        mb_logging_basicconfig(MbValue::from_int(10));
        let msg = MbValue::from_ptr(MbObject::new_str("test message".to_string()));
        mb_logging_debug(msg);
        mb_logging_info(msg);
        mb_logging_warning(msg);
    }

    #[test]
    fn test_log_suppressed_below_level() {
        // Set WARNING (30) then call debug (10) — suppressed, no panic
        mb_logging_basicconfig(MbValue::from_int(30));
        mb_logging_debug(MbValue::from_ptr(MbObject::new_str(
            "suppressed".to_string(),
        )));
    }

    #[test]
    fn test_log_error_and_critical() {
        mb_logging_basicconfig(MbValue::from_int(10));
        let msg = MbValue::from_ptr(MbObject::new_str("msg".to_string()));
        let r1 = mb_logging_error(msg);
        let r2 = mb_logging_critical(msg);
        assert!(r1.is_none());
        assert!(r2.is_none());
    }

    // --- getlogger ---
    #[test]
    fn test_getlogger_none_name() {
        let result = mb_logging_getlogger(MbValue::none());
        assert_eq!(str_field(result, "name"), Some("root".to_string()));
    }

    #[test]
    fn test_getlogger_str_name() {
        let name = MbValue::from_ptr(MbObject::new_str("mylogger".to_string()));
        let result = mb_logging_getlogger(name);
        assert_eq!(str_field(result, "name"), Some("mylogger".to_string()));
    }

    // --- basicconfig ---
    #[test]
    fn test_basicconfig_sets_level() {
        mb_logging_basicconfig(MbValue::from_int(10));
        // debug should emit (level 10 >= 10)
        let result = mb_logging_debug(MbValue::from_ptr(MbObject::new_str("x".to_string())));
        assert!(result.is_none());
        // restore
        mb_logging_basicconfig(MbValue::from_int(30));
    }

    #[test]
    fn test_basicconfig_non_int_noop() {
        // Store current level state — call with non-int, verify no panic
        mb_logging_basicconfig(MbValue::none());
    }
}
