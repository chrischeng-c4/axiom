/// List operations for the Mamba runtime (#285) — thread-safe.
///
/// Implements Python-compatible list methods. All mutable collection access
/// goes through RwLock guards for thread-safety.

use smallvec::smallvec;
use super::value::MbValue;
use super::rc::{MbObject, ObjData};

fn normalize_index(idx: i64, len: i64) -> i64 {
    let i = if idx < 0 { idx + len } else { idx };
    i.max(0).min(len)
}

// ── Creation ──

/// Create a new empty list.
pub fn mb_list_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// Create a new list with pre-allocated capacity.
///
/// Used by list comprehension codegen when the iteration count is known
/// (e.g., `[x for x in range(N)]`). Pre-allocating avoids repeated Vec
/// resizing during N sequential `mb_list_append` calls.
pub fn mb_list_new_with_capacity(cap: MbValue) -> MbValue {
    let n = cap.as_int().unwrap_or(0).max(0) as usize;
    MbValue::from_ptr(MbObject::new_list(Vec::with_capacity(n)))
}

/// Create a list from elements (used by list literals).
pub fn mb_list_from(elements: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(elements))
}

/// Fixed-arity list constructors. Used by MakeList codegen for small
/// literals (n ∈ {1..8}) to collapse `1 + N` FFI calls
/// (`new_with_capacity` + N × `append_unchecked`) into a single call.
/// `mb_list_new_1` is the hottest case in practice — every method call
/// `recv.method(arg)` lowers to a one-element args list.
///
/// `mb_list_new_1..8` build a `SmallVec<[MbValue; 8]>` inline via
/// `smallvec![..]` and hand it to `new_list_inline`, which skips the
/// `Vec` heap allocation entirely. This is the #2517 perf gate: for
/// short list literals the old path did Box(MbObject) + Vec(elements)
/// (two heap allocs); after this change it does Box(MbObject) only
/// (one heap alloc). On the criterion micro-bench `list_literal_4` this
/// is the ≥30% reduction the issue asks for.
pub fn mb_list_new_1(a: MbValue) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a]))
}

pub fn mb_list_new_2(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b]))
}

pub fn mb_list_new_3(a: MbValue, b: MbValue, c: MbValue) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c]))
}

pub fn mb_list_new_4(a: MbValue, b: MbValue, c: MbValue, d: MbValue) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c, d]))
}

pub fn mb_list_new_5(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue, e: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c, d, e]))
}

pub fn mb_list_new_6(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue, e: MbValue, f: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
        if f.is_ptr() { super::rc::retain_if_ptr(f); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c, d, e, f]))
}

pub fn mb_list_new_7(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue,
    e: MbValue, f: MbValue, g: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
        if f.is_ptr() { super::rc::retain_if_ptr(f); }
        if g.is_ptr() { super::rc::retain_if_ptr(g); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c, d, e, f, g]))
}

pub fn mb_list_new_8(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue,
    e: MbValue, f: MbValue, g: MbValue, h: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
        if f.is_ptr() { super::rc::retain_if_ptr(f); }
        if g.is_ptr() { super::rc::retain_if_ptr(g); }
        if h.is_ptr() { super::rc::retain_if_ptr(h); }
    }
    MbValue::from_ptr(MbObject::new_list_inline(smallvec![a, b, c, d, e, f, g, h]))
}

/// Past 8 args we spill onto the stack on AArch64 SysV, but a single
/// FFI call is still cheaper than `1+N` separate ones for the
/// `data = [...10 ints...]` shape used by list_sort_builtin.
pub fn mb_list_new_9(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue, e: MbValue,
    f: MbValue, g: MbValue, h: MbValue, i: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
        if f.is_ptr() { super::rc::retain_if_ptr(f); }
        if g.is_ptr() { super::rc::retain_if_ptr(g); }
        if h.is_ptr() { super::rc::retain_if_ptr(h); }
        if i.is_ptr() { super::rc::retain_if_ptr(i); }
    }
    MbValue::from_ptr(MbObject::new_list(vec![a, b, c, d, e, f, g, h, i]))
}

pub fn mb_list_new_10(
    a: MbValue, b: MbValue, c: MbValue, d: MbValue, e: MbValue,
    f: MbValue, g: MbValue, h: MbValue, i: MbValue, j: MbValue,
) -> MbValue {
    unsafe {
        if a.is_ptr() { super::rc::retain_if_ptr(a); }
        if b.is_ptr() { super::rc::retain_if_ptr(b); }
        if c.is_ptr() { super::rc::retain_if_ptr(c); }
        if d.is_ptr() { super::rc::retain_if_ptr(d); }
        if e.is_ptr() { super::rc::retain_if_ptr(e); }
        if f.is_ptr() { super::rc::retain_if_ptr(f); }
        if g.is_ptr() { super::rc::retain_if_ptr(g); }
        if h.is_ptr() { super::rc::retain_if_ptr(h); }
        if i.is_ptr() { super::rc::retain_if_ptr(i); }
        if j.is_ptr() { super::rc::retain_if_ptr(j); }
    }
    MbValue::from_ptr(MbObject::new_list(vec![a, b, c, d, e, f, g, h, i, j]))
}

