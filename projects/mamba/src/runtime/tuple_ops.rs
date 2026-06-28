use super::rc::{MbObject, ObjData};
/// Tuple operations for the Mamba runtime (#285).
///
/// Tuples are immutable sequences. All mutation attempts should raise TypeError.
use super::value::MbValue;

/// Helper: get immutable reference to tuple elements.
unsafe fn as_tuple(val: MbValue) -> Option<&'static Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            Some(items)
        } else {
            None
        }
    })
}

fn normalize_index(idx: i64, len: i64) -> i64 {
    let i = if idx < 0 { idx + len } else { idx };
    i.max(0).min(len)
}

// ── Creation ──

/// Create a new tuple from elements.
pub fn mb_tuple_from(elements: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(elements))
}

/// Create an empty tuple.
pub fn mb_tuple_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

// ── Fixed-arity tuple constructors (#2128) ──
//
// MakeTuple in the JIT historically built an intermediate List, appended N
// elements, then called `mb_list_to_tuple` to convert it. That path
// gc_track'd the transient list on every tuple-return — even when the final
// tuple held only primitives and was itself elided from the tracked set by
// `new_tuple`'s cycle-capable check. For hot scalar-tuple returns
// (`rgb_to_hls`, `divmod`, `divmod_int`, etc.) the intermediate list's
// gc_track + gc_untrack pair dominated runtime, yielding the ~150-220x
// penalty vs CPython documented in #2128 — while the structurally similar
// `Complex` return path (no intermediate container) ran at 3x faster than
// CPython.
//
// These fixed-arity constructors collapse the build to a single FFI call
// that allocates the tuple directly. `new_tuple` already elides gc_track
// when no element is cycle-capable, so primitive-only tuples allocate with
// zero GC-set churn — matching `new_complex` / `new_bytes`.
//
// Arities mirror `mb_list_new_1..mb_list_new_10`; 1..=8 covers every
// observed tuple-return shape (rgb_to_hls = 3, UUID.fields = 6, divmod = 2).

macro_rules! retain_ptr_args {
    ($($arg:ident),*) => {
        unsafe {
            $( if $arg.is_ptr() { super::rc::retain_if_ptr($arg); } )*
        }
    };
}

pub fn mb_tuple_new_1(a: MbValue) -> MbValue {
    retain_ptr_args!(a);
    MbValue::from_ptr(MbObject::new_tuple(vec![a]))
}

pub fn mb_tuple_new_2(a: MbValue, b: MbValue) -> MbValue {
    retain_ptr_args!(a, b);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b]))
}

pub fn mb_tuple_new_3(a: MbValue, b: MbValue, c: MbValue) -> MbValue {
    retain_ptr_args!(a, b, c);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c]))
}

pub fn mb_tuple_new_4(a: MbValue, b: MbValue, c: MbValue, d: MbValue) -> MbValue {
    retain_ptr_args!(a, b, c, d);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c, d]))
}

pub fn mb_tuple_new_5(a: MbValue, b: MbValue, c: MbValue, d: MbValue, e: MbValue) -> MbValue {
    retain_ptr_args!(a, b, c, d, e);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c, d, e]))
}

pub fn mb_tuple_new_6(
    a: MbValue,
    b: MbValue,
    c: MbValue,
    d: MbValue,
    e: MbValue,
    f: MbValue,
) -> MbValue {
    retain_ptr_args!(a, b, c, d, e, f);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c, d, e, f]))
}

pub fn mb_tuple_new_7(
    a: MbValue,
    b: MbValue,
    c: MbValue,
    d: MbValue,
    e: MbValue,
    f: MbValue,
    g: MbValue,
) -> MbValue {
    retain_ptr_args!(a, b, c, d, e, f, g);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c, d, e, f, g]))
}

