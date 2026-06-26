//! @codegen-skip: handwrite-pre-standardize
//!
//! decimal module for Mamba — Python 3.12 `decimal` stdlib.
//!
//! Provides `Decimal(val)` construction, exact decimal arithmetic
//! (`+ - * / // % **`, unary `+ - abs`), rich comparison against
//! int/float/Fraction/Decimal, and `str()` / `repr()` / `bool()` /
//! `int()` / `float()` readback, plus the fixture-covered method
//! surface (`as_tuple`, `as_integer_ratio`, `quantize`, `sqrt`,
//! `compare`, `max`/`min`, `scaleb`, `fma`, `is_nan`-family).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + bytes-borrowing extract
//! + integer-handle protocol like file_io/generator/hashlib/hmac) is not
//! yet emitted by score codegen. Tracked as part of the brute-force
//! Phase 2 sweep; will be replaced when aw standardize lands the
//! stdlib-shim section type.
//!
//! Implementation notes:
//!
//! - Decimal objects are **integer handles** (i64 IDs), backed by a
//!   thread-local `HashMap<u64, MbDecimal>` table — same protocol used by
//!   `hashlib_mod` / `fractions_mod`. The dynamic binary-op entry points
//!   in `builtins.rs` (`mb_add`, `mb_eq`, `mb_str`, ...) intercept
//!   Decimal handles before their int fast paths (#2129 carve-out), so
//!   `Decimal('0.1') + Decimal('0.2')` no longer adds raw handle ids.
//!
//! - Finite payloads use `rust_decimal::Decimal` (96-bit mantissa,
//!   scale 0..=28 — covers CPython's default 28-digit context for the
//!   fixture-covered range). Special values (Infinity / NaN / sNaN) are
//!   tracked alongside via `DecClass`, since `rust_decimal` cannot
//!   represent them. Arithmetic is performed exactly on BigInt
//!   coefficient/scale pairs and rounded half-even to 28 significant
//!   digits (CPython's default context), so `0.1 + 0.2 == 0.3` exactly
//!   and `1/3` matches CPython's 28-digit quotient.
//!
//! - Cross-type comparison (int / float / Fraction) is exact rational
//!   comparison over BigInt — `Decimal('0.1') != 0.1` holds because the
//!   binary float is expanded to its exact fraction, not round-tripped
//!   through shortest-repr parsing.
//!
//! - Known carve-outs: exponents outside rust_decimal's 0..=28 scale
//!   window (e.g. `1E-100`) collapse toward zero or stay unrepresented;
//!   context objects cover the fixture-backed subset of precision, flags,
//!   and localcontext routing, not the full CPython decimal context model.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN

use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive, Zero};
use rust_decimal::Decimal;
use std::str::FromStr;

/// CPython default context precision (significant digits).
const PREC: usize = 28;

/// Maximum scale representable by `rust_decimal`.
const MAX_SCALE: i64 = 28;

const DECIMAL_CONTEXT_FLAGS: &[&str] = &[
    "Clamped",
    "InvalidOperation",
    "DivisionByZero",
    "Inexact",
    "Rounded",
    "Subnormal",
    "Overflow",
    "Underflow",
    "FloatOperation",
];

/// Special-value classification for one Decimal handle.
#[derive(Clone, Copy, PartialEq, Eq)]
enum DecClass {
    Finite,
    Inf,
    QNan,
    SNan,
}

/// Backing state for one Decimal handle. For `Finite`, `value` carries
/// the payload (including negative zero via rust_decimal's sign flag)
/// and `neg` mirrors its sign; for specials, `neg` is the sign and
/// `value` is unused (zero).
#[derive(Clone, Copy)]
struct MbDecimal {
    class: DecClass,
    value: Decimal,
    neg: bool,
    tuple_exp: Option<i64>,
    /// Index into the `EXACT_VALUES` intern table when this finite payload
    /// carries an exact coefficient/scale that exceeds rust_decimal's window
    /// (e.g. `Decimal(0.1)` captures the full ~55-digit binary expansion).
    /// `None` for ordinary payloads, which render from `value` directly.
    /// A `Copy`-friendly index so every struct copy (arithmetic copy-through,
    /// `__copy__`, `Decimal(Decimal(...))`) carries the exact value for free.
    exact: Option<usize>,
}

impl MbDecimal {
    fn finite(value: Decimal) -> Self {
        MbDecimal {
            class: DecClass::Finite,
            neg: value.is_sign_negative(),
            value,
            tuple_exp: None,
            exact: None,
        }
    }
    fn inf(neg: bool) -> Self {
        MbDecimal {
            class: DecClass::Inf,
            value: Decimal::ZERO,
            neg,
            tuple_exp: None,
            exact: None,
        }
    }
    fn qnan() -> Self {
        MbDecimal {
            class: DecClass::QNan,
            value: Decimal::ZERO,
            neg: false,
            tuple_exp: None,
            exact: None,
        }
    }
    fn snan() -> Self {
        MbDecimal {
            class: DecClass::SNan,
            value: Decimal::ZERO,
            neg: false,
            tuple_exp: None,
            exact: None,
        }
    }
    fn is_nan(&self) -> bool {
        matches!(self.class, DecClass::QNan | DecClass::SNan)
    }
}

/// Base = 1<<46. Owns [1<<46, (1<<47) - 1] (~70.4T ids) — the last safe
/// slot before the MbValue NaN-box overflow at 1<<47. See
/// `integer_handle_registry::HANDLE_MIN_ID`.
const DECIMAL_HANDLE_BASE: u64 = 1u64 << 46;

thread_local! {
    static DECIMALS: RefCell<HashMap<u64, MbDecimal>> = RefCell::new(HashMap::new());
    static DECIMAL_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    /// Intern table for exact coefficient/scale pairs (signed coefficient,
    /// scale ≥ 0) that exceed rust_decimal's window — see `MbDecimal::exact`.
    /// Interned and never freed; entries are tiny and only float construction
    /// (a cold path) ever appends.
    static EXACT_VALUES: RefCell<Vec<(BigInt, i64)>> = const { RefCell::new(Vec::new()) };
    static NEXT_DECIMAL_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(DECIMAL_HANDLE_BASE) };
    /// Per-handle refcount (#2111).
    static DECIMAL_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_decimal_id() -> u64 {
    NEXT_DECIMAL_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Class.rs `mb_call_method` / `mb_getattr` and the builtins binary-op
/// hooks call this to decide whether to route an int value through the
/// decimal protocol. The range guard keeps primitive-int hot paths to a
/// single compare before the table probe.
pub fn is_decimal_handle(id: u64) -> bool {
    if id < DECIMAL_HANDLE_BASE {
        return false;
    }
    DECIMAL_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_decimal_handle(id: u64) {
    DECIMALS.with(|m| {
        m.borrow_mut().remove(&id);
    });
    DECIMAL_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    DECIMAL_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_decimal_handle(id) {
        return false;
    }
    DECIMAL_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_decimal_handle(id) {
        return false;
    }
    let should_drop = DECIMAL_REFCOUNTS.with(|r| {
        let mut map = r.borrow_mut();
        let rc = map.entry(id).or_insert(1);
        if *rc <= 1 {
            map.remove(&id);
            true
        } else {
            *rc -= 1;
            false
        }
    });
    if should_drop {
        drop_decimal_handle(id);
    }
    true
}

fn make_state_handle(state: MbDecimal) -> MbValue {
    let id = alloc_decimal_id();
    DECIMALS.with(|m| {
        m.borrow_mut().insert(id, state);
    });
    DECIMAL_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

fn make_handle(value: Decimal) -> MbValue {
    make_state_handle(MbDecimal::finite(value))
}

fn get_state(handle: MbValue) -> Option<MbDecimal> {
    handle.as_int().and_then(|id| {
        if (id as u64) < DECIMAL_HANDLE_BASE {
            return None;
        }
        DECIMALS.with(|m| m.borrow().get(&(id as u64)).copied())
    })
}

fn get_decimal(handle: MbValue) -> Option<Decimal> {
    get_state(handle).map(|d| d.value)
}

/// Borrow `&str` from an MbValue holding a string. Empty for other shapes.
#[inline]
fn with_str<R>(val: MbValue, f: impl FnOnce(&str) -> R) -> R {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return f(s.as_str());
            }
        }
    }
    f("")
}

/// Parse CPython special-value literals (Infinity / NaN / sNaN, optional
/// sign and NaN payload). Returns the matching state when recognised.
fn parse_special_decimal_str(s: &str) -> Option<MbDecimal> {
    let mut t = s.trim();
    if t.is_empty() {
        return None;
    }
    let mut neg = false;
    if let Some(rest) = t.strip_prefix('-') {
        neg = true;
        t = rest;
    } else if let Some(rest) = t.strip_prefix('+') {
        t = rest;
    }
    let lower = t.to_ascii_lowercase();
    if lower == "inf" || lower == "infinity" {
        return Some(MbDecimal::inf(neg));
    }
    let (payload, signaling) = if let Some(p) = lower.strip_prefix("snan") {
        (p, true)
    } else if let Some(p) = lower.strip_prefix("nan") {
        (p, false)
    } else {
        return None;
    };
    if payload.chars().all(|c| c.is_ascii_digit()) {
        let mut st = if signaling {
            MbDecimal::snan()
        } else {
            MbDecimal::qnan()
        };
        st.neg = neg;
        return Some(st);
    }
    None
}

