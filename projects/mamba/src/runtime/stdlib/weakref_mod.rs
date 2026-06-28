// HANDWRITE-BEGIN reason: #1466 — partial weakref semantics; full GC
// integration still required. Mamba now has a small refcount/GC collection
// notification path for selected proxy wrappers, so dead proxy access can
// raise ReferenceError for covered referents, and a narrow global-liveness
// sweep expires weakref.ref objects for covered module-global referents. Most
// weakref container, finalize, and general ref-expiry semantics still use
// strong stubs that preserve API shape without the full CPython contract.
// Closing the remaining gap requires broader liveness tracking.
// Tracked under #1466 (conformance(mamba/stdlib): weakref); also see legacy
// #437.
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
/// Carve-out: PARTIAL WEAK SEMANTICS.
///
/// Full weak references require broad GC integration (tracking live objects,
/// invalidating references, firing callbacks, and evicting containers when the
/// referent is collected). Mamba currently preserves API shape plus a covered
/// proxy-death and module-global ref-callback subsets:
///
///   - `ref(obj, callback=None)`: returns an Instance carrying a strong
///     pointer to `obj` for most referents. Refs expire, and callbacks fire,
///     only for the covered module-global liveness sweep.
///   - `proxy(obj, callback=None)`: returns `obj` unchanged for legacy
///     live-object alias cases. Hash-sensitive and socket referents receive a
///     proxy wrapper that can be marked dead by rc/GC collection notifications
///     and raises `ReferenceError` on non-internal access after expiry.
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
/// Tracked under #437/#1466 — promote to real weak references once the broader
/// GC / liveness-tracking work lands.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// A per-referent weak-reference registry. Mamba's runtime is still mostly
// refcount-only, so we model CPython's liveness contract for the common
// *live-object* cases by tracking the ref/proxy objects created for a referent.
// This lets the live-object semantics that the bulk of the CPython test-suite
// exercises behave correctly:
//
//   * `weakref.ref(obj)` (no callback) is *reused* — two no-callback refs to
//     the same object are the same object (`r1 is r2`).
//   * `getweakrefcount(obj)` returns the number of live ref/proxy objects.
//   * `getweakrefs(obj)` returns them in creation order.
//
// Selected proxy wrappers are also marked dead from rc/GC collection
// notifications. Ref callbacks fire for the covered module-global liveness
// sweep. General dead-ref timing and weak container eviction remain out of
// scope: that needs broader liveness work, tracked under gh #1466.
thread_local! {
    // referent identity (pointer-or-folded-bits key) -> ordered list of the
    // ref/proxy MbValue objects created against it.
    static WEAKREF_REGISTRY: std::cell::RefCell<HashMap<u64, Vec<MbValue>>> =
        std::cell::RefCell::new(HashMap::new());
    // weakref.finalize uses an internal weakref in CPython. Keep it separate
    // from getweakrefcount/getweakrefs so the public ref/proxy inventory stays
    // stable while gc.collect can still find pending finalizers.
    static FINALIZE_REGISTRY: std::cell::RefCell<HashMap<u64, Vec<MbValue>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Stable identity key for a referent. Pointer objects use their address;
/// immediates fold their bits the same way the old `_target_id` did.
fn referent_key(obj: MbValue) -> u64 {
    if let Some(ptr) = obj.as_ptr() {
        ptr as u64
    } else {
        ((obj.to_bits() >> 16) ^ obj.to_bits()) & 0x0000_7FFF_FFFF_FFFF
    }
}

/// Read the `__callback__` field of a ref/proxy Instance (None when absent).
fn ref_callback(wref: MbValue) -> MbValue {
    if let Some(ptr) = wref.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(cb) = fields.read().unwrap().get("__callback__") {
                    return *cb;
                }
            }
        }
    }
    MbValue::none()
}

/// Register a freshly-created ref/proxy object for `obj` in creation order.
fn registry_push(obj: MbValue, wref: MbValue) {
    let key = referent_key(obj);
    unsafe {
        super::super::rc::retain_if_ptr(wref);
    }
    WEAKREF_REGISTRY.with(|r| {
        r.borrow_mut().entry(key).or_default().push(wref);
    });
}

fn finalize_registry_push(obj: MbValue, fin: MbValue) {
    let key = referent_key(obj);
    unsafe {
        super::super::rc::retain_if_ptr(fin);
    }
    FINALIZE_REGISTRY.with(|r| {
        r.borrow_mut().entry(key).or_default().push(fin);
    });
}

/// Find an existing *no-callback* `ref` (class_name "ReferenceType") for `obj`.
/// CPython reuses these: `weakref.ref(o) is weakref.ref(o)`.
fn registry_find_plain_ref(obj: MbValue) -> Option<MbValue> {
    let key = referent_key(obj);
    WEAKREF_REGISTRY.with(|r| {
        r.borrow().get(&key).and_then(|v| {
            v.iter().copied().find(|&w| {
                if let Some(ptr) = w.as_ptr() {
                    unsafe {
                        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                            return class_name == "ReferenceType" && ref_callback(w).is_none();
                        }
                    }
                }
                false
            })
        })
    })
}

/// Find an existing *no-callback* proxy wrapper for `obj`.
fn registry_find_plain_proxy(obj: MbValue) -> Option<MbValue> {
    let key = referent_key(obj);
    WEAKREF_REGISTRY.with(|r| {
        r.borrow().get(&key).and_then(|v| {
            v.iter().copied().find(|&w| {
                if let Some(ptr) = w.as_ptr() {
                    unsafe {
                        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                            return matches!(class_name.as_str(), "ProxyType" | "CallableProxyType")
                                && ref_callback(w).is_none();
                        }
                    }
                }
                false
            })
        })
    })
}

fn referent_needs_proxy_hash_guard(obj: MbValue) -> bool {
    let Some(ptr) = obj.as_ptr() else { return false; };
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            return super::super::class::class_own_members(class_name)
                .into_iter()
                .any(|(name, _, _)| name == "__hash__");
        }
    }
    false
}

fn referent_needs_proxy_wrapper(obj: MbValue) -> bool {
    if is_int_backed_function_handle(obj) {
        return true;
    }
    if matches!(super::collections_mod::user_wrapper_data(obj), Some(("list", _))) {
        return true;
    }
    if referent_needs_proxy_hash_guard(obj) {
        return true;
    }
    let Some(ptr) = obj.as_ptr() else {
        return false;
    };
    unsafe {
        matches!(
            &(*ptr).data,
            ObjData::Instance { class_name, .. }
                if super::super::class::class_is_user_defined(class_name)
                    || class_name == "socket.socket"
        )
    }
}

fn is_int_backed_function_handle(obj: MbValue) -> bool {
    obj.as_int().is_some_and(|id| id >= 0)
        && (!super::super::closure::mb_closure_get_func(obj).is_none()
            || super::super::closure::mb_func_is_registered(obj))
}

