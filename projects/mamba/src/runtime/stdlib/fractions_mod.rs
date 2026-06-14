//! @codegen-skip: handwrite-pre-standardize
//!
//! fractions module for Mamba — Python 3.12 `fractions` stdlib.
//!
//! Provides the `Fraction` class as **integer handles** (i64 IDs)
//! backed by a thread-local `HashMap<u64, FractionState>` table.
//! Mirrors the OOP pattern established by `hashlib_mod` /
//! `hmac_mod` / `random_mod` / `array_mod` (see
//! `project_mamba_integer_handle_pattern`).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + integer-handle protocol)
//! is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1414 cluster anchor.
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - **Operator overloading (`a + b`, `a * b`, ...)**: mamba's JIT
//!   lowers `int + int` to a native i64 add and never reaches
//!   `class.rs::mb_call_method.__add__`. Since handles ARE ints, the
//!   dispatch table for arith dunders is unreachable from
//!   operator-overload syntax. **Workaround**: callers use
//!   module-level functions instead (`fractions.fraction_add(a, b)`,
//!   `fractions.fraction_mul(a, b)`, ...). The `__add__` etc.
//!   branches in `dispatch_method` remain (callable via
//!   `a.__add__(b)`-style explicit method calls) so the protocol is
//!   still exhaustive — but `+` `*` `/` `-` are not transparent.
//!   This is the same compromise `decimal_mod` accepts (callers
//!   write `decimal.add(a, b)`, not `a + b`). Properly fixing this
//!   needs a new `MbValue` tag class for typed handles, or a
//!   compile-time type inference that routes `a + b` through dunder
//!   dispatch when `a` is known to be a Fraction — out of scope
//!   for the Phase 2 shim.
//! - `Fraction.from_decimal` — requires `decimal.Decimal` round-trip
//!   protocol; deferred until decimal handle ↔ fractions handle bridge
//!   lands.
//! - `Fraction.__pow__(complex)` — requires complex handle dispatch
//!   on the fraction side; deferred.
//! - `numbers.Rational` ABC inheritance — mamba does not synthesize
//!   ABC MRO for stdlib classes; integer-handle dispatch covers the
//!   concrete surface end users actually call.
//! - `__divmod__` returns a 2-tuple and therefore hits #2128
//!   (`MbObject::new_tuple` calls `gc::gc_track`, ~150x penalty in
//!   tight loops). Pre-documented as Gate 2 carve-out in the bench
//!   fixture; correctness is unaffected.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// HANDWRITE-BEGIN

/// Compute greatest common divisor (positive result; `gcd(0, n) = |n|`).
fn gcd(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

/// Backing state for a `Fraction` instance. Invariants maintained on
/// every constructor + arithmetic result:
///   - `den > 0` (sign carried by `num`)
///   - `gcd(|num|, den) == 1`
///   - `den != 0` always (we coerce `Fraction(_, 0)` to `0/1`)
#[derive(Clone, Copy)]
struct FractionState {
    num: i64,
    den: i64,
}

impl FractionState {
    fn new(num: i64, den: i64) -> Self {
        if den == 0 {
            return FractionState { num: 0, den: 1 };
        }
        let g = gcd(num, den);
        let mut n = num / g;
        let mut d = den / g;
        if d < 0 {
            n = -n;
            d = -d;
        }
        FractionState { num: n, den: d }
    }
}

/// Handle IDs live in a high range (above 2^40) so they cannot collide
/// with small literal ints used as numerator or denominator. Without
/// this offset, `mb_fraction_new(MbValue::from_int(1), ...)` could
/// coerce the literal `1` as if it were a pre-existing handle once
/// `id=1` was allocated (the FRACTIONS table grows monotonically).
/// MbValue ints are 48-bit boxed (see value.rs:73), so this base
/// stays well within range while sitting far above any realistic
/// numerator/denominator literal.
const FRACTION_HANDLE_BASE: u64 = 1u64 << 40;

thread_local! {
    static FRACTIONS: RefCell<HashMap<u64, FractionState>> = RefCell::new(HashMap::new());
    static FRACTION_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_FRACTION_ID: Cell<u64> = const { Cell::new(FRACTION_HANDLE_BASE) };
    /// Per-handle refcount (#2111).
    static FRACTION_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_fraction_id() -> u64 {
    NEXT_FRACTION_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Class.rs `mb_getattr` / `mb_call_method` calls this to decide
/// whether to route `int.method()` / `int.attr` into the fractions
/// protocol or fall through to primitive int handling.
pub fn is_fraction_handle(id: u64) -> bool {
    if id < FRACTION_HANDLE_BASE {
        return false;
    }
    FRACTION_IDS.with(|s| s.borrow().contains(&id))
}

/// Read back `(numerator, denominator)` for a Fraction handle — used by
/// the shared numeric-handle comparison in `decimal_mod` and the
/// builtins hash hook. Returns `None` for foreign ids.
pub fn handle_num_den(id: u64) -> Option<(i64, i64)> {
    if !is_fraction_handle(id) {
        return None;
    }
    FRACTIONS.with(|m| m.borrow().get(&id).map(|s| (s.num, s.den)))
}

fn drop_fraction_handle(id: u64) {
    FRACTIONS.with(|m| { m.borrow_mut().remove(&id); });
    FRACTION_IDS.with(|s| { s.borrow_mut().remove(&id); });
    FRACTION_REFCOUNTS.with(|r| { r.borrow_mut().remove(&id); });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_fraction_handle(id) {
        return false;
    }
    FRACTION_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_fraction_handle(id) {
        return false;
    }
    let should_drop = FRACTION_REFCOUNTS.with(|r| {
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
        drop_fraction_handle(id);
    }
    true
}

fn make_handle(state: FractionState) -> MbValue {
    let id = alloc_fraction_id();
    FRACTIONS.with(|m| { m.borrow_mut().insert(id, state); });
    FRACTION_IDS.with(|s| { s.borrow_mut().insert(id); });
    MbValue::from_int(id as i64)
}

/// Load the backing state for a handle (or `0/1` if not a known handle).
fn load(handle: MbValue) -> FractionState {
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        if let Some(s) = FRACTIONS.with(|m| m.borrow().get(&id).copied()) {
            return s;
        }
    }
    FractionState { num: 0, den: 1 }
}

/// Coerce an MbValue to a `FractionState` — handle if known, else
/// treat as int (`n/1`); falls back to `0/1`.
fn coerce(val: MbValue) -> FractionState {
    if let Some(id) = val.as_int() {
        let id_u = id as u64;
        if let Some(s) = FRACTIONS.with(|m| m.borrow().get(&id_u).copied()) {
            return s;
        }
        return FractionState { num: id, den: 1 };
    }
    FractionState { num: 0, den: 1 }
}

// ── Public surface — free functions used by both dispatch thunks
//    and class.rs method/attr dispatch on Fraction handles.

/// Box an i64 as an MbValue, spilling to a heap BigInt when it exceeds
/// the 48-bit NaN-box window (exact `from_float` denominators reach
/// 2^62; `MbValue::from_int` would abort in debug builds).
fn int_mb(i: i64) -> MbValue {
    if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
        MbValue::from_int(i)
    } else {
        MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(i)))
    }
}

