//! ctypes module for Mamba (#875).
//!
//! Replaces the old `long_tail3_mod::register_ctypes()` empty-dict shells
//! with a real, self-contained FFI substrate:
//!
//!   * `CDLL(name_or_None)` — a genuine `libc::dlopen` (None → the global/
//!     main-program symbol table via `dlopen(NULL, RTLD_NOW)`).
//!   * Attribute access on a `CDLL` instance (`lib.abs`) — a genuine
//!     `libc::dlsym` lookup, returning a callable `CFuncPtr` instance.
//!   * `CFuncPtr.__call__` — real call marshalling for the scalar types
//!     required by the issue (`c_int`/`c_uint`/`c_long`/`c_ulong`/
//!     `c_double`/`c_char_p`/`c_void_p`/`c_size_t`), honoring `.argtypes`/
//!     `.restype` (default: C `int`). Hand-rolled AArch64 trampolines
//!     (`transmute` a raw address to the right `extern "C" fn` shape and
//!     call it) stand in for a general libffi — the workspace has no
//!     `libffi` crate, and the issue explicitly sanctions this approach for
//!     up to ~6 scalar args.
//!   * `byref()` / `POINTER()` — minimal out-param support: `byref(x)`
//!     allocates an 8-byte scratch buffer seeded from `x.value`, passes its
//!     address, and writes the (possibly-mutated) buffer back into `x.value`
//!     after the call.
//!   * `sizeof()` for the scalar types.
//!   * `ctypes.ArgumentError` exposed as an unregistered type-object (like
//!     `argparse.ArgumentError`), matched by bare type-name string so
//!     `except ctypes.ArgumentError` still works without colliding with
//!     argparse's own same-named registered class in the global
//!     CLASS_REGISTRY flat namespace.
//!
//! Everything else that was already a shell before this change (Structure/
//! Union ABI, Array element access, CFUNCTYPE/callbacks, the Windows-only
//! loaders, low-level buffer/cast primitives) stays a shell — those are
//! explicit follow-ups per the issue, not in scope here.
use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

// ── Small field/value helpers (mirrors argparse_mod's local copies) ──

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Bytes(ref b) = (*ptr).data {
            Some(b.clone())
        } else {
            None
        }
    })
}

fn instance_class_name(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

fn extract_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg));
    MbValue::none()
}

fn raise_argument_error(idx: usize, msg: &str) {
    super::super::exception::mb_raise(
        new_str("ArgumentError"),
        new_str(&format!("argument {}: {}", idx + 1, msg)),
    );
}

fn clamp48(v: i64) -> i64 {
    let max = (1i64 << 47) - 1;
    let min = -(1i64 << 47);
    v.clamp(min, max)
}

// ── Native pointer side-table ──
//
// Raw dlopen handles / dlsym'd function addresses are real pointers that can
// exceed mamba's 48-bit tagged-int range, and MUST stay bit-exact (they get
// called through / passed to dlsym). Instances can only carry `MbValue`
// fields, so we stash the actual `usize` in a side table and store just the
// small integer index on the instance.
thread_local! {
    static NATIVE_PTRS: std::cell::RefCell<Vec<usize>> = std::cell::RefCell::new(Vec::new());
}

fn store_ptr(p: usize) -> usize {
    NATIVE_PTRS.with(|v| {
        let mut v = v.borrow_mut();
        v.push(p);
        v.len() - 1
    })
}

fn load_ptr(id: usize) -> usize {
    NATIVE_PTRS.with(|v| v.borrow().get(id).copied().unwrap_or(0))
}