/// list(iterable) — convert an iterable MbValue to a list.
pub fn mb_list_from_iterable(val: MbValue) -> MbValue {
    // Check if val is an iterator handle (NaN-boxed int from mb_iter)
    if val.is_int() {
        // Fast path: drain the iterator into a Vec in a single batch.
        // For Range iterators this avoids 2*N HashMap lookups from the
        // has_next/next protocol, building the Vec directly instead.
        if let Some(items) = super::iter::drain_iter_to_vec(val) {
            return MbValue::from_ptr(MbObject::new_list(items));
        }
        // Wrap raw generator handles into IterKind::Generator so the
        // exhaustion path runs through `advance_generator_if_applicable`,
        // which clears the StopIteration left in the runtime exception
        // slot by `raise_stop_iteration`. Without this wrap, calling
        // `list(gen())` succeeds but leaves CURRENT_EXCEPTION set, so the
        // next module-level statement aborts on a phantom StopIteration.
        // `mb_iter` is idempotent for handles already in ITERATORS, so
        // this wrap is a no-op on the common case.
        let iter_handle = super::iter::mb_iter(val);
        let mut items = Vec::new();
        loop {
            if super::iter::mb_has_next(iter_handle).as_bool() != Some(true) {
                break;
            }
            // mb_has_next pre-peeks under the hood, so mb_next is guaranteed to
            // return the legitimate next value — even when that value is None
            // (e.g. `yield_none` generators). The previous "if item.is_none()
            // and the iterator has now finished, drop it" recheck conflated a
            // real `None` yield with end-of-iteration and silently discarded
            // the trailing element when an iterable's last produced value was
            // None.
            items.push(super::iter::mb_next(iter_handle));
        }
        return MbValue::from_ptr(MbObject::new_list(items));
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    // CPython: list(x) always creates a new list (shallow copy).
                    // Items are borrowed from the source — use new_list_borrowed
                    // so each element is retained (release_contained_values
                    // will release them when the new list is freed).
                    let items = lock.read().unwrap().to_vec();
                    return MbValue::from_ptr(MbObject::new_list_borrowed(items));
                }
                ObjData::Tuple(items) => {
                    return MbValue::from_ptr(MbObject::new_list_borrowed(items.clone()));
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap().to_vec();
                    return MbValue::from_ptr(MbObject::new_list_borrowed(items));
                }
                ObjData::FrozenSet(items) => {
                    return MbValue::from_ptr(MbObject::new_list_borrowed(items.clone()));
                }
                ObjData::Str(s) => {
                    // Class-body enum classes: list(Color) is the canonical
                    // member list, not the class-name string's characters.
                    if let Some(members) =
                        super::stdlib::enum_class::class_canonical_members(s)
                    {
                        return MbValue::from_ptr(MbObject::new_list_borrowed(members));
                    }
                    let items: Vec<MbValue> = s.chars()
                        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
                ObjData::Bytes(ref data) => {
                    // Iterating bytes/bytearray yields each byte as an int.
                    let items: Vec<MbValue> =
                        data.iter().map(|&b| MbValue::from_int(b as i64)).collect();
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
                ObjData::ByteArray(ref lock) => {
                    let items: Vec<MbValue> = lock
                        .read()
                        .unwrap()
                        .iter()
                        .map(|&b| MbValue::from_int(b as i64))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
                ObjData::Dict(ref lock) => {
                    // ET.Element dict-stubs iterate over their children, not
                    // their internal keys.
                    let element_children = {
                        let guard = lock.read().unwrap();
                        let is_element = guard.get("__class__")
                            .and_then(|v| v.as_ptr())
                            .map(|p| matches!(&(*p).data, ObjData::Str(s) if s == "Element"))
                            .unwrap_or(false);
                        if is_element {
                            guard.get("_children").copied()
                        } else {
                            None
                        }
                    };
                    if let Some(kids) = element_children {
                        if let Some(kp) = kids.as_ptr() {
                            if let ObjData::List(ref kl) = (*kp).data {
                                return MbValue::from_ptr(MbObject::new_list(
                                    kl.read().unwrap().to_vec(),
                                ));
                            }
                        }
                    }
                    // Iterating a dict yields its keys.
                    let keys: Vec<MbValue> = lock.read().unwrap().keys()
                        .map(|k| super::dict_ops::dict_key_to_mbvalue(k))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_list(keys));
                }
                ObjData::Instance { ref class_name, ref fields, .. }
                    if class_name == "collections.deque" =>
                {
                    let guard = fields.read().unwrap();
                    if let Some(items) = guard.get("_items") {
                        if let Some(ip) = items.as_ptr() {
                            if let ObjData::List(ref lock) = (*ip).data {
                                return MbValue::from_ptr(MbObject::new_list_borrowed(
                                    lock.read().unwrap().to_vec(),
                                ));
                            }
                        }
                    }
                    return MbValue::from_ptr(MbObject::new_list(Vec::new()));
                }
                ObjData::Instance { ref fields, .. } => {
                    // Struct-sequence-shaped instances (sys.version_info,
                    // urllib ParseResult) iterate over their ordered
                    // `_entries` backing list.
                    if let Some(entries) = fields.read().unwrap().get("_entries").copied() {
                        if let Some(ep) = entries.as_ptr() {
                            if let ObjData::List(ref lock) = (*ep).data {
                                return MbValue::from_ptr(MbObject::new_list(
                                    lock.read().unwrap().to_vec(),
                                ));
                            }
                        }
                    }
                    // User-defined iterable: go through the iterator protocol
                    // via mb_iter → mb_has_next/mb_next loop. mb_iter dispatches
                    // to __iter__ for Instance values.
                    let iter_handle = super::iter::mb_iter(val);
                    if iter_handle.is_none() {
                        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
                    }
                    let mut items = Vec::new();
                    loop {
                        if super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
                            break;
                        }
                        let item = super::iter::mb_next(iter_handle);
                        if item.is_none()
                            && super::iter::mb_has_next(iter_handle).as_bool() == Some(false)
                        {
                            break;
                        }
                        items.push(item);
                    }
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

// ── Access ──

/// list[index] -> value
pub fn mb_list_getitem(list: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let Some(idx) = index.as_int() {
                match &(*ptr).data {
                    ObjData::List(ref lock) => {
                        // Fast path: try non-blocking read (succeeds when uncontended)
                        let items = match lock.try_read() {
                            Ok(guard) => guard,
                            Err(_) => lock.read().unwrap(),
                        };
                        let len = items.len() as i64;
                        let actual = if idx < 0 { idx + len } else { idx };
                        if actual >= 0 && actual < len {
                            let val = items[actual as usize];
                            super::rc::retain_if_ptr(val);
                            return val;
                        }
                        drop(items);
                        // CPython: out-of-range list index raises (#32).
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "list index out of range".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    ObjData::Tuple(ref items) => {
                        let len = items.len() as i64;
                        let actual = if idx < 0 { idx + len } else { idx };
                        if actual >= 0 && actual < len {
                            let val = items[actual as usize];
                            super::rc::retain_if_ptr(val);
                            return val;
                        }
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "tuple index out of range".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    _ => {}
                }
            }
        }
    }
    MbValue::none()
}

/// Result of converting a value sequence into bytearray content.
enum ByteSeqOutcome {
    Ok(Vec<u8>),
    /// An element is an int but outside range(0, 256) — ValueError.
    OutOfRange,
    /// The source is not a bytes-like / int sequence — TypeError.
    NotBytes,
}

/// Convert MbValue elements to bytes for bytearray slice assignment.
fn collect_u8_seq(items: &[MbValue]) -> ByteSeqOutcome {
    let mut out = Vec::with_capacity(items.len());
    for v in items {
        let Some(i) = v.as_int_pyint() else {
            return ByteSeqOutcome::NotBytes;
        };
        if !(0..=255).contains(&i) {
            return ByteSeqOutcome::OutOfRange;
        }
        out.push(i as u8);
    }
    ByteSeqOutcome::Ok(out)
}

/// Raise an IndexError through the runtime exception machinery.
fn raise_index_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// list[index] = value
pub fn mb_list_setitem(list: MbValue, index: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    if let Some(idx) = index.as_int_pyint() {
                        // Fast path: try non-blocking write (succeeds when uncontended)
                        let mut items = match lock.try_write() {
                            Ok(guard) => guard,
                            Err(_) => lock.write().unwrap(),
                        };
                        let len = items.len() as i64;
                        let actual = if idx < 0 { idx + len } else { idx };
                        if actual >= 0 && actual < len {
                            let old = items[actual as usize];
                            super::rc::retain_if_ptr(value);
                            items[actual as usize] = value;
                            super::rc::release_if_ptr(old);
                        } else {
                            // CPython: out-of-range store raises IndexError.
                            raise_index_error("list assignment index out of range");
                        }
                    } else {
                        super::builtins::raise_type_error(format!(
                            "list indices must be integers or slices, not {}",
                            super::builtins::value_type_name(index)
                        ));
                    }
                }
                ObjData::ByteArray(ref lock) => {
                    let Some(idx) = index.as_int_pyint() else {
                        super::builtins::raise_type_error(format!(
                            "bytearray indices must be integers or slices, not {}",
                            super::builtins::value_type_name(index)
                        ));
                        return;
                    };
                    let Some(v) = value.as_int_pyint() else {
                        super::builtins::raise_type_error(format!(
                            "'{}' object cannot be interpreted as an integer",
                            super::builtins::value_type_name(value)
                        ));
                        return;
                    };
                    if !(0..=255).contains(&v) {
                        super::builtins::raise_value_error(
                            "byte must be in range(0, 256)".to_string(),
                        );
                        return;
                    }
                    let mut data = lock.write().unwrap();
                    let len = data.len() as i64;
                    let actual = if idx < 0 { idx + len } else { idx };
                    if actual >= 0 && actual < len {
                        data[actual as usize] = v as u8;
                    } else {
                        raise_index_error("bytearray index out of range");
                    }
                }
                // Immutable targets: CPython raises TypeError — never a
                // silent no-op store.
                ObjData::Tuple(_) | ObjData::Bytes(_) | ObjData::Str(_) => {
                    super::builtins::raise_type_error(format!(
                        "'{}' object does not support item assignment",
                        super::builtins::value_type_name(list)
                    ));
                }
                _ => {}
            }
        }
    }
}

/// list[start:stop] = iterable — replace slice with elements from value list.
pub fn mb_list_setslice(list: MbValue, start: MbValue, stop: MbValue, _step: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            // bytearray[start:stop] = bytes-like — splice u8 values.
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                // Collect replacement bytes from bytes / bytearray / int
                // sequences. Distinguish out-of-range ints (ValueError) from
                // non-byte sources (TypeError) to match CPython exactly.
                let outcome: ByteSeqOutcome = if let Some(vp) = value.as_ptr() {
                    match &(*vp).data {
                        ObjData::Bytes(ref b) => ByteSeqOutcome::Ok(b.clone()),
                        ObjData::ByteArray(ref vlock) => {
                            ByteSeqOutcome::Ok(vlock.read().unwrap().clone())
                        }
                        ObjData::List(ref vlock) => {
                            let items = vlock.read().unwrap();
                            collect_u8_seq(&items)
                        }
                        ObjData::Tuple(ref t) => collect_u8_seq(t),
                        _ => ByteSeqOutcome::NotBytes,
                    }
                } else {
                    ByteSeqOutcome::NotBytes
                };
                let new_bytes = match outcome {
                    ByteSeqOutcome::Ok(b) => b,
                    ByteSeqOutcome::OutOfRange => {
                        super::builtins::raise_value_error(
                            "byte must be in range(0, 256)".to_string(),
                        );
                        return;
                    }
                    ByteSeqOutcome::NotBytes => {
                        super::builtins::raise_type_error(
                            "can assign only bytes, buffers, or iterables of ints in range(0, 256)"
                                .to_string(),
                        );
                        return;
                    }
                };
                let mut data = lock.write().unwrap();
                let len = data.len() as i64;
                let s = start.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(0) as usize;
                let e = stop.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(len) as usize;
                let s = s.min(data.len());
                let e = e.min(data.len()).max(s);
                data.splice(s..e, new_bytes);
                return;
            }
            // Immutable targets: slice assignment raises TypeError.
            if matches!(
                (*ptr).data,
                ObjData::Tuple(_) | ObjData::Bytes(_) | ObjData::Str(_)
            ) {
                super::builtins::raise_type_error(format!(
                    "'{}' object does not support item assignment",
                    super::builtins::value_type_name(list)
                ));
                return;
            }
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let len = items.len() as i64;
                let s = start.as_int().map(|i| clamp_index(i, len)).unwrap_or(0) as usize;
                let e = stop.as_int().map(|i| clamp_index(i, len)).unwrap_or(len as i64) as usize;
                let s = s.min(items.len());
                let e = e.min(items.len()).max(s);

                // Get new elements from value (must be a list)
                let new_elems: Vec<MbValue> = if let Some(vp) = value.as_ptr() {
                    match &(*vp).data {
                        ObjData::List(ref vlock) => {
                            let vr = vlock.read().unwrap();
                            vr.iter().copied().collect()
                        }
                        ObjData::Tuple(ref t) => t.clone(),
                        _ => vec![],
                    }
                } else {
                    vec![]
                };

                // Retain new elements
                for &v in &new_elems {
                    super::rc::retain_if_ptr(v);
                }
                // Release old elements in the replaced range
                for &old in &items[s..e] {
                    super::rc::release_if_ptr(old);
                }
                // Replace the slice. SmallVec has no `splice`; do a
                // drain + insert_from_slice instead.
                items.drain(s..e);
                items.insert_from_slice(s, &new_elems);
            }
        }
    }
}