/// Raise `ValueError(msg)` and return `None` (CPython error contract).
fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `OverflowError(msg)` and return `None`.
fn raise_overflow_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `ZeroDivisionError(msg)` and return `None`.
fn raise_zero_division_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Parse a `Fraction` string literal — `"[ws][sign]int[/int][ws]"` or a
/// decimal `"[ws][sign]int[.frac][ws]"`. Returns `None` if the trimmed
/// string is not a valid rational literal (caller raises `ValueError`,
/// matching CPython). Big-int / exponent forms beyond i64 are out of
/// scope (the surrounding shim is i64-only); those simply fail to parse.
fn parse_fraction_str(raw: &str) -> Option<FractionState> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Some((n_str, d_str)) = s.split_once('/') {
        let n: i64 = n_str.trim().parse().ok()?;
        let d: i64 = d_str.trim().parse().ok()?;
        if d == 0 {
            return None;
        }
        return Some(FractionState::new(n, d));
    }
    if let Some((int_part, frac_part)) = s.split_once('.') {
        // Decimal literal: "[sign]int.frac" -> (int.frac scaled by 10^len).
        let (sign, int_digits) = match int_part.strip_prefix('-') {
            Some(rest) => (-1_i64, rest),
            None => (1_i64, int_part.strip_prefix('+').unwrap_or(int_part)),
        };
        if frac_part.is_empty()
            || !frac_part.bytes().all(|b| b.is_ascii_digit())
            || !(int_digits.is_empty() || int_digits.bytes().all(|b| b.is_ascii_digit()))
        {
            return None;
        }
        let int_val: i64 = if int_digits.is_empty() { 0 } else { int_digits.parse().ok()? };
        let frac_val: i64 = frac_part.parse().ok()?;
        let scale = 10_i64.checked_pow(frac_part.len() as u32)?;
        let num = sign * (int_val.checked_mul(scale)?.checked_add(frac_val)?);
        return Some(FractionState::new(num, scale));
    }
    let n: i64 = s.parse().ok()?;
    Some(FractionState::new(n, 1))
}

