use super::class;
use super::dict_ops::{dict_key_to_mbvalue, DictKey};
use super::rc::{MbObject, ObjData};
/// Iterator protocol for the Mamba runtime (#286).
///
/// Implements Python's __iter__/__next__ protocol with iterators for
/// built-in types (list, dict, tuple, str, range).
use super::value::MbValue;
use std::collections::HashMap;

/// Iterator state — stores the current position and source.
pub struct MbIterator {
    pub kind: IterKind,
    pub index: usize,
    pub exhausted: bool,
    /// Pre-fetched value from `mb_has_next`.  When `mb_has_next` is called
    /// it advances the iterator internally and caches the result here so
    /// that the subsequent `mb_next` call can return it without re-advancing.
    /// This makes the "check-then-next" for-loop pattern work correctly for
    /// ALL iterator kinds (including generators and composite iterators).
    pub peeked: Option<MbValue>,
}

pub enum IterKind {
    /// Iterating over a list
    List(MbValue),
    /// Iterating over dict keys (preserves the original key type:
    /// int, str, bool, None, instance, etc.).
    DictKeys(Vec<DictKey>),
    /// Iterating over a tuple
    Tuple(MbValue),
    /// Iterating over string characters
    Str(Vec<char>),
    /// Iterating over range(start, stop, step)
    Range { current: i64, stop: i64, step: i64 },
    /// Iterating over enumerate(iter).
    ///
    /// The inner iterator stays as a separate ITERATORS entry (`inner_id`)
    /// rather than an embedded Box, so the out-of-line advance can release the
    /// ITERATORS borrow before driving the inner via `mb_next`. The inner may
    /// itself be a Map/Filter/MapN whose advance calls user code (which can
    /// re-enter ITERATORS); advancing it while a borrow is held would panic.
    Enumerate { inner_id: u64, count: i64 },
    /// Iterating over zip(iter1, iter2[, ...], [strict=False]).
    /// `strict=True` makes the iterator validate that all inputs have the
    /// same length (PEP 618 / Py3.10+); a length mismatch raises ValueError.
    ///
    /// Each inner stays as a separate ITERATORS entry (`inner_ids`) for the
    /// same reentrancy rationale as Enumerate/Map/Filter.
    Zip { inner_ids: Vec<u64>, strict: bool },
    /// Iterating over map(func, iter).
    ///
    /// The inner iterator is stored as a separate ITERATORS entry (`inner_id`)
    /// rather than an embedded Box, so that the out-of-line advance function
    /// can release the ITERATORS borrow before calling `func` (user code may
    /// access ITERATORS via `mb_hasattr` → `mb_is_iterator_handle`).
    Map { func: MbValue, inner_id: u64 },
    /// Iterating over filter(func, iter). Same reentrancy rationale as Map.
    Filter { func: MbValue, inner_id: u64 },
    /// Iterating over map(func, iter1, iter2, ...) — multi-iterable form.
    /// Each inner iterator is a separate ITERATORS entry (same reentrancy
    /// rationale as Map/Filter).
    MapN { func: MbValue, inner_ids: Vec<u64> },
    /// Iterating over reversed(seq)
    Reversed { items: Vec<MbValue>, index: usize },
    /// User-defined iterator: object with __next__ dunder
    UserDefined(MbValue),
    /// Generator iterator: wraps a generator handle
    Generator(MbValue),
    /// Callable-sentinel iterator: iter(callable, sentinel) — calls callable()
    /// on each step; stops when return value equals sentinel (PEP 234).
    Callable { func: MbValue, sentinel: MbValue },
    /// itertools.count(start, step) — lazy infinite arithmetic counter.
    /// `is_float` selects the int (`cur_i`/`step_i`) or float
    /// (`cur_f`/`step_f`) state. Never exhausts.
    Count {
        is_float: bool,
        cur_i: i64,
        step_i: i64,
        cur_f: f64,
        step_f: f64,
    },
    /// itertools.repeat(value[, times]) — yields `value`. `remaining` is
    /// `None` for the infinite form and `Some(n)` for the bounded form.
    Repeat {
        val: MbValue,
        remaining: Option<usize>,
    },
    /// itertools.cycle(iterable) — yields the materialized `items` forever.
    /// The source is drained at construction (CPython caches on first pass),
    /// so a raising/empty source is handled eagerly. Empty `items` exhausts
    /// immediately; otherwise `pos` walks the cache modulo its length.
    Cycle { items: Vec<MbValue>, pos: usize },
}

/// Base for iterator IDs — must not overlap with generator IDs (which start at 1).
const ITER_ID_BASE: u64 = 0x1_0000_0000;

// Thread-local iterator storage.
thread_local! {
    static ITERATORS: std::cell::RefCell<HashMap<u64, MbIterator>> =
        std::cell::RefCell::new(HashMap::new());
    /// Iterator IDs start at 0x1_0000_0000 to avoid collisions with generator
    /// handles (which start at 1 from NEXT_GEN_ID). Both live in the same
    /// thread-local address space, so dispatch in mb_next_raise / mb_has_next
    /// must be able to distinguish them.
    static NEXT_ITER_ID: std::cell::Cell<u64> = std::cell::Cell::new(ITER_ID_BASE);
    /// StopIteration flag — set by __next__ to signal exhaustion.
    /// Separates "yielded None" from "iterator is done".
    static STOP_ITERATION: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

fn raise_type_error(msg: &str) -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_not_iterable_type_error(value: MbValue) -> MbValue {
    let type_name = if value.as_int().is_some() {
        "int"
    } else if value.is_bool() {
        "bool"
    } else if value.as_float().is_some() {
        "float"
    } else if value.is_none() {
        "NoneType"
    } else {
        "object"
    };
    raise_type_error(&format!("'{type_name}' object is not iterable"))
}

/// Return true if `val` is a live iterator handle (registered in ITERATORS).
///
/// Used by `mb_hasattr` / `mb_getattr` to surface `__next__` / `__iter__`
/// on iterator handles that are stored as integer IDs (not heap objects).
pub fn mb_is_iterator_handle(val: MbValue) -> bool {
    if let Some(id) = val.as_int() {
        ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)))
    } else {
        false
    }
}

/// Raise StopIteration — called by compiled __next__ when done.
pub fn mb_stop_iteration(_dummy: MbValue) -> MbValue {
    STOP_ITERATION.with(|f| f.set(true));
    MbValue::none()
}

/// Signal StopIteration from exception.rs (when `raise StopIteration` is used).
pub fn signal_stop_iteration() {
    STOP_ITERATION.with(|f| f.set(true));
}

/// Check and clear the StopIteration flag.
fn check_stop_iteration() -> bool {
    STOP_ITERATION.with(|f| {
        let was_set = f.get();
        f.set(false);
        was_set
    })
}

/// Public version: check and clear the StopIteration flag.
/// Used by generator yield_from to clear sub-generator's StopIteration.
pub fn check_and_clear_stop() -> bool {
    check_stop_iteration()
}

fn alloc_iter_id() -> u64 {
    NEXT_ITER_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Is `v` an iterator handle (i.e. a NaN-boxed int that maps to an entry
/// in the per-thread ITERATORS registry)? Constructors that distinguish
/// between "iterable source" and "integer count" (e.g. `bytes(N)` vs
/// `bytes(iter)`) need this probe — `as_int()` alone can't tell the cases
/// apart because iterator IDs are themselves NaN-boxed ints (#2103).
pub fn is_iter_handle(v: MbValue) -> bool {
    if let Some(id) = v.as_int() {
        ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)))
    } else {
        false
    }
}

/// CPython-visible type name for iterator handles backed by the iterator
/// registry. Handles are tagged ints, so callers must consult the registry
/// before primitive-int fallbacks.
pub fn mb_iter_type_name(v: MbValue) -> Option<&'static str> {
    let id = v.as_int()? as u64;
    ITERATORS.with(|iters| {
        iters.borrow().get(&id).map(|iter| match iter.kind {
            IterKind::Range { .. } => "range",
            IterKind::Enumerate { .. } => "enumerate",
            IterKind::Zip { .. } => "zip",
            IterKind::Map { .. } | IterKind::MapN { .. } => "map",
            IterKind::Filter { .. } => "filter",
            IterKind::Reversed { .. } => "list_reverseiterator",
            IterKind::List(_) => "list_iterator",
            IterKind::Tuple(_) => "tuple_iterator",
            IterKind::DictKeys(_) => "dict_keyiterator",
            IterKind::Str(_) => "str_ascii_iterator",
            IterKind::Generator(_) => "generator",
            IterKind::UserDefined { .. } => "iterator",
            IterKind::Callable { .. } => "callable_iterator",
            IterKind::Count { .. } => "count",
            IterKind::Repeat { .. } => "repeat",
            IterKind::Cycle { .. } => "cycle",
        })
    })
}

/// Register a freshly-created generator handle as an iterator keyed by its
/// own gen_id. CPython contract: `iter(g) is g` — generators ARE their own
/// iterators. By inserting an `IterKind::Generator` entry whose key equals
/// the gen handle's integer payload, `mb_iter(gen)` falls through the
/// "already an iter" shortcut and returns the gen handle unchanged, while
/// every dispatcher (`mb_next`, `mb_has_next`, `mb_next_or_stop`,
/// `mb_next_raise`) finds a Generator-kind entry with proper peek-ahead
/// semantics. Gen IDs (start at 1) and iter IDs (start at 2^32) live in
/// disjoint sub-ranges of the same u64 space, so no collision is possible
/// for any realistic process lifetime.
pub fn register_generator_iter(gen_handle: MbValue) {
    if let Some(id) = gen_handle.as_int() {
        let iter = MbIterator {
            kind: IterKind::Generator(gen_handle),
            index: 0,
            exhausted: false,
            peeked: None,
        };
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(id as u64, iter);
        });
    }
}

/// Tear down the ITERATORS entry for a generator handle. Called from
/// generator release / cleanup paths so the IterKind::Generator entry
/// doesn't outlive the gen registration.
pub fn unregister_generator_iter(gen_handle: MbValue) {
    if let Some(id) = gen_handle.as_int() {
        ITERATORS.with(|iters| {
            iters.borrow_mut().remove(&(id as u64));
        });
    }
}

/// Drain an iterator handle into a Vec<MbValue> in a single batch.
/// For Range iterators, this builds the Vec directly without per-element
/// has_next/next calls, avoiding 2*N HashMap lookups.
/// For other iterator kinds, falls back to the standard protocol.
/// Returns None if the handle is not a valid iterator (caller should fall back).
pub fn drain_iter_to_vec(handle: MbValue) -> Option<Vec<MbValue>> {
    let id = handle.as_int()? as u64;
    if let Some((current, stop, step)) = mb_iter_range_params(handle) {
        let count = if step > 0 {
            ((stop - current + step - 1) / step).max(0) as usize
        } else if step < 0 {
            ((current - stop - step - 1) / (-step)).max(0) as usize
        } else {
            0
        };
        let mut items = Vec::with_capacity(count);
        let mut cur = current;
        if step > 0 {
            while cur < stop {
                items.push(MbValue::from_int(cur));
                cur += step;
            }
        } else {
            while cur > stop {
                items.push(MbValue::from_int(cur));
                cur += step;
            }
        }
        return Some(items);
    }
    // Remove the iterator from storage to avoid borrowing issues.
    let iter = ITERATORS.with(|iters| iters.borrow_mut().remove(&id))?;

    let result = match iter.kind {
        IterKind::Range {
            current,
            stop,
            step,
        } => {
            // Compute the number of elements and build Vec in one allocation.
            let count = if step > 0 {
                ((stop - current + step - 1) / step).max(0) as usize
            } else if step < 0 {
                ((current - stop - step - 1) / (-step)).max(0) as usize
            } else {
                0
            };
            let mut items = Vec::with_capacity(count);
            let mut cur = current;
            if step > 0 {
                while cur < stop {
                    items.push(MbValue::from_int(cur));
                    cur += step;
                }
            } else {
                while cur > stop {
                    items.push(MbValue::from_int(cur));
                    cur += step;
                }
            }
            Some(items)
        }
        IterKind::List(val) => {
            // Drain a list iterator: clone the remaining items, then release the
            // iterator's retained reference on the list (see `mb_iter`). Items
            // are retained so they survive if the list is freed by the release.
            let out = if let Some(ptr) = val.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        let items = lock.read().unwrap();
                        // The list may have shrunk below the iterator's cursor
                        // after it was advanced (CPython then drains to empty);
                        // clamp so the slice cannot panic on an out-of-range start.
                        let start = iter.index.min(items.len());
                        for &it in &items[start..] {
                            super::rc::retain_if_ptr(it);
                        }
                        items[start..].to_vec()
                    } else {
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            unsafe {
                super::rc::release_if_ptr(val);
            }
            Some(out)
        }
        IterKind::Tuple(val) => {
            let out = if let Some(ptr) = val.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(ref items) = (*ptr).data {
                        let start = iter.index.min(items.len());
                        for &it in &items[start..] {
                            super::rc::retain_if_ptr(it);
                        }
                        items[start..].to_vec()
                    } else {
                        vec![]
                    }
                }
            } else {
                vec![]
            };
            unsafe {
                super::rc::release_if_ptr(val);
            }
            Some(out)
        }
        IterKind::Reversed { items, index } => {
            // Remaining items from the reversed iterator.
            let start = index.min(items.len());
            Some(items[start..].to_vec())
        }
        IterKind::DictKeys(ref keys) => {
            // Dict-key iterators (dict / Counter / defaultdict views) drain to
            // the original key values.
            let start = iter.index.min(keys.len());
            Some(keys[start..].iter().map(dict_key_to_mbvalue).collect())
        }
        IterKind::Str(ref chars) => {
            let start = iter.index.min(chars.len());
            Some(
                chars[start..]
                    .iter()
                    .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                    .collect(),
            )
        }
        _ => {
            // Put the iterator back for fallback protocol.
            ITERATORS.with(|iters| iters.borrow_mut().insert(id, iter));
            None
        }
    };

    // A CPython iterator re-iterates to empty once exhausted rather than
    // raising on the (now stale) handle. The eagerly-drained kinds above
    // consumed the registry entry; re-insert an exhausted empty placeholder
    // under the same id so a second `list(it)` / `tuple(it)` yields `[]`
    // instead of treating the bare int handle as a non-iterable. This matches
    // the incremental `mb_next` path, which marks `exhausted` and keeps the
    // entry. (The `_` fallback arm re-inserted the live iterator and returned
    // None, so only re-place when we actually drained — `result.is_some()`.)
    if result.is_some() {
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::Str(Vec::new()),
                    index: 0,
                    exhausted: true,
                    peeked: None,
                },
            );
        });
    }
    result
}

