use super::super::rc::ObjData;
use super::super::value::MbValue;
use super::{
    is_decimal_handle_value, is_fraction_handle_value, mb_bool, mb_len, mb_values_identical,
    raise_type_error, validate_len_result,
};

/// Check if a value is None. Returns bool MbValue.
/// Used by for-loop lowering to detect iterator exhaustion.
pub fn mb_is_none(val: MbValue) -> MbValue {
    MbValue::from_bool(val.is_none())
}

/// Check if a value is NOT None. Returns bool MbValue.
/// Used by except* lowering to check if matched/rest sub-groups exist.
pub fn mb_is_not_none(val: MbValue) -> MbValue {
    MbValue::from_bool(!val.is_none())
}

pub fn mb_is_identity(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(mb_values_identical(a, b))
}

pub fn mb_is_not_identity(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(!mb_values_identical(a, b))
}

pub fn mb_not(a: MbValue) -> MbValue {
    let truthy = mb_bool(a);
    MbValue::from_bool(!truthy.as_bool().unwrap_or(false))
}

/// Python truthiness for any MbValue — returns 1 (true) or 0 (false) as raw i64.
/// Used by guards in match/case and other conditions where the value may be a heap object.
pub fn mb_is_truthy(val: MbValue) -> i64 {
    if let Some(target) = super::super::stdlib::weakref_mod::proxy_target_or_raise(val) {
        if target.is_none() {
            return 0;
        }
        return mb_is_truthy(target);
    }
    if val.is_none() {
        return 0;
    }
    if val.is_bool() {
        return if val.as_bool() == Some(true) { 1 } else { 0 };
    }
    if val.is_int() {
        if is_decimal_handle_value(val) || is_fraction_handle_value(val) {
            return if mb_bool(val).as_bool() == Some(true) {
                1
            } else {
                0
            };
        }
        return if val.as_int().unwrap_or(0) != 0 { 1 } else { 0 };
    }
    if val.is_float() {
        return if val.as_float().unwrap_or(0.0) != 0.0 {
            1
        } else {
            0
        };
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(s) => {
                    if s.is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::List(l) => {
                    if l.read().unwrap().is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Tuple(t) => {
                    if t.is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Dict(d) => {
                    if d.read().unwrap().is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Set(s) => {
                    if s.read().unwrap().is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Bytes(b) => {
                    if b.is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::ByteArray(b) => {
                    if b.read().unwrap().is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::FrozenSet(s) => {
                    if s.is_empty() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::BigInt(b) => {
                    use num_traits::Zero;
                    if b.is_zero() {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Complex(re, im) => {
                    if *re == 0.0 && *im == 0.0 {
                        0
                    } else {
                        1
                    }
                }
                ObjData::Instance { class_name, .. } => {
                    // __bool__ dunder: Python calls __bool__() for truthiness
                    let bool_method = super::super::class::lookup_method(class_name, "__bool__");
                    if !bool_method.is_none() {
                        let result = super::super::class::mb_call_method1(bool_method, val);
                        if let Some(bv) = result.as_bool() {
                            return if bv { 1 } else { 0 };
                        }
                        if let Some(iv) = result.as_int() {
                            return if iv != 0 { 1 } else { 0 };
                        }
                    } else if super::super::class::class_bool_is_blocked(class_name) {
                        // `__bool__ = None` disables truth-testing entirely.
                        raise_type_error("'NoneType' object is not callable".to_string());
                        return 0;
                    }
                    // __len__ fallback: truthy if len != 0 (validated).
                    let len_method = super::super::class::lookup_method(class_name, "__len__");
                    if !len_method.is_none() {
                        let result = super::super::class::mb_call_method1(len_method, val);
                        let checked = validate_len_result(result);
                        if let Some(iv) = checked.as_int() {
                            return if iv != 0 { 1 } else { 0 };
                        }
                        if checked.is_bool() {
                            return if checked.as_bool() == Some(true) {
                                1
                            } else {
                                0
                            };
                        }
                        if let Some(p) = checked.as_ptr() {
                            if let ObjData::BigInt(ref b) = (*p).data {
                                use num_traits::Zero;
                                return if b.is_zero() { 0 } else { 1 };
                            }
                        }
                        // validate_len_result raised: fall through with a
                        // pending exception (the value below is discarded).
                    } else if let Some((_base, payload)) =
                        super::super::class::builtin_data_payload_if_unoverridden(val, "__len__")
                    {
                        return mb_is_truthy(payload);
                    } else if let Some((_kind, payload)) =
                        super::super::stdlib::collections_mod::user_wrapper_data(val)
                    {
                        return mb_is_truthy(payload);
                    } else if class_name == "collections.deque" {
                        // Mirrors the same deque case in mb_bool above: deque
                        // has no lookup_method-registered __len__, so this
                        // fast conditional path (`if d:` / `while d:`) also
                        // needs its own length check (#868 shlex investigation).
                        if let Some(n) = mb_len(val).as_int() {
                            return if n != 0 { 1 } else { 0 };
                        }
                    }
                    // Empty Flag members (value 0, e.g. `RED & BLUE`) are
                    // falsy; plain Enum members stay default-truthy.
                    if super::super::stdlib::enum_class::flag_member_is_empty(val) {
                        return 0;
                    }
                    1 // default: truthy
                }
                _ => 1, // Function, Class, CodeObject, etc. are always truthy
            };
        }
    }
    1 // fallback: truthy
}