pub fn mb_tuple_new_8(
    a: MbValue,
    b: MbValue,
    c: MbValue,
    d: MbValue,
    e: MbValue,
    f: MbValue,
    g: MbValue,
    h: MbValue,
) -> MbValue {
    retain_ptr_args!(a, b, c, d, e, f, g, h);
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b, c, d, e, f, g, h]))
}

/// tuple(iterable) — create a tuple from a list, string, range, or any iterable.
pub fn mb_tuple_from_iterable(iterable: MbValue) -> MbValue {
    if iterable.is_int() {
        // Wrap raw generator handles so the exhaustion path runs through
        // `advance_generator_if_applicable` and the trailing StopIteration
        // gets cleared from the runtime exception slot. Idempotent for
        // handles already in ITERATORS. See `mb_list_from_iterable` for
        // the matching list-side fix.
        let iter_handle = super::iter::mb_iter(iterable);
        let mut items = Vec::new();
        loop {
            if super::iter::mb_has_next(iter_handle).as_bool() != Some(true) {
                break;
            }
            // mb_has_next pre-peeks, so mb_next is guaranteed to return the
            // real next value — including legitimate `None` yields.
            items.push(super::iter::mb_next(iter_handle));
        }
        return MbValue::from_ptr(MbObject::new_tuple(items));
    }
    if let Some(ptr) = iterable.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap().to_vec();
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items));
                }
                ObjData::Tuple(items) => {
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items.clone()));
                }
                ObjData::Str(s) => {
                    let items: Vec<_> = s
                        .chars()
                        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_tuple(items));
                }
                ObjData::Bytes(ref data) => {
                    let items: Vec<MbValue> =
                        data.iter().map(|&b| MbValue::from_int(b as i64)).collect();
                    return MbValue::from_ptr(MbObject::new_tuple(items));
                }
                ObjData::ByteArray(ref lock) => {
                    let items: Vec<MbValue> = lock
                        .read()
                        .unwrap()
                        .iter()
                        .map(|&b| MbValue::from_int(b as i64))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_tuple(items));
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap().to_vec();
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items));
                }
                ObjData::FrozenSet(items) => {
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items.clone()));
                }
                ObjData::Dict(ref lock) => {
                    let keys: Vec<MbValue> = lock
                        .read()
                        .unwrap()
                        .keys()
                        .map(super::dict_ops::dict_key_to_mbvalue)
                        .collect();
                    return MbValue::from_ptr(MbObject::new_tuple(keys));
                }
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                    ..
                } if class_name == "collections.deque" => {
                    let guard = fields.read().unwrap();
                    if let Some(items) = guard.get("_items") {
                        if let Some(ip) = items.as_ptr() {
                            if let ObjData::List(ref lock) = (*ip).data {
                                return MbValue::from_ptr(MbObject::new_tuple_borrowed(
                                    lock.read().unwrap().to_vec(),
                                ));
                            }
                        }
                    }
                    return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
                }
                ObjData::Instance { .. } => {
                    let iter_handle = super::iter::mb_iter(iterable);
                    if iter_handle.is_none() {
                        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
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
                    return MbValue::from_ptr(MbObject::new_tuple(items));
                }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

/// Convert a list MbValue to a tuple MbValue (for codegen MakeTuple).
pub fn mb_list_to_tuple(list: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = list.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                // Items borrowed from the source list — retain.
                return MbValue::from_ptr(MbObject::new_tuple_borrowed(items.to_vec()));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
}

/// Materialize a packed `*args` value as the tuple Python guarantees, run once
/// at the entry of every variadic function. Call paths pack the extra
/// positional args inconsistently — most build a list, but some (e.g.
/// atexit-forwarded calls) already pass a tuple — so this is idempotent: a list
/// is converted, an existing tuple is copied (preserving its elements rather
/// than collapsing to empty like `mb_list_to_tuple` would), and any other value
/// passes through retained. Always yields a fresh owned value the body releases.
pub fn mb_star_args_to_tuple(val: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = val.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items.to_vec()));
                }
                ObjData::Tuple(ref items) => {
                    return MbValue::from_ptr(MbObject::new_tuple_borrowed(items.clone()));
                }
                _ => {}
            }
        }
    }
    unsafe {
        super::rc::retain_if_ptr(val);
    }
    val
}

