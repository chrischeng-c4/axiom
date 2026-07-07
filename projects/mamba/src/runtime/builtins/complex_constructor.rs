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

/// complex(real, imag) — create a complex number (R3 CPython 3.12 conformance).
/// Accepts numeric `real`/`imag`, an existing `Complex` for `real`, or a
/// string literal for `real` (CPython single-argument form).
pub fn mb_complex(real: MbValue, imag: MbValue) -> MbValue {
    // complex(P(7)) / complex(F(2.5)) - unwrap int/float-SUBCLASS instances
    // to their raw numeric payload up front so the real/imag extraction below
    // (which only recognizes plain int/float/bool) sees the underlying value
    // instead of silently defaulting to 0.0. Uses the #1030 unwrap-helper
    // family; user `__complex__` overrides aren't dispatched at all yet (a
    // separate, out-of-scope dunder-protocol gap), so this only affects
    // unoverridden numeric subclasses. (#1042)
    let real = numeric_subclass_unary_operand(real, "__complex__").unwrap_or(real);
    let imag = numeric_subclass_unary_operand(imag, "__complex__").unwrap_or(imag);
    // String form: `complex("1+2j")`. CPython rejects passing a second
    // argument with a string; we silently ignore `imag` when `real` is
    // a string for now (close enough for #1256 long-tail coverage).
    // Also: complex passthrough `complex(complex(1,2))` should equal arg.
    if let Some(ptr) = real.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                if let Some((re, im)) = parse_complex_str(s) {
                    return MbValue::from_ptr(MbObject::new_complex(re, im));
                }
                // CPython: an unparseable string raises ValueError, not a silent None.
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
    let real_parts = real.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Complex(re, im) = (*ptr).data {
            Some((re, im))
        } else {
            None
        }
    });
    let imag_parts = imag.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Complex(re, im) = (*ptr).data {
            Some((re, im))
        } else {
            None
        }
    });
    let (re0, im0) = if let Some((re, im)) = real_parts {
        (re, im)
    } else if let Some(f) = real.as_float() {
        (f, 0.0)
    } else if let Some(i) = real.as_int() {
        (i as f64, 0.0)
    } else if let Some(b) = real.as_bool() {
        (b as i64 as f64, 0.0)
    } else {
        (0.0, 0.0)
    };
    let (re1, im1) = if imag.is_none() {
        (0.0, 0.0)
    } else if let Some((re, im)) = imag_parts {
        (re, im)
    } else if let Some(f) = imag.as_float() {
        (f, 0.0)
    } else if let Some(i) = imag.as_int() {
        (i as f64, 0.0)
    } else if let Some(b) = imag.as_bool() {
        (b as i64 as f64, 0.0)
    } else {
        (0.0, 0.0)
    };
    MbValue::from_ptr(MbObject::new_complex(re0 - im1, im0 + re1))
}
