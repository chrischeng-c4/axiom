/// collections module for Mamba (#391).
///
/// Provides: Counter, counter.most_common, deque operations, OrderedDict.
/// Backed by existing MbValue list/dict primitives.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// Extract a String from an MbValue that wraps a heap Str.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Dispatch wrappers ──

/// Build a `&[MbValue]` from a (ptr, len) callback pair, treating a
/// null pointer as an empty slice. Required for nullary callsites like
/// `OrderedDict()`: the FFI ABI may pass `args_ptr = NULL` when
/// `nargs == 0`, but `std::slice::from_raw_parts` requires the pointer
/// to be aligned + non-null even for zero-length slices (UB precondition
/// check; aborts under debug builds, undefined behaviour otherwise).
/// Fixes #2212.
#[inline]
unsafe fn args_as_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn extract_list(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

unsafe extern "C" fn dispatch_counter_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_counter_new(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_counter_most_common(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_counter_most_common(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_deque_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    let initial = a.get(0).copied().unwrap_or_else(MbValue::none);
    let maxlen = a.get(1).and_then(|v| v.as_int());
    // Collect initial data
    let mut data: Vec<MbValue> = Vec::new();
    if let Some(ptr) = initial.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                data = lock.read().unwrap().to_vec();
            }
        }
    }
    // Apply maxlen: keep only the last `maxlen` elements
    if let Some(ml) = maxlen {
        let ml = ml as usize;
        if data.len() > ml {
            data = data[data.len() - ml..].to_vec();
        }
    }
    // Build Instance
    let items_list = MbValue::from_ptr(MbObject::new_list(data));
    let mut fields = FxHashMap::default();
    fields.insert("_items".into(), items_list);
    fields.insert("_maxlen".into(), maxlen.map(MbValue::from_int).unwrap_or(MbValue::none()));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.deque".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

unsafe extern "C" fn dispatch_deque_appendleft(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_deque_appendleft(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_deque_popleft(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_deque_popleft(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_deque_append(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_deque_append(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_deque_pop(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_deque_pop(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_deque_rotate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_deque_rotate(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_ordereddict_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_ordereddict_new(a)
}

unsafe extern "C" fn dispatch_defaultdict_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_defaultdict_new(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_namedtuple(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_namedtuple(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_userdict_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_userdict_new(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_userlist_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_userlist_new(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_userstring_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    mb_userstring_new(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_chainmap_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // ChainMap takes variadic dict arguments — pack into a list.
    let a: &[MbValue] = unsafe { args_as_slice(args_ptr, nargs) };
    let args_list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
    mb_chainmap_new(args_list)
}

/// Register the collections module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: [(&str, usize); 15] = [
        ("Counter", dispatch_counter_new as *const () as usize),
        ("counter_most_common", dispatch_counter_most_common as *const () as usize),
        ("deque", dispatch_deque_new as *const () as usize),
        ("deque_appendleft", dispatch_deque_appendleft as *const () as usize),
        ("deque_popleft", dispatch_deque_popleft as *const () as usize),
        ("deque_append", dispatch_deque_append as *const () as usize),
        ("deque_pop", dispatch_deque_pop as *const () as usize),
        ("deque_rotate", dispatch_deque_rotate as *const () as usize),
        ("OrderedDict", dispatch_ordereddict_new as *const () as usize),
        ("defaultdict", dispatch_defaultdict_new as *const () as usize),
        ("namedtuple", dispatch_namedtuple as *const () as usize),
        ("ChainMap", dispatch_chainmap_new as *const () as usize),
        ("UserDict", dispatch_userdict_new as *const () as usize),
        ("UserList", dispatch_userlist_new as *const () as usize),
        ("UserString", dispatch_userstring_new as *const () as usize),
    ];
    const COLLECTIONS_TYPES: &[&str] = &[
        "Counter", "deque", "OrderedDict", "defaultdict",
        "ChainMap", "UserDict", "UserList", "UserString",
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        if COLLECTIONS_TYPES.contains(&name) {
            // Register the FULL-DOTTED name so it matches the class_name these
            // dispatchers actually stamp on their instances (e.g. deque() makes
            // a "collections.deque" Instance) — keeps object.__new__ / isinstance
            // / type() consistent.
            let dotted = format!("collections.{name}");
            super::super::module::NATIVE_TYPE_NAMES.with(|m| {
                m.borrow_mut().insert(addr as u64, dotted);
            });
        }
    }

    super::register_module("collections", attrs);
}

// ── Runtime functions ──

/// Convert an MbValue to a string key for counting purposes.
#[allow(dead_code)]
fn value_to_key(val: MbValue) -> String {
    if val.is_none() {
        "None".to_string()
    } else if let Some(i) = val.as_int() {
        format!("int:{i}")
    } else if let Some(f) = val.as_float() {
        format!("float:{f}")
    } else if let Some(b) = val.as_bool() {
        format!("bool:{b}")
    } else if let Some(s) = extract_str(val) {
        format!("str:{s}")
    } else {
        format!("obj:{:?}", val)
    }
}

/// collections.Counter(iterable) -> collections.Counter Instance
///
/// Counts elements of an iterable (list or string) and stores the result
/// as a `collections.Counter` Instance. Method dispatch (`.most_common(n)`)
/// is handled in `runtime::class::mb_call_method`.
/// Raise a TypeError and return None (native raise convention).
fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise a ValueError and return None (native raise convention).
fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

pub fn mb_counter_new(iterable: MbValue) -> MbValue {
    use crate::runtime::dict_ops::DictKey;
    let mut counts: indexmap::IndexMap<DictKey, i64> = indexmap::IndexMap::new();

    // CPython: Counter(non-iterable scalar) raises TypeError.
    if iterable.as_int().is_some() || iterable.as_float().is_some() || iterable.is_bool() {
        return raise_type_error("'int' object is not iterable");
    }
    // CPython: unhashable elements (lists/dicts/sets) raise TypeError.
    if let Some(ptr) = iterable.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().iter() {
                    if let Some(ip) = item.as_ptr() {
                        let unhashable = matches!(
                            (*ip).data,
                            ObjData::List(_) | ObjData::Dict(_) | ObjData::Set(_)
                        );
                        if unhashable {
                            return raise_type_error("unhashable type: 'list'");
                        }
                    }
                }
            }
        }
    }

    if let Some(ptr) = iterable.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    for item in lock.read().unwrap().iter() {
                        if let Some(i) = item.as_int() {
                            *counts.entry(DictKey::Int(i)).or_insert(0) += 1;
                        } else if let Some(s) = extract_str(*item) {
                            *counts.entry(DictKey::Str(s)).or_insert(0) += 1;
                        } else if let Some(b) = item.as_bool() {
                            *counts.entry(DictKey::Bool(b)).or_insert(0) += 1;
                        }
                    }
                }
                ObjData::Tuple(ref items) => {
                    for item in items.iter() {
                        if let Some(i) = item.as_int() {
                            *counts.entry(DictKey::Int(i)).or_insert(0) += 1;
                        } else if let Some(s) = extract_str(*item) {
                            *counts.entry(DictKey::Str(s)).or_insert(0) += 1;
                        } else if let Some(b) = item.as_bool() {
                            *counts.entry(DictKey::Bool(b)).or_insert(0) += 1;
                        }
                    }
                }
                ObjData::Str(ref s) => {
                    for ch in s.chars() {
                        *counts.entry(DictKey::Str(ch.to_string())).or_insert(0) += 1;
                    }
                }
                _ => {}
            }
        }
    }

    // Build dict MbValue for the internal data.
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (key, count) in &counts {
                map.insert(key.clone().into(), MbValue::from_int(*count));
            }
        }
    }
    let dict_val = MbValue::from_ptr(dict);

    let mut fields = FxHashMap::default();
    fields.insert("_data".into(), dict_val);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.Counter".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Extract the internal IndexMap from a Counter (Instance or dict).
fn counter_data(counter: MbValue) -> indexmap::IndexMap<crate::runtime::dict_ops::DictKey, MbValue> {
    if let Some(ptr) = counter.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { ref fields, .. } => {
                    let f = fields.read().unwrap();
                    if let Some(data) = f.get("_data") {
                        if let Some(dp) = data.as_ptr() {
                            if let ObjData::Dict(ref lock) = (*dp).data {
                                return lock.read().unwrap().clone();
                            }
                        }
                    }
                }
                ObjData::Dict(ref lock) => return lock.read().unwrap().clone(),
                _ => {}
            }
        }
    }
    indexmap::IndexMap::new()
}

/// collections.Counter.most_common(n) -> list of (key, count) tuples
///
/// Returns the n most common elements as a list of tuples (Python semantics).
pub fn mb_counter_most_common(
    counter: MbValue,
    n: MbValue,
) -> MbValue {
    use crate::runtime::dict_ops::DictKey;
    let map = counter_data(counter);
    let limit = n.as_int().unwrap_or(map.len() as i64) as usize;

    let mut entries: Vec<(DictKey, i64)> = map
        .iter()
        .map(|(k, v)| (k.clone(), v.as_int().unwrap_or(0)))
        .collect();
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    let results: Vec<MbValue> = entries
        .into_iter()
        .take(limit)
        .map(|(key, count)| {
            let key_val = super::super::dict_ops::dict_key_to_mbvalue(&key);
            MbValue::from_ptr(MbObject::new_tuple(vec![
                key_val,
                MbValue::from_int(count),
            ]))
        })
        .collect();

    MbValue::from_ptr(MbObject::new_list(results))
}

/// Build a `collections.Counter` Instance from an `(DictKey, i64)` count map,
/// dropping entries whose count is `<= 0` (CPython multiset semantics for
/// `+`/`-`/`&`/`|`).
fn build_counter_from_counts(
    counts: indexmap::IndexMap<crate::runtime::dict_ops::DictKey, i64>,
) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (key, count) in counts {
                if count > 0 {
                    map.insert(key.into(), MbValue::from_int(count));
                }
            }
        }
    }
    let dict_val = MbValue::from_ptr(dict);

    let mut fields = FxHashMap::default();
    fields.insert("_data".into(), dict_val);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.Counter".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Extract `(DictKey, i64)` pairs from a Counter, preserving insertion order.