// ── Legacy shell dispatchers (kept for names that stay out of scope) ──

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}
unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_empty_str(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}
unsafe extern "C" fn dispatch_int_zero(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

extern "C" fn ctypes_array_getitem(_self_v: MbValue, _args: MbValue) -> MbValue {
    raise_type_error("indices must be integers")
}
extern "C" fn ctypes_array_setitem(_self_v: MbValue, _args: MbValue) -> MbValue {
    raise_type_error("indices must be integer")
}

unsafe extern "C" fn ctypes_array_getitem_direct(_a: *const MbValue, _n: usize) -> MbValue {
    raise_type_error("indices must be integers")
}
unsafe extern "C" fn ctypes_array_setitem_direct(_a: *const MbValue, _n: usize) -> MbValue {
    raise_type_error("indices must be integer")
}
unsafe extern "C" fn ctypes_alignment_direct(_a: *const MbValue, _n: usize) -> MbValue {
    raise_type_error("no alignment info")
}
unsafe extern "C" fn ctypes_buffer_info_direct(_a: *const MbValue, _n: usize) -> MbValue {
    raise_type_error("not a ctypes object")
}
unsafe extern "C" fn ctypes_py_incref_direct(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn ctypes_py_decref_direct(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

fn make_ctypes_callable_shell(name: &str, module: &str, methods: &[(&str, usize)]) -> MbValue {
    let obj = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*obj).data {
            let mut map = lock.write().unwrap();
            map.insert(
                DictKey::from("__name__"),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            map.insert(
                DictKey::from("__qualname__"),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            map.insert(
                DictKey::from("__module__"),
                MbValue::from_ptr(MbObject::new_str(module.to_string())),
            );
            for (method, addr) in methods {
                map.insert(DictKey::from(*method), MbValue::from_func(*addr));
            }
        }
    }
    MbValue::from_ptr(obj)
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

fn register_addrs(addrs: &[usize]) {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for a in addrs {
            set.insert(*a as u64);
        }
    });
}

// ── Trampolines ──
//
// A hand-rolled stand-in for libffi (absent from the workspace lock).
// Each trampoline just `transmute`s the raw dlsym'd address to the
// `extern "C" fn` type the real call site needs, then calls it — letting
// rustc/LLVM emit the correct AAPCS64 register placement. This only
// supports HOMOGENEOUS argument lists (all int-class registers, or all
// float-class registers) — a call that mixes int and float args in one
// invocation is rejected with a clear error (see `cfuncptr_call`); none of
// the issue's required verification scenarios need a mixed call.
macro_rules! def_int_call {
    ($name:ident, $ret:ty $(, $p:ident)*) => {
        #[allow(clippy::too_many_arguments)]
        unsafe fn $name(addr: usize $(, $p: i64)*) -> $ret {
            let f: extern "C" fn($(def_int_call!(@unit $p)),*) -> $ret =
                unsafe { std::mem::transmute(addr) };
            f($($p),*)
        }
    };
    (@unit $p:ident) => { i64 };
}
macro_rules! def_float_call {
    ($name:ident, $ret:ty $(, $p:ident)*) => {
        #[allow(clippy::too_many_arguments)]
        unsafe fn $name(addr: usize $(, $p: f64)*) -> $ret {
            let f: extern "C" fn($(def_float_call!(@unit $p)),*) -> $ret =
                unsafe { std::mem::transmute(addr) };
            f($($p),*)
        }
    };
    (@unit $p:ident) => { f64 };
}

def_int_call!(call_i0, i32);
def_int_call!(call_i1, i32, a0);
def_int_call!(call_i2, i32, a0, a1);
def_int_call!(call_i3, i32, a0, a1, a2);
def_int_call!(call_i4, i32, a0, a1, a2, a3);
def_int_call!(call_i5, i32, a0, a1, a2, a3, a4);
def_int_call!(call_i6, i32, a0, a1, a2, a3, a4, a5);

def_int_call!(call_i0_64, i64);
def_int_call!(call_i1_64, i64, a0);
def_int_call!(call_i2_64, i64, a0, a1);
def_int_call!(call_i3_64, i64, a0, a1, a2);
def_int_call!(call_i4_64, i64, a0, a1, a2, a3);
def_int_call!(call_i5_64, i64, a0, a1, a2, a3, a4);
def_int_call!(call_i6_64, i64, a0, a1, a2, a3, a4, a5);

def_int_call!(call_i0_d, f64);
def_int_call!(call_i1_d, f64, a0);
def_int_call!(call_i2_d, f64, a0, a1);
def_int_call!(call_i3_d, f64, a0, a1, a2);
def_int_call!(call_i4_d, f64, a0, a1, a2, a3);
def_int_call!(call_i5_d, f64, a0, a1, a2, a3, a4);
def_int_call!(call_i6_d, f64, a0, a1, a2, a3, a4, a5);

def_float_call!(call_d0, f64);
def_float_call!(call_d1, f64, a0);
def_float_call!(call_d2, f64, a0, a1);
def_float_call!(call_d3, f64, a0, a1, a2);
def_float_call!(call_d4, f64, a0, a1, a2, a3);

def_float_call!(call_d0_i, i32);
def_float_call!(call_d1_i, i32, a0);
def_float_call!(call_d2_i, i32, a0, a1);
def_float_call!(call_d3_i, i32, a0, a1, a2);
def_float_call!(call_d4_i, i32, a0, a1, a2, a3);

def_float_call!(call_d0_i64, i64);
def_float_call!(call_d1_i64, i64, a0);
def_float_call!(call_d2_i64, i64, a0, a1);
def_float_call!(call_d3_i64, i64, a0, a1, a2);
def_float_call!(call_d4_i64, i64, a0, a1, a2, a3);

#[derive(Clone, Copy)]
enum RetShape {
    I32,
    I64,
    F64,
}

#[derive(Clone, Copy)]
enum RawResult {
    I32(i32),
    I64(i64),
    F64(f64),
}

fn call_int_args(addr: usize, args: &[i64], shape: RetShape) -> Result<RawResult, ()> {
    unsafe {
        Ok(match shape {
            RetShape::I32 => RawResult::I32(match args.len() {
                0 => call_i0(addr),
                1 => call_i1(addr, args[0]),
                2 => call_i2(addr, args[0], args[1]),
                3 => call_i3(addr, args[0], args[1], args[2]),
                4 => call_i4(addr, args[0], args[1], args[2], args[3]),
                5 => call_i5(addr, args[0], args[1], args[2], args[3], args[4]),
                6 => call_i6(addr, args[0], args[1], args[2], args[3], args[4], args[5]),
                _ => {
                    raise_type_error("too many arguments (max 6 supported)");
                    return Err(());
                }
            }),
            RetShape::I64 => RawResult::I64(match args.len() {
                0 => call_i0_64(addr),
                1 => call_i1_64(addr, args[0]),
                2 => call_i2_64(addr, args[0], args[1]),
                3 => call_i3_64(addr, args[0], args[1], args[2]),
                4 => call_i4_64(addr, args[0], args[1], args[2], args[3]),
                5 => call_i5_64(addr, args[0], args[1], args[2], args[3], args[4]),
                6 => call_i6_64(addr, args[0], args[1], args[2], args[3], args[4], args[5]),
                _ => {
                    raise_type_error("too many arguments (max 6 supported)");
                    return Err(());
                }
            }),
            RetShape::F64 => RawResult::F64(match args.len() {
                0 => call_i0_d(addr),
                1 => call_i1_d(addr, args[0]),
                2 => call_i2_d(addr, args[0], args[1]),
                3 => call_i3_d(addr, args[0], args[1], args[2]),
                4 => call_i4_d(addr, args[0], args[1], args[2], args[3]),
                5 => call_i5_d(addr, args[0], args[1], args[2], args[3], args[4]),
                6 => call_i6_d(addr, args[0], args[1], args[2], args[3], args[4], args[5]),
                _ => {
                    raise_type_error("too many arguments (max 6 supported)");
                    return Err(());
                }
            }),
        })
    }
}

fn call_float_args(addr: usize, args: &[f64], shape: RetShape) -> Result<RawResult, ()> {
    unsafe {
        Ok(match shape {
            RetShape::F64 => RawResult::F64(match args.len() {
                0 => call_d0(addr),
                1 => call_d1(addr, args[0]),
                2 => call_d2(addr, args[0], args[1]),
                3 => call_d3(addr, args[0], args[1], args[2]),
                4 => call_d4(addr, args[0], args[1], args[2], args[3]),
                _ => {
                    raise_type_error("too many float arguments (max 4 supported)");
                    return Err(());
                }
            }),
            RetShape::I32 => RawResult::I32(match args.len() {
                0 => call_d0_i(addr),
                1 => call_d1_i(addr, args[0]),
                2 => call_d2_i(addr, args[0], args[1]),
                3 => call_d3_i(addr, args[0], args[1], args[2]),
                4 => call_d4_i(addr, args[0], args[1], args[2], args[3]),
                _ => {
                    raise_type_error("too many float arguments (max 4 supported)");
                    return Err(());
                }
            }),
            RetShape::I64 => RawResult::I64(match args.len() {
                0 => call_d0_i64(addr),
                1 => call_d1_i64(addr, args[0]),
                2 => call_d2_i64(addr, args[0], args[1]),
                3 => call_d3_i64(addr, args[0], args[1], args[2]),
                4 => call_d4_i64(addr, args[0], args[1], args[2], args[3]),
                _ => {
                    raise_type_error("too many float arguments (max 4 supported)");
                    return Err(());
                }
            }),
        })
    }
}

// ── ctype name tables ──

const SCALAR_CTYPES: &[&str] = &[
    "c_bool",
    "c_byte",
    "c_ubyte",
    "c_char",
    "c_char_p",
    "c_short",
    "c_ushort",
    "c_int16",
    "c_uint16",
    "c_int",
    "c_uint",
    "c_int32",
    "c_uint32",
    "c_long",
    "c_ulong",
    "c_longlong",
    "c_ulonglong",
    "c_int64",
    "c_uint64",
    "c_size_t",
    "c_ssize_t",
    "c_void_p",
    "c_voidp",
    "c_double",
    "c_float",
    "c_longdouble",
    "c_wchar",
    "c_wchar_p",
    "c_int8",
    "c_uint8",
];

fn is_known_ctype_name(n: &str) -> bool {
    SCALAR_CTYPES.contains(&n)
}

fn ctype_byte_size(name: &str) -> i64 {
    match name {
        "c_bool" | "c_byte" | "c_ubyte" | "c_char" | "c_int8" | "c_uint8" => 1,
        "c_short" | "c_ushort" | "c_int16" | "c_uint16" => 2,
        "c_int" | "c_uint" | "c_int32" | "c_uint32" | "c_float" | "c_wchar" => 4,
        _ => 8,
    }
}

/// Resolve a scalar-ctype CLASS reference (the func value exposed as
/// `ctypes.c_int` etc, e.g. an `argtypes`/`restype` entry) to its name via
/// the func-addr → class-name bridge.
fn ctype_name_of(v: MbValue) -> Option<String> {
    let addr = v.as_func()?;
    super::super::module::NATIVE_TYPE_NAMES.with(|m| m.borrow().get(&(addr as u64)).cloned())
}

fn ret_shape_for(name: Option<&str>) -> RetShape {
    match name {
        Some("c_double") | Some("c_float") | Some("c_longdouble") => RetShape::F64,
        Some("c_long") | Some("c_ulong") | Some("c_size_t") | Some("c_ssize_t")
        | Some("c_void_p") | Some("c_voidp") | Some("c_char_p") | Some("c_wchar_p")
        | Some("c_longlong") | Some("c_ulonglong") | Some("c_int64") | Some("c_uint64") => {
            RetShape::I64
        }
        _ => RetShape::I32,
    }
}

fn raw_to_value(raw: RawResult, name: &str) -> MbValue {
    match name {
        "c_double" | "c_float" | "c_longdouble" => MbValue::from_float(match raw {
            RawResult::F64(f) => f,
            RawResult::I32(i) => i as f64,
            RawResult::I64(i) => i as f64,
        }),
        "c_uint" | "c_uint32" | "c_uint16" | "c_ubyte" | "c_uint8" | "c_bool" => {
            let v = match raw {
                RawResult::I32(i) => (i as u32) as i64,
                RawResult::I64(i) => i,
                RawResult::F64(f) => f as i64,
            };
            MbValue::from_int(clamp48(v))
        }
        "c_long" | "c_ulong" | "c_size_t" | "c_ssize_t" | "c_void_p" | "c_voidp" | "c_char_p"
        | "c_wchar_p" | "c_longlong" | "c_ulonglong" | "c_int64" | "c_uint64" => {
            let v = match raw {
                RawResult::I64(i) => i,
                RawResult::I32(i) => i as i64,
                RawResult::F64(f) => f as i64,
            };
            MbValue::from_int(clamp48(v))
        }
        _ => {
            let v = match raw {
                RawResult::I32(i) => i as i64,
                RawResult::I64(i) => i,
                RawResult::F64(f) => f as i64,
            };
            MbValue::from_int(clamp48(v))
        }
    }
}

// ── CDLL ──

unsafe extern "C" fn dispatch_cdll_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let name_arg = a.first().copied().unwrap_or_else(MbValue::none);
    let name_val: Option<String> = if name_arg.is_none() {
        None
    } else {
        extract_str(name_arg)
            .or_else(|| extract_bytes(name_arg).map(|b| String::from_utf8_lossy(&b).into_owned()))
    };
    let handle = match &name_val {
        None => unsafe { libc::dlopen(std::ptr::null(), libc::RTLD_NOW) },
        Some(n) => match CString::new(n.as_str()) {
            Ok(cname) => unsafe { libc::dlopen(cname.as_ptr(), libc::RTLD_NOW) },
            Err(_) => {
                super::super::exception::mb_raise(
                    new_str("ValueError"),
                    new_str("embedded null byte"),
                );
                return MbValue::none();
            }
        },
    };
    if handle.is_null() {
        let err = unsafe {
            let p = libc::dlerror();
            if p.is_null() {
                "dlopen: image not found".to_string()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        super::super::exception::mb_raise(new_str("OSError"), new_str(&err));
        return MbValue::none();
    }
    let handle_id = store_ptr(handle as usize);
    let inst = MbValue::from_ptr(MbObject::new_instance("CDLL".to_string()));
    set_field(inst, "_handle_id", MbValue::from_int(handle_id as i64));
    set_field(
        inst,
        "_name",
        name_val.as_deref().map(new_str).unwrap_or_else(MbValue::none),
    );
    inst
}

// Mirrors CPython's `CDLL.__getattr__`, which does
// `func = self.__getitem__(name); setattr(self, name, func); return func` —
// the resolved `CFuncPtr` is cached directly into the CDLL instance's own
// fields. Since `mb_getattr`/`mb_getattr_impl` check instance fields BEFORE
// falling back to `__getattr__`, a later `lib.foo` access hits the cached
// instance straight away instead of re-dispatching here. This is required
// for correctness, not just a nicety: `lib.foo.restype = ...` mutates the
// *cached* instance, so a later `lib.foo(...)` call must see the same
// object or `.restype`/`.argtypes` overrides silently vanish (this is also
// the exact behavior CPython itself exhibits — `.restype` "doesn't stick"
// across separate attribute accesses unless CDLL caches).
extern "C" fn cdll_getattr(self_v: MbValue, name_v: MbValue) -> MbValue {
    let name = extract_str(name_v).unwrap_or_default();
    if name.starts_with('_') {
        raise_attribute_error("CDLL", &name);
        return MbValue::none();
    }
    let Some(handle_id) = get_field(self_v, "_handle_id").and_then(|v| v.as_int()) else {
        raise_attribute_error("CDLL", &name);
        return MbValue::none();
    };
    let handle = load_ptr(handle_id as usize);
    let Ok(csym) = CString::new(name.clone()) else {
        raise_attribute_error("CDLL", &name);
        return MbValue::none();
    };
    let addr = unsafe { libc::dlsym(handle as *mut std::ffi::c_void, csym.as_ptr()) };
    if addr.is_null() {
        raise_attribute_error("CDLL", &name);
        return MbValue::none();
    }
    let addr_id = store_ptr(addr as usize);
    let inst = MbValue::from_ptr(MbObject::new_instance("CFuncPtr".to_string()));
    set_field(inst, "_func_addr_id", MbValue::from_int(addr_id as i64));
    set_field(inst, "_name", new_str(&name));
    set_field(self_v, &name, inst);
    inst
}

fn raise_attribute_error(cls: &str, name: &str) {
    super::super::exception::mb_raise(
        new_str("AttributeError"),
        new_str(&format!("'{cls}' object has no attribute '{name}'")),
    );
}

// ── byref()/pointer()/POINTER() out-param wrappers ──

fn make_carg_wrapper(class_name: &str, target: MbValue) -> MbValue {
    let obj = MbValue::from_ptr(MbObject::new_instance(class_name.to_string()));
    set_field(obj, "_wrapped", target);
    set_field(obj, "contents", target);
    obj
}

/// True when `v` is a value ctypes byref()/pointer() can legally wrap: an
/// actual scalar-ctype instance (`c_int(5)`), or something already wrapped
/// (`byref`/`pointer` of one). A plain unrelated Python object is not a
/// ctypes instance and must raise TypeError — required by the
/// `_ctypes.pointer(obj: _CT)` type wall (typeshed contract).
fn is_ctypes_data_instance(v: MbValue) -> bool {
    if get_field(v, "_wrapped").is_some() {
        return true;
    }
    instance_class_name(v).is_some_and(|cn| is_known_ctype_name(&cn))
}

/// True when `v` is a value POINTER() can legally take: a scalar-ctype
/// constructor (the func value exposed as e.g. `ctypes.c_int`), or a
/// ctypes type-object (Structure/Union/another POINTER() result — Instance
/// class_name="type"). A plain unrelated class/instance is not a ctypes
/// type and must raise TypeError — required by the `_ctypes.POINTER(type:
/// type)` type wall (typeshed contract).
fn is_ctypes_type_ref(v: MbValue) -> bool {
    if ctype_name_of(v).is_some() {
        return true;
    }
    instance_class_name(v).is_some_and(|cn| cn == "type")
}

unsafe extern "C" fn dispatch_byref(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    match a.first().copied() {
        Some(target) if is_ctypes_data_instance(target) => make_carg_wrapper("_CArgObject", target),
        Some(_) => raise_type_error("byref() argument must be a ctypes instance"),
        None => raise_type_error("byref() takes at least 1 argument"),
    }
}

unsafe extern "C" fn dispatch_pointer(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    match a.first().copied() {
        Some(target) if is_ctypes_data_instance(target) => make_carg_wrapper("_Pointer", target),
        Some(_) => raise_type_error("must be a ctypes instance"),
        None => raise_type_error("pointer() takes at least 1 argument"),
    }
}

unsafe extern "C" fn dispatch_pointer_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(pointee) = a.first().copied() else {
        return raise_type_error("POINTER() takes at least 1 argument");
    };
    if !is_ctypes_type_ref(pointee) {
        return raise_type_error("must be a ctypes type");
    }
    let pointee_name = ctype_name_of(pointee).unwrap_or_else(|| "None".to_string());
    let obj = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*obj).data {
            let mut f = fields.write().unwrap();
            let qname = format!("LP_{pointee_name}");
            f.insert("__name__".to_string(), new_str(&qname));
            f.insert("__qualname__".to_string(), new_str(&qname));
            f.insert("__module__".to_string(), new_str("ctypes"));
            f.insert("_type_".to_string(), new_str(&pointee_name));
            f.insert("_ctypes_pointer_of".to_string(), new_str(&pointee_name));
        }
    }
    MbValue::from_ptr(obj)
}