// ── Access ──

/// tuple[index] → value
pub fn mb_tuple_getitem(tup: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let (Some(items), Some(idx)) = (as_tuple(tup), index.as_int()) {
            let len = items.len() as i64;
            let actual = if idx < 0 { idx + len } else { idx };
            if actual >= 0 && actual < len {
                let val = items[actual as usize];
                super::rc::retain_if_ptr(val);
                val
            } else {
                super::exception::mb_raise(
                    MbValue::from_ptr(super::rc::MbObject::new_str("IndexError".to_string())),
                    MbValue::from_ptr(super::rc::MbObject::new_str(
                        "tuple index out of range".to_string(),
                    )),
                );
                MbValue::none()
            }
        } else {
            MbValue::none()
        }
    }
}

/// tuple[start:stop] → new tuple
pub fn mb_tuple_slice(tup: MbValue, start: MbValue, stop: MbValue) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            let len = items.len() as i64;
            let s = normalize_index(start.as_int().unwrap_or(0), len) as usize;
            let e = normalize_index(stop.as_int().unwrap_or(len), len) as usize;
            let sliced = if s <= e {
                items[s..e].to_vec()
            } else {
                Vec::new()
            };
            MbValue::from_ptr(MbObject::new_tuple(sliced))
        } else {
            mb_tuple_new()
        }
    }
}

/// tuple[start:stop:step] → new tuple (step support)
pub fn mb_tuple_slice_full(tup: MbValue, start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            let len = items.len() as i64;
            let st = step.as_int().unwrap_or(1);
            if st == 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "slice step cannot be zero".to_string(),
                    )),
                );
                return mb_tuple_new();
            }
            let (s, e) = if st > 0 {
                let s = start.as_int().map(|i| clamp_index(i, len)).unwrap_or(0);
                let e = stop.as_int().map(|i| clamp_index(i, len)).unwrap_or(len);
                (s, e)
            } else {
                let s = start.as_int().map(|i| clamp_rev(i, len)).unwrap_or(len - 1);
                let e = stop.as_int().map(|i| clamp_rev(i, len)).unwrap_or(-1);
                (s, e)
            };
            let mut result = Vec::new();
            let mut i = s;
            if st > 0 {
                while i < e {
                    if i >= 0 && i < len {
                        result.push(items[i as usize]);
                    }
                    i += st;
                }
            } else {
                while i > e {
                    if i >= 0 && i < len {
                        result.push(items[i as usize]);
                    }
                    i += st;
                }
            }
            MbValue::from_ptr(MbObject::new_tuple(result))
        } else {
            mb_tuple_new()
        }
    }
}

fn clamp_index(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(0).min(len)
}

fn clamp_rev(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(-1).min(len - 1)
}

// ── Query Methods ──

/// len(tuple) → int
pub fn mb_tuple_len(tup: MbValue) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            MbValue::from_int(items.len() as i64)
        } else {
            MbValue::from_int(0)
        }
    }
}

/// value in tuple → bool. Uses Python-semantic equality (deep, via mb_eq)
/// so `"a" in ("a",)` and `True in (1,)` work despite pointer identity differing.
pub fn mb_tuple_contains(tup: MbValue, value: MbValue) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            for item in items.iter() {
                if super::builtins::mb_eq(value, *item).as_bool() == Some(true) {
                    return MbValue::from_bool(true);
                }
            }
            MbValue::from_bool(false)
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// tuple.count(value) → int. Uses Python-semantic equality.
pub fn mb_tuple_count(tup: MbValue, value: MbValue) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            let n = items
                .iter()
                .filter(|v| super::builtins::mb_eq(**v, value).as_bool() == Some(true))
                .count();
            MbValue::from_int(n as i64)
        } else {
            MbValue::from_int(0)
        }
    }
}

