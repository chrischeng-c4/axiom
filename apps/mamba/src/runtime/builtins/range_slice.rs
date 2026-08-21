use crate::runtime::builtins::{add_operand_type_name, raise_type_error, value_type_name};
use crate::runtime::class;
use crate::runtime::exception;
use crate::runtime::iter;
use crate::runtime::rc::{retain_if_ptr, MbObject, ObjData};
use crate::runtime::value::MbValue;

/// range(stop) — produce a CPython-style reusable range handle.
/// CPython requires every `range()` argument to be an integer (or a subclass
/// such as `bool`); a `float`/`str`/etc. raises
/// `TypeError: '<type>' object cannot be interpreted as an integer`.
/// mamba is type-strict, so we enforce the same rule instead of silently
/// coercing a non-int to `0` (the prior `as_int_pyint().unwrap_or(0)` behavior).
///
/// Coerce a `range()` constructor argument through CPython's SupportsIndex
/// protocol. Plain ints, bools, and boxed BigInt values pass through; user
/// instances may define `__index__`. Missing or invalid index values raise
/// TypeError, while exceptions raised by `__index__` propagate unchanged.
fn range_arg_to_index_value(v: MbValue) -> Option<MbValue> {
    if v.is_int() || v.is_bool() {
        return Some(v);
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::BigInt(_) => return Some(v),
                ObjData::Instance { class_name, .. } => {
                    let method = class::lookup_method(class_name, "__index__");
                    if !method.is_none() {
                        let result = class::mb_call_method1(method, v);
                        if exception::current_exception_type().is_some() {
                            return None;
                        }
                        if result.is_int() || result.is_bool() {
                            return Some(result);
                        }
                        if let Some(result_ptr) = result.as_ptr() {
                            if let ObjData::BigInt(_) = (*result_ptr).data {
                                return Some(result);
                            }
                        }
                        raise_type_error(format!(
                            "__index__ returned non-int (type {})",
                            value_type_name(result)
                        ));
                        return None;
                    }
                }
                _ => {}
            }
        }
    }
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "'{}' object cannot be interpreted as an integer",
            add_operand_type_name(v),
        ))),
    );
    None
}

fn range_index_value_is_zero(v: MbValue) -> bool {
    v.as_int_pyint() == Some(0)
        || unsafe { crate::runtime::bigint_ops::extract_bigint(v) }
            .is_some_and(|b| b == num_bigint::BigInt::from(0))
}

pub fn mb_range_no_args() -> MbValue {
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "range expected at least 1 argument, got 0".to_string(),
        )),
    );
    MbValue::none()
}

pub fn mb_range_too_many_args(n: MbValue) -> MbValue {
    let count = n.as_int_pyint().unwrap_or(0);
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "range expected at most 3 arguments, got {count}"
        ))),
    );
    MbValue::none()
}

pub fn mb_range(stop: MbValue) -> MbValue {
    let Some(stop) = range_arg_to_index_value(stop) else {
        return MbValue::none();
    };
    iter::mb_range_iter(MbValue::from_int(0), stop, MbValue::from_int(1))
}

/// `range(start, stop)` — produce a CPython-style reusable range handle.
pub fn mb_range_2(start: MbValue, stop: MbValue) -> MbValue {
    let Some(start) = range_arg_to_index_value(start) else {
        return MbValue::none();
    };
    let Some(stop) = range_arg_to_index_value(stop) else {
        return MbValue::none();
    };
    iter::mb_range_iter(start, stop, MbValue::from_int(1))
}

/// `slice()` — zero-arg form raises TypeError, matching CPython:
///   `TypeError: slice expected at least 1 argument, got 0`
///
/// Routed from the lower pass when the call site has zero positional args.
pub fn mb_slice_no_args() -> MbValue {
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "slice expected at least 1 argument, got 0".to_string(),
        )),
    );
    MbValue::none()
}

/// `slice(start, stop, step)` — Python slice constructor.
///
/// Always called with three args (codegen pads missing positions with None).
/// Python's `slice(stop)` 1-arg form is rewritten by the lower pass to
/// `mb_slice(None, stop, None)`. The returned object is an Instance with
/// `class_name = "slice"` and the fields `start`, `stop`, `step`; the print
/// and repr paths special-case that class to render `slice(start, stop, step)`.
pub fn mb_slice(start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    let inst = MbObject::new_instance_with_capacity("slice".to_string(), 3);
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut f = fields.write().unwrap();
            f.insert("start".to_string(), start);
            f.insert("stop".to_string(), stop);
            f.insert("step".to_string(), step);
            for v in [start, stop, step] {
                retain_if_ptr(v);
            }
        }
    }
    MbValue::from_ptr(inst)
}

/// `range(start, stop, step)` — produce a CPython-style reusable range handle.
pub fn mb_range_3(start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    // CPython validates argument types (TypeError) before the zero-step
    // ValueError: `range(0, 3, 0.0)` raises TypeError, not ValueError.
    let Some(start) = range_arg_to_index_value(start) else {
        return MbValue::none();
    };
    let Some(stop) = range_arg_to_index_value(stop) else {
        return MbValue::none();
    };
    let Some(step) = range_arg_to_index_value(step) else {
        return MbValue::none();
    };
    if range_index_value_is_zero(step) {
        exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "range() arg 3 must not be zero".to_string(),
            )),
        );
        return MbValue::none();
    }
    iter::mb_range_iter(start, stop, step)
}