unsafe extern "C" fn dispatch_sizeof(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let Some(v) = a.first().copied() else {
        return raise_type_error("sizeof() takes at least 1 argument");
    };
    let name = ctype_name_of(v).or_else(|| instance_class_name(v)).or_else(|| {
        v.as_ptr().and_then(|p| unsafe {
            if let ObjData::Instance { ref class_name, ref fields } = (*p).data {
                if class_name == "type" {
                    fields.read().unwrap().get("__name__").and_then(|n| extract_str(*n))
                } else {
                    None
                }
            } else {
                None
            }
        })
    });
    match name.as_deref() {
        Some(n) if is_known_ctype_name(n) => MbValue::from_int(ctype_byte_size(n)),
        _ => raise_type_error("this type has no size"),
    }
}

// ── CFuncPtr.__call__ — the real marshalling entry point ──

enum ArgSlot {
    Int(i64),
    Float(f64),
}

struct OutParam {
    buf: *mut [u8; 8],
    target: MbValue,
    ctype_name: String,
}

fn classify_arg(
    idx: usize,
    val: MbValue,
    hint: Option<&str>,
    out_params: &mut Vec<OutParam>,
) -> Result<ArgSlot, ()> {
    // byref()/pointer() wrapper: allocate an 8-byte scratch buffer seeded
    // from the wrapped instance's current value, pass its address, and
    // schedule a post-call writeback.
    if let Some(wrapped) = get_field(val, "_wrapped") {
        let cn = instance_class_name(wrapped).unwrap_or_else(|| "c_long".to_string());
        let is_float = matches!(cn.as_str(), "c_double" | "c_float" | "c_longdouble");
        let inner_val = get_field(wrapped, "value").unwrap_or_else(MbValue::none);
        let mut bytes = [0u8; 8];
        if is_float {
            let f = inner_val
                .as_float()
                .or_else(|| inner_val.as_int_pyint().map(|i| i as f64))
                .unwrap_or(0.0);
            bytes.copy_from_slice(&f.to_ne_bytes());
        } else {
            let i = inner_val.as_int_pyint().unwrap_or(0);
            bytes.copy_from_slice(&i.to_ne_bytes());
        }
        let raw_ptr = Box::into_raw(Box::new(bytes));
        out_params.push(OutParam { buf: raw_ptr, target: wrapped, ctype_name: cn });
        return Ok(ArgSlot::Int(raw_ptr as i64));
    }
    // An actual ctype scalar instance (e.g. `c_int(5)`): unwrap `.value`
    // and encode by-value per its own class.
    if let Some(cn) = instance_class_name(val) {
        if is_known_ctype_name(&cn) {
            let inner = get_field(val, "value").unwrap_or_else(MbValue::none);
            return encode_by_ctype(idx, &cn, inner);
        }
    }
    if let Some(h) = hint {
        if is_known_ctype_name(h) {
            return encode_by_ctype(idx, h, val);
        }
    }
    // No argtypes hint, no ctype instance: infer straight from the raw
    // Python value.
    if val.is_none() {
        return Ok(ArgSlot::Int(0));
    }
    if let Some(mut b) = extract_bytes(val) {
        b.push(0);
        let ptr = Box::leak(b.into_boxed_slice()).as_ptr() as usize;
        return Ok(ArgSlot::Int(ptr as i64));
    }
    if let Some(f) = val.as_float() {
        return Ok(ArgSlot::Float(f));
    }
    if let Some(i) = val.as_int_pyint() {
        return Ok(ArgSlot::Int(i));
    }
    raise_argument_error(idx, "don't know how to convert parameter");
    Err(())
}