/// Raise `decimal.InvalidOperation` and return `None`. The JIT checks the
/// pending-exception slot after the native call returns and unwinds into the
/// caller's `except decimal.InvalidOperation` handler — the module-level
/// `InvalidOperation` attribute resolves to the matching type-name string.
fn raise_invalid_operation(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("InvalidOperation".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_division_by_zero(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("DivisionByZero".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_overflow(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("Overflow".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// ── Exact coefficient/scale arithmetic core ─────────────────────────────

/// `10^n` as BigInt.
fn pow10(n: u32) -> BigInt {
    let mut p = BigInt::from(1u32);
    let ten = BigInt::from(10u32);
    for _ in 0..n {
        p *= &ten;
    }
    p
}

/// Significant-digit count of |c| (0 counts as 1 digit).
fn digit_count(c: &BigInt) -> usize {
    let s = c.magnitude().to_string();
    s.len()
}

/// Round `|c|` by dropping `drop` trailing decimal digits, half-even.
fn round_half_even_abs(c_abs: &BigInt, drop: u32) -> BigInt {
    if drop == 0 {
        return c_abs.clone();
    }
    let p = pow10(drop);
    let q = c_abs / &p;
    let r = c_abs % &p;
    let twice = &r * 2u32;
    if twice > p {
        q + 1u32
    } else if twice < p {
        q
    } else if (&q % 2u32).is_zero() {
        q
    } else {
        q + 1u32
    }
}

/// Rounding modes for `quantize` / `round_at`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rounding {
    Down,
    Up,
    HalfUp,
    HalfDown,
    HalfEven,
    Floor,
    Ceiling,
    ZeroFiveUp,
}

fn rounding_from_str(s: &str) -> Option<Rounding> {
    Some(match s {
        "ROUND_DOWN" => Rounding::Down,
        "ROUND_UP" => Rounding::Up,
        "ROUND_HALF_UP" => Rounding::HalfUp,
        "ROUND_HALF_DOWN" => Rounding::HalfDown,
        "ROUND_HALF_EVEN" => Rounding::HalfEven,
        "ROUND_FLOOR" => Rounding::Floor,
        "ROUND_CEILING" => Rounding::Ceiling,
        "ROUND_05UP" => Rounding::ZeroFiveUp,
        _ => return None,
    })
}

/// Round `|c|` by dropping `drop` trailing digits under `mode`, where
/// `neg` is the sign of the full value (modes FLOOR/CEILING need it).
fn round_abs_with_mode(c_abs: &BigInt, drop: u32, mode: Rounding, neg: bool) -> BigInt {
    if drop == 0 {
        return c_abs.clone();
    }
    let p = pow10(drop);
    let q = c_abs / &p;
    let r = c_abs % &p;
    if r.is_zero() {
        return q;
    }
    let twice = &r * 2u32;
    let round_up = match mode {
        Rounding::Down => false,
        Rounding::Up => true,
        Rounding::HalfUp => twice >= p,
        Rounding::HalfDown => twice > p,
        Rounding::HalfEven => twice > p || (twice == p && !(&q % 2u32).is_zero()),
        Rounding::Floor => neg,
        Rounding::Ceiling => !neg,
        Rounding::ZeroFiveUp => {
            let last = (&q % 10u32).to_u32().unwrap_or(0);
            last == 0 || last == 5
        }
    };
    if round_up {
        q + 1u32
    } else {
        q
    }
}

/// Build a finite `MbDecimal` from an exact signed coefficient and scale
/// (value = c * 10^-s), rounding half-even to 28 significant digits and
/// clamping into rust_decimal's representable window. Returns `Err(())`
/// only when the magnitude is too large to represent after rounding.
fn finite_from_coeff_scale(c: BigInt, s: i64) -> Result<MbDecimal, ()> {
    let neg = c.is_negative();
    let mut c_abs: BigInt = c.magnitude().to_owned().into();
    let mut s = s;
    // Negative scale → fold the zeros into the coefficient.
    if s < 0 {
        c_abs = &c_abs * pow10((-s) as u32);
        s = 0;
    }
    // Round to context precision.
    let digits = digit_count(&c_abs);
    if digits > PREC {
        let drop = (digits - PREC) as i64;
        if drop > s {
            // Rounding would push the exponent above 0 — magnitude beyond
            // rust_decimal's window (>= 1e28 integer part).
            return Err(());
        }
        c_abs = round_half_even_abs(&c_abs, drop as u32);
        s -= drop;
    }
    // Clamp the scale into rust_decimal's window (values below ~1e-28
    // collapse toward zero — documented carve-out).
    if s > MAX_SCALE {
        let drop = (s - MAX_SCALE) as u32;
        c_abs = round_half_even_abs(&c_abs, drop);
        s = MAX_SCALE;
    }
    // Rounding may have produced a 29-digit power of ten; re-check.
    if digit_count(&c_abs) > PREC {
        if s >= 1 {
            c_abs = round_half_even_abs(&c_abs, 1);
            s -= 1;
        } else {
            return Err(());
        }
    }
    let mag: BigInt = c_abs;
    let Some(i) = mag.to_i128() else {
        return Err(());
    };
    let signed = if neg { -i } else { i };
    match Decimal::try_from_i128_with_scale(signed, s as u32) {
        Ok(mut d) => {
            if neg && d.is_zero() {
                d.set_sign_negative(true);
            }
            Ok(MbDecimal::finite(d))
        }
        Err(_) => Err(()),
    }
}

/// Decompose a finite payload into (signed coefficient, scale).
fn coeff_scale(d: &Decimal) -> (BigInt, i64) {
    (BigInt::from(d.mantissa()), d.scale() as i64)
}

fn state_coeff_scale(st: &MbDecimal) -> (BigInt, i64) {
    if let Some(idx) = st.exact {
        exact_value(idx)
    } else {
        coeff_scale(&st.value)
    }
}

fn finite_from_coeff_scale_preserve(c: BigInt, s: i64) -> Option<MbDecimal> {
    if s >= 0 {
        return finite_from_coeff_scale(c, s).ok();
    }
    let value_coeff = &c * pow10((-s) as u32);
    let mut st = finite_from_coeff_scale(value_coeff, 0).ok()?;
    st.exact = Some(intern_exact(c, s));
    Some(st)
}

// ── Operand classification (shared with the builtins comparison hooks) ──

/// Numeric operand normal form for exact comparison.
enum NumOperand {
    Rational(BigInt, BigInt), // num / den, den > 0
    Inf(bool),                // negative?
    Nan,
}

/// Exact rational expansion of a finite f64 (mantissa × 2^exp).
fn f64_to_rational(f: f64) -> (BigInt, BigInt) {
    let bits = f.to_bits();
    let sign = (bits >> 63) & 1 == 1;
    let exp_field = ((bits >> 52) & 0x7FF) as i64;
    let frac = bits & 0x000F_FFFF_FFFF_FFFF;
    let (mant, e) = if exp_field == 0 {
        (frac, -1074i64)
    } else {
        (frac | (1u64 << 52), exp_field - 1075)
    };
    let mut num = BigInt::from(mant);
    let mut den = BigInt::from(1u32);
    if e >= 0 {
        num <<= e as usize;
    } else {
        den <<= (-e) as usize;
    }
    if sign {
        num = -num;
    }
    (num, den)
}

/// Exact terminating-decimal expansion of a finite f64, returned as
/// `(signed_coeff, scale)` so the value equals `signed_coeff * 10^-scale`.
///
/// The f64 is `mant * 2^e`; with `den = 2^k` (k ≥ 0) the value is
/// `num / 2^k = num * 5^k / 10^k`, an exact terminating decimal with
/// coefficient `num * 5^k` and scale `k`. Trailing zeros are stripped while
/// the scale stays ≥ 0, matching CPython's canonical `Decimal(float)` form
/// (so `Decimal(10.0)` is `10` with scale 0, while `Decimal(0.1)` keeps its
/// full 55-digit expansion).
fn f64_to_exact_decimal(f: f64) -> (BigInt, i64) {
    let (num, den) = f64_to_rational(f);
    // `den` is exactly a power of two: 2^k.
    let k = (den.bits() as i64) - 1; // bits() of 2^k is k+1; for den==1, k==0.
    let mut coeff = num;
    let mut scale = 0i64;
    if k > 0 {
        coeff *= pow5(k as u32);
        scale = k;
    }
    // Strip trailing zeros while keeping the scale non-negative.
    let ten = BigInt::from(10u32);
    while scale > 0 && (&coeff % &ten).is_zero() {
        coeff /= &ten;
        scale -= 1;
    }
    (coeff, scale)
}

/// `5^n` as BigInt.
fn pow5(n: u32) -> BigInt {
    let mut p = BigInt::from(1u32);
    let five = BigInt::from(5u32);
    for _ in 0..n {
        p *= &five;
    }
    p
}

/// Intern an exact `(signed_coeff, scale)` pair and return its index for
/// `MbDecimal::exact`.
fn intern_exact(coeff: BigInt, scale: i64) -> usize {
    EXACT_VALUES.with(|v| {
        let mut v = v.borrow_mut();
        v.push((coeff, scale));
        v.len() - 1
    })
}

/// Read back an interned exact `(signed_coeff, scale)` pair.
fn exact_value(idx: usize) -> (BigInt, i64) {
    EXACT_VALUES.with(|v| v.borrow()[idx].clone())
}

/// Classify an MbValue as a numeric operand for exact comparison.
/// Handles Decimal handles, Fraction handles, bool/int/BigInt, float.
/// Complex with zero imaginary part participates via its real float.
fn classify_numeric(v: MbValue) -> Option<NumOperand> {
    if let Some(st) = get_state(v) {
        return Some(match st.class {
            DecClass::Finite => {
                // Exact float payloads compare on their full expansion so
                // `Decimal(0.1) != Decimal('0.1')` and equals the exact string.
                let (mut c, s) = if let Some(idx) = st.exact {
                    exact_value(idx)
                } else {
                    coeff_scale(&st.value)
                };
                let den = if s < 0 {
                    c *= pow10((-s) as u32);
                    BigInt::from(1u32)
                } else {
                    pow10(s as u32)
                };
                NumOperand::Rational(c, den)
            }
            DecClass::Inf => NumOperand::Inf(st.neg),
            DecClass::QNan | DecClass::SNan => NumOperand::Nan,
        });
    }
    if let Some(id) = v.as_int() {
        let idu = id as u64;
        if idu >= super::super::integer_handle_registry::HANDLE_MIN_ID {
            if let Some((n, d)) = super::fractions_mod::handle_num_den(idu) {
                return Some(NumOperand::Rational(BigInt::from(n), BigInt::from(d)));
            }
        }
    }
    if let Some(b) = v.as_bool() {
        return Some(NumOperand::Rational(
            BigInt::from(b as i64),
            BigInt::from(1u32),
        ));
    }
    if let Some(i) = v.as_int() {
        return Some(NumOperand::Rational(BigInt::from(i), BigInt::from(1u32)));
    }
    if let Some(f) = v.as_float() {
        if f.is_nan() {
            return Some(NumOperand::Nan);
        }
        if f.is_infinite() {
            return Some(NumOperand::Inf(f < 0.0));
        }
        let (n, d) = f64_to_rational(f);
        return Some(NumOperand::Rational(n, d));
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::BigInt(b) => {
                    return Some(NumOperand::Rational(b.clone(), BigInt::from(1u32)));
                }
                ObjData::Complex(re, im) => {
                    if *im == 0.0 && re.is_finite() {
                        let (n, d) = f64_to_rational(*re);
                        return Some(NumOperand::Rational(n, d));
                    }
                    return None;
                }
                _ => {}
            }
        }
    }
    None
}

fn cmp_operands(a: &NumOperand, b: &NumOperand) -> Option<std::cmp::Ordering> {
    use std::cmp::Ordering;
    match (a, b) {
        (NumOperand::Nan, _) | (_, NumOperand::Nan) => None,
        (NumOperand::Inf(an), NumOperand::Inf(bn)) => Some(if an == bn {
            Ordering::Equal
        } else if *an {
            Ordering::Less
        } else {
            Ordering::Greater
        }),
        (NumOperand::Inf(an), _) => Some(if *an {
            Ordering::Less
        } else {
            Ordering::Greater
        }),
        (_, NumOperand::Inf(bn)) => Some(if *bn {
            Ordering::Greater
        } else {
            Ordering::Less
        }),
        (NumOperand::Rational(an, ad), NumOperand::Rational(bn, bd)) => {
            Some((an * bd).cmp(&(bn * ad)))
        }
    }
}

/// Shared `==` hook for Decimal/Fraction handles in `mb_values_eq`.
/// Returns `None` when the other operand is not numeric (caller treats
/// the pair as unequal, matching CPython's `False`).
pub fn mb_numeric_handle_eq(a: MbValue, b: MbValue) -> Option<bool> {
    let oa = classify_numeric(a)?;
    let ob = classify_numeric(b)?;
    match (&oa, &ob) {
        (NumOperand::Nan, _) | (_, NumOperand::Nan) => Some(false),
        _ => Some(cmp_operands(&oa, &ob) == Some(std::cmp::Ordering::Equal)),
    }
}

/// Shared `<` hook for Decimal/Fraction handles in `mb_values_lt`.
/// Returns `None` for non-numeric operands; raises `InvalidOperation`
/// (and yields `false`) when NaN participates in an ordering, matching
/// CPython.
pub fn mb_numeric_handle_lt(a: MbValue, b: MbValue) -> Option<bool> {
    let oa = classify_numeric(a)?;
    let ob = classify_numeric(b)?;
    if matches!(oa, NumOperand::Nan) || matches!(ob, NumOperand::Nan) {
        raise_invalid_operation("comparison involving NaN");
        return Some(false);
    }
    Some(cmp_operands(&oa, &ob) == Some(std::cmp::Ordering::Less))
}

/// Integral-and-fits-i64 readback used by the `hash()` hook so that
/// `hash(Decimal('2'))`/`hash(Fraction(2,1))` match `hash(2)`.
pub fn mb_numeric_handle_integral_i64(v: MbValue) -> Option<i64> {
    match classify_numeric(v)? {
        NumOperand::Rational(n, d) => {
            let (q, r) = (&n / &d, &n % &d);
            if r.is_zero() {
                q.to_i64()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Exact f64 readback: returns `Some(f)` when the handle's value is
/// exactly representable as that f64 (so the `hash()` hook can delegate
/// to the float hash and stay consistent with `==`).
pub fn mb_numeric_handle_exact_f64(v: MbValue) -> Option<f64> {
    let st = get_state(v);
    let approx: f64 = if let Some(st) = st {
        match st.class {
            DecClass::Finite => st.value.to_f64()?,
            DecClass::Inf => {
                return Some(if st.neg {
                    f64::NEG_INFINITY
                } else {
                    f64::INFINITY
                });
            }
            _ => return None,
        }
    } else if let Some(id) = v.as_int() {
        let (n, d) = super::fractions_mod::handle_num_den(id as u64)?;
        n as f64 / d as f64
    } else {
        return None;
    };
    if !approx.is_finite() {
        return None;
    }
    let (fa, fd) = f64_to_rational(approx);
    match classify_numeric(v)? {
        NumOperand::Rational(n, d) => {
            if n * fd == fa * d {
                Some(approx)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Decimal handle → exact (numerator, denominator) i64 pair when both
/// fit; used by `fractions.Fraction(Decimal(...))` construction.
pub fn decimal_to_num_den(v: MbValue) -> Option<(i64, i64)> {
    let st = get_state(v)?;
    if st.class != DecClass::Finite {
        return None;
    }
    let (c, s) = coeff_scale(&st.value);
    let den = pow10(s as u32);
    let g = gcd_big(&c, &den);
    let n = (&c / &g).to_i64()?;
    let d = (&den / &g).to_i64()?;
    Some((n, d))
}

fn gcd_big(a: &BigInt, b: &BigInt) -> BigInt {
    let mut a = a.magnitude().to_owned();
    let mut b = b.magnitude().to_owned();
    while !b.is_zero() {
        let t = b.clone();
        b = &a % &b;
        a = t;
    }
    BigInt::from(a)
}

// ── Arithmetic operand coercion ──────────────────────────────────────────

/// Result of coercing an arithmetic operand: either a Decimal state or
/// the Python type name of the unsupported operand (for the TypeError).
fn arith_operand(v: MbValue) -> Result<MbDecimal, &'static str> {
    if let Some(st) = get_state(v) {
        return Ok(st);
    }
    if let Some(id) = v.as_int() {
        let idu = id as u64;
        if idu >= super::super::integer_handle_registry::HANDLE_MIN_ID
            && super::fractions_mod::is_fraction_handle(idu)
        {
            return Err("fractions.Fraction");
        }
        // Plain int (bool handled below by as_int_pyint? as_int excludes bool)
        return Ok(MbDecimal::finite(Decimal::from(id)));
    }
    if let Some(b) = v.as_bool() {
        return Ok(MbDecimal::finite(Decimal::from(b as i64)));
    }
    if v.is_float() {
        return Err("float");
    }
    if v.is_none() {
        return Err("NoneType");
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::BigInt(b) => {
                    return finite_from_coeff_scale(b.clone(), 0).map_err(|_| "int");
                }
                ObjData::Complex(..) => return Err("complex"),
                ObjData::Str(_) => return Err("str"),
                ObjData::List(_) => return Err("list"),
                ObjData::Tuple(_) => return Err("tuple"),
                ObjData::Dict(_) => return Err("dict"),
                _ => return Err("object"),
            }
        }
    }
    Err("object")
}

fn raise_unsupported_operand(op: &str, other: &str) -> MbValue {
    raise_type_error(&format!(
        "unsupported operand type(s) for {op}: 'decimal.Decimal' and '{other}'"
    ))
}

/// Coerce both operands for a binary arithmetic op; raises TypeError and
/// returns `None` if either side is unsupported.
fn arith_pair(op: &str, a: MbValue, b: MbValue) -> Option<(MbDecimal, MbDecimal)> {
    let lhs = match arith_operand(a) {
        Ok(d) => d,
        Err(name) => {
            raise_unsupported_operand(op, name);
            return None;
        }
    };
    let rhs = match arith_operand(b) {
        Ok(d) => d,
        Err(name) => {
            raise_unsupported_operand(op, name);
            return None;
        }
    };
    if lhs.class == DecClass::SNan || rhs.class == DecClass::SNan {
        raise_invalid_operation("sNaN operand");
        return None;
    }
    Some((lhs, rhs))
}

fn overflow_result() -> MbValue {
    raise_overflow("above Emax")
}

// ── Public arithmetic surface ────────────────────────────────────────────

/// `a + b` — exact addition, rounded to 28 significant digits.
pub fn mb_decimal_add(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("+", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    match (lhs.class, rhs.class) {
        (DecClass::Inf, DecClass::Inf) => {
            if lhs.neg == rhs.neg {
                make_state_handle(MbDecimal::inf(lhs.neg))
            } else {
                raise_invalid_operation("-INF + INF")
            }
        }
        (DecClass::Inf, _) => make_state_handle(MbDecimal::inf(lhs.neg)),
        (_, DecClass::Inf) => make_state_handle(MbDecimal::inf(rhs.neg)),
        _ => {
            let (ca, sa) = coeff_scale(&lhs.value);
            let (cb, sb) = coeff_scale(&rhs.value);
            let s = sa.max(sb);
            let ca = ca * pow10((s - sa) as u32);
            let cb = cb * pow10((s - sb) as u32);
            match finite_from_coeff_scale(ca + cb, s) {
                Ok(st) => make_state_handle(st),
                Err(()) => overflow_result(),
            }
        }
    }
}

/// `a - b`.
pub fn mb_decimal_sub(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("-", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    match (lhs.class, rhs.class) {
        (DecClass::Inf, DecClass::Inf) => {
            if lhs.neg != rhs.neg {
                make_state_handle(MbDecimal::inf(lhs.neg))
            } else {
                raise_invalid_operation("INF - INF")
            }
        }
        (DecClass::Inf, _) => make_state_handle(MbDecimal::inf(lhs.neg)),
        (_, DecClass::Inf) => make_state_handle(MbDecimal::inf(!rhs.neg)),
        _ => {
            let (ca, sa) = coeff_scale(&lhs.value);
            let (cb, sb) = coeff_scale(&rhs.value);
            let s = sa.max(sb);
            let ca = ca * pow10((s - sa) as u32);
            let cb = cb * pow10((s - sb) as u32);
            match finite_from_coeff_scale(ca - cb, s) {
                Ok(st) => make_state_handle(st),
                Err(()) => overflow_result(),
            }
        }
    }
}

/// `a * b`.
pub fn mb_decimal_mul(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("*", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    let sign = lhs_sign(&lhs) != lhs_sign(&rhs);
    match (lhs.class, rhs.class) {
        (DecClass::Inf, _) | (_, DecClass::Inf) => {
            let other_zero = (lhs.class == DecClass::Finite && lhs.value.is_zero())
                || (rhs.class == DecClass::Finite && rhs.value.is_zero());
            if other_zero {
                raise_invalid_operation("0 * INF")
            } else {
                make_state_handle(MbDecimal::inf(sign))
            }
        }
        _ => {
            let (ca, sa) = coeff_scale(&lhs.value);
            let (cb, sb) = coeff_scale(&rhs.value);
            match finite_from_coeff_scale(ca * cb, sa + sb) {
                Ok(mut st) => {
                    if st.value.is_zero() {
                        st.value.set_sign_negative(sign);
                        st.neg = sign;
                    }
                    make_state_handle(st)
                }
                Err(()) => overflow_result(),
            }
        }
    }
}

fn lhs_sign(d: &MbDecimal) -> bool {
    match d.class {
        DecClass::Finite => d.value.is_sign_negative(),
        _ => d.neg,
    }
}

/// Exact division core on (coeff, scale) pairs implementing the spec's
/// divide: exact results take the exponent closest to the ideal exponent
/// (ea - eb); inexact results round half-even to 28 significant digits.
fn divide_finite(lhs: &Decimal, rhs: &Decimal) -> Result<MbDecimal, ()> {
    let (ca, sa) = coeff_scale(lhs);
    let (cb, sb) = coeff_scale(rhs);
    let neg = (ca.is_negative()) != (cb.is_negative());
    let ca_abs: BigInt = ca.magnitude().to_owned().into();
    let cb_abs: BigInt = cb.magnitude().to_owned().into();
    let ideal_exp = sb - sa; // in scale terms: ideal scale = sa - sb
    let ideal_scale = -ideal_exp;
    if ca_abs.is_zero() {
        // 0 / x → 0 with the ideal scale (clamped).
        let s = ideal_scale.clamp(0, MAX_SCALE);
        let mut st = finite_from_coeff_scale(BigInt::from(0u32), s)?;
        if neg {
            st.value.set_sign_negative(true);
            st.neg = true;
        }
        return Ok(st);
    }
    // Scale the dividend so the integer quotient has at least PREC+1 digits.
    let da = digit_count(&ca_abs) as i64;
    let db = digit_count(&cb_abs) as i64;
    let k = (db + PREC as i64 + 1 - da).max(0);
    let scaled = &ca_abs * pow10(k as u32);
    let mut q = &scaled / &cb_abs;
    let r = &scaled % &cb_abs;
    // exponent of q (power-of-ten terms): exp = -sa - k + sb
    if r.is_zero() {
        // Exact: bring the exponent as close to ideal as possible.
        let mut exp = -sa - k + sb;
        // Remove trailing zeros while below the ideal exponent.
        while exp < ideal_exp && (&q % 10u32).is_zero() {
            q /= 10u32;
            exp += 1;
        }
        // Pad toward the ideal exponent while staying within precision.
        while exp > ideal_exp && digit_count(&q) < PREC {
            q *= 10u32;
            exp -= 1;
        }
        let signed = if neg { -q } else { q };
        return finite_from_coeff_scale(signed, -exp);
    }
    // Inexact: round half-even at PREC digits. The live remainder `r` is
    // nonzero (sticky), so an exact midpoint cannot occur: dropped digits
    // at exactly half round up, anything below half stays down.
    let extra = (digit_count(&q) as i64 - PREC as i64).max(0) as u32;
    let mut exp = -sa - k + sb + extra as i64;
    let mut rounded = if extra > 0 {
        let p = pow10(extra);
        let qq = &q / &p;
        let rem = &q % &p;
        let twice = &rem * 2u32;
        if twice >= p {
            qq + 1u32
        } else {
            qq
        }
    } else {
        q
    };
    // Re-normalize if rounding produced PREC+1 digits (999..9 → 1000..0).
    if digit_count(&rounded) > PREC {
        rounded = round_half_even_abs(&rounded, 1);
        exp += 1;
    }
    let signed = if neg { -rounded } else { rounded };
    finite_from_coeff_scale(signed, -exp)
}

/// `a / b`. Raises `DivisionByZero` on zero divisor (CPython default
/// context traps it).
pub fn mb_decimal_truediv(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("/", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    let sign = lhs_sign(&lhs) != lhs_sign(&rhs);
    match (lhs.class, rhs.class) {
        (DecClass::Inf, DecClass::Inf) => raise_invalid_operation("INF / INF"),
        (DecClass::Inf, _) => make_state_handle(MbDecimal::inf(sign)),
        (_, DecClass::Inf) => make_state_handle(MbDecimal::finite(Decimal::ZERO)),
        _ => {
            if rhs.value.is_zero() {
                if lhs.value.is_zero() {
                    return raise_invalid_operation("0 / 0");
                }
                return raise_division_by_zero("x / 0");
            }
            match divide_finite(&lhs.value, &rhs.value) {
                Ok(st) => make_state_handle(st),
                Err(()) => overflow_result(),
            }
        }
    }
}

/// Truncating integer division and remainder on finite payloads —
/// `decimal`'s `//` and `%` truncate toward zero (remainder keeps the
/// dividend's sign), unlike Python int floor semantics.
fn divmod_finite(lhs: &Decimal, rhs: &Decimal) -> Option<(MbDecimal, MbDecimal)> {
    let (ca, sa) = coeff_scale(lhs);
    let (cb, sb) = coeff_scale(rhs);
    let s = sa.max(sb);
    let ca = ca * pow10((s - sa) as u32);
    let cb = cb * pow10((s - sb) as u32);
    let q = &ca / &cb; // BigInt division truncates toward zero
    let r = &ca % &cb;
    let q_st = finite_from_coeff_scale(q, 0).ok()?;
    let r_st = finite_from_coeff_scale(r, s).ok()?;
    Some((q_st, r_st))
}

/// `a // b` — truncating division to an integral Decimal.
pub fn mb_decimal_floordiv(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("//", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    match (lhs.class, rhs.class) {
        (DecClass::Inf, DecClass::Inf) => raise_invalid_operation("INF // INF"),
        (DecClass::Inf, _) => make_state_handle(MbDecimal::inf(lhs.neg != lhs_sign(&rhs))),
        (_, DecClass::Inf) => make_state_handle(MbDecimal::finite(Decimal::ZERO)),
        _ => {
            if rhs.value.is_zero() {
                return raise_division_by_zero("x // 0");
            }
            match divmod_finite(&lhs.value, &rhs.value) {
                Some((q, _)) => make_state_handle(q),
                None => overflow_result(),
            }
        }
    }
}

/// `a % b` — remainder with the dividend's sign.
pub fn mb_decimal_rem(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("%", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    match (lhs.class, rhs.class) {
        (DecClass::Inf, _) => raise_invalid_operation("INF % x"),
        (_, DecClass::Inf) => make_state_handle(lhs),
        _ => {
            if rhs.value.is_zero() {
                return raise_invalid_operation("x % 0");
            }
            match divmod_finite(&lhs.value, &rhs.value) {
                Some((_, r)) => make_state_handle(r),
                None => overflow_result(),
            }
        }
    }
}

/// `divmod(a, b)` — `(a // b, a % b)` as a tuple.
pub fn mb_decimal_divmod(a: MbValue, b: MbValue) -> MbValue {
    let q = mb_decimal_floordiv(a, b);
    if super::super::exception::mb_has_exception().as_bool() == Some(true) {
        return MbValue::none();
    }
    let r = mb_decimal_rem(a, b);
    MbValue::from_ptr(MbObject::new_tuple(vec![q, r]))
}

/// `a ** b` — integral exponents only (fixture-covered slice).
pub fn mb_decimal_pow(a: MbValue, b: MbValue) -> MbValue {
    let Some((lhs, rhs)) = arith_pair("**", a, b) else {
        return MbValue::none();
    };
    if lhs.is_nan() || rhs.is_nan() {
        return make_state_handle(MbDecimal::qnan());
    }
    if lhs.class != DecClass::Finite || rhs.class != DecClass::Finite {
        return raise_invalid_operation("power with infinite operand");
    }
    // The exponent must be integral.
    let (cb, sb) = coeff_scale(&rhs.value);
    let den = pow10(sb as u32);
    if !(&cb % &den).is_zero() {
        return raise_invalid_operation("x ** (non-integer)");
    }
    let n = match (&cb / &den).to_i64() {
        Some(n) => n,
        None => return raise_invalid_operation("exponent too large"),
    };
    if n == 0 {
        return make_handle(Decimal::ONE);
    }
    let (ca, sa) = coeff_scale(&lhs.value);
    if ca.is_zero() && n < 0 {
        return raise_division_by_zero("0 ** negative");
    }
    let abs_n = n.unsigned_abs();
    // Guard against runaway magnitude: estimate result digit count.
    let est_digits = (digit_count(&ca) as u64).saturating_mul(abs_n);
    if est_digits > 50_000 {
        return raise_invalid_operation("exponent too large for this build");
    }
    let mut coeff = BigInt::from(1u32);
    for _ in 0..abs_n {
        coeff *= &ca;
    }
    let scale = sa.saturating_mul(abs_n as i64);
    if n > 0 {
        match finite_from_coeff_scale(coeff, scale) {
            Ok(st) => make_state_handle(st),
            Err(()) => overflow_result(),
        }
    } else {
        // 1 / (lhs ** |n|) via the exact division core.
        let base = match finite_from_coeff_scale(coeff, scale) {
            Ok(st) => st,
            Err(()) => return overflow_result(),
        };
        match divide_finite(&Decimal::ONE, &base.value) {
            Ok(st) => make_state_handle(st),
            Err(()) => overflow_result(),
        }
    }
}

/// Unary `-` — CPython's minus applies the context, which normalizes
/// `-0` to `0`.
pub fn mb_decimal_neg(v: MbValue) -> MbValue {
    let Some(st) = get_state(v) else {
        return MbValue::none();
    };
    match st.class {
        DecClass::Finite => {
            let mut d = st.value;
            if d.is_zero() {
                d.set_sign_negative(false);
            } else {
                d.set_sign_negative(!d.is_sign_negative());
            }
            make_handle(d)
        }
        DecClass::Inf => make_state_handle(MbDecimal::inf(!st.neg)),
        _ => make_state_handle(MbDecimal::qnan()),
    }
}

/// Unary `+` — identity through the context (normalizes `-0` to `0`).
pub fn mb_decimal_pos(v: MbValue) -> MbValue {
    let Some(st) = get_state(v) else {
        return MbValue::none();
    };
    match st.class {
        DecClass::Finite => {
            let mut d = st.value;
            if d.is_zero() {
                d.set_sign_negative(false);
            }
            make_handle(d)
        }
        _ => make_state_handle(st),
    }
}

/// `abs(d)`.
pub fn mb_decimal_abs(v: MbValue) -> MbValue {
    let Some(st) = get_state(v) else {
        return MbValue::none();
    };
    match st.class {
        DecClass::Finite => make_handle(st.value.abs()),
        DecClass::Inf => make_state_handle(MbDecimal::inf(false)),
        _ => make_state_handle(MbDecimal::qnan()),
    }
}

// ── Conversion / readback ────────────────────────────────────────────────

/// Spec `to-sci-string` rendering from (neg, coefficient digits, exponent).
fn to_sci_string(neg: bool, digits: &str, exp: i64) -> String {
    let n = digits.len() as i64;
    let adjusted = exp + n - 1;
    let sign = if neg { "-" } else { "" };
    if exp <= 0 && adjusted >= -6 {
        // Plain notation.
        if exp == 0 {
            return format!("{sign}{digits}");
        }
        let point = n + exp; // digits before the decimal point
        if point > 0 {
            let (int_part, frac_part) = digits.split_at(point as usize);
            return format!("{sign}{int_part}.{frac_part}");
        }
        let zeros = "0".repeat((-point) as usize);
        return format!("{sign}0.{zeros}{digits}");
    }
    // Scientific notation with adjusted exponent.
    let exp_str = if adjusted >= 0 {
        format!("E+{adjusted}")
    } else {
        format!("E{adjusted}")
    };
    if digits.len() == 1 {
        format!("{sign}{digits}{exp_str}")
    } else {
        let (first, rest) = digits.split_at(1);
        format!("{sign}{first}.{rest}{exp_str}")
    }
}

fn state_to_string(st: &MbDecimal) -> String {
    match st.class {
        DecClass::Inf => {
            if st.neg {
                "-Infinity".to_string()
            } else {
                "Infinity".to_string()
            }
        }
        DecClass::QNan => {
            if st.neg {
                "-NaN".to_string()
            } else {
                "NaN".to_string()
            }
        }
        DecClass::SNan => {
            if st.neg {
                "-sNaN".to_string()
            } else {
                "sNaN".to_string()
            }
        }
        DecClass::Finite => {
            let d = &st.value;
            // Exact float-expansion payload: render from the interned
            // coefficient/scale, which carries digits beyond rust_decimal's
            // window (e.g. `Decimal(0.1)`'s full 55-digit expansion).
            if let Some(idx) = st.exact {
                let (coeff, scale) = exact_value(idx);
                let neg = st.neg || coeff.is_negative();
                let digits = coeff.magnitude().to_string();
                return to_sci_string(neg, &digits, -scale);
            }
            let neg = d.is_sign_negative();
            if d.is_zero() {
                if let Some(exp) = st.tuple_exp {
                    return to_sci_string(st.neg || neg, "0", exp);
                }
            }
            let digits = d.mantissa().unsigned_abs().to_string();
            to_sci_string(neg, &digits, -(d.scale() as i64))
        }
    }
}

/// `str(d)` — the canonical decimal string.
pub fn mb_decimal_str(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_ptr(MbObject::new_str("0".to_string()));
    };
    MbValue::from_ptr(MbObject::new_str(state_to_string(&st)))
}

/// `repr(d)` — `Decimal('...')`.
pub fn mb_decimal_repr(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_ptr(MbObject::new_str("Decimal('0')".to_string()));
    };
    MbValue::from_ptr(MbObject::new_str(format!(
        "Decimal('{}')",
        state_to_string(&st)
    )))
}

/// `bool(d)` — False only for (signed) zero.
pub fn mb_decimal_bool(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_bool(false);
    };
    MbValue::from_bool(match st.class {
        DecClass::Finite => !st.value.is_zero(),
        _ => true,
    })
}

/// `int(d)` / `math.trunc(d)` — truncate toward zero. Raises for NaN /
/// Infinity like CPython.
pub fn mb_decimal_int(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_int(0);
    };
    match st.class {
        DecClass::QNan | DecClass::SNan => raise_value_error("cannot convert NaN to integer"),
        DecClass::Inf => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "cannot convert Infinity to integer".to_string(),
                )),
            );
            MbValue::none()
        }
        DecClass::Finite => {
            let (c, s) = coeff_scale(&st.value);
            let q = &c / pow10(s as u32);
            big_to_mb_int(q)
        }
    }
}

fn big_to_mb_int(b: BigInt) -> MbValue {
    if let Some(i) = b.to_i64() {
        if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
            return MbValue::from_int(i);
        }
    }
    MbValue::from_ptr(MbObject::new_bigint(b))
}

/// `math.floor(d)` — exact floor to an int.
pub fn mb_decimal_floor(d: MbValue) -> MbValue {
    decimal_floor_ceil(d, true)
}

/// `math.ceil(d)` — exact ceiling to an int.
pub fn mb_decimal_ceil(d: MbValue) -> MbValue {
    decimal_floor_ceil(d, false)
}

fn decimal_floor_ceil(d: MbValue, floor: bool) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_int(0);
    };
    match st.class {
        DecClass::QNan | DecClass::SNan => raise_value_error("cannot convert NaN to integer"),
        DecClass::Inf => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "cannot convert Infinity to integer".to_string(),
                )),
            );
            MbValue::none()
        }
        DecClass::Finite => {
            let (c, s) = coeff_scale(&st.value);
            let den = pow10(s as u32);
            let q = &c / &den;
            let r = &c % &den;
            let adjust = if r.is_zero() {
                BigInt::from(0u32)
            } else if floor && c.is_negative() {
                BigInt::from(-1)
            } else if !floor && !c.is_negative() {
                BigInt::from(1)
            } else {
                BigInt::from(0u32)
            };
            big_to_mb_int(q + adjust)
        }
    }
}

