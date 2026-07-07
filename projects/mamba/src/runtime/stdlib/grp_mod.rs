//! `grp` module for Mamba — the POSIX group account database.
///
/// In CPython, `grp` is a C module wrapping `getgrnam(3)`/`getgrgid(3)`/
/// `getgrent(3)` and returns `struct_group` records: a 4-field
/// tuple-like object indexable by position AND accessible by attribute
/// name (`gr_name`, `gr_passwd`, `gr_gid`, `gr_mem` — the last a `list`
/// of member-name strings). This module mirrors that surface using the
/// `_r` (reentrant) libc variants for thread safety (REQ: R2).
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

// getgrent()/setgrent()/endgrent() are process-global non-reentrant state —
// serialize getgrall() with a lock (matches CPython's own
// Py_BEGIN_ALLOW_THREADS + global lock discipline for this call).
static GRENT_LOCK: Mutex<()> = Mutex::new(());

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

/// Resolve a gid argument: CPython accepts any `int` (incl. bool, which is
/// an int subclass) and raises `TypeError` for anything else. A gid that is
/// syntactically an int but out of `gid_t` range is a lookup miss
/// (`KeyError`), not a type error.
enum GidArg {
    Value(i64),
    OutOfRange,
    WrongType,
}

fn extract_gid_arg(val: MbValue) -> GidArg {
    if let Some(i) = val.as_int_pyint() {
        // gid_t is u32 on the platforms mamba targets. CPython never wraps
        // an out-of-range int onto a real gid via modular truncation (see
        // the identical reasoning in pwd_mod.rs::extract_uid_arg) — any
        // value that doesn't fit losslessly in gid_t is "not found".
        return match u32::try_from(i) {
            Ok(_) => GidArg::Value(i),
            Err(_) => GidArg::OutOfRange,
        };
    }
    if let Some(ptr) = val.as_ptr() {
        if unsafe { matches!((*ptr).data, ObjData::BigInt(_)) } {
            return GidArg::OutOfRange;
        }
    }
    GidArg::WrongType
}

fn gr_bufsize() -> usize {
    let n = unsafe { libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) };
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

