use super::super::stdlib::{datetime_mod, decimal_mod, fractions_mod};
use super::super::{bigint_ops, class, exception, rc, string_ops};
use super::*;

/// Python-compatible banker's rounding (round half to even) for a scaled float.
///
/// Applies to `f` directly (i.e. call with `f * factor` when rounding to N decimal places).
/// Exactly-halfway cases (fractional part == 0.5) round to the nearest even integer.
/// All other cases delegate to `f64::round()` (rounds half away from zero).
#[inline]
fn bankers_round(f: f64) -> f64 {
    let floor = f.floor();
    let frac = f - floor;
    if frac == 0.5 {
        // Exactly halfway: round to nearest even integer.
        if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        f.round()
    }
}

/// round(number, ndigits=0) — round a number using Python banker's rounding.
pub fn mb_round(val: MbValue, ndigits: MbValue) -> MbValue {
    // CPython rule: `round(x)` (no ndigits) → int; `round(x, n)` (ndigits
    // given, even `0` or negative) → same type as x. The dispatcher passes
    // `MbValue::none()` when ndigits was omitted, which is how we tell the
    // two forms apart.
    let ndigits_given = !ndigits.is_none();
    // round(x, ndigits): a given ndigits must be an integer (int / bool /
    // bignum). A str / float / other raises TypeError rather than being
    // silently coerced to 0.
    if ndigits_given {
        let is_intish = ndigits.as_int().is_some()
            || ndigits.as_bool().is_some()
            || ndigits
                .as_ptr()
                .is_some_and(|p| matches!(unsafe { &(*p).data }, ObjData::BigInt(_)));
        if !is_intish {
            exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'{}' object cannot be interpreted as an integer",
                    super::value_type_name(ndigits)
                ))),
            );
            return MbValue::none();
        }
    }
    if super::is_decimal_handle_value(val) {
        return decimal_mod::mb_decimal_round(val, ndigits, ndigits_given);
    }
    if super::is_fraction_handle_value(val) {
        return fractions_mod::mb_fraction_round(val, ndigits);
    }
    let n = match ndigits.as_int() {
        Some(i) => i,
        None => {
            // A bignum ndigits never fits i64 but is always far outside f64's
            // ~323-place resolution, so collapse it to a large sentinel of the
            // matching sign: positive → no-op rounding, negative → rounds the
            // value away toward (signed) zero.
            match unsafe { bigint_ops::extract_bigint(ndigits) } {
                Some(big) if big.sign() == num_bigint::Sign::Minus => -1024,
                Some(_) => 1024,
                None => 0,
            }
        }
    };
    if let Some(f) = val.as_float() {
        if !ndigits_given {
            // round(f) → int (banker's rounding).
            if f.is_nan() {
                exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "cannot convert float NaN to integer".to_string(),
                    )),
                );
                return MbValue::none();
            }
            if f.is_infinite() {
                exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "cannot convert float infinity to integer".to_string(),
                    )),
                );
                return MbValue::none();
            }
            return bigint_ops::int_from_f64_trunc(bankers_round(f));
        }
        if n > 0 {
            // A finite f64 resolves at most ~323 decimal places (smallest normal
            // ≈ 2.2e-308); rounding to more places than that cannot change the
            // value, and Rust's float formatter panics on an out-of-range
            // precision (e.g. `round(x, 2**31)`). Treat huge ndigits as a no-op.
            if n > 323 {
                return MbValue::from_float(f);
            }
            // Use format/parse to avoid FP multiply artifacts (e.g. 2.675*100=267.5 in f64).
            // Rust's {:.N} formatting rounds the actual f64 value correctly — matching CPython.
            let s = format!("{:.prec$}", f, prec = n as usize);
            return MbValue::from_float(s.parse::<f64>().unwrap_or(f));
        }
        // n <= 0: multiply-based rounding; CPython keeps the float type
        // when ndigits is given, so cast back through f64.
        let factor = 10.0_f64.powi(n as i32);
        // ndigits so negative the rounding unit (10**-n) exceeds the f64 range:
        // every finite value rounds to (signed) zero.
        if factor == 0.0 {
            return MbValue::from_float(if f.is_finite() {
                0.0_f64.copysign(f)
            } else {
                f
            });
        }
        let rounded = bankers_round(f * factor) / factor;
        // A finite input that rounds up past the f64 range raises OverflowError
        // (CPython: `round(1.6e308, -308)` → "rounded value too large").
        if rounded.is_infinite() && f.is_finite() {
            exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "rounded value too large to represent".to_string(),
                )),
            );
            return MbValue::none();
        }
        MbValue::from_float(rounded)
    } else if let Some(i) = val.as_int() {
        if n >= 0 {
            MbValue::from_int(i)
        } else {
            // Rounding up at the inline boundary can exceed 48 bits.
            bigint_ops::int_from_i64(round_int_half_even(i, -n))
        }
    } else if let Some(big) = unsafe { bigint_ops::extract_bigint(val) } {
        if n >= 0 {
            unsafe {
                rc::retain_if_ptr(val);
            }
            val
        } else {
            bigint_ops::normalize_bigint(round_bigint_half_even(big, -n))
        }
    } else {
        // round(instance[, ndigits]) — dispatch the __round__ dunder.
        if let Some(ptr) = val.as_ptr() {
            if let ObjData::Instance { ref class_name, .. } = unsafe { &(*ptr).data } {
                let method = class::lookup_method(class_name, "__round__");
                if !method.is_none() {
                    let name = MbValue::from_ptr(MbObject::new_str("__round__".to_string()));
                    let call_args = if ndigits_given { vec![ndigits] } else { vec![] };
                    let args = MbValue::from_ptr(MbObject::new_list(call_args));
                    return class::mb_call_method(val, name, args);
                }
            }
        }
        super::raise_type_error(format!(
            "type {} doesn't define __round__ method",
            super::value_type_name(val)
        ));
        MbValue::none()
    }
}

