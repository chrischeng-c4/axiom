/// copy module for Mamba (#414).
///
/// Provides:
///   copy.copy(obj)     — shallow copy
///   copy.deepcopy(obj) — recursive deep copy with a memo (cycles + shared refs)
///   copy.Error / copy.error — exception raised for un-copyable callables
///
/// CPython 3.12 semantics (Lib/copy.py) implemented here:
///   * Immutable atoms (None, int/bigint, float, bool, complex, str, bytes,
///     frozenset, function, code, type, range, slice, Ellipsis, NotImplemented)
///     are returned BY IDENTITY for both copy and deepcopy.
///   * Mutable containers (list, dict, set, bytearray) become a fresh,
///     equal object; shallow copy shares the members, deepcopy rebuilds them.
///   * A tuple is immutable, so shallow copy returns it by identity. deepcopy
///     of a tuple whose members are all atomic also returns identity; a tuple
///     holding a mutable member is rebuilt with independent inner copies.
///   * User-class instances honor __copy__ / __deepcopy__ / __reduce_ex__ /
///     __reduce__ / __getstate__ / __setstate__, falling back to a default
///     __dict__ copy when none are present.
///   * deepcopy threads a memo keyed on the source object's identity so a
///     repeated reference (or a self-referential cycle) is reconstructed as a
///     single shared object that points at the copy, not the original.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// True when a user method we just invoked left a pending exception (so the
/// copy must abort and let the runtime re-raise it).
fn exception_pending() -> bool {
    super::super::exception::mb_has_exception().as_bool() == Some(true)
}

/// Set a pending exception of `kind` (a builtin/exception class name) with
/// `msg`, then return None so the runtime re-raises at the call site.
fn raise_exc(kind: &str, msg: &str) -> MbValue {
    let k = MbValue::from_ptr(MbObject::new_str(kind.to_string()));
    let m = MbValue::from_ptr(MbObject::new_str(msg.to_string()));
    super::super::exception::mb_raise(k, m);
    MbValue::none()
}

/// Helper: extract a string from an MbValue.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// True for values that copy/deepcopy return by identity (immutable atoms).
///
/// In mamba a class-as-value and a builtin like `max` arrive as `ObjData::Str`
/// (the class / type name); a plain string literal is also `ObjData::Str`.
/// All three are atomic in CPython (str, type, builtin_function), so returning
/// identity for every `Str` is correct. Functions are `TAG_FUNC` (atomic).
/// `range`/`slice`/`property()` arrive as `None` or as immutable Instances and
/// are treated as atomic. Lists, dicts, sets, bytearrays and ordinary
/// user-class instances are NOT atomic.
fn is_atomic(obj: MbValue) -> bool {
    if obj.is_none()
        || obj.as_int().is_some()
        || obj.as_float().is_some()
        || obj.as_bool().is_some()
        || obj.is_not_implemented()
        || obj.as_func().is_some()
    {
        return true;
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_)
                | ObjData::Bytes(_)
                | ObjData::BigInt(_)
                | ObjData::Complex(_, _)
                | ObjData::FrozenSet(_)
                | ObjData::CodeObject { .. } => true,
                // Instances that copy/deepcopy treat as atomic (return
                // identity). `range`/`slice` are immutable scalar wrappers;
                // `weakref.ref` (ReferenceType) is atomic in CPython's copy
                // dispatch tables (Lib/copy.py maps it to _copy_immutable /
                // _deepcopy_atomic).
                ObjData::Instance { class_name, .. } => {
                    matches!(
                        class_name.as_str(),
                        // "code": CPython's _deepcopy_atomic covers CodeType.
                        "range" | "slice" | "ReferenceType" | "weakref" | "code"
                    )
                }
                _ => false,
            }
        }
    } else {
        // Any other inline value (e.g. StopIteration sentinel) is atomic.
        true
    }
}

/// Return `obj`, retaining if it is a heap pointer so the JIT can release both
/// the argument and the result VRegs.
fn return_identity(obj: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(obj); }
    obj
}

// ── copy.copy ──────────────────────────────────────────────────────────────

/// Dispatcher for copy.copy — unpacks args and calls mb_copy_copy.
unsafe extern "C" fn dispatch_copy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 { return MbValue::none(); }
    let arg = unsafe { *args_ptr };
    mb_copy_copy(arg)
}