// ── Iterator Creation ──

/// Create an iterator for any iterable object.
/// Checks built-in types first, then falls back to __iter__ dunder (R1).
pub fn mb_iter(obj: MbValue) -> MbValue {
    // If this is already a known iterator handle (from mb_enumerate, mb_zip,
    // mb_reversed, etc., or a generator handle pre-registered by
    // `mb_generator_create` via `register_generator_iter`), return it
    // directly — `iter(g) is g` for generators (CPython contract).
    if let Some(id) = obj.as_int() {
        let is_iter = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
        if is_iter {
            return obj;
        }
        // Text file handles are bare ints; `for line in f` iterates their
        // remaining lines. Read forward to EOF eagerly and wrap as a list
        // iterator so each `next()` yields one newline-kept line (CPython
        // file-object iteration semantics).
        if super::file_io::is_file_handle(id as u64) {
            let lines = super::file_io::mb_file_readlines(obj);
            // readlines() returns a freshly-allocated list (rc==1); the
            // iterator owns that reference, so no extra retain is needed.
            let kind = IterKind::List(lines);
            let iter = MbIterator {
                kind,
                index: 0,
                exhausted: false,
                peeked: None,
            };
            let new_id = alloc_iter_id();
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(new_id, iter);
            });
            return MbValue::from_int(new_id as i64);
        }
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            let kind = match &(*ptr).data {
                ObjData::List(_) => {
                    // Retain: iterator owns a reference to the list so its
                    // lifetime is independent of the caller's scope (e.g. a
                    // function returning `iter([...])` must keep the list
                    // alive after the function frame is torn down).
                    super::rc::retain_if_ptr(obj);
                    IterKind::List(obj)
                }
                ObjData::Tuple(_) => {
                    super::rc::retain_if_ptr(obj);
                    IterKind::Tuple(obj)
                }
                ObjData::Str(s) => {
                    // Class-body enum classes iterate canonical members in
                    // definition order (`for c in Color`), not the class-name
                    // string's characters. Gated: one flag read for programs
                    // without enums.
                    if let Some(members) = super::stdlib::enum_class::class_canonical_members(s) {
                        IterKind::List(MbValue::from_ptr(MbObject::new_list_borrowed(members)))
                    } else {
                        IterKind::Str(s.chars().collect())
                    }
                }
                ObjData::Dict(ref lock) => {
                    // ET.Element stub dicts iterate their children, not keys.
                    if let Some(children) = super::stdlib::xml_mod::element_stub_children(obj) {
                        if let Some(cp) = children.as_ptr() {
                            if let ObjData::List(ref clock) = (*cp).data {
                                let items = clock.read().unwrap().to_vec();
                                IterKind::List(MbValue::from_ptr(MbObject::new_list_borrowed(
                                    items,
                                )))
                            } else {
                                IterKind::DictKeys(lock.read().unwrap().keys().cloned().collect())
                            }
                        } else {
                            IterKind::DictKeys(lock.read().unwrap().keys().cloned().collect())
                        }
                    } else {
                        IterKind::DictKeys(lock.read().unwrap().keys().cloned().collect())
                    }
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap();
                    IterKind::List(MbValue::from_ptr(MbObject::new_list_borrowed(
                        items.to_vec(),
                    )))
                }
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    // tempfile NamedTemporaryFile / SpooledTemporaryFile
                    // iterate their remaining lines, like a real file object.
                    if let Some(lines) = super::stdlib::tempfile_mod::tempfile_iter_lines(obj) {
                        IterKind::List(lines)
                    } else
                    // namedtuple instances iterate over declared field values
                    // in order, matching CPython tuple-iter semantics. Built
                    // from a fresh tuple so the values stay alive via the
                    // tuple's retain.
                    if let Some(vals) =
                        super::stdlib::collections_mod::namedtuple_values(obj)
                    {
                        let t = MbValue::from_ptr(MbObject::new_tuple(vals));
                        IterKind::Tuple(t)
                    } else
                    // Functional-API enum class objects (`enum.Enum('M', 'a b')`)
                    // iterate their members in definition order (`for m in M`).
                    if let Some(items) =
                        super::stdlib::enum_mod::functional_enum_members(obj)
                    {
                        IterKind::List(MbValue::from_ptr(MbObject::new_list_borrowed(items)))
                    } else
                    // UserDict / UserList / UserString iterate their payload
                    // (dict keys / list items / characters).
                    if let Some((_, data)) =
                        super::stdlib::collections_mod::user_wrapper_data(obj)
                    {
                        return mb_iter(data);
                    } else
                    // memoryview iterates its underlying buffer's byte values —
                    // delegate to the bytes/bytearray iterator.
                    if class_name == "memoryview" {
                        let buf = fields
                            .read()
                            .unwrap()
                            .get("_buffer")
                            .copied()
                            .unwrap_or(MbValue::none());
                        return mb_iter(buf);
                    } else
                    // Dict-like collections classes: iterate over backing _data dict keys.
                    if class_name == "collections.defaultdict"
                        || class_name == "collections.Counter"
                        || class_name == "collections.OrderedDict"
                    {
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        if let Some(dptr) = data.as_ptr() {
                            if let ObjData::Dict(ref lock) = (*dptr).data {
                                IterKind::DictKeys(lock.read().unwrap().keys().cloned().collect())
                            } else {
                                return MbValue::none();
                            }
                        } else {
                            return MbValue::none();
                        }
                    } else {
                        // R1: Look up __iter__ in class methods (not instance fields)
                        // using mb_lookup_dunder for safety (only returns valid fn ptrs)
                        let iter_method = class::mb_lookup_dunder(
                            obj,
                            MbValue::from_ptr(MbObject::new_str("__iter__".into())),
                        );
                        if iter_method.is_none() {
                            return raise_type_error(&format!(
                                "'{}' object is not iterable",
                                class_name
                            ));
                        }
                        // Invoke __iter__(self) to get the iterator object
                        let iter_obj = class::mb_call_method1(iter_method, obj);
                        if iter_obj.is_none() {
                            return MbValue::none();
                        }
                        // If __iter__ returned a built-in iterator handle (e.g.
                        // `return iter([1,2,3])`), use it directly — wrapping it as
                        // UserDefined would try to look up __next__ on an int, fail,
                        // and produce an empty iterator.
                        if let Some(inner_id) = iter_obj.as_int() {
                            let is_known_iter = ITERATORS
                                .with(|iters| iters.borrow().contains_key(&(inner_id as u64)));
                            if is_known_iter {
                                return iter_obj;
                            }
                            // Generators are also ints (their handle id) with a separate registry.
                            if super::generator::is_known_generator(iter_obj) {
                                let iter = MbIterator {
                                    kind: IterKind::Generator(iter_obj),
                                    index: 0,
                                    exhausted: false,
                                    peeked: None,
                                };
                                let id = alloc_iter_id();
                                ITERATORS.with(|iters| {
                                    iters.borrow_mut().insert(id, iter);
                                });
                                return MbValue::from_int(id as i64);
                            }
                        }
                        // Retain iter_obj — it will live in ITERATORS map, GC must not free it.
                        super::rc::retain_if_ptr(iter_obj);
                        // The returned iterator object must have __next__
                        IterKind::UserDefined(iter_obj)
                    }
                }
                ObjData::FrozenSet(items) => IterKind::List(MbValue::from_ptr(
                    MbObject::new_list_borrowed(items.clone()),
                )),
                ObjData::Bytes(ref data) => {
                    let items: Vec<MbValue> =
                        data.iter().map(|b| MbValue::from_int(*b as i64)).collect();
                    IterKind::List(MbValue::from_ptr(MbObject::new_list(items)))
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    let items: Vec<MbValue> =
                        data.iter().map(|b| MbValue::from_int(*b as i64)).collect();
                    IterKind::List(MbValue::from_ptr(MbObject::new_list(items)))
                }
                ObjData::BigInt(_) => {
                    return raise_type_error("'int' object is not iterable");
                }
                ObjData::Complex(_, _) => {
                    return raise_type_error("'complex' object is not iterable");
                }
                ObjData::CodeObject { .. } => {
                    return raise_type_error("'code' object is not iterable");
                }
            };
            let iter = MbIterator {
                kind,
                index: 0,
                exhausted: false,
                peeked: None,
            };
            let id = alloc_iter_id();
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(id, iter);
            });
            MbValue::from_int(id as i64) // Iterator handle
        }
    } else {
        // Primitives (int, bool, float, None) are not iterable
        raise_not_iterable_type_error(obj)
    }
}

