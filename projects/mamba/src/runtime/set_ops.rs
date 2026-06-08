use super::rc::{MbObject, ObjData};
/// Set operations for Mamba (#386) — thread-safe.
///
/// All mutable set access goes through RwLock guards for thread-safety.
use super::value::MbValue;

#[inline]
fn eq_py(a: MbValue, b: MbValue) -> bool {
    super::builtins::mb_eq(a, b).as_bool() == Some(true)
}

/// CPython rule for binary set operations: the result inherits the type
/// of the *left* operand. `frozenset & set` → frozenset; `set & frozenset`
/// → set. Wrap the resulting `Vec<MbValue>` accordingly.
#[inline]
fn build_set_like_left(left: MbValue, items: Vec<MbValue>) -> MbValue {
    let is_frozen = left
        .as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::FrozenSet(_)) })
        .unwrap_or(false);
    if is_frozen {
        MbValue::from_ptr(MbObject::new_frozenset(items))
    } else {
        MbValue::from_ptr(MbObject::new_set(items))
    }
}

/// Create a new empty set.
pub fn mb_set_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_set(Vec::new()))
}

/// Create a set from a list of elements.
pub fn mb_set_from_list(list: MbValue) -> MbValue {
    let mut items = Vec::new();
    if let Some(ptr) = list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let elems = lock.read().unwrap();
                for elem in elems.iter() {
                    if !items.iter().any(|v: &MbValue| eq_py(*v, *elem)) {
                        items.push(*elem);
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_set(items))
}

/// set.add(elem) — add an element (no-op if already present).
pub fn mb_set_add(set_val: MbValue, elem: MbValue) {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if !items.iter().any(|v| eq_py(*v, elem)) {
                    super::rc::retain_if_ptr(elem);
                    items.push(elem);
                }
            }
        }
    }
}

/// set.remove(elem) — remove an element; raises KeyError if not present.
pub fn mb_set_remove(set_val: MbValue, elem: MbValue) {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if let Some(pos) = items.iter().position(|v| eq_py(*v, elem)) {
                    items.swap_remove(pos);
                    return;
                }
                // CPython raises KeyError(elem) with the bare element, not
                // repr(elem); KeyError.__str__ then applies repr-once when
                // printing the exception. Pre-repring here causes a double
                // quote escape (e.g. `KE: '\'zeta\''` instead of `KE: 'zeta'`).
                let str_val = super::builtins::mb_str(elem);
                let str_s = str_val
                    .as_ptr()
                    .and_then(|p| {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(str_s)),
                );
            }
        }
    }
}

/// set.discard(elem) — remove if present, no error if absent.
pub fn mb_set_discard(set_val: MbValue, elem: MbValue) {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if let Some(pos) = items.iter().position(|v| eq_py(*v, elem)) {
                    items.swap_remove(pos);
                }
            }
        }
    }
}

/// elem in set — check membership.
pub fn mb_set_contains(set_val: MbValue, elem: MbValue) -> MbValue {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                // REQ: R8
                ObjData::Set(ref lock) => {
                    return MbValue::from_bool(
                        lock.read().unwrap().iter().any(|v| eq_py(*v, elem)),
                    );
                }
                // REQ: R8
                ObjData::FrozenSet(ref items) => {
                    return MbValue::from_bool(items.iter().any(|v| eq_py(*v, elem)));
                }
                _ => {}
            }
        }
    }
    MbValue::from_bool(false)
}

/// len(set)
pub fn mb_set_len(set_val: MbValue) -> MbValue {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                // REQ: R8
                ObjData::Set(ref lock) => {
                    return MbValue::from_int(lock.read().unwrap().len() as i64);
                }
                // REQ: R8
                ObjData::FrozenSet(ref items) => {
                    return MbValue::from_int(items.len() as i64);
                }
                _ => {}
            }
        }
    }
    MbValue::from_int(0)
}