fn is_proxy_instance(val: MbValue) -> bool {
    let Some(ptr) = val.as_ptr() else {
        return false;
    };
    unsafe {
        matches!(
            &(*ptr).data,
            ObjData::Instance { class_name, .. }
                if matches!(class_name.as_str(), "ProxyType" | "CallableProxyType")
        )
    }
}

fn is_internal_proxy_attr(attr_name: &str) -> bool {
    matches!(
        attr_name,
        "__class__"
            | "__callback__"
            | "_callback"
            | "_target"
            | "_target_id"
            | "_dead"
            | "_global_tracked"
            | "_target_is_function_handle"
            | "_class_object_name"
    )
}

fn target_id_addr(target_id: MbValue) -> Option<usize> {
    if let Some(addr) = target_id.as_int() {
        return Some(addr as usize);
    }
    extract_str(target_id).and_then(|s| usize::from_str_radix(&s, 16).ok())
}

fn user_class_object_name(obj: MbValue) -> Option<String> {
    let name = super::super::class::resolve_class_name(obj)?;
    super::super::class::class_is_user_defined(&name).then_some(name)
}

pub fn reference_target(wref: MbValue) -> MbValue {
    let Some(ptr) = wref.as_ptr() else {
        return MbValue::none();
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "ReferenceType" => {
                let fields = fields.read().unwrap();
                if fields
                    .get("_dead")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return MbValue::none();
                }
                if let Some(target) = fields
                    .get("_target")
                    .copied()
                    .filter(|target| !target.is_none())
                {
                    super::super::rc::retain_if_ptr(target);
                    return target;
                }
                if let Some(target) = fields
                    .get("_target_id")
                    .copied()
                    .and_then(target_id_addr)
                    .map(|addr| MbValue::from_ptr(addr as *mut MbObject))
                {
                    super::super::rc::retain_if_ptr(target);
                    return target;
                }
                MbValue::none()
            }
            _ => MbValue::none(),
        }
    }
}

fn class_ref_target_name(wref: MbValue) -> Option<String> {
    let ptr = wref.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "ReferenceType" => {
                fields
                    .read()
                    .unwrap()
                    .get("_class_object_name")
                    .copied()
                    .and_then(extract_str)
            }
            _ => None,
        }
    }
}

fn class_ref_has_live_global(target: MbValue, target_name: &str) -> bool {
    super::super::closure::snapshot_global_id_namespace()
        .values()
        .copied()
        .any(|value| {
            if value.to_bits() == target.to_bits() {
                return true;
            }
            super::super::class::resolve_class_name(value).as_deref() == Some(target_name)
        })
}

fn value_has_live_global(target: MbValue) -> bool {
    super::super::closure::snapshot_global_id_namespace()
        .values()
        .copied()
        .any(|value| value.to_bits() == target.to_bits())
}

pub(crate) fn expire_unbound_class_refs() {
    let refs: Vec<MbValue> = WEAKREF_REGISTRY.with(|registry| {
        registry
            .borrow()
            .values()
            .flat_map(|items| items.iter().copied())
            .collect()
    });
    for wref in refs {
        let Some(target_name) = class_ref_target_name(wref) else {
            continue;
        };
        let target = reference_target(wref);
        if target.is_none() {
            mark_weakref_dead(wref);
            continue;
        }
        if !class_ref_has_live_global(target, &target_name) {
            mark_weakref_dead(wref);
        }
        unsafe {
            super::super::rc::release_if_ptr(target);
        }
    }
}

pub(crate) fn expire_unbound_ref_callbacks() {
    let refs: Vec<MbValue> = WEAKREF_REGISTRY.with(|registry| {
        registry
            .borrow()
            .values()
            .flat_map(|items| items.iter().copied())
            .collect()
    });
    for wref in refs {
        if !ref_tracks_global_liveness(wref) {
            continue;
        }
        let target = reference_target(wref);
        if target.is_none() {
            if mark_weakref_dead(wref) {
                fire_ref_callback(wref);
            }
            continue;
        }
        if !value_has_live_global(target) && mark_weakref_dead(wref) {
            fire_ref_callback(wref);
        }
        unsafe {
            super::super::rc::release_if_ptr(target);
        }
    }
}

fn finalize_is_alive(fin: MbValue) -> bool {
    let Some(ptr) = fin.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "finalize" => fields
                .read()
                .unwrap()
                .get("alive")
                .copied()
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            _ => false,
        }
    }
}

fn finalize_target(fin: MbValue) -> Option<MbValue> {
    let ptr = fin.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "finalize" => fields
                .read()
                .unwrap()
                .get("_obj")
                .copied()
                .filter(|target| !target.is_none()),
            _ => None,
        }
    }
}

fn finalizer_target_collectible(target: MbValue) -> bool {
    !value_has_live_global(target)
}

fn run_finalize_once(fin: MbValue) {
    if !finalize_is_alive(fin) {
        return;
    }
    unsafe {
        finalize_call(fin, MbValue::none());
    }
}

fn proxy_tracks_global_liveness(proxy: MbValue) -> bool {
    let Some(ptr) = proxy.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { fields, .. } => fields
                .read()
                .unwrap()
                .get("_global_tracked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            _ => false,
        }
    }
}

pub(crate) fn expire_unbound_finalizers() {
    let finalizers: Vec<MbValue> = FINALIZE_REGISTRY.with(|registry| {
        registry
            .borrow()
            .values()
            .flat_map(|items| items.iter().copied())
            .collect()
    });
    for fin in finalizers {
        let Some(target) = finalize_target(fin) else {
            continue;
        };
        if finalizer_target_collectible(target) {
            run_finalize_once(fin);
        }
    }
}

pub(crate) fn expire_unbound_proxies() {
    let refs: Vec<MbValue> = WEAKREF_REGISTRY.with(|registry| {
        registry
            .borrow()
            .values()
            .flat_map(|items| items.iter().copied())
            .collect()
    });
    for wref in refs {
        if !is_proxy_instance(wref) || proxy_is_dead(wref) {
            continue;
        }
        if !proxy_tracks_global_liveness(wref) {
            continue;
        }
        let Some(target) = proxy_target(wref) else {
            mark_weakref_dead(wref);
            continue;
        };
        if !value_has_live_global(target) {
            mark_weakref_dead(wref);
        }
    }
}

pub fn proxy_target(proxy: MbValue) -> Option<MbValue> {
    let ptr = proxy.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields }
                if matches!(class_name.as_str(), "ProxyType" | "CallableProxyType") =>
            {
                let fields = fields.read().unwrap();
                if fields
                    .get("_dead")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    return None;
                }
                if let Some(target) = fields
                    .get("_target")
                    .copied()
                    .filter(|target| !target.is_none())
                {
                    return Some(target);
                }
                fields
                    .get("_target_id")
                    .copied()
                    .and_then(target_id_addr)
                    .map(|addr| MbValue::from_ptr(addr as *mut MbObject))
            }
            _ => None,
        }
    }
}

