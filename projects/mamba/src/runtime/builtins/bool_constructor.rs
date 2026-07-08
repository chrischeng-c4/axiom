use super::super::rc::ObjData;
use super::super::value::MbValue;
use super::{
    is_decimal_handle_value, is_fraction_handle_value, mb_len, raise_type_error,
    validate_len_result, value_type_name,
};

/// bool(value) — truthiness check.
pub fn mb_bool(val: MbValue) -> MbValue {
    if let Some(target) = super::super::stdlib::weakref_mod::proxy_target_or_raise(val) {
        if target.is_none() {
            return MbValue::none();
        }
        return mb_bool(target);
    }
    if is_decimal_handle_value(val) {
        return super::super::stdlib::decimal_mod::mb_decimal_bool(val);
    }
    if is_fraction_handle_value(val) {
        return super::super::stdlib::fractions_mod::mb_fraction_bool(val);
    }
    let truthy = if val.is_none() {
        false
    } else if let Some(i) = val.as_int() {
        // Iterator handles share TAG_INT. For a `range` iter handle the
        // truth value reflects remaining length (matches CPython's
        // `bool(range(5, 5)) == False`); other iterator kinds are objects,
        // truthy by identity.
        if super::super::iter::is_iter_handle(val) {
            match super::super::iter::mb_iter_range_is_nonempty(val) {
                Some(nonempty) => nonempty,
                None => true,
            }
        } else {
            i != 0
        }
    } else if let Some(f) = val.as_float() {
        f != 0.0
    } else if let Some(b) = val.as_bool() {
        b
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(n) = super::super::string_ops::surrogate_len(val) {
            return MbValue::from_bool(n != 0);
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => !s.is_empty(),
                ObjData::List(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::Dict(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::Tuple(items) => !items.is_empty(),
                ObjData::Set(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::FrozenSet(items) => !items.is_empty(),
                ObjData::Bytes(b) => !b.is_empty(),
                ObjData::ByteArray(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::BigInt(b) => {
                    use num_traits::Zero;
                    !b.is_zero()
                }
                ObjData::Complex(re, im) => *re != 0.0 || *im != 0.0,
                ObjData::Instance { class_name, .. } => {
                    // __bool__ dunder dispatch
                    let bool_method = super::super::class::lookup_method(class_name, "__bool__");
                    if !bool_method.is_none() {
                        let result = super::super::class::mb_call_method1(bool_method, val);
                        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
                            return MbValue::from_bool(false);
                        }
                        if let Some(bv) = result.as_bool() {
                            return MbValue::from_bool(bv);
                        }
                        if let Some(iv) = result.as_int() {
                            return MbValue::from_bool(iv != 0);
                        }
                        raise_type_error(format!(
                            "__bool__ should return bool, returned {}",
                            value_type_name(result)
                        ));
                        return MbValue::from_bool(false);
                    } else if super::super::class::class_bool_is_blocked(class_name) {
                        // `__bool__ = None` disables truth-testing; calling the
                        // None slot raises, even when __len__ exists.
                        raise_type_error("'NoneType' object is not callable".to_string());
                        return MbValue::from_bool(false);
                    }
                    // __len__ fallback (validated like len(), so bool() and
                    // len() surface the same error for an illegal __len__).
                    let len_method = super::super::class::lookup_method(class_name, "__len__");
                    if !len_method.is_none() {
                        let result = super::super::class::mb_call_method1(len_method, val);
                        let checked = validate_len_result(result);
                        if let Some(iv) = checked.as_int() {
                            return MbValue::from_bool(iv != 0);
                        }
                        if checked.is_bool() {
                            return MbValue::from_bool(checked.as_bool() == Some(true));
                        }
                        if let Some(p) = checked.as_ptr() {
                            if let ObjData::BigInt(ref b) = (*p).data {
                                use num_traits::Zero;
                                return MbValue::from_bool(!b.is_zero());
                            }
                        }
                        // validate_len_result raised: fall through with a
                        // pending exception (the value below is discarded).
                    } else if let Some((_base, payload)) =
                        super::super::class::builtin_data_payload_if_unoverridden(val, "__len__")
                    {
                        return mb_bool(payload);
                    } else if let Some((_kind, payload)) =
                        super::super::stdlib::collections_mod::user_wrapper_data(val)
                    {
                        return mb_bool(payload);
                    } else if class_name == "collections.deque" {
                        // deque has no lookup_method-registered __len__ (its
                        // len() is hardcoded in mb_len below), so without this
                        // arm an empty deque fell through to the "true"
                        // default instead of CPython's length-based
                        // truthiness (#868 shlex investigation).
                        if let Some(n) = mb_len(val).as_int() {
                            return MbValue::from_bool(n != 0);
                        }
                    }
                    true
                }
                _ => true,
            }
        }
    } else if val.is_ellipsis() || val.is_not_implemented() {
        // Both singletons are truthy in CPython.
        true
    } else {
        false
    };
    MbValue::from_bool(truthy)
}
