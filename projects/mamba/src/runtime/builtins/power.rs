use crate::runtime::rc::MbObject;
use crate::runtime::value::MbValue;

/// Raise `ZeroDivisionError: 0.0 cannot be raised to a negative power` — the
/// CPython error for `0 ** -n` / `0.0 ** -n`.
fn raise_zero_neg_pow() -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "0.0 cannot be raised to a negative power".to_string(),
        )),
    );
    MbValue::none()
}

/// pow(base, exp) — power operator.
pub fn mb_pow(base: MbValue, exp: MbValue) -> MbValue {
    let base = super::int_enum_like_value(base).unwrap_or(base);
    let exp = super::int_enum_like_value(exp).unwrap_or(exp);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((nb, ne)) = super::numeric_subclass_operands(base, exp, "__pow__") {
        return mb_pow(nb, ne);
    }
    if let Some(r) = super::numeric_handle_binop("**", base, exp) {
        return r;
    }
    if let Some(r) = super::bigint_numeric_binop("**", base, exp) {
        return r;
    }
    // Complex base: route through complex pow so `complex(3,4) ** 2` works.
    // Either operand being `ObjData::Complex` promotes the whole op to
    // complex. (#1256 sub-priority 3 — complex arithmetic)
    if super::is_complex_obj(base) || super::is_complex_obj(exp) {
        if let (Some((br, bi)), Some((er, ei))) =
            (super::as_complex_pair(base), super::as_complex_pair(exp))
        {
            // Integer exponent on a complex base — exact via repeated
            // multiplication (avoids polar-form precision loss for the
            // common `c**2`, `c**3` cases).
            if ei == 0.0 && er.fract() == 0.0 && er.abs() < (i32::MAX as f64) {
                let n = er as i32;
                let (mut rr, mut ri) = if n < 0 {
                    // 1/(a+bi) = (a-bi)/(a²+b²)
                    let denom = br * br + bi * bi;
                    if denom == 0.0 {
                        return MbValue::none();
                    }
                    (br / denom, -bi / denom)
                } else {
                    (br, bi)
                };
                let count = n.unsigned_abs();
                if count == 0 {
                    return MbValue::from_ptr(MbObject::new_complex(1.0, 0.0));
                }
                let (sr, si) = (rr, ri);
                for _ in 1..count {
                    let new_r = rr * sr - ri * si;
                    let new_i = rr * si + ri * sr;
                    rr = new_r;
                    ri = new_i;
                }
                return MbValue::from_ptr(MbObject::new_complex(rr, ri));
            }
            // General complex pow via polar form: c**e = exp(e * log c)
            // where log(a+bi) = ln(r) + i*θ.
            let r = (br * br + bi * bi).sqrt();
            if r == 0.0 {
                return MbValue::from_ptr(MbObject::new_complex(0.0, 0.0));
            }
            let theta = bi.atan2(br);
            let ln_r = r.ln();
            // (er + ei*i) * (ln_r + theta*i) = (er*ln_r - ei*theta) + (er*theta + ei*ln_r)i
            let real_part = er * ln_r - ei * theta;
            let imag_part = er * theta + ei * ln_r;
            let mag = real_part.exp();
            return MbValue::from_ptr(MbObject::new_complex(
                mag * imag_part.cos(),
                mag * imag_part.sin(),
            ));
        }
        return MbValue::none();
    }
    match (base.as_int(), exp.as_int()) {
        (Some(b), Some(e)) => {
            if e >= 0 {
                // Promote out-of-payload results to BigInt (2**64 must not
                // wrap to 0 in the 48-bit NaN-boxed int payload).
                use num_bigint::BigInt;
                let big = BigInt::from(b).pow(e as u32);
                let fits = big >= BigInt::from(-(1i64 << 47)) && big < BigInt::from(1i64 << 47);
                if fits {
                    use num_traits::ToPrimitive;
                    MbValue::from_int(big.to_i64().unwrap_or(0))
                } else {
                    super::super::bigint_ops::bigint_from_big(big)
                }
            } else {
                // 0 ** -n: zero to a negative power has no finite value.
                if b == 0 {
                    return raise_zero_neg_pow();
                }
                MbValue::from_float((b as f64).powi(e as i32))
            }
        }
        _ => {
            let bf = base.as_int().map(|i| i as f64).or(base.as_float());
            let ef = exp.as_int().map(|i| i as f64).or(exp.as_float());
            match (bf, ef) {
                (Some(b), Some(e)) => {
                    // 0.0 ** -n raises ZeroDivisionError in CPython rather than
                    // returning inf.
                    if b == 0.0 && e < 0.0 {
                        return raise_zero_neg_pow();
                    }
                    MbValue::from_float(b.powf(e))
                }
                _ => {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "unsupported operand type(s) for pow()".to_string(),
                        )),
                    );
                    MbValue::none()
                }
            }
        }
    }
}

/// Modular multiplicative inverse of `a` modulo `m` via the extended
/// Euclidean algorithm, returning `x` in `[0, m)` such that `a*x ≡ 1 (mod m)`.
/// Returns `None` if `gcd(a, m) != 1` (CPython raises `ValueError` here).
fn mod_inverse_i128(a: i128, m: i128) -> Option<i128> {
    if m == 0 {
        return None;
    }
    let m_abs = m.abs();
    let (mut old_r, mut r) = (a.rem_euclid(m_abs), m_abs);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let nr = old_r - q * r;
        old_r = r;
        r = nr;
        let ns = old_s - q * s;
        old_s = s;
        s = ns;
    }
    if old_r != 1 {
        return None;
    }
    Some(old_s.rem_euclid(m_abs))
}

/// pow(base, exp, mod) — modular exponentiation.
/// CPython 3.8+: when `exp < 0`, computes the modular inverse of `base`
/// then raises it to `-exp`; valid only when `gcd(base, mod) == 1`.
pub fn mb_pow_mod(base: MbValue, exp: MbValue, modulus: MbValue) -> MbValue {
    match (
        base.as_int_pyint(),
        exp.as_int_pyint(),
        modulus.as_int_pyint(),
    ) {
        (Some(b), Some(e), Some(m)) => {
            if m == 0 {
                super::raise_value_error("pow() 3rd argument cannot be 0".to_string());
                return MbValue::none();
            }
            let m128 = m as i128;
            let (mut base_val, exp_pos): (i128, u64) = if e < 0 {
                match mod_inverse_i128(b as i128, m128) {
                    Some(inv) => (inv, (-e) as u64),
                    None => {
                        super::raise_value_error(
                            "base is not invertible for the given modulus".to_string(),
                        );
                        return MbValue::none();
                    }
                }
            } else {
                ((b as i128).rem_euclid(m128), e as u64)
            };
            let mut result: i128 = 1 % m128;
            let mut exp_val = exp_pos;
            while exp_val > 0 {
                if exp_val & 1 == 1 {
                    result = (result * base_val).rem_euclid(m128);
                }
                exp_val >>= 1;
                base_val = (base_val * base_val).rem_euclid(m128);
            }
            MbValue::from_int(result as i64)
        }
        _ => {
            super::raise_type_error(
                "pow() 3rd argument not allowed unless all arguments are integers".to_string(),
            );
            MbValue::none()
        }
    }
}