/// `float(d)`.
pub fn mb_decimal_float(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_float(0.0);
    };
    MbValue::from_float(match st.class {
        DecClass::Finite => st.value.to_f64().unwrap_or(0.0),
        DecClass::Inf => {
            if st.neg {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        }
        _ => f64::NAN,
    })
}

/// `round(d)` / `round(d, n)` — banker's rounding; no-ndigits form
/// returns an int, ndigits form returns a Decimal.
pub fn mb_decimal_round(d: MbValue, ndigits: MbValue, ndigits_given: bool) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_int(0);
    };
    if st.is_nan() {
        if !ndigits_given {
            return raise_value_error("cannot round a NaN");
        }
        return make_state_handle(MbDecimal::qnan());
    }
    if st.class == DecClass::Inf {
        if !ndigits_given {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                MbValue::from_ptr(MbObject::new_str("cannot round an infinity".to_string())),
            );
            return MbValue::none();
        }
        return make_state_handle(st);
    }
    let (c, s) = coeff_scale(&st.value);
    if !ndigits_given {
        // → int, half-even at the units place.
        let neg = c.is_negative();
        let c_abs: BigInt = c.magnitude().to_owned().into();
        let q = round_half_even_abs(&c_abs, s as u32);
        return big_to_mb_int(if neg { -q } else { q });
    }
    let n = match ndigits.as_int() {
        Some(n) => n,
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "'str' object cannot be interpreted as an integer".to_string(),
                )),
            );
            return MbValue::none();
        }
    };
    quantize_to_scale(&st, n.min(MAX_SCALE).max(-64), Rounding::HalfEven)
        .map(make_state_handle)
        .unwrap_or_else(|| make_handle(Decimal::ZERO))
}

