use super::super::rc::ObjData;
use super::super::value::MbValue;

/// Try to extract `(real, imag)` from any numeric `MbValue` — int, float,
/// bool, or `ObjData::Complex`. Returns `None` when `val` is non-numeric.
/// Used by complex-aware arithmetic helpers to coerce mixed operands.
/// (#1256 — complex arithmetic gap)
pub(super) fn as_complex_pair(val: MbValue) -> Option<(f64, f64)> {
    // Decimal / Fraction are tagged-int handles: as_int() below would read the
    // handle id as a bogus real component (`Decimal("3.0") == 3+0j` compared the
    // handle id against 3.0 → False). They are not directly complex-coercible
    // here; (Decimal/Fraction vs complex) equality flows through the
    // numeric-handle path in mb_values_eq instead.
    if super::is_decimal_handle_value(val) || super::is_fraction_handle_value(val) {
        return None;
    }
    if let Some(i) = val.as_int() {
        return Some((i as f64, 0.0));
    }
    if let Some(f) = val.as_float() {
        return Some((f, 0.0));
    }
    if let Some(b) = val.as_bool() {
        return Some((b as i64 as f64, 0.0));
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Complex(re, im) = (*ptr).data {
                return Some((re, im));
            }
        }
    }
    if let Some(("complex", payload)) = super::super::class::builtin_data_payload(val) {
        return as_complex_pair(payload);
    }
    None
}

/// True when `val` is an `ObjData::Complex` object — distinct from a real
/// number coercible to complex. Used to gate the complex-arithmetic
/// promotion in mb_add/sub/mul/div. (#1256)
pub(super) fn is_complex_obj(val: MbValue) -> bool {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if matches!((*ptr).data, ObjData::Complex(_, _)) {
                return true;
            }
        }
    }
    matches!(
        super::super::class::builtin_data_payload(val),
        Some(("complex", _))
    )
}

/// True iff `val` is a number complex comparison can be defined against
/// (int / float / bool / complex / arbitrary-precision int). Anything else
/// makes `complex.__eq__`/`__ne__` return NotImplemented, per CPython.
pub(super) fn is_complex_cmp_operand(val: MbValue) -> bool {
    if val.is_int() || val.is_float() || val.as_bool().is_some() {
        return true;
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if matches!((*ptr).data, ObjData::Complex(_, _) | ObjData::BigInt(_)) {
                return true;
            }
        }
    }
    matches!(
        super::super::class::builtin_data_payload(val),
        Some(("complex", _))
    )
}

/// Compute an unbound complex comparison dunder `complex.<method>(a, b)`.
/// Returns `Some(result)` for the six rich-comparison dunders, `None` for any
/// other method name. __eq__/__ne__ yield a bool when `b` is numeric and
/// NotImplemented otherwise; the ordering dunders are always NotImplemented
/// (complex has no ordering). Shared by the unbound-method-wrapper call path
/// and the direct `complex.__eq__(a, b)` method-call path (mb_call_method).
pub(crate) fn complex_cmp_dunder(method: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    match method {
        "__eq__" | "__ne__" => Some(if !is_complex_cmp_operand(b) {
            MbValue::not_implemented()
        } else if method == "__eq__" {
            super::mb_eq(a, b)
        } else {
            super::mb_ne(a, b)
        }),
        "__lt__" | "__le__" | "__gt__" | "__ge__" => Some(MbValue::not_implemented()),
        _ => None,
    }
}