pub(crate) fn proxy_is_dead(proxy: MbValue) -> bool {
    let Some(ptr) = proxy.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields }
                if matches!(class_name.as_str(), "ProxyType" | "CallableProxyType") =>
            {
                fields
                    .read()
                    .unwrap()
                    .get("_dead")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
}

pub(crate) fn proxy_target_or_raise(proxy: MbValue) -> Option<MbValue> {
    if is_proxy_instance(proxy) && proxy_is_dead(proxy) {
        raise_reference_error();
        return Some(MbValue::none());
    }
    proxy_target(proxy)
}

pub(crate) fn proxy_references_int_function(proxy: MbValue) -> bool {
    let Some(ptr) = proxy.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields }
                if matches!(class_name.as_str(), "ProxyType" | "CallableProxyType") =>
            {
                fields
                    .read()
                    .unwrap()
                    .get("_target_is_function_handle")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
}

pub(crate) fn proxy_dead_attr_access(proxy: MbValue, attr_name: &str) -> bool {
    is_proxy_instance(proxy) && proxy_is_dead(proxy) && !is_internal_proxy_attr(attr_name)
}

pub(crate) fn raise_reference_error() {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ReferenceError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "weakly-referenced object no longer exists".to_string(),
        )),
    );
}

fn mark_weakref_dead(wref: MbValue) -> bool {
    let Some(ptr) = wref.as_ptr() else {
        return false;
    };
    unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if !matches!(
                class_name.as_str(),
                "ReferenceType" | "ProxyType" | "CallableProxyType"
            ) {
                return false;
            }
            let mut fields = fields.write().unwrap();
            if fields
                .get("_dead")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                return false;
            }
            if fields.contains_key("_target") {
                fields.insert("_target".to_string(), MbValue::none());
            }
            if let Some(dead) = fields.get_mut("_dead") {
                *dead = MbValue::from_bool(true);
            } else {
                fields.insert("_dead".to_string(), MbValue::from_bool(true));
            }
            return true;
        }
    }
    false
}

fn ref_tracks_global_liveness(wref: MbValue) -> bool {
    let Some(ptr) = wref.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "ReferenceType" => fields
                .read()
                .unwrap()
                .get("_global_tracked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            _ => false,
        }
    }
}

fn fire_ref_callback(wref: MbValue) {
    let callback = ref_callback(wref);
    if callback.is_none() {
        return;
    }
    let _ = super::super::class::mb_call_method1(callback, wref);
}

pub(crate) unsafe fn notify_referent_collected(obj: *mut MbObject) {
    if obj.is_null() {
        return;
    }
    let referent = MbValue::from_ptr(obj);
    let key = referent_key(referent);
    let weakrefs = WEAKREF_REGISTRY.with(|r| r.borrow_mut().remove(&key));
    if let Some(weakrefs) = weakrefs {
        for wref in weakrefs {
            mark_weakref_dead(wref);
            // Do not release the registry's retained weakref/proxy during the
            // referent's deallocation path. rc/GC sweep can already be walking
            // the same object graph, and re-entering proxy field cleanup here is
            // not safe until weakref liveness is fully owned by GC.
        }
    }
    let finalizers = FINALIZE_REGISTRY.with(|r| r.borrow_mut().remove(&key));
    if let Some(finalizers) = finalizers {
        for fin in finalizers {
            run_finalize_once(fin);
            // The finalize registry owns a retain like WEAKREF_REGISTRY. Keep
            // it during deallocation to avoid re-entering Instance cleanup from
            // the referent's release path.
        }
    }
}

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
        "__qualname__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    // CPython: WeakSet is defined in the `_weakrefset` helper module; all the
    // other public weakref classes report `weakref` as their module.
    let module_name = if name == "WeakSet" {
        "_weakrefset"
    } else {
        "weakref"
    };
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str(module_name.to_string())),
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
    let fin = mb_weakref_finalize(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    );
    // Trailing call args (beyond obj+func) are stored for the eventual call.
    if a.len() > 2 {
        if let Some(ptr) = fin.as_ptr() {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(
                    "_args".to_string(),
                    MbValue::from_ptr(MbObject::new_list(a[2..].to_vec())),
                );
            }
        }
    }
    fin
}

/// `finalize.__call__` — first call runs func(*args), flips alive to False,
/// and returns the result; later calls return None.
unsafe extern "C" fn finalize_call(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(ptr) = self_v.as_ptr() else {
        return MbValue::none();
    };
    let (func, call_args, alive) = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            (
                f.get("_func").copied().unwrap_or_else(MbValue::none),
                f.get("_args")
                    .copied()
                    .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_list(Vec::new()))),
                f.get("alive")
                    .copied()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            )
        } else {
            return MbValue::none();
        }
    };
    if !alive {
        return MbValue::none();
    }
    let old_obj = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let mut fields = fields.write().unwrap();
            fields.insert("alive".to_string(), MbValue::from_bool(false));
            fields.insert("_obj".to_string(), MbValue::none())
        } else {
            None
        }
    };
    unsafe {
        if let Some(old_obj) = old_obj {
            super::super::rc::release_if_ptr(old_obj);
        }
    }
    super::super::builtins::mb_call_spread(func, call_args)
}

/// `finalize.detach()` / `finalize.peek()` — report (obj, func, args, kwargs)
/// while alive; detach also flips alive.
unsafe extern "C" fn finalize_detach(self_v: MbValue, _args: MbValue) -> MbValue {
    let r = finalize_peek(self_v, _args);
    if let Some(ptr) = self_v.as_ptr() {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields
                .write()
                .unwrap()
                .insert("alive".to_string(), MbValue::from_bool(false));
        }
    }
    r
}

unsafe extern "C" fn finalize_peek(self_v: MbValue, _args: MbValue) -> MbValue {
    let Some(ptr) = self_v.as_ptr() else {
        return MbValue::none();
    };
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            if f.get("alive").copied().and_then(|v| v.as_bool()) != Some(true) {
                return MbValue::none();
            }
            let obj = f.get("_obj").copied().unwrap_or_else(MbValue::none);
            let func = f.get("_func").copied().unwrap_or_else(MbValue::none);
            let args = f
                .get("_args")
                .copied()
                .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_list(Vec::new())));
            return MbValue::from_ptr(MbObject::new_tuple(vec![
                obj,
                func,
                args,
                MbValue::from_ptr(MbObject::new_dict()),
            ]));
        }
    }
    MbValue::none()
}

/// `ReferenceType.__init__` re-invocation: validate arity like CPython.
unsafe extern "C" fn reference_init(_self_v: MbValue, args: MbValue) -> MbValue {
    let n = args
        .as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().map(|g| g.len()).unwrap_or(0)
            } else {
                0
            }
        })
        .unwrap_or(0);
    if n > 2 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "__init__ expected at most 2 arguments, got {n}"
            ))),
        );
    }
    MbValue::none()
}

// ── WeakSet / WeakValueDictionary instance methods ──

fn wk_data(self_v: MbValue) -> MbValue {
    self_v
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().ok()?.get("data").copied()
            } else {
                None
            }
        })
        .unwrap_or_else(MbValue::none)
}

