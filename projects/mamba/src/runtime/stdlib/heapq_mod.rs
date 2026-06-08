//! @codegen-skip: handwrite-pre-standardize
//!
//! heapq module for Mamba (Phase 2 conformance, Task #8).
//!
//! Implements Python 3.12 `heapq` stdlib: `heappush`, `heappop`,
//! `heappushpop`, `heapreplace`, `heapify`, `nlargest`, `nsmallest`. The
//! heap discipline is the canonical CPython binary-heap algorithm — a
//! list-as-min-heap with `_siftup`/`_siftdown` O(log n) ops, NOT an
//! O(n) sorted-insert (which the previous shim used and which fails
//! CPython's heap invariant).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_{unary,binary,ternary} + list-as-heap
//! sift_up/sift_down primitives) is not yet emitted by score codegen.
//! Heap algorithms are a perfect codegen candidate (closed algorithm,
//! straightforward generic-comparator shape), but the section type
//! doesn't exist yet. Will convert to CODEGEN once the standardize
//! sweep grows an `algorithmic_primitive` section type.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_index_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_heap_list(val: MbValue) -> bool {
    val.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::List(_))
    })
}

macro_rules! dispatch_heap_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let heap = a.get(0).copied().unwrap_or_else(MbValue::none);
            if !is_heap_list(heap) {
                return raise_type_error("heap argument must be a list");
            }
            $fn(heap)
        }
    };
}

