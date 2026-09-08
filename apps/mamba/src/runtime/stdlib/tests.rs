//! Colocated unit tests for `re_mod`'s `re.Match` group accessor (issue
//! #4242).
//!
//! `mb_re_match_getitem` (backing `.group(i)` / `m[i]` / named-group lookup)
//! used to return the `MbValue` stored in the `re.Match` instance's own
//! field map by copy, without retaining it. Every other native accessor in
//! this runtime that hands back a value it does not otherwise own (e.g. list
//! indexing in `class::mod.rs`) calls `rc::retain_if_ptr` first, because the
//! caller's generated code assumes the returned value is a fresh, separately
//! owned reference and will release it exactly once. Skipping the retain
//! left the `re.Match` field dict and the caller sharing one refcount: once
//! the `re.Match` instance itself was dropped (its own fields released), the
//! string the caller still held had already been freed — observed as a
//! crash (`apps/mamba/e2e/tier1/concurrency/safety/re/match_group_in_child_does_not_corrupt_heap.py`)
//! whenever a worker thread built a match, read a group, and the caller's
//! copy of that string outlived the match object.
//!
//! This test does not depend on threading: it pins the refcount contract
//! directly at the accessor, which is the same contract a single-threaded
//! caller relies on too.

use crate::runtime::rc::{MbObject, ObjData};
use crate::runtime::stdlib::re_mod::{mb_re_match_getitem, mb_re_search};
use crate::runtime::value::MbValue;
use std::sync::atomic::Ordering;

fn refcount_of(val: MbValue) -> u32 {
    let ptr = val.as_ptr().expect("expected heap pointer");
    unsafe { (*ptr).header.rc.load(Ordering::SeqCst) }
}

/// `match.group(1)` must hand back a group string with its own retained
/// reference: the `re.Match` instance's field map keeps its own reference,
/// so the string's refcount must read >= 2 once the caller also holds it,
/// not 1 (which would mean the field map and the caller share a single
/// count, and whichever side releases first frees memory the other side
/// still reads).
#[test]
fn match_group_retains_returned_string_refcount() {
    let pattern = MbValue::from_ptr(MbObject::new_str(r"(\d+)".to_string()));
    let subject = MbValue::from_ptr(MbObject::new_str("xy 77".to_string()));

    let m = mb_re_search(pattern, subject);
    assert!(m.as_ptr().is_some(), "expected a Match instance, got None");

    let group1 = mb_re_match_getitem(m, MbValue::from_int(1));
    let ptr = group1.as_ptr().expect("group(1) should be a heap string");
    let text = unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => s.clone(),
            _ => panic!("expected Str"),
        }
    };
    assert_eq!(text, "77");

    // The field map inside `m` still owns one reference to this same
    // object; the accessor must have added a second one for the caller.
    assert!(
        refcount_of(group1) >= 2,
        "match.group(1) must retain the returned string; got refcount {}",
        refcount_of(group1)
    );

    // Named-group lookup goes through the same field-copy path and must
    // retain too. Group 0 (the full match) exercises the int-key branch a
    // second time with a different key.
    let group0 = mb_re_match_getitem(m, MbValue::from_int(0));
    assert!(
        refcount_of(group0) >= 2,
        "match.group(0) must retain the returned string; got refcount {}",
        refcount_of(group0)
    );
}