/// copy.copy(obj) -> shallow copy.
pub fn mb_copy_copy(obj: MbValue) -> MbValue {
    // Immutable atoms (and tuples — see below) return by identity.
    if is_atomic(obj) {
        return return_identity(obj);
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                // Tuple is immutable: shallow copy returns the same object.
                ObjData::Tuple(_) => return_identity(obj),
                ObjData::List(lock) => {
                    let items = lock.read().unwrap();
                    MbValue::from_ptr(MbObject::new_list_borrowed(items.to_vec()))
                }
                ObjData::Dict(lock) => {
                    let map = lock.read().unwrap();
                    let d = MbObject::new_dict();
                    if let ObjData::Dict(ref new_lock) = (*d).data {
                        let mut new_map = new_lock.write().unwrap();
                        *new_map = map.clone();
                        for val in new_map.values() {
                            super::super::rc::retain_if_ptr(*val);
                        }
                    }
                    MbValue::from_ptr(d)
                }
                ObjData::Set(lock) => {
                    let items = lock.read().unwrap();
                    MbValue::from_ptr(MbObject::new_set_borrowed(items.to_vec()))
                }
                ObjData::ByteArray(lock) => {
                    let data = lock.read().unwrap().clone();
                    MbValue::from_ptr(MbObject::new_bytearray(data))
                }
                ObjData::Instance { class_name, .. } => {
                    let cls = class_name.clone();
                    copy_instance(obj, &cls)
                }
                _ => return_identity(obj),
            }
        }
    } else {
        obj
    }
}

/// Shallow-copy a user-class instance (copy.copy fallback chain).
///
/// 1. `__copy__()` if defined.
/// 2. `__reduce_ex__(4)` / `__reduce__()` reconstruction with a shallow state.
/// 3. Default: new instance whose fields are shared with the original.
unsafe fn copy_instance(obj: MbValue, class_name: &str) -> MbValue {
    // 1. __copy__ hook.
    if has_method(class_name, "__copy__") {
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        return call_method(obj, "__copy__", empty);
    }
    // 2. __reduce_ex__ / __reduce__ reconstruction.
    match reduce(obj, class_name) {
        ReduceOutcome::Identity => return return_identity(obj),
        ReduceOutcome::Tuple(rv) => {
            return reconstruct(rv, /*deep=*/false, &mut FxHashMap::default());
        }
        ReduceOutcome::TypeError => {
            return raise_exc("TypeError", "copy._reconstruct() takes no state_setter");
        }
        ReduceOutcome::None => {}
    }
    // CPython resolves the default copier through `getattr(x, "__reduce_ex__")`
    // (inherited from object). A class whose `__getattribute__` deliberately
    // blocks `__reduce*` leaves copy with no usable reducer, so it raises
    // copy.Error. Detect that here before the default __dict__ copy.
    if reducer_blocked_by_getattribute(obj, class_name) {
        return raise_exc(
            "copy.Error",
            &format!("un(shallow-)copyable object of type {class_name}"),
        );
    }
    // 3. Default: copy the field map, members shared. Only for registered
    // user classes — opaque builtin instances (stdlib handles, weakrefs,
    // etc.) have no meaningful __dict__ and are returned by identity, which
    // also matches CPython for the atomic builtin types.
    if !super::super::class::class_is_registered(class_name) {
        return return_identity(obj);
    }
    // CPython's default reductor (object.__reduce_ex__ -> __getstate__) calls a
    // user __getstate__ when defined. Invoke it so any exception it raises
    // propagates (e.g. test_getstate_exc) and so an explicit __setstate__ runs.
    if has_method(class_name, "__getstate__") {
        if let Some(inst) = default_via_getstate(obj, class_name, /*deep=*/false, &mut FxHashMap::default()) {
            return inst;
        }
        // Exception pending (e.g. ValueError from __getstate__): bail with the
        // original object; the runtime sees the pending exception and raises.
        if exception_pending() {
            return return_identity(obj);
        }
    }
    let fields = read_fields(obj);
    let new_inst = MbObject::new_instance(class_name.to_string());
    if let ObjData::Instance { fields: ref new_lock, .. } = (*new_inst).data {
        let mut nf = new_lock.write().unwrap();
        for (k, v) in fields.iter() {
            super::super::rc::retain_if_ptr(*v);
            nf.insert(k.clone(), *v);
        }
    }
    MbValue::from_ptr(new_inst)
}

/// CPython default-reductor path when the class defines `__getstate__`:
/// build a fresh empty instance, capture `state = obj.__getstate__()`
/// (propagating any exception), then restore it via `__setstate__(state)` if
/// defined, else dict-merge a dict state. Returns `None` if `__getstate__`
/// raised (pending exception left set) so the caller can bail.
unsafe fn default_via_getstate(
    obj: MbValue,
    class_name: &str,
    deep: bool,
    memo: &mut FxHashMap<u64, MbValue>,
) -> Option<MbValue> {
    let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
    let state = call_method(obj, "__getstate__", empty);
    if exception_pending() {
        return None;
    }
    if state.is_none() {
        // No state to restore; fall back to the plain field copy.
        return None;
    }
    let applied = if deep { deepcopy_memo(state, memo) } else {
        super::super::rc::retain_if_ptr(state);
        state
    };
    let new_inst = MbObject::new_instance(class_name.to_string());
    let inst = MbValue::from_ptr(new_inst);
    apply_state(inst, applied);
    Some(inst)
}

