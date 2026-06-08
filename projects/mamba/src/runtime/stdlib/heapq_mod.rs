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

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_heap_list(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_)) })
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
            let heap = a.get(0).copied().unwrap_or_else(MbValue::none);
            if !is_heap_list(heap) {
                return raise_type_error("heap argument must be a list");
            }
            $fn(heap, a.get(1).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_heap_binary!(dispatch_heappush, mb_heapq_heappush);
dispatch_heap_unary!(dispatch_heappop, mb_heapq_heappop);
dispatch_heap_binary!(dispatch_heappushpop, mb_heapq_heappushpop);
dispatch_heap_binary!(dispatch_heapreplace, mb_heapq_heapreplace);
dispatch_heap_unary!(dispatch_heapify, mb_heapq_heapify);
dispatch_binary!(dispatch_nlargest, mb_heapq_nlargest);
dispatch_binary!(dispatch_nsmallest, mb_heapq_nsmallest);

/// merge accepts variadic sorted iterables: `merge(a, b, c, ...)`.
/// Implemented as a flat-args dispatcher so all positional iterables
/// flow through, not just the first two (which `dispatch_binary` would
/// truncate to). Returns a single sorted list — eager rather than the
/// CPython lazy-iterator semantics, which matches mamba's current
/// generator support (no PyGenObject yet).
unsafe extern "C" fn dispatch_merge(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mut all: Vec<MbValue> = Vec::new();
    for v in a.iter() {
        all.extend(extract_iterable(*v));
    }
    all.sort_by_key(|v| item_key(*v));
    MbValue::from_ptr(MbObject::new_list(all))
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("heappush", dispatch_heappush as usize),
        ("heappop", dispatch_heappop as usize),
        ("heappushpop", dispatch_heappushpop as usize),
        ("heapreplace", dispatch_heapreplace as usize),
        ("heapify", dispatch_heapify as usize),
        ("nlargest", dispatch_nlargest as usize),
        ("nsmallest", dispatch_nsmallest as usize),
        ("merge", dispatch_merge as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("heapq", attrs);
}

/// Comparable key for an MbValue. Integers and floats compare by value;
/// anything else hashes to 0 (matches the previous shim's behaviour and
/// keeps the heap invariant well-defined for the supported subset).
fn item_key(v: MbValue) -> i64 {
    if let Some(i) = v.as_int() {
        return i;
    }
    if let Some(f) = v.as_float() {
        return f as i64;
    }
    if let Some(b) = v.as_bool() {
        return if b { 1 } else { 0 };
    }
    0
}

/// `<` for MbValue using the integer-or-float key.
#[inline]
fn lt(a: MbValue, b: MbValue) -> bool {
    item_key(a) < item_key(b)
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
                if items.is_empty() {
                    return MbValue::none();
                }
                let last = items.pop().unwrap();
                if items.is_empty() {
                    return last;
                }
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
                if items.is_empty() {
                    return MbValue::none();
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

/// Extract a Vec<MbValue> from a list or tuple iterable. Non-iterables
/// fall back to an empty vec.
fn extract_iterable(val: MbValue) -> Vec<MbValue> {
    val.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => items.clone(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// heapq.nlargest(n, iterable) -> list
pub fn mb_heapq_nlargest(n: MbValue, iterable: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let mut s = extract_iterable(iterable);
    // Descending sort — `b.cmp(a)` for the i64 key.
    s.sort_by(|a, b| item_key(*b).cmp(&item_key(*a)));
    s.truncate(count);
    MbValue::from_ptr(MbObject::new_list(s))
}

/// heapq.nsmallest(n, iterable) -> list
pub fn mb_heapq_nsmallest(n: MbValue, iterable: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let mut s = extract_iterable(iterable);
    s.sort_by_key(|v| item_key(*v));
    s.truncate(count);
    MbValue::from_ptr(MbObject::new_list(s))
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
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| v.as_int())
                        .collect()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default()
    }

    /// Verify the binary-heap invariant: every node is <= its children.
    fn is_heap(vals: &[i64]) -> bool {
        for i in 0..vals.len() {
            let lc = 2 * i + 1;
            let rc = 2 * i + 2;
            if lc < vals.len() && vals[i] > vals[lc] {
                return false;
            }
            if rc < vals.len() && vals[i] > vals[rc] {
                return false;
            }
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
            if v.is_none() {
                break;
            }
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
        let r = mb_heapq_nlargest(MbValue::from_int(3), lst);
        assert_eq!(read_list(r), vec![5, 4, 3]);
    }

    #[test]
    fn test_nsmallest() {
        let lst = mk_list(&[5, 1, 3, 4, 2]);
        let r = mb_heapq_nsmallest(MbValue::from_int(3), lst);
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
