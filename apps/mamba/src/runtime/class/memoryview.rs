use super::*;

pub(super) fn memoryview_field(view: MbValue, name: &str) -> Option<MbValue> {
    let ptr = view.as_ptr()?;
    unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if class_name == "memoryview" {
                return fields.read().unwrap().get(name).copied();
            }
        }
    }
    None
}

pub(super) fn is_memoryview_instance(value: MbValue) -> bool {
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            ObjData::Instance { class_name, .. } if class_name == "memoryview"
        )
    })
}

pub(super) fn memoryview_tuple(values: &[i64]) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(
        values.iter().map(|v| MbValue::from_int(*v)).collect(),
    ))
}

pub(super) fn memoryview_i64_items(value: MbValue) -> Option<Vec<i64>> {
    let ptr = value.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(crate::runtime::builtins::resolve_index_value(*item)?);
                }
                Some(out)
            }
            ObjData::List(lock) => {
                let items = lock.read().unwrap();
                let mut out = Vec::with_capacity(items.len());
                for item in items.iter() {
                    out.push(crate::runtime::builtins::resolve_index_value(*item)?);
                }
                Some(out)
            }
            _ => None,
        }
    }
}

pub(super) fn memoryview_dict_str_get(value: MbValue, key: &str) -> Option<MbValue> {
    let ptr = value.as_ptr()?;
    unsafe {
        if let ObjData::Dict(lock) = &(*ptr).data {
            lock.read()
                .unwrap()
                .get(&crate::runtime::dict_ops::DictKey::Str(key.to_string()))
                .copied()
        } else {
            None
        }
    }
}

pub(super) fn memoryview_is_dict(value: MbValue) -> bool {
    value
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!(&(*ptr).data, ObjData::Dict(_)) })
}

pub(super) fn memoryview_format_itemsize(format: &str) -> Option<i64> {
    let core = format.strip_prefix('@').unwrap_or(format);
    if core.starts_with(|c| matches!(c, '=' | '<' | '>' | '!')) {
        return None;
    }
    if core.chars().count() != 1 {
        return None;
    }
    match core.chars().next().unwrap_or('B') {
        'b' | 'B' | 'c' => Some(1),
        'h' | 'H' | 'e' => Some(2),
        'i' | 'I' | 'l' | 'L' | 'f' => Some(4),
        'q' | 'Q' | 'd' => Some(8),
        _ => None,
    }
}

pub(super) fn memoryview_format(view: MbValue) -> String {
    memoryview_field(view, "_format")
        .and_then(extract_str)
        .unwrap_or_else(|| "B".to_string())
}

pub(super) fn memoryview_itemsize(view: MbValue) -> i64 {
    memoryview_field(view, "_itemsize")
        .and_then(|v| v.as_int())
        .or_else(|| memoryview_format_itemsize(&memoryview_format(view)))
        .unwrap_or(1)
}

pub(super) fn memoryview_nbytes(view: MbValue) -> i64 {
    memoryview_field(view, "_buffer")
        .and_then(crate::runtime::builtins::try_bytes_like)
        .map(|data| data.len() as i64)
        .unwrap_or(0)
}

pub(super) fn memoryview_shape_values(view: MbValue) -> Vec<i64> {
    if let Some(shape) = memoryview_field(view, "_shape").and_then(memoryview_i64_items) {
        return shape;
    }
    let itemsize = memoryview_itemsize(view).max(1);
    vec![memoryview_nbytes(view) / itemsize]
}

pub(super) fn memoryview_strides_from_shape(itemsize: i64, shape: &[i64]) -> Vec<i64> {
    if shape.is_empty() {
        return vec![];
    }
    let mut strides = vec![itemsize.max(1); shape.len()];
    let mut stride = itemsize.max(1);
    for idx in (0..shape.len()).rev() {
        strides[idx] = stride;
        stride = stride.saturating_mul(shape[idx].max(1));
    }
    strides
}

pub(super) fn memoryview_strides_values(view: MbValue) -> Vec<i64> {
    if let Some(strides) = memoryview_field(view, "_strides").and_then(memoryview_i64_items) {
        return strides;
    }
    let shape = memoryview_shape_values(view);
    memoryview_strides_from_shape(memoryview_itemsize(view), &shape)
}