// ── copy.deepcopy ──────────────────────────────────────────────────────────

/// Dispatcher for copy.deepcopy — unpacks args and calls mb_copy_deepcopy.
/// A caller-supplied memo dict (2nd arg) is threaded so that, on completion,
/// CPython's observable memo contents (`memo[id(orig)] = copy` for every
/// memoized object plus the `memo[id(memo)]` keep-alive list) are mirrored
/// into it. Immutable atoms are deliberately NOT memoized, matching CPython.
unsafe extern "C" fn dispatch_deepcopy(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 { return MbValue::none(); }
    let arg = unsafe { *args_ptr };
    let caller_memo = if nargs >= 2 {
        let m = unsafe { *args_ptr.add(1) };
        // Only a real dict participates; None / other values are ignored.
        if m.as_ptr().map(|p| matches!((*p).data, ObjData::Dict(_))).unwrap_or(false) {
            Some(m)
        } else {
            None
        }
    } else {
        None
    };
    mb_copy_deepcopy_with_memo(arg, caller_memo)
}

/// copy.deepcopy(obj) -> recursive deep copy with cycle-safe memo.
pub fn mb_copy_deepcopy(obj: MbValue) -> MbValue {
    mb_copy_deepcopy_with_memo(obj, None)
}

/// `id(val)` exactly as the runtime's builtin `id()` computes it, so memo keys
/// the caller observes (`id(x)`, `id(memo)`) line up with user-side `id()`.
fn runtime_id(val: MbValue) -> i64 {
    if let Some(ptr) = val.as_ptr() {
        (ptr as u64 & 0x0000_7FFF_FFFF_FFFF) as i64
    } else {
        (val.to_bits() >> 17) as i64
    }
}

/// deepcopy with an optional caller-supplied memo dict to populate on return.
pub fn mb_copy_deepcopy_with_memo(obj: MbValue, caller_memo: Option<MbValue>) -> MbValue {
    let mut memo: FxHashMap<u64, MbValue> = FxHashMap::default();
    // Track, in memoization order, the (original, copy) pairs so we can both
    // build the keep-alive list and mirror entries into the caller's dict.
    let mut order: Vec<(MbValue, MbValue)> = Vec::new();
    let result = deepcopy_memo_tracked(obj, &mut memo, &mut order);
    if let Some(cm) = caller_memo {
        unsafe { populate_caller_memo(cm, &order); }
    }
    result
}

/// Mirror the internal memo into the caller's dict using CPython's observable
/// layout: `memo[id(orig)] = copy` for every memoized object, then a keep-alive
/// list under `id(memo)` holding the originals (so they outlive the copy).
unsafe fn populate_caller_memo(cm: MbValue, order: &[(MbValue, MbValue)]) {
    let cm_ptr = match cm.as_ptr() { Some(p) => p, None => return };
    // Build the keep-alive list of originals.
    let keepalive_items: Vec<MbValue> = order.iter().map(|(orig, _)| {
        super::super::rc::retain_if_ptr(*orig);
        *orig
    }).collect();
    let keepalive = MbValue::from_ptr(MbObject::new_list(keepalive_items));
    if let ObjData::Dict(ref lock) = (*cm_ptr).data {
        let mut map = lock.write().unwrap();
        for (orig, copy) in order {
            let k = super::super::dict_ops::DictKey::Int(runtime_id(*orig));
            super::super::rc::retain_if_ptr(*copy);
            map.insert(k, *copy);
        }
        let memo_key = super::super::dict_ops::DictKey::Int(runtime_id(cm));
        map.insert(memo_key, keepalive);
    }
}

/// Thin wrapper that records memoization order into `order` as objects are
/// memoized, by delegating to `deepcopy_memo` with a side-channel.
fn deepcopy_memo_tracked(
    obj: MbValue,
    memo: &mut FxHashMap<u64, MbValue>,
    order: &mut Vec<(MbValue, MbValue)>,
) -> MbValue {
    DEEPCOPY_ORDER.with(|cell| {
        let prev = cell.borrow().is_some();
        if !prev {
            *cell.borrow_mut() = Some(Vec::new());
        }
        let result = deepcopy_memo(obj, memo);
        if !prev {
            if let Some(recorded) = cell.borrow_mut().take() {
                *order = recorded;
            }
        }
        result
    })
}