/// Create a callable-sentinel iterator: iter(callable, sentinel).
/// Calls callable() on each step; stops when the return value equals sentinel.
pub fn mb_iter_sentinel(callable: MbValue, sentinel: MbValue) -> MbValue {
    let iter = MbIterator {
        kind: IterKind::Callable {
            func: callable,
            sentinel,
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Compute the remaining-elements count of a Range-kind iterator handle.
/// Returns None for non-range or non-iterator values. Used by len(range(...))
/// and range.__len__ so `len(range(a, b, c))` answers in O(1) without
/// consuming the iterator.
pub fn mb_iter_range_len(handle: MbValue) -> Option<i64> {
    let id = handle.as_int()? as u64;
    ITERATORS.with(|iters| {
        let borrowed = iters.borrow();
        let it = borrowed.get(&id)?;
        if it.exhausted {
            return Some(0);
        }
        if let IterKind::Range {
            current,
            stop,
            step,
        } = it.kind
        {
            if step == 0 {
                return Some(0);
            }
            let remaining = if step > 0 {
                if current >= stop {
                    0
                } else {
                    (stop - current + step - 1) / step
                }
            } else {
                if current <= stop {
                    0
                } else {
                    (current - stop + (-step) - 1) / (-step)
                }
            };
            return Some(remaining);
        }
        None
    })
}

/// Best-effort `__length_hint__` for a live iterator handle: the number of
/// elements remaining. Returns None when the handle is not a length-known
/// iterator (generators, callable, map/filter/count/cycle, etc.) so callers can
/// fall back to a default. Used by `operator.length_hint`.
pub fn mb_iter_length_hint(handle: MbValue) -> Option<i64> {
    let id = handle.as_int()? as u64;
    ITERATORS.with(|iters| {
        let borrowed = iters.borrow();
        let it = borrowed.get(&id)?;
        if it.exhausted {
            return Some(0);
        }
        match &it.kind {
            IterKind::Range { .. } => {
                drop(borrowed);
                mb_iter_range_len(handle)
            }
            IterKind::List(v) | IterKind::Tuple(v) => {
                let total = v.as_ptr().map(|ptr| unsafe {
                    match &(*ptr).data {
                        ObjData::List(lock) => lock.read().unwrap().len(),
                        ObjData::Tuple(items) => items.len(),
                        _ => 0,
                    }
                })?;
                Some((total as i64 - it.index as i64).max(0))
            }
            IterKind::Str(chars) => Some((chars.len() as i64 - it.index as i64).max(0)),
            IterKind::DictKeys(keys) => Some((keys.len() as i64 - it.index as i64).max(0)),
            IterKind::Reversed { items, index } => {
                Some((items.len() as i64 - *index as i64).max(0))
            }
            IterKind::Repeat { remaining, .. } => remaining.map(|r| r as i64),
            _ => None,
        }
    })
}

/// If `handle` refers to an unexhausted Range iterator, return its
/// `(current, stop, step)`. Used by `mb_obj_contains` to implement
/// O(1) `value in range(...)` membership without iterating.
pub fn mb_iter_range_params(handle: MbValue) -> Option<(i64, i64, i64)> {
    let id = handle.as_int()? as u64;
    ITERATORS.with(|iters| {
        let borrowed = iters.borrow();
        let it = borrowed.get(&id)?;
        if it.exhausted {
            return None;
        }
        if let IterKind::Range {
            current,
            stop,
            step,
        } = it.kind
        {
            Some((current, stop, step))
        } else {
            None
        }
    })
}

/// CPython-compatible structural equality for two iterator handles **when
/// both are unexhausted Range-kind iterators**. Returns `Some(true|false)`
/// for that case and `None` when either side is not such a range — the
/// caller should fall through to identity comparison for non-range handles.
///
/// Rule (CPython `range_equal`): equal iff lengths match, and when length
/// >= 1 the starts (i.e. current cursors) match, and when length >= 2 the
/// steps also match. Empty ranges compare equal regardless of step/start.
pub fn ranges_value_eq(a: MbValue, b: MbValue) -> Option<bool> {
    let (ac, astop, ast) = mb_iter_range_params(a)?;
    let (bc, bstop, bst) = mb_iter_range_params(b)?;
    let len = |c: i64, e: i64, s: i64| -> i64 {
        if s == 0 {
            0
        } else if s > 0 {
            if c >= e {
                0
            } else {
                (e - c + s - 1) / s
            }
        } else {
            if c <= e {
                0
            } else {
                (c - e + (-s) - 1) / (-s)
            }
        }
    };
    let la = len(ac, astop, ast);
    let lb = len(bc, bstop, bst);
    if la != lb {
        return Some(false);
    }
    if la == 0 {
        return Some(true);
    }
    if ac != bc {
        return Some(false);
    }
    if la == 1 {
        return Some(true);
    }
    Some(ast == bst)
}

/// If `handle` refers to an unexhausted Range iterator, return the value at
/// position `idx` using CPython-compatible semantics (negative indices count
/// from the end). Returns None if `handle` is not a Range iter or if the
/// index is out of bounds; callers must raise IndexError when None is
/// returned to a known-Range input.
pub fn range_iter_getitem(handle: MbValue, idx: i64) -> Option<i64> {
    let (current, stop, step) = mb_iter_range_params(handle)?;
    let len = if step == 0 {
        0
    } else if step > 0 {
        if current >= stop {
            0
        } else {
            (stop - current + step - 1) / step
        }
    } else {
        if current <= stop {
            0
        } else {
            (current - stop + (-step) - 1) / (-step)
        }
    };
    let i = if idx < 0 { idx + len } else { idx };
    if i < 0 || i >= len {
        return None;
    }
    Some(current + i * step)
}

fn range_len(current: i64, stop: i64, step: i64) -> i64 {
    if step == 0 {
        0
    } else if step > 0 {
        if current >= stop {
            0
        } else {
            (stop - current + step - 1) / step
        }
    } else if current <= stop {
        0
    } else {
        (current - stop + (-step) - 1) / (-step)
    }
}

/// Slice a Range-kind iterator handle using CPython range slicing rules and
/// return a new Range-kind handle. The source handle is not consumed.
pub fn range_iter_slice(
    handle: MbValue,
    start_v: MbValue,
    stop_v: MbValue,
    step_v: MbValue,
) -> Option<MbValue> {
    let (current, stop, step) = mb_iter_range_params(handle)?;
    let len = range_len(current, stop, step);
    let slice_step = if step_v.is_none() {
        1
    } else {
        step_v.as_int_pyint().unwrap_or(1)
    };
    if slice_step == 0 {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str("slice step cannot be zero".to_string())),
        );
        return Some(MbValue::none());
    }

    let clamp_pos = |idx: i64| idx.max(0).min(len);
    let clamp_neg = |idx: i64| idx.max(-1).min(len - 1);

    let raw_start = start_v.as_int_pyint();
    let raw_stop = stop_v.as_int_pyint();
    let start_idx = if slice_step > 0 {
        raw_start
            .map(|i| {
                if i < 0 {
                    clamp_pos(i + len)
                } else {
                    clamp_pos(i)
                }
            })
            .unwrap_or(0)
    } else {
        raw_start
            .map(|i| {
                if i < 0 {
                    clamp_neg(i + len)
                } else {
                    clamp_neg(i)
                }
            })
            .unwrap_or(len - 1)
    };
    let stop_idx = if slice_step > 0 {
        raw_stop
            .map(|i| {
                if i < 0 {
                    clamp_pos(i + len)
                } else {
                    clamp_pos(i)
                }
            })
            .unwrap_or(len)
    } else {
        raw_stop
            .map(|i| {
                if i < 0 {
                    clamp_neg(i + len)
                } else {
                    clamp_neg(i)
                }
            })
            .unwrap_or(-1)
    };

    let new_start = current + start_idx * step;
    let new_stop = current + stop_idx * step;
    let new_step = step * slice_step;
    Some(mb_range_iter(
        MbValue::from_int(new_start),
        MbValue::from_int(new_stop),
        MbValue::from_int(new_step),
    ))
}

/// Create a range iterator.
/// Extract a range bound as i64. Accepts register ints and bools directly, and
/// unboxes a BigInt that fits i64 (range over `sys.maxsize`-scale bounds); any
/// other value (None, float, or a BigInt beyond i64) falls back to `default`.
fn range_bound(v: MbValue, default: i64) -> i64 {
    if let Some(i) = v.as_int_pyint() {
        return i;
    }
    use num_traits::ToPrimitive;
    if let Some(i) = unsafe { super::bigint_ops::extract_bigint(v) }.and_then(|b| b.to_i64()) {
        return i;
    }
    default
}

pub fn mb_range_iter(start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    // bool ≤ int (#1680). A bound above 2^47 is a NaN-box-promoted BigInt, so
    // `as_int_pyint` is None; unbox it to its exact i64 (range is i64-bounded)
    // so `range(sys.maxsize - 1)` spans its real length rather than collapsing
    // to an empty range.
    let s = range_bound(start, 0);
    let e = range_bound(stop, 0);
    let st = range_bound(step, 1);
    if st == 0 {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "range() arg 3 must not be zero".to_string(),
            )),
        );
        return MbValue::none();
    }

    let iter = MbIterator {
        kind: IterKind::Range {
            current: s,
            stop: e,
            step: st,
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a lazy `itertools.count(start, step)` iterator handle.
/// Promotes to float if either `start` or `step` is a float.
pub fn mb_count_iter(start: MbValue, step: MbValue) -> MbValue {
    let start_i = start.as_int();
    let step_i = step.as_int();
    let start_f = start.as_float();
    let step_f = step.as_float();
    let is_float =
        (start_f.is_some() && start_i.is_none()) || (step_f.is_some() && step_i.is_none());
    let iter = MbIterator {
        kind: IterKind::Count {
            is_float,
            cur_i: start_i.unwrap_or(0),
            step_i: step_i.unwrap_or(1),
            cur_f: start_f.or_else(|| start_i.map(|v| v as f64)).unwrap_or(0.0),
            step_f: step_f.or_else(|| step_i.map(|v| v as f64)).unwrap_or(1.0),
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a lazy `itertools.repeat(value[, times])` iterator handle.
/// `times` of `None` (not an int) yields the infinite form.
/// CPython-style repr for named itertools iterator handles. Returns None for
/// any handle that is not a recognized named iterator (callers fall back to
/// the default int formatting). Currently covers `repeat`.
pub fn mb_iter_repr(handle: MbValue) -> Option<(String)> {
    let id = handle.as_int()? as u64;
    let repeat = ITERATORS.with(|iters| {
        let iters = iters.borrow();
        match iters.get(&id).map(|it| &it.kind) {
            Some(IterKind::Repeat { val, remaining }) => Some((*val, *remaining)),
            _ => None,
        }
    })?;
    let (val, remaining) = repeat;
    let vr = super::builtins::mb_repr(val);
    let vr_s = vr
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_default();
    Some(match remaining {
        Some(n) => format!("repeat({vr_s}, {n})"),
        None => format!("repeat({vr_s})"),
    })
}

pub fn mb_repeat_iter(val: MbValue, times: MbValue) -> MbValue {
    let remaining = times.as_int().map(|n| n.max(0) as usize);
    unsafe {
        super::rc::retain_if_ptr(val);
    }
    let iter = MbIterator {
        kind: IterKind::Repeat { val, remaining },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a lazy `itertools.cycle(iterable)` iterator handle. The source is
/// drained at construction (CPython caches the first pass), so an empty source
/// exhausts immediately and a source that raises mid-drain propagates here.
pub fn mb_cycle_iter(iterable: MbValue) -> MbValue {
    let items = match drain_iter_to_vec(mb_iter(iterable)) {
        Some(v) => v,
        None => {
            // Fall back to the generic protocol for kinds drain_iter_to_vec
            // declines (generators, user __next__, composite iterators).
            let handle = mb_iter(iterable);
            let mut acc = Vec::new();
            loop {
                if mb_has_next(handle).as_bool() == Some(false) {
                    break;
                }
                let v = mb_next(handle);
                // A non-StopIteration exception means the source raised
                // mid-drain (e.g. cycle(gen()) where gen() raises) — propagate
                // it. StopIteration is the normal end-of-source signal.
                if let Some(t) = super::exception::current_exception_type() {
                    if t != "StopIteration" {
                        mb_iter_release(handle);
                        return MbValue::none();
                    }
                }
                if v.is_none() {
                    let exhausted = handle
                        .as_int()
                        .map(|id| {
                            ITERATORS.with(|it| {
                                it.borrow()
                                    .get(&(id as u64))
                                    .map(|i| i.exhausted)
                                    .unwrap_or(true)
                            })
                        })
                        .unwrap_or(true);
                    if exhausted {
                        break;
                    }
                }
                acc.push(v);
            }
            mb_iter_release(handle);
            acc
        }
    };
    for &v in &items {
        unsafe {
            super::rc::retain_if_ptr(v);
        }
    }
    let iter = MbIterator {
        kind: IterKind::Cycle { items, pos: 0 },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create an enumerate iterator.
pub fn mb_enumerate(iterable: MbValue, start: MbValue) -> MbValue {
    let inner_id = mb_iter(iterable);
    let start_count = start.as_int().unwrap_or(0);

    if let Some(inner_id_val) = inner_id.as_int() {
        // Keep the inner registered under its own id; the Enumerate kind drives
        // it out-of-line via `mb_next` (so a Map/Filter inner advances correctly).
        let inner_exists =
            ITERATORS.with(|iters| iters.borrow().contains_key(&(inner_id_val as u64)));
        if inner_exists {
            let iter = MbIterator {
                kind: IterKind::Enumerate {
                    inner_id: inner_id_val as u64,
                    count: start_count,
                },
                index: 0,
                exhausted: false,
                peeked: None,
            };
            let id = alloc_iter_id();
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(id, iter);
            });
            return MbValue::from_int(id as i64);
        }
    }
    MbValue::none()
}

/// Create a reversed iterator.
pub fn mb_reversed(seq: MbValue) -> MbValue {
    // Iterator handle path: support reversed(range(...)) and other iterators
    // whose full contents can be materialized.
    if let Some(id) = seq.as_int() {
        let is_iter = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
        if is_iter {
            let items: Vec<MbValue> = ITERATORS.with(|iters| {
                let borrowed = iters.borrow();
                let it = borrowed.get(&(id as u64)).unwrap();
                if let IterKind::Range {
                    current,
                    stop,
                    step,
                } = it.kind
                {
                    let mut out = Vec::new();
                    let mut c = current;
                    if step > 0 {
                        while c < stop {
                            out.push(MbValue::from_int(c));
                            c += step;
                        }
                    } else if step < 0 {
                        while c > stop {
                            out.push(MbValue::from_int(c));
                            c += step;
                        }
                    }
                    out.reverse();
                    out
                } else {
                    Vec::new()
                }
            });
            ITERATORS.with(|iters| {
                iters.borrow_mut().remove(&(id as u64));
            });
            let iter = MbIterator {
                kind: IterKind::Reversed { items, index: 0 },
                index: 0,
                exhausted: false,
                peeked: None,
            };
            let new_id = alloc_iter_id();
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(new_id, iter);
            });
            return MbValue::from_int(new_id as i64);
        }
    }
    if let Some(ptr) = seq.as_ptr() {
        unsafe {
            let items: Vec<MbValue> = match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    items.iter().rev().copied().collect()
                }
                ObjData::Tuple(items) => items.iter().rev().copied().collect(),
                ObjData::Str(ref s) => s
                    .chars()
                    .rev()
                    .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                    .collect(),
                ObjData::Bytes(ref data) => data
                    .iter()
                    .rev()
                    .map(|&b| MbValue::from_int(b as i64))
                    .collect(),
                // reversed(dict) yields keys in reverse insertion order (3.8+).
                ObjData::Dict(ref lock) => lock
                    .read()
                    .unwrap()
                    .keys()
                    .rev()
                    .map(|k| super::dict_ops::dict_key_to_mbvalue(k))
                    .collect(),
                ObjData::Instance { ref class_name, .. } => {
                    // User __reversed__ dunder returns its own iterator;
                    // without one, reversed(obj) is a TypeError, not None.
                    let cls = class_name.clone();
                    let m = super::class::lookup_method(&cls, "__reversed__");
                    if !m.is_none() {
                        let method =
                            MbValue::from_ptr(MbObject::new_str("__reversed__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                        return super::class::mb_call_method(seq, method, args);
                    }
                    // Sequence-protocol fallback: a class with __len__ AND
                    // __getitem__ is reversible (yield obj[len-1] .. obj[0]),
                    // matching CPython's reversed() even without __reversed__.
                    let has_len = !super::class::lookup_method(&cls, "__len__").is_none();
                    let has_getitem = !super::class::lookup_method(&cls, "__getitem__").is_none();
                    if has_len && has_getitem {
                        let n = super::builtins::mb_len(seq).as_int().unwrap_or(0).max(0);
                        let mut items: Vec<MbValue> = Vec::with_capacity(n as usize);
                        for i in (0..n).rev() {
                            items.push(super::class::mb_obj_getitem(seq, MbValue::from_int(i)));
                            if super::exception::current_exception_type().is_some() {
                                return MbValue::none();
                            }
                        }
                        items
                    } else {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'{cls}' object is not reversible"
                            ))),
                        );
                        return MbValue::none();
                    }
                }
                _ => return MbValue::none(),
            };
            let iter = MbIterator {
                kind: IterKind::Reversed { items, index: 0 },
                index: 0,
                exhausted: false,
                peeked: None,
            };
            let id = alloc_iter_id();
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(id, iter);
            });
            MbValue::from_int(id as i64)
        }
    } else {
        MbValue::none()
    }
}

/// Create a lazy map iterator: `map(func, iterable)` → iterator handle.
///
/// Returns an `IterKind::Map` handle that calls `func` on each element of
/// `iterable` on demand (lazy), matching CPython's `map` object semantics.
///
/// Special case: `map(None, iterable)` raises `TypeError` per Python 3
/// (unlike `filter(None, iterable)` which is valid and keeps truthy values).
pub fn mb_map_iter(func: MbValue, iterable: MbValue) -> MbValue {
    // map(None, …) → TypeError: 'NoneType' object is not callable
    if func.is_none() {
        super::exception::mb_raise(
            MbValue::from_ptr(super::rc::MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(super::rc::MbObject::new_str(
                "'NoneType' object is not callable".to_string(),
            )),
        );
        return MbValue::none();
    }

    let inner_handle = mb_iter(iterable);
    if let Some(inner_id_val) = inner_handle.as_int() {
        // The inner iterator stays in ITERATORS under its own ID.
        // `IterKind::Map` stores only the inner_id (not the embedded iterator)
        // so that the out-of-line advance can release the ITERATORS borrow
        // before calling `func` (which may re-enter ITERATORS via mb_hasattr).
        let inner_exists =
            ITERATORS.with(|iters| iters.borrow().contains_key(&(inner_id_val as u64)));
        if inner_exists {
            let id = alloc_iter_id();
            let iter = MbIterator {
                kind: IterKind::Map {
                    func,
                    inner_id: inner_id_val as u64,
                },
                index: 0,
                exhausted: false,
                peeked: None,
            };
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(id, iter);
            });
            return MbValue::from_int(id as i64);
        }
    }
    MbValue::none()
}