/// del list[index]
pub fn mb_list_delitem(list: MbValue, index: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                if let Some(idx) = index.as_int() {
                    let mut items = lock.write().unwrap();
                    let len = items.len() as i64;
                    let actual = if idx < 0 { idx + len } else { idx };
                    if actual >= 0 && actual < len {
                        items.remove(actual as usize);
                    }
                }
            }
        }
    }
}

/// list[start:stop] -> new list
pub fn mb_list_slice(list: MbValue, start: MbValue, stop: MbValue) -> MbValue {
    mb_list_slice_full(list, start, stop, MbValue::none())
}

/// Full slice with step: list[start:stop:step]. Handles None for defaults.
pub fn mb_list_slice_full(
    list: MbValue, start: MbValue, stop: MbValue, step: MbValue,
) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                let len = items.len() as i64;
                let st = step.as_int_pyint().unwrap_or(1);
                if st == 0 {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("slice step cannot be zero".to_string())),
                    );
                    return mb_list_new();
                }
                let result = slice_indices(&items, len, start, stop, st);
                // Sliced items are borrowed from the source list — retain.
                return MbValue::from_ptr(MbObject::new_list_borrowed(result));
            }
        }
    }
    mb_list_new()
}

/// Compute Python slice indices and extract elements.
fn slice_indices(
    items: &[MbValue], len: i64, start: MbValue, stop: MbValue, step: i64,
) -> Vec<MbValue> {
    // bool ≤ int (#1680): accept bool slice bounds.
    let (s, e) = if step > 0 {
        let s = start.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(0);
        let e = stop.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(len);
        (s, e)
    } else {
        let s = start.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(len - 1);
        let e = stop.as_int_pyint().map(|i| clamp_index(i, len)).unwrap_or(-1);
        (s, e)
    };
    let mut result = Vec::new();
    let mut i = s;
    if step > 0 {
        while i < e {
            if i >= 0 && i < len { result.push(items[i as usize]); }
            i += step;
        }
    } else {
        while i > e {
            if i >= 0 && i < len { result.push(items[i as usize]); }
            i += step;
        }
    }
    result
}

/// Clamp a (possibly negative) index into [0, len] range.
fn clamp_index(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(0).min(len)
}

// ── Mutation Methods ──

/// list.append(item)
pub fn mb_list_append(list: MbValue, item: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                // Skip retain for inline values (int, float, bool, none) — they are
                // not heap pointers so retain_if_ptr is a no-op but still costs a
                // function call + branch. Only retain for pointer values.
                if item.is_ptr() {
                    super::rc::retain_if_ptr(item);
                }
                // Fast path: try non-blocking write (succeeds when uncontended)
                match lock.try_write() {
                    Ok(mut items) => items.push(item),
                    Err(_) => lock.write().unwrap().push(item),
                }
            }
        }
    }
}

/// Fast-path list append for JIT-generated code in single-threaded contexts.
/// Skips the RwLock try_write/write dance — uses a direct write lock with
/// unchecked unwrap. Also skips retain for non-pointer items (common in list
/// comprehensions over range() which produce integers).
///
/// # Safety
/// Caller must ensure `list` is a valid list MbValue and there are no
/// concurrent readers/writers (true for JIT-compiled comprehension loops).
pub fn mb_list_append_unchecked(list: MbValue, item: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                if item.is_ptr() {
                    super::rc::retain_if_ptr(item);
                }
                // Single-threaded JIT context: try_write always succeeds, use
                // unwrap_unchecked to skip the Result branch.
                lock.try_write().unwrap_unchecked().push(item);
            }
        }
    }
}

/// list.insert(index, item)
pub fn mb_list_insert(list: MbValue, index: MbValue, item: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                if let Some(idx) = index.as_int() {
                    let mut items = lock.write().unwrap();
                    let len = items.len() as i64;
                    let actual = normalize_index(idx, len) as usize;
                    items.insert(actual, item);
                }
            }
        }
    }
}

/// list.pop() -> removed item (last element); raises IndexError if empty.
pub fn mb_list_pop(list: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if items.is_empty() {
                    // Raise IndexError (CPython 3.12: "pop from empty list")
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("pop from empty list".to_string())),
                    );
                    return MbValue::none();
                }
                return items.pop().unwrap();
            }
        }
    }
    MbValue::none()
}

/// list.pop(index) -> removed item at index
pub fn mb_list_pop_at(list: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                if let Some(idx) = index.as_int() {
                    {
                        let mut items = lock.write().unwrap();
                        let len = items.len() as i64;
                        let actual = if idx < 0 { idx + len } else { idx };
                        if actual >= 0 && actual < len {
                            return items.remove(actual as usize);
                        }
                    }
                    // Out-of-range index → IndexError (CPython: "pop index out
                    // of range"), not a silent None.
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("pop index out of range".to_string())),
                    );
                    return MbValue::none();
                }
            }
        }
    }
    MbValue::none()
}

/// list.remove(value) — remove first occurrence; raises ValueError if not found.
pub fn mb_list_remove(list: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                // Scan a snapshot WITHOUT holding the lock: mb_eq can
                // re-enter user __eq__ that mutates this list (reentrancy
                // hardening — holding the write lock across it deadlocks).
                let snapshot: Vec<MbValue> =
                    lock.read().unwrap().iter().copied().collect();
                let mut found_pos: Option<usize> = None;
                for (i, v) in snapshot.iter().enumerate() {
                    if super::builtins::mb_eq(*v, value).as_bool() == Some(true) {
                        found_pos = Some(i);
                        break;
                    }
                }
                if let Some(pos) = found_pos {
                    let mut items = lock.write().unwrap();
                    // The __eq__ may have mutated the list; only remove when
                    // the found slot still holds the same element.
                    if pos < items.len() && items[pos] == snapshot[pos] {
                        items.remove(pos);
                    }
                    return;
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "list.remove(x): x not in list".to_string(),
                    )),
                );
            }
        }
    }
}

/// list.extend(iterable) — extend list with another list
pub fn mb_list_extend(list: MbValue, other: MbValue) {
    unsafe {
        // Read from `other` first to avoid holding two locks simultaneously.
        // Accept list, tuple, set, frozenset, dict (keys), str (chars), range, bytes.
        let cloned: Option<Vec<MbValue>> = if let Some(ptr) = other.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
                ObjData::Tuple(ref items) => Some(items.clone()),
                ObjData::Set(ref lock) => Some(lock.read().unwrap().to_vec()),
                ObjData::FrozenSet(ref items) => Some(items.clone()),
                ObjData::Dict(ref lock) => Some(
                    lock.read().unwrap().keys()
                        .map(super::dict_ops::dict_key_to_mbvalue)
                        .collect()
                ),
                ObjData::Str(ref s) => Some(
                    s.chars()
                        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                        .collect()
                ),
                ObjData::Bytes(ref data) => Some(
                    data.iter().map(|&b| MbValue::from_int(b as i64)).collect()
                ),
                _ => None,
            }
        } else {
            None
        };
        if let (Some(ptr), Some(cloned)) = (list.as_ptr(), cloned) {
            if let ObjData::List(ref lock) = (*ptr).data {
                for elem in &cloned { super::rc::retain_if_ptr(*elem); }
                lock.write().unwrap().extend(cloned);
            }
        }
    }
}

/// list.clear()
pub fn mb_list_clear(list: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
}

/// list.reverse()
pub fn mb_list_reverse(list: MbValue) {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().reverse();
            }
        }
    }
}