/// `Fraction(num, den)` — primary constructor. Either argument may
/// itself be a Fraction handle (handled via `coerce`).
pub fn mb_fraction_new(num: MbValue, den: MbValue) -> MbValue {
    // ── String constructor: Fraction("3/4"), Fraction("0.5"), Fraction("-3/4").
    //    Parse the literal; unparseable input raises ValueError (CPython). Only
    //    fires when `num` is actually a str object, never on numeric args.
    if let Some(s) = extract_str(num) {
        return match parse_fraction_str(&s) {
            Some(st) => make_handle(st),
            None => raise_value_error(&format!("Invalid literal for Fraction: {s:?}")),
        };
    }
    // ── Decimal constructor: Fraction(Decimal('1.5')) → exact 3/2. Decimal
    //    handles are int-tagged, so this must run before the int coercion
    //    below (which would treat the handle id as a huge numerator).
    if den.is_none() {
        if let Some((n, d)) = super::decimal_mod::decimal_to_num_den(num) {
            return make_handle(FractionState::new(n, d));
        }
    }
    // ── Float constructor: Fraction(0.5). A non-finite float is a hard error in
    //    CPython: +/-inf -> OverflowError, NaN -> ValueError. Detected on the
    //    raw f64 (Python `nan != nan` is unreliable here); never fires on a
    //    finite float or on an int handle (ints are not `is_float`).
    if num.is_float() {
        if let Some(v) = num.as_float() {
            if v.is_nan() {
                return raise_value_error("cannot convert float NaN to integer");
            }
            if v.is_infinite() {
                return raise_overflow_error("cannot convert float infinity to integer");
            }
            return mb_fraction_from_float(num);
        }
    }
    // ── Explicit zero denominator: Fraction(n, 0) -> ZeroDivisionError. Guarded
    //    on `!den.is_none()` so a single-arg `Fraction(n)` (den omitted ->
    //    None) never trips this; only an actually-supplied zero denominator
    //    (int 0 or a zero-valued Fraction handle) raises.
    if !den.is_none() && coerce(den).num == 0 {
        return raise_zero_division_error("Fraction(n, 0)");
    }
    // An argument that is not a number / string / Rational (e.g. a list)
    // has no Fraction conversion: CPython raises TypeError rather than
    // silently coercing to Fraction(0).
    if let Some(p) = num.as_ptr() {
        if matches!(
            unsafe { &(*p).data },
            ObjData::List(_) | ObjData::Tuple(_) | ObjData::Dict(_)
                | ObjData::Set(_) | ObjData::FrozenSet(_)
                | ObjData::Bytes(_) | ObjData::ByteArray(_)
        ) {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "argument should be a string or a Rational instance, not '{}'",
                    super::super::builtins::value_type_name(num)
                ))),
            );
            return MbValue::none();
        }
    }
    let a = coerce(num);
    // Single-argument form: `Fraction(5)` (den omitted → None) means
    // denominator 1, NOT `coerce(None)` (which is 0/1 and collapsed every
    // single-arg integer Fraction to zero).
    let b = if den.is_none() {
        FractionState { num: 1, den: 1 }
    } else {
        coerce(den)
    };
    // (a_n / a_d) / (b_n / b_d)  =  a_n * b_d / (a_d * b_n)
    let n = a.num.saturating_mul(b.den);
    let d = a.den.saturating_mul(b.num);
    make_handle(FractionState::new(n, d))
}

/// `Fraction.from_float(f)` — build the exact rational expansion of the
/// binary float (mantissa × 2^exp) whenever numerator and denominator
/// both fit i64; falls back to a 10^9 scaling for extreme magnitudes
/// (mamba's fraction state is i64-only).
pub fn mb_fraction_from_float(f: MbValue) -> MbValue {
    let v = f.as_float().unwrap_or(0.0);
    if !v.is_finite() {
        return make_handle(FractionState { num: 0, den: 1 });
    }
    // Exact decomposition: v = ±mant * 2^e with mant < 2^53.
    let bits = v.to_bits();
    let sign = if (bits >> 63) & 1 == 1 { -1i64 } else { 1i64 };
    let exp_field = ((bits >> 52) & 0x7FF) as i64;
    let frac = (bits & 0x000F_FFFF_FFFF_FFFF) as i64;
    let (mut mant, mut e) = if exp_field == 0 {
        (frac, -1074i64)
    } else {
        (frac | (1i64 << 52), exp_field - 1075)
    };
    // Reduce the power of two shared with the mantissa.
    while mant != 0 && mant % 2 == 0 && e < 0 {
        mant /= 2;
        e += 1;
    }
    if (0..=10).contains(&e) && mant <= (i64::MAX >> e) {
        return make_handle(FractionState::new(sign * (mant << e), 1));
    }
    if (-62..0).contains(&e) {
        return make_handle(FractionState::new(sign * mant, 1i64 << (-e)));
    }
    // Fallback: magnitude outside the exact i64 window.
    let scale: f64 = 1_000_000_000.0;
    let n = (v * scale).round() as i64;
    make_handle(FractionState::new(n, scale as i64))
}

/// `f.numerator` attribute read.
pub fn mb_fraction_numerator(handle: MbValue) -> MbValue {
    int_mb(load(handle).num)
}

/// `f.denominator` attribute read.
pub fn mb_fraction_denominator(handle: MbValue) -> MbValue {
    int_mb(load(handle).den)
}

/// `f.real` — for rationals, identical to `f` itself.
pub fn mb_fraction_real(handle: MbValue) -> MbValue {
    handle
}

/// `f.imag` — always 0 for rationals.
pub fn mb_fraction_imag(_handle: MbValue) -> MbValue {
    MbValue::from_int(0)
}

/// `f.conjugate()` — rationals are self-conjugate.
pub fn mb_fraction_conjugate(handle: MbValue) -> MbValue {
    handle
}

/// `f.is_integer()` — `True` iff denominator is 1.
pub fn mb_fraction_is_integer(handle: MbValue) -> MbValue {
    MbValue::from_bool(load(handle).den == 1)
}

/// `f.as_integer_ratio()` — returns `(num, den)`. NOTE: tuple-return
/// path hits #2128.
pub fn mb_fraction_as_integer_ratio(handle: MbValue) -> MbValue {
    let s = load(handle);
    MbValue::from_ptr(MbObject::new_tuple(vec![int_mb(s.num), int_mb(s.den)]))
}