/// Rescale a finite state to `target_scale` under `mode`. Returns None
/// when the result cannot be represented.
fn quantize_to_scale(st: &MbDecimal, target_scale: i64, mode: Rounding) -> Option<MbDecimal> {
    let (c, s) = coeff_scale(&st.value);
    let neg = c.is_negative() || (st.value.is_zero() && st.value.is_sign_negative());
    let c_abs: BigInt = c.magnitude().to_owned().into();
    let rounded_abs = if target_scale >= s {
        // Scale up — exact (pad with zeros).
        &c_abs * pow10((target_scale - s) as u32)
    } else {
        round_abs_with_mode(&c_abs, (s - target_scale) as u32, mode, neg)
    };
    if digit_count(&rounded_abs) > PREC {
        return None;
    }
    if target_scale < 0 {
        let signed = if neg { -rounded_abs } else { rounded_abs };
        let mut s2 = finite_from_coeff_scale_preserve(signed, target_scale)?;
        if neg && s2.value.is_zero() {
            s2.value.set_sign_negative(true);
            s2.neg = true;
            s2.tuple_exp = Some(-target_scale);
        }
        if s2.value.is_zero() && target_scale < 0 {
            s2.tuple_exp = Some(-target_scale);
        }
        return Some(s2);
    }
    let clamped = target_scale.clamp(0, MAX_SCALE);
    let adjusted_abs = if clamped > target_scale {
        // Target exponent above 0 — only representable when the digits
        // carry the difference as trailing zeros (e.g. round(x, -10) on a
        // small value collapses to zero).
        &rounded_abs * pow10((clamped - target_scale) as u32)
    } else {
        rounded_abs
    };
    if digit_count(&adjusted_abs) > PREC {
        return None;
    }
    let signed = if neg { -adjusted_abs } else { adjusted_abs };
    finite_from_coeff_scale(signed, clamped).ok().map(|mut s2| {
        if neg && s2.value.is_zero() {
            s2.value.set_sign_negative(true);
            s2.neg = true;
        }
        s2
    })
}

// ── Validation helpers for the tuple constructor ─────────────────────────

/// Validate a CPython argument-tuple form `(sign, (d0, d1, ...), exponent)`.
/// Returns an error message string when a coefficient digit is out of the
/// `0..=9` range (CPython raises `ValueError`), or the sign is not 0/1.
fn validate_decimal_tuple(items: &[MbValue]) -> Result<(), String> {
    if items.len() != 3 {
        return Err("Invalid tuple size in creation of Decimal from list or tuple.".to_string());
    }
    // sign must be 0 or 1
    match items[0].as_int_pyint() {
        Some(0) | Some(1) => {}
        _ => {
            return Err(
                "Invalid sign.  The first value in the tuple should be an integer; either 0 for a positive number or 1 for a negative number."
                    .to_string(),
            );
        }
    }
    // digits tuple
    if let Some(ptr) = items[1].as_ptr() {
        let digits: Option<Vec<MbValue>> = unsafe {
            match &(*ptr).data {
                ObjData::Tuple(d) => Some(d.clone()),
                ObjData::List(lk) => Some(lk.read().unwrap().to_vec()),
                _ => None,
            }
        };
        if let Some(digits) = digits {
            for d in digits {
                match d.as_int_pyint() {
                    Some(v) if (0..=9).contains(&v) => {}
                    _ => {
                        return Err(
                            "The second value in the tuple must be composed of integers in the range 0 through 9."
                                .to_string(),
                        );
                    }
                }
            }
            return Ok(());
        }
    }
    Err(
        "The second value in the tuple must be composed of integers in the range 0 through 9."
            .to_string(),
    )
}

/// `'F'` exponent in the tuple form means Infinity.
fn tuple_exponent_is_infinity(exp: MbValue) -> bool {
    let mut is_f = false;
    with_str(exp, |s| {
        is_f = s == "F";
    });
    is_f
}

/// Build a finite state from a validated CPython `(sign, digits, exp)` tuple.
fn decimal_from_tuple(items: &[MbValue]) -> MbDecimal {
    let sign = items[0].as_int_pyint().unwrap_or(0);
    let exp = items[2].as_int_pyint().unwrap_or(0);
    let mut coeff = String::new();
    if let Some(ptr) = items[1].as_ptr() {
        let digits: Vec<MbValue> = unsafe {
            match &(*ptr).data {
                ObjData::Tuple(d) => d.clone(),
                ObjData::List(lk) => lk.read().unwrap().to_vec(),
                _ => Vec::new(),
            }
        };
        for d in digits {
            if let Some(v) = d.as_int_pyint() {
                coeff.push(char::from(b'0' + (v as u8)));
            }
        }
    }
    if coeff.is_empty() {
        coeff.push('0');
    }
    let c: BigInt = BigInt::from_str(&coeff).unwrap_or_else(|_| BigInt::from(0u32));
    let is_zero = c.is_zero();
    let signed = if sign == 1 { -c } else { c };
    let mut state = finite_from_coeff_scale(signed, -exp)
        .unwrap_or_else(|_| MbDecimal::finite(Decimal::ZERO));
    if is_zero {
        if sign == 1 {
            state.value.set_sign_negative(true);
            state.neg = true;
        }
        state.tuple_exp = Some(exp);
    }
    state
}

// ── Public surface — construction ────────────────────────────────────────

/// `Decimal(val)` — construct a new Decimal handle.
///
/// `provided` distinguishes `Decimal()` (no argument — CPython yields
/// `Decimal('0')`) from `Decimal(None)` (one argument that is `None` —
/// CPython raises `TypeError`). The dispatch thunk passes the real argument
/// count so the two cannot be confused.
pub fn mb_decimal_new_argc(val: MbValue, provided: bool) -> MbValue {
    // Zero-argument form: Decimal() == Decimal('0').
    if !provided {
        return make_handle(Decimal::ZERO);
    }

    // Already-allocated handle (Decimal(Decimal(...))) — copy through.
    if let Some(st) = get_state(val) {
        return make_state_handle(st);
    }

    // None argument is a TypeError in CPython (cannot convert NoneType).
    if val.is_none() {
        return raise_type_error("conversion from NoneType to Decimal is not supported");
    }

    // Plain pyint / bool.
    if let Some(i) = val.as_int_pyint() {
        return make_handle(Decimal::from(i));
    }

    // Float — captures the EXACT binary expansion (CPython: `Decimal(0.1)` is
    // the full ~55-digit `0.1000...625`). The rust_decimal payload holds a
    // best-effort approximation for fast paths, but the exact coefficient/scale
    // is interned and used for str/repr and comparison. Specials map to their
    // Decimal classes; signed zero is preserved.
    if let Some(f) = val.as_float() {
        if f.is_nan() {
            return make_state_handle(MbDecimal::qnan());
        }
        if f.is_infinite() {
            return make_state_handle(MbDecimal::inf(f < 0.0));
        }
        let mut d = Decimal::try_from(f).unwrap_or(Decimal::ZERO);
        if f == 0.0 && f.is_sign_negative() {
            d.set_sign_negative(true);
        }
        let mut st = MbDecimal::finite(d);
        // Exact expansion: zero stays a plain zero (no oversized coefficient),
        // non-zero captures the full terminating-decimal expansion.
        if f != 0.0 {
            let (coeff, scale) = f64_to_exact_decimal(f);
            st.neg = f < 0.0;
            st.exact = Some(intern_exact(coeff, scale));
        }
        return make_state_handle(st);
    }

    if let Some(ptr) = val.as_ptr() {
        unsafe {
            // Heap big integer.
            if let ObjData::BigInt(ref b) = (*ptr).data {
                return match finite_from_coeff_scale(b.clone(), 0) {
                    Ok(st) => make_state_handle(st),
                    Err(()) => overflow_result(),
                };
            }
            // Argument-tuple form: (sign, digits, exponent).
            if matches!(&(*ptr).data, ObjData::Tuple(_) | ObjData::List(_)) {
                let items: Vec<MbValue> = match &(*ptr).data {
                    ObjData::Tuple(d) => d.clone(),
                    ObjData::List(lk) => lk.read().unwrap().to_vec(),
                    _ => Vec::new(),
                };
                // 'F' exponent → Infinity (sign from the first element).
                if items.len() == 3 && tuple_exponent_is_infinity(items[2]) {
                    let neg = items[0].as_int_pyint() == Some(1);
                    return make_state_handle(MbDecimal::inf(neg));
                }
                return match validate_decimal_tuple(&items) {
                    Ok(()) => make_state_handle(decimal_from_tuple(&items)),
                    Err(msg) => raise_value_error(&msg),
                };
            }
        }
    }

    // String form — the canonical entry point. Empty/whitespace, special
    // values (nan/inf/snan), and well-formed numerics are accepted; anything
    // else is a conversion-syntax `InvalidOperation`.
    if let Some(sptr) = val.as_ptr() {
        let is_str = unsafe { matches!(&(*sptr).data, ObjData::Str(_)) };
        if is_str {
            let mut result: Option<MbValue> = None;
            with_str(val, |s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    // Decimal('') is actually InvalidOperation in CPython.
                    result = Some(raise_invalid_operation(
                        "[<class 'decimal.ConversionSyntax'>]",
                    ));
                } else if let Some(special) = parse_special_decimal_str(trimmed) {
                    result = Some(make_state_handle(special));
                } else if let Ok(mut d) = Decimal::from_str(trimmed) {
                    if d.is_zero() && trimmed.starts_with('-') {
                        d.set_sign_negative(true);
                    }
                    result = Some(make_handle(d));
                } else if let Ok(d) = Decimal::from_scientific(trimmed) {
                    result = Some(make_handle(d));
                } else {
                    result = Some(raise_invalid_operation(
                        "[<class 'decimal.ConversionSyntax'>]",
                    ));
                }
            });
            return result.unwrap_or_else(|| make_handle(Decimal::ZERO));
        }
    }

    // Unknown shape — InvalidOperation (conversion syntax).
    raise_invalid_operation("[<class 'decimal.ConversionSyntax'>]")
}

/// `Decimal(val)` — legacy single-argument entry point (always treats the
/// argument as provided). Retained for method-dispatch / internal callers.
pub fn mb_decimal_new(val: MbValue) -> MbValue {
    mb_decimal_new_argc(val, true)
}

/// `d.is_zero()` — kept as the bench-fixture readback (int 0/1 shape).
pub fn mb_decimal_is_zero(d: MbValue) -> MbValue {
    let Some(st) = get_state(d) else {
        return MbValue::from_int(0);
    };
    let z = st.class == DecClass::Finite && st.value.is_zero();
    MbValue::from_int(if z { 1 } else { 0 })
}

// ── Method dispatch (class.rs integer-handle arm) ────────────────────────

/// Extract a `rounding=` argument: a positional rounding-mode string or
/// a trailing kwargs dict (the method-call lowering packs keywords into
/// one trailing dict).
fn extract_rounding(args: &[MbValue]) -> Rounding {
    for &a in args {
        let mut found: Option<Rounding> = None;
        with_str(a, |s| {
            found = rounding_from_str(s);
        });
        if let Some(r) = found {
            return r;
        }
        if let Some(ptr) = a.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    let k = super::super::dict_ops::DictKey::Str("rounding".to_string());
                    if let Some(v) = guard.get(&k) {
                        let mut found2: Option<Rounding> = None;
                        with_str(*v, |s| {
                            found2 = rounding_from_str(s);
                        });
                        if let Some(r) = found2 {
                            return r;
                        }
                    }
                }
            }
        }
    }
    Rounding::HalfEven
}

fn extract_context_arg(args: &[MbValue]) -> Option<MbValue> {
    for &a in args {
        if ctx_field(a, "Emin").is_some() {
            return Some(a);
        }
        if let Some(ptr) = a.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    let k = super::super::dict_ops::DictKey::Str("context".to_string());
                    if let Some(v) = guard.get(&k) {
                        if v.is_none() {
                            return None;
                        }
                        if ctx_field(*v, "Emin").is_some() {
                            return Some(*v);
                        }
                    }
                }
            }
        }
    }
    None
}