/// tuple.index(value) → int. Uses Python-semantic equality.
pub fn mb_tuple_index(tup: MbValue, value: MbValue) -> MbValue {
    mb_tuple_index_range(tup, value, MbValue::none(), MbValue::none())
}

/// `tuple.index(value, start=0, stop=len)` — same shape as `list.index`.
/// Returns the absolute index of the first match within [start, stop). The
/// existing `-1` not-found sentinel is preserved (changing it to raise
/// ValueError is a separate fix — see `tuple_ops.rs:566` test expectation).
pub fn mb_tuple_index_range(
    tup: MbValue,
    value: MbValue,
    start: MbValue,
    stop: MbValue,
) -> MbValue {
    unsafe {
        if let Some(items) = as_tuple(tup) {
            let len = items.len() as i64;
            let mut s = start.as_int().unwrap_or(0);
            if s < 0 {
                s = (s + len).max(0);
            } else if s > len {
                s = len;
            }
            let mut e = stop.as_int().unwrap_or(len);
            if e < 0 {
                e = (e + len).max(0);
            } else if e > len {
                e = len;
            }
            if s < e {
                for i in (s as usize)..(e as usize) {
                    if super::builtins::mb_eq(items[i], value).as_bool() == Some(true) {
                        return MbValue::from_int(i as i64);
                    }
                }
            }
            // CPython: a missing value raises ValueError, never returns -1.
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "tuple.index(x): x not in tuple".to_string(),
                )),
            );
            MbValue::none()
        } else {
            MbValue::from_int(-1)
        }
    }
}

// ── Operators ──

/// tuple + tuple → new concatenated tuple
pub fn mb_tuple_concat(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(ta), Some(tb)) => {
                let mut result = ta.clone();
                result.extend_from_slice(tb);
                MbValue::from_ptr(MbObject::new_tuple(result))
            }
            _ => MbValue::none(),
        }
    }
}

/// tuple * n → repeated tuple
pub fn mb_tuple_repeat(tup: MbValue, n: MbValue) -> MbValue {
    unsafe {
        if let (Some(items), Some(count)) = (as_tuple(tup), n.as_int()) {
            if count <= 0 {
                return mb_tuple_new();
            }
            let mut result = Vec::with_capacity(items.len() * count as usize);
            for _ in 0..count {
                result.extend_from_slice(items);
            }
            MbValue::from_ptr(MbObject::new_tuple(result))
        } else {
            MbValue::none()
        }
    }
}

/// tuple == tuple → bool
pub fn mb_tuple_eq(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(_), Some(_)) => super::builtins::mb_eq(a, b),
            _ => MbValue::from_bool(false),
        }
    }
}

/// Hash a tuple (only works if all elements are hashable).
pub fn mb_tuple_hash(tup: MbValue) -> MbValue {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    unsafe {
        if let Some(items) = as_tuple(tup) {
            let mut hasher = DefaultHasher::new();
            items.len().hash(&mut hasher);
            for item in items {
                // Content-based element hash via mb_hash, which agrees with
                // mb_eq: equal values hash equal (ints by value, strings by
                // content, nested tuples recursively). Hashing raw to_bits()
                // here made `hash((1, "a")) != hash((1, "a"))` for two "a"
                // allocations — breaking CPython tuple-hash semantics and
                // value-equal tuple dict keys. Kinds without a value hash
                // (lists, plain instances) keep mb_hash's pointer-derived
                // identity hash, preserving the old behavior for them.
                super::builtins::mb_hash(*item)
                    .as_int()
                    .unwrap_or(0)
                    .hash(&mut hasher);
            }
            // Truncate to 47 bits to fit MbValue's 48-bit signed int range
            let h = (hasher.finish() >> 17) as i64;
            MbValue::from_int(h)
        } else {
            MbValue::from_int(0)
        }
    }
}

// ── Ordering Operators (#844) ──

