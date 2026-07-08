use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// str(value) — convert to string object.
pub fn mb_str(val: MbValue) -> MbValue {
    // TAG_FUNC user-defined functions render as `<function NAME at 0xADDR>`
    // to match CPython. Closure handles share TAG_INT with low-value ints
    // (closure IDs start at 1), so we restrict detection to TAG_FUNC only
    // to avoid corrupting integer rendering.
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        return MbValue::from_ptr(MbObject::new_str(format!(
            "<function {name} at 0x{addr:x}>"
        )));
    }
    if let Some(name) = pep695_display_name(val) {
        return MbValue::from_ptr(MbObject::new_str(name));
    }
    // UserDict / UserList / UserString stringify through their payload
    // (str(UserString("hi")) == "hi", str(UserList([1])) == "[1]").
    if let Some((_, data)) = super::super::stdlib::collections_mod::user_wrapper_data(val) {
        return mb_str(data);
    }
    if let Some(text) = super::super::stdlib::xml_mod::qname_text_value(val) {
        return MbValue::from_ptr(MbObject::new_str(text));
    }
    let s = if let Some(i) = val.as_int() {
        // UUID handles are int-tagged but render as the canonical
        // 8-4-4-4-12 form (#1475 — keep `print(uuid.uuid4())` honest
        // instead of leaking the i64 handle ID).
        if super::super::stdlib::uuid_mod::is_uuid_handle(i as u64) {
            return super::super::stdlib::uuid_mod::mb_uuid_str(val);
        }
        // Decimal / Fraction handles render their numeric value (#2129).
        if super::super::stdlib::decimal_mod::is_decimal_handle(i as u64) {
            return super::super::stdlib::decimal_mod::mb_decimal_str(val);
        }
        if super::super::stdlib::fractions_mod::is_fraction_handle(i as u64) {
            return super::super::stdlib::fractions_mod::mb_fraction_str(val);
        }
        format!("{i}")
    } else if let Some(f) = val.as_float() {
        super::super::string_ops::python_float_repr(f)
    } else if let Some(b) = val.as_bool() {
        (if b { "True" } else { "False" }).to_string()
    } else if val.is_none() {
        "None".to_string()
    } else if val.is_not_implemented() {
        "NotImplemented".to_string()
    } else if val.is_ellipsis() {
        "Ellipsis".to_string()
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(codepoints) = super::super::string_ops::surrogate_codepoints(val) {
            return super::super::string_ops::new_surrogate_codepoints_str(codepoints);
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    // str(x) must return a new object (owned reference) so
                    // the JIT can safely release input and output independently.
                    return MbValue::from_ptr(MbObject::new_str(s.clone()));
                }
                _ => super::super::string_ops::value_to_string(val),
            }
        }
    } else {
        String::new()
    };
    let obj = MbObject::new_str(s);
    MbValue::from_ptr(obj)
}

pub(crate) fn pep695_display_name(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance {
                class_name,
                ref fields,
            } if matches!(
                class_name.as_str(),
                "TypeVar" | "TypeVarTuple" | "ParamSpec" | "TypeAliasType"
            ) =>
            {
                fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__name__").copied())
                    .and_then(|v| v.as_ptr())
                    .and_then(|name_ptr| {
                        if let ObjData::Str(ref s) = (*name_ptr).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
            }
            _ => None,
        }
    }
}