/// `f.limit_denominator(max_denominator=1000000)` — Stern-Brocot best
/// approximation under denominator cap.
pub fn mb_fraction_limit_denominator(handle: MbValue, max_den: MbValue) -> MbValue {
    let s = load(handle);
    let m = max_den.as_int().unwrap_or(1_000_000).max(1);
    if s.den <= m {
        return handle;
    }
    let (mut p0, mut q0, mut p1, mut q1) = (0_i64, 1_i64, 1_i64, 0_i64);
    let (mut n, mut d) = (s.num, s.den);
    while d != 0 {
        let a = n.div_euclid(d);
        let (p2, q2) = (a.saturating_mul(p1).saturating_add(p0),
                        a.saturating_mul(q1).saturating_add(q0));
        if q2 > m { break; }
        p0 = p1; q0 = q1; p1 = p2; q1 = q2;
        let r = n - a * d;
        n = d; d = r;
    }
    let k = (m - q0) / q1.max(1);
    let bound1 = FractionState::new(p0 + k * p1, q0 + k * q1);
    let bound2 = FractionState::new(p1, q1);
    // Pick the one closer to s.
    let diff1 = (bound1.num.saturating_mul(s.den) - s.num.saturating_mul(bound1.den)).abs();
    let diff2 = (bound2.num.saturating_mul(s.den) - s.num.saturating_mul(bound2.den)).abs();
    if diff1 * bound2.den <= diff2 * bound1.den {
        make_handle(bound1)
    } else {
        make_handle(bound2)
    }
}

// ── Arithmetic — flat results, return a new handle.

fn add_states(a: FractionState, b: FractionState) -> FractionState {
    let num = a.num.saturating_mul(b.den).saturating_add(b.num.saturating_mul(a.den));
    let den = a.den.saturating_mul(b.den);
    FractionState::new(num, den)
}

fn sub_states(a: FractionState, b: FractionState) -> FractionState {
    let num = a.num.saturating_mul(b.den).saturating_sub(b.num.saturating_mul(a.den));
    let den = a.den.saturating_mul(b.den);
    FractionState::new(num, den)
}

fn mul_states(a: FractionState, b: FractionState) -> FractionState {
    FractionState::new(a.num.saturating_mul(b.num), a.den.saturating_mul(b.den))
}

fn div_states(a: FractionState, b: FractionState) -> FractionState {
    // (a_n / a_d) / (b_n / b_d) = a_n * b_d / (a_d * b_n)
    FractionState::new(a.num.saturating_mul(b.den), a.den.saturating_mul(b.num))
}

pub fn mb_fraction_add(a: MbValue, b: MbValue) -> MbValue {
    make_handle(add_states(coerce(a), coerce(b)))
}
pub fn mb_fraction_sub(a: MbValue, b: MbValue) -> MbValue {
    make_handle(sub_states(coerce(a), coerce(b)))
}
pub fn mb_fraction_rsub(a: MbValue, b: MbValue) -> MbValue {
    make_handle(sub_states(coerce(b), coerce(a)))
}
pub fn mb_fraction_mul(a: MbValue, b: MbValue) -> MbValue {
    make_handle(mul_states(coerce(a), coerce(b)))
}
pub fn mb_fraction_truediv(a: MbValue, b: MbValue) -> MbValue {
    let bv = coerce(b);
    if bv.num == 0 {
        return raise_zero_division_error("Fraction(1, 0)");
    }
    make_handle(div_states(coerce(a), bv))
}
pub fn mb_fraction_rtruediv(a: MbValue, b: MbValue) -> MbValue {
    let av = coerce(a);
    if av.num == 0 {
        return raise_zero_division_error("Fraction(1, 0)");
    }
    make_handle(div_states(coerce(b), av))
}

pub fn mb_fraction_floordiv(a: MbValue, b: MbValue) -> MbValue {
    let q = div_states(coerce(a), coerce(b));
    int_mb(q.num.div_euclid(q.den))
}
pub fn mb_fraction_rfloordiv(a: MbValue, b: MbValue) -> MbValue {
    let q = div_states(coerce(b), coerce(a));
    int_mb(q.num.div_euclid(q.den))
}

pub fn mb_fraction_mod(a: MbValue, b: MbValue) -> MbValue {
    // a mod b  =  a - (a // b) * b
    let av = coerce(a);
    let bv = coerce(b);
    let q = div_states(av, bv);
    let floor = q.num.div_euclid(q.den);
    let prod = mul_states(FractionState::new(floor, 1), bv);
    make_handle(sub_states(av, prod))
}
pub fn mb_fraction_rmod(a: MbValue, b: MbValue) -> MbValue {
    mb_fraction_mod(b, a)
}

/// `__divmod__(a, b)` — returns `(a // b, a % b)`. Tuple-return path
/// hits #2128.
pub fn mb_fraction_divmod(a: MbValue, b: MbValue) -> MbValue {
    let q = mb_fraction_floordiv(a, b);
    let r = mb_fraction_mod(a, b);
    MbValue::from_ptr(MbObject::new_tuple(vec![q, r]))
}

