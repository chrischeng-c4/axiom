use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// #2129 carve-out: `decimal.Decimal` and `fractions.Fraction` values are
/// integer HANDLES (NaN-boxed ints ≥ 2^40), so the dynamic binary-op
/// entry points must intercept them before their int fast paths —
/// otherwise `Decimal('0.1') + Decimal('0.2')` adds raw handle ids
/// (aborting on the 48-bit `from_int` range) and `==` compares ids.
/// The range guard inside each module's `is_*_handle` keeps primitive
/// int hot paths to a single compare before any table probe.
pub(crate) fn is_decimal_handle_value(v: MbValue) -> bool {
    v.as_int()
        .is_some_and(|id| super::super::stdlib::decimal_mod::is_decimal_handle(id as u64))
}

pub(crate) fn is_fraction_handle_value(v: MbValue) -> bool {
    v.as_int()
        .is_some_and(|id| super::super::stdlib::fractions_mod::is_fraction_handle(id as u64))
}

/// Approximate f64 readback of a Fraction handle (CPython coerces the
/// Fraction to float when the other operand is a float).
fn fraction_as_f64(v: MbValue) -> Option<f64> {
    let id = v.as_int()? as u64;
    let (n, d) = super::super::stdlib::fractions_mod::handle_num_den(id)?;
    Some(n as f64 / d as f64)
}

/// Route a binary arithmetic op through the Decimal/Fraction handle
/// protocol when either operand is such a handle. Returns `None` when
/// neither side is a numeric handle (caller falls through to its
/// regular paths).
/// True when `v` is a heap-allocated arbitrary-precision integer (BigInt) —
/// an int value too large for the 48-bit inline NaN-box range (e.g.
/// `sys.maxsize`, `2**70`).
#[inline]
pub(crate) fn is_bigint_value(v: MbValue) -> bool {
    v.as_ptr().map_or(false, |p| {
        matches!(unsafe { &(*p).data }, ObjData::BigInt(_))
    })
}

