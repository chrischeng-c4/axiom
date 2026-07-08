use super::super::rc::ObjData;
use super::super::value::MbValue;
use super::{mb_str, type_error_value, value_type_name};

fn is_str_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
}

fn is_bytes_or_bytearray_value(v: MbValue) -> bool {
    v.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
    })
}

/// str(object, encoding, errors) constructor form for bytes-like decoding.
pub fn mb_str_construct(object: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    let has_encoding = !encoding.is_none();
    let has_errors = !errors.is_none();
    if !has_encoding && !has_errors {
        return mb_str(object);
    }
    if !is_bytes_or_bytearray_value(object) {
        return type_error_value(format!(
            "decoding to str: need a bytes-like object, {} found",
            value_type_name(object)
        ));
    }
    if !has_encoding {
        return type_error_value("str() missing required argument 'encoding' (pos 2)");
    }
    if !is_str_value(encoding) {
        return type_error_value(format!(
            "str() argument 'encoding' must be str, not {}",
            value_type_name(encoding)
        ));
    }
    if has_errors && !is_str_value(errors) {
        return type_error_value(format!(
            "str() argument 'errors' must be str, not {}",
            value_type_name(errors)
        ));
    }
    super::super::bytes_ops::mb_bytes_decode_with(object, encoding, errors)
}
