use super::super::{rc::ObjData, value::MbValue};

/// Best-effort Python type name of an operand, for `+` TypeError messages.
pub(crate) fn add_operand_type_name(v: MbValue) -> &'static str {
    if v.is_int() {
        return "int";
    }
    if v.is_float() {
        return "float";
    }
    if v.is_bool() {
        return "bool";
    }
    if v.is_none() {
        return "NoneType";
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                _ => "object",
            };
        }
    }
    "object"
}
