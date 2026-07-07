//! `pwd` module for Mamba — the POSIX user account database.
///
/// In CPython, `pwd` is a C module wrapping `getpwnam(3)`/`getpwuid(3)`/
/// `getpwent(3)` and returns `struct_passwd` records: a 7-field
/// tuple-like object indexable by position AND accessible by attribute
/// name (`pw_name`, `pw_passwd`, `pw_uid`, `pw_gid`, `pw_gecos`, `pw_dir`,
/// `pw_shell`). This module mirrors that surface using the `_r`
/// (reentrant) libc variants for thread safety (REQ: R1).
///
/// Dispatch wrappers use the native-ABI calling convention
/// (`extern "C" fn(*const MbValue, usize) -> MbValue`, addresses registered
/// in `NATIVE_FUNC_ADDRS`) — the convention actually used for module
/// attribute calls (see `os_mod.rs`/`time_mod.rs`); the `fn(MbValue) ->
/// MbValue` packed-list convention seen in `posix_mod.rs` does not receive
/// arguments correctly for `mod.func(arg)`-style calls.
use super::super::rc::{MbObject, MbObjectHeader, MbRwLock, ObjData, ObjKind};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::atomic::AtomicU32;
use std::sync::Mutex;

// getpwent()/setpwent()/endpwent() are process-global non-reentrant state —
// serialize getpwall() with a lock (matches CPython's own
// Py_BEGIN_ALLOW_THREADS + global lock discipline for this call).
static PWENT_LOCK: Mutex<()> = Mutex::new(());

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

