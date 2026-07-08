use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// Plain Python type name of a value — for CPython-exact error messages.
/// Instances report their short class name (`pkg.Cls` → `Cls`).
pub(crate) fn value_type_name(val: MbValue) -> String {
    if val.is_bool() {
        return "bool".to_string();
    }
    if val.is_none() {
        return "NoneType".to_string();
    }
    if val.is_not_implemented() {
        return "NotImplementedType".to_string();
    }
    if val.is_ellipsis() {
        return "ellipsis".to_string();
    }
    if let Some(iter_type) = super::super::iter::mb_iter_type_name(val) {
        return iter_type.to_string();
    }
    if val.is_int() {
        return "int".to_string();
    }
    if val.is_float() {
        return "float".to_string();
    }
    if val.as_func().is_some() {
        return "function".to_string();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Instance { class_name, .. } if class_name == "__instance_dict_proxy__" => {
                    "dict"
                }
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                ObjData::CodeObject { .. } => "code",
                ObjData::Instance { class_name, .. } => {
                    return class_name
                        .rsplit('.')
                        .next()
                        .unwrap_or(class_name)
                        .to_string();
                }
            }
            .to_string();
        }
    }
    "object".to_string()
}

/// Raise a TypeError with the given message through the runtime exception
/// machinery so it is catchable from user code.
pub(crate) fn raise_type_error(msg: String) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

pub(super) fn type_error_value(msg: impl Into<String>) -> MbValue {
    raise_type_error(msg.into());
    MbValue::none()
}

/// Raise a ValueError with the given message through the runtime exception
/// machinery so it is catchable from user code.
pub(crate) fn raise_value_error(msg: String) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}
