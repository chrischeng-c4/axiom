use super::super::{
    rc::{MbObject, ObjData},
    value::MbValue,
};

/// If `v` is a `slice` instance, return its (start, stop, step) as a tuple
/// MbValue so the value-comparison paths can compare/hash slices the way
/// CPython does (slices compare and hash as the 3-tuple of their fields).
pub(super) fn slice_as_tuple(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "slice" {
                let g = fields.read().unwrap();
                let get = |k: &str| g.get(k).copied().unwrap_or_else(MbValue::none);
                return Some(MbValue::from_ptr(MbObject::new_tuple(vec![
                    get("start"),
                    get("stop"),
                    get("step"),
                ])));
            }
        }
    }
    None
}

pub(super) fn mappingproxy_mapping(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "mappingproxy" {
                return fields.read().unwrap().get("_mapping").copied();
            }
        }
    }
    None
}

pub(super) fn bound_method_parts(v: MbValue) -> Option<(MbValue, MbValue)> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "method" {
                let guard = fields.read().unwrap();
                let func = guard.get("__func__").copied().unwrap_or_else(MbValue::none);
                let recv = guard.get("__self__").copied().unwrap_or_else(MbValue::none);
                return Some((func, recv));
            }
        }
    }
    None
}