/// `__pow__(a, n)` — integer exponent only; carve `__pow__(complex)`.
pub fn mb_fraction_pow(a: MbValue, n: MbValue) -> MbValue {
    let base = coerce(a);
    let exp = n.as_int().unwrap_or(0);
    if exp == 0 {
        return make_handle(FractionState { num: 1, den: 1 });
    }
    let abs_exp = exp.unsigned_abs() as u32;
    let mut num = 1_i64;
    let mut den = 1_i64;
    for _ in 0..abs_exp {
        num = num.saturating_mul(base.num);
        den = den.saturating_mul(base.den);
    }
    if exp < 0 {
        std::mem::swap(&mut num, &mut den);
    }
    make_handle(FractionState::new(num, den))
}

// ── Unary

pub fn mb_fraction_pos(handle: MbValue) -> MbValue {
    handle
}
pub fn mb_fraction_neg(handle: MbValue) -> MbValue {
    let s = load(handle);
    make_handle(FractionState { num: -s.num, den: s.den })
}
pub fn mb_fraction_abs(handle: MbValue) -> MbValue {
    let s = load(handle);
    make_handle(FractionState { num: s.num.abs(), den: s.den })
}
pub fn mb_fraction_trunc(handle: MbValue) -> MbValue {
    let s = load(handle);
    // truncate toward zero
    let q = if (s.num >= 0) == (s.den >= 0) {
        s.num.abs() / s.den.abs()
    } else {
        -(s.num.abs() / s.den.abs())
    };
    int_mb(q)
}
pub fn mb_fraction_floor(handle: MbValue) -> MbValue {
    let s = load(handle);
    int_mb(s.num.div_euclid(s.den))
}
pub fn mb_fraction_ceil(handle: MbValue) -> MbValue {
    let s = load(handle);
    let q = s.num.div_euclid(s.den);
    let r = s.num.rem_euclid(s.den);
    int_mb(if r == 0 { q } else { q + 1 })
}
pub fn mb_fraction_round(handle: MbValue, ndigits: MbValue) -> MbValue {
    let s = load(handle);
    let nd = ndigits.as_int().unwrap_or(0);
    if nd == 0 {
        // Banker's rounding to nearest int (half-to-even, like CPython).
        let q = s.num.div_euclid(s.den);
        let r = s.num.rem_euclid(s.den);
        let twice = 2 * r;
        let out = if twice < s.den {
            q
        } else if twice > s.den {
            q + 1
        } else if q % 2 == 0 {
            q
        } else {
            q + 1
        };
        return int_mb(out);
    }
    // ndigits > 0: scale, round, return as Fraction.
    let scale = 10_i64.saturating_pow(nd.unsigned_abs() as u32);
    let scaled = FractionState::new(s.num.saturating_mul(scale), s.den);
    let q = scaled.num.div_euclid(scaled.den);
    let r = scaled.num.rem_euclid(scaled.den);
    let rounded_int = if 2 * r < scaled.den { q }
        else if 2 * r > scaled.den { q + 1 }
        else if q % 2 == 0 { q } else { q + 1 };
    make_handle(FractionState::new(rounded_int, scale))
}

// ── Comparison

fn cmp_states(a: FractionState, b: FractionState) -> std::cmp::Ordering {
    // a.num * b.den ? b.num * a.den (denominators are positive)
    let l = a.num.saturating_mul(b.den);
    let r = b.num.saturating_mul(a.den);
    l.cmp(&r)
}

pub fn mb_fraction_eq(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) == std::cmp::Ordering::Equal)
}
pub fn mb_fraction_ne(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) != std::cmp::Ordering::Equal)
}
/// CPython Fraction ordering returns NotImplemented for partners it does not
/// understand (e.g. a Decimal) so the reflected dunder can run.
fn fraction_orderable(b: MbValue) -> bool {
    !(super::super::builtins::is_decimal_handle_value(b)
        || b.is_none()
        || b.as_ptr().map(|p| unsafe {
            matches!((*p).data, ObjData::Str(_) | ObjData::Complex(..))
        }).unwrap_or(false))
}

pub fn mb_fraction_lt(a: MbValue, b: MbValue) -> MbValue {
    if !fraction_orderable(b) {
        return MbValue::not_implemented();
    }
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) == std::cmp::Ordering::Less)
}
pub fn mb_fraction_le(a: MbValue, b: MbValue) -> MbValue {
    if !fraction_orderable(b) {
        return MbValue::not_implemented();
    }
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) != std::cmp::Ordering::Greater)
}
pub fn mb_fraction_gt(a: MbValue, b: MbValue) -> MbValue {
    if !fraction_orderable(b) {
        return MbValue::not_implemented();
    }
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) == std::cmp::Ordering::Greater)
}
pub fn mb_fraction_ge(a: MbValue, b: MbValue) -> MbValue {
    if !fraction_orderable(b) {
        return MbValue::not_implemented();
    }
    MbValue::from_bool(cmp_states(coerce(a), coerce(b)) != std::cmp::Ordering::Less)
}
pub fn mb_fraction_bool(handle: MbValue) -> MbValue {
    MbValue::from_bool(load(handle).num != 0)
}
pub fn mb_fraction_hash(handle: MbValue) -> MbValue {
    let s = load(handle);
    // Cheap fold for handle-equality semantics; folded into the 48-bit
    // NaN-box window (`from_int` aborts beyond ±2^47 in debug builds).
    // The `hash()` builtin routes equality-consistent hashing through
    // `decimal_mod::mb_numeric_handle_integral_i64` / `_exact_f64` first.
    let h = s.num.wrapping_mul(0x9E3779B97F4A7C15u64 as i64).wrapping_add(s.den);
    MbValue::from_int((h << 17) >> 17)
}