fn counter_int_counts(
    counter: MbValue,
) -> indexmap::IndexMap<crate::runtime::dict_ops::DictKey, i64> {
    counter_data(counter)
        .into_iter()
        .map(|(k, v)| (k, v.as_int().unwrap_or(0)))
        .collect()
}

/// Counter + Counter — CPython multiset semantics: per-key sum, drop <= 0.
pub fn mb_counter_add(a: MbValue, b: MbValue) -> MbValue {
    let mut out = counter_int_counts(a);
    for (k, v) in counter_int_counts(b) {
        *out.entry(k).or_insert(0) += v;
    }
    build_counter_from_counts(out)
}

/// Counter - Counter — CPython multiset semantics: per-key diff, drop <= 0.
pub fn mb_counter_sub(a: MbValue, b: MbValue) -> MbValue {
    let mut out = counter_int_counts(a);
    for (k, v) in counter_int_counts(b) {
        *out.entry(k).or_insert(0) -= v;
    }
    build_counter_from_counts(out)
}

/// Counter & Counter — CPython multiset semantics: per-key min, drop <= 0.
/// Result keys = intersection of the two key sets.
pub fn mb_counter_and(a: MbValue, b: MbValue) -> MbValue {
    let lhs = counter_int_counts(a);
    let rhs = counter_int_counts(b);
    let mut out = indexmap::IndexMap::new();
    for (k, va) in &lhs {
        if let Some(vb) = rhs.get(k) {
            out.insert(k.clone(), (*va).min(*vb));
        }
    }
    build_counter_from_counts(out)
}