/// Create a lazy filter iterator: `filter(func, iterable)` → iterator handle.
///
/// Returns an `IterKind::Filter` handle. When `func` is `None`, keeps truthy
/// elements (matching CPython's `filter(None, iterable)` behaviour).
pub fn mb_filter_iter(func: MbValue, iterable: MbValue) -> MbValue {
    let inner_handle = mb_iter(iterable);
    if let Some(inner_id_val) = inner_handle.as_int() {
        let inner_exists =
            ITERATORS.with(|iters| iters.borrow().contains_key(&(inner_id_val as u64)));
        if inner_exists {
            let id = alloc_iter_id();
            let iter = MbIterator {
                kind: IterKind::Filter {
                    func,
                    inner_id: inner_id_val as u64,
                },
                index: 0,
                exhausted: false,
                peeked: None,
            };
            ITERATORS.with(|iters| {
                iters.borrow_mut().insert(id, iter);
            });
            return MbValue::from_int(id as i64);
        }
    }
    MbValue::none()
}

/// Create a map iterator from a function and multiple iterables: map(func, iter1, iter2, ...).
///
/// `iterables` is a List MbValue containing all iterable arguments.
/// Returns an iterator handle (MapN kind). Each inner iterable is registered
/// as a separate ITERATORS entry (same reentrancy rationale as Map/Filter).
pub fn mb_map_n(func: MbValue, iterables: MbValue) -> MbValue {
    if func.is_none() {
        super::exception::mb_raise(
            MbValue::from_ptr(super::rc::MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(super::rc::MbObject::new_str(
                "'NoneType' object is not callable".to_string(),
            )),
        );
        return MbValue::none();
    }
    let items: Vec<MbValue> = if let Some(ptr) = iterables.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().to_vec(),
                _ => return MbValue::none(),
            }
        }
    } else {
        return MbValue::none();
    };

    let mut inner_ids: Vec<u64> = Vec::with_capacity(items.len());
    for item in &items {
        let inner_handle = mb_iter(*item);
        if let Some(inner_id_val) = inner_handle.as_int() {
            let inner_exists =
                ITERATORS.with(|iters| iters.borrow().contains_key(&(inner_id_val as u64)));
            if inner_exists {
                inner_ids.push(inner_id_val as u64);
                continue;
            }
        }
        // Failed to create inner iterator; release already-created inner iters.
        for id in &inner_ids {
            let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(id));
            if let Some(inner) = removed {
                release_iter(&inner);
            }
        }
        return MbValue::none();
    }

    let id = alloc_iter_id();
    let iter = MbIterator {
        kind: IterKind::MapN { func, inner_ids },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a zip iterator from two iterables.
pub fn mb_zip(a: MbValue, b: MbValue) -> MbValue {
    let id_a = mb_iter(a);
    let id_b = mb_iter(b);
    if id_a.is_none() || id_b.is_none() {
        return MbValue::none();
    }

    // Keep each inner registered under its own id; the Zip kind drives them
    // out-of-line via `mb_next` so Map/Filter inners advance correctly.
    let mut inner_ids = Vec::new();
    for id_val in [id_a, id_b] {
        if let Some(id) = id_val.as_int() {
            let exists = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
            if exists {
                inner_ids.push(id as u64);
            }
        }
    }
    if inner_ids.len() != 2 {
        return MbValue::none();
    }

    let iter = MbIterator {
        kind: IterKind::Zip {
            inner_ids,
            strict: false,
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a zip iterator from N iterables (N >= 3).
/// Takes a list of iterables packed by the lowerer.
pub fn mb_zip_n(iterables: MbValue) -> MbValue {
    let iter_list = if let Some(ptr) = iterables.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => return MbValue::none(),
            }
        }
    } else {
        return MbValue::none();
    };

    let mut inner_ids = Vec::new();
    for iterable in iter_list {
        let id_val = mb_iter(iterable);
        if let Some(id) = id_val.as_int() {
            let exists = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
            if exists {
                inner_ids.push(id as u64);
            }
        }
    }
    if inner_ids.is_empty() {
        return MbValue::none();
    }

    let iter = MbIterator {
        kind: IterKind::Zip {
            inner_ids,
            strict: false,
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

/// Create a zip iterator with optional strict-length validation.
///
/// Per PEP 618 (Py3.10+), `zip(*iters, strict=True)` raises `ValueError`
/// when the inputs are not all the same length. The lowerer routes the
/// keyword form here; the trailing `strict` is plumbed into the kind so
/// `advance_iter` can do the per-step exhaustion check and emit
/// CPython's "argument N is shorter/longer than argument 1" message.
pub fn mb_zip_strict(iterables: MbValue, strict: MbValue) -> MbValue {
    let strict_bool = strict.as_bool().unwrap_or(false);
    let iter_list = if let Some(ptr) = iterables.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => return MbValue::none(),
            }
        }
    } else {
        return MbValue::none();
    };

    let mut inner_ids = Vec::new();
    for iterable in iter_list {
        let id_val = mb_iter(iterable);
        if let Some(id) = id_val.as_int() {
            let exists = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
            if exists {
                inner_ids.push(id as u64);
            }
        }
    }

    let iter = MbIterator {
        kind: IterKind::Zip {
            inner_ids,
            strict: strict_bool,
        },
        index: 0,
        exhausted: false,
        peeked: None,
    };
    let id = alloc_iter_id();
    ITERATORS.with(|iters| {
        iters.borrow_mut().insert(id, iter);
    });
    MbValue::from_int(id as i64)
}

// ── Iterator Advancement ──

/// Get the next value from an iterator or generator. Returns None when exhausted.
pub fn mb_next(iter_handle: MbValue) -> MbValue {
    // Safepoint poll at loop backedge (R4)
    super::gc::gc_safepoint();
    if let Some(id) = iter_handle.as_int() {
        let id_u = id as u64;
        // Generator-iterator fast path. Per-yield in `for x in gen()`, the
        // slow path below pays 3 ITERATORS hash lookups (is_iter probe +
        // advance_callable_if_applicable's borrow_mut + advance_generator_if_applicable's
        // borrow_mut). Collapse to one borrow_mut for the steady-state
        // generator case: peek-take if a value is cached (mb_has_next path),
        // otherwise extract the gen handle + clear cache flags + drop the
        // borrow before resuming.
        enum GenFast {
            Peeked(MbValue),
            Resume(MbValue),
            NotGen,
        }
        let action = ITERATORS.with(|iters| {
            let mut iters = iters.borrow_mut();
            let it = match iters.get_mut(&id_u) {
                Some(e) => e,
                None => return GenFast::NotGen,
            };
            match &it.kind {
                IterKind::Generator(h) => {
                    if it.exhausted {
                        GenFast::Peeked(MbValue::none())
                    } else if let Some(p) = it.peeked.take() {
                        GenFast::Peeked(p)
                    } else {
                        GenFast::Resume(*h)
                    }
                }
                _ => GenFast::NotGen,
            }
        });
        match action {
            GenFast::Peeked(val) => {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            GenFast::Resume(gen_handle) => {
                check_stop_iteration();
                let val = super::generator::mb_generator_next(gen_handle);
                if check_stop_iteration() {
                    super::exception::mb_clear_exception();
                    ITERATORS.with(|iters| {
                        if let Some(iter) = iters.borrow_mut().get_mut(&id_u) {
                            iter.exhausted = true;
                        }
                    });
                    return MbValue::none();
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            GenFast::NotGen => {} // fall through to existing slow path
        }
        // Check if it's a known iterator first
        let is_iter = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
        if is_iter {
            // Out-of-line advancement for iterators whose bodies may access
            // ITERATORS (callable, generator, map, filter). Prevents reentrant
            // borrow panic when user code (func/predicate) calls mb_is_iterator_handle.
            if let Some(val) = advance_callable_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // (Generator iterators are handled by the GenFast fast path at
            // the top of mb_next; advance_generator_if_applicable here was
            // dead post-fire-42.)
            if let Some(val) = advance_userdefined_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // Map/Filter: out-of-line advance (func/predicate is user JIT code).
            if let Some(val) = advance_map_filter_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // MapN: out-of-line advance (func is user JIT code; multiple inners).
            if let Some(val) = advance_map_n_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // Enumerate/Zip: out-of-line advance — they drive their inner(s) via
            // mb_next, which may be Map/Filter/MapN that re-enter ITERATORS.
            if let Some(val) = advance_enumerate_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            if let Some(val) = advance_zip_if_applicable(id as u64) {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            let val = ITERATORS.with(|iters| {
                let mut iters = iters.borrow_mut();
                if let Some(iter) = iters.get_mut(&(id as u64)) {
                    if iter.exhausted {
                        return MbValue::none();
                    }
                    if let Some(peeked) = iter.peeked.take() {
                        return peeked;
                    }
                    advance_iter(iter)
                } else {
                    MbValue::none()
                }
            });
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
        // Check if it's a generator handle. This branch fires when the caller
        // passes a bare generator handle (e.g. `next(gen)` where gen has not
        // been wrapped by mb_iter). Mirror the StopIteration-handling dance
        // from `advance_generator_if_applicable` so that an exhausted
        // generator does not leak a phantom StopIteration past the call site
        // — without this, the next module-level statement would see a stale
        // CURRENT_EXCEPTION and abort. (§4 step (c) precondition.)
        if super::generator::is_known_generator(iter_handle) {
            check_stop_iteration(); // clear stale flag
            let val = super::generator::mb_generator_next(iter_handle);
            if check_stop_iteration() {
                super::exception::mb_clear_exception();
                return MbValue::none();
            }
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
        MbValue::none()
    } else if let Some(ptr) = iter_handle.as_ptr() {
        // Custom iterator instance: dispatch to __next__.
        unsafe {
            if matches!(&(*ptr).data, ObjData::Instance { .. }) {
                let next_method = class::mb_lookup_dunder(
                    iter_handle,
                    MbValue::from_ptr(MbObject::new_str("__next__".into())),
                );
                if !next_method.is_none() {
                    return class::mb_call_method1(next_method, iter_handle);
                }
            }
        }
        MbValue::none()
    } else {
        MbValue::none()
    }
}

/// next(iterator, default) — return default when iterator is exhausted.
pub fn mb_next_default(iter_handle: MbValue, default: MbValue) -> MbValue {
    super::gc::gc_safepoint();
    if let Some(id) = iter_handle.as_int() {
        let handled = ITERATORS.with(|iters| {
            let mut iters = iters.borrow_mut();
            if let Some(iter) = iters.get_mut(&(id as u64)) {
                if iter.exhausted {
                    // Retain: JIT releases both default arg VReg and result VReg.
                    unsafe { super::rc::retain_if_ptr(default) };
                    return Some(default);
                }
                if let Some(peeked) = iter.peeked.take() {
                    return Some(peeked);
                }
                let val = advance_iter(iter);
                // If iterator just became exhausted, return default
                if iter.exhausted {
                    unsafe { super::rc::retain_if_ptr(default) };
                    Some(default)
                } else {
                    Some(val)
                }
            } else {
                // Int that is not a known handle iterator (e.g. a bare generator
                // handle) — fall through to the generic __next__ path below.
                None
            }
        });
        if let Some(v) = handled {
            return v;
        }
    }
    // Generic path: a raw user-defined iterator instance, generator handle, or
    // callable iterator passed directly to `next(it, default)`. Delegate to
    // mb_next_raise (which dispatches to __next__ / generator-next) and swallow
    // StopIteration → default, matching CPython's two-arg next().
    let val = mb_next_raise(iter_handle);
    if super::exception::current_exception_type().as_deref() == Some("StopIteration") {
        super::exception::mb_clear_exception();
        unsafe { super::rc::retain_if_ptr(default) };
        return default;
    }
    val
}

/// Single-call iterator advance: returns the next value, or
/// `MbValue::stop_iter_sentinel()` when the iterator is exhausted.
///
/// Used by for-loop and comprehension lowering to halve the FFI thunk
/// count per yield (vs the older `mb_has_next` + `mb_next` pair).
/// The generator-iter fast path uses a single `ITERATORS.borrow_mut()`
/// — same shape as fire-42's mb_next fast path. Non-generator iterators
/// fall back to the existing `mb_has_next` + `mb_next` pair (still
/// correct, but no FFI win on that path).
///
/// Critical: `None` is a valid yielded value, so the sentinel must be
/// distinct from `None`. See `value.rs::TAG_STOP_ITER`.
pub fn mb_next_or_stop(iter_handle: MbValue) -> MbValue {
    super::gc::gc_safepoint();
    if let Some(id) = iter_handle.as_int() {
        let id_u = id as u64;
        enum GenFast {
            Done,
            Peeked(MbValue),
            Resume(MbValue),
            NotGen,
        }
        let action = ITERATORS.with(|iters| {
            let mut iters = iters.borrow_mut();
            let it = match iters.get_mut(&id_u) {
                Some(e) => e,
                None => return GenFast::NotGen,
            };
            match &it.kind {
                IterKind::Generator(h) => {
                    if it.exhausted {
                        GenFast::Done
                    } else if let Some(p) = it.peeked.take() {
                        GenFast::Peeked(p)
                    } else {
                        GenFast::Resume(*h)
                    }
                }
                _ => GenFast::NotGen,
            }
        });
        match action {
            GenFast::Done => return MbValue::stop_iter_sentinel(),
            GenFast::Peeked(val) => {
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            GenFast::Resume(gen_handle) => {
                check_stop_iteration();
                let val = super::generator::mb_generator_next(gen_handle);
                if check_stop_iteration() {
                    super::exception::mb_clear_exception();
                    ITERATORS.with(|iters| {
                        if let Some(iter) = iters.borrow_mut().get_mut(&id_u) {
                            iter.exhausted = true;
                        }
                    });
                    return MbValue::stop_iter_sentinel();
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            GenFast::NotGen => {} // fall through
        }
    }
    // Slow path: emulate has_next + next. Correct for callable / userdefined /
    // ptr-based iterators. Same FFI cost as the legacy pair, so no regression.
    if mb_has_next(iter_handle).as_bool() == Some(false) {
        return MbValue::stop_iter_sentinel();
    }
    mb_next(iter_handle)
}

/// Identity check for the `mb_next_or_stop` sentinel. Returns a bool MbValue
/// so the for-loop's `Branch` terminator can consume it directly.
#[inline]
pub fn mb_is_stop_iter(v: MbValue) -> MbValue {
    MbValue::from_bool(v.is_stop_iter_sentinel())
}

/// next(iterator) — raise StopIteration when iterator is exhausted.
/// Used for direct `next(it)` calls (not in for-loop lowering which uses mb_next).
pub fn mb_next_raise(iter_handle: MbValue) -> MbValue {
    super::gc::gc_safepoint();
    if let Some(id) = iter_handle.as_int() {
        let is_iter = ITERATORS.with(|iters| iters.borrow().contains_key(&(id as u64)));
        if is_iter {
            // Out-of-line: callable and generator iterators
            if let Some(val) = advance_callable_if_applicable(id as u64) {
                if val.is_none() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            if let Some(val) = advance_generator_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(true)
                });
                if exhausted {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            if let Some(val) = advance_userdefined_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(true)
                });
                if exhausted {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // Map/Filter iterators: out-of-line advance (same reentrancy
            // rationale as callable/userdefined above).
            if let Some(val) = advance_map_filter_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(false)
                });
                if exhausted && val.is_none() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // MapN iterators: out-of-line advance (same reentrancy rationale).
            if let Some(val) = advance_map_n_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(false)
                });
                if exhausted && val.is_none() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            // Enumerate/Zip iterators: out-of-line advance (same reentrancy
            // rationale — inner(s) may be Map/Filter/MapN that re-enter ITERATORS).
            if let Some(val) = advance_enumerate_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(false)
                });
                if exhausted && val.is_none() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            if let Some(val) = advance_zip_if_applicable(id as u64) {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(false)
                });
                if exhausted && val.is_none() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                }
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                return val;
            }
            let val = ITERATORS.with(|iters| {
                let mut iters = iters.borrow_mut();
                if let Some(iter) = iters.get_mut(&(id as u64)) {
                    if iter.exhausted {
                        super::exception::set_current_exception(
                            super::exception::MbException::new("StopIteration", ""),
                        );
                        return MbValue::none();
                    }
                    if let Some(peeked) = iter.peeked.take() {
                        return peeked;
                    }
                    let val = advance_iter(iter);
                    if iter.exhausted {
                        super::exception::set_current_exception(
                            super::exception::MbException::new("StopIteration", ""),
                        );
                    }
                    val
                } else {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "StopIteration",
                        "",
                    ));
                    MbValue::none()
                }
            });
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
        if super::generator::is_known_generator(iter_handle) {
            let val = super::generator::mb_generator_next(iter_handle);
            if check_stop_iteration() {
                super::exception::set_current_exception(super::exception::MbException::new(
                    "StopIteration",
                    "",
                ));
            }
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            return val;
        }
        super::exception::set_current_exception(super::exception::MbException::new(
            "TypeError",
            "object is not an iterator",
        ));
        MbValue::none()
    } else if let Some(ptr) = iter_handle.as_ptr() {
        // Custom iterator instance: dispatch to __next__.
        // If __next__ raises StopIteration, the exception is already set via
        // the normal raise path — we just propagate the returned value (None).
        unsafe {
            if matches!(&(*ptr).data, ObjData::Instance { .. }) {
                let next_method = class::mb_lookup_dunder(
                    iter_handle,
                    MbValue::from_ptr(MbObject::new_str("__next__".into())),
                );
                if !next_method.is_none() {
                    return class::mb_call_method1(next_method, iter_handle);
                }
            }
        }
        super::exception::set_current_exception(super::exception::MbException::new(
            "TypeError",
            "object is not an iterator",
        ));
        MbValue::none()
    } else {
        super::exception::set_current_exception(super::exception::MbException::new(
            "TypeError",
            "object is not an iterator",
        ));
        MbValue::none()
    }
}