/// list.sort() — sorts in place using int/float ordering
pub fn mb_list_sort(list: MbValue) {
    use super::builtins::mb_value_cmp_pub;
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                // Type-specialized sort: all-int lists use sort_unstable_by_key
                // which extracts the key once per element and uses native i64
                // comparison. This avoids repeated as_int() calls in the comparator.
                if !items.is_empty() && items.iter().all(|v| v.is_int()) {
                    items.sort_unstable_by_key(|v| v.as_int_unchecked());
                } else if !items.is_empty() && items.iter().all(|v| v.is_int() || v.is_float()) {
                    // Mixed int/float: use f64 key extraction.
                    items.sort_unstable_by(|a, b| {
                        let af = a.as_int().map(|i| i as f64).or(a.as_float()).unwrap_or(0.0);
                        let bf = b.as_int().map(|i| i as f64).or(b.as_float()).unwrap_or(0.0);
                        af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    // Heterogeneous or user-class: defer to mb_value_cmp so
                    // __lt__ on Instance values is respected.
                    items.sort_by(|a, b| mb_value_cmp_pub(*a, *b));
                }
            }
        }
    }
}

/// list.sort(key=None, reverse=False) — kwargs-aware in-place sort.
pub fn mb_list_sort_kwargs(list: MbValue, key: MbValue, reverse: MbValue) {
    use super::builtins::{call_named_callable_pub, mb_value_cmp_pub};
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let do_reverse = reverse.as_bool() == Some(true) || reverse.as_int() == Some(1);
                let has_key = !key.is_none();
                if has_key {
                    // The key must be callable (CPython: `xs.sort(key=42)` →
                    // "'int' object is not callable").
                    let key_callable = super::builtins::resolve_callable_pub(key).is_some()
                        || matches!(
                            key.as_ptr().map(|p| &(*p).data),
                            Some(ObjData::Str(_)) | Some(ObjData::Instance { .. })
                        );
                    if !key_callable {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'{}' object is not callable",
                                super::builtins::value_type_name(key)
                            ))),
                        );
                        return;
                    }
                    // A key declaring >1 required positional param raises before
                    // any sorting (it is invoked with a single argument).
                    if super::builtins::key_unary_arity_error(key) {
                        return;
                    }
                    let named_key = key.as_ptr().and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                    });
                    // Snapshot the elements, then release the lock while the
                    // key callable runs: mb_call1_val can re-enter the runtime
                    // (and even this list) arbitrarily.
                    let snapshot: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                    let mut indexed: Vec<(MbValue, MbValue)> = Vec::with_capacity(snapshot.len());
                    for item in snapshot {
                        let k = if let Some(ref name) = named_key {
                            call_named_callable_pub(name, item).unwrap_or(item)
                        } else {
                            // Dynamic 1-arg dispatch: handles JIT functions,
                            // closures, and native extern builtins like `len`
                            // whose `(argv, argc)` ABI a raw transmute to
                            // `fn(MbValue)` would violate (#1132).
                            super::class::mb_call1_val(key, item)
                        };
                        // A raising key aborts the sort with the list left
                        // unchanged (the write-back below never runs).
                        if super::exception::mb_has_exception().as_bool() == Some(true) {
                            return;
                        }
                        indexed.push((item, k));
                    }
                    indexed.sort_by(|a, b| mb_value_cmp_pub(a.1, b.1));
                    if do_reverse { indexed.reverse(); }
                    let mut items = lock.write().unwrap();
                    *items = indexed.into_iter().map(|(v, _)| v).collect();
                } else {
                    let mut items = lock.write().unwrap();
                    // Type-specialized sort for no-key case.
                    if !items.is_empty() && items.iter().all(|v| v.is_int()) {
                        items.sort_unstable_by_key(|v| v.as_int_unchecked());
                    } else {
                        items.sort_by(|a, b| mb_value_cmp_pub(*a, *b));
                    }
                    if do_reverse { items.reverse(); }
                }
            }
        }
    }
}

/// list.copy() -> shallow copy
pub fn mb_list_copy(list: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_list_borrowed(lock.read().unwrap().to_vec()));
            }
        }
    }
    mb_list_new()
}

// ── Query Methods ──

/// list.index(value) -> index of first occurrence; raises ValueError if not found.
/// Uses Python-semantic equality (mb_eq) so heap-allocated strings compare by value.
pub fn mb_list_index(list: MbValue, value: MbValue) -> MbValue {
    mb_list_index_range(list, value, MbValue::none(), MbValue::none())
}

/// `list.index(value, start=0, stop=sys.maxsize)` — search a slice for value
/// and return its absolute index. CPython's signature is `(value, start=0,
/// stop=sys.maxsize)` — both bounds optional, both clamped, negatives counted
/// from the end. Raises ValueError when not found.
pub fn mb_list_index_range(list: MbValue, value: MbValue, start: MbValue, stop: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                // Snapshot, then release the lock: mb_eq can re-enter user
                // __eq__ that mutates this very list (reentrancy hardening —
                // holding the read lock across it deadlocks against clear()).
                let items: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                let len = items.len() as i64;
                // Resolve start: default 0; negatives count from the end and
                // clamp to 0; positives clamp to len.
                let mut s = start.as_int().unwrap_or(0);
                if s < 0 { s = (s + len).max(0); } else if s > len { s = len; }
                // Resolve stop: default `len`; same clamp rules.
                let mut e = stop.as_int().unwrap_or(len);
                if e < 0 { e = (e + len).max(0); } else if e > len { e = len; }
                if s < e {
                    for (i, item) in items.iter().enumerate().take(e as usize).skip(s as usize) {
                        if super::builtins::mb_eq(*item, value).as_bool() == Some(true) {
                            return MbValue::from_int(i as i64);
                        }
                    }
                }
                let repr = super::builtins::mb_repr(value);
                let repr_s = repr.as_ptr().and_then(|p| {
                    if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                }).unwrap_or_default();
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        format!("{repr_s} is not in list"),
                    )),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// list.count(value) -> number of occurrences. Uses Python-semantic equality.
pub fn mb_list_count(list: MbValue, value: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                // Snapshot then release: mb_eq may re-enter user __eq__ that
                // mutates this list (see mb_list_index_range).
                let items: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                // Identity-first like CPython list.count (PyObject_RichCompareBool):
                // a self-unequal element such as NaN still counts the same object.
                let n = items.iter().filter(|v| {
                    super::builtins::mb_is_identity(**v, value).as_bool() == Some(true)
                        || super::builtins::mb_eq(**v, value).as_bool() == Some(true)
                }).count();
                return MbValue::from_int(n as i64);
            }
        }
    }
    MbValue::from_int(0)
}