fn ctx_int(ctx: MbValue, key: &str) -> Option<i64> {
    ctx_field(ctx, key).and_then(|v| v.as_int())
}

fn context_precision(ctx: MbValue) -> i64 {
    ctx_int(ctx, "prec").unwrap_or(PREC as i64).clamp(1, PREC as i64)
}

fn round_state_to_context(st: &MbDecimal, ctx: MbValue) -> Option<MbDecimal> {
    if st.class != DecClass::Finite {
        return Some(*st);
    }
    let prec = context_precision(ctx);
    let (c, s) = state_coeff_scale(st);
    let neg = c.is_negative() || (st.value.is_zero() && st.value.is_sign_negative());
    let c_abs: BigInt = c.magnitude().to_owned().into();
    let digits = digit_count(&c_abs) as i64;
    if digits <= prec {
        return Some(*st);
    }
    let drop = digits - prec;
    let rounded_abs = round_abs_with_mode(&c_abs, drop as u32, Rounding::HalfEven, neg);
    let signed = if neg { -rounded_abs } else { rounded_abs };
    let mut out = finite_from_coeff_scale_preserve(signed, s - drop)?;
    if neg && out.value.is_zero() {
        out.value.set_sign_negative(true);
        out.neg = true;
    }
    Some(out)
}

fn normalize_state_with_context(st: &MbDecimal, context: Option<MbValue>) -> Option<MbDecimal> {
    if st.class != DecClass::Finite {
        return Some(*st);
    }
    let rounded = if let Some(ctx) = context {
        round_state_to_context(st, ctx)?
    } else {
        *st
    };
    let (mut c, mut s) = state_coeff_scale(&rounded);
    if c.is_zero() {
        let mut zero = finite_from_coeff_scale(BigInt::from(0u32), 0).ok()?;
        if rounded.neg || rounded.value.is_sign_negative() {
            zero.value.set_sign_negative(true);
            zero.neg = true;
        }
        return Some(zero);
    }
    while (&c % 10u32).is_zero() {
        c /= 10u32;
        s -= 1;
    }
    finite_from_coeff_scale_preserve(c, s)
}

fn decimal_sqrt_with_context(st: &MbDecimal, args: &[MbValue]) -> MbValue {
    let result = decimal_sqrt(st);
    let Some(ctx) = extract_context_arg(args) else {
        return result;
    };
    let Some(result_st) = get_state(result) else {
        return result;
    };
    round_state_to_context(&result_st, ctx)
        .map(make_state_handle)
        .unwrap_or_else(overflow_result)
}

fn decimal_exp_with_context(st: &MbDecimal, args: &[MbValue]) -> MbValue {
    match st.class {
        DecClass::QNan | DecClass::SNan => return make_state_handle(MbDecimal::qnan()),
        DecClass::Inf => {
            if st.neg {
                return make_handle(Decimal::ZERO);
            }
            return overflow_result();
        }
        DecClass::Finite => {}
    }
    let ctx = extract_context_arg(args).unwrap_or_else(current_ctx);
    let approx = st.value.to_f64().unwrap_or(0.0).exp();
    let emax = ctx_int(ctx, "Emax").unwrap_or(999_999);
    let adjusted = if approx == 0.0 {
        i64::MIN
    } else {
        approx.abs().log10().floor() as i64
    };
    if !approx.is_finite() || adjusted > emax {
        set_context_flag(ctx, "Overflow");
        return raise_overflow("above Emax");
    }
    let mut d = Decimal::try_from(approx).unwrap_or(Decimal::ZERO);
    d = d.normalize();
    let st = MbDecimal::finite(d);
    round_state_to_context(&st, ctx)
        .map(make_state_handle)
        .unwrap_or_else(overflow_result)
}

fn decimal_number_class(st: &MbDecimal, context: Option<MbValue>) -> String {
    let sign = if st.neg || (st.class == DecClass::Finite && st.value.is_sign_negative()) {
        "-"
    } else {
        "+"
    };
    match st.class {
        DecClass::SNan => "sNaN".to_string(),
        DecClass::QNan => "NaN".to_string(),
        DecClass::Inf => format!("{sign}Infinity"),
        DecClass::Finite => {
            if st.value.is_zero() {
                format!("{sign}Zero")
            } else {
                let ctx = context.unwrap_or_else(current_ctx);
                let emin = ctx_field(ctx, "Emin")
                    .and_then(|v| v.as_int())
                    .unwrap_or(-999_999);
                let (coeff, scale) = coeff_scale(&st.value);
                let adjusted = -scale + digit_count(&coeff) as i64 - 1;
                if adjusted < emin {
                    format!("{sign}Subnormal")
                } else {
                    format!("{sign}Normal")
                }
            }
        }
    }
}

/// Coerce a method operand (Decimal handle or int) into a state; raises
/// InvalidOperation on unsupported shapes and returns None.
fn method_operand(v: MbValue) -> Option<MbDecimal> {
    match arith_operand(v) {
        Ok(st) => Some(st),
        Err(_) => {
            raise_invalid_operation("unsupported operand");
            None
        }
    }
}

/// Integer square root (floor) of a non-negative BigInt.
fn isqrt_big(n: &BigInt) -> BigInt {
    if n.is_zero() {
        return BigInt::from(0u32);
    }
    // Newton's method seeded from the bit length.
    let bits = n.bits();
    let mut x = BigInt::from(1u32) << ((bits / 2) + 1) as usize;
    loop {
        let y = (&x + n / &x) / 2u32;
        if y >= x {
            return x;
        }
        x = y;
    }
}

/// `d.sqrt()` — correctly-rounded square root to 28 significant digits.
fn decimal_sqrt(st: &MbDecimal) -> MbValue {
    match st.class {
        DecClass::QNan | DecClass::SNan => return make_state_handle(MbDecimal::qnan()),
        DecClass::Inf => {
            if st.neg {
                return raise_invalid_operation("sqrt(-INF)");
            }
            return make_state_handle(MbDecimal::inf(false));
        }
        DecClass::Finite => {}
    }
    let (c, s) = coeff_scale(&st.value);
    if c.is_negative() {
        return raise_invalid_operation("sqrt of a negative number");
    }
    if c.is_zero() {
        // sqrt of (possibly signed) zero: exponent halves.
        let half = (s + 1) / 2;
        return match finite_from_coeff_scale(BigInt::from(0u32), half) {
            Ok(r) => make_state_handle(r),
            Err(()) => make_handle(Decimal::ZERO),
        };
    }
    // Make the exponent even: value = c * 10^-s.
    let mut c2 = c.clone();
    let mut s2 = s;
    if s2 % 2 != 0 {
        c2 *= 10u32;
        s2 += 1;
    }
    // Exact case first.
    let root = isqrt_big(&c2);
    if &root * &root == c2 {
        return match finite_from_coeff_scale(root, s2 / 2) {
            Ok(r) => make_state_handle(r),
            Err(()) => overflow_result(),
        };
    }
    // Inexact: extend to PREC digits and round (the spec rounds the
    // exact root half-even; an irrational root never lands exactly on a
    // half, so floor + midpoint test suffices).
    let shift = 2 * (PREC as i64 + 2);
    let scaled = &c2 * pow10(shift as u32);
    let r = isqrt_big(&scaled);
    // result = r * 10^-(PREC+2) * 10^(-s2/2); decide r vs r+1 by midpoint.
    let mid = (&r * 2u32 + 1u32) * (&r * 2u32 + 1u32);
    let q = if mid <= &scaled * 4u32 { r + 1u32 } else { r };
    let total_scale = s2 / 2 + PREC as i64 + 2;
    // Round to PREC digits half-even.
    let digits = digit_count(&q) as i64;
    let drop = (digits - PREC as i64).max(0);
    let rounded = round_half_even_abs(&q, drop as u32);
    match finite_from_coeff_scale(rounded, total_scale - drop) {
        Ok(rst) => make_state_handle(rst),
        Err(()) => overflow_result(),
    }
}

fn signed_zero_or_value_cmp(a: &MbDecimal, b: &MbDecimal) -> Option<std::cmp::Ordering> {
    let oa = state_operand(a);
    let ob = state_operand(b);
    cmp_operands(&oa, &ob)
}

fn state_operand(st: &MbDecimal) -> NumOperand {
    match st.class {
        DecClass::Finite => {
            let (c, s) = coeff_scale(&st.value);
            NumOperand::Rational(c, pow10(s as u32))
        }
        DecClass::Inf => NumOperand::Inf(st.neg),
        _ => NumOperand::Nan,
    }
}

