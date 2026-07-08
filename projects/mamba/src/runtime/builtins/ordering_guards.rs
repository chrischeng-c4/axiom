use super::super::{
    iter,
    rc::ObjData,
    stdlib::{collections_mod, dataclasses_mod, enum_class},
    value::MbValue,
};
use super::{is_complex_obj, raise_type_error, value_type_name};

/// Ordering on complex is undefined in Python: raise the CPython-exact
/// TypeError when either operand is a complex object. Returns true when an
/// exception was raised (caller must bail with a dummy value).
pub(super) fn complex_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if is_complex_obj(a) || is_complex_obj(b) {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

/// Plain `Enum` / non-int `Flag` members don't support ordering in CPython;
/// raise the exact TypeError when either operand is one. IntEnum / IntFlag /
/// StrEnum members compare via their raw int/str value and are NOT guarded.
pub(super) fn enum_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if enum_class::member_is_plain_unorderable(a) || enum_class::member_is_plain_unorderable(b) {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

pub(super) fn range_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if iter::is_range_handle(a) || iter::is_range_handle(b) {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

/// The elements of a set-like value (`set` or `frozenset`), or None for any
/// other value. Used so subset/superset comparisons treat the two types
/// interchangeably, matching CPython.
pub(super) fn setlike_items(v: MbValue) -> Option<Vec<MbValue>> {
    v.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Set(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::FrozenSet(items) => Some(items.iter().copied().collect()),
            _ => None,
        }
    })
}

pub(super) fn is_instance_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Instance { .. }) })
}

pub(super) fn same_datetime_instances(a: MbValue, b: MbValue) -> bool {
    match (a.as_ptr(), b.as_ptr()) {
        (Some(pa), Some(pb)) => unsafe {
            matches!(
                (&(*pa).data, &(*pb).data),
                (
                    ObjData::Instance { class_name: ca, .. },
                    ObjData::Instance { class_name: cb, .. },
                ) if ca == "datetime.datetime" && cb == "datetime.datetime"
            )
        },
        _ => false,
    }
}

pub(super) fn same_ordered_dataclass_instances(a: MbValue, b: MbValue) -> bool {
    match (a.as_ptr(), b.as_ptr()) {
        (Some(pa), Some(pb)) => unsafe {
            matches!(
                (&(*pa).data, &(*pb).data),
                (
                    ObjData::Instance { class_name: ca, .. },
                    ObjData::Instance { class_name: cb, .. },
                ) if ca == cb && dataclasses_mod::dc_order_field_names(ca).is_some()
            )
        },
        _ => false,
    }
}

pub(super) fn can_derive_ordering_from_lt_eq(a: MbValue, b: MbValue) -> bool {
    if !is_instance_value(a) && !is_instance_value(b) {
        return true;
    }
    if collections_mod::is_counter_instance(a) && collections_mod::is_counter_instance(b) {
        return true;
    }
    same_datetime_instances(a, b) || same_ordered_dataclass_instances(a, b)
}

pub(super) fn unsupported_ordering_bool(a: MbValue, b: MbValue, op: &str) -> MbValue {
    raise_type_error(format!(
        "'{op}' not supported between instances of '{}' and '{}'",
        value_type_name(a),
        value_type_name(b)
    ));
    MbValue::from_bool(false)
}