thread_local! {
    /// Side-channel recording (original, copy) pairs in memoization order for
    /// the active top-level deepcopy, used to populate a caller-supplied memo.
    static DEEPCOPY_ORDER: std::cell::RefCell<Option<Vec<(MbValue, MbValue)>>> =
        const { std::cell::RefCell::new(None) };
}

/// Record that `orig` was memoized to `copy` (for caller-memo mirroring).
fn record_memoized(orig: MbValue, copy: MbValue) {
    DEEPCOPY_ORDER.with(|cell| {
        if let Some(ref mut v) = *cell.borrow_mut() {
            v.push((orig, copy));
        }
    });
}

/// Core deepcopy recursion. `memo` maps a source object's NaN-boxed bits to its
/// freshly-built copy so repeated references and cycles collapse to one object.
fn deepcopy_memo(obj: MbValue, memo: &mut FxHashMap<u64, MbValue>) -> MbValue {
    if is_atomic(obj) {
        return return_identity(obj);
    }
    let ptr = match obj.as_ptr() {
        Some(p) => p,
        None => return obj,
    };
    let key = obj.to_bits();
    if let Some(existing) = memo.get(&key) {
        // Repeated reference / cycle — share the already-built copy.
        let v = *existing;
        unsafe { super::super::rc::retain_if_ptr(v); }
        return v;
    }
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => {
                // Mirrors CPython's _deepcopy_tuple: deepcopy the elements
                // first, then (a) if a cycle filled memo[id(x)] during the
                // recursion, return that shared copy; (b) if every element
                // came back unchanged, return the original tuple by identity;
                // (c) otherwise rebuild an independent tuple.
                let src: Vec<MbValue> = items.clone();
                let deep: Vec<MbValue> = src.iter().map(|v| deepcopy_memo(*v, memo)).collect();
                if let Some(existing) = memo.get(&key) {
                    let v = *existing;
                    super::super::rc::retain_if_ptr(v);
                    for d in &deep { super::super::rc::release_if_ptr(*d); }
                    return v;
                }
                let all_same = src.iter().zip(deep.iter())
                    .all(|(a, b)| a.to_bits() == b.to_bits());
                if all_same {
                    // No inner copy was needed: return the original tuple, and
                    // release the (identical) recursively-produced refs.
                    for d in &deep { super::super::rc::release_if_ptr(*d); }
                    return return_identity(obj);
                }
                let new_t = MbObject::new_tuple(deep);
                let result = MbValue::from_ptr(new_t);
                memo.insert(key, result);
                record_memoized(obj, result);
                result
            }
            ObjData::List(lock) => {
                let new_l = MbObject::new_list(vec![]);
                let result = MbValue::from_ptr(new_l);
                // Insert BEFORE recursing so a self-reference resolves here.
                memo.insert(key, result);
                record_memoized(obj, result);
                let src: Vec<MbValue> = lock.read().unwrap().to_vec();
                for v in src {
                    let dv = deepcopy_memo(v, memo);
                    if let ObjData::List(ref nl) = (*new_l).data {
                        nl.write().unwrap().push(dv);
                    }
                }
                result
            }
            ObjData::Dict(lock) => {
                let new_d = MbObject::new_dict();
                let result = MbValue::from_ptr(new_d);
                memo.insert(key, result);
                record_memoized(obj, result);
                let pairs: Vec<(super::super::dict_ops::DictKey, MbValue)> =
                    lock.read().unwrap().iter().map(|(k, v)| (k.clone(), *v)).collect();
                for (k, v) in pairs {
                    let dv = deepcopy_memo(v, memo);
                    if let ObjData::Dict(ref nl) = (*new_d).data {
                        nl.write().unwrap().insert(k, dv);
                    }
                }
                result
            }
            ObjData::Set(lock) => {
                let src: Vec<MbValue> = lock.read().unwrap().to_vec();
                let deep: Vec<MbValue> = src.iter().map(|v| deepcopy_memo(*v, memo)).collect();
                let result = MbValue::from_ptr(MbObject::new_set(deep));
                memo.insert(key, result);
                record_memoized(obj, result);
                result
            }
            ObjData::ByteArray(lock) => {
                let data = lock.read().unwrap().clone();
                let result = MbValue::from_ptr(MbObject::new_bytearray(data));
                memo.insert(key, result);
                record_memoized(obj, result);
                result
            }
            ObjData::Instance { class_name, .. } => {
                let cls = class_name.clone();
                deepcopy_instance(obj, &cls, key, memo)
            }
            _ => return_identity(obj),
        }
    }
}