/// Counter | Counter — CPython multiset semantics: per-key max, drop <= 0.
pub fn mb_counter_or(a: MbValue, b: MbValue) -> MbValue {
    let mut out = counter_int_counts(a);
    for (k, vb) in counter_int_counts(b) {
        out.entry(k)
            .and_modify(|va| *va = (*va).max(vb))
            .or_insert(vb);
    }
    build_counter_from_counts(out)
}

/// CPython-compatible `repr(Counter(...))`:
/// `Counter({'a': 3, 'b': 1})` — sorted by count descending, ties broken by
/// insertion order. Empty: `Counter()`. Returns the bare unqualified
/// `Counter` (not `collections.Counter`) to match CPython's
/// `Counter.__repr__`.
pub fn counter_repr(counter: MbValue) -> String {
    use crate::runtime::dict_ops::DictKey;
    let map = counter_data(counter);
    if map.is_empty() {
        return "Counter()".to_string();
    }
    let mut entries: Vec<(DictKey, MbValue, usize)> = map
        .iter()
        .enumerate()
        .map(|(i, (k, v))| (k.clone(), *v, i))
        .collect();
    // Sort by count descending; ties keep insertion order.
    entries.sort_by(|a, b| {
        let ai = a.1.as_int().unwrap_or(0);
        let bi = b.1.as_int().unwrap_or(0);
        bi.cmp(&ai).then(a.2.cmp(&b.2))
    });
    let parts: Vec<String> = entries
        .into_iter()
        .map(|(k, v, _)| {
            format!(
                "{}: {}",
                super::super::dict_ops::dict_key_display(&k),
                super::super::string_ops::value_to_string(v)
            )
        })
        .collect();
    format!("Counter({{{}}})", parts.join(", "))
}

/// CPython-compatible `repr(defaultdict(...))`:
/// `defaultdict(<class 'list'>, {'x': [1]})`. Empty data: `defaultdict(<factory>, {})`.
/// Renders the `_factory` field: bare-string factories ("list", "int", …) become
/// `<class 'X'>`; everything else delegates to `mb_repr`.
pub fn defaultdict_repr(dd: MbValue) -> String {
    let (factory_str, data_str) = unsafe {
        let mut factory_str = "None".to_string();
        let mut data_str = "{}".to_string();
        if let Some(ptr) = dd.as_ptr() {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(factory) = f.get("_factory").copied() {
                    factory_str = if let Some(fp) = factory.as_ptr() {
                        if let ObjData::Str(ref s) = (*fp).data {
                            format!("<class '{}'>", s)
                        } else {
                            let r = super::super::builtins::mb_repr(factory);
                            r.as_ptr()
                                .and_then(|p| {
                                    if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                                })
                                .unwrap_or_else(|| "None".to_string())
                        }
                    } else if factory.is_none() {
                        "None".to_string()
                    } else {
                        let r = super::super::builtins::mb_repr(factory);
                        r.as_ptr()
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                            })
                            .unwrap_or_else(|| "None".to_string())
                    };
                }
                if let Some(data) = f.get("_data").copied() {
                    data_str = super::super::string_ops::value_to_string(data);
                }
            }
        }
        (factory_str, data_str)
    };
    format!("defaultdict({}, {})", factory_str, data_str)
}

/// CPython-compatible `repr(deque(...))`:
/// `deque([1, 2, 3])`. Empty: `deque([])`. Honors `_maxlen` if set:
/// `deque([1, 2, 3], maxlen=N)`.
pub fn deque_repr(dq: MbValue) -> String {
    let (items_str, maxlen) = unsafe {
        let mut items_str = "[]".to_string();
        let mut maxlen: Option<i64> = None;
        if let Some(ptr) = dq.as_ptr() {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(items) = f.get("_items").copied() {
                    items_str = super::super::string_ops::value_to_string(items);
                }
                if let Some(ml) = f.get("_maxlen").copied() {
                    maxlen = ml.as_int();
                }
            }
        }
        (items_str, maxlen)
    };
    match maxlen {
        Some(n) => format!("deque({}, maxlen={})", items_str, n),
        None => format!("deque({})", items_str),
    }
}