fn encode_by_ctype(idx: usize, name: &str, val: MbValue) -> Result<ArgSlot, ()> {
    match name {
        "c_double" | "c_float" | "c_longdouble" => {
            if let Some(f) = val.as_float() {
                Ok(ArgSlot::Float(f))
            } else if let Some(i) = val.as_int_pyint() {
                Ok(ArgSlot::Float(i as f64))
            } else {
                raise_argument_error(idx, "wrong type (expected float)");
                Err(())
            }
        }
        "c_char_p" => {
            if let Some(mut b) = extract_bytes(val) {
                b.push(0);
                let ptr = Box::leak(b.into_boxed_slice()).as_ptr() as usize;
                Ok(ArgSlot::Int(ptr as i64))
            } else if let Some(i) = val.as_int_pyint() {
                Ok(ArgSlot::Int(i))
            } else if val.is_none() {
                Ok(ArgSlot::Int(0))
            } else {
                raise_argument_error(idx, "bytes, int address, or None expected for c_char_p");
                Err(())
            }
        }
        "c_wchar_p" | "c_wchar" => {
            raise_type_error(
                "c_wchar_p/c_wchar argument marshalling is not supported by this ctypes build",
            );
            Err(())
        }
        _ => {
            if let Some(i) = val.as_int_pyint() {
                Ok(ArgSlot::Int(i))
            } else if val.is_none() && (name == "c_void_p" || name == "c_voidp") {
                Ok(ArgSlot::Int(0))
            } else {
                raise_argument_error(idx, "wrong type (expected int)");
                Err(())
            }
        }
    }
}

