use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// Fallback guard for arithmetic on datetime.* instances: any combination
/// that no dedicated arm accepted is an unsupported-operand TypeError in
/// CPython (e.g. timedelta + 1, datetime + datetime, int // timedelta).
/// Raises and returns true when either operand is a datetime.* instance.
pub(super) fn raise_datetime_op_type_error(op: &str, a: MbValue, b: MbValue) -> bool {
    fn dt_class(v: MbValue) -> Option<String> {
        let ptr = v.as_ptr()?;
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name.starts_with("datetime.") {
                    return Some(class_name.clone());
                }
            }
        }
        None
    }
    let (ca, cb) = (dt_class(a), dt_class(b));
    if ca.is_none() && cb.is_none() {
        return false;
    }
    let na = ca.unwrap_or_else(|| super::add_operand_type_name(a).to_string());
    let nb = cb.unwrap_or_else(|| super::add_operand_type_name(b).to_string());
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "unsupported operand type(s) for {op}: '{na}' and '{nb}'"
        ))),
    );
    true
}