/// CPython-style `repr(namedtuple_instance)`: `Point(x=1, y=2)`.
/// Returns `Some` if `v` is a namedtuple Instance (carries `_namedtuple_fields`),
/// else `None` so callers can fall through.
pub fn namedtuple_repr(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        let ObjData::Instance { ref class_name, ref fields, .. } = (*ptr).data else {
            return None;
        };
        let f = fields.read().ok()?;
        let nt_fields = f.get("_namedtuple_fields")?;
        let nt_name = f.get("_namedtuple_name")
            .and_then(|v| v.as_ptr())
            .and_then(|p| if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None })
            .unwrap_or_else(|| class_name.clone());
        let field_names: Vec<String> = nt_fields.as_ptr()
            .and_then(|p| if let ObjData::List(ref lk) = (*p).data {
                Some(lk.read().unwrap().iter()
                    .filter_map(|fv| fv.as_ptr().and_then(|pp| {
                        if let ObjData::Str(ref s) = (*pp).data { Some(s.clone()) } else { None }
                    })).collect())
            } else { None })
            .unwrap_or_default();
        let mut parts = Vec::with_capacity(field_names.len());
        for fname in &field_names {
            let val = f.get(fname).copied().unwrap_or(MbValue::none());
            let r = super::super::builtins::mb_repr(val);
            let rs = r.as_ptr()
                .and_then(|p| if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None })
                .unwrap_or_else(|| "None".to_string());
            parts.push(format!("{fname}={rs}"));
        }
        Some(format!("{nt_name}({})", parts.join(", ")))
    }
}

/// If `v` is a namedtuple Instance, return its values in declared field order.
/// Used by `mb_len`, `mb_obj_subscript`, and `mb_iter` to give namedtuple
/// instances the same `len/iter/__getitem__` surface as a plain tuple, while
/// preserving the dotted attribute access provided by their Instance fields.
pub fn namedtuple_values(v: MbValue) -> Option<Vec<MbValue>> {
    let ptr = v.as_ptr()?;
    unsafe {
        let ObjData::Instance { ref fields, .. } = (*ptr).data else {
            return None;
        };
        let f = fields.read().ok()?;
        let nt_fields = f.get("_namedtuple_fields")?;
        let names: Vec<String> = nt_fields.as_ptr()
            .and_then(|p| if let ObjData::List(ref lk) = (*p).data {
                Some(lk.read().unwrap().iter()
                    .filter_map(|fv| fv.as_ptr().and_then(|pp| {
                        if let ObjData::Str(ref s) = (*pp).data { Some(s.clone()) } else { None }
                    })).collect())
            } else { None })?;
        Some(names.iter()
            .map(|n| f.get(n).copied().unwrap_or(MbValue::none()))
            .collect())
    }
}

/// True iff `v` is a `collections.Counter` Instance.
pub fn is_counter_instance(v: MbValue) -> bool {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return class_name == "collections.Counter";
            }
        }
    }
    false
}

/// collections.deque() -> empty list (deque backed by list)
pub fn mb_deque_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

// ── UserDict / UserList / UserString stubs (#1265 Task #76, Wave-7 ship #3) ──
//
// **Carve-out**: CPython's `UserDict` / `UserList` / `UserString` are
// thin Python-level wrapper classes that expose a `.data` attribute and
// let users override methods (the original use-case from before dict /
// list could be subclassed directly). Modern code typically subclasses
// `collections.abc.MutableMapping` or uses `__missing__` on dict, so
// the wrapper-class subclassing surface is rarely needed in practice.
//
// These stubs collapse the wrapper-class surface to its underlying
// primitive — UserDict()→dict, UserList()→list, UserString(s)→str. The
// `.data` attribute and method-override surface are NOT yet wired
// (they require attribute-on-builtin-subclass plumbing that the rest
// of mamba doesn't yet need). Code that *constructs* these types will
// work; code that *subclasses* and *overrides* won't.
//
// Tracked under #1449 conformance.

/// collections.UserDict(initialdata=None) -> dict
pub fn mb_userdict_new(initialdata: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    if let Some(ptr) = initialdata.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref src_lock) = (*ptr).data {
                if let ObjData::Dict(ref dst_lock) = (*dict).data {
                    let src = src_lock.read().unwrap();
                    let mut dst = dst_lock.write().unwrap();
                    for (k, v) in src.iter() {
                        dst.insert(k.clone(), *v);
                    }
                }
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// collections.UserList(initlist=None) -> list
pub fn mb_userlist_new(initlist: MbValue) -> MbValue {
    let mut data: Vec<MbValue> = Vec::new();
    if let Some(ptr) = initlist.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                data = lock.read().unwrap().to_vec();
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(data))
}

/// collections.UserString(seq) -> str(seq)
///
/// Accepts a str directly (returns a clone) or an int (returns its
/// decimal string form, matching CPython's `str(int)` coercion). For
/// any other type, returns an empty string — code reaching the
/// non-str / non-int path on UserString construction is rare enough
/// that a stub is acceptable until full str() coercion ships.
pub fn mb_userstring_new(seq: MbValue) -> MbValue {
    if let Some(s) = extract_str(seq) {
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    if let Some(n) = seq.as_int() {
        return MbValue::from_ptr(MbObject::new_str(n.to_string()));
    }
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

/// collections.deque_appendleft(deque, val) -> None
///
/// Inserts `val` at index 0 of the deque list.
pub fn mb_deque_appendleft(deque: MbValue, val: MbValue) -> MbValue {
    if let Some(ptr) = deque.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                items.insert(0, val);
            }
        }
    }
    MbValue::none()
}

/// collections.deque_popleft(deque) -> removed first element, or None
pub fn mb_deque_popleft(deque: MbValue) -> MbValue {
    if let Some(ptr) = deque.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if !items.is_empty() {
                    return items.remove(0);
                }
                drop(items);
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("pop from an empty deque".to_string())),
                );
            }
        }
    }
    MbValue::none()
}