/// set.union(other) — return new set with elements from both.
/// Result type matches the LEFT operand (CPython rule).
pub fn mb_set_union(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let mut result = items_a;
    for elem in items_b {
        if !result.iter().any(|v| eq_py(*v, elem)) {
            result.push(elem);
        }
    }
    build_set_like_left(a, result)
}

/// set.intersection(other) — return new set with common elements.
/// Result type matches the LEFT operand (CPython rule).
pub fn mb_set_intersection(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let result: Vec<MbValue> = items_a
        .into_iter()
        .filter(|elem| items_b.iter().any(|v| eq_py(*v, *elem)))
        .collect();
    build_set_like_left(a, result)
}

/// set.difference(other) — return new set with elements in self but not other.
/// Result type matches the LEFT operand (CPython rule).
pub fn mb_set_difference(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let result: Vec<MbValue> = items_a
        .into_iter()
        .filter(|elem| !items_b.iter().any(|v| eq_py(*v, *elem)))
        .collect();
    build_set_like_left(a, result)
}

/// set.symmetric_difference(other) — elements in either but not both.
/// Result type matches the LEFT operand (CPython rule).
pub fn mb_set_symmetric_difference(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let mut result: Vec<MbValue> = items_a
        .iter()
        .filter(|elem| !items_b.iter().any(|v| eq_py(*v, **elem)))
        .copied()
        .collect();
    for elem in &items_b {
        if !items_a.iter().any(|v| eq_py(*v, *elem)) {
            result.push(*elem);
        }
    }
    build_set_like_left(a, result)
}

/// set.pop() — remove and return an arbitrary element; raises KeyError on empty set.
pub fn mb_set_pop(receiver: MbValue) -> MbValue {
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                if items.is_empty() {
                    drop(items);
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("pop from an empty set".to_string())),
                    );
                    return MbValue::none();
                }
                return items.swap_remove(0);
            }
        }
    }
    MbValue::none()
}