fn writeback(out: &OutParam) {
    let bytes = unsafe { *out.buf };
    let new_val = match out.ctype_name.as_str() {
        "c_double" | "c_float" | "c_longdouble" => MbValue::from_float(f64::from_ne_bytes(bytes)),
        "c_int" | "c_uint" | "c_int32" | "c_uint32" | "c_short" | "c_ushort" | "c_int16"
        | "c_uint16" | "c_byte" | "c_ubyte" | "c_int8" | "c_uint8" | "c_bool" => {
            let v32 = i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            MbValue::from_int(v32 as i64)
        }
        _ => MbValue::from_int(clamp48(i64::from_ne_bytes(bytes))),
    };
    set_field(out.target, "value", new_val);
}

extern "C" fn cfuncptr_call(self_v: MbValue, args_list: MbValue) -> MbValue {
    let call_args = extract_args(args_list);
    let Some(addr_id) = get_field(self_v, "_func_addr_id").and_then(|v| v.as_int()) else {
        return raise_type_error("invalid foreign function object");
    };
    let func_addr = load_ptr(addr_id as usize);

    let argtypes_names: Option<Vec<Option<String>>> =
        get_field(self_v, "argtypes").and_then(|v| {
            if v.is_none() {
                None
            } else {
                Some(extract_args(v).iter().map(|t| ctype_name_of(*t)).collect())
            }
        });
    if let Some(ref names) = argtypes_names {
        if names.len() != call_args.len() {
            return raise_type_error(&format!(
                "this function takes {} argument{} ({} given)",
                names.len(),
                if names.len() == 1 { "" } else { "s" },
                call_args.len()
            ));
        }
    }

    let mut out_params: Vec<OutParam> = Vec::new();
    let mut int_args: Vec<i64> = Vec::new();
    let mut float_args: Vec<f64> = Vec::new();
    let mut saw_int = false;
    let mut saw_float = false;
    for (i, val) in call_args.iter().copied().enumerate() {
        let hint = argtypes_names.as_ref().and_then(|v| v.get(i).cloned().flatten());
        match classify_arg(i, val, hint.as_deref(), &mut out_params) {
            Ok(ArgSlot::Int(x)) => {
                saw_int = true;
                int_args.push(x);
            }
            Ok(ArgSlot::Float(f)) => {
                saw_float = true;
                float_args.push(f);
            }
            Err(()) => {
                for out in &out_params {
                    unsafe {
                        drop(Box::from_raw(out.buf));
                    }
                }
                return MbValue::none();
            }
        }
    }
    if saw_int && saw_float {
        for out in &out_params {
            unsafe {
                drop(Box::from_raw(out.buf));
            }
        }
        return raise_type_error(
            "mixed int/float argument marshalling in one call is not supported by this hand-rolled ctypes FFI (see #875 follow-ups)",
        );
    }

    let restype_field = get_field(self_v, "restype");
    let is_void = matches!(restype_field, Some(v) if v.is_none());
    let restype_name = restype_field.and_then(|v| if v.is_none() { None } else { ctype_name_of(v) });
    let shape = ret_shape_for(restype_name.as_deref());

    let raw = if saw_float {
        call_float_args(func_addr, &float_args, shape)
    } else {
        call_int_args(func_addr, &int_args, shape)
    };

    for out in &out_params {
        if raw.is_ok() {
            writeback(out);
        }
        unsafe {
            drop(Box::from_raw(out.buf));
        }
    }

    match raw {
        Ok(r) => {
            if is_void {
                MbValue::none()
            } else {
                raw_to_value(r, restype_name.as_deref().unwrap_or("c_int"))
            }
        }
        Err(()) => MbValue::none(),
    }
}