/// Check if an iterator has more values.
///
/// Uses a peeked-value cache: advances the iterator internally and stores the
/// result so the subsequent `mb_next` call can return it without re-advancing.
/// This makes the "check-then-next" for-loop pattern correct for ALL iterator
/// kinds (list, range, generator, zip, enumerate, …).
pub fn mb_has_next(iter_handle: MbValue) -> MbValue {
    if let Some(id) = iter_handle.as_int() {
        let id_u = id as u64;
        // Generator-iterator fast path. The slow-path code below does up to 4
        // separate ITERATORS borrows per call (is_callable probe, is_gen
        // probe, short-circuit probe, write-peeked). Collapse to a single
        // borrow that returns either the short-circuit answer or the gen
        // handle to advance. Advancement runs with the borrow released
        // (resume_generator may reenter ITERATORS).
        enum GenHas {
            False,
            True,
            Advance(MbValue),
            NotGen,
        }
        let action = ITERATORS.with(|iters| {
            let it = iters.borrow();
            let entry = match it.get(&id_u) {
                Some(e) => e,
                None => return GenHas::NotGen,
            };
            if !matches!(&entry.kind, IterKind::Generator(_)) {
                return GenHas::NotGen;
            }
            if entry.exhausted {
                return GenHas::False;
            }
            if entry.peeked.is_some() {
                return GenHas::True;
            }
            match &entry.kind {
                IterKind::Generator(h) => GenHas::Advance(*h),
                _ => GenHas::NotGen,
            }
        });
        match action {
            GenHas::False => return MbValue::from_bool(false),
            GenHas::True => return MbValue::from_bool(true),
            GenHas::Advance(gen_handle) => {
                check_stop_iteration();
                let val = super::generator::mb_generator_next(gen_handle);
                if check_stop_iteration() {
                    super::exception::mb_clear_exception();
                    return ITERATORS.with(|iters| {
                        if let Some(iter) = iters.borrow_mut().get_mut(&id_u) {
                            iter.exhausted = true;
                        }
                        MbValue::from_bool(false)
                    });
                }
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&id_u) {
                        iter.peeked = Some(val);
                        MbValue::from_bool(true)
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
            GenHas::NotGen => {} // fall through
        }
        // Callable-sentinel iterators: peek by calling the callable out-of-line
        // (the callable may itself touch ITERATORS, see advance_callable_if_applicable).
        {
            let is_callable = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&(id as u64))
                    .map(|it| matches!(it.kind, IterKind::Callable { .. }))
                    .unwrap_or(false)
            });
            if is_callable {
                // If we already have a peeked value or are exhausted, answer
                // directly without re-invoking the callable.
                let short = ITERATORS.with(|iters| {
                    let it = iters.borrow();
                    it.get(&(id as u64))
                        .map(|i| {
                            if i.exhausted {
                                Some(MbValue::from_bool(false))
                            } else if i.peeked.is_some() {
                                Some(MbValue::from_bool(true))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                });
                if let Some(v) = short {
                    return v;
                }
                // Otherwise advance once and cache as peeked.
                let val = advance_callable_if_applicable(id as u64).unwrap_or_else(MbValue::none);
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&(id as u64)) {
                        if iter.exhausted {
                            MbValue::from_bool(false)
                        } else {
                            iter.peeked = Some(val);
                            MbValue::from_bool(true)
                        }
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
        }
        // (Generator iterators are handled by the GenHas fast path above; the
        // matching kind-dispatch block here was removed as dead code.)
        // Map/Filter iterators: advance out-of-line to avoid reentrant borrow.
        // (func/predicate is JIT-compiled user code that may access ITERATORS
        // via mb_hasattr → mb_is_iterator_handle.)
        {
            let is_mapfilter = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&(id as u64))
                    .map(|it| matches!(it.kind, IterKind::Map { .. } | IterKind::Filter { .. }))
                    .unwrap_or(false)
            });
            if is_mapfilter {
                let short = ITERATORS.with(|iters| {
                    let it = iters.borrow();
                    it.get(&(id as u64))
                        .map(|i| {
                            if i.exhausted {
                                Some(MbValue::from_bool(false))
                            } else if i.peeked.is_some() {
                                Some(MbValue::from_bool(true))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                });
                if let Some(v) = short {
                    return v;
                }
                let val = advance_map_filter_if_applicable(id as u64).unwrap_or_else(MbValue::none);
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&(id as u64)) {
                        if iter.exhausted {
                            MbValue::from_bool(false)
                        } else {
                            iter.peeked = Some(val);
                            MbValue::from_bool(true)
                        }
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
        }
        // MapN iterators: advance out-of-line (func is user JIT code, multiple inners).
        {
            let is_mapn = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&(id as u64))
                    .map(|it| matches!(it.kind, IterKind::MapN { .. }))
                    .unwrap_or(false)
            });
            if is_mapn {
                let short = ITERATORS.with(|iters| {
                    let it = iters.borrow();
                    it.get(&(id as u64))
                        .map(|i| {
                            if i.exhausted {
                                Some(MbValue::from_bool(false))
                            } else if i.peeked.is_some() {
                                Some(MbValue::from_bool(true))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                });
                if let Some(v) = short {
                    return v;
                }
                let val = advance_map_n_if_applicable(id as u64).unwrap_or_else(MbValue::none);
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&(id as u64)) {
                        if iter.exhausted {
                            MbValue::from_bool(false)
                        } else {
                            iter.peeked = Some(val);
                            MbValue::from_bool(true)
                        }
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
        }
        // Enumerate/Zip iterators: advance out-of-line — inner(s) may be
        // Map/Filter/MapN that re-enter ITERATORS via user code.
        {
            let is_enum_zip = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&(id as u64))
                    .map(|it| matches!(it.kind, IterKind::Enumerate { .. } | IterKind::Zip { .. }))
                    .unwrap_or(false)
            });
            if is_enum_zip {
                let short = ITERATORS.with(|iters| {
                    let it = iters.borrow();
                    it.get(&(id as u64))
                        .map(|i| {
                            if i.exhausted {
                                Some(MbValue::from_bool(false))
                            } else if i.peeked.is_some() {
                                Some(MbValue::from_bool(true))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                });
                if let Some(v) = short {
                    return v;
                }
                let val = advance_enumerate_if_applicable(id as u64)
                    .or_else(|| advance_zip_if_applicable(id as u64))
                    .unwrap_or_else(MbValue::none);
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&(id as u64)) {
                        if iter.exhausted {
                            MbValue::from_bool(false)
                        } else {
                            iter.peeked = Some(val);
                            MbValue::from_bool(true)
                        }
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
        }
        // UserDefined iterators: advance out-of-line to prevent reentrant borrow.
        {
            let is_userdefined = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&(id as u64))
                    .map(|it| matches!(it.kind, IterKind::UserDefined(_)))
                    .unwrap_or(false)
            });
            if is_userdefined {
                let short = ITERATORS.with(|iters| {
                    let it = iters.borrow();
                    it.get(&(id as u64))
                        .map(|i| {
                            if i.exhausted {
                                Some(MbValue::from_bool(false))
                            } else if i.peeked.is_some() {
                                Some(MbValue::from_bool(true))
                            } else {
                                None
                            }
                        })
                        .unwrap_or(None)
                });
                if let Some(v) = short {
                    return v;
                }
                let val =
                    advance_userdefined_if_applicable(id as u64).unwrap_or_else(MbValue::none);
                return ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&(id as u64)) {
                        if iter.exhausted {
                            MbValue::from_bool(false)
                        } else {
                            iter.peeked = Some(val);
                            MbValue::from_bool(true)
                        }
                    } else {
                        MbValue::from_bool(false)
                    }
                });
            }
        }
        ITERATORS.with(|iters| {
            let mut iters = iters.borrow_mut();
            if let Some(iter) = iters.get_mut(&(id as u64)) {
                if iter.exhausted {
                    return MbValue::from_bool(false);
                }
                if iter.peeked.is_some() {
                    return MbValue::from_bool(true);
                }
                // Range fast path: check bounds without advancing.
                if let IterKind::Range {
                    current,
                    stop,
                    step,
                } = &iter.kind
                {
                    let has = (*step > 0 && *current < *stop) || (*step < 0 && *current > *stop);
                    if !has {
                        iter.exhausted = true;
                    }
                    return MbValue::from_bool(has);
                }
                let val = advance_iter(iter);
                if iter.exhausted {
                    return MbValue::from_bool(false);
                }
                iter.peeked = Some(val);
                MbValue::from_bool(true)
            } else {
                if super::generator::is_known_generator(iter_handle) {
                    return super::generator::mb_generator_is_exhausted(iter_handle)
                        .as_bool()
                        .map(|exh| MbValue::from_bool(!exh))
                        .unwrap_or(MbValue::from_bool(false));
                }
                MbValue::from_bool(false)
            }
        })
    } else {
        MbValue::from_bool(false)
    }
}