fn wk_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().ok().map(|g| g.to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn prune_weak_set_data(data: MbValue) {
    let Some(ptr) = data.as_ptr() else {
        return;
    };
    let stale = unsafe {
        if let ObjData::Set(ref lock) = (*ptr).data {
            lock.read()
                .unwrap()
                .iter()
                .copied()
                .filter(|item| !value_has_live_global(*item))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };
    if stale.is_empty() {
        return;
    }
    unsafe {
        if let ObjData::Set(ref lock) = (*ptr).data {
            let mut set = lock.write().unwrap();
            for item in stale {
                if let Some(removed) = set.set_remove(item) {
                    super::super::rc::release_if_ptr(removed);
                }
            }
        }
    }
}

fn ws_data_pruned(self_v: MbValue) -> MbValue {
    let data = wk_data(self_v);
    prune_weak_set_data(data);
    data
}

unsafe extern "C" fn ws_add(self_v: MbValue, args: MbValue) -> MbValue {
    let item = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    if reject_non_weakreferenceable(item) {
        return MbValue::none();
    }
    super::super::set_ops::mb_set_add(ws_data_pruned(self_v), item);
    MbValue::none()
}

unsafe extern "C" fn ws_discard(self_v: MbValue, args: MbValue) -> MbValue {
    let item = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::set_ops::mb_set_discard(ws_data_pruned(self_v), item);
    MbValue::none()
}

unsafe extern "C" fn ws_remove(self_v: MbValue, args: MbValue) -> MbValue {
    let item = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::set_ops::mb_set_remove(ws_data_pruned(self_v), item);
    MbValue::none()
}

unsafe extern "C" fn ws_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let item = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::set_ops::mb_set_contains(ws_data_pruned(self_v), item)
}

unsafe extern "C" fn ws_len(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::set_ops::mb_set_len(ws_data_pruned(self_v))
}

fn ws_dispatch_set_method(name: &str, self_v: MbValue, args: MbValue) -> MbValue {
    super::super::set_ops::dispatch_set_method(name, ws_data_pruned(self_v), args)
}

unsafe extern "C" fn ws_clear(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("clear", self_v, args)
}

unsafe extern "C" fn ws_copy(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("copy", self_v, args)
}

unsafe extern "C" fn ws_difference(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("difference", self_v, args)
}

unsafe extern "C" fn ws_difference_update(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("difference_update", self_v, args)
}

unsafe extern "C" fn ws_intersection(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("intersection", self_v, args)
}

unsafe extern "C" fn ws_intersection_update(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("intersection_update", self_v, args)
}

unsafe extern "C" fn ws_isdisjoint(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("isdisjoint", self_v, args)
}

unsafe extern "C" fn ws_issubset(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("issubset", self_v, args)
}

unsafe extern "C" fn ws_issuperset(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("issuperset", self_v, args)
}

unsafe extern "C" fn ws_pop(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("pop", self_v, args)
}

unsafe extern "C" fn ws_symmetric_difference(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("symmetric_difference", self_v, args)
}

unsafe extern "C" fn ws_symmetric_difference_update(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("symmetric_difference_update", self_v, args)
}

unsafe extern "C" fn ws_union(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("union", self_v, args)
}

unsafe extern "C" fn ws_update(self_v: MbValue, args: MbValue) -> MbValue {
    ws_dispatch_set_method("update", self_v, args)
}

unsafe extern "C" fn wvd_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let a = wk_args(args);
    let key = a.first().copied().unwrap_or_else(MbValue::none);
    let val = a.get(1).copied().unwrap_or_else(MbValue::none);
    if reject_non_weakreferenceable(val) {
        return MbValue::none();
    }
    super::super::dict_ops::mb_dict_setitem(wvd_data_pruned(self_v), key, val);
    MbValue::none()
}

unsafe extern "C" fn wvd_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    let sentinel = MbValue::from_bits(u64::MAX);
    let found = super::super::dict_ops::mb_dict_get(wvd_data_pruned(self_v), key, sentinel);
    if found.to_bits() == u64::MAX {
        let key_repr = super::super::builtins::mb_repr(key);
        let ks = key_repr
            .as_ptr()
            .and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
            MbValue::from_ptr(MbObject::new_str(ks)),
        );
        return MbValue::none();
    }
    found
}

unsafe extern "C" fn wvd_get(self_v: MbValue, args: MbValue) -> MbValue {
    let a = wk_args(args);
    let key = a.first().copied().unwrap_or_else(MbValue::none);
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_get(wvd_data_pruned(self_v), key, default)
}

unsafe extern "C" fn wvd_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_contains(wvd_data_pruned(self_v), key)
}

unsafe extern "C" fn wvd_len(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_len(wvd_data_pruned(self_v))
}

unsafe extern "C" fn wvd_delitem(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_delitem(wvd_data_pruned(self_v), key);
    MbValue::none()
}

unsafe extern "C" fn wvd_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(wvd_data_pruned(self_v))
}

unsafe extern "C" fn wvd_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(wvd_data_pruned(self_v))
}

unsafe extern "C" fn wvd_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(wvd_data_pruned(self_v))
}

fn prune_weak_value_dict_data(data: MbValue) {
    let Some(ptr) = data.as_ptr() else {
        return;
    };
    let stale_keys = unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read()
                .unwrap()
                .iter()
                .filter_map(|(key, value)| {
                    (!value_has_live_global(*value))
                        .then(|| super::super::dict_ops::dict_key_to_mbvalue(key))
                })
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    };
    for key in stale_keys {
        super::super::dict_ops::mb_dict_delitem(data, key);
        unsafe {
            super::super::rc::release_if_ptr(key);
        }
    }
}

fn wvd_data_pruned(self_v: MbValue) -> MbValue {
    let data = wk_data(self_v);
    prune_weak_value_dict_data(data);
    data
}

fn weak_key_dict_key_is_live(key: &super::super::dict_ops::DictKey) -> bool {
    let key_val = super::super::dict_ops::dict_key_to_mbvalue(key);
    let live = value_has_live_global(key_val);
    unsafe {
        super::super::rc::release_if_ptr(key_val);
    }
    live
}

fn prune_weak_key_dict_data(data: MbValue) {
    let Some(ptr) = data.as_ptr() else {
        return;
    };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.write()
                .unwrap()
                .retain(|key, _| weak_key_dict_key_is_live(key));
        }
    }
}

fn wkd_data_pruned(self_v: MbValue) -> MbValue {
    let data = wk_data(self_v);
    prune_weak_key_dict_data(data);
    data
}

unsafe extern "C" fn wkd_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let a = wk_args(args);
    let key = a.first().copied().unwrap_or_else(MbValue::none);
    let val = a.get(1).copied().unwrap_or_else(MbValue::none);
    if reject_non_weakreferenceable(key) {
        return MbValue::none();
    }
    super::super::dict_ops::mb_dict_setitem(wkd_data_pruned(self_v), key, val);
    MbValue::none()
}

