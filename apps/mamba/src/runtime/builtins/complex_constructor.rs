use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use super::numeric_subclass_unary_operand;

/// Parse one real/imag coefficient: validate+strip PEP 515 underscores
/// (`1_000.5`), then preserve the sign of a parsed NaN (`complex("-nan")`
/// → negative-signed NaN; `f64::from_str` does not reliably set the NaN
/// sign bit).
fn parse_complex_part(part: &str) -> Option<f64> {
    let t = part.trim();
    let v = super::strip_float_underscores(t)?.parse::<f64>().ok()?;
    if v.is_nan() && t.starts_with('-') {
        Some(v.copysign(-1.0))
    } else {
        Some(v)
    }
}

/// Parse a CPython-style complex literal from a string body.
/// Accepts forms like `1+2j`, `3-4j`, `5j`, `1.5e-3+2.5e+2j`, `+j`, `j`, `1`.
/// Surrounding whitespace and a single layer of outer `(...)` are tolerated.
/// Returns `(real, imag)` on success, `None` if the string is not a valid
/// complex literal.
fn parse_complex_str(input: &str) -> Option<(f64, f64)> {
    let mut s = input.trim();
    if s.starts_with('(') && s.ends_with(')') {
        s = s[1..s.len() - 1].trim();
    }
    if s.is_empty() {
        return None;
    }
    // No 'j'/'J' -> real-only number.
    let has_imag = s.ends_with('j') || s.ends_with('J');
    if !has_imag {
        return parse_complex_part(s).map(|r| (r, 0.0));
    }
    // Strip the trailing 'j'/'J'.
    let body = &s[..s.len() - 1];
    if body.is_empty() {
        // Bare "j" -> 1j.
        return Some((0.0, 1.0));
    }
    // Find the rightmost '+' or '-' that does NOT follow an 'e'/'E'
    // (exponent sign). Skip a leading sign at position 0.
    let bytes = body.as_bytes();
    let mut split: Option<usize> = None;
    for i in (1..bytes.len()).rev() {
        let c = bytes[i];
        if c == b'+' || c == b'-' {
            let prev = bytes[i - 1];
            if prev != b'e' && prev != b'E' {
                split = Some(i);
                break;
            }
        }
    }
    match split {
        Some(idx) => {
            let real_part = body[..idx].trim();
            let imag_part = body[idx..].trim();
            let re = parse_complex_part(real_part)?;
            // Bare "+"/"-" prefix on imag means +/-1j.
            let im = if imag_part == "+" {
                1.0
            } else if imag_part == "-" {
                -1.0
            } else {
                parse_complex_part(imag_part)?
            };
            Some((re, im))
        }
        None => {
            // Whole body is the imag coefficient.
            let im = if body == "+" {
                1.0
            } else if body == "-" {
                -1.0
            } else {
                parse_complex_part(body)?
            };
            Some((0.0, im))
        }
    }
}

fn extract_complex_component(val: MbValue, is_imag: bool) -> Result<(f64, f64), ()> {
    let unwrapped = numeric_subclass_unary_operand(val, if is_imag { "__float__" } else { "__complex__" }).unwrap_or(val);
    if let Some(i) = unwrapped.as_int() {
        return Ok((i as f64, 0.0));
    }
    if let Some(f) = unwrapped.as_float() {
        return Ok((f, 0.0));
    }
    if let Some(b) = unwrapped.as_bool() {
        return Ok((b as i64 as f64, 0.0));
    }
    if let Some(ptr) = unwrapped.as_ptr() {
        unsafe {
            if let ObjData::Complex(re, im) = (*ptr).data {
                return Ok((re, im));
            }
            if let ObjData::BigInt(_) = (*ptr).data {
                match super::super::bigint_ops::int_as_f64_checked(unwrapped) {
                    Ok(f) => return Ok((f, 0.0)),
                    Err(msg) => {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(super::super::rc::MbObject::new_str("OverflowError".to_string())),
                            MbValue::from_ptr(super::super::rc::MbObject::new_str(msg)),
                        );
                        return Err(());
                    }
                }
            }
        }
    }
    if !is_imag {
        if let Some(method) = super::super::class::try_get_dunder(val, "__complex__") {
            let res = super::super::class::mb_call_method1(method, val);
            if super::super::exception::current_exception_type().is_some() {
                return Err(());
            }
            if let Some(ptr) = res.as_ptr() {
                unsafe {
                    if let ObjData::Complex(re, im) = (*ptr).data {
                        return Ok((re, im));
                    }
                }
            }
        }
    }
    if let Some(method) = super::super::class::try_get_dunder(val, "__float__") {
        let res = super::super::class::mb_call_method1(method, val);
        if super::super::exception::current_exception_type().is_some() {
            return Err(());
        }
        if let Some(f) = res.as_float() {
            return Ok((f, 0.0));
        }
        if let Some(i) = res.as_int() {
            return Ok((i as f64, 0.0));
        }
    }
    if let Some(method) = super::super::class::try_get_dunder(val, "__index__") {
        let res = super::super::class::mb_call_method1(method, val);
        if super::super::exception::current_exception_type().is_some() {
            return Err(());
        }
        if let Some(i) = res.as_int_pyint() {
            return Ok((i as f64, 0.0));
        }
    }
    let type_name = super::value_type_name(val);
    let msg = if is_imag {
        format!("complex() second argument must be a number, not '{type_name}'")
    } else {
        format!("complex() first argument must be a string or a number, not '{type_name}'")
    };
    super::raise_type_error(msg);
    Err(())
}

/// complex(real, imag) — create a complex number (R3 CPython 3.12 conformance).
/// Accepts numeric `real`/`imag`, an existing `Complex` for `real`, or a
/// string literal for `real` (CPython single-argument form).
pub fn mb_complex(real: MbValue, imag: MbValue) -> MbValue {
    let real_unwrapped = numeric_subclass_unary_operand(real, "__complex__").unwrap_or(real);
    if let Some(ptr) = real_unwrapped.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                if !imag.is_none() {
                    super::raise_type_error("complex() can't take second arg if first is a string".to_string());
                    return MbValue::none();
                }
                if let Some((re, im)) = parse_complex_str(s) {
                    return MbValue::from_ptr(MbObject::new_complex(re, im));
                }
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "could not convert string to complex: '{s}'"
                    ))),
                );
                return MbValue::none();
            }
            if let ObjData::Complex(re, im) = (*ptr).data {
                if imag.is_none() {
                    return MbValue::from_ptr(MbObject::new_complex(re, im));
                }
            }
        }
    }
    if !imag.is_none() {
        let imag_unwrapped = numeric_subclass_unary_operand(imag, "__complex__").unwrap_or(imag);
        if let Some(ptr) = imag_unwrapped.as_ptr() {
            unsafe {
                if matches!(&(*ptr).data, ObjData::Str(_)) {
                    super::raise_type_error("complex() second arg can't be string".to_string());
                    return MbValue::none();
                }
            }
        }
    }

    let (re0, im0) = match extract_complex_component(real, false) {
        Ok(pair) => pair,
        Err(()) => return MbValue::none(),
    };
    let (re1, im1) = if imag.is_none() {
        (0.0, 0.0)
    } else {
        match extract_complex_component(imag, true) {
            Ok(pair) => pair,
            Err(()) => return MbValue::none(),
        }
    };
    MbValue::from_ptr(MbObject::new_complex(re0 - im1, im0 + re1))
}
