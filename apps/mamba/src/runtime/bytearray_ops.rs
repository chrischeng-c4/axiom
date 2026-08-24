/// ByteArray operations for the Mamba runtime.
///
/// Implements Python-compatible bytearray methods and mutations.

use super::bytes_ops::{
    as_bytes_cloned, bytes_strip_impl, drain_handle_to_u8s, raise_count_overflow,
    raise_index_error, raise_negative_count, raise_type_error, raise_value_error,
    str_from_value, try_iterable_to_u8s, try_sequence_getitem_to_u8s, validated_bytes_from_items,
    validated_bytes_from_list,
};
use super::rc::{MbObject, ObjData};
use super::value::MbValue;

/// Create bytearray from a string, bytes, or iterable.
pub fn mb_bytearray_new(source: MbValue) -> MbValue {
    if source.is_none() {
        return MbValue::from_ptr(MbObject::new_bytearray(Vec::new()));
    }
    if super::iter::is_iter_handle(source) {
        unsafe {
            return match drain_handle_to_u8s(source) {
                Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                None => MbValue::none(),
            };
        }
    }
    if let Some(n) = source.as_int() {
        if n < 0 {
            raise_negative_count();
            return MbValue::none();
        }
        return MbValue::from_ptr(MbObject::new_bytearray(vec![0u8; n as usize]));
    }
    if let Some(ptr) = source.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    return MbValue::from_ptr(MbObject::new_bytearray(s.as_bytes().to_vec()));
                }
                ObjData::Bytes(data) => {
                    return MbValue::from_ptr(MbObject::new_bytearray(data.clone()));
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    return MbValue::from_ptr(MbObject::new_bytearray(data.clone()));
                }
                ObjData::List(ref lock) => {
                    let guard = lock.read().unwrap();
                    return match validated_bytes_from_list(&guard) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                        None => MbValue::none(),
                    };
                }
                ObjData::Tuple(ref items) => {
                    return match validated_bytes_from_items(items) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                        None => MbValue::none(),
                    };
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap().to_vec();
                    return match validated_bytes_from_items(&items) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                        None => MbValue::none(),
                    };
                }
                ObjData::FrozenSet(ref items) => {
                    return match validated_bytes_from_items(items) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                        None => MbValue::none(),
                    };
                }
                ObjData::Instance { ref class_name, .. } if class_name == "memoryview" => {
                    if let Some(data) = super::builtins::try_bytes_like(source) {
                        return MbValue::from_ptr(MbObject::new_bytearray(data));
                    }
                }
                ObjData::Instance { ref class_name, .. } => {
                    match try_iterable_to_u8s(source) {
                        Some(Some(data)) => {
                            return MbValue::from_ptr(MbObject::new_bytearray(data));
                        }
                        Some(None) => {}
                        None => return MbValue::none(),
                    }
                    match try_sequence_getitem_to_u8s(source, class_name) {
                        Some(Some(data)) => {
                            return MbValue::from_ptr(MbObject::new_bytearray(data));
                        }
                        Some(None) => {}
                        None => return MbValue::none(),
                    }
                }
                ObjData::BigInt(_) => {
                    raise_count_overflow();
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_bytearray(Vec::new()))
}

pub fn mb_bytearray_new_checked(source: MbValue) -> MbValue {
    if str_from_value(source).is_some() {
        raise_type_error("string argument without an encoding");
        return MbValue::none();
    }
    mb_bytearray_new(source)
}

pub fn mb_bytearray_new_encoded(source: MbValue, encoding: MbValue) -> MbValue {
    if let Some(s) = str_from_value(source) {
        let Some(enc) = str_from_value(encoding) else {
            raise_type_error("encoding must be str");
            return MbValue::none();
        };
        return match super::bytes_ops::encode_str_with_encoding(&s, &enc) {
            Some(data) => MbValue::from_ptr(MbObject::new_bytearray(data)),
            None => MbValue::none(),
        };
    }
    mb_bytearray_new(source)
}

// ── ByteArray mutations ──

/// bytearray.append(int) — append an integer byte (range 0..256).
pub fn mb_bytearray_append(ba: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let Some(v) = value.as_int() else {
                    raise_type_error(&format!(
                        "an integer is required (got type {})",
                        super::builtins::value_type_name(value)
                    ));
                    return;
                };
                if !(0..=255).contains(&v) {
                    raise_value_error("byte must be in range(0, 256)");
                    return;
                }
                lock.write().unwrap().push(v as u8);
            }
        }
    }
}

