use super::super::value::MbValue;
use super::mb_bool;

// ── Missing builtins (#420) ──

/// any(iterable) — return True if any element is truthy.
pub fn mb_any(args: MbValue) -> MbValue {
    let iter = super::super::iter::mb_iter(args);
    if iter.is_none() {
        return MbValue::none();
    }
    while super::super::iter::mb_has_next(iter).as_bool() == Some(true) {
        let item = super::super::iter::mb_next(iter);
        if mb_bool(item).as_bool().unwrap_or(false) {
            return MbValue::from_bool(true);
        }
    }
    MbValue::from_bool(false)
}

/// all(iterable) — return True if all elements are truthy.
pub fn mb_all(args: MbValue) -> MbValue {
    let iter = super::super::iter::mb_iter(args);
    if iter.is_none() {
        return MbValue::none();
    }
    while super::super::iter::mb_has_next(iter).as_bool() == Some(true) {
        let item = super::super::iter::mb_next(iter);
        if !mb_bool(item).as_bool().unwrap_or(false) {
            return MbValue::from_bool(false);
        }
    }
    MbValue::from_bool(true)
}