/// collections.deque_append(deque, val) -> None
///
/// Pushes `val` to the end (right side) of the deque list.
pub fn mb_deque_append(deque: MbValue, val: MbValue) -> MbValue {
    if let Some(ptr) = deque.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                items.push(val);
            }
        }
    }
    MbValue::none()
}

/// collections.deque_pop(deque) -> removed last element, or None
pub fn mb_deque_pop(deque: MbValue) -> MbValue {
    if let Some(ptr) = deque.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if !items.is_empty() {
                    return items.pop().unwrap_or_else(MbValue::none);
                }
                drop(items);
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("pop from an empty deque".to_string())),
                );
            }
        }
    }
    MbValue::none()
}

/// collections.deque_rotate(deque, n) -> None
///
/// Rotates the deque n steps to the right. Positive n moves elements from
/// the right end to the left; negative n moves elements from the left end
/// to the right.
pub fn mb_deque_rotate(deque: MbValue, n: MbValue) -> MbValue {
    let steps = n.as_int().unwrap_or(1);
    if let Some(ptr) = deque.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if items.is_empty() {
                    return MbValue::none();
                }
                if steps > 0 {
                    for _ in 0..steps {
                        if let Some(last) = items.pop() {
                            items.insert(0, last);
                        }
                    }
                } else if steps < 0 {
                    for _ in 0..steps.abs() {
                        if !items.is_empty() {
                            let first = items.remove(0);
                            items.push(first);
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// collections.OrderedDict([iterable], **kwargs) -> OrderedDict Instance.
///
/// Returns a `collections.OrderedDict` Instance with a backing dict under
/// `_data`. The dict-like fast paths in `class::unwrap_dictlike_data` already
/// recognise this class, so reads/writes/iteration go through the standard
/// dict machinery. The Instance wrapper exists so repr can render
/// `OrderedDict({...})` instead of falling through to plain dict repr.
pub fn mb_ordereddict_new(args: &[MbValue]) -> MbValue {
    // Build the backing dict. CPython accepts a mapping, an iterable of pairs,
    // or no arg; kwargs lower into a trailing dict positional in mamba's call
    // convention.
    let backing = super::super::dict_ops::mb_dict_new();
    for arg in args {
        if arg.is_none() { continue; }
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Dict(_) => {
                        super::super::dict_ops::mb_dict_update(backing, *arg);
                    }
                    ObjData::List(_) | ObjData::Tuple(_) => {
                        // Iterable of (k, v) pairs.
                        let pairs_dict = super::super::dict_ops::mb_dict_from_pairs(*arg);
                        super::super::dict_ops::mb_dict_update(backing, pairs_dict);
                    }
                    ObjData::Instance { .. } => {
                        // Other dict-like Instances flow through update().
                        super::super::dict_ops::mb_dict_update(backing, *arg);
                    }
                    _ => {}
                }
            }
        }
    }
    let mut fields = FxHashMap::default();
    fields.insert("_data".into(), backing);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.OrderedDict".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// `OrderedDict.move_to_end(key, last=True)` — move an existing `key` to the
/// end (`last=True`) or the front (`last=False`) of the backing order. Raises
/// KeyError if `key` is absent. `data` is the backing `_data` dict.
pub fn mb_ordereddict_move_to_end(data: MbValue, key: MbValue, last: bool) -> MbValue {
    let dk = super::super::dict_ops::to_dict_key(key);
    unsafe {
        if let Some(ptr) = data.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut guard = lock.write().unwrap();
                if let Some(idx) = guard.get_index_of(&dk) {
                    let target = if last { guard.len() - 1 } else { 0 };
                    guard.move_index(idx, target);
                    return MbValue::none();
                }
                drop(guard);
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        super::super::dict_ops::dict_key_raw_str(&dk))),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// `OrderedDict.popitem(last=True)` — remove and return a `(key, value)` pair,
/// LIFO when `last=True`, FIFO when `last=False`. Raises KeyError when empty.
pub fn mb_ordereddict_popitem(data: MbValue, last: bool) -> MbValue {
    unsafe {
        if let Some(ptr) = data.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut guard = lock.write().unwrap();
                if guard.is_empty() {
                    drop(guard);
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("'dictionary is empty'".to_string())),
                    );
                    return MbValue::none();
                }
                let kv = if last {
                    guard.pop()
                } else {
                    guard.shift_remove_index(0)
                };
                drop(guard);
                if let Some((k, v)) = kv {
                    let key_val = super::super::dict_ops::dict_key_to_mbvalue(&k);
                    return MbValue::from_ptr(MbObject::new_tuple(vec![key_val, v]));
                }
            }
        }
    }
    MbValue::none()
}

/// CPython-compatible `repr(OrderedDict(...))`:
/// `OrderedDict({'a': 1, 'b': 2})`. Empty: `OrderedDict()`.
pub fn ordereddict_repr(od: MbValue) -> String {
    let backing = super::super::class::unwrap_dictlike_data(od);
    let inner_str = backing
        .map(super::super::string_ops::value_to_string)
        .unwrap_or_else(|| "{}".to_string());
    if inner_str == "{}" {
        "OrderedDict()".to_string()
    } else {
        format!("OrderedDict({})", inner_str)
    }
}