// ── Scalar c_* constructors ──

fn truthy_ctype(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int_pyint() {
        return i != 0;
    }
    !v.is_none()
}

fn raise_type_error_scalar(cls: &str, msg: &str) {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(&format!("{cls}: {msg}")));
}

fn default_value_for(cls: &str) -> MbValue {
    match cls {
        "c_double" | "c_float" | "c_longdouble" => MbValue::from_float(0.0),
        "c_char_p" | "c_wchar_p" | "c_void_p" | "c_voidp" => MbValue::none(),
        "c_bool" => MbValue::from_bool(false),
        "c_char" => MbValue::from_ptr(MbObject::new_bytes(vec![0])),
        "c_wchar" => new_str("\u{0}"),
        _ => MbValue::from_int(0),
    }
}

fn coerce_ctype_construct(cls: &str, raw: Option<MbValue>) -> Result<MbValue, ()> {
    let Some(v) = raw else {
        return Ok(default_value_for(cls));
    };
    match cls {
        "c_double" | "c_float" | "c_longdouble" => {
            if let Some(f) = v.as_float() {
                Ok(MbValue::from_float(f))
            } else if let Some(i) = v.as_int_pyint() {
                Ok(MbValue::from_float(i as f64))
            } else {
                raise_type_error_scalar(cls, "a float is required");
                Err(())
            }
        }
        "c_char_p" => {
            if v.is_none() {
                Ok(MbValue::none())
            } else if extract_bytes(v).is_some() {
                Ok(v)
            } else if v.as_int_pyint().is_some() {
                Ok(v)
            } else {
                raise_type_error_scalar(cls, "bytes or integer address expected");
                Err(())
            }
        }
        "c_wchar_p" => {
            if v.is_none() {
                Ok(MbValue::none())
            } else if extract_str(v).is_some() {
                Ok(v)
            } else {
                raise_type_error_scalar(cls, "unicode string expected");
                Err(())
            }
        }
        "c_wchar" => {
            if let Some(s) = extract_str(v) {
                Ok(new_str(&s))
            } else {
                raise_type_error_scalar(cls, "a unicode character is required");
                Err(())
            }
        }
        "c_char" => {
            if let Some(b) = extract_bytes(v) {
                Ok(MbValue::from_ptr(MbObject::new_bytes(b)))
            } else {
                raise_type_error_scalar(cls, "a bytes object of length 1 is required");
                Err(())
            }
        }
        "c_void_p" | "c_voidp" => {
            if v.is_none() {
                Ok(MbValue::none())
            } else if let Some(i) = v.as_int_pyint() {
                Ok(MbValue::from_int(clamp48(i)))
            } else {
                raise_type_error_scalar(cls, "an integer is required");
                Err(())
            }
        }
        "c_bool" => Ok(MbValue::from_bool(truthy_ctype(v))),
        _ => {
            if let Some(i) = v.as_int_pyint() {
                Ok(MbValue::from_int(clamp48(i)))
            } else {
                raise_type_error_scalar(cls, "an integer is required");
                Err(())
            }
        }
    }
}

fn generic_scalar_new(cls: &'static str, raw: Option<MbValue>) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(cls.to_string()));
    match coerce_ctype_construct(cls, raw) {
        Ok(v) => {
            set_field(inst, "value", v);
            inst
        }
        Err(()) => MbValue::none(),
    }
}

macro_rules! scalar_ctor {
    ($fn_name:ident, $cls:literal) => {
        unsafe extern "C" fn $fn_name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            generic_scalar_new($cls, a.first().copied())
        }
    };
}

scalar_ctor!(ctor_c_bool, "c_bool");
scalar_ctor!(ctor_c_byte, "c_byte");
scalar_ctor!(ctor_c_ubyte, "c_ubyte");
scalar_ctor!(ctor_c_char, "c_char");
scalar_ctor!(ctor_c_short, "c_short");
scalar_ctor!(ctor_c_ushort, "c_ushort");
scalar_ctor!(ctor_c_int, "c_int");
scalar_ctor!(ctor_c_uint, "c_uint");
scalar_ctor!(ctor_c_long, "c_long");
scalar_ctor!(ctor_c_ulong, "c_ulong");
scalar_ctor!(ctor_c_longlong, "c_longlong");
scalar_ctor!(ctor_c_ulonglong, "c_ulonglong");
scalar_ctor!(ctor_c_int8, "c_int8");
scalar_ctor!(ctor_c_uint8, "c_uint8");
scalar_ctor!(ctor_c_int16, "c_int16");
scalar_ctor!(ctor_c_uint16, "c_uint16");
scalar_ctor!(ctor_c_int32, "c_int32");
scalar_ctor!(ctor_c_uint32, "c_uint32");
scalar_ctor!(ctor_c_int64, "c_int64");
scalar_ctor!(ctor_c_uint64, "c_uint64");
scalar_ctor!(ctor_c_size_t, "c_size_t");
scalar_ctor!(ctor_c_ssize_t, "c_ssize_t");
scalar_ctor!(ctor_c_double, "c_double");
scalar_ctor!(ctor_c_float, "c_float");
scalar_ctor!(ctor_c_longdouble, "c_longdouble");
scalar_ctor!(ctor_c_void_p, "c_void_p");
scalar_ctor!(ctor_c_char_p, "c_char_p");
scalar_ctor!(ctor_c_wchar, "c_wchar");
scalar_ctor!(ctor_c_wchar_p, "c_wchar_p");