/// set.update(other) — in-place union; adds all elements from other (set, list, or tuple).
pub fn mb_set_update(receiver: MbValue, other: MbValue) -> MbValue {
    let new_items: Vec<MbValue> = if let Some(ptr) = other.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Set(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(ref items) => items.clone(),
                _ => vec![],
            }
        }
    } else {
        vec![]
    };

    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                for elem in new_items {
                    if !items.iter().any(|v| eq_py(*v, elem)) {
                        super::rc::retain_if_ptr(elem);
                        items.push(elem);
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// set.intersection_update(other) — in-place; keep only elements also in `other`.
pub fn mb_set_intersection_update(receiver: MbValue, other: MbValue) -> MbValue {
    let other_items = extract_set_items(other);
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let mut i = 0;
                while i < items.len() {
                    if other_items.iter().any(|v| eq_py(*v, items[i])) {
                        i += 1;
                    } else {
                        let dropped = items.swap_remove(i);
                        super::rc::release_if_ptr(dropped);
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// set.difference_update(other) — in-place; remove every element also in `other`.
pub fn mb_set_difference_update(receiver: MbValue, other: MbValue) -> MbValue {
    let other_items = extract_set_items(other);
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let mut i = 0;
                while i < items.len() {
                    if other_items.iter().any(|v| eq_py(*v, items[i])) {
                        let dropped = items.swap_remove(i);
                        super::rc::release_if_ptr(dropped);
                    } else {
                        i += 1;
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// set.symmetric_difference_update(other) — in-place; keep elements in exactly one set.
pub fn mb_set_symmetric_difference_update(receiver: MbValue, other: MbValue) -> MbValue {
    let other_items = extract_set_items(other);
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                // Drop intersection first.
                let mut i = 0;
                let mut shared: Vec<MbValue> = Vec::new();
                while i < items.len() {
                    if other_items.iter().any(|v| eq_py(*v, items[i])) {
                        shared.push(items[i]);
                        let dropped = items.swap_remove(i);
                        super::rc::release_if_ptr(dropped);
                    } else {
                        i += 1;
                    }
                }
                // Add elements unique to `other`.
                for elem in other_items {
                    if !shared.iter().any(|v| eq_py(*v, elem)) {
                        super::rc::retain_if_ptr(elem);
                        items.push(elem);
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// set.clear()
pub fn mb_set_clear(set_val: MbValue) {
    if let Some(ptr) = set_val.as_ptr() {
        unsafe {
            if let ObjData::Set(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
}

/// set.copy() / frozenset.copy() — return shallow copy preserving receiver kind.
pub fn mb_set_copy(set_val: MbValue) -> MbValue {
    let items = extract_set_items(set_val);
    let is_frozen = if let Some(ptr) = set_val.as_ptr() {
        unsafe { matches!((*ptr).data, ObjData::FrozenSet(_)) }
    } else {
        false
    };
    if is_frozen {
        MbValue::from_ptr(MbObject::new_frozenset(items))
    } else {
        MbValue::from_ptr(MbObject::new_set(items))
    }
}

/// set.issubset(other)
pub fn mb_set_issubset(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let result = items_a
        .iter()
        .all(|elem| items_b.iter().any(|v| eq_py(*v, *elem)));
    MbValue::from_bool(result)
}

/// set.issuperset(other)
pub fn mb_set_issuperset(a: MbValue, b: MbValue) -> MbValue {
    mb_set_issubset(b, a)
}

/// set.isdisjoint(other) — return True if the sets have no elements in common.
pub fn mb_set_isdisjoint(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_set_items(a);
    let items_b = extract_set_items(b);
    let has_overlap = items_a
        .iter()
        .any(|elem| items_b.iter().any(|v| eq_py(*v, *elem)));
    MbValue::from_bool(!has_overlap)
}

/// Method dispatch for set objects.
/// Name of the unhashable built-in type for `v`, if it is one of the mutable
/// containers Python rejects as a set member / mapping key (`list`, `dict`,
/// `set`, `bytearray`). Returns `None` for hashable values. Used to surface the
/// `TypeError: unhashable type: '…'` that membership tests (`x in s`,
/// `s.__contains__(x)`) raise on a mutable argument.
fn unhashable_type_name(v: MbValue) -> Option<&'static str> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::List(_) => Some("list"),
            ObjData::Dict(_) => Some("dict"),
            ObjData::Set(_) => Some("set"),
            ObjData::ByteArray(_) => Some("bytearray"),
            _ => None,
        }
    }
}

pub fn dispatch_set_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock
                        .read()
                        .unwrap()
                        .get(i)
                        .copied()
                        .unwrap_or(MbValue::none());
                }
            }
            MbValue::none()
        }
    };
    // REQ: R7 — frozenset immutability: reject mutation methods
    if let Some(ptr) = receiver.as_ptr() {
        if unsafe { matches!((*ptr).data, ObjData::FrozenSet(_)) } {
            match name {
                "add"
                | "remove"
                | "discard"
                | "pop"
                | "clear"
                | "update"
                | "intersection_update"
                | "difference_update"
                | "symmetric_difference_update" => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'frozenset' object has no attribute '{name}'"
                        ))),
                    );
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    match name {
        "add" => {
            mb_set_add(receiver, arg(0));
            MbValue::none()
        }
        "remove" => {
            mb_set_remove(receiver, arg(0));
            MbValue::none()
        }
        "discard" => {
            mb_set_discard(receiver, arg(0));
            MbValue::none()
        }
        "clear" => {
            mb_set_clear(receiver);
            MbValue::none()
        }
        "copy" => mb_set_copy(receiver),
        "union" => mb_set_union(receiver, arg(0)),
        "intersection" => mb_set_intersection(receiver, arg(0)),
        "difference" => mb_set_difference(receiver, arg(0)),
        "symmetric_difference" => mb_set_symmetric_difference(receiver, arg(0)),
        "pop" => mb_set_pop(receiver),
        "update" => mb_set_update(receiver, arg(0)),
        "intersection_update" => mb_set_intersection_update(receiver, arg(0)),
        "difference_update" => mb_set_difference_update(receiver, arg(0)),
        "symmetric_difference_update" => mb_set_symmetric_difference_update(receiver, arg(0)),
        "issubset" => mb_set_issubset(receiver, arg(0)),
        "issuperset" => mb_set_issuperset(receiver, arg(0)),
        "isdisjoint" => mb_set_isdisjoint(receiver, arg(0)),
        "__contains__" => {
            let elem = arg(0);
            if let Some(tn) = unhashable_type_name(elem) {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!("unhashable type: '{tn}'"))),
                );
                return MbValue::none();
            }
            mb_set_contains(receiver, elem)
        }
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'set' object has no attribute '{name}'"
                ))),
            );
            MbValue::none()
        }
    }
}

