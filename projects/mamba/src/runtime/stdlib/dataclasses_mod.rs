use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// dataclasses module for Mamba (#410).
///
/// Provides: @dataclass decorator support, field(), fields(), asdict(), astuple().
/// Dataclasses are implemented as Instance objects with auto-generated
/// __init__, __repr__, __eq__ support via runtime metadata.
use std::collections::HashMap;

#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// @dataclass decorator — marks a class as a dataclass.
/// In practice this is an identity function that registers metadata;
/// the class system already supports field access.
pub fn mb_dataclass(cls: MbValue) -> MbValue {
    // Mark the class with __dataclass__ = True metadata
    if let Some(ptr) = cls.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut fields = fields.write().unwrap();
                fields.insert("__dataclass__".into(), MbValue::from_bool(true));
                fields.insert("__frozen__".into(), MbValue::from_bool(false));
            }
        }
    }
    cls
}

/// @dataclass(frozen=True) — creates a frozen dataclass.
pub fn mb_dataclass_frozen(cls: MbValue) -> MbValue {
    if let Some(ptr) = cls.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut fields = fields.write().unwrap();
                fields.insert("__dataclass__".into(), MbValue::from_bool(true));
                fields.insert("__frozen__".into(), MbValue::from_bool(true));
            }
        }
    }
    cls
}

/// field(default=..., default_factory=...) — create a field descriptor.
/// Returns a dict with field metadata.
pub fn mb_field(args: MbValue) -> MbValue {
    let ptr = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let mut map = lock.write().unwrap();
            map.insert("default".into(), args);
            map.insert("repr".into(), MbValue::from_bool(true));
            map.insert("compare".into(), MbValue::from_bool(true));
        }
    }
    MbValue::from_ptr(ptr)
}

/// fields(instance) — return list of field names.
pub fn mb_fields(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let names: Vec<MbValue> = fields
                    .keys()
                    .filter(|k| !k.starts_with('_'))
                    .map(|k| MbValue::from_ptr(MbObject::new_str(k.clone())))
                    .collect();
                return MbValue::from_ptr(MbObject::new_list(names));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// asdict(instance) — convert dataclass instance to dict.
pub fn mb_asdict(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let dict = MbObject::new_dict();
                if let ObjData::Dict(ref lock) = (*dict).data {
                    let mut map = lock.write().unwrap();
                    for (k, v) in fields.iter() {
                        if !k.starts_with('_') {
                            map.insert(k.clone().into(), *v);
                        }
                    }
                }
                return MbValue::from_ptr(dict);
            }
        }
    }
    MbValue::from_ptr(MbObject::new_dict())
}

/// astuple(instance) — convert dataclass instance to tuple.
pub fn mb_astuple(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let values: Vec<MbValue> = fields
                    .iter()
                    .filter(|(k, _)| !k.starts_with('_'))
                    .map(|(_, v)| *v)
                    .collect();
                return MbValue::from_ptr(MbObject::new_tuple(values));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![]))
}

