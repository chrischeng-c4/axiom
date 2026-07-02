//! `resource` module for Mamba — POSIX process resource usage/limits (#873).
///
/// In CPython, `resource` is a C module wrapping `getrusage(2)` /
/// `getrlimit(2)` / `setrlimit(2)`. `getrusage()` returns a `struct_rusage`
/// record: a 16-field tuple-like object indexable by position AND
/// accessible by attribute name (`ru_utime`, `ru_stime`, `ru_maxrss`, ...).
/// This module mirrors that surface via the `libc` crate (REQ: R1-R3),
/// following the `pwd_mod.rs` struct-record + libc-wrapper pattern.
///
/// Dispatch wrappers use the native-ABI calling convention
/// (`extern "C" fn(*const MbValue, usize) -> MbValue`, addresses registered
/// in `NATIVE_FUNC_ADDRS`) — the convention actually used for module
/// attribute calls (see `os_mod.rs`/`time_mod.rs`/`pwd_mod.rs`); the
/// `fn(MbValue) -> MbValue` packed-list convention seen in `posix_mod.rs`
/// does not receive arguments correctly for `mod.func(arg)`-style calls.
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind, MbRwLock};
use super::super::value::MbValue;
use num_traits::ToPrimitive;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// ── Small local helpers (mirrors the convention duplicated across stdlib/*) ──

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn raise(exc_type: &str, msg: String) -> MbValue {
    super::super::exception::mb_raise(new_str(exc_type), new_str(msg));
    MbValue::none()
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    raise("TypeError", msg.into())
}

fn raise_value_error(msg: impl Into<String>) -> MbValue {
    raise("ValueError", msg.into())
}

fn raise_overflow_error(msg: impl Into<String>) -> MbValue {
    raise("OverflowError", msg.into())
}

fn raise_os_error_errno(err: &std::io::Error) -> MbValue {
    let errno = err.raw_os_error().unwrap_or(0);
    raise("OSError", format!("[Errno {}] {}", errno, err))
}

fn type_name_of(val: MbValue) -> &'static str {
    if val.is_none() {
        "NoneType"
    } else if val.is_bool() {
        "bool"
    } else if val.is_int() {
        "int"
    } else if val.as_float().is_some() {
        "float"
    } else {
        match val.as_ptr() {
            Some(ptr) => unsafe {
                match &(*ptr).data {
                    ObjData::Str(_) => "str",
                    ObjData::List(_) => "list",
                    ObjData::Dict(_) => "dict",
                    ObjData::Tuple(_) => "tuple",
                    ObjData::BigInt(_) => "int",
                    ObjData::Instance { class_name, .. } => class_name.as_str(),
                    _ => "object",
                }
            },
            None => "object",
        }
    }
}

fn is_bigint(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::BigInt(_)) })
        .unwrap_or(false)
}

/// Convert a Python-int argument to a C `int`, matching CPython's
/// `PyArg_ParseTuple(..., "i", ...)` contract: fixnums out of `c_int` range
/// and heap `BigInt`s raise `OverflowError`; non-ints raise `TypeError`.
fn extract_c_int_arg(val: MbValue) -> Result<i32, MbValue> {
    if let Some(i) = val.as_int_pyint() {
        if i < i32::MIN as i64 || i > i32::MAX as i64 {
            return Err(raise_overflow_error(
                "Python int too large to convert to C int",
            ));
        }
        return Ok(i as i32);
    }
    if is_bigint(val) {
        return Err(raise_overflow_error(
            "Python int too large to convert to C int",
        ));
    }
    Err(raise_type_error(format!(
        "'{}' object cannot be interpreted as an integer",
        type_name_of(val)
    )))
}

/// Resolve a `setrlimit()` cur/max argument: any Python int that fits in a
/// signed 64-bit value converts to the `rlim_t` (u64) via a two's-complement
/// reinterpret — matching the observed oracle contract where the kernel
/// itself clamps out-of-range values at the `setrlimit(2)` syscall (e.g.
/// `-1` round-trips through the syscall to `RLIM_INFINITY`). Ints that don't
/// fit in `i64` at all (e.g. `10**50`) are a genuine `OverflowError`.
enum RlimArg {
    Value(u64),
    Overflow,
    WrongType,
}

fn extract_rlim_arg(val: MbValue) -> RlimArg {
    if let Some(i) = val.as_int_pyint() {
        return RlimArg::Value(i as u64);
    }
    match unsafe { super::super::bigint_ops::extract_bigint(val) } {
        Some(big) => match big.to_i64() {
            Some(i) => RlimArg::Value(i as u64),
            None => RlimArg::Overflow,
        },
        None => RlimArg::WrongType,
    }
}

