use super::super::rc::ObjData;
use super::super::value::MbValue;

pub(super) fn int_enum_like_value(val: MbValue) -> Option<MbValue> {
    super::super::stdlib::enum_class::int_member_value(val)
        .or_else(|| super::super::stdlib::signal_mod::signal_enum_int_value(val))
        .or_else(|| super::super::stdlib::http_mod::http_status_member_value(val))
}

pub(super) fn int_subclass_payload_for_dunder(val: MbValue, dunder: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*p).data
        {
            if !super::super::class::check_class_hierarchy(class_name, "int")
                || super::super::class::class_defines_own_method(class_name, dunder)
            {
                return None;
            }
            fields
                .read()
                .unwrap()
                .get(super::super::class::INT_SUBCLASS_VALUE_FIELD)
                .copied()
        } else {
            None
        }
    })
}

pub(super) fn float_subclass_payload_for_dunder(val: MbValue, dunder: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*p).data
        {
            if !super::super::class::check_class_hierarchy(class_name, "float")
                || super::super::class::class_defines_own_method(class_name, dunder)
            {
                return None;
            }
            fields
                .read()
                .unwrap()
                .get(super::super::class::FLOAT_SUBCLASS_VALUE_FIELD)
                .copied()
        } else {
            None
        }
    })
}

/// CPython's `PyLong_Check` — used to validate `__int__`/`__index__`
/// dunder-loop results before returning them: plain int, bool, BigInt, and
/// int-subclass instances all pass; anything else (float, str, ...) is a
/// `TypeError` per CPython's return-type contract for these dunders. (#1063)
pub(super) fn value_is_long_check(v: MbValue) -> bool {
    if v.is_int() || v.is_bool() {
        return true;
    }
    if unsafe { super::super::bigint_ops::extract_bigint(v) }.is_some() {
        return true;
    }
    v.as_ptr().is_some_and(|p| unsafe {
        matches!(&(*p).data, ObjData::Instance { class_name, .. }
            if super::super::class::check_class_hierarchy(class_name, "int"))
    })
}

/// CPython's `PyFloat_Check` — used to validate `__float__` dunder-loop
/// results: plain float or a float-subclass instance pass; anything else
/// (int, bool, str, ...) is a `TypeError`. (#1063)
pub(super) fn value_is_float_check(v: MbValue) -> bool {
    if v.is_float() {
        return true;
    }
    v.as_ptr().is_some_and(|p| unsafe {
        matches!(&(*p).data, ObjData::Instance { class_name, .. }
            if super::super::class::check_class_hierarchy(class_name, "float"))
    })
}

pub(super) fn int_subclass_numeric_operands(
    a: MbValue,
    b: MbValue,
    dunder: &str,
) -> Option<(MbValue, MbValue)> {
    let av = int_subclass_payload_for_dunder(a, dunder);
    let bv = int_subclass_payload_for_dunder(b, dunder);
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

pub(super) fn str_subclass_ordering_operands(
    a: MbValue,
    b: MbValue,
    dunder: &str,
) -> Option<(MbValue, MbValue)> {
    let av = super::super::class::builtin_data_payload_if_unoverridden(a, dunder)
        .filter(|(base, _)| *base == "str")
        .map(|(_, payload)| payload);
    let bv = super::super::class::builtin_data_payload_if_unoverridden(b, dunder)
        .filter(|(base, _)| *base == "str")
        .map(|(_, payload)| payload);
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

pub(super) fn numeric_subclass_operands(
    a: MbValue,
    b: MbValue,
    dunder: &str,
) -> Option<(MbValue, MbValue)> {
    let av = int_subclass_payload_for_dunder(a, dunder)
        .or_else(|| float_subclass_payload_for_dunder(a, dunder));
    let bv = int_subclass_payload_for_dunder(b, dunder)
        .or_else(|| float_subclass_payload_for_dunder(b, dunder));
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

/// Unary counterpart of `numeric_subclass_operands`: unwrap a single
/// int/float-SUBCLASS instance (`class P(int): pass` / `class F(float):
/// pass`) to its raw payload for the unary `+`/`-` fallbacks (`mb_neg`,
/// `mb_dispatch_unaryop`'s pos/neg arms). `None` when `a` isn't an
/// unoverridden numeric-derived-class instance (plain int/float, a
/// non-numeric class, or a subclass that defines its own override for
/// `dunder` — the caller's override check/invocation keeps priority). (#1030)
pub(crate) fn numeric_subclass_unary_operand(a: MbValue, dunder: &str) -> Option<MbValue> {
    int_subclass_payload_for_dunder(a, dunder)
        .or_else(|| float_subclass_payload_for_dunder(a, dunder))
}

/// Int-only unary counterpart for `~` (`__invert__`) — float has no
/// `__invert__`, so only an int-derived-class instance unwraps here. (#1030)
pub(crate) fn int_subclass_unary_operand(a: MbValue, dunder: &str) -> Option<MbValue> {
    int_subclass_payload_for_dunder(a, dunder)
}