/// Deep-copy a user-class instance (deepcopy fallback chain).
unsafe fn deepcopy_instance(
    obj: MbValue,
    class_name: &str,
    key: u64,
    memo: &mut FxHashMap<u64, MbValue>,
) -> MbValue {
    // 1. __deepcopy__(memo) hook.
    if has_method(class_name, "__deepcopy__") {
        // Pass a real (empty) dict so the user hook can recurse with it.
        let memo_dict = MbValue::from_ptr(MbObject::new_dict());
        let args = MbValue::from_ptr(MbObject::new_list(vec![memo_dict]));
        let result = call_method(obj, "__deepcopy__", args);
        memo.insert(key, result);
        record_memoized(obj, result);
        unsafe { super::super::rc::retain_if_ptr(result); }
        return result;
    }
    // 2. __reduce_ex__ / __reduce__ reconstruction.
    match reduce(obj, class_name) {
        ReduceOutcome::Identity => {
            let v = return_identity(obj);
            memo.insert(key, v);
            return v;
        }
        ReduceOutcome::Tuple(rv) => {
            // Seed the memo with a fresh shell before reconstructing so a cyclic
            // state can resolve to the copy. reconstruct() will fill it.
            return reconstruct_instance_memo(rv, class_name, key, memo);
        }
        ReduceOutcome::TypeError => {
            return raise_exc("TypeError", "copy._reconstruct() takes no state_setter");
        }
        ReduceOutcome::None => {}
    }
    // A class whose `__getattribute__` blocks `__reduce*` leaves deepcopy with
    // no usable reducer → copy.Error (see copy_instance for the rationale).
    if reducer_blocked_by_getattribute(obj, class_name) {
        return raise_exc(
            "copy.Error",
            &format!("un(deep-)copyable object of type {class_name}"),
        );
    }
    // 3. Default: new instance whose fields are deepcopied. Only for
    // registered user classes — opaque builtin instances are returned by
    // identity (preserving baseline behavior + CPython's atomic dispatch).
    if !super::super::class::class_is_registered(class_name) {
        let v = return_identity(obj);
        memo.insert(key, v);
        return v;
    }
    // CPython default reductor calls a user __getstate__ when defined; mirror it
    // so its exception propagates and an explicit __setstate__ runs on a copy.
    if has_method(class_name, "__getstate__") {
        if let Some(inst) = default_via_getstate(obj, class_name, /*deep=*/true, memo) {
            memo.insert(key, inst);
            record_memoized(obj, inst);
            super::super::rc::retain_if_ptr(inst);
            return inst;
        }
        if exception_pending() {
            let v = return_identity(obj);
            memo.insert(key, v);
            return v;
        }
    }
    let new_inst = MbObject::new_instance(class_name.to_string());
    let result = MbValue::from_ptr(new_inst);
    memo.insert(key, result); // seed for cycles (self-referential attribute)
    record_memoized(obj, result);
    let fields = read_fields(obj);
    for (k, v) in fields.iter() {
        let dv = deepcopy_memo(*v, memo);
        if let ObjData::Instance { fields: ref nl, .. } = (*new_inst).data {
            nl.write().unwrap().insert(k.clone(), dv);
        }
    }
    result
}

// ── pickle-protocol reconstruction ─────────────────────────────────────────

/// The relevant pieces of a __reduce__ result tuple: callable, args, state.
struct Reduced {
    callable: MbValue,
    args: Vec<MbValue>,
    state: Option<MbValue>,
}

/// Outcome of consulting an instance's reducer protocol.
enum ReduceOutcome {
    /// No reducer defined — caller should use the default __dict__ copy.
    None,
    /// `__reduce__`/`__reduce_ex__` returned a string (global-reference pickle
    /// convention) — copy/deepcopy must return the object by identity.
    Identity,
    /// A reconstruction tuple was supplied.
    Tuple(Reduced),
    /// The reduce tuple was malformed for copy (e.g. a 6-element tuple carrying
    /// a protocol-5 `state_setter`, which `copy._reconstruct` cannot accept).
    /// copy/deepcopy must raise TypeError, mirroring CPython where `*rv` exceeds
    /// `_reconstruct`'s positional arity.
    TypeError,
}