/// Release an iterator's backing refcounts. Recurses through composite
/// iterators (Enumerate, Zip, Map, Filter) to release inner iterators' refs.
fn release_iter(iter: &MbIterator) {
    match &iter.kind {
        // UserDefined: released only when not exhausted — the exhausted path
        // already released in advance_userdefined.
        IterKind::UserDefined(iter_obj) => {
            if !iter.exhausted {
                unsafe {
                    super::rc::release_if_ptr(*iter_obj);
                }
            }
        }
        // List/Tuple: iterator holds a reference to its backing object (see
        // `mb_iter`); release that reference on drop so the object can be
        // freed.
        IterKind::List(v) | IterKind::Tuple(v) => unsafe {
            super::rc::release_if_ptr(*v);
        },
        // Enumerate/Zip: inner iterator(s) are separate ITERATORS entries
        // identified by id. Release each by removing and dropping via release_iter.
        IterKind::Enumerate { inner_id, .. } => {
            let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(inner_id));
            if let Some(inner) = removed {
                release_iter(&inner);
            }
        }
        IterKind::Zip { inner_ids, .. } => {
            for inner_id in inner_ids.iter() {
                let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(inner_id));
                if let Some(inner) = removed {
                    release_iter(&inner);
                }
            }
        }
        // Map/Filter: inner iterator is a separate ITERATORS entry identified
        // by inner_id. Release it by removing and dropping via release_iter.
        IterKind::Map { inner_id, .. } | IterKind::Filter { inner_id, .. } => {
            let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(inner_id));
            if let Some(inner) = removed {
                release_iter(&inner);
            }
        }
        // MapN: all inner iterators are separate ITERATORS entries.
        IterKind::MapN { inner_ids, .. } => {
            for inner_id in inner_ids.iter() {
                let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(inner_id));
                if let Some(inner) = removed {
                    release_iter(&inner);
                }
            }
        }
        _ => {}
    }
}

/// Release an iterator (called when for-loop exits).
pub fn mb_iter_release(iter_handle: MbValue) {
    if let Some(id) = iter_handle.as_int() {
        let removed = ITERATORS.with(|iters| iters.borrow_mut().remove(&(id as u64)));
        if let Some(iter) = removed {
            release_iter(&iter);
        }
    }
}

/// Advance a Callable-sentinel iterator out-of-line. Returns `Some(value)` if
/// the iterator is Callable (caller should treat this as the next result),
/// or `None` if the iterator is of a different kind and the caller should
/// fall back to the normal in-line advancement path.
///
/// Callable iterators cannot be advanced while holding the ITERATORS borrow
/// because the user-provided callable may touch other iterators (e.g. via
/// `next(...)`), which would cause a reentrant `borrow_mut` panic.
fn advance_callable_if_applicable(id: u64) -> Option<MbValue> {
    // First check the kind and consume any peeked value under a short borrow.
    let info = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match iter.kind {
            IterKind::Callable { func, sentinel } => {
                if iter.exhausted {
                    Some((None, Some(MbValue::none())))
                } else if let Some(p) = iter.peeked.take() {
                    Some((None, Some(p)))
                } else {
                    Some((Some((func, sentinel)), None))
                }
            }
            _ => None,
        }
    })?;
    if let (_, Some(val)) = info {
        return Some(val);
    }
    let (func, sentinel) = info.0.expect("checked by info.1 branch above");
    // Invoke the callable with the borrow released.
    let result = class::mb_call0(func);
    let eq = super::builtins::mb_eq(result, sentinel);
    if eq.as_bool().unwrap_or(false) {
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        return Some(MbValue::none());
    }
    Some(result)
}

/// Advance a Generator iterator out-of-line. Returns `Some(value)` if the
/// iterator wraps a generator, `None` if not a generator kind.
///
/// Generator iterators must NOT hold the ITERATORS borrow during resume
/// because the generator body runs on the same thread (coroutine) and may
/// create/access iterators itself (e.g. `for j in range(n)` inside the
/// generator body), which would cause a reentrant `borrow_mut` panic.
fn advance_generator_if_applicable(id: u64) -> Option<MbValue> {
    let info = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match iter.kind {
            IterKind::Generator(gen_handle) => {
                if iter.exhausted {
                    Some((None, Some(MbValue::none()), true))
                } else if let Some(p) = iter.peeked.take() {
                    Some((None, Some(p), false))
                } else {
                    Some((Some(gen_handle), None, false))
                }
            }
            _ => None,
        }
    })?;

    // Short-circuit for exhausted or peeked
    if let (_, Some(val), exhausted) = info {
        if exhausted {
            return Some(MbValue::none());
        }
        return Some(val);
    }

    let gen_handle = info.0.expect("checked above");

    // Resume generator with ITERATORS borrow released
    check_stop_iteration(); // clear stale flag
    let val = super::generator::mb_generator_next(gen_handle);

    if check_stop_iteration() {
        // Generator exhausted: raise_stop_iteration sets both the local
        // STOP_ITERATION flag (cleared by check_stop_iteration above) AND
        // CURRENT_EXCEPTION. The flag is the iterator-protocol signal we
        // want; the exception is bookkeeping that must NOT escape past the
        // for-loop boundary, otherwise the next module-level statement sees
        // a phantom StopIteration and the program aborts. Clear it.
        super::exception::mb_clear_exception();
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        return Some(MbValue::none());
    }

    Some(val)
}

/// Advance a UserDefined iterator out-of-line. Returns `Some(value)` if the
/// iterator wraps a user-defined object (Instance with __next__), `None` if not.
///
/// UserDefined iterators must NOT hold the ITERATORS borrow during the __next__
/// call because the user method body runs JIT-compiled code that may:
///   - call gc_safepoint() → GC runs → potential reentrant borrow
///   - raise StopIteration (sets STOP_ITERATION flag)
///   - access other iterators (nested for loops inside __next__)
/// All of this would cause a reentrant `borrow_mut` panic if ITERATORS were held.
fn advance_userdefined_if_applicable(id: u64) -> Option<MbValue> {
    // Extract iter_obj and handle peeked/exhausted under a short borrow.
    let info = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match iter.kind {
            IterKind::UserDefined(iter_obj) => {
                if iter.exhausted {
                    Some((None, Some(MbValue::none())))
                } else if let Some(p) = iter.peeked.take() {
                    Some((Some(iter_obj), Some(p)))
                } else {
                    Some((Some(iter_obj), None))
                }
            }
            _ => None,
        }
    })?;

    // If exhausted, iter_obj is None; if peeked, return the peeked value.
    let iter_obj = match info {
        (None, Some(val)) => return Some(val), // exhausted → None
        (Some(_obj), Some(peeked)) => return Some(peeked), // peeked value
        (Some(obj), None) => obj,
        _ => return Some(MbValue::none()),
    };

    // Look up __next__ with borrow released.
    let next_method = class::mb_lookup_dunder(
        iter_obj,
        MbValue::from_ptr(MbObject::new_str("__next__".into())),
    );
    if next_method.is_none() {
        // No __next__ — mark exhausted and release our retain on iter_obj.
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        unsafe {
            super::rc::release_if_ptr(iter_obj);
        }
        return Some(MbValue::none());
    }

    // Clear any stale StopIteration flag before calling __next__.
    check_stop_iteration();

    // Invoke __next__(self) with borrow fully released.
    let result = class::mb_call_method1(next_method, iter_obj);

    // Check StopIteration — set either by mb_stop_iteration() or signal_stop_iteration().
    let stopped = check_stop_iteration();
    // A non-StopIteration exception raised inside __next__ (e.g. RuntimeError)
    // is a real error: CPython propagates it out of the drive loop rather than
    // treating it as end-of-iteration. Mark exhausted so the loop stops, but
    // PRESERVE the pending exception so the caller (for-loop, list(),
    // extract_list, zip_longest, ...) propagates it instead of silently
    // truncating. Only StopIteration / a bare None result is normal exhaustion.
    let real_exc = !stopped
        && super::exception::current_exception_type()
            .map(|t| t != "StopIteration")
            .unwrap_or(false);
    if real_exc {
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        unsafe {
            super::rc::release_if_ptr(iter_obj);
        }
        return Some(MbValue::none());
    }
    if stopped || result.is_none() {
        // Iterator exhausted — release our retained reference to iter_obj.
        // Also clear the pending exception slot: `raise StopIteration`
        // inside user __next__ sets BOTH the iterator flag and
        // CURRENT_EXCEPTION, and the for-loop protocol consumes the
        // former but never looks at the latter, which would otherwise
        // leak out to the caller as an uncaught StopIteration.
        super::exception::mb_clear_exception();
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        unsafe {
            super::rc::release_if_ptr(iter_obj);
        }
        return Some(MbValue::none());
    }

    Some(result)
}

/// Advance a Map or Filter iterator out-of-line.
///
/// Map/Filter iterators MUST be advanced without holding the ITERATORS borrow
/// because the user-provided function/predicate runs JIT-compiled code that
/// may call `mb_hasattr` → `mb_is_iterator_handle` → borrow ITERATORS, which
/// would panic if the outer borrow is still held (#map-filter-reentrant).
///
/// Protocol:
/// 1. Check for peeked/exhausted under a short immutable borrow.
/// 2. Remove the outer Map/Filter iter from ITERATORS (releasing the slot).
/// 3. Advance the inner iterator and apply func/predicate with borrow free.
/// 4. Reinsert the updated outer iterator.
///
/// Returns `Some(value)` if this is a Map/Filter iter, `None` otherwise.
fn advance_map_filter_if_applicable(id: u64) -> Option<MbValue> {
    // Step 1: detect kind; consume peeked value if present; short-circuit exhausted.
    // Returns: Some((is_map, func, inner_id, Some(peeked))) or
    //          Some((is_map, func, inner_id, None))  — need to advance inner
    //          None — not a Map/Filter iterator (caller falls through)
    struct Info {
        is_map: bool,
        func: MbValue,
        inner_id: u64,
        peeked: Option<MbValue>,
    }
    let info_opt = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match &iter.kind {
            IterKind::Map { func, inner_id } => {
                let func = *func;
                let inner_id = *inner_id;
                if iter.exhausted {
                    return None;
                } // caller will see None → fall through to borrow_mut block
                let peeked = iter.peeked.take();
                Some(Info {
                    is_map: true,
                    func,
                    inner_id,
                    peeked,
                })
            }
            IterKind::Filter { func, inner_id } => {
                let func = *func;
                let inner_id = *inner_id;
                if iter.exhausted {
                    return None;
                }
                let peeked = iter.peeked.take();
                Some(Info {
                    is_map: false,
                    func,
                    inner_id,
                    peeked,
                })
            }
            _ => None,
        }
    })?;

    // Short-circuit: return cached peeked value without advancing inner.
    if let Some(peeked) = info_opt.peeked {
        return Some(peeked);
    }

    let Info {
        is_map,
        func,
        inner_id,
        ..
    } = info_opt;
    let inner_handle = MbValue::from_int(inner_id as i64);

    if is_map {
        // Advance inner once (with ITERATORS borrow released).
        let inner_val = mb_next(inner_handle);
        // Check exhaustion.
        let inner_exhausted = ITERATORS.with(|iters| {
            iters
                .borrow()
                .get(&inner_id)
                .map(|i| i.exhausted)
                .unwrap_or(false)
        });
        if inner_exhausted && inner_val.is_none() {
            ITERATORS.with(|iters| {
                if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                    iter.exhausted = true;
                }
            });
            return Some(MbValue::none());
        }
        // Apply func.
        let result = super::builtins::call_any_callable(func, inner_val);
        Some(result)
    } else {
        // Filter: advance inner until predicate passes or inner is exhausted.
        loop {
            let cur_val = mb_next(inner_handle);
            let inner_exhausted = ITERATORS.with(|iters| {
                iters
                    .borrow()
                    .get(&inner_id)
                    .map(|i| i.exhausted)
                    .unwrap_or(false)
            });
            if inner_exhausted && cur_val.is_none() {
                ITERATORS.with(|iters| {
                    if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                        iter.exhausted = true;
                    }
                });
                return Some(MbValue::none());
            }
            // Test predicate.
            let passes = if func.is_none() {
                super::builtins::mb_bool(cur_val).as_bool().unwrap_or(false)
            } else {
                let r = super::builtins::call_any_callable(func, cur_val);
                super::builtins::mb_bool(r).as_bool().unwrap_or(false)
            };
            if passes {
                return Some(cur_val);
            }
        }
    }
}

/// Advance a MapN (multi-iterable map) iterator out-of-line.
///
/// Same reentrancy rationale as `advance_map_filter_if_applicable`: calling
/// func (user JIT code) may access ITERATORS via `mb_hasattr` /
/// `mb_is_iterator_handle`, so we must not hold the ITERATORS borrow.
///
/// Returns `Some(value)` when a value was produced (possibly `None` if
/// exhausted — caller checks `iter.exhausted`), or `None` when `id` is not
/// a MapN iterator.
fn advance_map_n_if_applicable(id: u64) -> Option<MbValue> {
    struct Info {
        func: MbValue,
        inner_ids: Vec<u64>,
        peeked: Option<MbValue>,
    }
    let info_opt = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match &iter.kind {
            IterKind::MapN { func, inner_ids } => {
                let func = *func;
                let inner_ids = inner_ids.clone();
                if iter.exhausted {
                    return None;
                }
                let peeked = iter.peeked.take();
                Some(Info {
                    func,
                    inner_ids,
                    peeked,
                })
            }
            _ => None,
        }
    })?;

    // Return cached peeked value without re-advancing.
    if let Some(peeked) = info_opt.peeked {
        return Some(peeked);
    }

    let Info {
        func, inner_ids, ..
    } = info_opt;

    // Collect one value from each inner iterator (ITERATORS borrow released).
    let mut values: Vec<MbValue> = Vec::with_capacity(inner_ids.len());
    for &inner_id in &inner_ids {
        let inner_handle = MbValue::from_int(inner_id as i64);
        let val = mb_next(inner_handle);
        let inner_exhausted = ITERATORS.with(|iters| {
            iters
                .borrow()
                .get(&inner_id)
                .map(|i| i.exhausted)
                .unwrap_or(true)
        });
        if inner_exhausted {
            ITERATORS.with(|iters| {
                if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                    iter.exhausted = true;
                }
            });
            return Some(MbValue::none()); // signal exhaustion
        }
        values.push(val);
    }

    // Call func with all collected values (ITERATORS borrow released).
    let args_list = MbValue::from_ptr(super::rc::MbObject::new_list(values));
    let result = super::builtins::mb_call_spread(func, args_list);
    Some(result)
}

