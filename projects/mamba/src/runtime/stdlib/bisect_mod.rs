//! @codegen-skip: handwrite-pre-standardize
//!
//! bisect module for Mamba — Python 3.12 `bisect` stdlib.
//!
//! Provides `bisect_left`, `bisect_right`, `insort_left`, `insort_right`
//! over arbitrary ordered MbValue sequences (int / float / mixed numeric
//! / str / list / tuple, per `mb_lt`).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_binary + value-ordered binary
//! search) is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1414 cluster anchor + see
//! `.aw/handoffs/1414-patrol-handoff.md`.
//!
//! The previous shim registered each attribute as a *string* MbValue
//! ("mb_bisect_bisect_left", etc.) which made `bisect.bisect_left(...)`
//! raise `AttributeError: 'dict' object has no attribute 'bisect_left'`
//! at import-time (caller would get a non-callable str). The fix wires
//! real `dispatch_*` extern "C" thunks through `NATIVE_FUNC_ADDRS` so
//! mamba's callable-resolution can find them — same shape as
//! `keyword_mod` and `struct_mod`.
use super::super::builtins::mb_lt;
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

fn is_bisect_sequence(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_) | ObjData::Tuple(_)) })
}

unsafe extern "C" fn dispatch_bisect_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    if !is_bisect_sequence(seq) {
        return raise_type_error("bisect_left() argument 1 must be a sequence");
    }
    mb_bisect_bisect_left(seq, a.get(1).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_bisect_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    if !is_bisect_sequence(seq) {
        return raise_type_error("bisect_right() argument 1 must be a sequence");
    }
    mb_bisect_bisect_right(seq, a.get(1).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_insort_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    if !is_bisect_sequence(seq) {
        return raise_type_error("insort_left() argument 1 must be a sequence");
    }
    mb_bisect_insort_left(seq, a.get(1).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_insort_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    if !is_bisect_sequence(seq) {
        return raise_type_error("insort_right() argument 1 must be a sequence");
    }
    mb_bisect_insort_right(seq, a.get(1).copied().unwrap_or_else(MbValue::none))
}

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("bisect_left", dispatch_bisect_left as usize),
        ("bisect_right", dispatch_bisect_right as usize),
        // CPython provides `bisect` as an alias for `bisect_right`, and
        // `insort` for `insort_right`. Mirror that here.
        ("bisect", dispatch_bisect_right as usize),
        ("insort_left", dispatch_insort_left as usize),
        ("insort_right", dispatch_insort_right as usize),
        ("insort", dispatch_insort_right as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("bisect", attrs);
}

/// True iff `a < b` per mamba's ordering primitives (int/float/mixed numeric,
/// str, list, tuple). Used as the only ordering relation for binary search,
/// matching CPython `bisect` which calls `__lt__` only.
#[inline]
fn lt(a: MbValue, b: MbValue) -> bool {
    mb_lt(a, b).as_bool() == Some(true)
}

/// Run `f` against a borrowed view of the list elements held by `a`,
/// holding the read lock for the duration of the closure. Returns
/// `f(&[])` for non-sequence values in direct helper calls; the public module
/// dispatchers validate call-boundary TypeError behavior.
///
/// Borrowing instead of cloning is essential: the binary-search hot
/// loop does ~log2(n) comparisons, but a naive `read_list()` would
/// clone all n MbValues per call — making `bisect_left` on a 1000-elem
/// list ~100× slower than CPython instead of ~equal.
fn with_list<R>(a: MbValue, f: impl FnOnce(&[MbValue]) -> R) -> R
where
    R: Default,
{
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                return f(guard.as_slice());
            } else if let ObjData::Tuple(ref items) = (*ptr).data {
                return f(items.as_slice());
            }
        }
    }
    f(&[])
}

pub fn mb_bisect_bisect_left(a: MbValue, x: MbValue) -> MbValue {
    let pos = with_list(a, |items| {
        let (mut lo, mut hi) = (0usize, items.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if lt(items[mid], x) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    });
    MbValue::from_int(pos as i64)
}

pub fn mb_bisect_bisect_right(a: MbValue, x: MbValue) -> MbValue {
    let pos = with_list(a, |items| {
        let (mut lo, mut hi) = (0usize, items.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            // CPython's bisect_right inserts AFTER any equal elements, i.e.
            // the loop advances `lo` whenever `x !< items[mid]`. With a
            // strict-less-than primitive, that condition is `!lt(x, items[mid])`.
            if lt(x, items[mid]) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    });
    MbValue::from_int(pos as i64)
}

pub fn mb_bisect_insort_left(a: MbValue, x: MbValue) -> MbValue {
    let pos = mb_bisect_bisect_left(a, x).as_int().unwrap_or(0) as usize;
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().insert(pos, x);
            }
        }
    }
    MbValue::none()
}

pub fn mb_bisect_insort_right(a: MbValue, x: MbValue) -> MbValue {
    let pos = mb_bisect_bisect_right(a, x).as_int().unwrap_or(0) as usize;
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().insert(pos, x);
            }
        }
    }
    MbValue::none()
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::super::super::rc::{MbObject, ObjData};
    use super::super::super::value::MbValue;
    use super::*;

    fn make_int_list(items: &[i64]) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|&i| MbValue::from_int(i)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    fn list_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    }

    fn list_len(val: MbValue) -> usize {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read().unwrap().len()
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[test]
    fn test_bisect_left_duplicates() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(2)).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_bisect_right_duplicates() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a, MbValue::from_int(2)).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_bisect_boundary_before() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(0)).as_int(),
            Some(0)
        );
        let a2 = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a2, MbValue::from_int(0)).as_int(),
            Some(0)
        );
    }

    #[test]
    fn test_bisect_boundary_after() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(4)).as_int(),
            Some(4)
        );
        let a2 = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a2, MbValue::from_int(4)).as_int(),
            Some(4)
        );
    }

    #[test]
    fn test_insort_left() {
        let a = make_int_list(&[1, 3]);
        mb_bisect_insort_left(a, MbValue::from_int(2));
        assert_eq!(list_len(a), 3);
        assert_eq!(list_int_at(a, 0), Some(1));
        assert_eq!(list_int_at(a, 1), Some(2));
        assert_eq!(list_int_at(a, 2), Some(3));
    }

    #[test]
    fn test_insort_right() {
        let a = make_int_list(&[1, 2, 3]);
        mb_bisect_insort_right(a, MbValue::from_int(2));
        assert_eq!(list_len(a), 4);
        assert_eq!(list_int_at(a, 1), Some(2));
        assert_eq!(list_int_at(a, 2), Some(2));
        mb_bisect_insort_left(MbValue::none(), MbValue::from_int(1));
        mb_bisect_insort_right(MbValue::none(), MbValue::from_int(1));
    }

    #[test]
    fn test_float_ordering_no_truncation() {
        // Regression: old shim mapped float→i64 via item_key, so 1.4 was
        // treated as equal to 1, producing wrong positions. With strict
        // `lt`, 1.4 sorts between 1 and 2 — bisect_left must return 1.
        let a = make_int_list(&[1, 2, 3]);
        // Insert a float value
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_float(1.4)).as_int(),
            Some(1),
        );
    }
}