// ── Coercion / serialize

pub fn mb_fraction_int(handle: MbValue) -> MbValue {
    mb_fraction_trunc(handle)
}
pub fn mb_fraction_float(handle: MbValue) -> MbValue {
    let s = load(handle);
    MbValue::from_float(s.num as f64 / s.den as f64)
}
pub fn mb_fraction_str(handle: MbValue) -> MbValue {
    let s = load(handle);
    let repr = if s.den == 1 {
        format!("{}", s.num)
    } else {
        format!("{}/{}", s.num, s.den)
    };
    MbValue::from_ptr(MbObject::new_str(repr))
}
pub fn mb_fraction_repr(handle: MbValue) -> MbValue {
    let s = load(handle);
    MbValue::from_ptr(MbObject::new_str(format!("Fraction({}, {})", s.num, s.den)))
}
pub fn mb_fraction_copy(handle: MbValue) -> MbValue {
    make_handle(load(handle))
}

// ── Flat-args dispatch thunks for module-level entries.

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

dispatch_binary!(dispatch_Fraction, mb_fraction_new);
dispatch_unary!(dispatch_from_float, mb_fraction_from_float);
dispatch_binary!(dispatch_fraction_add, mb_fraction_add);
dispatch_binary!(dispatch_fraction_sub, mb_fraction_sub);
dispatch_binary!(dispatch_fraction_mul, mb_fraction_mul);
dispatch_binary!(dispatch_fraction_truediv, mb_fraction_truediv);
dispatch_binary!(dispatch_fraction_floordiv, mb_fraction_floordiv);
dispatch_binary!(dispatch_fraction_mod, mb_fraction_mod);
dispatch_binary!(dispatch_fraction_divmod, mb_fraction_divmod);
dispatch_binary!(dispatch_fraction_pow, mb_fraction_pow);
dispatch_unary!(dispatch_fraction_neg, mb_fraction_neg);
dispatch_unary!(dispatch_fraction_abs, mb_fraction_abs);
dispatch_unary!(dispatch_fraction_int, mb_fraction_int);
dispatch_unary!(dispatch_fraction_float, mb_fraction_float);
dispatch_unary!(dispatch_fraction_str, mb_fraction_str);
dispatch_unary!(dispatch_fraction_repr, mb_fraction_repr);
dispatch_unary!(dispatch_fraction_numerator, mb_fraction_numerator);
dispatch_unary!(dispatch_fraction_denominator, mb_fraction_denominator);
dispatch_unary!(dispatch_fraction_is_integer, mb_fraction_is_integer);
dispatch_unary!(dispatch_fraction_as_integer_ratio, mb_fraction_as_integer_ratio);
dispatch_binary!(dispatch_fraction_limit_denominator, mb_fraction_limit_denominator);
dispatch_binary!(dispatch_fraction_eq, mb_fraction_eq);
dispatch_binary!(dispatch_fraction_lt, mb_fraction_lt);
dispatch_binary!(dispatch_fraction_le, mb_fraction_le);
dispatch_binary!(dispatch_fraction_gt, mb_fraction_gt);
dispatch_binary!(dispatch_fraction_ge, mb_fraction_ge);

/// Method-name dispatcher used by `class.rs::mb_call_method` for
/// Fraction-handle receivers. Returns `MbValue::none()` if the
/// method name is unknown (caller falls through to int methods).
pub fn dispatch_method(handle: MbValue, method: &str, args: &[MbValue]) -> MbValue {
    let a0 = args.first().copied().unwrap_or_else(MbValue::none);
    let a1 = args.get(1).copied().unwrap_or_else(MbValue::none);
    match method {
        "numerator" => mb_fraction_numerator(handle),
        "denominator" => mb_fraction_denominator(handle),
        "real" => mb_fraction_real(handle),
        "imag" => mb_fraction_imag(handle),
        "conjugate" => mb_fraction_conjugate(handle),
        "is_integer" => mb_fraction_is_integer(handle),
        "as_integer_ratio" => mb_fraction_as_integer_ratio(handle),
        "limit_denominator" => mb_fraction_limit_denominator(handle, a0),
        "__add__" | "__radd__" => mb_fraction_add(handle, a0),
        "__sub__" => mb_fraction_sub(handle, a0),
        "__rsub__" => mb_fraction_rsub(handle, a0),
        "__mul__" | "__rmul__" => mb_fraction_mul(handle, a0),
        "__truediv__" => mb_fraction_truediv(handle, a0),
        "__rtruediv__" => mb_fraction_rtruediv(handle, a0),
        "__floordiv__" => mb_fraction_floordiv(handle, a0),
        "__rfloordiv__" => mb_fraction_rfloordiv(handle, a0),
        "__mod__" => mb_fraction_mod(handle, a0),
        "__rmod__" => mb_fraction_rmod(handle, a0),
        "__divmod__" => mb_fraction_divmod(handle, a0),
        "__pow__" => mb_fraction_pow(handle, a0),
        "__pos__" => mb_fraction_pos(handle),
        "__neg__" => mb_fraction_neg(handle),
        "__abs__" => mb_fraction_abs(handle),
        "__trunc__" => mb_fraction_trunc(handle),
        "__floor__" => mb_fraction_floor(handle),
        "__ceil__" => mb_fraction_ceil(handle),
        "__round__" => mb_fraction_round(handle, a0),
        "__eq__" => mb_fraction_eq(handle, a0),
        "__ne__" => mb_fraction_ne(handle, a0),
        "__lt__" => mb_fraction_lt(handle, a0),
        "__le__" => mb_fraction_le(handle, a0),
        "__gt__" => mb_fraction_gt(handle, a0),
        "__ge__" => mb_fraction_ge(handle, a0),
        "__bool__" => mb_fraction_bool(handle),
        "__hash__" => mb_fraction_hash(handle),
        "__int__" => mb_fraction_int(handle),
        "__float__" => mb_fraction_float(handle),
        "__str__" => mb_fraction_str(handle),
        "__repr__" => mb_fraction_repr(handle),
        "__copy__" | "__deepcopy__" => mb_fraction_copy(handle),
        _ => {
            let _ = a1;
            MbValue::none()
        }
    }
}