/// round(i, -digits) for inline ints — CPython rounds half to even at the
/// 10^digits boundary (round(1350, -2) == 1400, round(1250, -2) == 1200).
/// Inline ints are < 2^47 so i128 intermediates never overflow.
fn round_int_half_even(i: i64, digits: i64) -> i64 {
    // 10^16 already exceeds twice the inline range, so everything rounds to 0.
    if digits > 16 {
        return 0;
    }
    let factor = 10i128.pow(digits as u32);
    let v = i as i128;
    let q = v.div_euclid(factor);
    let r = v.rem_euclid(factor);
    let q = match (2 * r).cmp(&factor) {
        std::cmp::Ordering::Greater => q + 1,
        std::cmp::Ordering::Equal if q % 2 != 0 => q + 1,
        _ => q,
    };
    (q * factor) as i64
}

/// round(bigint, -digits) — the same half-to-even rule over arbitrary precision.
fn round_bigint_half_even(v: num_bigint::BigInt, digits: i64) -> num_bigint::BigInt {
    use num_bigint::BigInt;
    use num_traits::Zero;

    // More digits than the number has → 0 (guards absurd factors too).
    if digits as usize > v.magnitude().to_string().len() + 1 {
        return BigInt::zero();
    }
    let factor = BigInt::from(10).pow(digits as u32);
    let mut q = &v / &factor;
    let mut r = &v % &factor;
    if r < BigInt::zero() {
        q -= 1;
        r += &factor;
    }
    let twice = 2 * &r;
    if twice > factor || (twice == factor && (&q % 2) != BigInt::zero()) {
        q += 1;
    }
    q * factor
}

