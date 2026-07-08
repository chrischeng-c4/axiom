use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use super::{resolve_index_value, strip_pep515_underscores};

/// int(value, base) — convert string to integer with given base; raises ValueError on bad input.
pub fn mb_int_base(val: MbValue, base: MbValue) -> MbValue {
    // base accepts any SupportsIndex (int / bool / object with __index__),
    // e.g. `int("ff", Indexable(16))`.
    let Some(base_int) = resolve_index_value(base) else {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "int() base must be an integer".to_string(),
            )),
        );
        return MbValue::none();
    };
    // CPython: base is 0 (prefix auto-detect) or 2..=36; anything else raises
    // ValueError. Rust's from_str_radix panics on a radix outside 2..=36, and a
    // negative base would wrap when cast to u32 — so reject up front.
    if base_int != 0 && !(2..=36).contains(&base_int) {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "int() base must be >= 2 and <= 36, or 0".to_string(),
            )),
        );
        return MbValue::none();
    }
    let base_num = base_int as u32;
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                let full = s.clone();
                let trimmed = s.trim();
                // Returns (sign, effective_base, stripped_digits) on a valid
                // parse so the caller can build an inline i64 or, on overflow,
                // a heap BigInt. Validation of digits is deferred to the
                // numeric conversion step (i64/BigInt both reject bad digits).
                let try_parse = |t: &str| -> Option<(i64, u32, String)> {
                    // Pull off optional sign first so the radix prefix
                    // detection sees `0x`/`0o`/`0b` rather than `-0x`.
                    let (sign, rest) = match t.as_bytes().first() {
                        Some(b'-') => (-1i64, &t[1..]),
                        Some(b'+') => (1, &t[1..]),
                        _ => (1, t),
                    };
                    // base == 0: auto-detect from prefix (CPython behaviour).
                    // `0x`/`0X` → 16, `0o`/`0O` → 8, `0b`/`0B` → 2, else 10.
                    // Without a prefix the value cannot have leading zeros
                    // (CPython raises on `int("010", 0)`); we replicate that.
                    let (effective_base, digits, prefix_stripped) = if base_num == 0 {
                        if let Some(d) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X"))
                        {
                            (16u32, d, true)
                        } else if let Some(d) =
                            rest.strip_prefix("0o").or_else(|| rest.strip_prefix("0O"))
                        {
                            (8, d, true)
                        } else if let Some(d) =
                            rest.strip_prefix("0b").or_else(|| rest.strip_prefix("0B"))
                        {
                            (2, d, true)
                        } else {
                            // Decimal: forbid leading zeros except the literal
                            // `0` (or `0_0`, etc.) — match CPython.
                            let bare = rest.trim_start_matches('+');
                            let nonzero = bare.trim_start_matches('0').trim_start_matches('_');
                            if !bare.is_empty()
                                && bare != "0"
                                && nonzero != bare
                                && !nonzero.is_empty()
                            {
                                return None;
                            }
                            (10, rest, false)
                        }
                    } else if base_num == 16 {
                        rest.strip_prefix("0x")
                            .or_else(|| rest.strip_prefix("0X"))
                            .map(|d| (16, d, true))
                            .unwrap_or((16, rest, false))
                    } else if base_num == 8 {
                        rest.strip_prefix("0o")
                            .or_else(|| rest.strip_prefix("0O"))
                            .map(|d| (8, d, true))
                            .unwrap_or((8, rest, false))
                    } else if base_num == 2 {
                        rest.strip_prefix("0b")
                            .or_else(|| rest.strip_prefix("0B"))
                            .map(|d| (2, d, true))
                            .unwrap_or((2, rest, false))
                    } else {
                        (base_num, rest, false)
                    };
                    // PEP 515: a single underscore is allowed immediately
                    // after a radix prefix (e.g. `0x_FF`). Otherwise no
                    // leading/trailing/consecutive underscores.
                    let stripped = strip_pep515_underscores(digits, prefix_stripped)?;
                    if stripped.is_empty() {
                        return None;
                    }
                    Some((sign, effective_base, stripped))
                };
                if let Some((sign, effective_base, stripped)) = try_parse(trimmed) {
                    // Fast path: fits in i64 inline range.
                    if let Ok(mag) = i64::from_str_radix(&stripped, effective_base) {
                        if super::super::bigint_ops::fits_inline(sign * mag) {
                            return MbValue::from_int(sign * mag);
                        }
                    }
                    // Overflow path: parse as an arbitrary-precision BigInt so
                    // values beyond the 48-bit inline range (e.g. a 128-bit
                    // `int(uuid.hex, 16)`) round-trip exactly.
                    if let Some(big) =
                        num_bigint::BigInt::parse_bytes(stripped.as_bytes(), effective_base)
                    {
                        let signed = if sign < 0 { -big } else { big };
                        return super::super::bigint_ops::bigint_from_big(signed);
                    }
                }
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "invalid literal for int() with base {base_num}: '{full}'"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // base given but the value is bytes-like: parse its ASCII like a string
    // (`int(b"ff", 16) == 255`).
    if let Some(ptr) = val.as_ptr() {
        let bytes_text: Option<Vec<u8>> = unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => Some(b.clone()),
                ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
                _ => None,
            }
        };
        if let Some(raw) = bytes_text {
            let text = String::from_utf8_lossy(&raw).into_owned();
            let s_obj = MbValue::from_ptr(MbObject::new_str(text));
            return mb_int_base(s_obj, base);
        }
    }
    // An explicit base requires a string/bytes value (CPython: `int(123, 10)`
    // raises TypeError, not a silent 0).
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "int() can't convert non-string with explicit base".to_string(),
        )),
    );
    MbValue::none()
}