/// Method-name dispatcher used by `class.rs::mb_call_method` for Decimal
/// handle receivers. Returns `None` when the method name is unknown
/// (caller falls through to generic handling).
pub fn dispatch_method(handle: MbValue, method: &str, args: &[MbValue]) -> Option<MbValue> {
    let a0 = args.first().copied().unwrap_or_else(MbValue::none);
    let st = get_state(handle)?;
    Some(match method {
        // Legacy bench/free-function names.
        "add" | "__add__" | "__radd__" => mb_decimal_add(handle, a0),
        "sub" | "__sub__" => mb_decimal_sub(handle, a0),
        "__rsub__" => mb_decimal_sub(a0, handle),
        "mul" | "__mul__" | "__rmul__" => mb_decimal_mul(handle, a0),
        "truediv" | "__truediv__" => mb_decimal_truediv(handle, a0),
        "__rtruediv__" => mb_decimal_truediv(a0, handle),
        "__floordiv__" => mb_decimal_floordiv(handle, a0),
        "__rfloordiv__" => mb_decimal_floordiv(a0, handle),
        "__mod__" => mb_decimal_rem(handle, a0),
        "__rmod__" => mb_decimal_rem(a0, handle),
        "__divmod__" => mb_decimal_divmod(handle, a0),
        "__pow__" => mb_decimal_pow(handle, a0),
        "__pos__" => mb_decimal_pos(handle),
        "__neg__" => mb_decimal_neg(handle),
        "__abs__" | "copy_abs" => mb_decimal_abs(handle),
        "copy_negate" => {
            // Unlike unary minus, copy_negate flips the sign without
            // context normalization (keeps -0).
            match st.class {
                DecClass::Finite => {
                    let mut d = st.value;
                    d.set_sign_negative(!d.is_sign_negative());
                    make_handle(d)
                }
                DecClass::Inf => make_state_handle(MbDecimal::inf(!st.neg)),
                _ => make_state_handle(st),
            }
        }
        "__eq__" | "__ne__" => {
            let eq = mb_numeric_handle_eq(handle, a0);
            match eq {
                Some(r) => MbValue::from_bool(if method == "__eq__" { r } else { !r }),
                None => MbValue::from_bool(method == "__ne__"),
            }
        }
        "__lt__" | "__le__" | "__gt__" | "__ge__" => {
            // Ordering against complex is NotImplemented in CPython even
            // when the imaginary part is zero (equality is the exception).
            if let Some(ptr) = a0.as_ptr() {
                unsafe {
                    if matches!(&(*ptr).data, ObjData::Complex(..)) {
                        return Some(MbValue::not_implemented());
                    }
                }
            }
            let oa = classify_numeric(handle);
            let ob = classify_numeric(a0);
            match (oa, ob) {
                (Some(x), Some(y)) => {
                    if matches!(x, NumOperand::Nan) || matches!(y, NumOperand::Nan) {
                        raise_invalid_operation("comparison involving NaN");
                        return Some(MbValue::none());
                    }
                    let ord = cmp_operands(&x, &y);
                    let r = match method {
                        "__lt__" => ord == Some(std::cmp::Ordering::Less),
                        "__le__" => ord != Some(std::cmp::Ordering::Greater),
                        "__gt__" => ord == Some(std::cmp::Ordering::Greater),
                        _ => ord != Some(std::cmp::Ordering::Less),
                    };
                    MbValue::from_bool(r)
                }
                _ => MbValue::not_implemented(),
            }
        }
        "__format__" => {
            // CPython: the format spec must be a str — a bytes (or other
            // non-str) spec is a TypeError, raised before any formatting.
            let spec = a0.as_ptr().and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            });
            match spec {
                Some(s) => {
                    mb_numeric_handle_format(handle, &s).unwrap_or_else(|| mb_decimal_str(handle))
                }
                None => raise_type_error("__format__() argument 1 must be str, not bytes"),
            }
        }
        "__str__" | "str_" | "to_eng_string" => mb_decimal_str(handle),
        "__repr__" => mb_decimal_repr(handle),
        "__bool__" => mb_decimal_bool(handle),
        "__int__" | "__trunc__" | "to_integral" | "to_integral_value" | "to_integral_exact" => {
            if matches!(
                method,
                "to_integral" | "to_integral_value" | "to_integral_exact"
            ) {
                if st.class != DecClass::Finite {
                    return Some(make_state_handle(st));
                }
                let mode = extract_rounding(args);
                return Some(
                    quantize_to_scale(&st, 0, mode)
                        .map(make_state_handle)
                        .unwrap_or_else(|| {
                            raise_invalid_operation("quantize result has too many digits")
                        }),
                );
            }
            mb_decimal_int(handle)
        }
        "__float__" => mb_decimal_float(handle),
        "__round__" => {
            let given = !args.is_empty() && !a0.is_none();
            mb_decimal_round(handle, a0, given)
        }
        "__copy__" | "__deepcopy__" | "canonical" => make_state_handle(st),
        "is_zero" => mb_decimal_bool_inverse_zero(&st),
        "is_nan" => MbValue::from_bool(st.is_nan()),
        "is_qnan" => MbValue::from_bool(st.class == DecClass::QNan),
        "is_snan" => MbValue::from_bool(st.class == DecClass::SNan),
        "is_infinite" => MbValue::from_bool(st.class == DecClass::Inf),
        "is_finite" => MbValue::from_bool(st.class == DecClass::Finite),
        "is_signed" => MbValue::from_bool(match st.class {
            DecClass::Finite => st.value.is_sign_negative(),
            _ => st.neg,
        }),
        "number_class" => str_val(&decimal_number_class(&st, extract_context_arg(args))),
        "is_canonical" => MbValue::from_bool(true),
        "adjusted" => {
            if st.class != DecClass::Finite {
                MbValue::from_int(0)
            } else {
                let (c, s) = coeff_scale(&st.value);
                let digits = digit_count(&c) as i64;
                MbValue::from_int(-s + digits - 1)
            }
        }
        "as_tuple" => {
            let (sign, digits, exp): (i64, Vec<MbValue>, MbValue) = match st.class {
                DecClass::Inf => (
                    if st.neg { 1 } else { 0 },
                    vec![MbValue::from_int(0)],
                    MbValue::from_ptr(MbObject::new_str("F".to_string())),
                ),
                DecClass::QNan | DecClass::SNan => (
                    if st.neg { 1 } else { 0 },
                    vec![],
                    MbValue::from_ptr(MbObject::new_str(
                        if st.class == DecClass::QNan { "n" } else { "N" }.to_string(),
                    )),
                ),
                DecClass::Finite => {
                    let d = &st.value;
                    let sign = if st.neg || d.is_sign_negative() { 1 } else { 0 };
                    if d.is_zero() {
                        let exp = st.tuple_exp.unwrap_or_else(|| -(d.scale() as i64));
                        (sign, vec![MbValue::from_int(0)], MbValue::from_int(exp))
                    } else {
                        let digits_str = d.mantissa().unsigned_abs().to_string();
                        let digits: Vec<MbValue> = digits_str
                            .bytes()
                            .map(|b| MbValue::from_int((b - b'0') as i64))
                            .collect();
                        (sign, digits, MbValue::from_int(-(d.scale() as i64)))
                    }
                }
            };
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_int(sign),
                MbValue::from_ptr(MbObject::new_tuple(digits)),
                exp,
            ]))
        }
        "as_integer_ratio" => match st.class {
            DecClass::Finite => {
                let (c, s) = coeff_scale(&st.value);
                let den = pow10(s as u32);
                let g = gcd_big(&c, &den);
                let n = &c / &g;
                let d = &den / &g;
                MbValue::from_ptr(MbObject::new_tuple(vec![
                    big_to_mb_int(n),
                    big_to_mb_int(d),
                ]))
            }
            DecClass::Inf => {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "cannot convert Infinity to integer ratio".to_string(),
                    )),
                );
                MbValue::none()
            }
            _ => raise_value_error("cannot convert NaN to integer ratio"),
        },
        "compare" | "compare_signal" | "compare_total" | "compare_total_mag" => {
            let other = method_operand(a0)?;
            let other = if method == "compare_total_mag" {
                let mut o = other;
                if o.class == DecClass::Finite {
                    o.value = o.value.abs();
                } else {
                    o.neg = false;
                }
                o
            } else {
                other
            };
            let me = if method == "compare_total_mag" {
                let mut m = st;
                if m.class == DecClass::Finite {
                    m.value = m.value.abs();
                } else {
                    m.neg = false;
                }
                m
            } else {
                st
            };
            if me.is_nan() || other.is_nan() {
                return Some(make_state_handle(MbDecimal::qnan()));
            }
            let ord = signed_zero_or_value_cmp(&me, &other);
            let v = match ord {
                Some(std::cmp::Ordering::Less) => Decimal::from(-1),
                Some(std::cmp::Ordering::Greater) => Decimal::ONE,
                _ => Decimal::ZERO,
            };
            make_handle(v)
        }
        "max" | "min" | "max_mag" | "min_mag" => {
            let other = method_operand(a0)?;
            if st.is_nan() {
                return Some(make_state_handle(other));
            }
            if other.is_nan() {
                return Some(make_state_handle(st));
            }
            let (me_c, ot_c) = if method.ends_with("_mag") {
                let mut m = st;
                let mut o = other;
                if m.class == DecClass::Finite {
                    m.value = m.value.abs();
                } else {
                    m.neg = false;
                }
                if o.class == DecClass::Finite {
                    o.value = o.value.abs();
                } else {
                    o.neg = false;
                }
                (m, o)
            } else {
                (st, other)
            };
            let ord = signed_zero_or_value_cmp(&me_c, &ot_c).unwrap_or(std::cmp::Ordering::Equal);
            let pick_self = if method.starts_with("max") {
                ord != std::cmp::Ordering::Less
            } else {
                ord != std::cmp::Ordering::Greater
            };
            make_state_handle(if pick_self { st } else { other })
        }
        "quantize" => {
            let exp_st = method_operand(a0)?;
            if st.class != DecClass::Finite || exp_st.class != DecClass::Finite {
                if st.is_nan() || exp_st.is_nan() {
                    return Some(make_state_handle(MbDecimal::qnan()));
                }
                return Some(raise_invalid_operation("quantize with non-finite operand"));
            }
            let target_scale = exp_st.value.scale() as i64;
            let mode = extract_rounding(&args[1.min(args.len())..]);
            quantize_to_scale(&st, target_scale, mode)
                .map(make_state_handle)
                .unwrap_or_else(|| raise_invalid_operation("quantize result has too many digits"))
        }
        "same_quantum" => {
            let other = method_operand(a0)?;
            let same = match (st.class, other.class) {
                (DecClass::Finite, DecClass::Finite) => st.value.scale() == other.value.scale(),
                (a, b) => a == b,
            };
            MbValue::from_bool(same)
        }
        "scaleb" => {
            // Decimal handles are int-tagged, so the handle test must run
            // before `as_int_pyint` (which would read the raw id).
            let n = if let Some(s2) = get_state(a0) {
                if s2.class == DecClass::Finite {
                    let (c, sc) = coeff_scale(&s2.value);
                    let den = pow10(sc as u32);
                    if (&c % &den).is_zero() {
                        (&c / &den).to_i64()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                a0.as_int_pyint()
            };
            let Some(n) = n else {
                return Some(raise_invalid_operation("scaleb operand must be integral"));
            };
            if st.class != DecClass::Finite {
                return Some(make_state_handle(st));
            }
            let (c, s) = coeff_scale(&st.value);
            match finite_from_coeff_scale(c, s - n) {
                Ok(r) => make_state_handle(r),
                Err(()) => overflow_result(),
            }
        }
        "fma" => {
            // d.fma(other, third) — exact multiply-add, then one rounding.
            let other = method_operand(a0)?;
            let third = method_operand(args.get(1).copied().unwrap_or_else(MbValue::none))?;
            if st.is_nan() || other.is_nan() || third.is_nan() {
                return Some(make_state_handle(MbDecimal::qnan()));
            }
            if st.class != DecClass::Finite
                || other.class != DecClass::Finite
                || third.class != DecClass::Finite
            {
                return Some(raise_invalid_operation("fma with non-finite operand"));
            }
            let (ca, sa) = coeff_scale(&st.value);
            let (cb, sb) = coeff_scale(&other.value);
            let (cc, sc) = coeff_scale(&third.value);
            let prod = ca * cb;
            let ps = sa + sb;
            let s = ps.max(sc);
            let p2 = prod * pow10((s - ps) as u32);
            let c2 = cc * pow10((s - sc) as u32);
            match finite_from_coeff_scale(p2 + c2, s) {
                Ok(r) => make_state_handle(r),
                Err(()) => overflow_result(),
            }
        }
        "sqrt" => decimal_sqrt_with_context(&st, args),
        "normalize" => {
            if st.class != DecClass::Finite {
                return Some(make_state_handle(st));
            }
            normalize_state_with_context(&st, extract_context_arg(args))
                .map(make_state_handle)
                .unwrap_or_else(overflow_result)
        }
        "exp" => decimal_exp_with_context(&st, args),
        "conjugate" | "real" => make_state_handle(st),
        "imag" => make_handle(Decimal::ZERO),
        "radix" => make_handle(Decimal::from(10)),
        "__hash__" => {
            let small = mb_numeric_handle_integral_i64(handle)
                .filter(|i| (-(1i64 << 47)..(1i64 << 47)).contains(i));
            if let Some(i) = small {
                MbValue::from_int(if i == -1 { -2 } else { i })
            } else {
                let s = state_to_string(&st);
                let mut h: i64 = 0;
                for b in s.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(b as i64);
                }
                let h = ((h << 16) >> 16).max(i64::MIN + 1);
                MbValue::from_int(if h == -1 { -2 } else { h })
            }
        }
        _ => return None,
    })
}

fn mb_decimal_bool_inverse_zero(st: &MbDecimal) -> MbValue {
    MbValue::from_bool(st.class == DecClass::Finite && st.value.is_zero())
}

// ── __format__ mini-language (decimal subset) ────────────────────────────
//
// Port of the `_pydecimal.__format__` pipeline for the fixture-covered
// subset: [[fill]align][sign][z][#][0][width][,|_][.precision][eEfFgGn%].

struct FormatSpec {
    fill: char,
    align: char, // '<' '>' '=' '^'
    sign: char,  // '-' '+' ' '
    coerce_z: bool,
    alt: bool,
    width: usize,
    thousands: Option<char>,
    precision: Option<i64>,
    typ: Option<char>,
}

fn parse_format_spec(spec: &str) -> Option<FormatSpec> {
    let chars: Vec<char> = spec.chars().collect();
    let mut i = 0usize;
    let mut fill = ' ';
    let mut align: Option<char> = None;
    if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '=' | '^') {
        fill = chars[0];
        align = Some(chars[1]);
        i = 2;
    } else if !chars.is_empty() && matches!(chars[0], '<' | '>' | '=' | '^') {
        align = Some(chars[0]);
        i = 1;
    }
    let mut sign = '-';
    if i < chars.len() && matches!(chars[i], '+' | '-' | ' ') {
        sign = chars[i];
        i += 1;
    }
    let mut coerce_z = false;
    if i < chars.len() && chars[i] == 'z' {
        coerce_z = true;
        i += 1;
    }
    let mut alt = false;
    if i < chars.len() && chars[i] == '#' {
        alt = true;
        i += 1;
    }
    if i < chars.len() && chars[i] == '0' && align.is_none() {
        fill = '0';
        align = Some('=');
        i += 1;
    }
    let mut width = 0usize;
    while i < chars.len() && chars[i].is_ascii_digit() {
        width = width * 10 + (chars[i] as usize - '0' as usize);
        i += 1;
    }
    let mut thousands = None;
    if i < chars.len() && (chars[i] == ',' || chars[i] == '_') {
        thousands = Some(chars[i]);
        i += 1;
    }
    let mut precision = None;
    if i < chars.len() && chars[i] == '.' {
        i += 1;
        let mut p = 0i64;
        let mut any = false;
        while i < chars.len() && chars[i].is_ascii_digit() {
            p = p * 10 + (chars[i] as i64 - '0' as i64);
            i += 1;
            any = true;
        }
        if !any {
            return None;
        }
        precision = Some(p);
    }
    let mut typ = None;
    if i < chars.len() {
        let t = chars[i];
        if matches!(t, 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | 'n' | '%') {
            typ = Some(t);
            i += 1;
        } else {
            return None;
        }
    }
    if i != chars.len() {
        return None;
    }
    Some(FormatSpec {
        fill,
        align: align.unwrap_or('>'),
        sign,
        coerce_z,
        alt,
        width,
        thousands,
        precision,
        typ,
    })
}

/// Round `|c| * 10^exp` to `prec_digits` significant digits, half-even.
fn round_sig(c_abs: BigInt, exp: i64, prec_digits: i64) -> (BigInt, i64) {
    if c_abs.is_zero() {
        return (c_abs, exp);
    }
    let digits = digit_count(&c_abs) as i64;
    if digits <= prec_digits {
        return (c_abs, exp);
    }
    let drop = digits - prec_digits;
    let mut r = round_half_even_abs(&c_abs, drop as u32);
    let mut e = exp + drop;
    if digit_count(&r) as i64 > prec_digits {
        r = round_half_even_abs(&r, 1);
        e += 1;
    }
    (r, e)
}

fn sign_prefix(neg: bool, sign: char) -> String {
    if neg {
        "-".to_string()
    } else {
        match sign {
            '+' => "+".to_string(),
            ' ' => " ".to_string(),
            _ => String::new(),
        }
    }
}

/// CPython `_insert_thousands_sep` port: group the integer digits in 3s
/// from the right, zero-padding the leftmost group(s) until the grouped
/// string is at least `min_width` chars (zero-pad-with-grouping rule).
fn insert_thousands_sep(mut digits: String, sep: Option<char>, mut min_width: i64) -> String {
    let Some(sep) = sep else {
        let pad = (min_width.max(0) as usize).saturating_sub(digits.len());
        return "0".repeat(pad) + &digits;
    };
    let mut groups: Vec<String> = Vec::new();
    loop {
        let l = (digits.len() as i64).max(min_width.max(1)).min(3) as usize;
        let take = l.min(digits.len());
        let tail = digits.split_off(digits.len() - take);
        groups.push("0".repeat(l - take) + &tail);
        min_width -= l as i64;
        if digits.is_empty() && min_width <= 0 {
            break;
        }
        min_width -= 1; // separator width
    }
    let mut out = String::new();
    for (i, g) in groups.iter().rev().enumerate() {
        if i > 0 {
            out.push(sep);
        }
        out.push_str(g);
    }
    out
}

fn format_align_signed(sign: &str, body: &str, spec: &FormatSpec) -> String {
    let content_len = sign.chars().count() + body.chars().count();
    if content_len >= spec.width {
        return format!("{sign}{body}");
    }
    let pad = spec.width - content_len;
    let fill: String = std::iter::repeat(spec.fill).take(pad).collect();
    match spec.align {
        '<' => format!("{sign}{body}{fill}"),
        '=' => format!("{sign}{fill}{body}"),
        '^' => {
            let half = pad / 2;
            let left: String = std::iter::repeat(spec.fill).take(half).collect();
            let right: String = std::iter::repeat(spec.fill).take(pad - half).collect();
            format!("{left}{sign}{body}{right}")
        }
        _ => format!("{fill}{sign}{body}"),
    }
}

fn format_decimal_state(st: &MbDecimal, spec: &FormatSpec) -> String {
    let neg0 = match st.class {
        DecClass::Finite => st.value.is_sign_negative(),
        _ => st.neg,
    };
    // Special values format as their words; type/precision are ignored.
    if st.class != DecClass::Finite {
        let body = match st.class {
            DecClass::Inf => "Infinity",
            DecClass::QNan => "NaN",
            DecClass::SNan => "sNaN",
            DecClass::Finite => unreachable!(),
        };
        let sign_str = sign_prefix(neg0, spec.sign);
        return format_align_signed(&sign_str, body, spec);
    }
    let typ = spec.typ.unwrap_or('\0'); // '\0' = no presentation type
    let mut neg = neg0;
    let (c, s) = coeff_scale(&st.value);
    let mut c_abs: BigInt = c.magnitude().to_owned().into();
    let mut exp = -s;
    if typ == '%' {
        exp += 2;
    }
    // Rounding per presentation type.
    match typ {
        'e' | 'E' => {
            if let Some(p) = spec.precision {
                let (r, e) = round_sig(c_abs, exp, p + 1);
                c_abs = r;
                exp = e;
            }
        }
        'f' | 'F' | '%' => {
            if let Some(p) = spec.precision {
                let target = -p;
                if exp < target {
                    c_abs = round_half_even_abs(&c_abs, (target - exp) as u32);
                    exp = target;
                } else if exp > target {
                    c_abs = &c_abs * pow10((exp - target) as u32);
                    exp = target;
                }
            }
        }
        _ => {
            // g/G/n and no-type behave alike for rounding.
            if let Some(p) = spec.precision {
                let (r, e) = round_sig(c_abs, exp, p.max(1));
                c_abs = r;
                exp = e;
            }
        }
    }
    // 'z': coerce a negative-zero result to positive zero.
    if spec.coerce_z && c_abs.is_zero() {
        neg = false;
    }
    let digits = c_abs.to_string();
    let leftdigits = exp + digits.len() as i64;
    let dotplace: i64 = match typ {
        'e' | 'E' => {
            if c_abs.is_zero() && spec.precision.is_some() {
                1 - spec.precision.unwrap()
            } else {
                1
            }
        }
        'f' | 'F' | '%' => leftdigits,
        _ => {
            if exp <= 0 && leftdigits > -6 {
                leftdigits
            } else {
                1
            }
        }
    };
    let (intpart, fracpart) = if dotplace <= 0 {
        ("0".to_string(), "0".repeat((-dotplace) as usize) + &digits)
    } else if dotplace as usize >= digits.len() {
        (
            digits.clone() + &"0".repeat(dotplace as usize - digits.len()),
            String::new(),
        )
    } else {
        (
            digits[..dotplace as usize].to_string(),
            digits[dotplace as usize..].to_string(),
        )
    };
    let out_exp = leftdigits - dotplace;
    let mut tail = String::new();
    if !fracpart.is_empty() || spec.alt {
        tail.push('.');
        tail.push_str(&fracpart);
    }
    if out_exp != 0 || matches!(typ, 'e' | 'E') {
        let echar = match typ {
            'e' | 'g' | 'n' => 'e',
            'E' | 'G' => 'E',
            _ => 'E', // no-type renders like str()
        };
        tail.push(echar);
        if out_exp >= 0 {
            tail.push('+');
        }
        tail.push_str(&out_exp.to_string());
    }
    if typ == '%' {
        tail.push('%');
    }
    let sign_str = sign_prefix(neg, spec.sign);
    // Zero-padding interacts with thousands grouping: pad the integer
    // digits (inside the grouping) so the whole field reaches `width`.
    let min_width: i64 = if spec.fill == '0' && spec.align == '=' {
        spec.width as i64 - (tail.chars().count() + sign_str.chars().count()) as i64
    } else {
        0
    };
    let int_grouped = if spec.thousands.is_some() || min_width > 0 {
        insert_thousands_sep(intpart, spec.thousands, min_width)
    } else {
        intpart
    };
    let body = int_grouped + &tail;
    format_align_signed(&sign_str, &body, spec)
}