// ── Module registration

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Fraction", dispatch_Fraction as usize),
        ("from_float", dispatch_from_float as usize),
        ("fraction_add", dispatch_fraction_add as usize),
        ("fraction_sub", dispatch_fraction_sub as usize),
        ("fraction_mul", dispatch_fraction_mul as usize),
        ("fraction_truediv", dispatch_fraction_truediv as usize),
        ("fraction_floordiv", dispatch_fraction_floordiv as usize),
        ("fraction_mod", dispatch_fraction_mod as usize),
        ("fraction_divmod", dispatch_fraction_divmod as usize),
        ("fraction_pow", dispatch_fraction_pow as usize),
        ("fraction_neg", dispatch_fraction_neg as usize),
        ("fraction_abs", dispatch_fraction_abs as usize),
        ("fraction_int", dispatch_fraction_int as usize),
        ("fraction_float", dispatch_fraction_float as usize),
        ("fraction_str", dispatch_fraction_str as usize),
        ("fraction_repr", dispatch_fraction_repr as usize),
        ("fraction_numerator", dispatch_fraction_numerator as usize),
        ("fraction_denominator", dispatch_fraction_denominator as usize),
        ("fraction_is_integer", dispatch_fraction_is_integer as usize),
        ("fraction_as_integer_ratio", dispatch_fraction_as_integer_ratio as usize),
        ("fraction_limit_denominator", dispatch_fraction_limit_denominator as usize),
        ("fraction_eq", dispatch_fraction_eq as usize),
        ("fraction_lt", dispatch_fraction_lt as usize),
        ("fraction_le", dispatch_fraction_le as usize),
        ("fraction_gt", dispatch_fraction_gt as usize),
        ("fraction_ge", dispatch_fraction_ge as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("fractions", attrs);

    // Bridge the `Fraction` constructor func -> its class name so accessing a
    // registered method on the class (`Fraction.as_integer_ratio`) resolves to a
    // callable unbound method via mb_getattr's func->native-class method bridge
    // (which looks the func addr up in NATIVE_TYPE_NAMES, then lookup_method in
    // the CLASS_REGISTRY table mb_class_register populates below). Without both
    // the mapping and the class registration, `callable(Fraction.is_integer)` is
    // False even though the integer-handle dispatch already implements them.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(dispatch_Fraction as *const () as usize as u64, "Fraction".to_string());
    });

    // Register the Fraction class method table so the class-attribute method
    // bridge above can validate + resolve `Fraction.<method>`. These mirror the
    // concrete callable surface implemented by the integer-handle dispatch
    // (instance receivers route through `dispatch_method`); the stub addresses
    // registered here exist purely so class-attribute access is a real callable
    // unbound method. CPython exposes these as class attributes.
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    let class_methods: Vec<(&str, usize)> = vec![
        ("as_integer_ratio", dispatch_fraction_as_integer_ratio as usize),
        ("is_integer", dispatch_fraction_is_integer as usize),
        ("limit_denominator", dispatch_fraction_limit_denominator as usize),
        ("conjugate", dispatch_fraction_numerator as usize),
        ("numerator", dispatch_fraction_numerator as usize),
        ("denominator", dispatch_fraction_denominator as usize),
        ("from_float", dispatch_from_float as usize),
    ];
    for (name, addr) in &class_methods {
        methods.insert(name.to_string(), MbValue::from_func(*addr));
    }
    super::super::class::mb_class_register("Fraction", Vec::new(), methods);

    // #2111: integer-handle refcount hooks.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// HANDWRITE-END

