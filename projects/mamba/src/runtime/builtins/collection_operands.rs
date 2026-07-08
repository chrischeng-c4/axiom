use super::super::{class::builtin_data_payload, rc::ObjData, value::MbValue};

/// Unwrap a set/frozenset-SUBCLASS instance (`class Foo(set): pass`, stored
/// as `ObjData::Instance` carrying a hidden builtin-data payload field) to
/// its raw `ObjData::Set`/`FrozenSet` payload, for the `-`/`|`/`&`/`^`
/// operator dispatch below. Returns the value unchanged when it is already a
/// raw set/frozenset; `None` when it isn't set-like at all. Without this, a
/// set/frozenset-subclass operand was invisible to the `a_is_setlike`/
/// `b_is_setlike` checks in `mb_sub`/`mb_bitand`/`mb_bitor`/`mb_bitxor`
/// (which only matched the raw ObjData kind), so e.g. `plain_set -
/// SetSubclass(...)` raised "unsupported operand type(s)" even though
/// CPython performs the operation — CPython's binary set operators always
/// drop subclass typing, returning a plain set/frozenset keyed off the LEFT
/// operand's raw kind (`build_set_like_left`), so passing the unwrapped
/// payload through is correct here (#975).
pub(super) fn set_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Set(_) | ObjData::FrozenSet(_)) {
                return Some(value);
            }
        }
    }
    match builtin_data_payload(value) {
        Some(("set", payload)) | Some(("frozenset", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a list-SUBCLASS instance (`class LS(list): pass`) to its raw
/// `ObjData::List` payload, for the `+`/`*` concatenation/repetition
/// fallbacks in `mb_add`/`mb_mul`. Returns the value unchanged when it is
/// already a raw list; `None` when it isn't list-like at all. Mirrors
/// `set_like_operand` (#975): without this, a list-subclass operand was
/// invisible to the raw `ObjData::List` match (it's `ObjData::Instance`
/// wrapping a hidden payload field), so e.g. `LS([1]) + [2]` produced None
/// instead of CPython's plain `[1, 2]` — CPython's binary list operators
/// always drop subclass typing here (unlike `+=`/`*=`, which mutate the
/// receiver in place and DO preserve it; see `mb_inplace` in class.rs).
/// (#1026)
pub(super) fn list_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::List(_)) {
                return Some(value);
            }
        }
    }
    match builtin_data_payload(value) {
        Some(("list", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a tuple-SUBCLASS instance (`class TS(tuple): pass`) to its raw
/// `ObjData::Tuple` payload, for the `+`/`*` fallbacks in `mb_add`/`mb_mul`.
/// Tuple has no in-place mutating dunder at all, so `TS((1,)) += (2,)`
/// reaches this same unwrap via the ordinary `__add__` binop dispatch
/// (`mb_inplace` falls through when `builtin_inplace_update_fn` has no
/// `"tuple"` entry) — CPython drops the subclass there too, returning a
/// plain tuple. See `list_like_operand` above for the general shape. (#1026)
pub(super) fn tuple_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Tuple(_)) {
                return Some(value);
            }
        }
    }
    match builtin_data_payload(value) {
        Some(("tuple", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a dict-SUBCLASS instance (`class DS(dict): pass`) to its raw
/// `ObjData::Dict` payload, for the `|` merge fallback in `mb_bitor`. See
/// `list_like_operand` above for the general shape. (#1026)
pub(super) fn dict_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Dict(_)) {
                return Some(value);
            }
        }
    }
    match builtin_data_payload(value) {
        Some(("dict", payload)) => Some(payload),
        _ => None,
    }
}