pub(super) fn memoryview_readonly(view: MbValue) -> bool {
    if let Some(ro) = memoryview_field(view, "_readonly") {
        return ro.as_bool() == Some(true);
    }
    let Some(buf) = memoryview_field(view, "_buffer") else {
        return true;
    };
    if let Some(id) = buf.as_int() {
        if crate::runtime::stdlib::array_mod::is_array_handle(id as u64) {
            return false;
        }
    }
    let writable = buf.as_ptr().map_or(false, |bp| unsafe {
        matches!((*bp).data, ObjData::ByteArray(_))
    });
    !writable
}

/// True once `release()` has flipped the view's `_released` flag (issue #903:
/// released views must reject index/slice/cast access, matching CPython's
/// "operation forbidden on released memoryview object" ValueError).
pub(super) fn memoryview_released(view: MbValue) -> bool {
    memoryview_field(view, "_released")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false)
}

pub(super) fn memoryview_raise_released() {
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "operation forbidden on released memoryview object".to_string(),
        )),
    );
}

/// Number of logical elements a (start, stop, step) slice selects out of a
/// dimension of length `len`, mirroring the walk in `memoryview_slice`
/// without materializing the bytes. Used to validate slice-assignment RHS
/// length (CPython: memoryview never resizes on `mv[a:b] = rhs`).
pub(super) fn memoryview_slice_item_count(
    len: i64,
    start: MbValue,
    stop: MbValue,
    step: MbValue,
) -> i64 {
    let stride = step.as_int().unwrap_or(1);
    let stride = if stride == 0 { 1 } else { stride };
    let normalize = |value: MbValue, default: i64| -> i64 {
        let mut idx = value.as_int().unwrap_or(default);
        if idx < 0 {
            idx += len;
        }
        idx.max(0).min(len)
    };
    if stride > 0 {
        let idx0 = normalize(start, 0);
        let end = normalize(stop, len);
        if end > idx0 {
            (end - idx0 + stride - 1) / stride
        } else {
            0
        }
    } else {
        let idx0 = if start.is_none() {
            len - 1
        } else {
            normalize(start, len - 1)
        };
        let end = if stop.is_none() {
            -1
        } else {
            normalize(stop, -1)
        };
        if idx0 > end {
            (idx0 - end + (-stride) - 1) / (-stride)
        } else {
            0
        }
    }
}

pub(super) fn memoryview_element_at(data: &[u8], format: &str, index: usize) -> MbValue {
    let Some(itemsize) = memoryview_format_itemsize(format).map(|v| v as usize) else {
        return MbValue::none();
    };
    let offset = index.saturating_mul(itemsize);
    if offset + itemsize > data.len() {
        return MbValue::none();
    }
    let core = format.strip_prefix('@').unwrap_or(format);
    match core.chars().next().unwrap_or('B') {
        'B' => MbValue::from_int(data[offset] as i64),
        'b' => MbValue::from_int(i8::from_ne_bytes([data[offset]]) as i64),
        'c' => MbValue::from_ptr(MbObject::new_bytes(vec![data[offset]])),
        'H' => MbValue::from_int(u16::from_ne_bytes([data[offset], data[offset + 1]]) as i64),
        'h' => MbValue::from_int(i16::from_ne_bytes([data[offset], data[offset + 1]]) as i64),
        'I' | 'L' => MbValue::from_int(u32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as i64),
        'i' | 'l' => MbValue::from_int(i32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as i64),
        'Q' => MbValue::from_int(u64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as i64),
        'q' => MbValue::from_int(i64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ])),
        'f' => MbValue::from_float(f32::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as f64),
        'd' => MbValue::from_float(f64::from_ne_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ])),
        _ => MbValue::none(),
    }
}

pub(super) fn memoryview_flat_items(data: &[u8], format: &str) -> Vec<MbValue> {
    let itemsize = memoryview_format_itemsize(format).unwrap_or(1).max(1) as usize;
    let len = data.len() / itemsize;
    (0..len)
        .map(|idx| memoryview_element_at(data, format, idx))
        .collect()
}

