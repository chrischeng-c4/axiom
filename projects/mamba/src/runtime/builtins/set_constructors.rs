use crate::runtime::rc::{retain_if_ptr, MbObject, ObjData};
use crate::runtime::value::MbValue;

use super::{extract_items, mb_values_eq};

/// frozenset() — create an empty frozenset (zero-arg fast path).
pub fn mb_frozenset_empty() -> MbValue {
    MbValue::from_ptr(MbObject::new_frozenset(Vec::new()))
}

/// frozenset(iterable) — create an immutable frozenset from an iterable.
pub fn mb_frozenset_new(args: MbValue) -> MbValue {
    if args.is_none() {
        return MbValue::from_ptr(MbObject::new_frozenset(vec![]));
    }
    let items = extract_items(args);
    // Dedup via Python-semantic equality (dispatches __eq__ on instances).
    let mut unique: Vec<MbValue> = Vec::new();
    for item in items {
        if !unique.iter().any(|v| mb_values_eq(*v, item)) {
            unique.push(item);
        }
    }
    MbValue::from_ptr(MbObject::new_frozenset(unique))
}

/// set(iterable) — create a mutable set from an iterable.
pub fn mb_set_from_iterable(args: MbValue) -> MbValue {
    if args.is_none() {
        return MbValue::from_ptr(MbObject::new_set(vec![]));
    }
    // Heap-container sources (list/tuple/set/frozenset/dict) lend their
    // elements: extract_items copies the MbValues without retaining, so the
    // set must retain what it keeps — otherwise releasing the source (e.g. a
    // temporary list from a method call inside a function) leaves the set
    // holding dangling pointers. Iterator/str sources hand over fresh values.
    let borrowed_source = args.as_ptr().is_some_and(|p| unsafe {
        matches!(
            (*p).data,
            ObjData::List(_)
                | ObjData::Tuple(_)
                | ObjData::Set(_)
                | ObjData::FrozenSet(_)
                | ObjData::Dict(_)
        )
    });
    let items = extract_items(args);
    let mut unique: Vec<MbValue> = Vec::new();
    for item in items {
        if !unique.iter().any(|v| mb_values_eq(*v, item)) {
            if borrowed_source {
                unsafe { retain_if_ptr(item) };
            }
            unique.push(item);
        }
    }
    MbValue::from_ptr(MbObject::new_set(unique))
}