macro_rules! dispatch_heap_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            // CPython's heappush/heappushpop/heapreplace take exactly two
            // arguments; calling with one (`heappush([])`) raises TypeError,
            // not a silent push-of-None.
            if nargs < 2 {
                return raise_type_error(concat!(
                    stringify!($fn), " expected 2 arguments"
                ));
            }
            let heap = a.get(0).copied().unwrap_or_else(MbValue::none);
            if !is_heap_list(heap) {
                return raise_type_error("heap argument must be a list");
            }
            $fn(
                heap,
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_heap_binary!(dispatch_heappush,        mb_heapq_heappush);
dispatch_heap_unary! (dispatch_heappop,         mb_heapq_heappop);
dispatch_heap_binary!(dispatch_heappushpop,     mb_heapq_heappushpop);
dispatch_heap_binary!(dispatch_heapreplace,     mb_heapq_heapreplace);
dispatch_heap_unary! (dispatch_heapify,         mb_heapq_heapify);
dispatch_heap_unary! (dispatch_heapify_max,     mb_heapq_heapify_max);
dispatch_heap_unary! (dispatch_heappop_max,     mb_heapq_heappop_max);
dispatch_heap_binary!(dispatch_heapreplace_max, mb_heapq_heapreplace_max);

/// nlargest(n, iterable, key=None). The optional `key=` keyword is
/// delivered as a trailing kwargs dict positional by HIR lowering, so we
/// accept the variadic shape and forward args[2] (the dict, if present).
unsafe extern "C" fn dispatch_nlargest(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let n = a.first().copied().unwrap_or_else(MbValue::none);
    let iterable = a.get(1).copied().unwrap_or_else(MbValue::none);
    let kwargs = a.get(2).copied().unwrap_or_else(MbValue::none);
    mb_heapq_nlargest(n, iterable, kwargs)
}

/// nsmallest(n, iterable, key=None). See `dispatch_nlargest`.
unsafe extern "C" fn dispatch_nsmallest(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let n = a.first().copied().unwrap_or_else(MbValue::none);
    let iterable = a.get(1).copied().unwrap_or_else(MbValue::none);
    let kwargs = a.get(2).copied().unwrap_or_else(MbValue::none);
    mb_heapq_nsmallest(n, iterable, kwargs)
}

/// merge accepts variadic sorted iterables: `merge(*iterables, key=None,
/// reverse=False)`. Implemented as a flat-args dispatcher so all
/// positional iterables flow through, not just the first two. Returns a
/// single sorted list — eager rather than the CPython lazy-iterator
/// semantics, which matches mamba's current generator support (no
/// PyGenObject yet).
///
/// The `key=`/`reverse=` keywords arrive as a trailing kwargs dict
/// positional (HIR lowering for an attribute call with kwargs). It is
/// detected and consumed rather than treated as another iterable.
/// CPython merges presorted inputs with a stable k-way heap merge; an
/// eager stable sort by the same comparison key reproduces the observable
/// output order (including ties).
unsafe extern "C" fn dispatch_merge(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // A trailing dict positional is the kwargs bundle, not an iterable.
    let (iterables, kwargs): (&[MbValue], MbValue) = match a.last() {
        Some(last) if last.as_ptr().is_some_and(|ptr| unsafe {
            matches!((*ptr).data, ObjData::Dict(_))
        }) => (&a[..a.len() - 1], *last),
        _ => (a, MbValue::none()),
    };
    let key = kwarg(kwargs, "key");
    let reverse = is_truthy(kwarg(kwargs, "reverse"));

    let mut all: Vec<MbValue> = Vec::new();
    for v in iterables.iter() {
        all.extend(super::super::builtins::extract_items(*v));
        // CPython's merge() does not swallow exceptions raised by an input
        // iterator (e.g. a generator that over-indexes its backing list and
        // raises IndexError). Our eager drain leaves such an exception
        // pending; surface it instead of producing a partial result.
        if super::super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
    }
    // Stable sort by the (optional) key; reverse flips ordering while
    // preserving stability, matching heapq.merge(reverse=True).
    all.sort_by(|x, y| {
        let ord = super::super::builtins::mb_value_cmp_pub(apply_key(key, *x), apply_key(key, *y));
        if reverse { ord.reverse() } else { ord }
    });
    // Elements are borrowed from the source iterables — retain on the way out.
    MbValue::from_ptr(MbObject::new_list_borrowed(all))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("heappush",     dispatch_heappush     as usize),
        ("heappop",      dispatch_heappop      as usize),
        ("heappushpop",  dispatch_heappushpop  as usize),
        ("heapreplace",  dispatch_heapreplace  as usize),
        ("heapify",      dispatch_heapify      as usize),
        ("nlargest",     dispatch_nlargest     as usize),
        ("nsmallest",    dispatch_nsmallest    as usize),
        ("merge",        dispatch_merge        as usize),
        // CPython's private max-heap helpers, exercised by the test-suite
        // and used internally by nlargest/nsmallest with key=.
        ("_heapify_max",     dispatch_heapify_max     as usize),
        ("_heappop_max",     dispatch_heappop_max     as usize),
        ("_heapreplace_max", dispatch_heapreplace_max as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("heapq", attrs);
}

/// CPython-faithful `<` for two MbValues. Delegates to the runtime's
/// general comparison (`mb_value_cmp_pub`), which handles int, float,
/// mixed int/float, str (lexicographic), tuple/list (element-wise), and
/// user instances with `__lt__`. This is what makes a heap of tuples or
/// strings order correctly — the previous `item_key` projected every
/// non-numeric value to 0, which silently broke the heap invariant.
#[inline]
fn lt(a: MbValue, b: MbValue) -> bool {
    super::super::builtins::mb_value_cmp_pub(a, b) == std::cmp::Ordering::Less
}

/// Read a string-keyed entry out of a trailing keyword-args dict.
///
/// `heapq.nlargest`/`nsmallest`/`merge` accept the `key=`/`reverse=`
/// keyword arguments. At the call site (an attribute call with kwargs),
/// HIR lowering appends a single trailing dict positional `{"key": ...,
/// "reverse": ...}`. This pulls a named value out of that dict, or
/// returns `None` when the arg isn't a dict / the name is absent.
fn kwarg(maybe_dict: MbValue, name: &str) -> MbValue {
    if !maybe_dict.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Dict(_))
    }) {
        return MbValue::none();
    }
    let key = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    super::super::dict_ops::mb_dict_get(maybe_dict, key, MbValue::none())
}

/// Apply a `key=` callable to an item, mirroring `sorted(..., key=fn)`.
/// Supports JIT/user callables, named builtins (`len`, `abs`, ...), and
/// instance callables. A None key is the identity.
fn apply_key(key: MbValue, item: MbValue) -> MbValue {
    if key.is_none() {
        return item;
    }
    if super::super::builtins::resolve_callable_pub(key).is_some() {
        return super::super::class::mb_call1_val(key, item);
    }
    // Named builtin callable passed as a bare string handle (e.g. `len`).
    if let Some(name) = key.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    }) {
        if let Some(v) = super::super::builtins::call_named_callable_pub(&name, item) {
            return v;
        }
    }
    // Instance callables (__call__, functools.partial, bound methods, ...).
    if key.as_ptr().is_some() {
        return super::super::class::mb_call1_val(key, item);
    }
    item
}