/// bytearray.extend(iterable) — accepts bytes/bytearray, int iterators, or list/tuple of ints.
pub fn mb_bytearray_extend(ba: MbValue, other: MbValue) {
    unsafe {
        if let Some(other_data) = as_bytes_cloned(other) {
            if let Some(ptr) = ba.as_ptr() {
                if let ObjData::ByteArray(ref lock) = (*ptr).data {
                    lock.write().unwrap().extend_from_slice(&other_data);
                }
            }
            return;
        }
        if str_from_value(other).is_some() {
            raise_type_error("can't concat str to bytearray");
            return;
        }
        if super::iter::is_iter_handle(other) {
            if let Some(data) = drain_handle_to_u8s(other) {
                if let Some(ptr) = ba.as_ptr() {
                    if let ObjData::ByteArray(ref lock) = (*ptr).data {
                        lock.write().unwrap().extend_from_slice(&data);
                    }
                }
            }
            return;
        }
        if let Some(ptr) = other.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                if let Some(data) = validated_bytes_from_list(&guard) {
                    if let Some(ba_ptr) = ba.as_ptr() {
                        if let ObjData::ByteArray(ref ba_lock) = (*ba_ptr).data {
                            ba_lock.write().unwrap().extend_from_slice(&data);
                        }
                    }
                }
            }
        }
    }
}

/// bytearray.clear() — clear the buffer in place.
pub fn mb_bytearray_clear(ba: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
}

/// bytearray.pop() -> int — remove and return the last byte.
pub fn mb_bytearray_pop(ba: MbValue) -> MbValue {
    mb_bytearray_pop_at(ba, MbValue::none())
}

/// bytearray.pop(index=-1) -> int — remove and return byte at index.
pub fn mb_bytearray_pop_at(ba: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let mut v = lock.write().unwrap();
                let len = v.len() as i64;
                if len == 0 {
                    drop(v);
                    raise_index_error("pop from empty bytearray");
                    return MbValue::none();
                }
                let raw = index.as_int_pyint().unwrap_or(-1);
                let actual = if raw < 0 { raw + len } else { raw };
                if actual < 0 || actual >= len {
                    drop(v);
                    raise_index_error("pop index out of range");
                    return MbValue::none();
                }
                let b = v.remove(actual as usize);
                return MbValue::from_int(b as i64);
            }
        }
        MbValue::none()
    }
}

/// bytearray.insert(index, byte) — insert integer byte before index.
pub fn mb_bytearray_insert(ba: MbValue, index: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let Some(b) = value.as_int() else {
                    raise_type_error(&format!(
                        "an integer is required (got type {})",
                        super::builtins::value_type_name(value)
                    ));
                    return;
                };
                if !(0..=255).contains(&b) {
                    raise_value_error("byte must be in range(0, 256)");
                    return;
                }
                let mut v = lock.write().unwrap();
                let len = v.len() as i64;
                let mut idx = index.as_int().unwrap_or(0);
                if idx < 0 {
                    idx += len;
                    if idx < 0 {
                        idx = 0;
                    }
                }
                if idx > len {
                    idx = len;
                }
                v.insert(idx as usize, b as u8);
            }
        }
    }
}

/// bytearray.remove(value) — remove first occurrence of byte.
pub fn mb_bytearray_remove(ba: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                if let Some(b) = value.as_int() {
                    let target = b as u8;
                    let mut v = lock.write().unwrap();
                    if let Some(pos) = v.iter().position(|&x| x == target) {
                        v.remove(pos);
                    } else {
                        drop(v);
                        raise_value_error("value not found in bytearray");
                    }
                }
            }
        }
    }
}

/// del bytearray[i] / del bytearray[start:stop:step]
pub fn mb_bytearray_delitem(ba: MbValue, key: MbValue) {
    unsafe {
        let Some(ptr) = ba.as_ptr() else { return };
        let ObjData::ByteArray(ref lock) = (*ptr).data else {
            return;
        };
        if let Some(kp) = key.as_ptr() {
            if let ObjData::Tuple(ref t) = (*kp).data {
                if t.len() == 3 {
                    let mut v = lock.write().unwrap();
                    let len = v.len() as i64;
                    let norm = |o: Option<i64>, dflt: i64| -> i64 {
                        match o {
                            Some(i) => (if i < 0 { i + len } else { i }).clamp(0, len),
                            None => dflt,
                        }
                    };
                    let step = t[2].as_int().unwrap_or(1);
                    if step == 1 {
                        let s = norm(t[0].as_int(), 0);
                        let e = norm(t[1].as_int(), len).max(s);
                        v.drain(s as usize..e as usize);
                    } else if step > 1 {
                        let s = norm(t[0].as_int(), 0);
                        let e = norm(t[1].as_int(), len);
                        let mut idx = s;
                        let mut rm: Vec<usize> = Vec::new();
                        while idx < e {
                            rm.push(idx as usize);
                            idx += step;
                        }
                        for &r in rm.iter().rev() {
                            v.remove(r);
                        }
                    }
                    return;
                }
            }
        }
        if let Some(idx) = key.as_int() {
            let mut v = lock.write().unwrap();
            let len = v.len() as i64;
            let actual = if idx < 0 { idx + len } else { idx };
            if actual >= 0 && actual < len {
                v.remove(actual as usize);
            }
        }
    }
}