#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fraction_simplifies_on_construct() {
        let f = mb_fraction_new(MbValue::from_int(6), MbValue::from_int(4));
        let s = load(f);
        assert_eq!(s.num, 3);
        assert_eq!(s.den, 2);
    }

    #[test]
    fn test_fraction_negative_sign_normalizes() {
        let f = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(-3));
        let s = load(f);
        assert_eq!(s.num, -1);
        assert_eq!(s.den, 3);
    }

    #[test]
    fn test_fraction_add_one_third_plus_one_sixth() {
        let a = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(3));
        let b = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(6));
        let s = load(mb_fraction_add(a, b));
        assert_eq!((s.num, s.den), (1, 2));
    }

    #[test]
    fn test_fraction_sub() {
        // 5/6 - 1/3 = 1/2
        let a = mb_fraction_new(MbValue::from_int(5), MbValue::from_int(6));
        let b = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(3));
        let s = load(mb_fraction_sub(a, b));
        assert_eq!((s.num, s.den), (1, 2));
    }

    #[test]
    fn test_fraction_mul() {
        // 2/3 * 3/4 = 1/2
        let a = mb_fraction_new(MbValue::from_int(2), MbValue::from_int(3));
        let b = mb_fraction_new(MbValue::from_int(3), MbValue::from_int(4));
        let s = load(mb_fraction_mul(a, b));
        assert_eq!((s.num, s.den), (1, 2));
    }

    #[test]
    fn test_fraction_truediv() {
        // (1/2) / (3/4) = 2/3
        let a = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(2));
        let b = mb_fraction_new(MbValue::from_int(3), MbValue::from_int(4));
        let s = load(mb_fraction_truediv(a, b));
        assert_eq!((s.num, s.den), (2, 3));
    }

    #[test]
    fn test_fraction_str() {
        let f = mb_fraction_new(MbValue::from_int(7), MbValue::from_int(3));
        let s = mb_fraction_str(f);
        unsafe {
            if let ObjData::Str(ref sv) = (*s.as_ptr().unwrap()).data {
                assert_eq!(sv, "7/3");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_fraction_str_integer_denominator_omitted() {
        // 4/2 normalizes to 2/1 → str = "2"
        let f = mb_fraction_new(MbValue::from_int(4), MbValue::from_int(2));
        let s = mb_fraction_str(f);
        unsafe {
            if let ObjData::Str(ref sv) = (*s.as_ptr().unwrap()).data {
                assert_eq!(sv, "2");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_is_fraction_handle_distinguishes() {
        let f = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(2));
        let id = f.as_int().unwrap() as u64;
        assert!(is_fraction_handle(id));
        // A bare int that was never used as a fraction handle.
        assert!(!is_fraction_handle(99_999_999));
    }

    #[test]
    fn test_fraction_cmp() {
        // 1/2 < 2/3
        let a = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(2));
        let b = mb_fraction_new(MbValue::from_int(2), MbValue::from_int(3));
        assert!(mb_fraction_lt(a, b).as_bool().unwrap_or(false));
        assert!(!mb_fraction_gt(a, b).as_bool().unwrap_or(true));
        assert!(mb_fraction_eq(a, a).as_bool().unwrap_or(false));
    }

    #[test]
    fn test_fraction_neg_abs() {
        let f = mb_fraction_new(MbValue::from_int(-3), MbValue::from_int(4));
        let n = load(mb_fraction_neg(f));
        assert_eq!((n.num, n.den), (3, 4));
        let a = load(mb_fraction_abs(f));
        assert_eq!((a.num, a.den), (3, 4));
    }

    #[test]
    fn test_fraction_floor_ceil_trunc() {
        // 7/3 → floor=2 ceil=3 trunc=2
        let f = mb_fraction_new(MbValue::from_int(7), MbValue::from_int(3));
        assert_eq!(mb_fraction_floor(f).as_int(), Some(2));
        assert_eq!(mb_fraction_ceil(f).as_int(), Some(3));
        assert_eq!(mb_fraction_trunc(f).as_int(), Some(2));
        // -7/3 → floor=-3 ceil=-2 trunc=-2
        let g = mb_fraction_new(MbValue::from_int(-7), MbValue::from_int(3));
        assert_eq!(mb_fraction_floor(g).as_int(), Some(-3));
        assert_eq!(mb_fraction_ceil(g).as_int(), Some(-2));
        assert_eq!(mb_fraction_trunc(g).as_int(), Some(-2));
    }

    #[test]
    fn test_fraction_pow() {
        // (2/3)^3 = 8/27
        let f = mb_fraction_new(MbValue::from_int(2), MbValue::from_int(3));
        let s = load(mb_fraction_pow(f, MbValue::from_int(3)));
        assert_eq!((s.num, s.den), (8, 27));
        // (2/3)^-1 = 3/2
        let r = load(mb_fraction_pow(f, MbValue::from_int(-1)));
        assert_eq!((r.num, r.den), (3, 2));
    }

    #[test]
    fn test_dispatch_method_routes_arith() {
        let a = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(4));
        let b = mb_fraction_new(MbValue::from_int(1), MbValue::from_int(4));
        let s = load(dispatch_method(a, "__add__", &[b]));
        assert_eq!((s.num, s.den), (1, 2));
    }

    #[test]
    fn test_dispatch_method_routes_attrs() {
        let f = mb_fraction_new(MbValue::from_int(7), MbValue::from_int(3));
        assert_eq!(dispatch_method(f, "numerator", &[]).as_int(), Some(7));
        assert_eq!(dispatch_method(f, "denominator", &[]).as_int(), Some(3));
    }
}
