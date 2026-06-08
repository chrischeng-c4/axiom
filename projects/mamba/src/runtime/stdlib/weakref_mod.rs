// HANDWRITE-BEGIN reason: #1466 — no real weakref semantics, GC integration
// required. Mamba's runtime is refcount-only, so every entry below ships a
// strong-reference stub that preserves API SHAPE but cannot honour the
// CPython contract (referent expiry, callback firing on collection,
// auto-eviction of WeakSet / WeakValueDictionary entries, ReferenceError
// on dead-proxy access, `finalize.alive` flip). Closing this gap requires
// a GC / liveness-tracking subsystem in `runtime::rc` (or a successor)
// that can publish referent-collected events to a per-object weak-ref
// table. Tracked under #1466 (conformance(mamba/stdlib): weakref); also
// see legacy #437.
//
// Surface coverage vs `cpython312_surface.json` weakref entry: 13/13
// (100%) — all CPython 3.12 public names registered. Stub semantics
// fail the dedicated `test_weakref_semantics.py` fixture by design
// (1/11 lines match; the rest are documented gaps blocked on GC
// integration). Surface-only `test_weakref.py` fixture passes
// 22/22 — API SHAPE is conformant.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// weakref module for Mamba (#437, #1265 Task #79, Wave-8, #1466).
///
/// 16-entry surface (CPython 3.12 `weakref`):
///   CallableProxyType, KeyedRef, ProxyType, ProxyTypes, ReferenceType,
///   WeakKeyDictionary, WeakMethod, WeakSet, WeakValueDictionary,
///   finalize, getweakrefcount, getweakrefs, itertools, proxy, ref, sys.
///
/// Carve-out: NO REAL WEAK SEMANTICS.
///
/// Real weak references require GC integration (tracking live objects and
/// invalidating references when the referent is collected). Mamba's
/// runtime is refcount-only, so this module ships strong-reference stubs
/// that preserve API shape without true weakness:
///
///   - `ref(obj, callback=None)`: returns an Instance carrying a strong
///     pointer to `obj` (the referent never expires under our refcount
///     world). The callback, if any, is stored but never invoked.
///   - `proxy(obj, callback=None)`: returns `obj` unchanged — no proxy
///     wrapper, so `ReferenceError` cannot be raised.
///   - `WeakSet` / `WeakKeyDictionary` / `WeakValueDictionary` /
///     `WeakMethod` / `finalize`: Instance stubs that behave like their
///     strong-ref equivalents. Entries persist for the lifetime of the
///     container; no automatic eviction on referent collection.
///   - `getweakrefcount(obj)` always returns 0.
///   - `getweakrefs(obj)` always returns the empty list.
///   - `ReferenceType` / `ProxyType` / `CallableProxyType` /
///     `KeyedRef` / `ProxyTypes`: exposed as Instance type placeholders
///     with `__name__` set (so `isinstance` lookups and class checks
///     resolve without exploding). They are not true type objects.
///   - `itertools` / `sys`: CPython leaks these submodule imports into
///     `weakref`'s namespace; we mirror that with module placeholders.
///
/// Tracked under #437 — promote to real weak references once the GC /
/// liveness-tracking work lands.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