/// Advance an Enumerate iterator out-of-line.
///
/// Same reentrancy rationale as `advance_map_filter_if_applicable`: the inner
/// iterator may be a Map/Filter/MapN whose advance runs user JIT code that
/// accesses ITERATORS (via `mb_hasattr` / `mb_is_iterator_handle`), so we must
/// drive it via `mb_next` with the ITERATORS borrow released.
///
/// Returns `Some(value)` when this is an Enumerate iterator (possibly
/// `MbValue::none()` on exhaustion — caller checks `iter.exhausted`), or `None`
/// when `id` is not an Enumerate iterator (caller falls through).
fn advance_enumerate_if_applicable(id: u64) -> Option<MbValue> {
    struct Info {
        inner_id: u64,
        count: i64,
        peeked: Option<MbValue>,
    }
    let info_opt = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match &iter.kind {
            IterKind::Enumerate { inner_id, count } => {
                let inner_id = *inner_id;
                let count = *count;
                if iter.exhausted {
                    return None;
                }
                let peeked = iter.peeked.take();
                Some(Info {
                    inner_id,
                    count,
                    peeked,
                })
            }
            _ => None,
        }
    })?;

    // Return cached peeked value without advancing the inner.
    if let Some(peeked) = info_opt.peeked {
        return Some(peeked);
    }

    let Info {
        inner_id, count, ..
    } = info_opt;
    let inner_handle = MbValue::from_int(inner_id as i64);

    // Advance inner once (ITERATORS borrow released).
    let inner_val = mb_next(inner_handle);
    let inner_exhausted = ITERATORS.with(|iters| {
        iters
            .borrow()
            .get(&inner_id)
            .map(|i| i.exhausted)
            .unwrap_or(true)
    });
    if inner_exhausted && inner_val.is_none() {
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        return Some(MbValue::none());
    }

    // Bump the stored count and emit (index, value).
    ITERATORS.with(|iters| {
        if let Some(iter) = iters.borrow_mut().get_mut(&id) {
            if let IterKind::Enumerate { count, .. } = &mut iter.kind {
                *count += 1;
            }
        }
    });
    let idx = MbValue::from_int(count);
    Some(MbValue::from_ptr(MbObject::new_tuple(vec![idx, inner_val])))
}

/// Advance a Zip iterator out-of-line.
///
/// Same reentrancy rationale as `advance_map_n_if_applicable`: any inner may be
/// a Map/Filter/MapN whose advance runs user JIT code that accesses ITERATORS,
/// so we drive each inner via `mb_next` with the ITERATORS borrow released.
///
/// Preserves CPython's exact PEP 618 strict-mode `ValueError` messages
/// ("zip() argument N is shorter/longer than argument 1").
///
/// Returns `Some(value)` when this is a Zip iterator (possibly `MbValue::none()`
/// on exhaustion — caller checks `iter.exhausted`), or `None` when `id` is not a
/// Zip iterator (caller falls through).
fn advance_zip_if_applicable(id: u64) -> Option<MbValue> {
    struct Info {
        inner_ids: Vec<u64>,
        strict: bool,
        peeked: Option<MbValue>,
    }
    let info_opt = ITERATORS.with(|iters| {
        let mut iters = iters.borrow_mut();
        let iter = iters.get_mut(&id)?;
        match &iter.kind {
            IterKind::Zip { inner_ids, strict } => {
                let inner_ids = inner_ids.clone();
                let strict = *strict;
                if iter.exhausted {
                    return None;
                }
                let peeked = iter.peeked.take();
                Some(Info {
                    inner_ids,
                    strict,
                    peeked,
                })
            }
            _ => None,
        }
    })?;

    // Return cached peeked value without advancing the inners.
    if let Some(peeked) = info_opt.peeked {
        return Some(peeked);
    }

    let Info {
        inner_ids, strict, ..
    } = info_opt;
    let n = inner_ids.len();

    // Pull one value from each inner (ITERATORS borrow released between calls).
    let mut vals = Vec::with_capacity(n);
    let mut exhausted_at: Option<usize> = None;
    for i in 0..n {
        let inner_handle = MbValue::from_int(inner_ids[i] as i64);
        let val = mb_next(inner_handle);
        let inner_exhausted = ITERATORS.with(|iters| {
            iters
                .borrow()
                .get(&inner_ids[i])
                .map(|it| it.exhausted)
                .unwrap_or(true)
        });
        if val.is_none() && inner_exhausted {
            exhausted_at = Some(i);
            break;
        }
        vals.push(val);
    }

    if let Some(i) = exhausted_at {
        ITERATORS.with(|iters| {
            if let Some(iter) = iters.borrow_mut().get_mut(&id) {
                iter.exhausted = true;
            }
        });
        if strict {
            if i > 0 {
                // inner[0..i] yielded values for this row, but inner[i] just
                // exhausted → arg{i+1} is shorter than arg 1 (CPython wording).
                let typ = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
                let msg = MbValue::from_ptr(MbObject::new_str(format!(
                    "zip() argument {} is shorter than argument 1",
                    i + 1
                )));
                super::exception::mb_raise(typ, msg);
            } else {
                // inner[0] exhausted at row start. CPython advances each
                // remaining inner once to confirm they are also empty; the
                // first one that still yields a value names the "longer" arg.
                for j in 1..n {
                    let inner_handle = MbValue::from_int(inner_ids[j] as i64);
                    let v = mb_next(inner_handle);
                    let je = ITERATORS.with(|iters| {
                        iters
                            .borrow()
                            .get(&inner_ids[j])
                            .map(|it| it.exhausted)
                            .unwrap_or(true)
                    });
                    if !(v.is_none() && je) {
                        let typ = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
                        let msg = MbValue::from_ptr(MbObject::new_str(format!(
                            "zip() argument {} is longer than argument 1",
                            j + 1
                        )));
                        super::exception::mb_raise(typ, msg);
                        break;
                    }
                }
            }
        }
        return Some(MbValue::none());
    }

    Some(MbValue::from_ptr(MbObject::new_tuple(vals)))
}

/// Advance an iterator and return the next value.
fn advance_iter(iter: &mut MbIterator) -> MbValue {
    // Consume any pre-fetched peeked value before re-advancing.
    if let Some(peeked) = iter.peeked.take() {
        return peeked;
    }
    match &mut iter.kind {
        IterKind::List(list_val) => {
            if let Some(ptr) = list_val.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        let items = lock.read().unwrap();
                        if iter.index < items.len() {
                            let val = items[iter.index];
                            iter.index += 1;
                            return val;
                        }
                    }
                }
            }
            iter.exhausted = true;
            MbValue::none()
        }
        IterKind::Tuple(tup_val) => {
            if let Some(ptr) = tup_val.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(ref items) = (*ptr).data {
                        if iter.index < items.len() {
                            let val = items[iter.index];
                            iter.index += 1;
                            return val;
                        }
                    }
                }
            }
            iter.exhausted = true;
            MbValue::none()
        }
        IterKind::Str(chars) => {
            if iter.index < chars.len() {
                let c = chars[iter.index];
                iter.index += 1;
                MbValue::from_ptr(MbObject::new_str(c.to_string()))
            } else {
                iter.exhausted = true;
                MbValue::none()
            }
        }
        IterKind::DictKeys(keys) => {
            if iter.index < keys.len() {
                let key = &keys[iter.index];
                iter.index += 1;
                dict_key_to_mbvalue(key)
            } else {
                iter.exhausted = true;
                MbValue::none()
            }
        }
        IterKind::Range {
            current,
            stop,
            step,
        } => {
            if (*step > 0 && *current < *stop) || (*step < 0 && *current > *stop) {
                let val = MbValue::from_int(*current);
                *current += *step;
                val
            } else {
                iter.exhausted = true;
                MbValue::none()
            }
        }
        IterKind::Enumerate { .. } => {
            // Enumerate advance is handled out-of-line by
            // `advance_enumerate_if_applicable` in `mb_has_next` / `mb_next` /
            // `mb_next_raise`. Reaching here while ITERATORS is held would cause
            // a reentrant borrow panic when the inner is a Map/Filter/MapN whose
            // advance runs JIT code that accesses ITERATORS. Safety net only.
            iter.exhausted = true;
            MbValue::none()
        }
        IterKind::Reversed { items, index } => {
            if *index < items.len() {
                let val = items[*index];
                *index += 1;
                val
            } else {
                iter.exhausted = true;
                MbValue::none()
            }
        }
        IterKind::UserDefined(iter_obj) => {
            // R2: Call __next__ on the user-defined iterator
            // Use mb_lookup_dunder for safety (class methods only)
            let next_method = class::mb_lookup_dunder(
                *iter_obj,
                MbValue::from_ptr(MbObject::new_str("__next__".into())),
            );
            if next_method.is_none() {
                iter.exhausted = true;
                return MbValue::none();
            }
            // Clear any stale flag before calling __next__
            check_stop_iteration();
            // Invoke __next__(self) to get next value
            let result = class::mb_call_method1(next_method, *iter_obj);
            // Check StopIteration flag first (explicit signal from __next__)
            if check_stop_iteration() {
                iter.exhausted = true;
                return MbValue::none();
            }
            // Fallback: if __next__ returned None without setting the flag,
            // treat as exhaustion for backward compat (until raise is compiled).
            if result.is_none() {
                iter.exhausted = true;
            }
            result
        }
        IterKind::Generator(gen_handle) => {
            let gen_handle = *gen_handle;
            // Clear stale StopIteration flag
            check_stop_iteration();
            let val = super::generator::mb_generator_next(gen_handle);
            if check_stop_iteration() {
                iter.exhausted = true;
                // resume_generator sets CURRENT_EXCEPTION StopIteration on
                // completion in addition to the local flag. Composite
                // iterators (Enumerate, Zip, Map, Filter) reach this path
                // via advance_iter and must not let the runtime exception
                // leak past the wrapper boundary, otherwise consuming
                // `enumerate(gen())` etc. via list/tuple/dict aborts on a
                // phantom StopIteration after the legitimate result.
                super::exception::mb_clear_exception();
                return MbValue::none();
            }
            val
        }
        IterKind::Zip { .. } => {
            // Zip advance is handled out-of-line by
            // `advance_zip_if_applicable` in `mb_has_next` / `mb_next` /
            // `mb_next_raise`. Reaching here while ITERATORS is held would cause
            // a reentrant borrow panic when an inner is a Map/Filter/MapN whose
            // advance runs JIT code that accesses ITERATORS. Safety net only.
            iter.exhausted = true;
            MbValue::none()
        }
        IterKind::Map { .. } | IterKind::Filter { .. } | IterKind::MapN { .. } => {
            // Map/Filter/MapN advance is handled out-of-line by
            // `advance_map_filter_if_applicable` / `advance_map_n_if_applicable`
            // in `mb_has_next` / `mb_next` / `mb_next_raise`. Reaching here
            // while ITERATORS is held would cause a reentrant borrow panic when
            // `call_any_callable` / `mb_call_spread` runs JIT code that accesses
            // ITERATORS via `mb_is_iterator_handle`.
            // Return None and mark exhausted as a safety net.
            iter.exhausted = true;
            MbValue::none()
        }
        IterKind::Callable { func, sentinel } => {
            // Call callable() with no arguments
            let result = class::mb_call0(*func);
            // Compare result to sentinel using Python equality
            let eq = super::builtins::mb_eq(result, *sentinel);
            if eq.as_bool().unwrap_or(false) {
                iter.exhausted = true;
                return MbValue::none();
            }
            result
        }
        IterKind::Count {
            is_float,
            cur_i,
            step_i,
            cur_f,
            step_f,
        } => {
            // Infinite arithmetic counter — never exhausts.
            if *is_float {
                let val = MbValue::from_float(*cur_f);
                *cur_f += *step_f;
                val
            } else {
                let val = MbValue::from_int(*cur_i);
                *cur_i += *step_i;
                val
            }
        }
        IterKind::Repeat { val, remaining } => match remaining {
            Some(0) => {
                iter.exhausted = true;
                MbValue::none()
            }
            Some(n) => {
                *n -= 1;
                let v = *val;
                unsafe {
                    super::rc::retain_if_ptr(v);
                }
                v
            }
            None => {
                let v = *val;
                unsafe {
                    super::rc::retain_if_ptr(v);
                }
                v
            }
        },
        IterKind::Cycle { items, pos } => {
            if items.is_empty() {
                iter.exhausted = true;
                MbValue::none()
            } else {
                let v = items[*pos];
                *pos = (*pos + 1) % items.len();
                unsafe {
                    super::rc::retain_if_ptr(v);
                }
                v
            }
        }
    }
}

// ── Utility: collect iterator to list ──

/// Collect all remaining values from an iterator into a list.
pub fn mb_list_from_iter(iter_handle: MbValue) -> MbValue {
    let mut items = Vec::new();
    loop {
        let val = mb_next(iter_handle);
        if val.is_none() {
            if let Some(id) = iter_handle.as_int() {
                let exhausted = ITERATORS.with(|iters| {
                    iters
                        .borrow()
                        .get(&(id as u64))
                        .map(|i| i.exhausted)
                        .unwrap_or(true)
                });
                if exhausted {
                    break;
                }
            } else {
                break;
            }
        }
        items.push(val);
    }
    mb_iter_release(iter_handle);
    MbValue::from_ptr(MbObject::new_list(items))
}

// ── Cleanup ──