/// bytearray *= n
pub fn mb_bytearray_imul(ba: MbValue, n: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let count = n.as_int().unwrap_or(1);
                let mut v = lock.write().unwrap();
                if count <= 0 {
                    v.clear();
                    return;
                }
                let orig: Vec<u8> = v.to_vec();
                for _ in 1..count {
                    v.extend_from_slice(&orig);
                }
            }
        }
    }
}

/// bytearray.reverse()
pub fn mb_bytearray_reverse(ba: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                lock.write().unwrap().reverse();
            }
        }
    }
}

pub fn mb_bytearray_strip(ba: MbValue, chars: MbValue) -> MbValue {
    if let Some(data) = unsafe { as_bytes_cloned(ba) } {
        let stripped = bytes_strip_impl(&data, chars, true, true);
        MbValue::from_ptr(MbObject::new_bytearray(stripped))
    } else {
        MbValue::none()
    }
}

pub fn mb_bytearray_lstrip(ba: MbValue, chars: MbValue) -> MbValue {
    if let Some(data) = unsafe { as_bytes_cloned(ba) } {
        let stripped = bytes_strip_impl(&data, chars, true, false);
        MbValue::from_ptr(MbObject::new_bytearray(stripped))
    } else {
        MbValue::none()
    }
}

pub fn mb_bytearray_rstrip(ba: MbValue, chars: MbValue) -> MbValue {
    if let Some(data) = unsafe { as_bytes_cloned(ba) } {
        let stripped = bytes_strip_impl(&data, chars, false, true);
        MbValue::from_ptr(MbObject::new_bytearray(stripped))
    } else {
        MbValue::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytearray_in_place_mutations() {
        let ba = mb_bytearray_new(MbValue::from_ptr(MbObject::new_bytes(b"hello".to_vec())));

        // append
        mb_bytearray_append(ba, MbValue::from_int(b'!' as i64));
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b"hello!".to_vec()));

        // insert
        mb_bytearray_insert(ba, MbValue::from_int(0), MbValue::from_int(b'>' as i64));
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b">hello!".to_vec()));

        // pop
        let popped = mb_bytearray_pop(ba);
        assert_eq!(popped.as_int(), Some(b'!' as i64));
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b">hello".to_vec()));

        // reverse
        mb_bytearray_reverse(ba);
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b"olleh>".to_vec()));

        // clear
        mb_bytearray_clear(ba);
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(vec![]));
    }

    #[test]
    fn test_bytes_immutability_attribute_error() {
        let b = MbValue::from_ptr(MbObject::new_bytes(b"immutable".to_vec()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(65)]));
        let res = crate::runtime::bytes_ops::dispatch_bytes_method("append", b, args);
        assert!(res.is_none());
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("AttributeError")
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_memoryview_slicing_write_through() {
        let ba = mb_bytearray_new(MbValue::from_ptr(MbObject::new_bytes(b"hello world".to_vec())));
        let mv = crate::runtime::builtins::mb_memoryview(ba);

        // Slice memoryview: mv[1:5] -> "ello"
        let start = MbValue::from_int(1);
        let stop = MbValue::from_int(5);
        let step = MbValue::from_int(1);
        let sub_mv = crate::runtime::class::mb_obj_getitem(
            mv,
            MbValue::from_ptr(MbObject::new_tuple(vec![start, stop, step])),
        );

        // Write through index 0 of sub_mv ('e' -> 'E')
        crate::runtime::class::mb_obj_setitem(
            sub_mv,
            MbValue::from_int(0),
            MbValue::from_int(b'E' as i64),
        );

        // Underlying bytearray should reflect change: "hEllo world"
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b"hEllo world".to_vec()));

        // Write through slice assignment: sub_mv[1:3] = b"LL"
        let slice_key = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1),
            MbValue::from_int(3),
            MbValue::from_int(1),
        ]));
        let rhs = MbValue::from_ptr(MbObject::new_bytes(b"LL".to_vec()));
        crate::runtime::class::mb_obj_setitem(sub_mv, slice_key, rhs);

        // Underlying bytearray should reflect slice change: "hELLo world"
        assert_eq!(unsafe { as_bytes_cloned(ba) }, Some(b"hELLo world".to_vec()));
    }

    #[test]
    fn test_memoryview_cast_tolist_tobytes() {
        let ba = mb_bytearray_new(MbValue::from_ptr(MbObject::new_bytes(vec![0, 1, 2, 3])));
        let mv = crate::runtime::builtins::mb_memoryview(ba);

        let tobytes_val = crate::runtime::class::mb_call_method(
            mv,
            MbValue::from_ptr(MbObject::new_str("tobytes".to_string())),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert_eq!(unsafe { as_bytes_cloned(tobytes_val) }, Some(vec![0, 1, 2, 3]));

        let tolist_val = crate::runtime::class::mb_call_method(
            mv,
            MbValue::from_ptr(MbObject::new_str("tolist".to_string())),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert!(!tolist_val.is_none());
    }
}