/// `setrlimit()`'s `(soft, hard)` argument accepts any 2-item list/tuple
/// (CPython parses it via `PyArg_ParseTuple`, which works over the generic
/// sequence protocol, not just literal tuples).
fn extract_limits_pair(val: MbValue) -> Option<(MbValue, MbValue)> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) if items.len() == 2 => Some((items[0], items[1])),
            ObjData::List(lock) => {
                let items = lock.read().unwrap();
                if items.len() == 2 {
                    Some((items[0], items[1]))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Build a `struct_rusage` record: an `Instance` carrying both the named
/// fields (for attribute access) and an ordered `_entries` list (backing
/// the shared struct-sequence `__len__`/`__getitem__`/`__iter__` dunders
/// registered via `sys_mod::register_struct_seq_class`).
fn make_struct_rusage(ru: &libc::rusage) -> MbValue {
    let ru_utime = ru.ru_utime.tv_sec as f64 + ru.ru_utime.tv_usec as f64 / 1_000_000.0;
    let ru_stime = ru.ru_stime.tv_sec as f64 + ru.ru_stime.tv_usec as f64 / 1_000_000.0;

    let field_values: [(&str, MbValue); 16] = [
        ("ru_utime", MbValue::from_float(ru_utime)),
        ("ru_stime", MbValue::from_float(ru_stime)),
        (
            "ru_maxrss",
            super::super::bigint_ops::int_from_i64(ru.ru_maxrss as i64),
        ),
        (
            "ru_ixrss",
            super::super::bigint_ops::int_from_i64(ru.ru_ixrss as i64),
        ),
        (
            "ru_idrss",
            super::super::bigint_ops::int_from_i64(ru.ru_idrss as i64),
        ),
        (
            "ru_isrss",
            super::super::bigint_ops::int_from_i64(ru.ru_isrss as i64),
        ),
        (
            "ru_minflt",
            super::super::bigint_ops::int_from_i64(ru.ru_minflt as i64),
        ),
        (
            "ru_majflt",
            super::super::bigint_ops::int_from_i64(ru.ru_majflt as i64),
        ),
        (
            "ru_nswap",
            super::super::bigint_ops::int_from_i64(ru.ru_nswap as i64),
        ),
        (
            "ru_inblock",
            super::super::bigint_ops::int_from_i64(ru.ru_inblock as i64),
        ),
        (
            "ru_oublock",
            super::super::bigint_ops::int_from_i64(ru.ru_oublock as i64),
        ),
        (
            "ru_msgsnd",
            super::super::bigint_ops::int_from_i64(ru.ru_msgsnd as i64),
        ),
        (
            "ru_msgrcv",
            super::super::bigint_ops::int_from_i64(ru.ru_msgrcv as i64),
        ),
        (
            "ru_nsignals",
            super::super::bigint_ops::int_from_i64(ru.ru_nsignals as i64),
        ),
        (
            "ru_nvcsw",
            super::super::bigint_ops::int_from_i64(ru.ru_nvcsw as i64),
        ),
        (
            "ru_nivcsw",
            super::super::bigint_ops::int_from_i64(ru.ru_nivcsw as i64),
        ),
    ];

    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    let mut entries: Vec<MbValue> = Vec::with_capacity(field_values.len());
    for (name, val) in field_values {
        fields.insert(name.to_string(), val);
        entries.push(val);
    }
    fields.insert(
        "_entries".to_string(),
        MbValue::from_ptr(MbObject::new_list(entries)),
    );

    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "struct_rusage".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── resource.getrusage(who) ──

pub fn mb_resource_getrusage(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getrusage() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    let who = match extract_c_int_arg(args[0]) {
        Ok(w) => w,
        Err(e) => return e,
    };

    let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::getrusage(who, &mut ru) };
    if rc == -1 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINVAL) {
            return raise_value_error("invalid who parameter");
        }
        return raise_os_error_errno(&err);
    }

    make_struct_rusage(&ru)
}

// ── resource.getrlimit(resource) ──

pub fn mb_resource_getrlimit(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getrlimit() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    let resource_id = match extract_c_int_arg(args[0]) {
        Ok(r) => r,
        Err(e) => return e,
    };
    if resource_id < 0 || resource_id >= libc::RLIM_NLIMITS {
        return raise_value_error("invalid resource specified");
    }

    let mut rl: libc::rlimit = unsafe { std::mem::zeroed() };
    let rc = unsafe { libc::getrlimit(resource_id, &mut rl) };
    if rc == -1 {
        return raise_os_error_errno(&std::io::Error::last_os_error());
    }

    let tup = MbObject::new_tuple(vec![
        super::super::bigint_ops::int_from_i64(rl.rlim_cur as i64),
        super::super::bigint_ops::int_from_i64(rl.rlim_max as i64),
    ]);
    MbValue::from_ptr(tup)
}