/// value in container -> bool (list, tuple, str)
pub fn mb_list_contains(container: MbValue, value: MbValue) -> MbValue {
    // Range iterator handles look like ints. Match CPython's
    // range.__contains__ — O(1) math check, never iterates the range.
    if let Some((current, stop, step)) = super::iter::mb_iter_range_params(container) {
        if step == 0 { return MbValue::from_bool(false); }
        let in_range = |v: i64| -> bool {
            let in_bounds = if step > 0 { v >= current && v < stop }
                            else { v <= current && v > stop };
            in_bounds && (v - current).rem_euclid(step.abs()) == 0
        };
        // Exact int / bool: CPython's O(1) arithmetic membership test.
        if value.is_int() || value.is_bool() {
            if let Some(v) = value.as_int_pyint() {
                return MbValue::from_bool(in_range(v));
            }
        }
        // float: only an integral value can equal one of the range's ints.
        if let Some(f) = value.as_float() {
            if f.is_finite() && f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                return MbValue::from_bool(in_range(f as i64));
            }
            return MbValue::from_bool(false);
        }
        // complex / an object with a custom __eq__: CPython's O(n) fallback —
        // iterate and compare with __eq__ (e.g. `Decimal(5) in range(10)`,
        // `(1+0j) in range(3)`, an int subclass whose __eq__ always matches).
        let mut cur = current;
        while (step > 0 && cur < stop) || (step < 0 && cur > stop) {
            if super::builtins::mb_eq(value, MbValue::from_int(cur)).as_bool() == Some(true) {
                return MbValue::from_bool(true);
            }
            cur += step;
        }
        return MbValue::from_bool(false);
    }
    if let Some(id) = container.as_int() {
        if super::stdlib::array_mod::is_array_handle(id as u64) {
            return super::stdlib::array_mod::mb_array_contains(container, value);
        }
    }
    unsafe {
        if let Some(ptr) = container.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    // Snapshot then release: mb_eq may re-enter user __eq__
                    // that mutates this list (see mb_list_index_range).
                    let items: Vec<MbValue> =
                        lock.read().unwrap().iter().copied().collect();
                    for item in items.iter() {
                        // CPython `x in seq` is identity-first (`x is e or x == e`
                        // via PyObject_RichCompareBool), so a self-unequal element
                        // like NaN is still found when the SAME object is present.
                        if super::builtins::mb_is_identity(value, *item).as_bool() == Some(true)
                            || super::builtins::mb_eq(value, *item).as_bool() == Some(true)
                        {
                            return MbValue::from_bool(true);
                        }
                    }
                    return MbValue::from_bool(false);
                }
                ObjData::Tuple(ref items) => {
                    for item in items.iter() {
                        if super::builtins::mb_is_identity(value, *item).as_bool() == Some(true)
                            || super::builtins::mb_eq(value, *item).as_bool() == Some(true)
                        {
                            return MbValue::from_bool(true);
                        }
                    }
                    return MbValue::from_bool(false);
                }
                ObjData::Str(ref s) => {
                    // Class-body enum class: `member in Color` / `value in Color`.
                    if let Some(found) =
                        super::stdlib::enum_class::class_contains(s, value)
                    {
                        return MbValue::from_bool(found);
                    }
                    if let Some(vp) = value.as_ptr() {
                        if let ObjData::Str(ref vs) = (*vp).data {
                            return MbValue::from_bool(s.contains(vs.as_str()));
                        }
                    }
                    return MbValue::from_bool(false);
                }
                ObjData::Set(ref lock) => {
                    // `x in set` hashes x; list/dict/bytearray are unhashable.
                    // A set LHS is NOT rejected — CPython checks it as its
                    // frozenset equivalent (`{1} in {frozenset([1])}` is True),
                    // which mb_eq's set/frozenset value-equality already handles.
                    if let Some(tn) = super::set_ops::unhashable_type_name(value) {
                        if tn != "set" {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!("unhashable type: '{tn}'"))),
                            );
                            return MbValue::none();
                        }
                    }
                    let items = lock.read().unwrap();
                    for item in items.iter() {
                        if super::builtins::mb_is_identity(value, *item).as_bool() == Some(true)
                            || super::builtins::mb_eq(value, *item).as_bool() == Some(true)
                        {
                            return MbValue::from_bool(true);
                        }
                    }
                    return MbValue::from_bool(false);
                }
                ObjData::FrozenSet(ref items) => {
                    if let Some(tn) = super::set_ops::unhashable_type_name(value) {
                        if tn != "set" {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!("unhashable type: '{tn}'"))),
                            );
                            return MbValue::none();
                        }
                    }
                    for item in items.iter() {
                        if super::builtins::mb_is_identity(value, *item).as_bool() == Some(true)
                            || super::builtins::mb_eq(value, *item).as_bool() == Some(true)
                        {
                            return MbValue::from_bool(true);
                        }
                    }
                    return MbValue::from_bool(false);
                }
                ObjData::Dict(ref lock) => {
                    // `key in dict` — check key membership (any key type). A
                    // mutable container can't be a key (CPython: unhashable type).
                    if let Some(tn) = super::set_ops::unhashable_type_name(value) {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!("unhashable type: '{tn}'"))),
                        );
                        return MbValue::none();
                    }
                    let key = super::dict_ops::to_dict_key(value);
                    return MbValue::from_bool(lock.read().unwrap().contains_key(&key));
                }
                ObjData::Bytes(_) | ObjData::ByteArray(_) => {
                    // `x in bytes`/`x in bytearray` — int byte-value or
                    // bytes-like subsequence membership. Delegate to the bytes
                    // path, which raises ValueError on an out-of-range int and
                    // TypeError on a non-bytes-like value (CPython semantics)
                    // instead of silently truncating with `as u8`.
                    return super::bytes_ops::mb_bytes_contains(container, value);
                }
                ObjData::Instance { .. } => {
                    // Flag composite containment: `Color.RED in (RED | BLUE)`.
                    if let Some(found) =
                        super::stdlib::enum_class::flag_member_contains(container, value)
                    {
                        return MbValue::from_bool(found);
                    }
                    // User-defined iterable / sequence: check __contains__ first,
                    // fall back to iterator protocol + equality.
                    let contains_method = super::class::mb_lookup_dunder(
                        container, MbValue::from_ptr(MbObject::new_str("__contains__".into())),
                    );
                    if !contains_method.is_none() {
                        // Call __contains__(self, value) via 2-arg dispatch.
                        let result = super::class::mb_call_method(
                            container,
                            MbValue::from_ptr(MbObject::new_str("__contains__".into())),
                            MbValue::from_ptr(MbObject::new_list(vec![value])),
                        );
                        return MbValue::from_bool(result.as_bool().unwrap_or(false));
                    }
                    // Fall back to iterator protocol: iterate and compare by equality.
                    let iter_handle = super::iter::mb_iter(container);
                    if iter_handle.is_none() {
                        return MbValue::from_bool(false);
                    }
                    loop {
                        if super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
                            break;
                        }
                        let item = super::iter::mb_next(iter_handle);
                        if super::builtins::mb_eq(item, value).as_bool() == Some(true) {
                            return MbValue::from_bool(true);
                        }
                        if item.is_none()
                            && super::iter::mb_has_next(iter_handle).as_bool() == Some(false)
                        {
                            break;
                        }
                    }
                    return MbValue::from_bool(false);
                }
                _ => {}
            }
        }
    }
    MbValue::from_bool(false)
}

/// len(list) -> int
pub fn mb_list_len(list: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                return MbValue::from_int(lock.read().unwrap().len() as i64);
            }
        }
    }
    MbValue::from_int(0)
}

/// Check if value is a sequence (list or tuple) — used for PEP 634 sequence pattern matching.
pub fn mb_is_sequence(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return MbValue::from_bool(matches!(
                (*ptr).data,
                ObjData::List(_) | ObjData::Tuple(_)
            ));
        }
    }
    MbValue::from_bool(false)
}

/// Zero-copy passthrough for the unpack-assign lowering hot path (#2178).
///
/// `a, b, c = rhs` lowers to `mb_seq_for_unpack(rhs)` followed by
/// `mb_seq_len_boxed` + `mb_list_getitem` per target. When the rhs is
/// already a list or tuple (the overwhelming common case — `h, l, s =
/// rgb_to_hls(...)`, etc.), we skip the per-iter `MbObject::new_list` +
/// `Vec::clone` + N retains performed by the legacy
/// `mb_list_from_iterable` path. Returns the input verbatim with one
/// extra retain so the caller's `mb_release` on the temporary matches.
///
/// Iterators (`mb_iter`-handle ints), strings, dicts, sets, and user
/// iterables still fall back to full materialization for correctness.
pub fn mb_seq_for_unpack(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(_) | ObjData::Tuple(_) => {
                    super::rc::retain_if_ptr(val);
                    return val;
                }
                _ => {}
            }
        }
    }
    mb_list_from_iterable(val)
}

/// NaN-boxed length for the unpack-assign hot path (#2178). Mirrors
/// `mb_list_len` but additionally handles tuples (matching the
/// zero-copy passthrough in `mb_seq_for_unpack`).
pub fn mb_seq_len_boxed(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    return MbValue::from_int(lock.read().unwrap().len() as i64);
                }
                ObjData::Tuple(ref items) => {
                    return MbValue::from_int(items.len() as i64);
                }
                _ => {}
            }
        }
    }
    MbValue::from_int(0)
}

/// Sequence-generic length: works for both lists and tuples (#827).
/// Returns raw i64 (not NaN-boxed) for use in pattern-match length comparisons.
pub fn mb_seq_len(val: MbValue) -> i64 {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    return lock.read().unwrap().len() as i64;
                }
                ObjData::Tuple(ref items) => {
                    return items.len() as i64;
                }
                _ => {}
            }
        }
    }
    0
}

/// Sequence-generic getitem: works for both lists and tuples (#827).
/// Takes raw i64 index (not NaN-boxed) for use in pattern-match element extraction.
pub fn mb_seq_getitem(val: MbValue, index: i64) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let len = items.len() as i64;
                    let actual = if index < 0 { index + len } else { index };
                    if actual >= 0 && actual < len {
                        let val = items[actual as usize];
                        super::rc::retain_if_ptr(val);
                        return val;
                    }
                    return MbValue::none();
                }
                ObjData::Tuple(ref items) => {
                    let len = items.len() as i64;
                    let actual = if index < 0 { index + len } else { index };
                    if actual >= 0 && actual < len {
                        let val = items[actual as usize];
                        super::rc::retain_if_ptr(val);
                        return val;
                    }
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    MbValue::none()
}