/// `CFuncPtr(...)` / `CFuncPtr.__new__(CFuncPtr, ...)` direct construction —
/// out of scope (CFUNCTYPE-callback path; real `CFuncPtr` instances in this
/// module are produced only internally by `cdll_getattr` via dlsym, which
/// never runs through `__new__`). Mirrors the original shell's unconditional
/// TypeError so the `type/std-libs/_ctypes` type wall stays enforced. Uses
/// the plain free-function convention (not the bound `(self, args)`
/// variadic-method convention) because `_ctypes.CFuncPtr` is exposed as a
/// dict-shaped shell object (see `register()`), so `CFuncPtr.__new__` is a
/// direct attribute-value call, not a class-dispatched method call.
unsafe extern "C" fn cfuncptr_new_direct(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    raise_type_error("argument must be callable or integer function address")
}

// ── class registration ──

type MethodSpec = (&'static str, usize, bool);

fn register_class(class_name: &str, bases: &[&str], methods: &[MethodSpec]) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr, variadic) in methods {
        map.insert((*name).to_string(), MbValue::from_func(*addr));
        if *variadic {
            super::super::module::register_variadic_func(*addr as u64);
        }
    }
    super::super::class::mb_class_register(
        class_name,
        bases.iter().map(|s| s.to_string()).collect(),
        map,
    );
}

fn register_classes() {
    register_class("CDLL", &["object"], &[("__getattr__", cdll_getattr as usize, false)]);
    register_class("CFuncPtr", &["object"], &[("__call__", cfuncptr_call as usize, true)]);
    register_class(
        "Array",
        &["object"],
        &[
            ("__getitem__", ctypes_array_getitem as usize, true),
            ("__setitem__", ctypes_array_setitem as usize, true),
        ],
    );
}

/// `ctypes.ArgumentError` as an unregistered type-object (mirrors
/// `argparse_mod::make_exception_type_object`). Deliberately NOT run through
/// `mb_class_register` under the bare name "ArgumentError": argparse already
/// owns that name in the global flat CLASS_REGISTRY, and registering again
/// here (this module runs after argparse_mod) would silently overwrite
/// argparse's `ArgumentError` class. Bare-string exact-name matching in
/// `mb_exception_matches` still makes `except ctypes.ArgumentError` work for
/// `mb_raise("ArgumentError", ...)` — the tradeoff (documented, out of
/// required scope) is that `except Exception:` won't broadly catch it.
fn make_ctypes_exception_type_object(name: &str) -> MbValue {
    let cls = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*cls).data {
            let mut f = fields.write().unwrap();
            f.insert("__name__".to_string(), new_str(name));
            f.insert("__qualname__".to_string(), new_str(name));
            f.insert("__module__".to_string(), new_str("ctypes"));
        }
    }
    MbValue::from_ptr(cls)
}

