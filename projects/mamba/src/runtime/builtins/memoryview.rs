use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// memoryview(obj) — view over a bytes-like source.
/// Stored as Instance(class_name="memoryview") with `_buffer` holding the
/// readable bytes-like payload and buffer metadata used by cast/index/list.
pub fn mb_memoryview(obj: MbValue) -> MbValue {
    let Some(bytes) = super::try_bytes_like(obj) else {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "memoryview: a bytes-like object is required, not '{}'",
                super::value_type_name(obj)
            ))),
        );
        return MbValue::none();
    };
    let inherited = obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if class_name == "memoryview" {
                let f = fields.read().unwrap();
                return Some((
                    f.get("_obj").copied(),
                    f.get("_contiguous").copied(),
                    f.get("_stride").copied(),
                    f.get("_readonly").copied(),
                    f.get("_format").copied(),
                    f.get("_itemsize").copied(),
                    f.get("_shape").copied(),
                    f.get("_strides").copied(),
                ));
            }
        }
        None
    });
    let array_metadata = obj.as_int().and_then(|id| {
        if super::super::stdlib::array_mod::is_array_handle(id as u64) {
            let format =
                super::mb_str_value(super::super::stdlib::array_mod::mb_array_typecode_attr(obj))
                    .unwrap_or_else(|| "B".to_string());
            let itemsize = super::super::stdlib::array_mod::mb_array_itemsize_attr(obj)
                .as_int()
                .unwrap_or(1)
                .max(1);
            Some((format, itemsize))
        } else {
            None
        }
    });
    let inst = MbObject::new_instance("memoryview".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut f = fields.write().unwrap();
            super::super::rc::retain_if_ptr(obj);
            f.insert("_buffer".to_string(), obj);
            let obj_field = inherited.and_then(|m| m.0).unwrap_or(obj);
            super::super::rc::retain_if_ptr(obj_field);
            f.insert("_obj".to_string(), obj_field);
            if let Some((_, contiguous, stride, readonly, format, itemsize, shape, strides)) =
                inherited
            {
                if let Some(v) = contiguous {
                    f.insert("_contiguous".to_string(), v);
                }
                if let Some(v) = stride {
                    f.insert("_stride".to_string(), v);
                }
                if let Some(v) = readonly {
                    f.insert("_readonly".to_string(), v);
                }
                for (key, value) in [
                    ("_format", format),
                    ("_itemsize", itemsize),
                    ("_shape", shape),
                    ("_strides", strides),
                ] {
                    if let Some(v) = value {
                        super::super::rc::retain_if_ptr(v);
                        f.insert(key.to_string(), v);
                    }
                }
            } else {
                let (format, itemsize) = array_metadata.unwrap_or_else(|| ("B".to_string(), 1));
                let elements = (bytes.len() as i64) / itemsize;
                f.insert("_contiguous".to_string(), MbValue::from_bool(true));
                f.insert("_stride".to_string(), MbValue::from_int(1));
                f.insert(
                    "_format".to_string(),
                    MbValue::from_ptr(MbObject::new_str(format)),
                );
                f.insert("_itemsize".to_string(), MbValue::from_int(itemsize));
                f.insert(
                    "_shape".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(elements)])),
                );
                f.insert(
                    "_strides".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(itemsize)])),
                );
            }
        }
    }
    MbValue::from_ptr(inst)
}