/// Truthiness of a `reverse=` keyword value.
fn is_truthy(v: MbValue) -> bool {
    v.as_bool() == Some(true) || v.as_int() == Some(1)
}

/// CPython's `_siftdown(heap, startpos, pos)` — bubble a fresh item up
/// the heap (yes, CPython names it "siftdown" even though it walks up).
fn sift_down(heap: &mut [MbValue], startpos: usize, mut pos: usize) {
    let newitem = heap[pos];
    while pos > startpos {
        let parent = (pos - 1) >> 1;
        if lt(newitem, heap[parent]) {
            heap[pos] = heap[parent];
            pos = parent;
            continue;
        }
        break;
    }
    heap[pos] = newitem;
}

/// CPython's `_siftup(heap, pos)` — bubble a hole down by promoting the
/// smaller child, then siftdown the original item to restore the
/// invariant. This is the "leaf sift" variant that gives heappop O(log n).
fn sift_up(heap: &mut [MbValue], mut pos: usize) {
    let endpos = heap.len();
    let startpos = pos;
    let newitem = heap[pos];
    let mut childpos = 2 * pos + 1;
    while childpos < endpos {
        let rightpos = childpos + 1;
        if rightpos < endpos && !lt(heap[childpos], heap[rightpos]) {
            childpos = rightpos;
        }
        heap[pos] = heap[childpos];
        pos = childpos;
        childpos = 2 * pos + 1;
    }
    heap[pos] = newitem;
    sift_down(heap, startpos, pos);
}

// ── Max-heap variants (CPython's private `_*_max` helpers) ──
//
// heapq exposes `_heapify_max`, `_heappop_max`, and `_heapreplace_max`
// (used internally by nlargest/merge and exercised directly by the CPython
// test-suite). They are the dual of the min-heap ops with the comparison
// reversed, so the root is the LARGEST element.

/// Max-heap `_siftdown_max` — bubble a fresh item up toward the root while
/// it is GREATER than its parent.
fn sift_down_max(heap: &mut [MbValue], startpos: usize, mut pos: usize) {
    let newitem = heap[pos];
    while pos > startpos {
        let parent = (pos - 1) >> 1;
        if lt(heap[parent], newitem) {
            heap[pos] = heap[parent];
            pos = parent;
            continue;
        }
        break;
    }
    heap[pos] = newitem;
}

/// Max-heap `_siftup_max` — promote the LARGER child as the hole descends.
fn sift_up_max(heap: &mut [MbValue], mut pos: usize) {
    let endpos = heap.len();
    let startpos = pos;
    let newitem = heap[pos];
    let mut childpos = 2 * pos + 1;
    while childpos < endpos {
        let rightpos = childpos + 1;
        if rightpos < endpos && !lt(heap[rightpos], heap[childpos]) {
            childpos = rightpos;
        }
        heap[pos] = heap[childpos];
        pos = childpos;
        childpos = 2 * pos + 1;
    }
    heap[pos] = newitem;
    sift_down_max(heap, startpos, pos);
}

/// heapq._heapify_max(list) -> None (in-place, max-heap order)
pub fn mb_heapq_heapify_max(lst: MbValue) -> MbValue {
    if let Some(ptr) = lst.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let n = items.len();
                if n > 1 {
                    let mut i = n / 2;
                    while i > 0 {
                        i -= 1;
                        sift_up_max(&mut items, i);
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// heapq._heappop_max(heap) -> largest item
pub fn mb_heapq_heappop_max(heap: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if items.is_empty() {
                    drop(items);
                    return raise_index_error("index out of range");
                }
                let last = items.pop().unwrap();
                if items.is_empty() { return last; }
                let returnitem = items[0];
                items[0] = last;
                sift_up_max(&mut items, 0);
                return returnitem;
            }
        }
    }
    MbValue::none()
}

/// heapq._heapreplace_max(heap, item) -> largest item, then push item.
pub fn mb_heapq_heapreplace_max(heap: MbValue, item: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if items.is_empty() {
                    drop(items);
                    return raise_index_error("index out of range");
                }
                let returnitem = items[0];
                items[0] = item;
                sift_up_max(&mut items, 0);
                return returnitem;
            }
        }
    }
    MbValue::none()
}