pub fn register() {
    register_classes();

    let shell = dispatch_class_shell as *const () as usize;
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    // Baseline: every name ctypes has ever exposed here gets a shell first,
    // so nothing observable via `hasattr`/presence fixtures disappears.
    // Real implementations below overwrite the ones this issue is scoped to.
    for name in [
        "CDLL", "PyDLL", "WinDLL", "OleDLL", "LibraryLoader", "Structure", "Union", "Array",
        "BigEndianStructure", "LittleEndianStructure", "c_byte", "c_ubyte", "c_char", "c_char_p",
        "c_double", "c_longdouble", "c_float", "c_int", "c_uint", "c_int8", "c_uint8", "c_int16",
        "c_uint16", "c_int32", "c_uint32", "c_int64", "c_uint64", "c_long", "c_ulong",
        "c_longlong", "c_ulonglong", "c_short", "c_ushort", "c_size_t", "c_ssize_t", "c_void_p",
        "c_wchar", "c_wchar_p", "c_bool", "POINTER", "pointer", "byref", "cast", "addressof",
        "alignment", "sizeof", "string_at", "wstring_at", "memmove", "memset", "CFUNCTYPE",
        "WINFUNCTYPE", "PYFUNCTYPE", "HRESULT", "ArgumentError", "Error", "ARRAY",
        "BigEndianUnion", "LittleEndianUnion", "SetPointerType", "c_buffer", "c_time_t",
        "c_voidp", "create_string_buffer", "create_unicode_buffer", "py_object", "resize",
        "pythonapi",
    ] {
        attrs.insert(name.to_string(), MbValue::from_func(shell));
    }
    attrs.insert("cdll".into(), MbValue::from_func(dispatch_class_shell as usize));
    attrs.insert("windll".into(), MbValue::from_func(dispatch_class_shell as usize));
    attrs.insert("oledll".into(), MbValue::from_func(dispatch_class_shell as usize));
    attrs.insert("pydll".into(), MbValue::from_func(dispatch_class_shell as usize));
    attrs.insert("get_errno".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("set_errno".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("get_last_error".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("set_last_error".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("FormatError".into(), MbValue::from_func(dispatch_empty_str as usize));
    attrs.insert("WinError".into(), MbValue::from_func(dispatch_class_shell as usize));
    attrs.insert("DllCanUnloadNow".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("DllGetClassObject".into(), MbValue::from_func(dispatch_int_zero as usize));
    attrs.insert("GetLastError".into(), MbValue::from_func(dispatch_int_zero as usize));
    register_addrs(&[
        shell,
        dispatch_int_zero as usize,
        dispatch_empty_str as usize,
    ]);

    attrs.insert("DEFAULT_MODE".into(), MbValue::from_int(0));
    attrs.insert("RTLD_LOCAL".into(), MbValue::from_int(0));
    attrs.insert("RTLD_GLOBAL".into(), MbValue::from_int(256));
    attrs.insert("FUNCFLAG_CDECL".into(), MbValue::from_int(1));
    attrs.insert("FUNCFLAG_HRESULT".into(), MbValue::from_int(2));
    attrs.insert("FUNCFLAG_PYTHONAPI".into(), MbValue::from_int(4));
    attrs.insert("FUNCFLAG_USE_ERRNO".into(), MbValue::from_int(8));
    attrs.insert("FUNCFLAG_USE_LASTERROR".into(), MbValue::from_int(16));
    attrs.insert("SIZEOF_TIME_T".into(), MbValue::from_int(8));

    // ── Real substrate: CDLL/PyDLL construction ──
    attrs.insert("CDLL".into(), MbValue::from_func(dispatch_cdll_new as usize));
    attrs.insert("PyDLL".into(), MbValue::from_func(dispatch_cdll_new as usize));
    register_addrs(&[dispatch_cdll_new as usize]);
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(dispatch_cdll_new as u64, "CDLL".to_string());
    });

    // ── Real substrate: scalar c_* constructors ──
    let scalar_ctors: [(&str, usize); 29] = [
        ("c_bool", ctor_c_bool as usize),
        ("c_byte", ctor_c_byte as usize),
        ("c_ubyte", ctor_c_ubyte as usize),
        ("c_char", ctor_c_char as usize),
        ("c_short", ctor_c_short as usize),
        ("c_ushort", ctor_c_ushort as usize),
        ("c_int", ctor_c_int as usize),
        ("c_uint", ctor_c_uint as usize),
        ("c_long", ctor_c_long as usize),
        ("c_ulong", ctor_c_ulong as usize),
        ("c_longlong", ctor_c_longlong as usize),
        ("c_ulonglong", ctor_c_ulonglong as usize),
        ("c_int8", ctor_c_int8 as usize),
        ("c_uint8", ctor_c_uint8 as usize),
        ("c_int16", ctor_c_int16 as usize),
        ("c_uint16", ctor_c_uint16 as usize),
        ("c_int32", ctor_c_int32 as usize),
        ("c_uint32", ctor_c_uint32 as usize),
        ("c_int64", ctor_c_int64 as usize),
        ("c_uint64", ctor_c_uint64 as usize),
        ("c_size_t", ctor_c_size_t as usize),
        ("c_ssize_t", ctor_c_ssize_t as usize),
        ("c_double", ctor_c_double as usize),
        ("c_float", ctor_c_float as usize),
        ("c_longdouble", ctor_c_longdouble as usize),
        ("c_void_p", ctor_c_void_p as usize),
        ("c_char_p", ctor_c_char_p as usize),
        ("c_wchar", ctor_c_wchar as usize),
        ("c_wchar_p", ctor_c_wchar_p as usize),
    ];
    for (name, addr) in scalar_ctors {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        register_addrs(&[addr]);
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(addr as u64, name.to_string());
        });
    }
    // c_voidp is a spelling alias for c_void_p.
    attrs.insert("c_voidp".into(), MbValue::from_func(ctor_c_void_p as usize));

    // ── Real substrate: POINTER/pointer/byref/sizeof ──
    attrs.insert("POINTER".into(), MbValue::from_func(dispatch_pointer_type as usize));
    attrs.insert("pointer".into(), MbValue::from_func(dispatch_pointer as usize));
    attrs.insert("byref".into(), MbValue::from_func(dispatch_byref as usize));
    attrs.insert("sizeof".into(), MbValue::from_func(dispatch_sizeof as usize));
    register_addrs(&[
        dispatch_pointer_type as usize,
        dispatch_pointer as usize,
        dispatch_byref as usize,
        dispatch_sizeof as usize,
    ]);

    // ── ctypes.ArgumentError — unregistered type-object, see doc comment. ──
    attrs.insert("ArgumentError".into(), make_ctypes_exception_type_object("ArgumentError"));

    super::register_module("ctypes", attrs);

    // ── `_ctypes` internal module (unchanged shells except sizeof/POINTER/
    // pointer, which now delegate to the same real dispatchers). ──
    let mut ctypes_internal = HashMap::new();
    let array_getitem = ctypes_array_getitem_direct as *const () as usize;
    let array_setitem = ctypes_array_setitem_direct as *const () as usize;
    let alignment = ctypes_alignment_direct as *const () as usize;
    let buffer_info = ctypes_buffer_info_direct as *const () as usize;
    let py_incref = ctypes_py_incref_direct as *const () as usize;
    let py_decref = ctypes_py_decref_direct as *const () as usize;
    let cfuncptr_new = cfuncptr_new_direct as *const () as usize;
    register_addrs(&[
        array_getitem,
        array_setitem,
        alignment,
        buffer_info,
        py_incref,
        py_decref,
        cfuncptr_new,
    ]);
    ctypes_internal.insert(
        "Array".to_string(),
        make_ctypes_callable_shell(
            "Array",
            "_ctypes",
            &[("__getitem__", array_getitem), ("__setitem__", array_setitem)],
        ),
    );
    ctypes_internal.insert(
        "CFuncPtr".to_string(),
        make_ctypes_callable_shell("CFuncPtr", "_ctypes", &[("__new__", cfuncptr_new)]),
    );
    for name in ["Structure", "Union"] {
        ctypes_internal.insert(name.to_string(), make_type_obj(name, "_ctypes"));
    }
    ctypes_internal.insert("POINTER".to_string(), MbValue::from_func(dispatch_pointer_type as usize));
    ctypes_internal.insert("pointer".to_string(), MbValue::from_func(dispatch_pointer as usize));
    ctypes_internal.insert("alignment".to_string(), MbValue::from_func(alignment));
    ctypes_internal.insert("buffer_info".to_string(), MbValue::from_func(buffer_info));
    ctypes_internal.insert("sizeof".to_string(), MbValue::from_func(dispatch_sizeof as usize));
    ctypes_internal.insert("Py_INCREF".to_string(), MbValue::from_func(py_incref));
    ctypes_internal.insert("Py_DECREF".to_string(), MbValue::from_func(py_decref));
    super::register_module("_ctypes", ctypes_internal);

    // ── ctypes.util / ctypes.wintypes — unchanged shells. ──
    let mut util_attrs = HashMap::new();
    util_attrs.insert("find_library".to_string(), MbValue::from_func(dispatch_noop as usize));
    util_attrs.insert("find_msvcrt".to_string(), MbValue::from_func(dispatch_noop as usize));
    register_addrs(&[dispatch_noop as usize]);
    super::register_module("ctypes.util", util_attrs);

    let mut wintypes_attrs = HashMap::new();
    for cn in [
        "BOOL", "BYTE", "WORD", "DWORD", "UINT", "INT", "FLOAT", "LPVOID", "LPCVOID", "HANDLE",
        "HWND", "HMODULE", "HINSTANCE", "HKEY", "HMENU", "HRESULT", "LPCWSTR", "LPWSTR", "LPCSTR",
        "LPSTR", "LARGE_INTEGER", "ULARGE_INTEGER", "SIZE", "POINT", "RECT", "FILETIME",
        "SYSTEMTIME", "MSG", "BSTR",
    ] {
        wintypes_attrs.insert(cn.to_string(), MbValue::from_func(shell));
    }
    super::register_module("ctypes.wintypes", wintypes_attrs);
}