// ── resource.setrlimit(resource, (soft, hard)) ──

pub fn mb_resource_setrlimit(args: &[MbValue]) -> MbValue {
    if args.len() != 2 {
        return raise_type_error(format!(
            "setrlimit() takes exactly 2 arguments ({} given)",
            args.len()
        ));
    }
    let resource_id = match extract_c_int_arg(args[0]) {
        Ok(r) => r,
        Err(e) => return e,
    };
    if resource_id < 0 || resource_id >= libc::RLIM_NLIMITS {
        return raise_value_error("invalid resource specified");
    }

    let Some((soft_v, hard_v)) = extract_limits_pair(args[1]) else {
        return raise_value_error("expected a tuple of 2 integers");
    };

    let soft = match extract_rlim_arg(soft_v) {
        RlimArg::Value(v) => v,
        RlimArg::Overflow => {
            return raise_overflow_error("Python int too large to convert to C long")
        }
        RlimArg::WrongType => {
            return raise_type_error(format!(
                "'{}' object cannot be interpreted as an integer",
                type_name_of(soft_v)
            ));
        }
    };
    let hard = match extract_rlim_arg(hard_v) {
        RlimArg::Value(v) => v,
        RlimArg::Overflow => {
            return raise_overflow_error("Python int too large to convert to C long")
        }
        RlimArg::WrongType => {
            return raise_type_error(format!(
                "'{}' object cannot be interpreted as an integer",
                type_name_of(hard_v)
            ));
        }
    };

    let rl = libc::rlimit {
        rlim_cur: soft,
        rlim_max: hard,
    };
    let rc = unsafe { libc::setrlimit(resource_id, &rl) };
    if rc == -1 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::EINVAL) {
            return raise_value_error("current limit exceeds maximum limit");
        }
        return raise_os_error_errno(&err);
    }
    MbValue::none()
}

// ── resource.getpagesize() ──

pub fn mb_resource_getpagesize(args: &[MbValue]) -> MbValue {
    if !args.is_empty() {
        return raise_type_error(format!(
            "getpagesize() takes no arguments ({} given)",
            args.len()
        ));
    }
    // `getpagesize(3)` isn't exposed by libc on every target (notably
    // macOS/apple) — `sysconf(_SC_PAGESIZE)` is the portable POSIX
    // equivalent and matches CPython's own implementation.
    let sz = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    MbValue::from_int(if sz > 0 { sz as i64 } else { 4096 })
}

// ── Native-ABI dispatch wrappers ──

unsafe extern "C" fn dispatch_getrusage(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_resource_getrusage(a)
}

unsafe extern "C" fn dispatch_getrlimit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_resource_getrlimit(a)
}

unsafe extern "C" fn dispatch_setrlimit(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_resource_setrlimit(a)
}

unsafe extern "C" fn dispatch_getpagesize(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_resource_getpagesize(a)
}