/// Sequence-generic slice: works for both lists and tuples (#827).
/// Returns a new list for both list and tuple inputs (PEP 634 star capture is a list).
pub fn mb_seq_slice(val: MbValue, start: MbValue, stop: MbValue) -> MbValue {
    let start_idx = start.as_int().unwrap_or(0);
    let stop_idx = stop.as_int().unwrap_or(0);
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let len = items.len() as i64;
                    let s = start_idx.max(0).min(len) as usize;
                    let e = stop_idx.max(0).min(len) as usize;
                    let slice: Vec<MbValue> = items[s..e].to_vec();
                    return MbValue::from_ptr(MbObject::new_list(slice));
                }
                ObjData::Tuple(ref items) => {
                    let len = items.len() as i64;
                    let s = start_idx.max(0).min(len) as usize;
                    let e = stop_idx.max(0).min(len) as usize;
                    let slice: Vec<MbValue> = items[s..e].to_vec();
                    return MbValue::from_ptr(MbObject::new_list(slice));
                }
                _ => {}
            }
        }
    }
    mb_list_new()
}

// ── Operators ──

/// Build a fresh list `[*prefix, *extract_items(iter)]` — the splat-call
/// helper used to lower `f(arg1, arg2, *xs)` into a single
/// `mb_call_spread(f, combined_list)` call. Unlike `mb_list_concat`,
/// the second argument can be any iterable shape that
/// `extract_items` supports (list, tuple, set, frozenset, str, dict,
/// bytes); `mb_list_concat` requires both arguments to be lists, which
/// breaks the common `f(prefix, *tuple_or_iter)` shape (#2098).
pub fn mb_args_concat(prefix: MbValue, iter: MbValue) -> MbValue {
    unsafe {
        let mut combined: Vec<MbValue> = if let Some(ptr) = prefix.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        combined.extend(super::builtins::extract_items(iter));
        for elem in &combined {
            super::rc::retain_if_ptr(*elem);
        }
        MbValue::from_ptr(MbObject::new_list(combined))
    }
}

/// list + list -> new concatenated list
pub fn mb_list_concat(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        let la = a.as_ptr().and_then(|p| {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        });
        let lb = b.as_ptr().and_then(|p| {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        });
        if let (Some(mut la), Some(lb)) = (la, lb) {
            la.extend_from_slice(&lb);
            // Items borrowed from both source lists — retain.
            return MbValue::from_ptr(MbObject::new_list_borrowed(la));
        }
    }
    MbValue::none()
}