/// `format(x, spec)` / f-string hook for Decimal and Fraction handles.
/// Returns `None` when `v` is not such a handle or the spec is outside
/// the supported mini-language (caller falls back to its formatter).
pub fn mb_numeric_handle_format(v: MbValue, spec_str: &str) -> Option<MbValue> {
    // Cheap handle check first — this hook sits on the hot f-string
    // formatting path, so non-handle values must exit on a range compare.
    let id = v.as_int()? as u64;
    if id < super::super::integer_handle_registry::HANDLE_MIN_ID {
        return None;
    }
    if is_decimal_handle(id) {
        let spec = parse_format_spec(spec_str)?;
        let st = get_state(v)?;
        return Some(MbValue::from_ptr(MbObject::new_str(format_decimal_state(
            &st, &spec,
        ))));
    }
    let (n, d) = super::fractions_mod::handle_num_den(id)?;
    let spec = parse_format_spec(spec_str)?;
    if spec.typ.is_none() {
        // Untyped spec formats the rational's str() form.
        let body = if d == 1 {
            format!("{}", n.abs())
        } else {
            format!("{}/{}", n.abs(), d)
        };
        let sign_str = sign_prefix(n < 0, spec.sign);
        return Some(MbValue::from_ptr(MbObject::new_str(format_align_signed(
            &sign_str, &body, &spec,
        ))));
    }
    // Typed spec: divide exactly through the 28-digit core and reuse the
    // decimal pipeline.
    let st = divide_finite(&Decimal::from(n), &Decimal::from(d)).ok()?;
    Some(MbValue::from_ptr(MbObject::new_str(format_decimal_state(
        &st, &spec,
    ))))
}

// ── Flat-args dispatch thunks (free-function entry points) ───────────────

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.first().copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.first().copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

/// `Decimal(...)` dispatch — passes the real argument count so the
/// constructor can tell `Decimal()` (-> Decimal('0')) apart from
/// `Decimal(None)` (-> TypeError).
unsafe extern "C" fn dispatch_Decimal(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let provided = nargs >= 1;
    let val = a.first().copied().unwrap_or_else(MbValue::none);
    mb_decimal_new_argc(val, provided)
}

// `getcontext()` / `setcontext(c)` / `localcontext(...)` context state. The
// shim models the fixture-backed subset: precision, core exponent fields,
// clearable flags, and scoped localcontext identity.
thread_local! {
    /// The active `localcontext()` stack (top = current). Empty → the default.
    static CTX_STACK: RefCell<Vec<MbValue>> = const { RefCell::new(Vec::new()) };
    /// The process-default context, created lazily with a STABLE identity so
    /// `getcontext() is getcontext()` holds (CPython thread-local singleton).
    static DEFAULT_CTX: RefCell<Option<MbValue>> = const { RefCell::new(None) };
}

/// A fresh `Context` instance carrying its core settings as fields.
fn new_context(prec: i64, rounding: &str, emin: i64, emax: i64) -> MbValue {
    let inst = MbObject::new_instance("Context".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut m = fields.write().unwrap();
            m.insert("prec".to_string(), int_val(prec));
            m.insert("rounding".to_string(), str_val(rounding));
            m.insert("Emin".to_string(), int_val(emin));
            m.insert("Emax".to_string(), int_val(emax));
            m.insert("capitals".to_string(), MbValue::from_int(1));
            m.insert("clamp".to_string(), MbValue::from_int(0));
            m.insert("flags".to_string(), new_flags_dict());
        }
    }
    MbValue::from_ptr(inst)
}

fn ctx_field(ctx: MbValue, key: &str) -> Option<MbValue> {
    ctx.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn new_flags_dict() -> MbValue {
    let dict = MbValue::from_ptr(MbObject::new_dict());
    reset_flags_dict(dict);
    dict
}

fn ensure_context_flags(ctx: MbValue) -> Option<MbValue> {
    if let Some(flags) = ctx_field(ctx, "flags") {
        return Some(flags);
    }
    let flags = new_flags_dict();
    if let Some(ptr) = ctx.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert("flags".to_string(), flags);
                return Some(flags);
            }
        }
    }
    None
}

fn reset_flags_dict(flags: MbValue) {
    if let Some(ptr) = flags.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut guard = lock.write().unwrap();
                guard.clear();
                for name in DECIMAL_CONTEXT_FLAGS {
                    guard.insert(
                        super::super::dict_ops::DictKey::Str((*name).to_string()),
                        MbValue::from_bool(false),
                    );
                }
            }
        }
    }
}

fn clear_context_flags(ctx: MbValue) {
    if let Some(flags) = ensure_context_flags(ctx) {
        reset_flags_dict(flags);
    }
}

fn set_context_flag(ctx: MbValue, name: &str) {
    if let Some(flags) = ensure_context_flags(ctx) {
        if let Some(ptr) = flags.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    lock.write().unwrap().insert(
                        super::super::dict_ops::DictKey::Str(name.to_string()),
                        MbValue::from_bool(true),
                    );
                }
            }
        }
    }
}

/// A new, independent `Context` copying `src`'s fields (flags become a fresh
/// dict so the copy's flags never alias the source's).
fn copy_context(src: MbValue) -> MbValue {
    let inst = MbObject::new_instance("Context".to_string());
    unsafe {
        if let (Some(sp), ObjData::Instance { ref fields, .. }) = (src.as_ptr(), &(*inst).data) {
            if let ObjData::Instance { fields: ref sfields, .. } = (*sp).data {
                let s = sfields.read().unwrap();
                let mut m = fields.write().unwrap();
                for (k, v) in s.iter() {
                    if k == "flags" {
                        m.insert(k.clone(), new_flags_dict());
                    } else {
                        super::super::rc::retain_if_ptr(*v);
                        m.insert(k.clone(), *v);
                    }
                }
            }
        }
    }
    MbValue::from_ptr(inst)
}

fn default_ctx() -> MbValue {
    DEFAULT_CTX.with(|d| {
        let mut g = d.borrow_mut();
        if g.is_none() {
            *g = Some(new_context(28, "ROUND_HALF_EVEN", -999999, 999999));
        }
        let c = g.unwrap();
        unsafe { super::super::rc::retain_if_ptr(c); }
        c
    })
}

/// The current context: the stack top if a `localcontext()` is active, else the
/// stable process-default singleton.
fn current_ctx() -> MbValue {
    match CTX_STACK.with(|s| s.borrow().last().copied()) {
        Some(c) => {
            unsafe { super::super::rc::retain_if_ptr(c); }
            c
        }
        None => default_ctx(),
    }
}

unsafe extern "C" fn dispatch_decimal_getcontext(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    current_ctx()
}

unsafe extern "C" fn dispatch_decimal_setcontext(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    if let Some(c) = a.first().copied() {
        if ctx_field(c, "prec").is_some() {
            unsafe { super::super::rc::retain_if_ptr(c); }
            DEFAULT_CTX.with(|d| *d.borrow_mut() = Some(c));
        }
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_decimal_localcontext(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    // Trailing kwargs dict (prec=/rounding=/Emin=/Emax=/capitals=/clamp=).
    let kw = a.last().copied().filter(|v| {
        v.as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
            .unwrap_or(false)
    });
    // capitals/clamp accept only 0 or 1.
    if let Some(kwv) = kw {
        if let Some(ptr) = kwv.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    for key in ["capitals", "clamp"] {
                        if let Some(v) = guard.get(key) {
                            let ok =
                                matches!(v.as_int(), Some(0) | Some(1)) || v.as_bool().is_some();
                            if !ok {
                                drop(guard);
                                super::super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(format!(
                                        "{key} must be 0 or 1"
                                    ))),
                                );
                                return MbValue::none();
                            }
                        }
                    }
                }
            }
        }
    }
    // Base: an explicit positional Context, else the current context.
    let pos_ctx = a.iter().copied().find(|v| ctx_field(*v, "prec").is_some());
    let base = pos_ctx.unwrap_or_else(current_ctx);
    let copy = copy_context(base);
    // Apply keyword overrides onto the copy.
    if let Some(kwv) = kw {
        if let Some(ptr) = kwv.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    if let Some(cp) = copy.as_ptr() {
                        if let ObjData::Instance { ref fields, .. } = (*cp).data {
                            let mut m = fields.write().unwrap();
                            for (k, v) in g.iter() {
                                if let super::super::dict_ops::DictKey::Str(s) = k {
                                    super::super::rc::retain_if_ptr(*v);
                                    m.insert(s.clone(), *v);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Wrap the prepared context in a `_LocalCtx` context manager.
    let cm = MbObject::new_instance("_LocalCtx".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*cm).data {
            fields.write().unwrap().insert("_ctx".to_string(), copy);
        }
    }
    MbValue::from_ptr(cm)
}

/// `_LocalCtx.__enter__` — push the prepared context as current, return it.
unsafe extern "C" fn localctx_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    let ctx = ctx_field(self_v, "_ctx").unwrap_or_else(MbValue::none);
    if !ctx.is_none() {
        unsafe { super::super::rc::retain_if_ptr(ctx); }
        CTX_STACK.with(|s| s.borrow_mut().push(ctx));
    }
    unsafe { super::super::rc::retain_if_ptr(ctx); }
    ctx
}

/// `_LocalCtx.__exit__` — pop the context, restoring the previous current one.
unsafe extern "C" fn localctx_exit(_self: MbValue, _args: MbValue) -> MbValue {
    CTX_STACK.with(|s| {
        if let Some(c) = s.borrow_mut().pop() {
            unsafe { super::super::rc::release_if_ptr(c); }
        }
    });
    MbValue::from_bool(false)
}

/// Surface-only stub for `Decimal` instance methods registered on the class so
/// that `hasattr(decimal.Decimal, "is_finite")` / `callable(...)` resolve to a
/// callable unbound method via mb_getattr's func->native-class method bridge.
/// Instance receivers route through `dispatch_method` instead.
unsafe extern "C" fn dispatch_decimal_method_stub(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::none()
}

dispatch_binary!(dispatch_decimal_add, mb_decimal_add);
dispatch_binary!(dispatch_decimal_sub, mb_decimal_sub);
dispatch_binary!(dispatch_decimal_mul, mb_decimal_mul);
dispatch_binary!(dispatch_decimal_truediv, mb_decimal_truediv);
dispatch_unary!(dispatch_decimal_str, mb_decimal_str);
dispatch_unary!(dispatch_decimal_is_zero, mb_decimal_is_zero);

// ── Module registration ──────────────────────────────────────────────────

/// Helper: a heap string MbValue (used for exception-name / rounding-mode
/// constants whose CPython value is the same string as their name).
fn str_val(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

/// Helper: an integer MbValue, using a BigInt heap object when the value
/// exceeds the 48-bit immediate range (`decimal.MAX_PREC` and friends are
/// ~1e18, far past `from_int`'s ±2^47 limit).
fn int_val(i: i64) -> MbValue {
    if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
        MbValue::from_int(i)
    } else {
        MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(i)))
    }
}

/// Build a callable *type-object* class shell — an `Instance` whose
/// `class_name` is `"type"` (so `callable(...)` reports `True`, matching the
/// CPython "a class is callable" surface) and whose `__name__` field names the
/// class. Each `(attr, value)` in `attrs` is stored as an instance field so
/// `hasattr(Cls, attr)` resolves through `mb_getattr`'s generic instance
/// field-lookup arm (a `class_name=="type"` miss falls through to the
/// `ObjData::Instance` field probe, which returns the stored value → `hasattr`
/// reports `True`).
///
/// Used for `decimal.Context`, which the surface fixtures probe with
/// `callable(decimal.Context)` **and** `hasattr(decimal.Context, "prec")` /
/// `hasattr(decimal.Context, "rounding")`.
fn make_type_class_shell(name: &str, attrs: &[(&str, MbValue)]) -> MbValue {
    let inst_ptr = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name.to_string())),
            );
            for (attr, value) in attrs {
                map.insert((*attr).to_string(), *value);
            }
        }
    }
    MbValue::from_ptr(inst_ptr)
}

// ── Context arithmetic/utility methods (int operands coerced to Decimal) ──

/// Positional args of a Context instance method (runtime passes a List/Tuple).
fn ctx_method_args(args: MbValue) -> Vec<MbValue> {
    args.as_ptr().map(|p| unsafe {
        match &(*p).data {
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => Vec::new(),
        }
    }).unwrap_or_default()
}

/// First positional argument coerced to a Decimal handle (CPython Context
/// methods accept int/str and coerce via the Decimal constructor).
fn ctx_first_decimal(args: MbValue) -> MbValue {
    let pos = ctx_method_args(args);
    let a = pos.first().copied().unwrap_or_else(MbValue::none);
    if a.as_int().map(|i| is_decimal_handle(i as u64)).unwrap_or(false) {
        return a; // already a Decimal handle
    }
    // CPython Context unary methods accept only int or Decimal; a str/float
    // operand is a TypeError (not silently coerced).
    if a.as_int_pyint().is_some() {
        return mb_decimal_new(a); // int / bool → Decimal
    }
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "argument must be int or Decimal".to_string(),
        )),
    );
    MbValue::none()
}