/// Lexicographic less-than: element-by-element, shorter tuple is less.
pub fn mb_tuple_lt(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(ta), Some(tb)) => MbValue::from_bool(super::builtins::seq_lt(ta, tb)),
            _ => MbValue::from_bool(false),
        }
    }
}

/// Lexicographic less-or-equal.
pub fn mb_tuple_le(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(ta), Some(tb)) => {
                // a <= b iff !(b < a)
                MbValue::from_bool(!super::builtins::seq_lt(tb, ta))
            }
            _ => MbValue::from_bool(false),
        }
    }
}

/// Lexicographic greater-than.
pub fn mb_tuple_gt(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(ta), Some(tb)) => MbValue::from_bool(super::builtins::seq_lt(tb, ta)),
            _ => MbValue::from_bool(false),
        }
    }
}

/// Lexicographic greater-or-equal.
pub fn mb_tuple_ge(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_tuple(a), as_tuple(b)) {
            (Some(ta), Some(tb)) => {
                // a >= b iff !(a < b)
                MbValue::from_bool(!super::builtins::seq_lt(ta, tb))
            }
            _ => MbValue::from_bool(false),
        }
    }
}

// ── Method Dispatch ──

/// Dispatch a method call on a tuple object.
pub fn dispatch_tuple_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    return items.get(i).copied().unwrap_or(MbValue::none());
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
        "count" => mb_tuple_count(receiver, arg(0)),
        "index" => {
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let stop = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_tuple_index_range(receiver, arg(0), start, stop)
        }
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'tuple' object has no attribute '{name}'"
                ))),
            );
            MbValue::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::gc::{gc_disable, gc_enable};
    use super::*;

    /// RAII guard: disables GC on creation, re-enables on drop (even on panic).
    /// Prevents concurrent test threads from collecting our MbObjects.
    struct GcGuard;
    impl GcGuard {
        fn new() -> Self {
            gc_disable();
            Self
        }
    }
    impl Drop for GcGuard {
        fn drop(&mut self) {
            gc_enable();
        }
    }

    // ── Creation ──

    #[test]
    fn test_new_empty() {
        let _gc = GcGuard::new();
        let t = mb_tuple_new();
        assert_eq!(mb_tuple_len(t).as_int(), Some(0));
    }

    #[test]
    fn test_from_elements() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_len(t).as_int(), Some(2));
    }

    #[test]
    fn test_list_to_tuple() {
        let _gc = GcGuard::new();
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
        ]));
        let t = mb_list_to_tuple(list);
        assert_eq!(mb_tuple_len(t).as_int(), Some(2));
        assert_eq!(mb_tuple_getitem(t, MbValue::from_int(0)).as_int(), Some(10));
    }

    #[test]
    fn test_list_to_tuple_non_list() {
        let _gc = GcGuard::new();
        let t = mb_list_to_tuple(MbValue::from_int(0));
        assert_eq!(mb_tuple_len(t).as_int(), Some(0));
    }

    // ── getitem ──

    #[test]
    fn test_getitem() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]);
        assert_eq!(mb_tuple_getitem(t, MbValue::from_int(0)).as_int(), Some(10));
        assert_eq!(mb_tuple_getitem(t, MbValue::from_int(2)).as_int(), Some(30));
    }

    #[test]
    fn test_getitem_negative() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
        assert_eq!(
            mb_tuple_getitem(t, MbValue::from_int(-1)).as_int(),
            Some(20)
        );
        assert_eq!(
            mb_tuple_getitem(t, MbValue::from_int(-2)).as_int(),
            Some(10)
        );
    }

    #[test]
    fn test_getitem_out_of_bounds() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1)]);
        assert!(mb_tuple_getitem(t, MbValue::from_int(5)).is_none());
        assert!(mb_tuple_getitem(t, MbValue::from_int(-5)).is_none());
    }

    #[test]
    fn test_getitem_non_tuple() {
        assert!(mb_tuple_getitem(MbValue::from_int(0), MbValue::from_int(0)).is_none());
    }

    // ── slice ──

    #[test]
    fn test_slice() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let s = mb_tuple_slice(t, MbValue::from_int(1), MbValue::from_int(3));
        assert_eq!(mb_tuple_len(s).as_int(), Some(2));
        assert_eq!(mb_tuple_getitem(s, MbValue::from_int(0)).as_int(), Some(1));
    }

    #[test]
    fn test_slice_empty_range() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1)]);
        let s = mb_tuple_slice(t, MbValue::from_int(1), MbValue::from_int(0));
        assert_eq!(mb_tuple_len(s).as_int(), Some(0));
    }

    // ── len / contains ──

    #[test]
    fn test_len_contains() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_len(t).as_int(), Some(2));
        assert_eq!(
            mb_tuple_contains(t, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_tuple_contains(t, MbValue::from_int(3)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_contains_non_tuple() {
        assert_eq!(
            mb_tuple_contains(MbValue::from_int(0), MbValue::from_int(0)).as_bool(),
            Some(false)
        );
    }

    // ── count / index ──

    #[test]
    fn test_count() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(1),
        ]);
        assert_eq!(mb_tuple_count(t, MbValue::from_int(1)).as_int(), Some(2));
        assert_eq!(mb_tuple_count(t, MbValue::from_int(9)).as_int(), Some(0));
    }

    #[test]
    fn test_index() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]);
        assert_eq!(mb_tuple_index(t, MbValue::from_int(20)).as_int(), Some(1));
        // CPython: a missing value raises ValueError; the runtime entry
        // returns None after routing through the exception machinery.
        assert!(mb_tuple_index(t, MbValue::from_int(99)).is_none());
        crate::runtime::exception::mb_clear_exception();
    }

    // ── concat ──

    #[test]
    fn test_concat() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1)]);
        let b = mb_tuple_from(vec![MbValue::from_int(2)]);
        let c = mb_tuple_concat(a, b);
        assert_eq!(mb_tuple_len(c).as_int(), Some(2));
        assert_eq!(mb_tuple_getitem(c, MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_tuple_getitem(c, MbValue::from_int(1)).as_int(), Some(2));
    }

    #[test]
    fn test_concat_non_tuple() {
        assert!(mb_tuple_concat(MbValue::from_int(1), MbValue::from_int(2)).is_none());
    }

    // ── repeat ──

    #[test]
    fn test_repeat() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1)]);
        let r = mb_tuple_repeat(t, MbValue::from_int(3));
        assert_eq!(mb_tuple_len(r).as_int(), Some(3));
    }

    #[test]
    fn test_repeat_zero() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1)]);
        let r = mb_tuple_repeat(t, MbValue::from_int(0));
        assert_eq!(mb_tuple_len(r).as_int(), Some(0));
    }

    #[test]
    fn test_repeat_negative() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1)]);
        let r = mb_tuple_repeat(t, MbValue::from_int(-1));
        assert_eq!(mb_tuple_len(r).as_int(), Some(0));
    }

    // ── eq ──

    #[test]
    fn test_eq() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_different() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1)]);
        let b = mb_tuple_from(vec![MbValue::from_int(2)]);
        assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_non_tuple() {
        assert_eq!(
            mb_tuple_eq(MbValue::from_int(1), MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    // ── hash ──

    #[test]
    fn test_hash_non_tuple() {
        // mb_tuple_hash on non-tuple returns 0
        assert_eq!(mb_tuple_hash(MbValue::from_int(0)).as_int(), Some(0));
    }

    // ── dispatch ──

    #[test]
    fn test_dispatch_count() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(1)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        assert_eq!(dispatch_tuple_method("count", t, args).as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_index() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(10), MbValue::from_int(20)]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(20)]));
        assert_eq!(dispatch_tuple_method("index", t, args).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_unknown() {
        let _gc = GcGuard::new();
        let t = mb_tuple_new();
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert!(dispatch_tuple_method("nonexistent", t, args).is_none());
    }

    // ── normalize_index ──

    #[test]
    fn test_normalize_index() {
        assert_eq!(normalize_index(-1, 5), 4);
        assert_eq!(normalize_index(-10, 5), 0);
        assert_eq!(normalize_index(10, 5), 5);
        assert_eq!(normalize_index(2, 5), 2);
    }

    // -- Py3.12 conformance: ordering --

    #[test]
    fn test_py312_tuple_ordering() {
        let _gc = GcGuard::new();
        let t1 = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let t2 = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(3)]);
        assert_eq!(mb_tuple_lt(t1, t2).as_bool(), Some(true));
        assert_eq!(mb_tuple_lt(t2, t1).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_tuple_prefix_ordering() {
        let _gc = GcGuard::new();
        let short = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let long_t = mb_tuple_from(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(0),
        ]);
        assert_eq!(mb_tuple_lt(short, long_t).as_bool(), Some(true));
    }

    #[test]
    fn test_py312_tuple_le() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_le(a, b).as_bool(), Some(true)); // equal → le true
        let c = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(3)]);
        assert_eq!(mb_tuple_le(a, c).as_bool(), Some(true)); // less → le true
        assert_eq!(mb_tuple_le(c, a).as_bool(), Some(false)); // greater → le false
    }

    #[test]
    fn test_py312_tuple_gt() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(3)]);
        let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_gt(a, b).as_bool(), Some(true));
        assert_eq!(mb_tuple_gt(b, a).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_tuple_ge() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_ge(a, b).as_bool(), Some(true)); // equal → ge true
        let c = mb_tuple_from(vec![MbValue::from_int(2)]);
        assert_eq!(mb_tuple_ge(c, a).as_bool(), Some(true)); // greater → ge true
    }

    #[test]
    fn test_py312_tuple_empty_cmp() {
        let _gc = GcGuard::new();
        let empty = mb_tuple_new();
        let nonempty = mb_tuple_from(vec![MbValue::from_int(1)]);
        assert_eq!(mb_tuple_lt(empty, nonempty).as_bool(), Some(true));
        assert_eq!(mb_tuple_le(empty, empty).as_bool(), Some(true));
        assert_eq!(mb_tuple_ge(empty, empty).as_bool(), Some(true));
        assert_eq!(mb_tuple_gt(empty, nonempty).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_tuple_concat() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_tuple_from(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        let c = mb_tuple_concat(a, b);
        assert_eq!(mb_tuple_len(c).as_int(), Some(4));
        assert_eq!(mb_tuple_getitem(c, MbValue::from_int(0)).as_int(), Some(1));
        assert_eq!(mb_tuple_getitem(c, MbValue::from_int(3)).as_int(), Some(4));
    }

    #[test]
    fn test_py312_tuple_repeat() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let rep = mb_tuple_repeat(t, MbValue::from_int(3));
        assert_eq!(mb_tuple_len(rep).as_int(), Some(6));
    }

    #[test]
    fn test_py312_tuple_contains() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_bool(true)]);
        assert_eq!(
            mb_tuple_contains(t, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_tuple_contains(t, MbValue::from_int(99)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_py312_tuple_eq_same_content() {
        let _gc = GcGuard::new();
        let a = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = mb_tuple_from(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        assert_eq!(mb_tuple_eq(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_py312_tuple_count() {
        let _gc = GcGuard::new();
        let t = mb_tuple_from(vec![
            MbValue::from_int(1),
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]);
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        assert_eq!(dispatch_tuple_method("count", t, args).as_int(), Some(2));
    }

    // TODO: enable when tuple.index(value, start) is implemented
    // #[test]
    // fn test_py312_tuple_index_with_start() {
    //     let t = mb_tuple_from(vec![
    //         MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(1),
    //     ]);
    //     let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(1)]));
    //     assert_eq!(dispatch_tuple_method("index", t, args).as_int(), Some(2));
    // }
}