unsafe extern "C" fn wkd_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    let sentinel = MbValue::from_bits(u64::MAX);
    let found = super::super::dict_ops::mb_dict_get(wkd_data_pruned(self_v), key, sentinel);
    if found.to_bits() == u64::MAX {
        let key_repr = super::super::builtins::mb_repr(key);
        let ks = key_repr
            .as_ptr()
            .and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
            MbValue::from_ptr(MbObject::new_str(ks)),
        );
        return MbValue::none();
    }
    found
}

unsafe extern "C" fn wkd_get(self_v: MbValue, args: MbValue) -> MbValue {
    let a = wk_args(args);
    let key = a.first().copied().unwrap_or_else(MbValue::none);
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_get(wkd_data_pruned(self_v), key, default)
}

unsafe extern "C" fn wkd_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_contains(wkd_data_pruned(self_v), key)
}

unsafe extern "C" fn wkd_len(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_len(wkd_data_pruned(self_v))
}

unsafe extern "C" fn wkd_delitem(self_v: MbValue, args: MbValue) -> MbValue {
    let key = wk_args(args).first().copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_delitem(wkd_data_pruned(self_v), key);
    MbValue::none()
}

unsafe extern "C" fn wkd_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(wkd_data_pruned(self_v))
}

unsafe extern "C" fn wkd_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(wkd_data_pruned(self_v))
}

unsafe extern "C" fn wkd_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(wkd_data_pruned(self_v))
}

// ---------------------------------------------------------------------------
// Helpers (public surface implementations)
// ---------------------------------------------------------------------------

/// weakref.ref(obj, callback=None) -> Instance wrapping a strong ref.
///
/// Carve-out: holds a strong pointer; never expires.
/// CPython: plain builtins (int/float/bool/None/str/bytes/tuple/list/dict)
/// are not weak-referenceable; raise TypeError with the type name.
fn reject_non_weakreferenceable(obj: MbValue) -> bool {
    let type_name: Option<&str> = if obj.is_none() {
        Some("NoneType")
    } else if obj.is_bool() {
        Some("bool")
    } else if obj
        .as_int()
        .is_some_and(|id| {
            id < 0
                || !(super::uuid_mod::is_uuid_handle(id as u64)
                    || super::super::generator::is_known_generator(obj)
                    || super::super::iter::mb_is_iterator_handle(obj)
                    || is_int_backed_function_handle(obj))
        })
    {
        Some("int")
    } else if obj.is_float() {
        Some("float")
    } else if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Str(ref s) => {
                    // mamba models class objects as their name strings; a
                    // registered class (or builtin exception type) IS
                    // weak-referenceable in CPython.
                    if super::super::class::class_is_registered(s)
                        || super::super::exception::is_subclass_of(s, "BaseException")
                        || s == "Exception"
                        || s == "BaseException"
                    {
                        None
                    } else {
                        Some("str")
                    }
                }
                ObjData::Bytes(_) => Some("bytes"),
                ObjData::ByteArray(_) => Some("bytearray"),
                ObjData::Tuple(_) => Some("tuple"),
                ObjData::List(_) => Some("list"),
                ObjData::Dict(_) => Some("dict"),
                _ => None,
            }
        }
    } else {
        None
    };
    if let Some(name) = type_name {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "cannot create weak reference to '{name}' object"
            ))),
        );
        return true;
    }
    false
}

pub fn mb_weakref_ref(obj: MbValue, callback: MbValue) -> MbValue {
    if reject_non_weakreferenceable(obj) {
        return MbValue::none();
    }
    // CPython reuses no-callback refs: `weakref.ref(o) is weakref.ref(o)` and
    // `weakref.ref(o, None) is weakref.ref(o)`. Refs created with a real
    // callback are always distinct.
    if callback.is_none() {
        if let Some(existing) = registry_find_plain_ref(obj) {
            unsafe {
                super::super::rc::retain_if_ptr(existing);
            }
            return existing;
        }
    }
    let target_id = MbValue::from_int(referent_key(obj) as i64);
    let class_object_name = user_class_object_name(obj);
    let mut fields = FxHashMap::default();
    fields.insert("_target".to_string(), obj);
    fields.insert("_target_id".to_string(), target_id);
    fields.insert("_dead".to_string(), MbValue::from_bool(false));
    fields.insert(
        "_global_tracked".to_string(),
        MbValue::from_bool(value_has_live_global(obj)),
    );
    if let Some(name) = class_object_name {
        fields.insert(
            "_class_object_name".to_string(),
            MbValue::from_ptr(MbObject::new_str(name)),
        );
    }
    // `__callback__` is the public CPython attribute (None when no callback);
    // `_callback` kept as a legacy alias.
    fields.insert("__callback__".to_string(), callback);
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
    let wref = MbValue::from_ptr(Box::into_raw(obj_inst));
    registry_push(obj, wref);
    wref
}

/// weakref.deref(wref) -> referent or None (stub).
///
/// Legacy mamba surface (not part of CPython's weakref module). Real
/// CPython users call the ref like a function: `r()`. We keep this
/// helper for backward compatibility; it now returns the stored
/// live target, or None after a target-id-only weak ref expires.
pub fn mb_weakref_deref(wref: MbValue) -> MbValue {
    reference_target(wref)
}

/// weakref.proxy(obj, callback=None) -> proxy wrapper for referents whose
/// observable contract needs proxy identity/dead-target checks; otherwise keep
/// the legacy live-object alias carve-out.
pub fn mb_weakref_proxy(obj: MbValue, callback: MbValue) -> MbValue {
    if reject_non_weakreferenceable(obj) {
        return MbValue::none();
    }
    if referent_needs_proxy_wrapper(obj) {
        if callback.is_none() {
            if let Some(existing) = registry_find_plain_proxy(obj) {
                unsafe { super::super::rc::retain_if_ptr(existing); }
                return existing;
            }
        }
        let class_name = if super::super::builtins::mb_callable(obj)
            .as_bool()
            .unwrap_or(false)
        {
            "CallableProxyType"
        } else {
            "ProxyType"
        };
        let function_handle = is_int_backed_function_handle(obj);
        let target_id = MbValue::from_ptr(MbObject::new_str(format!("{:012x}", referent_key(obj))));
        let mut fields = FxHashMap::default();
        fields.insert("_target_id".to_string(), target_id);
        if function_handle {
            fields.insert("_target".to_string(), obj);
            fields.insert(
                "_target_is_function_handle".to_string(),
                MbValue::from_bool(true),
            );
        }
        fields.insert("_dead".to_string(), MbValue::from_bool(false));
        fields.insert(
            "_global_tracked".to_string(),
            MbValue::from_bool(value_has_live_global(obj)),
        );
        fields.insert("__callback__".to_string(), callback);
        fields.insert("_callback".to_string(), callback);
        fields.insert(
            "__class__".to_string(),
            MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
        );
        let proxy = Box::new(MbObject {
            header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
            data: ObjData::Instance {
                class_name: class_name.to_string(),
                fields: RwLock::new(fields),
            },
        });
        let proxy = MbValue::from_ptr(Box::into_raw(proxy));
        registry_push(obj, proxy);
        return proxy;
    }
    // The proxy carve-out returns the referent itself. The argument cleanup and
    // returned alias each consume an owned slot, so keep both references alive.
    unsafe { super::super::rc::retain_if_ptr(obj); }
    unsafe { super::super::rc::retain_if_ptr(obj); }
    obj
}