/// heapq.heappush(heap, item) -> None
pub fn mb_heapq_heappush(heap: MbValue, item: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                items.push(item);
                let pos = items.len() - 1;
                sift_down(&mut items, 0, pos);
            }
        }
    }
    MbValue::none()
}

/// heapq.heappop(heap) -> smallest item
pub fn mb_heapq_heappop(heap: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                // CPython raises IndexError on an empty heap.
                if items.is_empty() {
                    drop(items);
                    return raise_index_error("index out of range");
                }
                let last = items.pop().unwrap();
                if items.is_empty() { return last; }
                let returnitem = items[0];
                items[0] = last;
                sift_up(&mut items, 0);
                return returnitem;
            }
        }
    }
    MbValue::none()
}

/// heapq.heappushpop(heap, item) -> smallest of (item, *heap)
///
/// Equivalent to `heappush` then `heappop` but more efficient: if the
/// new item is smaller than the current min, just return it; otherwise
/// swap with the min and sift the new root down.
pub fn mb_heapq_heappushpop(heap: MbValue, item: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if !items.is_empty() && lt(items[0], item) {
                    let result = items[0];
                    items[0] = item;
                    sift_up(&mut items, 0);
                    return result;
                }
                return item;
            }
        }
    }
    item
}

/// heapq.heapreplace(heap, item) -> smallest of current heap, then push item.
///
/// More efficient than `heappop` + `heappush` because it only sifts once.
/// Returns the original heap root; raises IndexError on empty heap in
/// CPython — mamba returns None for that edge case until exceptions land.
pub fn mb_heapq_heapreplace(heap: MbValue, item: MbValue) -> MbValue {
    if let Some(ptr) = heap.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                // CPython raises IndexError when the heap is empty.
                if items.is_empty() {
                    drop(items);
                    return raise_index_error("index out of range");
                }
                let returnitem = items[0];
                items[0] = item;
                sift_up(&mut items, 0);
                return returnitem;
            }
        }
    }
    MbValue::none()
}