/// Invoke __reduce_ex__(4) or __reduce__() if the class defines one, returning
/// the parsed (callable, args, state). Returns `None` when the instance has no
/// custom reducer (so the caller falls back to a plain __dict__ copy). A string
/// result from __reduce__ (the "return self by name" protocol) also yields
/// `None` so the object is returned by identity.
unsafe fn reduce(obj: MbValue, class_name: &str) -> ReduceOutcome {
    let rv = if has_method(class_name, "__reduce_ex__") {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(4)]));
        call_method(obj, "__reduce_ex__", args)
    } else if has_method(class_name, "__reduce__") {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        call_method(obj, "__reduce__", args)
    } else {
        return ReduceOutcome::None;
    };
    // A bare string result means "return self by name" — return identity.
    if rv.as_ptr().map(|p| matches!((*p).data, ObjData::Str(_))).unwrap_or(false) {
        return ReduceOutcome::Identity;
    }
    let tuple_items: Vec<MbValue> = match rv.as_ptr() {
        Some(p) => match &(*p).data {
            ObjData::Tuple(items) => items.clone(),
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            _ => return ReduceOutcome::None,
        },
        None => return ReduceOutcome::None,
    };
    if tuple_items.len() < 2 {
        return ReduceOutcome::None;
    }
    // A 6-element reduce tuple carries a protocol-5 `state_setter` as its final
    // member. CPython's `copy._reconstruct` accepts only 5 reduce elements
    // (func, args, state, listiter, dictiter), so `_reconstruct(x, memo, *rv)`
    // overflows its arity and raises TypeError for both copy and deepcopy —
    // regardless of whether the state_setter is None.
    if tuple_items.len() >= 6 {
        return ReduceOutcome::TypeError;
    }
    let callable = tuple_items[0];
    let args = match tuple_items[1].as_ptr() {
        Some(p) => match &(*p).data {
            ObjData::Tuple(items) => items.clone(),
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    };
    let state = if tuple_items.len() >= 3 && !tuple_items[2].is_none() {
        Some(tuple_items[2])
    } else {
        None
    };
    ReduceOutcome::Tuple(Reduced { callable, args, state })
}

/// Reconstruct an object from a Reduced for copy.copy (shallow state apply).
unsafe fn reconstruct(rv: Reduced, deep: bool, memo: &mut FxHashMap<u64, MbValue>) -> MbValue {
    let args_list = MbValue::from_ptr(MbObject::new_list(rv.args.clone()));
    let inst = super::super::builtins::mb_call_spread(rv.callable, args_list);
    if let Some(state) = rv.state {
        let applied = if deep { deepcopy_memo(state, memo) } else {
            super::super::rc::retain_if_ptr(state);
            state
        };
        apply_state(inst, applied);
    }
    inst
}

/// Reconstruct an instance for deepcopy, seeding the memo with the new shell so
/// a cyclic state resolves to the copy.
unsafe fn reconstruct_instance_memo(
    rv: Reduced,
    _class_name: &str,
    key: u64,
    memo: &mut FxHashMap<u64, MbValue>,
) -> MbValue {
    let args_list = MbValue::from_ptr(MbObject::new_list(rv.args.clone()));
    let inst = super::super::builtins::mb_call_spread(rv.callable, args_list);
    memo.insert(key, inst);
    super::super::rc::retain_if_ptr(inst);
    if let Some(state) = rv.state {
        let applied = deepcopy_memo(state, memo);
        apply_state(inst, applied);
    }
    inst
}

/// Apply a reconstructed state to an instance: prefer __setstate__, else merge
/// a dict state into the instance __dict__ (CPython's default).
unsafe fn apply_state(inst: MbValue, state: MbValue) {
    let cls = instance_class_name(inst);
    if let Some(ref c) = cls {
        if has_method(c, "__setstate__") {
            let args = MbValue::from_ptr(MbObject::new_list(vec![state]));
            call_method(inst, "__setstate__", args);
            return;
        }
    }
    // Default: state is a dict of attributes to set on the instance.
    if let Some(p) = state.as_ptr() {
        if let ObjData::Dict(ref lock) = (*p).data {
            let pairs: Vec<(super::super::dict_ops::DictKey, MbValue)> =
                lock.read().unwrap().iter().map(|(k, v)| (k.clone(), *v)).collect();
            if let Some(ip) = inst.as_ptr() {
                if let ObjData::Instance { fields: ref flock, .. } = (*ip).data {
                    let mut f = flock.write().unwrap();
                    for (k, v) in pairs {
                        if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                            super::super::rc::retain_if_ptr(v);
                            f.insert(ks.clone(), v);
                        }
                    }
                }
            }
        }
    }
}

// ── instance helpers ───────────────────────────────────────────────────────

/// True if `class_name` (via MRO) defines `method`.
fn has_method(class_name: &str, method: &str) -> bool {
    !super::super::class::lookup_method(class_name, method).is_none()
}