/// divmod(a, b) — return (a // b, a % b) as a tuple.
/// Uses Python floor division (not C truncated division).
/// Python: q = floor(a/b), r = a - q*b  — remainder has same sign as divisor.
/// Either operand may be float; if so, both result components are floats.
pub fn mb_divmod(a: MbValue, b: MbValue) -> MbValue {
    if let Some(r) = super::numeric_handle_binop("divmod", a, b) {
        return r;
    }
    // Arbitrary-precision (BigInt) operands — `as_int()` is None for these, so
    // the inline integer arm below would miss them. (#sys.maxsize)
    if super::is_bigint_value(a) || super::is_bigint_value(b) {
        let int_like = |v: MbValue| v.is_int() || v.is_bool() || super::is_bigint_value(v);
        if int_like(a) && int_like(b) {
            return match unsafe { bigint_ops::mb_int_divmod(a, b) } {
                Some((q, r)) => MbValue::from_ptr(MbObject::new_tuple(vec![q, r])),
                None => super::raise_zero_div("integer division or modulo by zero"),
            };
        }
        if (a.is_float() || b.is_float())
            && (int_like(a) || a.is_float())
            && (int_like(b) || b.is_float())
        {
            let as_f = |v: MbValue| -> f64 {
                v.as_float()
                    .or_else(|| unsafe { bigint_ops::int_as_f64(v) })
                    .unwrap_or(f64::NAN)
            };
            let (af, bf) = (as_f(a), as_f(b));
            if bf == 0.0 {
                return super::raise_zero_div("float floor division by zero");
            }
            let q = (af / bf).floor();
            let r = af - q * bf;
            return MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_float(q),
                MbValue::from_float(r),
            ]));
        }
    }
    // divmod(timedelta, timedelta) -> (int, timedelta); int divisor raises TypeError.
    if let Some(ua) = datetime_mod::timedelta_total_us(a) {
        if let Some(ub) = datetime_mod::timedelta_total_us(b) {
            if ub == 0 {
                exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            let (q, r) = super::floor_divmod_i128(ua, ub);
            return MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(q as i64),
                datetime_mod::timedelta_from_us(r),
            ]));
        }
        exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "unsupported operand type(s) for divmod(): 'datetime.timedelta' and 'int'"
                    .to_string(),
            )),
        );
        return MbValue::none();
    }
    if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
        if bi == 0 {
            exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "integer division or modulo by zero".to_string(),
                )),
            );
            return MbValue::none();
        }
        let (q_trunc, r_trunc) = (ai / bi, ai % bi);
        let (q, r) = if r_trunc != 0 && (r_trunc < 0) != (bi < 0) {
            (q_trunc - 1, r_trunc + bi)
        } else {
            (q_trunc, r_trunc)
        };
        return MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(q),
            MbValue::from_int(r),
        ]));
    }
    let af = a.as_float().or_else(|| a.as_int().map(|v| v as f64));
    let bf = b.as_float().or_else(|| b.as_int().map(|v| v as f64));
    if let (Some(af), Some(bf)) = (af, bf) {
        if bf == 0.0 {
            exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "float floor division by zero".to_string(),
                )),
            );
            return MbValue::none();
        }
        let q = (af / bf).floor();
        let r = af - q * bf;
        return MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_float(q),
            MbValue::from_float(r),
        ]));
    }
    exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "unsupported operand type(s) for divmod(): '{}' and '{}'",
            super::value_type_name(a),
            super::value_type_name(b)
        ))),
    );
    MbValue::none()
}

/// format(value, format_spec) — format a value using a format spec string.
pub fn mb_format(val: MbValue, spec: MbValue) -> MbValue {
    // One-arg form: format(x) is format(x, "").
    if spec.is_none() {
        return string_ops::mb_format_value(
            val,
            MbValue::from_ptr(MbObject::new_str(String::new())),
        );
    }
    if !matches!(spec.as_ptr(), Some(ptr) if unsafe { matches!(&(*ptr).data, ObjData::Str(_)) }) {
        exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "format() argument 2 must be str".to_string(),
            )),
        );
        return MbValue::none();
    }
    // bytes/bytearray have no __format__ of their own: `format(b, "")` falls
    // through to str(b), but any non-empty spec raises TypeError (CPython).
    if let Some(ptr) = val.as_ptr() {
        let tn = unsafe {
            match &(*ptr).data {
                ObjData::Bytes(_) => Some("bytes"),
                ObjData::ByteArray(_) => Some("bytearray"),
                _ => None,
            }
        };
        if let Some(tn) = tn {
            let spec_nonempty = unsafe {
                spec.as_ptr()
                    .is_some_and(|p| matches!(&(*p).data, ObjData::Str(s) if !s.is_empty()))
            };
            if spec_nonempty {
                exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "unsupported format string passed to {tn}.__format__"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    string_ops::mb_format_value(val, spec)
}