/// CPython-style repr for a `weakref.ref` (ReferenceType) instance:
///   `<weakref at 0xADDR; to 'CLASS' at 0xADDR>`
/// Names the referent's class (gh-99184: even when the referent overrides
/// `__getattr__`). Returns `None`-MbValue for non-ReferenceType instances so
/// the generic repr path can take over.
pub fn reference_repr(wref: MbValue) -> MbValue {
    let Some(wptr) = wref.as_ptr() else {
        return MbValue::none();
    };
    let target = unsafe {
        match &(*wptr).data {
            ObjData::Instance { class_name, fields } if class_name == "ReferenceType" => fields
                .read()
                .unwrap()
                .get("_target")
                .copied()
                .unwrap_or_else(MbValue::none),
            _ => return MbValue::none(),
        }
    };
    let cls = referent_type_name(target);
    let s = if target.is_none() {
        // Dead referent (not reachable under the strong-hold carve-out, but
        // kept for completeness): CPython prints `(dead)`.
        format!("<weakref at 0x{:x}; dead>", wref.to_bits())
    } else {
        format!(
            "<weakref at 0x{:x}; to '{}' at 0x{:x}>",
            wref.to_bits(),
            cls,
            target
                .as_ptr()
                .map(|p| p as u64)
                .unwrap_or_else(|| target.to_bits()),
        )
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

/// Best-effort type name of a weakref referent, used for repr.
fn referent_type_name(target: MbValue) -> String {
    if let Some(ptr) = target.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, .. } => return class_name.clone(),
                ObjData::Str(_) => return "str".to_string(),
                ObjData::Bytes(_) => return "bytes".to_string(),
                ObjData::List(_) => return "list".to_string(),
                ObjData::Dict(_) => return "dict".to_string(),
                ObjData::Set(_) => return "set".to_string(),
                ObjData::Tuple(_) => return "tuple".to_string(),
                _ => {}
            }
        }
    }
    if target
        .as_int()
        .is_some_and(|id| id >= 0 && super::uuid_mod::is_uuid_handle(id as u64))
    {
        return "UUID".to_string();
    }
    if target.as_int().is_some() {
        return "int".to_string();
    }
    if target.as_float().is_some() {
        return "float".to_string();
    }
    if target.as_bool().is_some() {
        return "bool".to_string();
    }
    "object".to_string()
}

fn weakref_entry_is_publicly_live(
    wref: MbValue,
    globals: &std::collections::HashMap<i64, MbValue>,
) -> bool {
    if !globals
        .values()
        .copied()
        .any(|value| value.to_bits() == wref.to_bits())
    {
        return false;
    }
    let Some(ptr) = wref.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields }
                if matches!(
                    class_name.as_str(),
                    "ReferenceType" | "ProxyType" | "CallableProxyType"
                ) =>
            {
                !fields
                    .read()
                    .unwrap()
                    .get("_dead")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
}

fn live_registry_items(obj: MbValue) -> Vec<MbValue> {
    let key = referent_key(obj);
    let globals = super::super::closure::snapshot_global_id_namespace();
    WEAKREF_REGISTRY.with(|r| {
        r.borrow()
            .get(&key)
            .map(|items| {
                items
                    .iter()
                    .copied()
                    .filter(|wref| weakref_entry_is_publicly_live(*wref, &globals))
                    .collect()
            })
            .unwrap_or_default()
    })
}

/// weakref.getweakrefcount(obj) -> number of live ref/proxy objects.
pub fn mb_weakref_getweakrefcount(obj: MbValue) -> MbValue {
    MbValue::from_int(live_registry_items(obj).len() as i64)
}

/// weakref.getweakrefs(obj) -> list of live ref/proxy objects (creation order).
pub fn mb_weakref_getweakrefs(obj: MbValue) -> MbValue {
    let items = live_registry_items(obj);
    for &w in &items {
        unsafe {
            super::super::rc::retain_if_ptr(w);
        }
    }
    MbValue::from_ptr(MbObject::new_list(items))
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
    // CPython WeakMethod requires a bound method; a plain function raises.
    if method.as_func().is_some() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "argument should be a bound method, not <class 'function'>".to_string(),
            )),
        );
        return MbValue::none();
    }
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
pub fn mb_weakref_finalize(obj: MbValue, func: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(obj);
        super::super::rc::retain_if_ptr(func);
    }
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
    let fin = MbValue::from_ptr(Box::into_raw(obj_inst));
    finalize_registry_push(obj, fin);
    fin
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

fn register_weakref_classes() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    let mut fin: Map<String, MbValue> = Map::new();
    fin.insert("__call__".into(), var(finalize_call as *const () as usize));
    fin.insert("detach".into(), var(finalize_detach as *const () as usize));
    fin.insert("peek".into(), var(finalize_peek as *const () as usize));
    super::super::class::mb_class_register("finalize", vec![], fin);

    let mut rt: Map<String, MbValue> = Map::new();
    rt.insert("__init__".into(), var(reference_init as *const () as usize));
    super::super::class::mb_class_register("ReferenceType", vec![], rt);

    let mut ws: Map<String, MbValue> = Map::new();
    let ws_methods: &[(&str, usize)] = &[
        ("__contains__", ws_contains as usize),
        ("__len__", ws_len as usize),
        ("add", ws_add as usize),
        ("clear", ws_clear as usize),
        ("copy", ws_copy as usize),
        ("difference", ws_difference as usize),
        ("difference_update", ws_difference_update as usize),
        ("discard", ws_discard as usize),
        ("intersection", ws_intersection as usize),
        ("intersection_update", ws_intersection_update as usize),
        ("isdisjoint", ws_isdisjoint as usize),
        ("issubset", ws_issubset as usize),
        ("issuperset", ws_issuperset as usize),
        ("pop", ws_pop as usize),
        ("remove", ws_remove as usize),
        ("symmetric_difference", ws_symmetric_difference as usize),
        (
            "symmetric_difference_update",
            ws_symmetric_difference_update as usize,
        ),
        ("union", ws_union as usize),
        ("update", ws_update as usize),
    ];
    for (name, addr) in ws_methods {
        ws.insert((*name).into(), var(*addr));
    }
    super::super::class::mb_class_register("WeakSet", vec![], ws);

    let wvd_methods: &[(&str, usize)] = &[
        ("__setitem__", wvd_setitem as usize),
        ("__getitem__", wvd_getitem as usize),
        ("__delitem__", wvd_delitem as usize),
        ("__contains__", wvd_contains as usize),
        ("__len__", wvd_len as usize),
        ("get", wvd_get as usize),
        ("keys", wvd_keys as usize),
        ("values", wvd_values as usize),
        ("items", wvd_items as usize),
    ];
    let wkd_methods: &[(&str, usize)] = &[
        ("__setitem__", wkd_setitem as usize),
        ("__getitem__", wkd_getitem as usize),
        ("__delitem__", wkd_delitem as usize),
        ("__contains__", wkd_contains as usize),
        ("__len__", wkd_len as usize),
        ("get", wkd_get as usize),
        ("keys", wkd_keys as usize),
        ("values", wkd_values as usize),
        ("items", wkd_items as usize),
    ];
    for (cls, methods) in [
        ("WeakValueDictionary", wvd_methods),
        ("WeakKeyDictionary", wkd_methods),
    ] {
        let mut m: Map<String, MbValue> = Map::new();
        for (name, addr) in methods {
            m.insert((*name).to_string(), var(*addr));
        }
        super::super::class::mb_class_register(cls, vec![], m);
    }
}