pub(super) fn memoryview_nested_list(flat: &[MbValue], shape: &[i64]) -> MbValue {
    if shape.len() <= 1 {
        let len = shape.first().copied().unwrap_or(flat.len() as i64).max(0) as usize;
        return MbValue::from_ptr(MbObject::new_list(flat.iter().take(len).copied().collect()));
    }
    let rows = shape[0].max(0) as usize;
    let row_width = shape[1..].iter().fold(1usize, |acc, dim| {
        acc.saturating_mul((*dim).max(0) as usize)
    });
    let mut out = Vec::with_capacity(rows);
    for row in 0..rows {
        let start = row.saturating_mul(row_width);
        let end = (start + row_width).min(flat.len());
        out.push(memoryview_nested_list(&flat[start..end], &shape[1..]));
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

pub(super) fn memoryview_slice(
    view: MbValue,
    start: MbValue,
    stop: MbValue,
    step: MbValue,
) -> MbValue {
    let Some(buf) = memoryview_field(view, "_buffer") else {
        return MbValue::from_ptr(MbObject::new_instance("memoryview".to_string()));
    };
    let itemsize = memoryview_itemsize(view).max(1);
    let data = crate::runtime::builtins::try_bytes_like(buf).unwrap_or_default();
    let len = memoryview_shape_values(view).first().copied().unwrap_or(0);
    let stride = step.as_int().unwrap_or(1);
    let stride = if stride == 0 { 1 } else { stride };
    let normalize = |value: MbValue, default: i64| -> i64 {
        let mut idx = value.as_int().unwrap_or(default);
        if idx < 0 {
            idx += len;
        }
        idx.max(0).min(len)
    };
    let mut result = Vec::new();
    if stride > 0 {
        let mut idx = normalize(start, 0);
        let end = normalize(stop, len);
        while idx < end {
            let offset = idx.saturating_mul(itemsize) as usize;
            let width = itemsize as usize;
            if offset + width <= data.len() {
                result.extend_from_slice(&data[offset..offset + width]);
            }
            idx += stride;
        }
    } else {
        let mut idx = if start.is_none() {
            len - 1
        } else {
            normalize(start, len - 1)
        };
        let end = if stop.is_none() {
            -1
        } else {
            normalize(stop, -1)
        };
        while idx > end {
            let offset = idx.saturating_mul(itemsize) as usize;
            let width = itemsize as usize;
            if offset + width <= data.len() {
                result.extend_from_slice(&data[offset..offset + width]);
            }
            idx += stride;
        }
    }
    let sliced = MbValue::from_ptr(MbObject::new_bytes(result));
    let nbytes = crate::runtime::builtins::try_bytes_like(sliced)
        .map(|data| data.len() as i64)
        .unwrap_or(0);
    let shape = vec![nbytes / itemsize];
    let format = memoryview_format(view);
    let strides = memoryview_strides_from_shape(itemsize, &shape);
    let inst = MbObject::new_instance("memoryview".to_string());
    unsafe {
        if let ObjData::Instance { fields, .. } = &(*inst).data {
            let mut f = fields.write().unwrap();
            f.insert("_buffer".to_string(), sliced);
            let obj = memoryview_field(view, "_obj").unwrap_or(buf);
            crate::runtime::rc::retain_if_ptr(obj);
            f.insert("_obj".to_string(), obj);
            f.insert(
                "_readonly".to_string(),
                MbValue::from_bool(memoryview_readonly(view)),
            );
            f.insert("_contiguous".to_string(), MbValue::from_bool(stride == 1));
            f.insert("_stride".to_string(), MbValue::from_int(stride));
            f.insert(
                "_format".to_string(),
                MbValue::from_ptr(MbObject::new_str(format)),
            );
            f.insert("_itemsize".to_string(), MbValue::from_int(itemsize));
            f.insert("_shape".to_string(), memoryview_tuple(&shape));
            f.insert("_strides".to_string(), memoryview_tuple(&strides));
        }
    }
    MbValue::from_ptr(inst)
}