pub fn register() {
    let mut attrs = HashMap::new();
    attrs.insert(
        "dataclass".into(),
        MbValue::from_int(mb_dataclass as *const () as usize as i64),
    );
    attrs.insert(
        "field".into(),
        MbValue::from_int(mb_field as *const () as usize as i64),
    );
    attrs.insert(
        "fields".into(),
        MbValue::from_int(mb_fields as *const () as usize as i64),
    );
    attrs.insert(
        "asdict".into(),
        MbValue::from_int(mb_asdict as *const () as usize as i64),
    );
    attrs.insert(
        "astuple".into(),
        MbValue::from_int(mb_astuple as *const () as usize as i64),
    );
    super::register_module("dataclasses", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_instance(class: &str, field_pairs: &[(&str, MbValue)]) -> MbValue {
        let ptr = MbObject::new_instance(class.to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                for (k, v) in field_pairs {
                    f.insert(k.to_string(), *v);
                }
            }
        }
        MbValue::from_ptr(ptr)
    }

    fn get_field(inst: MbValue, key: &str) -> MbValue {
        if let Some(ptr) = inst.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    if let Some(v) = fields.read().unwrap().get(key) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_dict_val(dict: MbValue, key: &str) -> MbValue {
        if let Some(ptr) = dict.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    if let Some(v) = lock.read().unwrap().get(key) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn list_strs(val: MbValue) -> Vec<String> {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default()
    }

    fn tuple_vals(val: MbValue) -> Vec<MbValue> {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    items.clone()
                } else {
                    vec![]
                }
            })
            .unwrap_or_default()
    }

    // -- mb_dataclass tests --

    #[test]
    fn test_dataclass_marks_instance() {
        let inst = make_instance("Point", &[("x", MbValue::from_int(1))]);
        let r = mb_dataclass(inst);
        // Should have __dataclass__ = true
        assert_eq!(get_field(r, "__dataclass__").as_bool(), Some(true));
        assert_eq!(get_field(r, "__frozen__").as_bool(), Some(false));
    }

    #[test]
    fn test_dataclass_returns_same_value() {
        let inst = make_instance("Point", &[]);
        let r = mb_dataclass(inst);
        // Should return the same object (identity)
        assert_eq!(r.as_ptr(), inst.as_ptr());
    }

    #[test]
    fn test_dataclass_non_instance() {
        // Passing a non-instance should not panic
        let v = mb_dataclass(MbValue::from_int(5));
        assert_eq!(v.as_int(), Some(5));
    }

    // -- mb_dataclass_frozen tests --

    #[test]
    fn test_dataclass_frozen() {
        let inst = make_instance("Point", &[]);
        let r = mb_dataclass_frozen(inst);
        assert_eq!(get_field(r, "__dataclass__").as_bool(), Some(true));
        assert_eq!(get_field(r, "__frozen__").as_bool(), Some(true));
    }

    // -- mb_field tests --

    #[test]
    fn test_field_creates_dict() {
        let f = mb_field(MbValue::from_int(42));
        assert!(f.as_ptr().is_some());
        assert_eq!(get_dict_val(f, "default").as_int(), Some(42));
        assert_eq!(get_dict_val(f, "repr").as_bool(), Some(true));
        assert_eq!(get_dict_val(f, "compare").as_bool(), Some(true));
    }

    #[test]
    fn test_field_none_default() {
        let f = mb_field(MbValue::none());
        assert!(get_dict_val(f, "default").is_none());
    }

    // -- mb_fields tests --

    #[test]
    fn test_fields_returns_public_names() {
        let inst = make_instance(
            "Pt",
            &[
                ("x", MbValue::from_int(1)),
                ("y", MbValue::from_int(2)),
                ("_internal", MbValue::from_int(0)),
            ],
        );
        let mut names = list_strs(mb_fields(inst));
        names.sort();
        assert_eq!(names, vec!["x", "y"]);
    }

    #[test]
    fn test_fields_empty_instance() {
        let inst = make_instance("Empty", &[]);
        let names = list_strs(mb_fields(inst));
        assert!(names.is_empty());
    }

    #[test]
    fn test_fields_non_instance() {
        let r = mb_fields(MbValue::from_int(5));
        let names = list_strs(r);
        assert!(names.is_empty());
    }

    // -- mb_asdict tests --

    #[test]
    fn test_asdict_basic() {
        let inst = make_instance(
            "Pt",
            &[("x", MbValue::from_int(10)), ("y", MbValue::from_int(20))],
        );
        let d = mb_asdict(inst);
        assert_eq!(get_dict_val(d, "x").as_int(), Some(10));
        assert_eq!(get_dict_val(d, "y").as_int(), Some(20));
    }

    #[test]
    fn test_asdict_excludes_private() {
        let inst = make_instance(
            "Pt",
            &[
                ("x", MbValue::from_int(1)),
                ("_hidden", MbValue::from_int(99)),
            ],
        );
        let d = mb_asdict(inst);
        assert_eq!(get_dict_val(d, "x").as_int(), Some(1));
        assert!(get_dict_val(d, "_hidden").is_none());
    }

    #[test]
    fn test_asdict_non_instance() {
        let d = mb_asdict(MbValue::from_int(5));
        // Should return empty dict
        assert!(d.as_ptr().is_some());
    }

    // -- mb_astuple tests --

    #[test]
    fn test_astuple_basic() {
        let inst = make_instance(
            "Pt",
            &[("x", MbValue::from_int(1)), ("y", MbValue::from_int(2))],
        );
        let t = mb_astuple(inst);
        let vals = tuple_vals(t);
        assert_eq!(vals.len(), 2);
        // Values should be 1 and 2 (order may vary due to HashMap)
        let ints: Vec<i64> = vals.iter().filter_map(|v| v.as_int()).collect();
        assert!(ints.contains(&1));
        assert!(ints.contains(&2));
    }

    #[test]
    fn test_astuple_excludes_private() {
        let inst = make_instance(
            "Pt",
            &[("a", MbValue::from_int(10)), ("_b", MbValue::from_int(20))],
        );
        let t = mb_astuple(inst);
        let vals = tuple_vals(t);
        assert_eq!(vals.len(), 1);
        assert_eq!(vals[0].as_int(), Some(10));
    }

    #[test]
    fn test_astuple_non_instance() {
        let t = mb_astuple(MbValue::from_int(5));
        let vals = tuple_vals(t);
        assert!(vals.is_empty());
    }
}