/// Reset all iterator-related thread_local state to defaults.
/// Called as part of centralized runtime cleanup between test executions.
/// Iterator contents may include borrowed or imprecisely retained MbValues, so
/// shutdown cleanup clears the registry without releasing pointees. Process
/// teardown reclaims the leaked objects.
pub(crate) fn cleanup_all_iterators() {
    let _ = ITERATORS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = NEXT_ITER_ID.with(|c| c.set(ITER_ID_BASE));
    let _ = STOP_ITERATION.with(|c| c.set(false));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_iter() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]));
        let it = mb_iter(list);
        assert_eq!(mb_next(it).as_int(), Some(10));
        assert_eq!(mb_next(it).as_int(), Some(20));
        assert_eq!(mb_next(it).as_int(), Some(30));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_range_iter() {
        let it = mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(5),
            MbValue::from_int(2),
        );
        assert_eq!(mb_next(it).as_int(), Some(0));
        assert_eq!(mb_next(it).as_int(), Some(2));
        assert_eq!(mb_next(it).as_int(), Some(4));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_string_iter() {
        let s = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        let it = mb_iter(s);
        let a = mb_next(it);
        assert!(a.is_ptr());
        let b = mb_next(it);
        assert!(b.is_ptr());
        let c = mb_next(it);
        assert!(c.is_ptr());
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_dict_iter() {
        use super::super::dict_ops::mb_dict_setitem;
        let obj = MbValue::from_ptr(MbObject::new_dict());
        let key_a = MbValue::from_ptr(MbObject::new_str("a".into()));
        let key_b = MbValue::from_ptr(MbObject::new_str("b".into()));
        mb_dict_setitem(obj, key_a, MbValue::from_int(1));
        mb_dict_setitem(obj, key_b, MbValue::from_int(2));
        let it = mb_iter(obj);
        let v1 = mb_next(it);
        assert!(v1.is_ptr()); // String key
        let v2 = mb_next(it);
        assert!(v2.is_ptr()); // String key
        assert!(mb_next(it).is_none()); // exhausted
        mb_iter_release(it);
    }

    #[test]
    fn test_tuple_iter() {
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(100),
            MbValue::from_int(200),
        ]));
        let it = mb_iter(tup);
        assert_eq!(mb_next(it).as_int(), Some(100));
        assert_eq!(mb_next(it).as_int(), Some(200));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_instance_without_iter_not_iterable() {
        // Instance without __iter__ should return none (TypeError)
        let inst = MbValue::from_ptr(MbObject::new_instance("Foo".to_string()));
        let it = mb_iter(inst);
        assert!(it.is_none()); // Not iterable
    }

    #[test]
    fn test_collect() {
        let it = mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(3),
            MbValue::from_int(1),
        );
        let list = mb_list_from_iter(it);
        assert!(list.is_ptr());
        unsafe {
            if let ObjData::List(ref lock) = (*list.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(0));
                assert_eq!(items[1].as_int(), Some(1));
                assert_eq!(items[2].as_int(), Some(2));
            }
            super::super::rc::mb_release(list.as_ptr().unwrap());
        }
    }

    #[test]
    fn test_range_iter_negative_step() {
        let it = mb_range_iter(
            MbValue::from_int(5),
            MbValue::from_int(0),
            MbValue::from_int(-1),
        );
        assert_eq!(mb_next(it).as_int(), Some(5));
        assert_eq!(mb_next(it).as_int(), Some(4));
        assert_eq!(mb_next(it).as_int(), Some(3));
        assert_eq!(mb_next(it).as_int(), Some(2));
        assert_eq!(mb_next(it).as_int(), Some(1));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_range_iter_zero_step() {
        let it = mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(5),
            MbValue::from_int(0),
        );
        assert!(it.is_none()); // ValueError
    }

    #[test]
    fn test_has_next() {
        let it = mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(1),
        );
        assert_eq!(mb_has_next(it).as_bool(), Some(true));
        mb_next(it); // consume the single element
        mb_next(it); // exhaust
        assert_eq!(mb_has_next(it).as_bool(), Some(false));
        mb_iter_release(it);
    }

    #[test]
    fn test_iter_primitive_not_iterable() {
        let it = mb_iter(MbValue::from_int(42));
        assert!(it.is_none());
    }

    #[test]
    fn test_set_iter() {
        let set = MbValue::from_ptr(MbObject::new_set(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let it = mb_iter(set);
        assert!(it.is_int()); // valid iterator handle
        mb_next(it);
        mb_next(it);
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_frozenset_iter() {
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(10)]));
        let it = mb_iter(fs);
        assert_eq!(mb_next(it).as_int(), Some(10));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_bytes_iter() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![65, 66]));
        let it = mb_iter(b);
        assert_eq!(mb_next(it).as_int(), Some(65));
        assert_eq!(mb_next(it).as_int(), Some(66));
        assert!(mb_next(it).is_none());
        mb_iter_release(it);
    }

    #[test]
    fn test_enumerate() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
        ]));
        let it = mb_enumerate(list, MbValue::from_int(0));
        let first = mb_next(it);
        // Should return a tuple (0, 10)
        unsafe {
            if let ObjData::Tuple(ref items) = (*first.as_ptr().unwrap()).data {
                assert_eq!(items[0].as_int(), Some(0));
                assert_eq!(items[1].as_int(), Some(10));
            } else {
                panic!("expected tuple");
            }
        }
        mb_iter_release(it);
    }

    #[test]
    fn test_reversed_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let it = mb_reversed(list);
        assert!(it.as_int().is_some(), "reversed should return a handle");
        assert_eq!(mb_next(it).as_int(), Some(3));
        assert_eq!(mb_next(it).as_int(), Some(2));
        assert_eq!(mb_next(it).as_int(), Some(1));
        assert!(mb_next(it).is_none(), "should be exhausted");
        mb_iter_release(it);
    }

    #[test]
    fn test_stop_iteration_flag() {
        mb_stop_iteration(MbValue::none());
        assert!(check_stop_iteration());
        // Flag should be cleared after check
        assert!(!check_stop_iteration());
    }

    #[test]
    fn test_iter_release_invalid() {
        mb_iter_release(MbValue::from_int(999999));
        // Should not crash
    }

    // ── Cleanup tests (R1: per-module cleanup for iterators) ──

    #[test]
    fn test_cleanup_all_iterators_clears_state() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
        ]));
        let it = mb_iter(list);
        assert!(it.is_int(), "should get a valid iterator");
        // Consume one element to prove it works
        assert_eq!(mb_next(it).as_int(), Some(10));

        cleanup_all_iterators();

        // After cleanup, the iterator handle should be gone
        assert!(
            mb_next(it).is_none(),
            "ITERATORS should be empty after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_iterators_resets_id_counter() {
        let list1 = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let it1 = mb_iter(list1);

        cleanup_all_iterators();

        let list2 = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(2)]));
        let it2 = mb_iter(list2);
        // Both should get the same ID (1) since counter was reset
        assert_eq!(
            it1.as_int(),
            it2.as_int(),
            "iter ID counter should reset after cleanup"
        );
    }

    #[test]
    fn test_cleanup_all_iterators_on_empty() {
        cleanup_all_iterators();
        // No panic = success
    }

    #[test]
    fn test_cleanup_all_iterators_multiple_iterators() {
        let l1 = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let l2 = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(2)]));
        let l3 = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(3)]));
        let _it1 = mb_iter(l1);
        let _it2 = mb_iter(l2);
        let _it3 = mb_iter(l3);

        cleanup_all_iterators();

        // All should be gone
        assert!(mb_next(_it1).is_none());
        assert!(mb_next(_it2).is_none());
        assert!(mb_next(_it3).is_none());
    }

    // ── R12: iter(callable, sentinel) creation ────────────────────────────

    /// Verify that mb_iter_sentinel creates a valid iterator handle.
    #[test]
    fn test_iter_sentinel_creates_handle() {
        let callable = MbValue::none(); // placeholder
        let sentinel = MbValue::from_int(0);
        let it = mb_iter_sentinel(callable, sentinel);
        assert!(it.is_int(), "iter_sentinel should return an int handle");
        mb_iter_release(it);
    }

    /// Verify that multiple callable-sentinel iterators get distinct handles.
    #[test]
    fn test_iter_sentinel_distinct_handles() {
        let it1 = mb_iter_sentinel(MbValue::none(), MbValue::from_int(0));
        let it2 = mb_iter_sentinel(MbValue::none(), MbValue::from_int(0));
        assert_ne!(
            it1.as_int(),
            it2.as_int(),
            "different sentinel iterators should have distinct IDs"
        );
        mb_iter_release(it1);
        mb_iter_release(it2);
    }

    /// Verify that a sentinel iterator handle is registered in the thread-local
    /// store and can be released without panic.
    #[test]
    fn test_iter_sentinel_release() {
        let it = mb_iter_sentinel(MbValue::none(), MbValue::from_int(42));
        assert!(it.is_int());
        mb_iter_release(it);
        // After release, next() should return None
        assert!(
            mb_next(it).is_none(),
            "next() on released sentinel iterator should return None"
        );
    }

    // ── UserDefined iterator out-of-line fix (REQ: P1 bug fix) ──────────────

    /// Verify advance_userdefined_if_applicable returns None for non-UserDefined
    /// iterator kinds (list, range, etc.), ensuring fallthrough to advance_iter.
    /// REQ: P1-USER-ITER
    #[test]
    fn test_advance_userdefined_returns_none_for_list_iter() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let it = mb_iter(list);
        let id = it.as_int().unwrap() as u64;
        // advance_userdefined_if_applicable should return None for IterKind::List
        let result = advance_userdefined_if_applicable(id);
        assert!(result.is_none(), "should not handle List kind");
        mb_iter_release(it);
    }

    /// Verify advance_userdefined_if_applicable returns None for range iters.
    /// REQ: P1-USER-ITER
    #[test]
    fn test_advance_userdefined_returns_none_for_range_iter() {
        let it = mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(3),
            MbValue::from_int(1),
        );
        let id = it.as_int().unwrap() as u64;
        let result = advance_userdefined_if_applicable(id);
        assert!(result.is_none(), "should not handle Range kind");
        mb_iter_release(it);
    }

    /// Verify that a manually inserted UserDefined iterator that is already
    /// exhausted returns Some(None) immediately without calling __next__.
    /// REQ: P1-USER-ITER
    #[test]
    fn test_advance_userdefined_exhausted_returns_none_immediately() {
        // Insert an exhausted UserDefined iterator directly
        let fake_obj = MbValue::from_int(0); // non-ptr — retain_if_ptr is a no-op
        let id = alloc_iter_id();
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::UserDefined(fake_obj),
                    index: 0,
                    exhausted: true,
                    peeked: None,
                },
            );
        });
        let result = advance_userdefined_if_applicable(id);
        assert!(result.is_some(), "should return Some for UserDefined kind");
        assert!(
            result.unwrap().is_none(),
            "exhausted should return None value"
        );
        // Clean up
        ITERATORS.with(|iters| {
            iters.borrow_mut().remove(&id);
        });
    }

    /// Verify that a peeked value is returned immediately without re-advancing.
    /// REQ: P1-USER-ITER
    #[test]
    fn test_advance_userdefined_returns_peeked_value() {
        let fake_obj = MbValue::from_int(0);
        let peeked_val = MbValue::from_int(42);
        let id = alloc_iter_id();
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::UserDefined(fake_obj),
                    index: 0,
                    exhausted: false,
                    peeked: Some(peeked_val),
                },
            );
        });
        let result = advance_userdefined_if_applicable(id);
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().as_int(),
            Some(42),
            "should return peeked value"
        );
        // peeked should be consumed — next call should hit the __next__ path
        let peeked_after = ITERATORS.with(|iters| iters.borrow().get(&id).and_then(|i| i.peeked));
        assert!(peeked_after.is_none(), "peeked should be consumed");
        // Clean up (no ptr retain so no release needed)
        ITERATORS.with(|iters| {
            iters.borrow_mut().remove(&id);
        });
    }

    /// Verify mb_iter_release correctly handles a UserDefined iterator
    /// without crashing (non-pointer iter_obj is a no-op for release_if_ptr).
    /// REQ: P1-USER-ITER
    #[test]
    fn test_userdefined_iter_release_no_crash() {
        let fake_obj = MbValue::from_int(99); // non-ptr; release_if_ptr is a no-op
        let id = alloc_iter_id();
        let handle = MbValue::from_int(id as i64);
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::UserDefined(fake_obj),
                    index: 0,
                    exhausted: false,
                    peeked: None,
                },
            );
        });
        // Should not crash — release_if_ptr on non-ptr is a no-op
        mb_iter_release(handle);
        // Verify it was removed
        let exists = ITERATORS.with(|iters| iters.borrow().contains_key(&id));
        assert!(!exists, "iterator should be removed after release");
    }

    /// Verify mb_has_next correctly identifies UserDefined iterators and uses
    /// the out-of-line path (no reentrant borrow).
    /// REQ: P1-USER-ITER
    #[test]
    fn test_has_next_userdefined_exhausted_returns_false() {
        let fake_obj = MbValue::from_int(0);
        let id = alloc_iter_id();
        let handle = MbValue::from_int(id as i64);
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::UserDefined(fake_obj),
                    index: 0,
                    exhausted: true,
                    peeked: None,
                },
            );
        });
        let result = mb_has_next(handle);
        assert_eq!(
            result.as_bool(),
            Some(false),
            "exhausted UserDefined should return false"
        );
        ITERATORS.with(|iters| {
            iters.borrow_mut().remove(&id);
        });
    }

    /// Verify mb_has_next returns true when a peeked value is cached.
    /// REQ: P1-USER-ITER
    #[test]
    fn test_has_next_userdefined_peeked_returns_true() {
        let fake_obj = MbValue::from_int(0);
        let id = alloc_iter_id();
        let handle = MbValue::from_int(id as i64);
        ITERATORS.with(|iters| {
            iters.borrow_mut().insert(
                id,
                MbIterator {
                    kind: IterKind::UserDefined(fake_obj),
                    index: 0,
                    exhausted: false,
                    peeked: Some(MbValue::from_int(7)),
                },
            );
        });
        let result = mb_has_next(handle);
        assert_eq!(
            result.as_bool(),
            Some(true),
            "peeked UserDefined should return true"
        );
        ITERATORS.with(|iters| {
            iters.borrow_mut().remove(&id);
        });
    }
}