/// Detect CPython's "uncopyable" condition: a class with a custom
/// `__getattribute__` that intercepts the inherited reducer (`__reduce_ex__` /
/// `__reduce__`) so neither is reachable. We probe each name through the user
/// `__getattribute__`; if both probes raise, the reducer is blocked and
/// copy/deepcopy must raise copy.Error. Any pending exception from the probe is
/// cleared so it does not leak past this check. Returns false when the class
/// has no custom `__getattribute__` (the normal default-copy path applies).
unsafe fn reducer_blocked_by_getattribute(obj: MbValue, class_name: &str) -> bool {
    if !has_method(class_name, "__getattribute__") {
        return false;
    }
    let probe = |name: &str| -> bool {
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
        ]));
        super::super::exception::mb_clear_exception();
        call_method(obj, "__getattribute__", args);
        let raised = exception_pending();
        super::super::exception::mb_clear_exception();
        raised
    };
    probe("__reduce_ex__") && probe("__reduce__")
}

/// Call `obj.method(*args)` where `args` is a list MbValue. Self-binding is
/// handled by mb_call_method.
fn call_method(obj: MbValue, method: &str, args: MbValue) -> MbValue {
    let name = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    super::super::class::mb_call_method(obj, name, args)
}

/// Snapshot an instance's field map.
unsafe fn read_fields(obj: MbValue) -> FxHashMap<String, MbValue> {
    if let Some(ptr) = obj.as_ptr() {
        if let ObjData::Instance { fields, .. } = &(*ptr).data {
            return fields.read().unwrap().clone();
        }
    }
    FxHashMap::default()
}