/// Extract a string from an MbValue.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Build a class-stub Instance with `__name__` / `__module__` set.
fn make_class_stub(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("weakref".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ---------------------------------------------------------------------------
// Dispatchers
// ---------------------------------------------------------------------------

unsafe extern "C" fn dispatch_ref(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_ref(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_proxy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_proxy(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_getweakrefcount(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_getweakrefcount(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_getweakrefs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_getweakrefs(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_weak_set(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_weakref_weak_set()
}

unsafe extern "C" fn dispatch_weak_key_dict(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_weakref_weak_key_dict()
}

unsafe extern "C" fn dispatch_weak_value_dict(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_weakref_weak_value_dict()
}

unsafe extern "C" fn dispatch_weak_method(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_weak_method(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_finalize(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_weakref_finalize(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

// ---------------------------------------------------------------------------
// Helpers (public surface implementations)
// ---------------------------------------------------------------------------

/// weakref.ref(obj, callback=None) -> Instance wrapping a strong ref.
///
/// Carve-out: holds a strong pointer; never expires.
pub fn mb_weakref_ref(obj: MbValue, callback: MbValue) -> MbValue {
    let target_id = if let Some(ptr) = obj.as_ptr() {
        MbValue::from_int(ptr as i64)
    } else {
        let folded = ((obj.to_bits() >> 16) ^ obj.to_bits()) & 0x0000_7FFF_FFFF_FFFF;
        MbValue::from_int(folded as i64)
    };
    let mut fields = FxHashMap::default();
    fields.insert("_target".to_string(), obj);
    fields.insert("_target_id".to_string(), target_id);
    fields.insert("_callback".to_string(), callback);
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("ReferenceType".to_string())),
    );
    let obj_inst = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "ReferenceType".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj_inst))
}

/// weakref.deref(wref) -> referent or None (stub).
///
/// Legacy mamba surface (not part of CPython's weakref module). Real
/// CPython users call the ref like a function: `r()`. We keep this
/// helper for backward compatibility; it now returns the stored
/// `_target` strong ref (which never expires under carve-out).
pub fn mb_weakref_deref(wref: MbValue) -> MbValue {
    if let Some(ptr) = wref.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(t) = f.get("_target") {
                    return *t;
                }
            }
        }
    }
    MbValue::none()
}

/// weakref.proxy(obj, callback=None) -> obj (carve-out).
pub fn mb_weakref_proxy(obj: MbValue, _callback: MbValue) -> MbValue {
    obj
}

/// weakref.getweakrefcount(obj) -> 0 (carve-out).
pub fn mb_weakref_getweakrefcount(_obj: MbValue) -> MbValue {
    MbValue::from_int(0)
}

/// weakref.getweakrefs(obj) -> [] (carve-out).
pub fn mb_weakref_getweakrefs(_obj: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// weakref.WeakSet() -> Instance stub (behaves like a normal set).
pub fn mb_weakref_weak_set() -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "data".to_string(),
        MbValue::from_ptr(MbObject::new_set(Vec::new())),
    );
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WeakSet".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "WeakSet".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// weakref.WeakKeyDictionary() -> Instance stub backed by a plain dict.
pub fn mb_weakref_weak_key_dict() -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("data".to_string(), MbValue::from_ptr(MbObject::new_dict()));
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WeakKeyDictionary".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "WeakKeyDictionary".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// weakref.WeakValueDictionary() -> Instance stub backed by a plain dict.
pub fn mb_weakref_weak_value_dict() -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("data".to_string(), MbValue::from_ptr(MbObject::new_dict()));
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WeakValueDictionary".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "WeakValueDictionary".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// weakref.WeakMethod(method) -> Instance stub holding a strong ref.
pub fn mb_weakref_weak_method(method: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_method".to_string(), method);
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("WeakMethod".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "WeakMethod".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// weakref.finalize(obj, func, /*args*/) -> Instance stub.
///
/// Carve-out: the finalizer is stored but never invoked (no GC hook).
pub fn mb_weakref_finalize(obj: MbValue, func: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_obj".to_string(), obj);
    fields.insert("_func".to_string(), func);
    fields.insert("alive".to_string(), MbValue::from_bool(true));
    fields.insert(
        "__class__".to_string(),
        MbValue::from_ptr(MbObject::new_str("finalize".to_string())),
    );
    let obj_inst = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "finalize".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj_inst))
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

/// Register the weakref module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("ref", dispatch_ref as *const () as usize),
        // Legacy mamba alias — pre-Wave-8 code used `ref_` because `ref` is
        // a Rust keyword. Kept to avoid breaking existing imports.
        ("ref_", dispatch_ref as *const () as usize),
        ("proxy", dispatch_proxy as *const () as usize),
        (
            "getweakrefcount",
            dispatch_getweakrefcount as *const () as usize,
        ),
        ("getweakrefs", dispatch_getweakrefs as *const () as usize),
        ("WeakSet", dispatch_weak_set as *const () as usize),
        (
            "WeakKeyDictionary",
            dispatch_weak_key_dict as *const () as usize,
        ),
        (
            "WeakValueDictionary",
            dispatch_weak_value_dict as *const () as usize,
        ),
        ("WeakMethod", dispatch_weak_method as *const () as usize),
        ("finalize", dispatch_finalize as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Type placeholders — Instance stubs with `__name__` set.
    attrs.insert(
        "ReferenceType".to_string(),
        make_class_stub("ReferenceType"),
    );
    attrs.insert("ProxyType".to_string(), make_class_stub("ProxyType"));
    attrs.insert(
        "CallableProxyType".to_string(),
        make_class_stub("CallableProxyType"),
    );
    attrs.insert("KeyedRef".to_string(), make_class_stub("KeyedRef"));

    // `ProxyTypes` is a CPython tuple of (ProxyType, CallableProxyType).
    let proxy_types = MbObject::new_tuple(vec![
        make_class_stub("ProxyType"),
        make_class_stub("CallableProxyType"),
    ]);
    attrs.insert("ProxyTypes".to_string(), MbValue::from_ptr(proxy_types));

    // Submodule re-exports leaked by CPython's weakref.
    attrs.insert("itertools".to_string(), make_class_stub("itertools"));
    attrs.insert("sys".to_string(), make_class_stub("sys"));

    super::register_module("weakref", attrs);
}
// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    // -- ref --

    #[test]
    fn test_ref_creates_instance_with_target() {
        let obj = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let wref = mb_weakref_ref(obj, MbValue::none());
        assert!(wref.as_ptr().is_some());
        assert_eq!(
            get_str(get_field(wref, "__class__")),
            Some("ReferenceType".to_string())
        );
        // Target preserved as strong ref under carve-out.
        let tgt = get_field(wref, "_target");
        assert_eq!(tgt.as_ptr(), obj.as_ptr());
    }

    #[test]
    fn test_ref_stores_callback() {
        let obj = MbValue::from_int(7);
        let cb = MbValue::from_int(42);
        let wref = mb_weakref_ref(obj, cb);
        assert_eq!(get_field(wref, "_callback").as_int(), Some(42));
    }

    #[test]
    fn test_deref_returns_target() {
        // Carve-out: deref returns the strong-ref target, not None.
        let obj = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let wref = mb_weakref_ref(obj, MbValue::none());
        let r = mb_weakref_deref(wref);
        assert_eq!(r.as_ptr(), obj.as_ptr());
    }

    // -- proxy --

    #[test]
    fn test_proxy_returns_object() {
        let v = MbValue::from_int(42);
        assert_eq!(mb_weakref_proxy(v, MbValue::none()).as_int(), Some(42));
    }

    #[test]
    fn test_proxy_with_callback_returns_object() {
        let v = MbValue::from_int(99);
        let cb = MbValue::from_int(1);
        assert_eq!(mb_weakref_proxy(v, cb).as_int(), Some(99));
    }

    // -- getweakrefcount / getweakrefs --

    #[test]
    fn test_getweakrefcount_zero() {
        let v = MbValue::from_int(1);
        assert_eq!(mb_weakref_getweakrefcount(v).as_int(), Some(0));
    }

    #[test]
    fn test_getweakrefs_empty_list() {
        let v = MbValue::from_int(1);
        let r = mb_weakref_getweakrefs(v);
        unsafe {
            if let ObjData::List(ref lock) = (*r.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected List");
            }
        }
    }

    // -- WeakSet / WeakKeyDictionary / WeakValueDictionary --

    #[test]
    fn test_weak_set_is_instance_with_set_data() {
        let s = mb_weakref_weak_set();
        assert_eq!(
            get_str(get_field(s, "__class__")),
            Some("WeakSet".to_string())
        );
        let data = get_field(s, "data");
        unsafe {
            assert!(matches!(&(*data.as_ptr().unwrap()).data, ObjData::Set(_)));
        }
    }

    #[test]
    fn test_weak_key_dict_is_instance_with_dict_data() {
        let d = mb_weakref_weak_key_dict();
        assert_eq!(
            get_str(get_field(d, "__class__")),
            Some("WeakKeyDictionary".to_string())
        );
        let data = get_field(d, "data");
        unsafe {
            assert!(matches!(&(*data.as_ptr().unwrap()).data, ObjData::Dict(_)));
        }
    }

    #[test]
    fn test_weak_value_dict_is_instance_with_dict_data() {
        let d = mb_weakref_weak_value_dict();
        assert_eq!(
            get_str(get_field(d, "__class__")),
            Some("WeakValueDictionary".to_string())
        );
        let data = get_field(d, "data");
        unsafe {
            assert!(matches!(&(*data.as_ptr().unwrap()).data, ObjData::Dict(_)));
        }
    }

    // -- WeakMethod --

    #[test]
    fn test_weak_method_stores_method() {
        let m = MbValue::from_int(0xdead);
        let wm = mb_weakref_weak_method(m);
        assert_eq!(
            get_str(get_field(wm, "__class__")),
            Some("WeakMethod".to_string())
        );
        assert_eq!(get_field(wm, "_method").as_int(), Some(0xdead));
    }

    // -- finalize --

    #[test]
    fn test_finalize_returns_alive_instance() {
        let obj = MbValue::from_int(1);
        let func = MbValue::from_int(2);
        let f = mb_weakref_finalize(obj, func);
        assert_eq!(
            get_str(get_field(f, "__class__")),
            Some("finalize".to_string())
        );
        assert_eq!(get_field(f, "alive").as_bool(), Some(true));
        assert_eq!(get_field(f, "_obj").as_int(), Some(1));
        assert_eq!(get_field(f, "_func").as_int(), Some(2));
    }

    // -- class stubs --

    #[test]
    fn test_class_stub_has_name_and_module() {
        let c = make_class_stub("ReferenceType");
        assert_eq!(
            get_str(get_field(c, "__name__")),
            Some("ReferenceType".to_string())
        );
        assert_eq!(
            get_str(get_field(c, "__module__")),
            Some("weakref".to_string())
        );
    }

    #[test]
    fn test_class_stubs_have_distinct_names() {
        let r = make_class_stub("ReferenceType");
        let p = make_class_stub("ProxyType");
        let c = make_class_stub("CallableProxyType");
        let k = make_class_stub("KeyedRef");
        assert_eq!(
            get_str(get_field(r, "__name__")),
            Some("ReferenceType".to_string())
        );
        assert_eq!(
            get_str(get_field(p, "__name__")),
            Some("ProxyType".to_string())
        );
        assert_eq!(
            get_str(get_field(c, "__name__")),
            Some("CallableProxyType".to_string())
        );
        assert_eq!(
            get_str(get_field(k, "__name__")),
            Some("KeyedRef".to_string())
        );
    }

    // -- registration smoke test --

    #[test]
    fn test_register_does_not_panic() {
        // Registration is a global side effect; we just ensure the call
        // sequence (building dispatchers + class stubs + ProxyTypes
        // tuple) doesn't panic and that the helper-side shape holds.
        register();
        // Re-running register() is idempotent at the test-call level
        // (it replaces the entry). Calling once is enough.
    }
}
