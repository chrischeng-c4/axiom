use crate::runtime::{class, exception, rc::MbObject, value::MbValue};

/// assert statement failure — raise AssertionError via exception system.
pub fn mb_assertion_error(msg: MbValue) {
    let exc_type = MbValue::from_ptr(MbObject::new_str("AssertionError".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![msg]));
    let instance = exception::mb_exception_new_with_args(exc_type, args);
    class::mb_raise_instance(instance);
}

/// assert statement failure — no message variant.
pub fn mb_assertion_error_no_msg() {
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("AssertionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
    );
}
