use super::super::{
    class,
    rc::{MbObject, ObjData},
    value::MbValue,
};

/// CPython `PyObject_RichCompareBool(a, b, Py_EQ)`: identity-first, then value
/// equality. Container comparisons (list/tuple/set/dict `==`, `in`, count,
/// subset/superset) use this so a self-unequal element such as NaN still
/// matches when the SAME object appears on both sides — `[nan] == [nan]` is True
/// for one shared NaN. Scalar `==` (mb_eq) deliberately does NOT add this, so
/// `nan == nan` stays False.
pub(super) fn mb_richcmp_eq(a: MbValue, b: MbValue) -> bool {
    super::mb_values_identical(a, b) || super::mb_values_eq(a, b)
}

/// Try the reflected __eq__ on `obj` (i.e. obj.__eq__(other)).
/// Returns true/false if the reflected op gives a definitive answer, false if not.
pub(super) fn try_reflected_eq(obj: MbValue, other: MbValue) -> bool {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { class_name, .. } = &(*ptr).data {
                let eq_method = class::lookup_method(class_name, "__eq__");
                if !eq_method.is_none() {
                    let eq_name = MbValue::from_ptr(MbObject::new_str("__eq__".to_string()));
                    let args = MbValue::from_ptr(MbObject::new_list(vec![other]));
                    let result = class::mb_call_method(obj, eq_name, args);
                    if result.is_not_implemented() {
                        return false;
                    }
                    if let Some(bv) = result.as_bool() {
                        return bv;
                    }
                    if let Some(iv) = result.as_int() {
                        return iv != 0;
                    }
                }
            }
        }
    }
    false
}

enum RichcmpDunderCall {
    Missing,
    NotImplemented,
    Value(bool),
}

fn instance_class_name_for_richcmp(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

fn class_has_method(class_name: &str, method: &str) -> bool {
    !class::lookup_method(class_name, method).is_none()
}

fn call_instance_richcmp_dunder(
    inst: MbValue,
    class_name: &str,
    method: &str,
    other: MbValue,
) -> RichcmpDunderCall {
    if !class_has_method(class_name, method) {
        return RichcmpDunderCall::Missing;
    }
    let method_name = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![other]));
    let result = class::mb_call_method(inst, method_name, args);
    if result.is_not_implemented() {
        return RichcmpDunderCall::NotImplemented;
    }
    if let Some(bv) = result.as_bool() {
        return RichcmpDunderCall::Value(bv);
    }
    if let Some(iv) = result.as_int() {
        return RichcmpDunderCall::Value(iv != 0);
    }
    RichcmpDunderCall::Value(false)
}

fn instance_rhs_has_priority(lhs_class: &str, rhs_class: &str) -> bool {
    lhs_class != rhs_class && class::class_mro_any(rhs_class, |name| name == lhs_class)
}

pub(super) fn mb_instance_ne(a: MbValue, b: MbValue) -> Option<bool> {
    let a_class = instance_class_name_for_richcmp(a);
    let b_class = instance_class_name_for_richcmp(b);
    let a_ne = a_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__ne__"));
    let a_eq = a_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__eq__"));
    let b_ne = b_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__ne__"));
    let b_eq = b_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__eq__"));
    if !a_ne && !a_eq && !b_ne && !b_eq {
        return None;
    }

    let rhs_priority = match (a_class.as_deref(), b_class.as_deref()) {
        (Some(lhs), Some(rhs)) => instance_rhs_has_priority(lhs, rhs),
        _ => false,
    };

    if rhs_priority && b_ne {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__ne__", a)
            {
                return Some(v);
            }
        }
        if !a_ne {
            if let Some(class_name) = a_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(a, class_name, "__eq__", b)
                {
                    return Some(!v);
                }
            }
            return Some(true);
        }
    }

    if a_ne {
        if let Some(class_name) = a_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(a, class_name, "__ne__", b)
            {
                return Some(v);
            }
        }
        if !rhs_priority && b_ne {
            if let Some(class_name) = b_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(b, class_name, "__ne__", a)
                {
                    return Some(v);
                }
            }
            return Some(true);
        }
        if !b_ne && b_eq {
            if let Some(class_name) = b_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(b, class_name, "__eq__", a)
                {
                    return Some(!v);
                }
            }
        }
        return Some(true);
    }

    if a_eq {
        if let Some(class_name) = a_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(a, class_name, "__eq__", b)
            {
                return Some(!v);
            }
        }
    }

    if !rhs_priority && b_ne {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__ne__", a)
            {
                return Some(v);
            }
        }
        return Some(true);
    }

    if b_eq {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__eq__", a)
            {
                return Some(!v);
            }
        }
        return Some(true);
    }

    if a_eq {
        return Some(true);
    }
    None
}