// ── Registration ──

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getrusage", dispatch_getrusage as *const () as usize),
        ("getrlimit", dispatch_getrlimit as *const () as usize),
        ("setrlimit", dispatch_setrlimit as *const () as usize),
        ("getpagesize", dispatch_getpagesize as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    attrs.insert(
        "RUSAGE_SELF".into(),
        MbValue::from_int(libc::RUSAGE_SELF as i64),
    );
    attrs.insert(
        "RUSAGE_CHILDREN".into(),
        MbValue::from_int(libc::RUSAGE_CHILDREN as i64),
    );
    attrs.insert(
        "RLIM_INFINITY".into(),
        super::super::bigint_ops::int_from_i64(libc::RLIM_INFINITY as i64),
    );

    attrs.insert(
        "RLIMIT_CPU".into(),
        MbValue::from_int(libc::RLIMIT_CPU as i64),
    );
    attrs.insert(
        "RLIMIT_FSIZE".into(),
        MbValue::from_int(libc::RLIMIT_FSIZE as i64),
    );
    attrs.insert(
        "RLIMIT_DATA".into(),
        MbValue::from_int(libc::RLIMIT_DATA as i64),
    );
    attrs.insert(
        "RLIMIT_STACK".into(),
        MbValue::from_int(libc::RLIMIT_STACK as i64),
    );
    attrs.insert(
        "RLIMIT_CORE".into(),
        MbValue::from_int(libc::RLIMIT_CORE as i64),
    );
    attrs.insert(
        "RLIMIT_AS".into(),
        MbValue::from_int(libc::RLIMIT_AS as i64),
    );
    attrs.insert(
        "RLIMIT_RSS".into(),
        MbValue::from_int(libc::RLIMIT_RSS as i64),
    );
    attrs.insert(
        "RLIMIT_MEMLOCK".into(),
        MbValue::from_int(libc::RLIMIT_MEMLOCK as i64),
    );
    attrs.insert(
        "RLIMIT_NPROC".into(),
        MbValue::from_int(libc::RLIMIT_NPROC as i64),
    );
    attrs.insert(
        "RLIMIT_NOFILE".into(),
        MbValue::from_int(libc::RLIMIT_NOFILE as i64),
    );

    // Linux-only resource kinds — cfg-gated since they don't exist in the
    // libc crate's macOS/BSD constant set (mirrors fcntl_mod.rs's
    // F_GETPIPE_SZ/F_SETPIPE_SZ gating).
    #[cfg(target_os = "linux")]
    {
        attrs.insert(
            "RLIMIT_LOCKS".into(),
            MbValue::from_int(libc::RLIMIT_LOCKS as i64),
        );
        attrs.insert(
            "RLIMIT_SIGPENDING".into(),
            MbValue::from_int(libc::RLIMIT_SIGPENDING as i64),
        );
        attrs.insert(
            "RLIMIT_MSGQUEUE".into(),
            MbValue::from_int(libc::RLIMIT_MSGQUEUE as i64),
        );
        attrs.insert(
            "RLIMIT_NICE".into(),
            MbValue::from_int(libc::RLIMIT_NICE as i64),
        );
        attrs.insert(
            "RLIMIT_RTPRIO".into(),
            MbValue::from_int(libc::RLIMIT_RTPRIO as i64),
        );
        attrs.insert(
            "RLIMIT_RTTIME".into(),
            MbValue::from_int(libc::RLIMIT_RTTIME as i64),
        );
    }

    // resource.error is an alias for the builtin OSError (matches the
    // os.error/select.error/mmap.error convention elsewhere in stdlib/*).
    attrs.insert("error".to_string(), new_str("OSError"));

    super::register_module("resource", attrs);
    super::sys_mod::register_struct_seq_class("struct_rusage");
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R1
    #[test]
    fn test_register_does_not_panic() {
        register();
    }

    // REQ: R1
    #[test]
    fn test_getrusage_self_has_16_fields() {
        let result = mb_resource_getrusage(&[MbValue::from_int(libc::RUSAGE_SELF as i64)]);
        let ptr = result.as_ptr().expect("getrusage should return an Instance");
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                assert!(fields.contains_key("ru_utime"));
                assert!(fields.contains_key("ru_maxrss"));
                let entries = fields.get("_entries").expect("_entries present");
                let list_ptr = entries.as_ptr().expect("_entries is a list");
                if let ObjData::List(ref lock) = (*list_ptr).data {
                    assert_eq!(lock.read().unwrap().len(), 16);
                } else {
                    panic!("expected List");
                }
            } else {
                panic!("expected Instance");
            }
        }
    }

    // REQ: R2
    #[test]
    fn test_getrlimit_invalid_resource_raises_valueerror() {
        let _ = mb_resource_getrlimit(&[MbValue::from_int(999)]);
        let exc_type = super::super::super::exception::current_exception_type();
        assert_eq!(exc_type.as_deref(), Some("ValueError"));
        super::super::super::exception::clear_current_exception();
    }

    // REQ: R2
    #[test]
    fn test_getrlimit_setrlimit_round_trip_nofile() {
        let resid = MbValue::from_int(libc::RLIMIT_NOFILE as i64);
        let before = mb_resource_getrlimit(&[resid]);
        let before_ptr = before.as_ptr().expect("tuple");
        let (soft, hard) = unsafe {
            if let ObjData::Tuple(ref items) = (*before_ptr).data {
                (items[0], items[1])
            } else {
                panic!("expected tuple");
            }
        };
        let soft_i = soft.as_int_pyint().or_else(|| {
            unsafe { super::super::super::bigint_ops::extract_bigint(soft) }
                .and_then(|b| b.to_i64())
        });
        let Some(soft_i) = soft_i else {
            panic!("soft limit should be representable as i64");
        };
        // Lower the soft limit by 1, then restore it exactly.
        let lowered = MbObject::new_tuple(vec![MbValue::from_int(soft_i - 1), hard]);
        let _ = mb_resource_setrlimit(&[resid, MbValue::from_ptr(lowered)]);
        assert_eq!(
            super::super::super::exception::current_exception_type(),
            None
        );

        let restored = MbObject::new_tuple(vec![soft, hard]);
        let _ = mb_resource_setrlimit(&[resid, MbValue::from_ptr(restored)]);
        assert_eq!(
            super::super::super::exception::current_exception_type(),
            None
        );
    }

    // REQ: R3
    #[test]
    fn test_rusage_self_children_constants() {
        assert_eq!(libc::RUSAGE_SELF, 0);
        assert_eq!(libc::RUSAGE_CHILDREN, -1);
    }
}