/// collections.defaultdict(default_factory) -> defaultdict Instance
///
/// Returns a `collections.defaultdict` Instance with an internal dict `_data`
/// and a `_factory` callable. `mb_obj_getitem` in `class.rs` handles
/// auto-vivification by calling the factory when a key is missing.
pub fn mb_defaultdict_new(factory: MbValue) -> MbValue {
    // CPython: the first argument must be callable or None.
    if !factory.is_none()
        && crate::runtime::builtins::mb_callable(factory).as_bool() != Some(true)
    {
        return raise_type_error("first argument must be callable or None");
    }
    let dict = MbValue::from_ptr(MbObject::new_dict());
    let mut fields = FxHashMap::default();
    fields.insert("_data".into(), dict);
    fields.insert("_factory".into(), factory);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.defaultdict".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// collections.namedtuple(name, fields) -> factory Instance
///
/// Returns a `collections.namedtuple_factory` Instance that, when called
/// with positional args, creates a namedtuple Instance with the declared
/// field names. Dispatch is handled in `mb_call_spread` (builtins.rs).
pub fn mb_namedtuple(name: MbValue, fields: MbValue) -> MbValue {
    // Extract field names from a list of strings
    let field_names: Vec<String> = if let Some(ptr) = fields.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().iter()
                    .filter_map(|v| extract_str(*v))
                    .collect(),
                ObjData::Str(ref s) => s.replace(',', " ").split_whitespace()
                    .map(|f| f.to_string()).collect(),
                _ => vec![],
            }
        }
    } else {
        vec![]
    };
    // CPython: field names must be valid non-keyword identifiers without
    // leading underscores or duplicates; violations raise ValueError.
    const PY_KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await",
        "break", "class", "continue", "def", "del", "elif", "else", "except",
        "finally", "for", "from", "global", "if", "import", "in", "is",
        "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
        "while", "with", "yield",
    ];
    let mut seen = std::collections::HashSet::new();
    for f in &field_names {
        let valid_ident = !f.is_empty()
            && f.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_')
            && f.chars().all(|c| c.is_alphanumeric() || c == '_');
        if !valid_ident {
            return raise_value_error(&format!(
                "Type names and field names must be valid identifiers: {f:?}"
            ));
        }
        if PY_KEYWORDS.contains(&f.as_str()) {
            return raise_value_error(&format!(
                "Type names and field names cannot be a keyword: {f:?}"
            ));
        }
        if f.starts_with('_') {
            return raise_value_error(&format!(
                "Field names cannot start with an underscore: {f:?}"
            ));
        }
        if !seen.insert(f.clone()) {
            return raise_value_error(&format!("Encountered duplicate field name: {f:?}"));
        }
    }
    let tuple_name = extract_str(name).unwrap_or_else(|| "namedtuple".to_string());
    let mut factory_fields = FxHashMap::default();
    factory_fields.insert("_tuple_name".into(),
        MbValue::from_ptr(MbObject::new_str(tuple_name)));
    let field_vals: Vec<MbValue> = field_names.iter()
        .map(|f| MbValue::from_ptr(MbObject::new_str(f.clone())))
        .collect();
    factory_fields.insert("_field_names".into(),
        MbValue::from_ptr(MbObject::new_list(field_vals)));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "collections.namedtuple_factory".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(factory_fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Create a namedtuple instance from the factory and positional args.
pub fn mb_namedtuple_create(factory: MbValue, args: &[MbValue]) -> MbValue {
    let (tuple_name, field_names) = if let Some(ptr) = factory.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let tn = f.get("_tuple_name").and_then(|v| extract_str(*v))
                    .unwrap_or_else(|| "namedtuple".to_string());
                let fns: Vec<String> = f.get("_field_names")
                    .and_then(|v| v.as_ptr())
                    .map(|p| {
                        if let ObjData::List(ref lock) = (*p).data {
                            lock.read().unwrap().iter()
                                .filter_map(|v| extract_str(*v))
                                .collect()
                        } else { vec![] }
                    })
                    .unwrap_or_default();
                (tn, fns)
            } else {
                return MbValue::none();
            }
        }
    } else {
        return MbValue::none();
    };
    // CPython: a namedtuple call must supply exactly the declared fields
    // (defaults are not modeled); arity mismatches raise TypeError.
    if args.len() != field_names.len() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "{}() takes {} positional arguments but {} were given",
                tuple_name,
                field_names.len(),
                args.len(),
            ))),
        );
        return MbValue::none();
    }
    let mut inst_fields = FxHashMap::default();
    inst_fields.insert("_namedtuple_name".into(),
        MbValue::from_ptr(MbObject::new_str(tuple_name.clone())));
    let ordered: Vec<MbValue> = field_names.iter()
        .map(|f| MbValue::from_ptr(MbObject::new_str(f.clone())))
        .collect();
    inst_fields.insert("_namedtuple_fields".into(),
        MbValue::from_ptr(MbObject::new_list(ordered)));
    for (i, fname) in field_names.iter().enumerate() {
        let val = args.get(i).copied().unwrap_or(MbValue::none());
        inst_fields.insert(fname.clone(), val);
    }
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: tuple_name,
            fields: crate::runtime::rc::MbRwLock::new(inst_fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// collections.ChainMap(*maps) -> dict that chains multiple dicts
///
/// Creates a dict by merging the input dicts in reverse order
/// (first dict has highest priority).
pub fn mb_chainmap_new(args: MbValue) -> MbValue {
    let maps = extract_list(args);
    let result = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*result).data {
            let mut out = lock.write().unwrap();
            // Iterate in reverse: last map is lowest priority
            for map_val in maps.iter().rev() {
                if let Some(ptr) = map_val.as_ptr() {
                    if let ObjData::Dict(ref map_lock) = (*ptr).data {
                        let m = map_lock.read().unwrap();
                        for (k, v) in m.iter() {
                            out.insert(k.clone(), *v);
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    /// Helper: extract the _data dict from a Counter Instance.
    fn counter_get_data(counter: MbValue) -> indexmap::IndexMap<crate::runtime::dict_ops::DictKey, MbValue> {
        super::counter_data(counter)
    }

    #[test]
    fn test_counter_new() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            s("a"),
            s("b"),
            s("a"),
            s("c"),
            s("a"),
            s("b"),
        ]));
        let counter = mb_counter_new(items);
        let data = counter_get_data(counter);
        assert_eq!(data.get("a").and_then(|v| v.as_int()), Some(3));
        assert_eq!(data.get("b").and_then(|v| v.as_int()), Some(2));
        assert_eq!(data.get("c").and_then(|v| v.as_int()), Some(1));
    }

    #[test]
    fn test_counter_most_common() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            s("a"),
            s("b"),
            s("a"),
            s("c"),
            s("a"),
            s("b"),
        ]));
        let counter = mb_counter_new(items);
        let top = mb_counter_most_common(counter, MbValue::from_int(2));
        unsafe {
            let ptr = top.as_ptr().expect("expected list");
            if let ObjData::List(ref lock) = (*ptr).data {
                let pairs = lock.read().unwrap();
                assert_eq!(pairs.len(), 2);
                // First should be "a" with count 3
                let first = pairs[0].as_ptr().unwrap();
                if let ObjData::List(ref lock2) = (*first).data {
                    let pair = lock2.read().unwrap();
                    let key = extract_str(pair[0]).unwrap();
                    assert_eq!(key, "a");
                    assert_eq!(pair[1].as_int(), Some(3));
                }
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_deque_operations() {
        let deque = mb_deque_new();
        mb_deque_appendleft(deque, MbValue::from_int(1));
        mb_deque_appendleft(deque, MbValue::from_int(2));
        mb_deque_appendleft(deque, MbValue::from_int(3));

        // Deque should be [3, 2, 1]
        let first = mb_deque_popleft(deque);
        assert_eq!(first.as_int(), Some(3));

        let second = mb_deque_popleft(deque);
        assert_eq!(second.as_int(), Some(2));
    }

    #[test]
    fn test_ordereddict_new() {
        let od = mb_ordereddict_new(&[]);
        assert!(od.as_ptr().is_some());
        // Empty constructor reprs as `OrderedDict()`.
        assert_eq!(ordereddict_repr(od), "OrderedDict()");
    }

    #[test]
    fn test_deque_popleft_empty() {
        let deque = mb_deque_new();
        let result = mb_deque_popleft(deque);
        assert!(result.is_none());
    }

    #[test]
    fn test_counter_empty_list() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![]));
        let counter = mb_counter_new(items);
        let data = counter_get_data(counter);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_counter_non_list_input() {
        let result = mb_counter_new(MbValue::from_int(42));
        // Should return empty counter for non-list
        let data = counter_get_data(result);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_counter_with_integers() {
        use crate::runtime::dict_ops::DictKey;
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(1),
        ]));
        let counter = mb_counter_new(items);
        let data = counter_get_data(counter);
        // Int items become DictKey::Int (Python: Counter([1,2,1]) keys are ints,
        // not stringified — only str-typed lookups go through Equivalent<DictKey>).
        assert_eq!(data.get(&DictKey::Int(1)).and_then(|v| v.as_int()), Some(2));
        assert_eq!(data.get(&DictKey::Int(2)).and_then(|v| v.as_int()), Some(1));
    }

    #[test]
    fn test_counter_most_common_all() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            s("x"), s("y"), s("x"),
        ]));
        let counter = mb_counter_new(items);
        // Pass no limit (use None to get all)
        let top = mb_counter_most_common(counter, MbValue::none());
        unsafe {
            let ptr = top.as_ptr().expect("expected list");
            if let ObjData::List(ref lock) = (*ptr).data {
                let pairs = lock.read().unwrap();
                assert_eq!(pairs.len(), 2);
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_deque_multiple_ops() {
        let deque = mb_deque_new();
        mb_deque_appendleft(deque, MbValue::from_int(10));
        mb_deque_appendleft(deque, MbValue::from_int(20));

        let first = mb_deque_popleft(deque);
        assert_eq!(first.as_int(), Some(20));

        let second = mb_deque_popleft(deque);
        assert_eq!(second.as_int(), Some(10));

        let empty = mb_deque_popleft(deque);
        assert!(empty.is_none());
    }

    #[test]
    fn test_value_to_key() {
        assert_eq!(value_to_key(MbValue::none()), "None");
        assert_eq!(value_to_key(MbValue::from_int(42)), "int:42");
        assert_eq!(value_to_key(MbValue::from_bool(true)), "bool:true");
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_counter_counts_correctly() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            s("a"), s("b"), s("a"), s("c"), s("a"),
        ]));
        let counter = mb_counter_new(items);
        assert!(counter.is_ptr());
        let data = counter_get_data(counter);
        assert_eq!(data.get("a").and_then(|v| v.as_int()), Some(3));
        assert_eq!(data.get("b").and_then(|v| v.as_int()), Some(1));
        assert_eq!(data.get("c").and_then(|v| v.as_int()), Some(1));
    }

    #[test]
    fn test_py312_counter_most_common_limited() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![
            s("x"), s("x"), s("y"), s("z"), s("x"),
        ]));
        let counter = mb_counter_new(items);
        let top = mb_counter_most_common(counter, MbValue::from_int(1));
        unsafe {
            if let ObjData::List(ref lock) = (*top.as_ptr().unwrap()).data {
                let pairs = lock.read().unwrap();
                assert_eq!(pairs.len(), 1);
            }
        }
    }

    #[test]
    fn test_py312_deque_fifo_order() {
        let dq = mb_deque_new();
        mb_deque_appendleft(dq, MbValue::from_int(1));
        mb_deque_appendleft(dq, MbValue::from_int(2));
        mb_deque_appendleft(dq, MbValue::from_int(3));
        assert_eq!(mb_deque_popleft(dq).as_int(), Some(3));
        assert_eq!(mb_deque_popleft(dq).as_int(), Some(2));
        assert_eq!(mb_deque_popleft(dq).as_int(), Some(1));
    }

    #[test]
    fn test_py312_deque_empty_popleft_returns_none() {
        let dq = mb_deque_new();
        assert!(mb_deque_popleft(dq).is_none());
    }

    #[test]
    fn test_py312_value_to_key_none() {
        assert_eq!(value_to_key(MbValue::none()), "None");
    }

    #[test]
    fn test_py312_value_to_key_int() {
        assert_eq!(value_to_key(MbValue::from_int(0)), "int:0");
        assert_eq!(value_to_key(MbValue::from_int(-1)), "int:-1");
    }

    // REQ: R5

    #[test]
    fn test_deque_append() {
        let dq = mb_deque_new();
        mb_deque_append(dq, MbValue::from_int(1));
        mb_deque_append(dq, MbValue::from_int(2));
        mb_deque_append(dq, MbValue::from_int(3));
        // Items are appended to the right; the last item should be 3
        unsafe {
            if let Some(ptr) = dq.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    assert_eq!(items.len(), 3);
                    assert_eq!(items[2].as_int(), Some(3));
                    assert_eq!(items[0].as_int(), Some(1));
                } else {
                    panic!("expected List");
                }
            }
        }
    }

    #[test]
    fn test_deque_pop() {
        let dq = mb_deque_new();
        mb_deque_append(dq, MbValue::from_int(10));
        mb_deque_append(dq, MbValue::from_int(20));
        mb_deque_append(dq, MbValue::from_int(30));
        // pop returns the last element
        let last = mb_deque_pop(dq);
        assert_eq!(last.as_int(), Some(30));
        let second_last = mb_deque_pop(dq);
        assert_eq!(second_last.as_int(), Some(20));
    }

    #[test]
    fn test_deque_pop_empty() {
        let dq = mb_deque_new();
        let result = mb_deque_pop(dq);
        assert!(result.is_none());
    }

    #[test]
    fn test_deque_rotate_positive() {
        // [1,2,3] rotate(1) -> [3,1,2]
        let dq = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        mb_deque_rotate(dq, MbValue::from_int(1));
        unsafe {
            if let Some(ptr) = dq.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    assert_eq!(items[0].as_int(), Some(3));
                    assert_eq!(items[1].as_int(), Some(1));
                    assert_eq!(items[2].as_int(), Some(2));
                } else {
                    panic!("expected List");
                }
            }
        }
    }

    #[test]
    fn test_deque_rotate_negative() {
        // [1,2,3] rotate(-1) -> [2,3,1]
        let dq = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        mb_deque_rotate(dq, MbValue::from_int(-1));
        unsafe {
            if let Some(ptr) = dq.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    assert_eq!(items[0].as_int(), Some(2));
                    assert_eq!(items[1].as_int(), Some(3));
                    assert_eq!(items[2].as_int(), Some(1));
                } else {
                    panic!("expected List");
                }
            }
        }
    }

    // -- UserDict / UserList / UserString tests (Wave-7 ship #3) --

    #[test]
    fn test_userdict_empty() {
        let d = mb_userdict_new(MbValue::none());
        unsafe {
            let ptr = d.as_ptr().expect("UserDict should be a ptr");
            if let ObjData::Dict(ref lock) = (*ptr).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_userdict_copies_initial() {
        let src = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*src).data {
                let mut m = lock.write().unwrap();
                m.insert("k".into(), MbValue::from_int(7));
                m.insert("k2".into(), MbValue::from_int(8));
            }
        }
        let d = mb_userdict_new(MbValue::from_ptr(src));
        unsafe {
            let ptr = d.as_ptr().unwrap();
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let m = lock.read().unwrap();
                assert_eq!(m.len(), 2);
                assert_eq!(m.get("k").and_then(|v| v.as_int()), Some(7));
                assert_eq!(m.get("k2").and_then(|v| v.as_int()), Some(8));
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_userlist_empty() {
        let l = mb_userlist_new(MbValue::none());
        unsafe {
            let ptr = l.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_userlist_copies_initial() {
        let src = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ]));
        let l = mb_userlist_new(src);
        unsafe {
            let ptr = l.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(1));
                assert_eq!(items[2].as_int(), Some(3));
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_userstring_from_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let r = mb_userstring_new(s);
        assert_eq!(extract_str(r), Some("hello".to_string()));
    }

    #[test]
    fn test_userstring_from_int_coerces() {
        let r = mb_userstring_new(MbValue::from_int(42));
        assert_eq!(extract_str(r), Some("42".to_string()));
    }

    #[test]
    fn test_userstring_from_none_empty() {
        let r = mb_userstring_new(MbValue::none());
        assert_eq!(extract_str(r), Some(String::new()));
    }
}