unsafe extern "C" fn ctx_to_eng_string(_self: MbValue, args: MbValue) -> MbValue {
    dispatch_method(ctx_first_decimal(args), "to_eng_string", &[]).unwrap_or_else(MbValue::none)
}
unsafe extern "C" fn ctx_normalize(self_v: MbValue, args: MbValue) -> MbValue {
    dispatch_method(ctx_first_decimal(args), "normalize", &[self_v]).unwrap_or_else(MbValue::none)
}
unsafe extern "C" fn ctx_to_integral_value(_self: MbValue, args: MbValue) -> MbValue {
    dispatch_method(ctx_first_decimal(args), "to_integral_value", &[]).unwrap_or_else(MbValue::none)
}
unsafe extern "C" fn ctx_copy_decimal(_self: MbValue, args: MbValue) -> MbValue {
    ctx_first_decimal(args)
}
/// Context.number_class(x) -> CPython class string ("+Normal", "-Zero", …).
unsafe extern "C" fn ctx_number_class(self_v: MbValue, args: MbValue) -> MbValue {
    let dec = ctx_first_decimal(args);
    let Some(st) = get_state(dec) else {
        return MbValue::none();
    };
    let cls = decimal_number_class(&st, Some(self_v));
    str_val(&cls)
}

unsafe extern "C" fn ctx_clear_flags(self_v: MbValue, _args: MbValue) -> MbValue {
    clear_context_flags(self_v);
    MbValue::none()
}

/// Register Context arithmetic/utility methods on the `Context` class.
fn register_context_methods() {
    let mut m: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in [
        ("to_eng_string",     ctx_to_eng_string     as *const () as usize),
        ("normalize",         ctx_normalize         as *const () as usize),
        ("to_integral_value", ctx_to_integral_value as *const () as usize),
        ("copy_decimal",      ctx_copy_decimal      as *const () as usize),
        ("number_class",      ctx_number_class      as *const () as usize),
        ("clear_flags",       ctx_clear_flags       as *const () as usize),
    ] {
        super::super::module::register_variadic_func(addr as u64);
        m.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("Context", Vec::new(), m);

    // `_LocalCtx` — the context-manager returned by `localcontext()`.
    let mut lm: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in [
        ("__enter__", localctx_enter as *const () as usize),
        ("__exit__", localctx_exit as *const () as usize),
    ] {
        super::super::module::register_variadic_func(addr as u64);
        lm.insert(name.to_string(), MbValue::from_func(addr));
    }
    super::super::class::mb_class_register("_LocalCtx", Vec::new(), lm);
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Decimal", dispatch_Decimal as usize),
        ("getcontext", dispatch_decimal_getcontext as usize),
        ("setcontext", dispatch_decimal_setcontext as usize),
        ("localcontext", dispatch_decimal_localcontext as usize),
        ("decimal_add", dispatch_decimal_add as usize),
        ("decimal_sub", dispatch_decimal_sub as usize),
        ("decimal_mul", dispatch_decimal_mul as usize),
        ("decimal_truediv", dispatch_decimal_truediv as usize),
        ("decimal_str", dispatch_decimal_str as usize),
        ("decimal_is_zero", dispatch_decimal_is_zero as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Signal / exception class names. CPython's `decimal` exception tree is a
    // family of `ArithmeticError` subclasses. This shim does not model the
    // full subclass MRO (that lives in class.rs / exception.rs), but it does
    // expose each name as a module attribute whose value is the matching
    // type-name string. That makes both `hasattr(decimal, "InvalidOperation")`
    // (surface) and `except decimal.InvalidOperation:` / `raise
    // InvalidOperation` (errors) resolve correctly: `mb_exception_matches`
    // compares the raised type-name string against the resolved attribute.
    // Register the signal tree with CPython's bases so issubclass()
    // resolves the full hierarchy through the class-registry MRO.
    let signal_tree: &[(&str, &[&str])] = &[
        ("DecimalException", &["ArithmeticError"]),
        ("Clamped", &["DecimalException"]),
        ("InvalidOperation", &["DecimalException"]),
        ("ConversionSyntax", &["InvalidOperation"]),
        ("DivisionByZero", &["DecimalException", "ZeroDivisionError"]),
        ("DivisionImpossible", &["InvalidOperation"]),
        (
            "DivisionUndefined",
            &["InvalidOperation", "ZeroDivisionError"],
        ),
        ("Inexact", &["DecimalException"]),
        ("InvalidContext", &["InvalidOperation"]),
        ("Rounded", &["DecimalException"]),
        ("Subnormal", &["DecimalException"]),
        ("Overflow", &["Inexact", "Rounded"]),
        ("Underflow", &["Inexact", "Rounded", "Subnormal"]),
        ("FloatOperation", &["DecimalException", "TypeError"]),
    ];
    for (name, bases) in signal_tree {
        super::super::class::mb_class_register(
            name,
            bases.iter().map(|b| b.to_string()).collect(),
            std::collections::HashMap::new(),
        );
        attrs.insert(name.to_string(), str_val(name));
    }

    // Rounding mode constants. In CPython each is a plain string equal to its
    // own name (e.g. `ROUND_CEILING == 'ROUND_CEILING'`).
    for name in [
        "ROUND_DOWN",
        "ROUND_HALF_UP",
        "ROUND_HALF_EVEN",
        "ROUND_CEILING",
        "ROUND_FLOOR",
        "ROUND_UP",
        "ROUND_HALF_DOWN",
        "ROUND_05UP",
    ] {
        attrs.insert(name.to_string(), str_val(name));
    }

    // Module flags / numeric limits (Python 3.12 values).
    attrs.insert("HAVE_THREADS".to_string(), MbValue::from_bool(true));
    attrs.insert("HAVE_CONTEXTVAR".to_string(), MbValue::from_bool(true));
    attrs.insert("MAX_PREC".to_string(), int_val(999_999_999_999_999_999));
    attrs.insert("MAX_EMAX".to_string(), int_val(999_999_999_999_999_999));
    attrs.insert("MIN_EMIN".to_string(), int_val(-999_999_999_999_999_999));
    attrs.insert("MIN_ETINY".to_string(), int_val(-1_999_999_999_999_999_997));

    // `Context` — the arithmetic-context class. Surface fixtures probe it with
    // `callable(decimal.Context)` AND `hasattr(decimal.Context, "prec")` /
    // `hasattr(decimal.Context, "rounding")`, so a bare `from_func` stub is not
    // enough (a func value carries no attributes). Register a callable
    // type-object shell that carries the CPython default context attributes
    // (`prec == 28`, `rounding == ROUND_HALF_EVEN`) as fields.
    attrs.insert(
        "Context".to_string(),
        make_type_class_shell(
            "Context",
            &[
                ("prec", MbValue::from_int(28)),
                ("rounding", str_val("ROUND_HALF_EVEN")),
            ],
        ),
    );

    // Pre-built context singletons / the DecimalTuple namedtuple type. These
    // remain presence placeholders; runtime-created Context instances carry
    // their own fields and flags.
    for name in [
        "BasicContext",
        "ExtendedContext",
        "DefaultContext",
        "DecimalTuple",
    ] {
        attrs.insert(name.to_string(), str_val(name));
    }

    super::register_module("decimal", attrs);

    // Register `Decimal`'s instance methods as a runtime class so that accessing
    // a method on the class (`decimal.Decimal.is_finite`) resolves to a callable
    // unbound method via mb_getattr's func->native-class method bridge. The
    // bridge keys on NATIVE_TYPE_NAMES[ctor_addr] -> class name, then validates
    // the attribute against the CLASS_REGISTRY table mb_class_register populates.
    // Instance receivers route through `dispatch_method` (real behaviour);
    // these stub funcs exist so class-attribute access stays callable.
    {
        let stub = dispatch_decimal_method_stub as usize as u64;
        super::super::module::register_variadic_func(stub);
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for name in [
            "adjusted",
            "as_integer_ratio",
            "as_tuple",
            "canonical",
            "compare",
            "compare_signal",
            "compare_total",
            "compare_total_mag",
            "conjugate",
            "copy_abs",
            "copy_negate",
            "copy_sign",
            "exp",
            "fma",
            "from_float",
            "is_canonical",
            "is_finite",
            "is_infinite",
            "is_nan",
            "is_normal",
            "is_qnan",
            "is_signed",
            "is_snan",
            "is_subnormal",
            "is_zero",
            "ln",
            "log10",
            "logb",
            "logical_and",
            "logical_invert",
            "logical_or",
            "logical_xor",
            "max",
            "max_mag",
            "min",
            "min_mag",
            "next_minus",
            "next_plus",
            "next_toward",
            "normalize",
            "number_class",
            "quantize",
            "radix",
            "remainder_near",
            "rotate",
            "same_quantum",
            "scaleb",
            "shift",
            "sqrt",
            "to_eng_string",
            "to_integral",
            "to_integral_exact",
            "to_integral_value",
        ] {
            methods.insert(name.to_string(), MbValue::from_func(stub as usize));
        }
        super::super::class::mb_class_register("Decimal", vec![], methods);

        // Context arithmetic/utility methods (coerce int operands -> Decimal).
        register_context_methods();

        // Bridge the `Decimal` constructor func -> its class name so the
        // func->native-class method bridge in mb_getattr fires.
        super::super::module::NATIVE_TYPE_NAMES.with(|m| {
            m.borrow_mut().insert(
                dispatch_Decimal as *const () as usize as u64,
                "Decimal".to_string(),
            );
        });
    }

    // #2111: integer-handle refcount hooks.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn read_str(val: MbValue) -> String {
        let mut out = String::new();
        with_str(val, |s| out.push_str(s));
        out
    }

    #[test]
    fn test_decimal_new_from_string() {
        let d = mb_decimal_new(s("3.14"));
        assert_eq!(read_str(mb_decimal_str(d)), "3.14");
    }

    #[test]
    fn test_decimal_new_from_int() {
        let d = mb_decimal_new(MbValue::from_int(42));
        assert_eq!(read_str(mb_decimal_str(d)), "42");
    }

    #[test]
    fn test_decimal_arithmetic_exact() {
        // The whole point of `decimal` — 0.1 + 0.2 == 0.3 exactly.
        let a = mb_decimal_new(s("0.1"));
        let b = mb_decimal_new(s("0.2"));
        let sum = mb_decimal_add(a, b);
        assert_eq!(read_str(mb_decimal_str(sum)), "0.3");
    }

    #[test]
    fn test_decimal_sub_mul() {
        let a = mb_decimal_new(s("10.5"));
        let b = mb_decimal_new(s("3.2"));
        assert_eq!(read_str(mb_decimal_str(mb_decimal_sub(a, b))), "7.3");
        assert_eq!(read_str(mb_decimal_str(mb_decimal_mul(a, b))), "33.60");
    }

    #[test]
    fn test_decimal_truediv() {
        let a = mb_decimal_new(s("10"));
        let b = mb_decimal_new(s("4"));
        // CPython: Decimal('10') / Decimal('4') == Decimal('2.5').
        let out = read_str(mb_decimal_str(mb_decimal_truediv(a, b)));
        assert_eq!(out, "2.5");
    }

    #[test]
    fn test_decimal_truediv_exact_pads_to_ideal_exponent() {
        // CPython: Decimal('10.0') / Decimal('2') == Decimal('5.0').
        let a = mb_decimal_new(s("10.0"));
        let b = mb_decimal_new(s("2"));
        assert_eq!(read_str(mb_decimal_str(mb_decimal_truediv(a, b))), "5.0");
        // CPython: Decimal('20') / Decimal('2') == Decimal('10').
        let c = mb_decimal_new(s("20"));
        let d = mb_decimal_new(s("2"));
        assert_eq!(read_str(mb_decimal_str(mb_decimal_truediv(c, d))), "10");
    }

    #[test]
    fn test_decimal_truediv_inexact_rounds_to_28_digits() {
        let a = mb_decimal_new(s("1"));
        let b = mb_decimal_new(s("3"));
        assert_eq!(
            read_str(mb_decimal_str(mb_decimal_truediv(a, b))),
            "0.3333333333333333333333333333"
        );
        let c = mb_decimal_new(s("2"));
        let d = mb_decimal_new(s("3"));
        assert_eq!(
            read_str(mb_decimal_str(mb_decimal_truediv(c, d))),
            "0.6666666666666666666666666667"
        );
    }

    #[test]
    fn test_decimal_truediv_by_zero_raises() {
        let a = mb_decimal_new(s("10"));
        let b = mb_decimal_new(s("0"));
        let out = mb_decimal_truediv(a, b);
        assert!(out.is_none());
        assert_eq!(
            super::super::super::exception::mb_has_exception().as_bool(),
            Some(true)
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_decimal_is_zero() {
        let z = mb_decimal_new(s("0"));
        assert_eq!(mb_decimal_is_zero(z).as_int(), Some(1));
        let nz = mb_decimal_new(s("0.0001"));
        assert_eq!(mb_decimal_is_zero(nz).as_int(), Some(0));
    }

    #[test]
    fn test_handle_recognition() {
        let d = mb_decimal_new(s("1.5"));
        let id = d.as_int().expect("handle is int") as u64;
        assert!(is_decimal_handle(id));
        // Foreign id is not recognised.
        assert!(!is_decimal_handle(u64::MAX));
    }

    #[test]
    fn test_decimal_specials_and_compare() {
        let inf = mb_decimal_new(s("Infinity"));
        assert_eq!(read_str(mb_decimal_str(inf)), "Infinity");
        let neg = mb_decimal_neg(inf);
        assert_eq!(read_str(mb_decimal_str(neg)), "-Infinity");
        let big = mb_decimal_new(s("999999"));
        assert_eq!(mb_numeric_handle_lt(big, inf), Some(true));
        let nan = mb_decimal_new(s("NaN"));
        let one = mb_decimal_new(s("1"));
        let sum = mb_decimal_add(nan, one);
        assert_eq!(read_str(mb_decimal_str(sum)), "NaN");
    }

    #[test]
    fn test_decimal_eq_cross_type_exact() {
        let d = mb_decimal_new(s("0.1"));
        // Decimal('0.1') != float 0.1 (binary expansion differs).
        assert_eq!(
            mb_numeric_handle_eq(d, MbValue::from_float(0.1)),
            Some(false)
        );
        let q = mb_decimal_new(s("0.25"));
        assert_eq!(
            mb_numeric_handle_eq(q, MbValue::from_float(0.25)),
            Some(true)
        );
        let ten = mb_decimal_new(s("10"));
        assert_eq!(mb_numeric_handle_eq(ten, MbValue::from_int(10)), Some(true));
    }

    #[test]
    fn test_decimal_sqrt() {
        let nine = mb_decimal_new(s("9"));
        let r = decimal_sqrt(&get_state(nine).unwrap());
        assert_eq!(read_str(mb_decimal_str(r)), "3");
        let two = mb_decimal_new(s("2"));
        let r2 = decimal_sqrt(&get_state(two).unwrap());
        assert_eq!(
            read_str(mb_decimal_str(r2)),
            "1.414213562373095048801688724"
        );
    }
}