// REQ: R6 — extract_set_items handles both Set and FrozenSet
fn extract_set_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Set(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::FrozenSet(ref items) => return items.clone(),
                // CPython's set methods (difference, difference_update,
                // intersection_update, issubset, isdisjoint, …) accept any
                // iterable; mirror that for the most common iterable shapes.
                ObjData::List(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(ref items) => return items.clone(),
                _ => {}
            }
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_add_contains() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_add(set, MbValue::from_int(2));
        mb_set_add(set, MbValue::from_int(1)); // duplicate

        assert_eq!(mb_set_len(set).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(3)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_remove() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_add(set, MbValue::from_int(2));
        mb_set_remove(set, MbValue::from_int(1));
        assert_eq!(mb_set_len(set).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_union() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));

        let result = mb_set_union(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(3));
    }

    #[test]
    fn test_set_intersection() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));

        let result = mb_set_intersection(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(1));
    }

    // ── mb_set_new ──

    #[test]
    fn test_set_new_is_empty() {
        let set = mb_set_new();
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    // ── mb_set_discard ──

    #[test]
    fn test_set_discard_existing() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(10));
        mb_set_add(set, MbValue::from_int(20));
        mb_set_discard(set, MbValue::from_int(10));
        assert_eq!(mb_set_len(set).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(10)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_discard_absent_no_error() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_discard(set, MbValue::from_int(99)); // not present — should not panic
        assert_eq!(mb_set_len(set).as_int(), Some(1));
    }

    // ── mb_set_remove edge case ──

    #[test]
    fn test_set_remove_absent_no_panic() {
        let set = mb_set_new();
        mb_set_remove(set, MbValue::from_int(42)); // empty set — no-op
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    // ── mb_set_contains on empty ──

    #[test]
    fn test_set_contains_empty() {
        let set = mb_set_new();
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_contains_non_set_value() {
        // Passing a non-set MbValue should return false.
        let val = MbValue::from_int(42);
        assert_eq!(
            mb_set_contains(val, MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    // ── mb_set_len on non-set ──

    #[test]
    fn test_set_len_non_set() {
        let val = MbValue::from_int(5);
        assert_eq!(mb_set_len(val).as_int(), Some(0));
    }

    // ── mb_set_clear ──

    #[test]
    fn test_set_clear() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_add(set, MbValue::from_int(2));
        mb_set_add(set, MbValue::from_int(3));
        mb_set_clear(set);
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    #[test]
    fn test_set_clear_empty() {
        let set = mb_set_new();
        mb_set_clear(set); // clearing empty set — no-op
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    // ── mb_set_copy ──

    #[test]
    fn test_set_copy() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_add(set, MbValue::from_int(2));

        let copy = mb_set_copy(set);
        assert_eq!(mb_set_len(copy).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(copy, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(copy, MbValue::from_int(2)).as_bool(),
            Some(true)
        );

        // Mutating original should not affect the copy.
        mb_set_add(set, MbValue::from_int(3));
        assert_eq!(mb_set_len(copy).as_int(), Some(2));
    }

    // ── mb_set_difference ──

    #[test]
    fn test_set_difference() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(4));

        let result = mb_set_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_difference_disjoint() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));

        let result = mb_set_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_set_difference_empty_rhs() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        let result = mb_set_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
    }

    // ── mb_set_symmetric_difference ──

    #[test]
    fn test_set_symmetric_difference() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));

        let result = mb_set_symmetric_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_symmetric_difference_identical() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        let result = mb_set_symmetric_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(0));
    }

    #[test]
    fn test_set_symmetric_difference_disjoint() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));

        let result = mb_set_symmetric_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
    }

    // ── mb_set_issubset ──

    #[test]
    fn test_set_issubset_true() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        assert_eq!(mb_set_issubset(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_set_issubset_false() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(3));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        assert_eq!(mb_set_issubset(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_set_issubset_equal_sets() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        assert_eq!(mb_set_issubset(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_set_issubset_empty_is_subset_of_any() {
        let a = mb_set_new();
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));

        assert_eq!(mb_set_issubset(a, b).as_bool(), Some(true));
    }

    // ── mb_set_issuperset ──

    #[test]
    fn test_set_issuperset_true() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        assert_eq!(mb_set_issuperset(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_set_issuperset_false() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));

        assert_eq!(mb_set_issuperset(a, b).as_bool(), Some(false));
    }

    // ── union / intersection with empty sets ──

    #[test]
    fn test_set_union_with_empty() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();

        let result = mb_set_union(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_set_union_both_empty() {
        let a = mb_set_new();
        let b = mb_set_new();
        let result = mb_set_union(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(0));
    }

    #[test]
    fn test_set_intersection_disjoint() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));

        let result = mb_set_intersection(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(0));
    }

    #[test]
    fn test_set_intersection_with_empty() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));

        let b = mb_set_new();
        let result = mb_set_intersection(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(0));
    }

    // ── dispatch_set_method ──

    fn make_args(vals: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    #[test]
    fn test_dispatch_add() {
        let set = mb_set_new();
        let result = dispatch_set_method("add", set, make_args(vec![MbValue::from_int(5)]));
        assert!(result.is_none());
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(5)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_remove() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(5));
        let result = dispatch_set_method("remove", set, make_args(vec![MbValue::from_int(5)]));
        assert!(result.is_none());
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    #[test]
    fn test_dispatch_discard() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(7));
        let result = dispatch_set_method("discard", set, make_args(vec![MbValue::from_int(7)]));
        assert!(result.is_none());
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    #[test]
    fn test_dispatch_clear() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        let result = dispatch_set_method("clear", set, make_args(vec![]));
        assert!(result.is_none());
        assert_eq!(mb_set_len(set).as_int(), Some(0));
    }

    #[test]
    fn test_dispatch_copy() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        let copy = dispatch_set_method("copy", set, make_args(vec![]));
        assert_eq!(mb_set_len(copy).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(copy, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_union() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        let result = dispatch_set_method("union", a, make_args(vec![b]));
        assert_eq!(mb_set_len(result).as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_intersection() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        let result = dispatch_set_method("intersection", a, make_args(vec![b]));
        assert_eq!(mb_set_len(result).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_difference() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        let result = dispatch_set_method("difference", a, make_args(vec![b]));
        assert_eq!(mb_set_len(result).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_symmetric_difference() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        let result = dispatch_set_method("symmetric_difference", a, make_args(vec![b]));
        assert_eq!(mb_set_len(result).as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_issubset() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));
        let result = dispatch_set_method("issubset", a, make_args(vec![b]));
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_issuperset() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        let result = dispatch_set_method("issuperset", a, make_args(vec![b]));
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_unknown_method() {
        super::super::exception::mb_clear_exception();
        let set = mb_set_new();
        let result = dispatch_set_method("nonexistent", set, make_args(vec![]));
        assert!(result.is_none());
        // Should have set an exception via mb_raise.
        let has_exc = super::super::exception::mb_has_exception();
        assert_eq!(has_exc.as_bool(), Some(true));
        super::super::exception::mb_clear_exception();
    }

    // ── frozenset: immutability (R7 — mutations raise AttributeError) ──

    // REQ: R7
    #[test]
    fn test_frozenset_add_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        // R7: dispatch_set_method must raise AttributeError for mutation methods on frozenset
        let result = dispatch_set_method("add", fs, make_args(vec![MbValue::from_int(2)]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        // frozenset is still intact: R8 len works
        assert_eq!(mb_set_len(fs).as_int(), Some(1));
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7
    #[test]
    fn test_frozenset_remove_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        let result = dispatch_set_method("remove", fs, make_args(vec![MbValue::from_int(1)]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7
    #[test]
    fn test_frozenset_clear_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        let result = dispatch_set_method("clear", fs, make_args(vec![]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::exception::mb_clear_exception();
    }

    // ── extract_set_items: non-set returns empty vec ──

    #[test]
    fn test_extract_set_items_non_set() {
        // extract_set_items is private, but we exercise it through union with a non-set.
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let non_set = MbValue::from_int(99);
        let result = mb_set_union(a, non_set);
        // union with a non-set should treat it as empty.
        assert_eq!(mb_set_len(result).as_int(), Some(1));
    }

    // ── multiple adds / removes round-trip ──

    #[test]
    fn test_set_add_remove_add_roundtrip() {
        let set = mb_set_new();
        mb_set_add(set, MbValue::from_int(1));
        mb_set_remove(set, MbValue::from_int(1));
        assert_eq!(mb_set_len(set).as_int(), Some(0));
        mb_set_add(set, MbValue::from_int(1));
        assert_eq!(mb_set_len(set).as_int(), Some(1));
        assert_eq!(
            mb_set_contains(set, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
    }

    // ── union returns a new set (not aliased) ──

    #[test]
    fn test_set_union_returns_new_set() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        let result = mb_set_union(a, b);

        // Mutating the result should not affect a or b.
        mb_set_add(result, MbValue::from_int(3));
        assert_eq!(mb_set_len(a).as_int(), Some(1));
        assert_eq!(mb_set_len(b).as_int(), Some(1));
        assert_eq!(mb_set_len(result).as_int(), Some(3));
    }

    // ── intersection contents verification ──

    #[test]
    fn test_set_intersection_contents() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));
        mb_set_add(b, MbValue::from_int(4));

        let result = mb_set_intersection(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(false)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(4)).as_bool(),
            Some(false)
        );
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_set_symmetric_difference() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(3));
        mb_set_add(b, MbValue::from_int(4));
        let result = mb_set_symmetric_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(3));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(4)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_py312_set_issubset() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));
        assert_eq!(mb_set_issubset(a, b).as_bool(), Some(true));
        assert_eq!(mb_set_issubset(b, a).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_set_issuperset() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        assert_eq!(mb_set_issuperset(a, b).as_bool(), Some(true));
        assert_eq!(mb_set_issuperset(b, a).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_set_isdisjoint_true() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(3));
        mb_set_add(b, MbValue::from_int(4));
        // isdisjoint: intersection should be empty
        let inter = mb_set_intersection(a, b);
        assert_eq!(mb_set_len(inter).as_int(), Some(0));
    }

    #[test]
    fn test_py312_set_isdisjoint_false() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(1));
        mb_set_add(b, MbValue::from_int(2));
        // not disjoint: intersection should be non-empty
        let inter = mb_set_intersection(a, b);
        assert!(mb_set_len(inter).as_int().unwrap() > 0);
    }

    // ── mb_set_isdisjoint ──

    // REQ: R3
    #[test]
    fn test_set_isdisjoint_true_returns_true() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(3));
        mb_set_add(b, MbValue::from_int(4));

        assert_eq!(mb_set_isdisjoint(a, b).as_bool(), Some(true));
    }

    // REQ: R3
    #[test]
    fn test_set_isdisjoint_false_returns_false() {
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));

        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));

        assert_eq!(mb_set_isdisjoint(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_set_discard_no_error() {
        let s = mb_set_new();
        mb_set_add(s, MbValue::from_int(1));
        mb_set_discard(s, MbValue::from_int(99));
        assert_eq!(mb_set_len(s).as_int(), Some(1));
    }

    #[test]
    fn test_py312_set_update() {
        // set.update(other) — add all elements from other into self
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(3));
        // Simulate update by extracting items from b and adding to a
        let union = mb_set_union(a, b);
        assert_eq!(mb_set_len(union).as_int(), Some(3));
    }

    #[test]
    fn test_py312_set_difference_update() {
        // difference_update: remove elements found in other
        let a = mb_set_new();
        mb_set_add(a, MbValue::from_int(1));
        mb_set_add(a, MbValue::from_int(2));
        mb_set_add(a, MbValue::from_int(3));
        let b = mb_set_new();
        mb_set_add(b, MbValue::from_int(2));
        mb_set_add(b, MbValue::from_int(4));
        // Simulate difference_update via mb_set_difference (returns new set)
        let result = mb_set_difference(a, b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_set_from_list_deduplicates() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(1),
            MbValue::from_int(3),
            MbValue::from_int(2),
        ]));
        let s = mb_set_from_list(list);
        assert_eq!(mb_set_len(s).as_int(), Some(3));
        assert_eq!(
            mb_set_contains(s, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(s, MbValue::from_int(2)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(s, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
    }

    // ── frozenset R8: len and contains ──

    // REQ: R8
    #[test]
    fn test_frozenset_len_and_contains() {
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]));
        assert_eq!(mb_set_len(fs).as_int(), Some(3));
        assert_eq!(
            mb_set_contains(fs, MbValue::from_int(20)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(fs, MbValue::from_int(99)).as_bool(),
            Some(false)
        );
    }

    // ── frozenset R6: algebra operations ──

    // REQ: R6
    #[test]
    fn test_frozenset_union() {
        let fs_a = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let fs_b = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let result = mb_set_union(fs_a, fs_b);
        assert_eq!(mb_set_len(result).as_int(), Some(3));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(1)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
    }

    // REQ: R6
    #[test]
    fn test_frozenset_intersection() {
        let fs_a = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let fs_b = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]));
        let result = mb_set_intersection(fs_a, fs_b);
        assert_eq!(mb_set_len(result).as_int(), Some(2));
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(2)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_set_contains(result, MbValue::from_int(3)).as_bool(),
            Some(true)
        );
    }

    // REQ: R6
    #[test]
    fn test_frozenset_issubset() {
        let fs_a = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let fs_b = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        assert_eq!(mb_set_issubset(fs_a, fs_b).as_bool(), Some(true));
        assert_eq!(mb_set_issubset(fs_b, fs_a).as_bool(), Some(false));
    }

    // REQ: R7
    #[test]
    fn test_frozenset_discard_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        let result = dispatch_set_method("discard", fs, make_args(vec![MbValue::from_int(1)]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7
    #[test]
    fn test_frozenset_pop_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        let result = dispatch_set_method("pop", fs, make_args(vec![]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7
    #[test]
    fn test_frozenset_update_raises_attribute_error() {
        super::super::exception::mb_clear_exception();
        let fs = MbValue::from_ptr(MbObject::new_frozenset(vec![MbValue::from_int(1)]));
        let other = mb_set_new();
        mb_set_add(other, MbValue::from_int(2));
        let result = dispatch_set_method("update", fs, make_args(vec![other]));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7 — read-only methods still work on frozenset
    #[test]
    fn test_frozenset_readonly_methods_work() {
        let fs_a = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let fs_b = MbValue::from_ptr(MbObject::new_frozenset(vec![
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        // union via dispatch
        let union_result = dispatch_set_method("union", fs_a, make_args(vec![fs_b]));
        assert_eq!(mb_set_len(union_result).as_int(), Some(3));
        // issubset via dispatch
        let sub_result = dispatch_set_method("issubset", fs_a, make_args(vec![fs_b]));
        // {1,2} is NOT a subset of {2,3}
        assert_eq!(sub_result.as_bool(), Some(false));
    }
}