/// Get the class name of an instance value, if it is one.
unsafe fn instance_class_name(obj: MbValue) -> Option<String> {
    obj.as_ptr().and_then(|ptr| {
        if let ObjData::Instance { class_name, .. } = &(*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

// ── registration ───────────────────────────────────────────────────────────

/// Register the copy module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("copy", dispatch_copy as *const () as usize),
        ("deepcopy", dispatch_deepcopy as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // copy.Error: a real exception class so callers can `raise copy.Error(...)`
    // and `except copy.Error`. We register it in the class machinery (base
    // Exception) and expose the bound name as the class-name string, which is
    // how mamba threads class objects (callable() + construction + except
    // matching all key off the registered name).
    super::super::class::mb_class_register(
        "copy.Error",
        vec!["Exception".to_string()],
        HashMap::new(),
    );
    // CPython binds `copy.error = Error` (Lib/copy.py), so the two names refer
    // to the *same* class object: `copy.Error is copy.error` is True. Mamba
    // threads a class as its name-string `MbValue`; sharing one heap pointer
    // (one `new_str` allocation, retained once for the alias) makes the two
    // module attributes bit-identical, so the `is` identity check holds.
    let error_cls = MbValue::from_ptr(MbObject::new_str("copy.Error".to_string()));
    unsafe { super::super::rc::retain_if_ptr(error_cls); }
    attrs.insert("Error".to_string(), error_cls);
    // PEP-deprecated lowercase alias `copy.error` IS the same class object.
    attrs.insert("error".to_string(), error_cls);

    super::register_module("copy", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_primitives() {
        let int_val = MbValue::from_int(42);
        assert_eq!(mb_copy_copy(int_val).as_int(), Some(42));

        let float_val = MbValue::from_float(3.14);
        assert_eq!(mb_copy_copy(float_val).as_float(), Some(3.14));

        let bool_val = MbValue::from_bool(true);
        assert_eq!(mb_copy_copy(bool_val).as_bool(), Some(true));

        let none_val = MbValue::none();
        assert!(mb_copy_copy(none_val).is_none());
    }

    #[test]
    fn test_copy_str_identity() {
        // str is atomic: copy returns identity (same bits).
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let copied = mb_copy_copy(s);
        assert_eq!(s.to_bits(), copied.to_bits());
    }

    #[test]
    fn test_copy_tuple_identity() {
        // tuple is immutable: shallow copy returns identity.
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ]));
        let copied = mb_copy_copy(t);
        assert_eq!(t.to_bits(), copied.to_bits());
    }

    #[test]
    fn test_shallow_copy_list() {
        let inner = MbValue::from_int(10);
        let list = MbValue::from_ptr(MbObject::new_list(vec![inner]));
        let copied = mb_copy_copy(list);
        assert_ne!(list.as_ptr(), copied.as_ptr());
        unsafe {
            if let ObjData::List(ref lock) = (*copied.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].as_int(), Some(10));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_deepcopy_nested_list() {
        let inner_list = MbValue::from_ptr(
            MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]),
        );
        let outer = MbValue::from_ptr(MbObject::new_list(vec![inner_list]));
        let deep = mb_copy_deepcopy(outer);
        assert_ne!(outer.as_ptr(), deep.as_ptr());
        unsafe {
            let outer_items = if let ObjData::List(ref lock) = (*outer.as_ptr().unwrap()).data {
                lock.read().unwrap()[0].as_ptr().unwrap()
            } else { panic!("expected list") };
            let deep_items = if let ObjData::List(ref lock) = (*deep.as_ptr().unwrap()).data {
                lock.read().unwrap()[0].as_ptr().unwrap()
            } else { panic!("expected list") };
            assert_ne!(outer_items, deep_items);
        }
    }

    #[test]
    fn test_deepcopy_shared_ref_shared() {
        // [shared, shared] -> deep[0] is deep[1], both != shared.
        let shared = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(99)]));
        let container = MbValue::from_ptr(MbObject::new_list(vec![shared, shared]));
        let deep = mb_copy_deepcopy(container);
        unsafe {
            if let ObjData::List(ref lock) = (*deep.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items[0].to_bits(), items[1].to_bits(), "shared ref kept");
                assert_ne!(items[0].to_bits(), shared.to_bits(), "fresh copy");
            } else { panic!("expected list") }
        }
    }

    #[test]
    fn test_deepcopy_cycle_no_overflow() {
        // cyclic = [1, 2]; cyclic.append(cyclic)
        let cyclic = MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let cyclic_val = MbValue::from_ptr(cyclic);
        unsafe {
            if let ObjData::List(ref lock) = (*cyclic).data {
                lock.write().unwrap().push(cyclic_val);
            }
        }
        let deep = mb_copy_deepcopy(cyclic_val);
        assert_ne!(deep.as_ptr(), cyclic_val.as_ptr());
        unsafe {
            if let ObjData::List(ref lock) = (*deep.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items[2].to_bits(), deep.to_bits(), "cycle rebuilt to copy");
                assert_ne!(items[2].to_bits(), cyclic_val.to_bits(), "no leak to original");
            } else { panic!("expected list") }
        }
    }

    #[test]
    fn test_deepcopy_primitives() {
        assert_eq!(mb_copy_deepcopy(MbValue::from_int(7)).as_int(), Some(7));
        assert_eq!(mb_copy_deepcopy(MbValue::from_float(2.5)).as_float(), Some(2.5));
        assert!(mb_copy_deepcopy(MbValue::none()).is_none());
        assert_eq!(mb_copy_deepcopy(MbValue::from_bool(false)).as_bool(), Some(false));
    }

    #[test]
    fn test_deepcopy_dict() {
        let inner_list = MbValue::from_ptr(
            MbObject::new_list(vec![MbValue::from_int(10)]),
        );
        let d = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*d).data {
                lock.write().unwrap().insert("lst".into(), inner_list);
            }
        }
        let original = MbValue::from_ptr(d);
        let deep = mb_copy_deepcopy(original);
        assert_ne!(original.as_ptr(), deep.as_ptr());
        unsafe {
            let orig_list = if let ObjData::Dict(ref lock) = (*original.as_ptr().unwrap()).data {
                lock.read().unwrap().get("lst").unwrap().as_ptr().unwrap()
            } else { panic!("expected Dict") };
            let deep_list = if let ObjData::Dict(ref lock) = (*deep.as_ptr().unwrap()).data {
                lock.read().unwrap().get("lst").unwrap().as_ptr().unwrap()
            } else { panic!("expected Dict") };
            assert_ne!(orig_list, deep_list);
        }
    }

    #[test]
    fn test_deepcopy_tuple_of_immutables_identity() {
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ]));
        let deep = mb_copy_deepcopy(t);
        assert_eq!(t.to_bits(), deep.to_bits(), "tuple of immutables -> identity");
    }

    #[test]
    fn test_deepcopy_tuple_with_mutable_rebuilt() {
        let inner = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(2)]));
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(1), inner]));
        let deep = mb_copy_deepcopy(t);
        assert_ne!(t.to_bits(), deep.to_bits(), "tuple with mutable -> new");
        unsafe {
            if let ObjData::Tuple(ref items) = (*deep.as_ptr().unwrap()).data {
                assert_ne!(items[1].as_ptr(), inner.as_ptr(), "inner list rebuilt");
            } else { panic!("expected tuple") }
        }
    }

    #[test]
    fn test_copy_set() {
        let s = MbValue::from_ptr(MbObject::new_set(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ]));
        let copied = mb_copy_copy(s);
        assert_ne!(s.as_ptr(), copied.as_ptr());
        unsafe {
            if let ObjData::Set(ref lock) = (*copied.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            } else {
                panic!("expected Set");
            }
        }
    }
}