/// Register the weakref module.
pub fn register() {
    register_weakref_classes();
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
        let fval = MbValue::from_func(addr);
        attrs.insert(name.to_string(), fval);
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        // Make `func.__name__` / `__qualname__` / `__module__` resolve so the
        // constructor dispatchers (which model class objects like
        // `weakref.WeakSet`) report clean type metadata.
        super::super::closure::mb_func_set_name(
            fval,
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
        );
        super::super::closure::mb_func_set_module(
            fval,
            MbValue::from_ptr(MbObject::new_str("weakref".to_string())),
        );
    }

    // Constructor dispatchers double as type objects: register their pointers
    // so `isinstance(x, weakref.ref)` / `type(x) is weakref.WeakSet` resolve.
    // `weakref.ref is weakref.ReferenceType`, so `ref` maps to "ReferenceType".
    let type_dispatchers: &[(&str, usize)] = &[
        ("ReferenceType", dispatch_ref as *const () as usize),
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
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        for (name, addr) in type_dispatchers {
            map.insert(*addr as u64, name.to_string());
        }
    });

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
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

    static FINALIZE_CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
    static REF_CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
    static REF_CALLBACK_ARG_BITS: AtomicU64 = AtomicU64::new(0);

    unsafe extern "C" fn finalize_test_callback(
        _args_ptr: *const MbValue,
        _nargs: usize,
    ) -> MbValue {
        FINALIZE_CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
        MbValue::none()
    }

    fn finalize_test_func() -> MbValue {
        let addr = finalize_test_callback as *const () as usize;
        crate::runtime::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    }

    extern "C" fn ref_test_callback(wref: MbValue) -> MbValue {
        REF_CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
        REF_CALLBACK_ARG_BITS.store(wref.to_bits(), Ordering::SeqCst);
        MbValue::none()
    }

    fn registered_ref_test_callback() -> MbValue {
        let addr = ref_test_callback as *const () as usize;
        let callback = MbValue::from_func(addr);
        let mut methods = HashMap::new();
        methods.insert("__call__".to_string(), callback);
        crate::runtime::class::mb_class_register("WeakRefCallbackHook", vec![], methods);
        callback
    }

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

    /// Weak-referenceable test target: a plain Instance (ints/strs are
    /// correctly rejected by reject_non_weakreferenceable now).
    fn target() -> MbValue {
        MbValue::from_ptr(MbObject::new_instance("WeakTarget".to_string()))
    }

    // -- ref --

    #[test]
    fn test_ref_creates_instance_with_target() {
        let obj = target();
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
        let obj = target();
        let cb = MbValue::from_int(42);
        let wref = mb_weakref_ref(obj, cb);
        assert_eq!(get_field(wref, "_callback").as_int(), Some(42));
    }

    #[test]
    fn test_ref_callback_fires_once_after_global_rebind() {
        REF_CALLBACK_COUNT.store(0, Ordering::SeqCst);
        REF_CALLBACK_ARG_BITS.store(0, Ordering::SeqCst);

        let obj = target();
        let global_id = MbValue::from_bits(9021);
        crate::runtime::closure::mb_global_set_id(global_id, obj);
        let wref = mb_weakref_ref(obj, registered_ref_test_callback());

        expire_unbound_ref_callbacks();
        assert_eq!(REF_CALLBACK_COUNT.load(Ordering::SeqCst), 0);

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        expire_unbound_ref_callbacks();
        assert_eq!(REF_CALLBACK_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(REF_CALLBACK_ARG_BITS.load(Ordering::SeqCst), wref.to_bits());
        assert!(reference_target(wref).is_none());

        expire_unbound_ref_callbacks();
        assert_eq!(REF_CALLBACK_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_plain_ref_expires_after_global_rebind() {
        let obj = target();
        let global_id = MbValue::from_bits(9022);
        crate::runtime::closure::mb_global_set_id(global_id, obj);
        let wref = mb_weakref_ref(obj, MbValue::none());

        expire_unbound_ref_callbacks();
        let alive = reference_target(wref);
        assert_eq!(alive.as_ptr(), obj.as_ptr());
        unsafe {
            crate::runtime::rc::release_if_ptr(alive);
        }

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        expire_unbound_ref_callbacks();
        assert!(reference_target(wref).is_none());
    }

    #[test]
    fn test_deref_returns_target() {
        // Carve-out: deref returns the strong-ref target, not None.
        let obj = target();
        let wref = mb_weakref_ref(obj, MbValue::none());
        let r = mb_weakref_deref(wref);
        assert_eq!(r.as_ptr(), obj.as_ptr());
    }

    #[test]
    fn test_class_object_ref_expires_after_global_rebind() {
        let class_name = "WeakRefClassTarget";
        crate::runtime::class::mb_class_define_multi(
            MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            MbValue::none(),
            MbValue::from_ptr(MbObject::new_list(vec![])),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let obj = crate::runtime::builtins::make_type_object(class_name);
        let global_id = MbValue::from_bits(9001);
        crate::runtime::closure::mb_global_set_id(global_id, obj);
        let wref = mb_weakref_ref(obj, MbValue::none());

        let alive = reference_target(wref);
        assert_eq!(alive.as_ptr(), obj.as_ptr());
        unsafe {
            crate::runtime::rc::release_if_ptr(alive);
        }

        expire_unbound_class_refs();
        let still_alive = reference_target(wref);
        assert_eq!(still_alive.as_ptr(), obj.as_ptr());
        unsafe {
            crate::runtime::rc::release_if_ptr(still_alive);
        }

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        expire_unbound_class_refs();

        assert!(reference_target(wref).is_none());
    }

    #[test]
    fn test_ref_accepts_uuid_integer_handle() {
        let obj = super::super::uuid_mod::mb_uuid_uuid4();
        let wref = mb_weakref_ref(obj, MbValue::none());
        assert!(wref.as_ptr().is_some());
        assert_eq!(get_field(wref, "_target").to_bits(), obj.to_bits());
        assert_eq!(referent_type_name(obj), "UUID");
    }

    #[test]
    fn test_ref_still_rejects_plain_int() {
        assert!(reject_non_weakreferenceable(MbValue::from_int(1)));
        crate::runtime::exception::mb_clear_exception();
    }

    // -- proxy --

    #[test]
    fn test_proxy_returns_object() {
        let v = target();
        assert_eq!(mb_weakref_proxy(v, MbValue::none()).as_ptr(), v.as_ptr());
    }

    #[test]
    fn test_proxy_wraps_userlist() {
        let v = super::super::collections_mod::mb_userlist_new(MbValue::none());
        let proxy = mb_weakref_proxy(v, MbValue::none());
        assert!(is_proxy_instance(proxy));
        assert_eq!(proxy_target(proxy).and_then(|t| t.as_ptr()), v.as_ptr());
    }

    #[test]
    fn test_userlist_proxy_bool_forwards_to_backing_list() {
        let v = super::super::collections_mod::mb_userlist_new(MbValue::none());
        let proxy = mb_weakref_proxy(v, MbValue::none());
        assert_eq!(crate::runtime::builtins::mb_bool(proxy).as_bool(), Some(false));

        let backing = super::super::collections_mod::user_wrapper_data(v)
            .expect("expected UserList backing")
            .1;
        crate::runtime::list_ops::mb_list_append(backing, MbValue::from_int(12));
        assert_eq!(crate::runtime::builtins::mb_bool(proxy).as_bool(), Some(true));
    }

    #[test]
    fn test_proxy_with_callback_returns_object() {
        let v = target();
        let cb = MbValue::from_int(1);
        assert_eq!(mb_weakref_proxy(v, cb).as_ptr(), v.as_ptr());
    }

    // -- getweakrefcount / getweakrefs --

    #[test]
    fn test_getweakrefcount_zero() {
        let v = MbValue::from_int(1);
        assert_eq!(mb_weakref_getweakrefcount(v).as_int(), Some(0));
    }

    #[test]
    fn test_getweakrefcount_drops_deleted_global_ref() {
        let obj = target();
        let r1 = mb_weakref_ref(obj, MbValue::from_int(1));
        let r2 = mb_weakref_ref(obj, MbValue::from_int(2));
        let r1_global = MbValue::from_bits(9101);
        let r2_global = MbValue::from_bits(9102);
        crate::runtime::closure::mb_global_set_id(r1_global, r1);
        crate::runtime::closure::mb_global_set_id(r2_global, r2);

        assert_eq!(mb_weakref_getweakrefcount(obj).as_int(), Some(2));

        crate::runtime::closure::mb_global_set_id(r1_global, MbValue::none());
        assert_eq!(mb_weakref_getweakrefcount(obj).as_int(), Some(1));

        crate::runtime::closure::mb_global_set_id(r2_global, MbValue::none());
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
    fn test_weak_set_prunes_unbound_member() {
        let s = mb_weakref_weak_set();
        let item = target();
        let global_id = MbValue::from_bits(9202);
        crate::runtime::closure::mb_global_set_id(global_id, item);
        let args = MbValue::from_ptr(MbObject::new_list(vec![item]));

        unsafe {
            ws_add(s, args);
        }
        let contains_args = MbValue::from_ptr(MbObject::new_list(vec![item]));
        assert_eq!(
            unsafe { ws_contains(s, contains_args) }.as_bool(),
            Some(true)
        );
        assert_eq!(unsafe { ws_len(s, MbValue::none()) }.as_int(), Some(1));

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        assert_eq!(unsafe { ws_len(s, MbValue::none()) }.as_int(), Some(0));
    }

    #[test]
    fn test_weak_set_registers_public_set_methods() {
        register_weakref_classes();
        for method in [
            "add",
            "clear",
            "copy",
            "difference",
            "difference_update",
            "discard",
            "intersection",
            "intersection_update",
            "isdisjoint",
            "issubset",
            "issuperset",
            "pop",
            "remove",
            "symmetric_difference",
            "symmetric_difference_update",
            "union",
            "update",
        ] {
            assert!(
                !crate::runtime::class::lookup_method("WeakSet", method).is_none(),
                "missing WeakSet method {method}"
            );
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
    fn test_weak_key_dict_accepts_plain_value_and_prunes_unbound_key() {
        let d = mb_weakref_weak_key_dict();
        let key = target();
        let global_id = MbValue::from_bits(9201);
        crate::runtime::closure::mb_global_set_id(global_id, key);
        let value = MbValue::from_ptr(MbObject::new_str("data".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![key, value]));

        unsafe {
            wkd_setitem(d, args);
        }
        let contains_args = MbValue::from_ptr(MbObject::new_list(vec![key]));
        assert_eq!(
            unsafe { wkd_contains(d, contains_args) }.as_bool(),
            Some(true)
        );
        assert_eq!(unsafe { wkd_len(d, MbValue::none()) }.as_int(), Some(1));

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        assert_eq!(unsafe { wkd_len(d, MbValue::none()) }.as_int(), Some(0));
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

    #[test]
    fn test_weak_value_dict_prunes_unbound_value() {
        let d = mb_weakref_weak_value_dict();
        let value = target();
        let global_id = MbValue::from_bits(9203);
        crate::runtime::closure::mb_global_set_id(global_id, value);
        let key = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![key, value]));

        unsafe {
            wvd_setitem(d, args);
        }
        let contains_args = MbValue::from_ptr(MbObject::new_list(vec![key]));
        assert_eq!(
            unsafe { wvd_contains(d, contains_args) }.as_bool(),
            Some(true)
        );
        assert_eq!(unsafe { wvd_len(d, MbValue::none()) }.as_int(), Some(1));

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        let contains_args = MbValue::from_ptr(MbObject::new_list(vec![key]));
        assert_eq!(
            unsafe { wvd_contains(d, contains_args) }.as_bool(),
            Some(false)
        );
        assert_eq!(unsafe { wvd_len(d, MbValue::none()) }.as_int(), Some(0));
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

    #[test]
    fn test_finalize_expires_once_after_global_rebind() {
        FINALIZE_CALLBACK_COUNT.store(0, Ordering::SeqCst);
        let obj = target();
        let global_id = MbValue::from_bits(9011);
        crate::runtime::closure::mb_global_set_id(global_id, obj);
        let f = mb_weakref_finalize(obj, finalize_test_func());
        unsafe {
            crate::runtime::rc::release_if_ptr(obj);
        }

        expire_unbound_finalizers();
        assert_eq!(get_field(f, "alive").as_bool(), Some(true));
        assert_eq!(FINALIZE_CALLBACK_COUNT.load(Ordering::SeqCst), 0);

        crate::runtime::closure::mb_global_set_id(global_id, MbValue::none());
        expire_unbound_finalizers();
        assert_eq!(get_field(f, "alive").as_bool(), Some(false));
        assert_eq!(FINALIZE_CALLBACK_COUNT.load(Ordering::SeqCst), 1);

        expire_unbound_finalizers();
        assert_eq!(FINALIZE_CALLBACK_COUNT.load(Ordering::SeqCst), 1);
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