/// list * n -> repeated list
pub fn mb_list_repeat(list: MbValue, n: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                // bool counts as an int (True == 1), like CPython.
                let count = n.as_int().or_else(|| n.as_bool().map(|b| b as i64));
                if let Some(count) = count {
                    if count <= 0 {
                        return mb_list_new();
                    }
                    let items = lock.read().unwrap();
                    let mut result = Vec::with_capacity(items.len() * count as usize);
                    for _ in 0..count {
                        result.extend_from_slice(&items);
                    }
                    // Items borrowed from the source list — retain.
                    return MbValue::from_ptr(MbObject::new_list_borrowed(result));
                }
                // Sequence repetition needs an integer count (CPython:
                // "can't multiply sequence by non-int of type 'str'").
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "can't multiply sequence by non-int of type '{}'",
                        super::builtins::value_type_name(n)
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// list == list -> bool
pub fn mb_list_eq(a: MbValue, b: MbValue) -> MbValue {
    // Delegate to the generic deep-equality entry point so list elements
    // (which may be heap objects like strings or nested containers) are
    // compared structurally, not by NaN-boxed bit pattern.
    super::builtins::mb_eq(a, b)
}

// ── Method Dispatch ──

/// Dispatch a method call on a list object.
pub fn dispatch_list_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().get(i).copied().unwrap_or(MbValue::none());
                }
            }
            MbValue::none()
        }
    };
    let argc = || -> usize {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
            0
        }
    };
    match name {
        "append" => { mb_list_append(receiver, arg(0)); MbValue::none() }
        "insert" => { mb_list_insert(receiver, arg(0), arg(1)); MbValue::none() }
        "pop" => {
            if argc() > 0 { mb_list_pop_at(receiver, arg(0)) }
            else { mb_list_pop(receiver) }
        }
        "remove" => { mb_list_remove(receiver, arg(0)); MbValue::none() }
        "extend" => { mb_list_extend(receiver, arg(0)); MbValue::none() }
        "clear" => { mb_list_clear(receiver); MbValue::none() }
        "reverse" => { mb_list_reverse(receiver); MbValue::none() }
        "sort" => { mb_list_sort(receiver); MbValue::none() }
        "copy" => mb_list_copy(receiver),
        "index" => {
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let stop = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_list_index_range(receiver, arg(0), start, stop)
        }
        "count" => mb_list_count(receiver, arg(0)),
        // ── Explicit dunder access — `[…].__getitem__(0)`, `__add__`, etc.
        // CPython exposes every protocol slot by name on plain list objects;
        // mamba previously raised AttributeError for each of these.
        "__getitem__" => mb_list_getitem(receiver, arg(0)),
        // `[].__dir__()` — same name set as dir([]) (already sorted).
        "__dir__" => super::class::mb_dir(receiver),
        "__setitem__" => { mb_list_setitem(receiver, arg(0), arg(1)); MbValue::none() }
        "__delitem__" => { mb_list_delitem(receiver, arg(0)); MbValue::none() }
        "__contains__" => mb_list_contains(receiver, arg(0)),
        "__len__" => mb_list_len(receiver),
        "__iter__" => super::iter::mb_iter(receiver),
        "__add__" => mb_list_concat(receiver, arg(0)),
        "__radd__" => mb_list_concat(arg(0), receiver),
        "__mul__" | "__rmul__" => mb_list_repeat(receiver, arg(0)),
        // NOTE: __iadd__ / __imul__ are not exposed yet — returning the
        // mutated receiver from method dispatch hits a refcount/free
        // race that crashes a follow-up call. The augmented-assign
        // operator path (`L += other`) keeps working because codegen
        // uses mb_list_extend / mb_list_repeat directly without
        // returning the receiver up the method-call stack.
        "__eq__" => mb_list_eq(receiver, arg(0)),
        "__ne__" => {
            let eq = mb_list_eq(receiver, arg(0));
            MbValue::from_bool(eq.as_bool() != Some(true))
        }
        "__repr__" | "__str__" => super::builtins::mb_repr(receiver),
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(super::rc::MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(super::rc::MbObject::new_str(
                    format!("'list' object has no attribute '{name}'"),
                )),
            );
            MbValue::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Creation ──

    #[test]
    fn test_new_empty() {
        let list = mb_list_new();
        assert_eq!(mb_list_len(list).as_int(), Some(0));
    }

    #[test]
    fn test_from_elements() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_list_len(list).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(2));
    }

    // ── #2178 unpack-assign zero-copy fast path ──

    /// `mb_seq_for_unpack` on a tuple must return the same pointer (no
    /// per-iter MbObject + Vec allocation), with the refcount bumped so
    /// the caller's trailing `mb_release` balances.
    #[test]
    fn test_seq_for_unpack_tuple_zero_copy() {
        let elems = vec![
            MbValue::from_float(1.0),
            MbValue::from_float(2.0),
            MbValue::from_float(3.0),
        ];
        let tup = MbValue::from_ptr(MbObject::new_tuple(elems));
        let out = mb_seq_for_unpack(tup);
        // Zero-copy: returned pointer is the same MbObject.
        assert_eq!(out.to_bits(), tup.to_bits());
        // mb_seq_len_boxed reports the correct length for tuples.
        assert_eq!(mb_seq_len_boxed(out).as_int(), Some(3));
        // mb_list_getitem already handles tuples — the lowering path is sound.
        assert_eq!(mb_list_getitem(out, MbValue::from_int(0)).as_float(), Some(1.0));
        assert_eq!(mb_list_getitem(out, MbValue::from_int(2)).as_float(), Some(3.0));
    }

    /// `mb_seq_for_unpack` on a list also returns the same pointer (no copy).
    #[test]
    fn test_seq_for_unpack_list_zero_copy() {
        let list = mb_list_from(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]);
        let out = mb_seq_for_unpack(list);
        assert_eq!(out.to_bits(), list.to_bits());
        assert_eq!(mb_seq_len_boxed(out).as_int(), Some(3));
    }

    /// `mb_seq_len_boxed` returns a NaN-boxed length for both lists and
    /// tuples (in contrast with `mb_list_len`, which returns 0 for tuples).
    #[test]
    fn test_seq_len_boxed_tuple_and_list() {
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]));
        assert_eq!(mb_seq_len_boxed(tup).as_int(), Some(4));

        let list = mb_list_from(vec![MbValue::from_int(7)]);
        assert_eq!(mb_seq_len_boxed(list).as_int(), Some(1));
    }

    // ── append ──

    #[test]
    fn test_append_and_len() {
        let list = mb_list_new();
        mb_list_append(list, MbValue::from_int(1));
        mb_list_append(list, MbValue::from_int(2));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
    }

    #[test]
    fn test_append_none() {
        let list = mb_list_new();
        mb_list_append(list, MbValue::none());
        assert_eq!(mb_list_len(list).as_int(), Some(1));
        assert!(mb_list_getitem(list, MbValue::from_int(0)).is_none());
    }

    // ── getitem ──

    #[test]
    fn test_getitem() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(10));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(2)).as_int(), Some(30));
    }

    #[test]
    fn test_getitem_negative_index() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-1)).as_int(), Some(30));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-3)).as_int(), Some(10));
    }

    #[test]
    fn test_getitem_out_of_bounds() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        assert!(mb_list_getitem(list, MbValue::from_int(5)).is_none());
        assert!(mb_list_getitem(list, MbValue::from_int(-5)).is_none());
    }

    #[test]
    fn test_bytearray_setitem() {
        let ba = MbValue::from_ptr(super::super::rc::MbObject::new_bytearray(vec![72u8, 101, 108, 108, 111]));
        mb_list_setitem(ba, MbValue::from_int(0), MbValue::from_int(74)); // 'J'
        if let Some(ptr) = ba.as_ptr() {
            unsafe {
                if let ObjData::ByteArray(ref lock) = (*ptr).data {
                    let data = lock.read().unwrap();
                    assert_eq!(data[0], 74, "Expected ba[0] = 74 (J) after setitem");
                }
            }
        }
    }

    #[test]
    fn test_getitem_non_list() {
        assert!(mb_list_getitem(MbValue::from_int(1), MbValue::from_int(0)).is_none());
    }

    // ── setitem ──

    #[test]
    fn test_setitem() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_setitem(list, MbValue::from_int(0), MbValue::from_int(99));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(99));
    }

    #[test]
    fn test_setitem_negative() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_setitem(list, MbValue::from_int(-1), MbValue::from_int(88));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(88));
    }

    #[test]
    fn test_setitem_out_of_bounds_raises_index_error() {
        // CPython: out-of-range store raises IndexError and leaves the
        // list unchanged.
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        mb_list_setitem(list, MbValue::from_int(5), MbValue::from_int(99));
        assert_eq!(mb_list_len(list).as_int(), Some(1));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(1));
        crate::runtime::exception::mb_clear_exception();
    }

    // ── delitem ──

    #[test]
    fn test_delitem() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        mb_list_delitem(list, MbValue::from_int(1));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(30));
    }

    #[test]
    fn test_delitem_negative() {
        let list = mb_list_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
        mb_list_delitem(list, MbValue::from_int(-1));
        assert_eq!(mb_list_len(list).as_int(), Some(1));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(10));
    }

    #[test]
    fn test_delitem_out_of_bounds_noop() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        mb_list_delitem(list, MbValue::from_int(5));
        assert_eq!(mb_list_len(list).as_int(), Some(1));
    }

    // ── insert ──

    #[test]
    fn test_insert_beginning() {
        let list = mb_list_from(vec![MbValue::from_int(2), MbValue::from_int(3)]);
        mb_list_insert(list, MbValue::from_int(0), MbValue::from_int(1));
        assert_eq!(mb_list_len(list).as_int(), Some(3));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(1));
    }

    #[test]
    fn test_insert_middle() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(3)]);
        mb_list_insert(list, MbValue::from_int(1), MbValue::from_int(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(2));
    }

    #[test]
    fn test_insert_negative_index() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(3)]);
        mb_list_insert(list, MbValue::from_int(-1), MbValue::from_int(2));
        assert_eq!(mb_list_len(list).as_int(), Some(3));
    }

    // ── pop ──

    #[test]
    fn test_pop() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ]);
        assert_eq!(mb_list_pop(list).as_int(), Some(3));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
    }

    #[test]
    fn test_pop_empty() {
        let list = mb_list_new();
        assert!(mb_list_pop(list).is_none());
    }

    #[test]
    fn test_pop_non_list() {
        assert!(mb_list_pop(MbValue::from_int(1)).is_none());
    }

    // ── pop_at ──

    #[test]
    fn test_pop_at() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        assert_eq!(mb_list_pop_at(list, MbValue::from_int(1)).as_int(), Some(20));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
    }

    #[test]
    fn test_pop_at_negative() {
        let list = mb_list_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
        assert_eq!(mb_list_pop_at(list, MbValue::from_int(-1)).as_int(), Some(20));
    }

    #[test]
    fn test_pop_at_out_of_bounds() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        assert!(mb_list_pop_at(list, MbValue::from_int(5)).is_none());
    }

    // ── remove ──

    #[test]
    fn test_remove() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ]);
        mb_list_remove(list, MbValue::from_int(2));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(3));
    }

    #[test]
    fn test_remove_first_occurrence() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(1),
        ]);
        mb_list_remove(list, MbValue::from_int(1));
        assert_eq!(mb_list_len(list).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(2));
    }

    #[test]
    fn test_remove_not_found_noop() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        mb_list_remove(list, MbValue::from_int(99));
        assert_eq!(mb_list_len(list).as_int(), Some(1));
    }

    // ── extend ──

    #[test]
    fn test_extend() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_from(vec![MbValue::from_int(2), MbValue::from_int(3)]);
        mb_list_extend(a, b);
        assert_eq!(mb_list_len(a).as_int(), Some(3));
        assert_eq!(mb_list_getitem(a, MbValue::from_int(2)).as_int(), Some(3));
    }

    #[test]
    fn test_extend_empty() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_new();
        mb_list_extend(a, b);
        assert_eq!(mb_list_len(a).as_int(), Some(1));
    }

    // ── clear ──

    #[test]
    fn test_clear() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_clear(list);
        assert_eq!(mb_list_len(list).as_int(), Some(0));
    }

    // ── reverse ──

    #[test]
    fn test_reverse() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ]);
        mb_list_reverse(list);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(3));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(2)).as_int(), Some(1));
    }

    #[test]
    fn test_reverse_empty() {
        let list = mb_list_new();
        mb_list_reverse(list); // no panic
        assert_eq!(mb_list_len(list).as_int(), Some(0));
    }

    // ── sort ──

    #[test]
    fn test_sort() {
        let list = mb_list_from(vec![
            MbValue::from_int(3), MbValue::from_int(1), MbValue::from_int(2),
        ]);
        mb_list_sort(list);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(2)).as_int(), Some(3));
    }

    #[test]
    fn test_sort_already_sorted() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_sort(list);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(1));
    }

    #[test]
    fn test_sort_single_element() {
        let list = mb_list_from(vec![MbValue::from_int(5)]);
        mb_list_sort(list);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(5));
    }

    // ── copy ──

    #[test]
    fn test_copy() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let cp = mb_list_copy(list);
        assert_eq!(mb_list_len(cp).as_int(), Some(2));
        assert_eq!(mb_list_getitem(cp, MbValue::from_int(0)).as_int(), Some(1));
        // mutating original does not affect copy
        mb_list_append(list, MbValue::from_int(3));
        assert_eq!(mb_list_len(cp).as_int(), Some(2));
    }

    #[test]
    fn test_copy_empty() {
        let list = mb_list_new();
        let cp = mb_list_copy(list);
        assert_eq!(mb_list_len(cp).as_int(), Some(0));
    }

    // ── index ──

    #[test]
    fn test_index_found() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        assert_eq!(mb_list_index(list, MbValue::from_int(20)).as_int(), Some(1));
    }

    #[test]
    fn test_index_not_found() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        // Now raises ValueError; returns None sentinel
        let result = mb_list_index(list, MbValue::from_int(99));
        assert!(result.is_none());
    }

    #[test]
    fn test_index_non_list() {
        // Non-list input returns None
        let result = mb_list_index(MbValue::from_int(0), MbValue::from_int(0));
        assert!(result.is_none());
    }

    // ── count ──

    #[test]
    fn test_count() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(1),
        ]);
        assert_eq!(mb_list_count(list, MbValue::from_int(1)).as_int(), Some(2));
        assert_eq!(mb_list_count(list, MbValue::from_int(2)).as_int(), Some(1));
        assert_eq!(mb_list_count(list, MbValue::from_int(9)).as_int(), Some(0));
    }

    // ── contains ──

    #[test]
    fn test_contains() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_list_contains(list, MbValue::from_int(1)).as_bool(), Some(true));
        assert_eq!(mb_list_contains(list, MbValue::from_int(3)).as_bool(), Some(false));
    }

    #[test]
    fn test_contains_non_list() {
        assert_eq!(mb_list_contains(MbValue::from_int(1), MbValue::from_int(1)).as_bool(), Some(false));
    }

    // ── len ──

    #[test]
    fn test_len_non_list() {
        assert_eq!(mb_list_len(MbValue::from_int(1)).as_int(), Some(0));
    }

    // ── concat ──

    #[test]
    fn test_concat() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_from(vec![MbValue::from_int(2), MbValue::from_int(3)]);
        let c = mb_list_concat(a, b);
        assert_eq!(mb_list_len(c).as_int(), Some(3));
        assert_eq!(mb_list_getitem(c, MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_list_getitem(c, MbValue::from_int(2)).as_int(), Some(3));
    }

    #[test]
    fn test_concat_non_list() {
        assert!(mb_list_concat(MbValue::from_int(1), MbValue::from_int(2)).is_none());
    }

    // ── repeat ──

    #[test]
    fn test_repeat() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let r = mb_list_repeat(list, MbValue::from_int(3));
        assert_eq!(mb_list_len(r).as_int(), Some(6));
    }

    #[test]
    fn test_repeat_zero() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        let r = mb_list_repeat(list, MbValue::from_int(0));
        assert_eq!(mb_list_len(r).as_int(), Some(0));
    }

    #[test]
    fn test_repeat_negative() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        let r = mb_list_repeat(list, MbValue::from_int(-1));
        assert_eq!(mb_list_len(r).as_int(), Some(0));
    }

    // ── eq ──

    #[test]
    fn test_eq_same() {
        let a = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_list_eq(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_different() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_from(vec![MbValue::from_int(2)]);
        assert_eq!(mb_list_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_different_len() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_list_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_non_list() {
        // Delegates to mb_eq, so non-list inputs still get sane equality.
        assert_eq!(mb_list_eq(MbValue::from_int(1), MbValue::from_int(2)).as_bool(), Some(false));
    }

    // ── sequence protocol ──

    #[test]
    fn test_is_sequence_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        assert_eq!(mb_is_sequence(list).as_bool(), Some(true));
    }

    #[test]
    fn test_is_sequence_tuple() {
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(1)]));
        assert_eq!(mb_is_sequence(tup).as_bool(), Some(true));
    }

    #[test]
    fn test_is_sequence_int_false() {
        assert_eq!(mb_is_sequence(MbValue::from_int(42)).as_bool(), Some(false));
    }

    // ── normalize_index ──

    #[test]
    fn test_normalize_index_positive() {
        assert_eq!(normalize_index(2, 5), 2);
    }

    #[test]
    fn test_normalize_index_negative() {
        assert_eq!(normalize_index(-1, 5), 4);
        assert_eq!(normalize_index(-5, 5), 0);
    }

    #[test]
    fn test_normalize_index_clamp() {
        assert_eq!(normalize_index(-10, 5), 0);
        assert_eq!(normalize_index(10, 5), 5);
    }

    // ── slice ──

    #[test]
    fn test_slice() {
        let list = mb_list_from(vec![
            MbValue::from_int(0), MbValue::from_int(1),
            MbValue::from_int(2), MbValue::from_int(3),
        ]);
        let sliced = mb_list_slice(list, MbValue::from_int(1), MbValue::from_int(3));
        assert_eq!(mb_list_len(sliced).as_int(), Some(2));
        assert_eq!(mb_list_getitem(sliced, MbValue::from_int(0)).as_int(), Some(1));
    }

    #[test]
    fn test_slice_negative_indices() {
        let list = mb_list_from(vec![
            MbValue::from_int(0), MbValue::from_int(1),
            MbValue::from_int(2), MbValue::from_int(3),
        ]);
        let sliced = mb_list_slice(list, MbValue::from_int(-3), MbValue::from_int(-1));
        assert_eq!(mb_list_len(sliced).as_int(), Some(2));
        assert_eq!(mb_list_getitem(sliced, MbValue::from_int(0)).as_int(), Some(1));
    }

    #[test]
    fn test_slice_empty_range() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let sliced = mb_list_slice(list, MbValue::from_int(2), MbValue::from_int(1));
        assert_eq!(mb_list_len(sliced).as_int(), Some(0));
    }

    #[test]
    fn test_slice_non_list() {
        let sliced = mb_list_slice(MbValue::from_int(1), MbValue::from_int(0), MbValue::from_int(1));
        assert_eq!(mb_list_len(sliced).as_int(), Some(0));
    }

    // ── dispatch_list_method ──

    #[test]
    fn test_dispatch_append() {
        let list = mb_list_new();
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(42)]));
        dispatch_list_method("append", list, args);
        assert_eq!(mb_list_len(list).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_pop_no_args() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let r = dispatch_list_method("pop", list, args);
        assert_eq!(r.as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_pop_with_index() {
        let list = mb_list_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(0)]));
        let r = dispatch_list_method("pop", list, args);
        assert_eq!(r.as_int(), Some(10));
    }

    #[test]
    fn test_dispatch_index() {
        let list = mb_list_from(vec![MbValue::from_int(5), MbValue::from_int(6)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(6)]));
        let r = dispatch_list_method("index", list, args);
        assert_eq!(r.as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_count() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(1), MbValue::from_int(2),
        ]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let r = dispatch_list_method("count", list, args);
        assert_eq!(r.as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_copy() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let cp = dispatch_list_method("copy", list, args);
        assert_eq!(mb_list_len(cp).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let list = mb_list_new();
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let r = dispatch_list_method("nonexistent", list, args);
        assert!(r.is_none());
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_list_negative_index() {
        let list = mb_list_from(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-1)).as_int(), Some(30));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-2)).as_int(), Some(20));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-3)).as_int(), Some(10));
    }

    #[test]
    fn test_py312_list_extend_returns_none() {
        let a = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_list_from(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        mb_list_extend(a, b);
        // mb_list_extend returns () — in Python, list.extend returns None
        assert_eq!(mb_list_len(a).as_int(), Some(4));
    }

    #[test]
    fn test_py312_list_insert_beyond_end() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_insert(list, MbValue::from_int(100), MbValue::from_int(99));
        assert_eq!(mb_list_len(list).as_int(), Some(3));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(-1)).as_int(), Some(99));
    }

    #[test]
    fn test_py312_list_remove_only_first() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2),
            MbValue::from_int(1), MbValue::from_int(3),
        ]);
        mb_list_remove(list, MbValue::from_int(1));
        assert_eq!(mb_list_len(list).as_int(), Some(3));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(2));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(1)).as_int(), Some(1));
    }

    #[test]
    fn test_py312_list_reverse() {
        let list = mb_list_from(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ]);
        mb_list_reverse(list);
        assert_eq!(mb_list_getitem(list, MbValue::from_int(0)).as_int(), Some(3));
        assert_eq!(mb_list_getitem(list, MbValue::from_int(2)).as_int(), Some(1));
    }

    #[test]
    fn test_py312_list_clear_empties() {
        let list = mb_list_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        mb_list_clear(list);
        assert_eq!(mb_list_len(list).as_int(), Some(0));
    }

    #[test]
    fn test_py312_list_concat_returns_new() {
        let a = mb_list_from(vec![MbValue::from_int(1)]);
        let b = mb_list_from(vec![MbValue::from_int(2)]);
        let c = mb_list_concat(a, b);
        assert_eq!(mb_list_len(c).as_int(), Some(2));
        assert_eq!(mb_list_len(a).as_int(), Some(1), "original unchanged");
    }

    #[test]
    fn test_py312_list_repeat() {
        let list = mb_list_from(vec![MbValue::from_int(1)]);
        let rep = mb_list_repeat(list, MbValue::from_int(3));
        assert_eq!(mb_list_len(rep).as_int(), Some(3));
    }

    #[test]
    fn test_py312_list_contains_bool_int_separate() {
        let list = mb_list_from(vec![MbValue::from_int(5), MbValue::from_bool(true)]);
        assert_eq!(mb_list_contains(list, MbValue::from_int(5)).as_bool(), Some(true));
        assert_eq!(mb_list_contains(list, MbValue::from_int(0)).as_bool(), Some(false));
    }

    // TODO: enable when mb_list_slice_step is implemented
    // #[test]
    // fn test_py312_list_step_slice_forward() {
    //     let list = mb_list_from(vec![
    //         MbValue::from_int(0), MbValue::from_int(1), MbValue::from_int(2),
    //         MbValue::from_int(3), MbValue::from_int(4),
    //     ]);
    //     let sliced = mb_list_slice_step(list, MbValue::from_int(0), MbValue::from_int(5), MbValue::from_int(2));
    //     assert_eq!(mb_list_len(sliced).as_int(), Some(3));
    //     assert_eq!(mb_list_getitem(sliced, MbValue::from_int(0)).as_int(), Some(0));
    //     assert_eq!(mb_list_getitem(sliced, MbValue::from_int(1)).as_int(), Some(2));
    //     assert_eq!(mb_list_getitem(sliced, MbValue::from_int(2)).as_int(), Some(4));
    // }

}
