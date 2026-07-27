use super::super::value::MbValue;
use super::mb_bool;

// ── Missing builtins (#420) ──

/// any(iterable) — return True if any element is truthy.
pub fn mb_any(args: MbValue) -> MbValue {
    let iter = super::super::iter::mb_iter(args);
    if iter.is_none() || super::super::exception::has_current_exception() {
        return MbValue::none();
    }
    while super::super::iter::mb_has_next(iter).as_bool() == Some(true) {
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        let item = super::super::iter::mb_next(iter);
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        let b = mb_bool(item);
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        if b.as_bool().unwrap_or(false) {
            return MbValue::from_bool(true);
        }
    }
    if super::super::exception::has_current_exception() {
        return MbValue::none();
    }
    MbValue::from_bool(false)
}

/// all(iterable) — return True if all elements are truthy.
pub fn mb_all(args: MbValue) -> MbValue {
    let iter = super::super::iter::mb_iter(args);
    if iter.is_none() || super::super::exception::has_current_exception() {
        return MbValue::none();
    }
    while super::super::iter::mb_has_next(iter).as_bool() == Some(true) {
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        let item = super::super::iter::mb_next(iter);
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        let b = mb_bool(item);
        if super::super::exception::has_current_exception() {
            return MbValue::none();
        }
        if !b.as_bool().unwrap_or(false) {
            return MbValue::from_bool(false);
        }
    }
    if super::super::exception::has_current_exception() {
        return MbValue::none();
    }
    MbValue::from_bool(true)
}