/// Raise `ZeroDivisionError(msg)` and return `None` (the value the arithmetic
/// builtins yield after raising).
pub(crate) fn raise_zero_div(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Route a numeric binary op to arbitrary-precision arithmetic when either
/// operand is a heap BigInt. The inline arms in `mb_add`/`mb_sub`/… use
/// `as_int()`, which returns `None` for a heap BigInt, so a BigInt operand
/// would otherwise skip every numeric arm and fall through to a spurious
/// `None` (e.g. `sys.maxsize - 1`) or `unsupported operand` TypeError
/// (`sys.maxsize + 1`).
///
/// Returns `None` when neither operand is a BigInt (the inline hot path is
/// untouched) or an operand is non-numeric (so the caller's type-specific arms
/// — str/list/set/datetime/… — keep running). For `//`/`%` by zero it raises
/// ZeroDivisionError directly, matching the inline integer arms' messages.
pub(super) fn bigint_numeric_binop(op: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    if !(is_bigint_value(a) || is_bigint_value(b)) {
        return None;
    }
    let num_like = |v: MbValue| v.is_int() || v.is_bool() || v.is_float() || is_bigint_value(v);
    if !(num_like(a) && num_like(b)) {
        return None;
    }
    // BigInt ⊕ float → float (CPython widens the integer operand to f64,
    // possibly to ±inf for very large magnitudes).
    if a.is_float() || b.is_float() {
        let as_f = |v: MbValue| -> f64 {
            v.as_float()
                .or_else(|| unsafe { super::super::bigint_ops::int_as_f64(v) })
                .unwrap_or(f64::NAN)
        };
        let (af, bf) = (as_f(a), as_f(b));
        return Some(match op {
            "+" => MbValue::from_float(af + bf),
            "-" => MbValue::from_float(af - bf),
            "*" => MbValue::from_float(af * bf),
            "**" => MbValue::from_float(af.powf(bf)),
            "//" => {
                if bf == 0.0 {
                    return Some(raise_zero_div("float floor division by zero"));
                }
                MbValue::from_float((af / bf).floor())
            }
            "%" => {
                if bf == 0.0 {
                    return Some(raise_zero_div("float modulo"));
                }
                let r = af % bf;
                MbValue::from_float(if r != 0.0 && r.signum() != bf.signum() {
                    r + bf
                } else {
                    r
                })
            }
            _ => return None,
        });
    }
    // Pure integer (inline / bool / BigInt) arithmetic.
    Some(unsafe {
        match op {
            "+" => super::super::bigint_ops::mb_int_add(a, b),
            "-" => super::super::bigint_ops::mb_int_sub(a, b),
            "*" => super::super::bigint_ops::mb_int_mul(a, b),
            "//" => super::super::bigint_ops::mb_int_floordiv(a, b)
                .unwrap_or_else(|| raise_zero_div("integer division or modulo by zero")),
            "%" => super::super::bigint_ops::mb_int_mod(a, b)
                .unwrap_or_else(|| raise_zero_div("integer modulo by zero")),
            "**" => match super::super::bigint_ops::mb_int_pow(a, b) {
                Some(r) => r,
                None => {
                    // Negative (or astronomically large) exponent → float.
                    let bf = super::super::bigint_ops::int_as_f64(a).unwrap_or(f64::NAN);
                    let ef = super::super::bigint_ops::int_as_f64(b).unwrap_or(f64::NAN);
                    MbValue::from_float(bf.powf(ef))
                }
            },
            _ => return None,
        }
    })
}

/// #2129/#961 guard: also called directly from `bigint_ops::mb_bigint_{add,sub,mul}`
/// (the `CheckedAdd`/`CheckedSub`/`CheckedMul` MIR lowering for statically
/// `Ty::Int` operands) so that `a + b` on a `fractions.Fraction` / `decimal.Decimal`
/// handle dispatches through the numeric-handle protocol instead of the raw
/// BigInt-aware int path — see the guard note on `mb_bigint_add`.
pub(crate) fn numeric_handle_binop(op: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    use super::super::stdlib::{decimal_mod, fractions_mod};
    if is_decimal_handle_value(a) || is_decimal_handle_value(b) {
        return Some(match op {
            "+" => decimal_mod::mb_decimal_add(a, b),
            "-" => decimal_mod::mb_decimal_sub(a, b),
            "*" => decimal_mod::mb_decimal_mul(a, b),
            "/" => decimal_mod::mb_decimal_truediv(a, b),
            "//" => decimal_mod::mb_decimal_floordiv(a, b),
            "%" => decimal_mod::mb_decimal_rem(a, b),
            "**" => decimal_mod::mb_decimal_pow(a, b),
            "divmod" => decimal_mod::mb_decimal_divmod(a, b),
            _ => return None,
        });
    }
    let a_frac = is_fraction_handle_value(a);
    let b_frac = is_fraction_handle_value(b);
    if !(a_frac || b_frac) {
        return None;
    }
    // Fraction ⊕ float → float (CPython converts the Fraction).
    if a.is_float() || b.is_float() {
        let to_f = |v: MbValue, frac: bool| {
            if frac {
                fraction_as_f64(v)
            } else {
                v.as_float()
            }
        };
        if let (Some(af), Some(bf)) = (to_f(a, a_frac), to_f(b, b_frac)) {
            return Some(match op {
                "+" => MbValue::from_float(af + bf),
                "-" => MbValue::from_float(af - bf),
                "*" => MbValue::from_float(af * bf),
                "/" => MbValue::from_float(af / bf),
                "//" => MbValue::from_float((af / bf).floor()),
                "%" => {
                    let r = af % bf;
                    MbValue::from_float(if r != 0.0 && r.signum() != bf.signum() {
                        r + bf
                    } else {
                        r
                    })
                }
                "**" => MbValue::from_float(af.powf(bf)),
                _ => return None,
            });
        }
    }
    Some(match op {
        "+" => fractions_mod::mb_fraction_add(a, b),
        "-" => fractions_mod::mb_fraction_sub(a, b),
        "*" => fractions_mod::mb_fraction_mul(a, b),
        "/" => fractions_mod::mb_fraction_truediv(a, b),
        "//" => fractions_mod::mb_fraction_floordiv(a, b),
        "%" => fractions_mod::mb_fraction_mod(a, b),
        "**" => fractions_mod::mb_fraction_pow(a, b),
        "divmod" => fractions_mod::mb_fraction_divmod(a, b),
        _ => return None,
    })
}

/// Python floor division/modulo on i128 (quotient rounds toward -inf,
/// remainder takes the divisor's sign).
pub(crate) fn floor_divmod_i128(a: i128, b: i128) -> (i128, i128) {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        (q - 1, r + b)
    } else {
        (q, r)
    }
}