/// Walk a NULL-terminated `*mut *mut c_char` member-name array into a
/// Python `list` of `str`.
unsafe fn build_gr_mem(mut members: *mut *mut std::os::raw::c_char) -> MbValue {
    let mut out: Vec<MbValue> = Vec::new();
    if !members.is_null() {
        loop {
            let entry = *members;
            if entry.is_null() {
                break;
            }
            out.push(new_str(cstr_to_string(entry)));
            members = members.add(1);
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// Build a `struct_group` record: an `Instance` carrying both the named
/// fields (for attribute access) and an ordered `_entries` list (backing
/// the shared struct-sequence `__len__`/`__getitem__`/`__iter__` dunders
/// registered via `sys_mod::register_struct_seq_class`).
fn make_struct_group(gr: &libc::group) -> MbValue {
    let name = unsafe { cstr_to_string(gr.gr_name) };
    let passwd = unsafe { cstr_to_string(gr.gr_passwd) };
    let gid = gr.gr_gid as i64;
    let mem = unsafe { build_gr_mem(gr.gr_mem) };

    let v_name = new_str(name);
    let v_passwd = new_str(passwd);
    let v_gid = MbValue::from_int(gid);
    let v_mem = mem;

    let mut fields: FxHashMap<String, MbValue> = FxHashMap::default();
    fields.insert("gr_name".to_string(), v_name);
    fields.insert("gr_passwd".to_string(), v_passwd);
    fields.insert("gr_gid".to_string(), v_gid);
    fields.insert("gr_mem".to_string(), v_mem);

    let entries = vec![v_name, v_passwd, v_gid, v_mem];
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
            class_name: "struct_group".to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── grp.getgrgid(id) ──

pub fn mb_grp_getgrgid(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getgrgid() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    let gid = match extract_gid_arg(args[0]) {
        GidArg::Value(v) => v,
        GidArg::OutOfRange => {
            return raise_key_error("getgrgid(): gid not found".to_string());
        }
        GidArg::WrongType => {
            return raise_type_error(format!(
                "getgrgid() argument must be int, not {}",
                type_name_of(args[0])
            ));
        }
    };

    let gid_t = gid as u32;
    let bufsize = gr_bufsize();
    let mut buf: Vec<i8> = vec![0; bufsize];
    let mut gr_storage: libc::group = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::group = std::ptr::null_mut();

    let rc = unsafe {
        libc::getgrgid_r(
            gid_t,
            &mut gr_storage,
            buf.as_mut_ptr(),
            buf.len(),
            &mut result,
        )
    };

    if rc != 0 || result.is_null() {
        return raise_key_error(format!("getgrgid(): gid not found: {}", gid));
    }

    make_struct_group(&gr_storage)
}

// ── grp.getgrnam(name) ──

pub fn mb_grp_getgrnam(args: &[MbValue]) -> MbValue {
    if args.len() != 1 {
        return raise_type_error(format!(
            "getgrnam() takes exactly one argument ({} given)",
            args.len()
        ));
    }
    if !is_str(args[0]) {
        return raise_type_error(format!(
            "getgrnam() argument must be str, not {}",
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

    let bufsize = gr_bufsize();
    let mut buf: Vec<i8> = vec![0; bufsize];
    let mut gr_storage: libc::group = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::group = std::ptr::null_mut();

    let rc = unsafe {
        libc::getgrnam_r(
            c_name.as_ptr(),
            &mut gr_storage,
            buf.as_mut_ptr(),
            buf.len(),
            &mut result,
        )
    };

    if rc != 0 || result.is_null() {
        let repr_name = super::super::builtins::mb_repr(args[0]);
        let repr_str = extract_str_strict(repr_name).unwrap_or_else(|| format!("'{}'", name));
        return raise_key_error(format!("getgrnam(): name not found: {}", repr_str));
    }

    make_struct_group(&gr_storage)
}

// ── grp.getgrall() ──

pub fn mb_grp_getgrall(args: &[MbValue]) -> MbValue {
    if !args.is_empty() {
        return raise_type_error(format!(
            "getgrall() takes no arguments ({} given)",
            args.len()
        ));
    }

    let _guard = GRENT_LOCK.lock().unwrap();
    let mut out: Vec<MbValue> = Vec::new();
    unsafe {
        libc::setgrent();
        loop {
            let ent = libc::getgrent();
            if ent.is_null() {
                break;
            }
            out.push(make_struct_group(&*ent));
        }
        libc::endgrent();
    }

    MbValue::from_ptr(MbObject::new_list(out))
}

// ── Native-ABI dispatch wrappers ──

unsafe extern "C" fn dispatch_getgrgid(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_grp_getgrgid(a)
}

unsafe extern "C" fn dispatch_getgrnam(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_grp_getgrnam(a)
}

unsafe extern "C" fn dispatch_getgrall(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_grp_getgrall(a)
}

// ── Registration ──

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getgrgid", dispatch_getgrgid as *const () as usize),
        ("getgrnam", dispatch_getgrnam as *const () as usize),
        ("getgrall", dispatch_getgrall as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("grp", attrs);
    super::sys_mod::register_struct_seq_class("struct_group");
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ: R2
    #[test]
    fn test_register_does_not_panic() {
        register();
    }

    // REQ: R2
    #[test]
    fn test_getgrgid_current_group() {
        let gid = unsafe { libc::getgid() } as i64;
        let result = mb_grp_getgrgid(&[MbValue::from_int(gid)]);
        let ptr = result.as_ptr().expect("getgrgid should return an Instance");
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                assert!(fields.contains_key("gr_name"));
                assert_eq!(fields.get("gr_gid").copied(), Some(MbValue::from_int(gid)));
            } else {
                panic!("expected Instance");
            }
        }
    }

    // REQ: R2
    #[test]
    fn test_getgrgid_missing_raises_keyerror() {
        let _ = mb_grp_getgrgid(&[MbValue::from_int(i64::MAX - 1)]);
        let exc_type = super::super::super::exception::current_exception_type();
        assert_eq!(exc_type.as_deref(), Some("KeyError"));
        super::super::super::exception::clear_current_exception();
    }

    // REQ: R2
    #[test]
    fn test_getgrall_nonempty() {
        let result = mb_grp_getgrall(&[]);
        let ptr = result.as_ptr().expect("getgrall should return a list");
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                assert!(!lock.read().unwrap().is_empty());
            } else {
                panic!("expected List");
            }
        }
    }
}