fn raise_key_error(msg: impl Into<String>) -> MbValue {
    raise("KeyError", msg.into())
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

fn is_str(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
        .unwrap_or(false)
}

fn extract_str_strict(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Resolve a uid argument: CPython accepts any `int` (incl. bool, which is
/// an int subclass) and raises `TypeError` for anything else. A uid that is
/// syntactically an int but out of `uid_t` range is a lookup miss
/// (`KeyError`), not a type error.
enum UidArg {
    Value(i64),
    OutOfRange,
    WrongType,
}

fn extract_uid_arg(val: MbValue) -> UidArg {
    if let Some(i) = val.as_int_pyint() {
        // uid_t is u32 on the platforms mamba targets. CPython never wraps
        // an out-of-range int onto a real uid via modular truncation — any
        // value that doesn't fit losslessly in uid_t is simply "not found"
        // (verified against the oracle: getpwuid(2**63-2) -> KeyError, not
        // a wrapped match on a real low-numbered account).
        return match u32::try_from(i) {
            Ok(_) => UidArg::Value(i),
            Err(_) => UidArg::OutOfRange,
        };
    }
    if let Some(ptr) = val.as_ptr() {
        if unsafe { matches!((*ptr).data, ObjData::BigInt(_)) } {
            return UidArg::OutOfRange;
        }
    }
    UidArg::WrongType
}

fn pw_bufsize() -> usize {
    let n = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    if n > 0 {
        n as usize
    } else {
        16384
    }
}

unsafe fn cstr_to_string(ptr: *mut std::os::raw::c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// Build a `struct_passwd` record: an `Instance` carrying both the named
/// fields (for attribute access) and an ordered `_entries` list (backing
/// the shared struct-sequence `__len__`/`__getitem__`/`__iter__` dunders
/// registered via `sys_mod::register_struct_seq_class`).
fn make_struct_passwd(pw: &libc::passwd) -> MbValue {
    let name = unsafe { cstr_to_string(pw.pw_name) };
    let passwd = unsafe { cstr_to_string(pw.pw_passwd) };
    let uid = pw.pw_uid as i64;
    let gid = pw.pw_gid as i64;
    let gecos = unsafe { cstr_to_string(pw.pw_gecos) };
    let dir = unsafe { cstr_to_string(pw.pw_dir) };
    let shell = unsafe { cstr_to_string(pw.pw_shell) };

    let v_name = new_str(name);
    let v_passwd = new_str(passwd);
    let v_uid = MbValue::from_int(uid);
    let v_gid = MbValue::from_int(gid);
    let v_gecos = new_str(gecos);
    let v_dir = new_str(dir);
    let v_shell = new_str(shell);

    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert("pw_name".to_string(), v_name);
    fields.insert("pw_passwd".to_string(), v_passwd);
    fields.insert("pw_uid".to_string(), v_uid);
    fields.insert("pw_gid".to_string(), v_gid);
    fields.insert("pw_gecos".to_string(), v_gecos);
    fields.insert("pw_dir".to_string(), v_dir);
    fields.insert("pw_shell".to_string(), v_shell);

    let entries = vec![v_name, v_passwd, v_uid, v_gid, v_gecos, v_dir, v_shell];
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
            class_name: "struct_passwd".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── pwd.getpwuid(uid) ──

pub fn mb_pwd_getpwuid(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getpwuid() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    let uid = match extract_uid_arg(args[0]) {
        UidArg::Value(v) => v,
        UidArg::OutOfRange => {
            return raise_key_error("getpwuid(): uid not found".to_string());
        }
        UidArg::WrongType => {
            return raise_type_error(format!(
                "getpwuid() argument must be int, not {}",
                type_name_of(args[0])
            ));
        }
    };

    let uid_t = uid as u32;
    let bufsize = pw_bufsize();
    let mut buf: Vec<i8> = vec![0; bufsize];
    let mut pwd_storage: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();

    let rc = unsafe {
        libc::getpwuid_r(
            uid_t,
            &mut pwd_storage,
            buf.as_mut_ptr(),
            buf.len(),
            &mut result,
        )
    };

    if rc != 0 || result.is_null() {
        return raise_key_error(format!("getpwuid(): uid not found: {}", uid));
    }

    make_struct_passwd(&pwd_storage)
}

// ── pwd.getpwnam(name) ──

pub fn mb_pwd_getpwnam(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getpwnam() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    if !is_str(args[0]) {
        return raise_type_error(format!(
            "getpwnam() argument must be str, not {}",
            type_name_of(args[0])
        ));
    }
    let name = extract_str_strict(args[0]).unwrap_or_default();
    let c_name = match CString::new(name.clone()) {
        Ok(c) => c,
        Err(_) => {
            return raise_value_error("embedded null byte");
        }
    };

    let bufsize = pw_bufsize();
    let mut buf: Vec<i8> = vec![0; bufsize];
    let mut pwd_storage: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = std::ptr::null_mut();

    let rc = unsafe {
        libc::getpwnam_r(
            c_name.as_ptr(),
            &mut pwd_storage,
            buf.as_mut_ptr(),
            buf.len(),
            &mut result,
        )
    };

    if rc != 0 || result.is_null() {
        let repr_name = super::super::builtins::mb_repr(args[0]);
        let repr_str = extract_str_strict(repr_name).unwrap_or_else(|| format!("'{}'", name));
        return raise_key_error(format!("getpwnam(): name not found: {}", repr_str));
    }

    make_struct_passwd(&pwd_storage)
}

// ── pwd.getpwall() ──

pub fn mb_pwd_getpwall(args: &[MbValue]) -> MbValue {
    if !args.is_empty() {
        return raise_type_error(format!(
            "getpwall() takes no arguments ({} given)",
            args.len()
        ));
    }

    let _guard = PWENT_LOCK.lock().unwrap();
    let mut out: Vec<MbValue> = Vec::new();
    unsafe {
        libc::setpwent();
        loop {
            let ent = libc::getpwent();
            if ent.is_null() {
                break;
            }
            out.push(make_struct_passwd(&*ent));
        }
        libc::endpwent();
    }

    MbValue::from_ptr(MbObject::new_list(out))
}

// ── Native-ABI dispatch wrappers ──

unsafe extern "C" fn dispatch_getpwuid(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pwd_getpwuid(a)
}

unsafe extern "C" fn dispatch_getpwnam(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pwd_getpwnam(a)
}

unsafe extern "C" fn dispatch_getpwall(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pwd_getpwall(a)
}

// ── Registration ──

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getpwuid", dispatch_getpwuid as *const () as usize),
        ("getpwnam", dispatch_getpwnam as *const () as usize),
        ("getpwall", dispatch_getpwall as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("pwd", attrs);
    super::sys_mod::register_struct_seq_class("struct_passwd");
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
    fn test_getpwuid_current_user() {
        let uid = unsafe { libc::getuid() } as i64;
        let result = mb_pwd_getpwuid(&[MbValue::from_int(uid)]);
        let ptr = result.as_ptr().expect("getpwuid should return an Instance");
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                assert!(fields.contains_key("pw_name"));
                assert_eq!(fields.get("pw_uid").copied(), Some(MbValue::from_int(uid)));
            } else {
                panic!("expected Instance");
            }
        }
    }

    // REQ: R1
    #[test]
    fn test_getpwuid_missing_raises_keyerror() {
        let _ = mb_pwd_getpwuid(&[MbValue::from_int(i64::MAX - 1)]);
        let exc_type = super::super::super::exception::current_exception_type();
        assert_eq!(exc_type.as_deref(), Some("KeyError"));
        super::super::super::exception::clear_current_exception();
    }

    // REQ: R1
    #[test]
    fn test_getpwall_nonempty() {
        let result = mb_pwd_getpwall(&[]);
        let ptr = result.as_ptr().expect("getpwall should return a list");
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                assert!(!lock.read().unwrap().is_empty());
            } else {
                panic!("expected List");
            }
        }
    }
}
