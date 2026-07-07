use crate::runtime::bigint_ops;
use crate::runtime::class;
use crate::runtime::rc::{MbObject, ObjData};
use crate::runtime::string_ops;
use crate::runtime::value::MbValue;

use super::{raise_type_error, raise_value_error, value_type_name};

/// Resolve a value to an integer "index" the way CPython's
/// `__index__`-accepting builtins (chr/hex/oct/bin) do: ints and bools pass
/// through, instances dispatch their `__index__` dunder. Returns None (no
/// exception raised) when the value cannot be interpreted as an integer.
pub(crate) fn resolve_index_value(val: MbValue) -> Option<i64> {
    if let Some(i) = val.as_int() {
        return Some(i);
    }
    if let Some(b) = val.as_bool() {
        return Some(b as i64);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let method = class::lookup_method(class_name, "__index__");
                if !method.is_none() {
                    let result = class::mb_call_method1(method, val);
                    return result.as_int();
                }
            }
        }
    }
    None
}

/// Raise CPython's "cannot be interpreted as an integer" TypeError.
fn raise_not_integer(val: MbValue) {
    raise_type_error(format!(
        "'{}' object cannot be interpreted as an integer",
        value_type_name(val)
    ));
}

/// chr(i) — return character for Unicode code point.
pub fn mb_chr(val: MbValue) -> MbValue {
    if let Some(i) = resolve_index_value(val) {
        if !(0..=0x10FFFF).contains(&i) {
            raise_value_error("chr() arg not in range(0x110000)".to_string());
            return MbValue::none();
        }
        if let Some(c) = char::from_u32(i as u32) {
            return MbValue::from_ptr(MbObject::new_str(c.to_string()));
        }
        return string_ops::new_lone_surrogate_str(i as u32);
    }
    // BigInt code points are always out of the 0x110000 range.
    if unsafe { bigint_ops::extract_bigint(val).is_some() } {
        raise_value_error("chr() arg not in range(0x110000)".to_string());
        return MbValue::none();
    }
    raise_not_integer(val);
    MbValue::none()
}

/// ord(c) — return Unicode code point for a single character.
pub fn mb_ord(val: MbValue) -> MbValue {
    if let Some(codepoint) = string_ops::surrogate_single_codepoint(val) {
        return MbValue::from_int(codepoint as i64);
    }
    if let Some(n) = string_ops::surrogate_len(val) {
        raise_type_error(format!(
            "ord() expected a character, but string of length {n} found"
        ));
        return MbValue::none();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Str(ref s) => {
                    let n = s.chars().count();
                    if n == 1 {
                        return MbValue::from_int(s.chars().next().unwrap() as i64);
                    }
                    raise_type_error(format!(
                        "ord() expected a character, but string of length {n} found"
                    ));
                    return MbValue::none();
                }
                // bytes / bytearray of length 1 are accepted by CPython.
                ObjData::Bytes(ref b) => {
                    if b.len() == 1 {
                        return MbValue::from_int(b[0] as i64);
                    }
                    raise_type_error(format!(
                        "ord() expected a character, but string of length {} found",
                        b.len()
                    ));
                    return MbValue::none();
                }
                ObjData::ByteArray(ref lock) => {
                    let b = lock.read().unwrap();
                    if b.len() == 1 {
                        return MbValue::from_int(b[0] as i64);
                    }
                    raise_type_error(format!(
                        "ord() expected a character, but string of length {} found",
                        b.len()
                    ));
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    raise_type_error(format!(
        "ord() expected string of length 1, but {} found",
        value_type_name(val)
    ));
    MbValue::none()
}

/// hex(x) — return hex string representation of an integer.
pub fn mb_hex(val: MbValue) -> MbValue {
    if let Some(i) = resolve_index_value(val) {
        let s = if i < 0 {
            format!("-0x{:x}", -i)
        } else {
            format!("0x{:x}", i)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    if let Some(big) = unsafe { bigint_ops::extract_bigint(val) } {
        let s = if big.sign() == num_bigint::Sign::Minus {
            format!("-0x{:x}", -big)
        } else {
            format!("0x{:x}", big)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    raise_not_integer(val);
    MbValue::none()
}

/// oct(x) — return octal string representation of an integer.
pub fn mb_oct(val: MbValue) -> MbValue {
    if let Some(i) = resolve_index_value(val) {
        let s = if i < 0 {
            format!("-0o{:o}", -i)
        } else {
            format!("0o{:o}", i)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    if let Some(big) = unsafe { bigint_ops::extract_bigint(val) } {
        let s = if big.sign() == num_bigint::Sign::Minus {
            format!("-0o{:o}", -big)
        } else {
            format!("0o{:o}", big)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    raise_not_integer(val);
    MbValue::none()
}

/// bin(x) — return binary string representation of an integer.
pub fn mb_bin(val: MbValue) -> MbValue {
    if let Some(i) = resolve_index_value(val) {
        let s = if i < 0 {
            format!("-0b{:b}", -i)
        } else {
            format!("0b{:b}", i)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    if let Some(big) = unsafe { bigint_ops::extract_bigint(val) } {
        let s = if big.sign() == num_bigint::Sign::Minus {
            format!("-0b{:b}", -big)
        } else {
            format!("0b{:b}", big)
        };
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    raise_not_integer(val);
    MbValue::none()
}