/// heapq.heapify(list) -> None (in-place)
///
/// Build a heap from an arbitrary list in O(n) by sifting up from the
/// last internal node down to the root.
pub fn mb_heapq_heapify(lst: MbValue) -> MbValue {
    if let Some(ptr) = lst.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let n = items.len();
                if n > 1 {
                    let mut i = n / 2;
                    while i > 0 {
                        i -= 1;
                        sift_up(&mut items, i);
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// heapq.nlargest(n, iterable, key=None) -> list of the n largest items,
/// ordered largest-first. With `key=`, items are ranked by `key(item)`
/// (and returned as the items themselves, not their keys). Ties keep
/// original input order — CPython decorates with a descending counter to
/// make `nlargest` stable, so an equal-key run comes out in input order.
pub fn mb_heapq_nlargest(n: MbValue, iterable: MbValue, kwargs: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let key = kwarg(kwargs, "key");
    let items = super::super::builtins::extract_items(iterable);
    // Decorate with original index so the sort is stable on key ties.
    let mut decorated: Vec<(MbValue, usize)> = items
        .into_iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    // Descending by key; ties broken by ascending original index (stable).
    decorated.sort_by(|(a, ia), (b, ib)| {
        match super::super::builtins::mb_value_cmp_pub(apply_key(key, *b), apply_key(key, *a)) {
            std::cmp::Ordering::Equal => ia.cmp(ib),
            ord => ord,
        }
    });
    let mut s: Vec<MbValue> = decorated.into_iter().map(|(v, _)| v).collect();
    s.truncate(count);
    // Elements are borrowed from the source iterable — retain on the way out.
    MbValue::from_ptr(MbObject::new_list_borrowed(s))
}

/// heapq.nsmallest(n, iterable, key=None) -> list of the n smallest items,
/// ordered smallest-first. See `mb_heapq_nlargest` for `key=`/tie rules.
pub fn mb_heapq_nsmallest(n: MbValue, iterable: MbValue, kwargs: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let key = kwarg(kwargs, "key");
    let items = super::super::builtins::extract_items(iterable);
    let mut decorated: Vec<(MbValue, usize)> = items
        .into_iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    // Ascending by key; ties broken by ascending original index (stable).
    decorated.sort_by(|(a, ia), (b, ib)| {
        match super::super::builtins::mb_value_cmp_pub(apply_key(key, *a), apply_key(key, *b)) {
            std::cmp::Ordering::Equal => ia.cmp(ib),
            ord => ord,
        }
    });
    let mut s: Vec<MbValue> = decorated.into_iter().map(|(v, _)| v).collect();
    s.truncate(count);
    // Elements are borrowed from the source iterable — retain on the way out.
    MbValue::from_ptr(MbObject::new_list_borrowed(s))
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_list(vals: &[i64]) -> MbValue {
        let items: Vec<MbValue> = vals.iter().map(|&v| MbValue::from_int(v)).collect();
        MbValue::from_ptr(MbObject::new_list(items))
    }

    fn read_list(val: MbValue) -> Vec<i64> {
        val.as_ptr().map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().iter().filter_map(|v| v.as_int()).collect()
            } else { vec![] }
        }).unwrap_or_default()
    }

    /// Verify the binary-heap invariant: every node is <= its children.
    fn is_heap(vals: &[i64]) -> bool {
        for i in 0..vals.len() {
            let lc = 2 * i + 1;
            let rc = 2 * i + 2;
            if lc < vals.len() && vals[i] > vals[lc] { return false; }
            if rc < vals.len() && vals[i] > vals[rc] { return false; }
        }
        true
    }

    #[test]
    fn test_heapify_satisfies_invariant() {
        let h = mk_list(&[5, 3, 1, 4, 2, 9, 7, 6, 8]);
        mb_heapq_heapify(h);
        assert!(is_heap(&read_list(h)));
    }

    #[test]
    fn test_heappush_pop_yields_sorted() {
        let h = mk_list(&[]);
        for v in [5, 3, 1, 4, 2, 9, 7, 6, 8] {
            mb_heapq_heappush(h, MbValue::from_int(v));
            assert!(is_heap(&read_list(h)));
        }
        let mut out = Vec::new();
        loop {
            let v = mb_heapq_heappop(h);
            if v.is_none() { break; }
            out.push(v.as_int().unwrap());
        }
        assert_eq!(out, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_heappushpop_smaller_returns_input() {
        let h = mk_list(&[5, 6, 7]);
        mb_heapq_heapify(h);
        let r = mb_heapq_heappushpop(h, MbValue::from_int(1));
        assert_eq!(r.as_int(), Some(1));
        assert!(is_heap(&read_list(h)));
    }

    #[test]
    fn test_heappushpop_larger_swaps() {
        let h = mk_list(&[1, 5, 7]);
        mb_heapq_heapify(h);
        let r = mb_heapq_heappushpop(h, MbValue::from_int(10));
        assert_eq!(r.as_int(), Some(1));
        let after = read_list(h);
        assert!(is_heap(&after));
        assert!(after.contains(&10));
        assert!(after.contains(&5));
        assert!(after.contains(&7));
    }

    #[test]
    fn test_heapreplace() {
        let h = mk_list(&[1, 5, 7]);
        mb_heapq_heapify(h);
        let r = mb_heapq_heapreplace(h, MbValue::from_int(3));
        assert_eq!(r.as_int(), Some(1));
        let after = read_list(h);
        assert!(is_heap(&after));
        assert!(after.contains(&3));
    }

    #[test]
    fn test_nlargest() {
        let lst = mk_list(&[1, 5, 3, 4, 2]);
        let r = mb_heapq_nlargest(MbValue::from_int(3), lst, MbValue::none());
        assert_eq!(read_list(r), vec![5, 4, 3]);
    }

    #[test]
    fn test_nsmallest() {
        let lst = mk_list(&[5, 1, 3, 4, 2]);
        let r = mb_heapq_nsmallest(MbValue::from_int(3), lst, MbValue::none());
        assert_eq!(read_list(r), vec![1, 2, 3]);
    }

    #[test]
    fn test_heappop_empty_returns_none() {
        let h = mk_list(&[]);
        assert!(mb_heapq_heappop(h).is_none());
    }

    #[test]
    fn test_merge_three_sorted_iterables() {
        let a = mk_list(&[1, 4, 7]);
        let b = mk_list(&[2, 5, 8]);
        let c = mk_list(&[3, 6, 9]);
        let args = [a, b, c];
        let r = unsafe { dispatch_merge(args.as_ptr(), args.len()) };
        assert_eq!(read_list(r), vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }
}
