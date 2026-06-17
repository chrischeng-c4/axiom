/// BigInt fallback arithmetic for 48-bit NaN-boxed integer overflow (R3).
///
/// Fast path: inline 48-bit integers encoded directly in MbValue NaN-box.
/// Slow path: heap-allocated `num_bigint::BigInt` wrapped in MbObject.
///
/// Overflow detection uses Rust's checked arithmetic (`checked_add`, etc.).
/// When the result exceeds the 48-bit signed range [-(2^47), 2^47-1], the
/// value is promoted to a BigInt heap object returned as an MbValue pointer.
///
/// Interoperability:
/// - Any MbValue that `is_ptr()` with `ObjKind::BigInt` is a big integer.
/// - Comparison and hashing convert both sides to `BigInt` before operating.
/// - Mixed arithmetic (inline + big) promotes the inline operand first.

use num_bigint::BigInt;
use num_traits::{ToPrimitive, Zero};

use super::value::MbValue;
use super::rc::{MbObject, ObjData, ObjKind, mb_retain, mb_release};

/// 48-bit signed integer bounds (inline NaN-box range).
const INT48_MAX: i64 = (1i64 << 47) - 1;
const INT48_MIN: i64 = -(1i64 << 47);

// ── Inline range check ──────────────────────────────────────────────────────

/// Returns `true` if `v` fits in the 48-bit inline integer range.
#[inline]
pub fn fits_inline(v: i64) -> bool {
    v >= INT48_MIN && v <= INT48_MAX
}

// ── Heap BigInt helpers ─────────────────────────────────────────────────────

/// Allocate a new heap BigInt from an `i128` (result of widened arithmetic).
pub fn bigint_from_i128(v: i128) -> MbValue {
    let obj = MbObject::new_bigint(BigInt::from(v));
    MbValue::from_ptr(obj)
}

/// Allocate a new heap BigInt from a `BigInt`.
pub fn bigint_from_big(v: BigInt) -> MbValue {
    let obj = MbObject::new_bigint(v);
    MbValue::from_ptr(obj)
}

/// Extract the `BigInt` from a pointer MbValue.
/// Returns `None` if the value is not a BigInt heap object.
///
/// # Safety
/// `val` must be a valid MbValue whose pointer has not been freed.
pub unsafe fn extract_bigint(val: MbValue) -> Option<BigInt> {
    let ptr = val.as_ptr()?;
    if (*ptr).header.kind != ObjKind::BigInt {
        return None;
    }
    if let ObjData::BigInt(ref big) = (*ptr).data {
        Some(big.clone())
    } else {
        None
    }
}

/// Convert any integer MbValue (inline or BigInt heap) to a `BigInt`.
///
/// # Safety
/// `val` must be a valid MbValue.
pub unsafe fn to_bigint(val: MbValue) -> Option<BigInt> {
    if let Some(i) = val.as_int() {
        return Some(BigInt::from(i));
    }
    extract_bigint(val)
}

// ── Overflow-checked arithmetic ─────────────────────────────────────────────

/// Add two integer MbValues, promoting to BigInt on overflow.
///
/// # Safety
/// Both `a` and `b` must be valid integer MbValues (inline or BigInt).
pub unsafe fn mb_int_add(a: MbValue, b: MbValue) -> MbValue {
    // Fast path: both inline.
    if let (Some(ia), Some(ib)) = (a.as_int(), b.as_int()) {
        if let Some(result) = ia.checked_add(ib) {
            if fits_inline(result) {
                return MbValue::from_int(result);
            }
            // Result too large for inline — widen via i128.
            return bigint_from_i128(ia as i128 + ib as i128);
        }
        // i64 overflow — go via i128.
        return bigint_from_i128(ia as i128 + ib as i128);
    }
    // Slow path: at least one BigInt.
    let ba = to_bigint(a).unwrap_or_else(BigInt::zero);
    let bb = to_bigint(b).unwrap_or_else(BigInt::zero);
    normalize_bigint(ba + bb)
}

/// Subtract two integer MbValues, promoting to BigInt on overflow.
///
/// # Safety
/// Both `a` and `b` must be valid integer MbValues.
pub unsafe fn mb_int_sub(a: MbValue, b: MbValue) -> MbValue {
    if let (Some(ia), Some(ib)) = (a.as_int(), b.as_int()) {
        if let Some(result) = ia.checked_sub(ib) {
            if fits_inline(result) {
                return MbValue::from_int(result);
            }
            return bigint_from_i128(ia as i128 - ib as i128);
        }
        return bigint_from_i128(ia as i128 - ib as i128);
    }
    let ba = to_bigint(a).unwrap_or_else(BigInt::zero);
    let bb = to_bigint(b).unwrap_or_else(BigInt::zero);
    normalize_bigint(ba - bb)
}

/// Multiply two integer MbValues, promoting to BigInt on overflow.
///
/// # Safety
/// Both `a` and `b` must be valid integer MbValues.
pub unsafe fn mb_int_mul(a: MbValue, b: MbValue) -> MbValue {
    if let (Some(ia), Some(ib)) = (a.as_int(), b.as_int()) {
        if let Some(result) = ia.checked_mul(ib) {
            if fits_inline(result) {
                return MbValue::from_int(result);
            }
        }
        // Widen to i128 for intermediate then BigInt if needed.
        let wide = ia as i128 * ib as i128;
        if wide >= INT48_MIN as i128 && wide <= INT48_MAX as i128 {
            return MbValue::from_int(wide as i64);
        }
        return bigint_from_i128(wide);
    }
    let ba = to_bigint(a).unwrap_or_else(BigInt::zero);
    let bb = to_bigint(b).unwrap_or_else(BigInt::zero);
    normalize_bigint(ba * bb)
}

// ── Floor division / modulo / divmod / pow ──────────────────────────────────

/// Floor-division quotient and remainder for big integers, with Python sign
/// semantics: the quotient rounds toward −∞ and the remainder takes the
/// divisor's sign. `b` must be non-zero.
fn floor_div_mod(a: &BigInt, b: &BigInt) -> (BigInt, BigInt) {
    let q = a / b;       // truncates toward zero
    let r = a - &q * b;  // truncated remainder (sign of `a`)
    // Step the quotient toward −∞ when the remainder's sign disagrees with the
    // divisor (matches CPython `//` / `%`).
    if r.sign() != num_bigint::Sign::NoSign && r.sign() != b.sign() {
        (q - 1, r + b)
    } else {
        (q, r)
    }
}

/// Python floor division for integer MbValues (inline or BigInt). Returns
/// `None` when the divisor is zero — the caller raises ZeroDivisionError.
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_floordiv(a: MbValue, b: MbValue) -> Option<MbValue> {
    let ba = to_bigint(a)?;
    let bb = to_bigint(b)?;
    if bb.is_zero() {
        return None;
    }
    Some(normalize_bigint(floor_div_mod(&ba, &bb).0))
}

/// Python modulo for integer MbValues (result takes the divisor's sign).
/// Returns `None` when the divisor is zero.
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_mod(a: MbValue, b: MbValue) -> Option<MbValue> {
    let ba = to_bigint(a)?;
    let bb = to_bigint(b)?;
    if bb.is_zero() {
        return None;
    }
    Some(normalize_bigint(floor_div_mod(&ba, &bb).1))
}

/// Python `divmod` for integer MbValues. Returns `None` when the divisor is
/// zero.
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_divmod(a: MbValue, b: MbValue) -> Option<(MbValue, MbValue)> {
    let ba = to_bigint(a)?;
    let bb = to_bigint(b)?;
    if bb.is_zero() {
        return None;
    }
    let (q, r) = floor_div_mod(&ba, &bb);
    Some((normalize_bigint(q), normalize_bigint(r)))
}

/// Integer exponentiation for non-negative exponents. Returns `None` when the
/// exponent is negative (the caller produces a float result) or too large to
/// materialize (> `u32::MAX`, which is astronomically unlikely in real code).
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_pow(base: MbValue, exp: MbValue) -> Option<MbValue> {
    let be = to_bigint(exp)?;
    if be.sign() == num_bigint::Sign::Minus {
        return None;
    }
    let e = be.to_u32()?;
    let bb = to_bigint(base)?;
    Some(normalize_bigint(bb.pow(e)))
}

/// Convert an integer MbValue (inline or BigInt) to `f64`, saturating to ±inf
/// for magnitudes beyond the f64 range (CPython int→float widening). Returns
/// `None` for non-integers.
///
/// # Safety
/// `val` must be a valid MbValue.
pub unsafe fn int_as_f64(val: MbValue) -> Option<f64> {
    if let Some(i) = val.as_int() {
        return Some(i as f64);
    }
    let big = extract_bigint(val)?;
    Some(big.to_f64().unwrap_or_else(|| {
        if big.sign() == num_bigint::Sign::Minus {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    }))
}

/// Make an int MbValue from an i64 — inline when it fits, heap BigInt otherwise.
pub fn int_from_i64(v: i64) -> MbValue {
    if fits_inline(v) {
        MbValue::from_int(v)
    } else {
        bigint_from_i128(v as i128)
    }
}

/// Make an int MbValue from a finite f64, truncating toward zero.
/// Exact for every finite f64 magnitude via the BigInt fallback.
/// Callers must reject NaN/infinity first (CPython raises there).
pub fn int_from_f64_trunc(f: f64) -> MbValue {
    if f >= INT48_MIN as f64 && f <= INT48_MAX as f64 {
        return MbValue::from_int(f as i64);
    }
    use num_traits::FromPrimitive;
    match BigInt::from_f64(f.trunc()) {
        Some(b) => normalize_bigint(b),
        None => MbValue::from_int(0),
    }
}

/// Normalize a BigInt result: if it fits inline, return an inline MbValue.
pub fn normalize_bigint(v: BigInt) -> MbValue {
    if let Some(small) = v.to_i64() {
        if fits_inline(small) {
            return MbValue::from_int(small);
        }
    }
    bigint_from_big(v)
}

// ── Comparison ───────────────────────────────────────────────────────────────

/// Compare two integer MbValues (both inline or BigInt).
/// Returns negative/zero/positive like `Ord::cmp`.
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_cmp(a: MbValue, b: MbValue) -> std::cmp::Ordering {
    match (a.as_int(), b.as_int()) {
        (Some(ia), Some(ib)) => ia.cmp(&ib),
        _ => {
            let ba = to_bigint(a).unwrap_or_else(BigInt::zero);
            let bb = to_bigint(b).unwrap_or_else(BigInt::zero);
            ba.cmp(&bb)
        }
    }
}

/// Equality check for integer MbValues (handles inline/BigInt mix).
///
/// # Safety
/// Both must be valid integer MbValues.
pub unsafe fn mb_int_eq(a: MbValue, b: MbValue) -> bool {
    mb_int_cmp(a, b) == std::cmp::Ordering::Equal
}

// ── Hashing ──────────────────────────────────────────────────────────────────

/// Compute a Python-compatible hash for an integer MbValue.
/// Inline integers: hash is the value itself (mod Python hash modulus).
/// BigInt: reduce modulo `sys.hash_info.modulus` (2^61 - 1 on 64-bit).
///
/// # Safety
/// `val` must be a valid integer MbValue.
pub unsafe fn mb_int_hash(val: MbValue) -> i64 {
    // Python hash modulus for integers on 64-bit: 2^61 - 1
    const HASH_MODULUS: i64 = (1i64 << 61) - 1;
    if let Some(i) = val.as_int() {
        // Small integers hash to themselves (Python semantics).
        return i;
    }
    if let Some(big) = extract_bigint(val) {
        // Reduce magnitude modulo HASH_MODULUS, preserve sign.
        if let Some(small) = big.to_i64() {
            return small % HASH_MODULUS;
        }
        // For very large values, use the low 61 bits with sign.
        let low: i64 = big.iter_u64_digits()
            .next()
            .map(|d| (d & (HASH_MODULUS as u64)) as i64)
            .unwrap_or(0);
        return if big < BigInt::zero() { -low } else { low };
    }
    0
}

// ── ABI helpers (callable from JIT-generated code) ──────────────────────────

/// Convert a raw register value to MbValue.
///
/// After CheckedAdd/Sub/Mul, a register may hold either:
/// - A raw i64 (small int, from inline unbox path) → box as inline int
/// - NaN-boxed BigInt pointer bits (from overflow path) → use directly
///
/// Distinction: NaN-boxed values have NaN prefix AND tag ∈ {0,1,2,3}.
/// Raw negative i64 values also have the NaN prefix pattern, but their
/// tag bits (48-50) are 7 (not a valid NaN-box tag).
fn reg_to_mbvalue(bits: u64) -> MbValue {
    const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
    if bits & NAN_PREFIX == NAN_PREFIX {
        let tag = (bits >> 48) & 7;
        if tag <= 3 {
            // Valid NaN-boxed value (inline int, BigInt pointer, bool, or None)
            return MbValue::from_bits(bits);
        }
    }
    // Raw i64 — box as inline int
    let raw = bits as i64;
    if fits_inline(raw) {
        MbValue::from_int(raw)
    } else {
        bigint_from_i128(raw as i128)
    }
}

/// JIT extern: add two integer register values, returning raw i64 result.
/// Inputs may be raw i64 or NaN-boxed BigInt from prior overflow.
///
/// Fast path: if both inputs are raw i64 (no NaN-box tag), use checked_add
/// and return raw i64. This avoids reg_to_mbvalue + boxing overhead (~60ns)
/// for the common case (no overflow, no BigInt).
#[no_mangle]
pub extern "C" fn mb_bigint_add(a_bits: u64, b_bits: u64) -> u64 {
    // Fast path: both are raw i64 (MSBs not set to NaN-box prefix)
    const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
    if (a_bits & NAN_PREFIX != NAN_PREFIX) && (b_bits & NAN_PREFIX != NAN_PREFIX) {
        let a = a_bits as i64;
        let b = b_bits as i64;
        if let Some(result) = a.checked_add(b) {
            // Stay on raw fast path only if the result still fits in the
            // 48-bit inline-int range. Otherwise the value would silently
            // wrap when the caller re-NaN-boxes it (#1212 §5b).
            if fits_inline(result) {
                return result as u64;
            }
        }
        // Overflow (i64 or INT48) → fall through to BigInt path.
    }
    let a = reg_to_mbvalue(a_bits);
    let b = reg_to_mbvalue(b_bits);
    unsafe { mb_int_add(a, b) }.to_bits()
}

/// JIT extern: subtract two integer register values.
#[no_mangle]
pub extern "C" fn mb_bigint_sub(a_bits: u64, b_bits: u64) -> u64 {
    const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
    if (a_bits & NAN_PREFIX != NAN_PREFIX) && (b_bits & NAN_PREFIX != NAN_PREFIX) {
        let a = a_bits as i64;
        let b = b_bits as i64;
        if let Some(result) = a.checked_sub(b) {
            if fits_inline(result) {
                return result as u64;
            }
        }
    }
    let a = reg_to_mbvalue(a_bits);
    let b = reg_to_mbvalue(b_bits);
    unsafe { mb_int_sub(a, b) }.to_bits()
}

/// JIT extern: multiply two integer register values.
#[no_mangle]
pub extern "C" fn mb_bigint_mul(a_bits: u64, b_bits: u64) -> u64 {
    const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
    if (a_bits & NAN_PREFIX != NAN_PREFIX) && (b_bits & NAN_PREFIX != NAN_PREFIX) {
        let a = a_bits as i64;
        let b = b_bits as i64;
        if let Some(result) = a.checked_mul(b) {
            if fits_inline(result) {
                return result as u64;
            }
        }
    }
    let a = reg_to_mbvalue(a_bits);
    let b = reg_to_mbvalue(b_bits);
    unsafe { mb_int_mul(a, b) }.to_bits()
}

/// JIT extern: compare two integer MbValues.
/// Returns -1, 0, or 1 as i64.
#[no_mangle]
pub extern "C" fn mb_bigint_cmp(a_bits: u64, b_bits: u64) -> i64 {
    let a = MbValue::from_bits(a_bits);
    let b = MbValue::from_bits(b_bits);
    unsafe {
        match mb_int_cmp(a, b) {
            std::cmp::Ordering::Less    => -1,
            std::cmp::Ordering::Equal   =>  0,
            std::cmp::Ordering::Greater =>  1,
        }
    }
}

/// JIT extern: check two integer MbValues for equality.
/// Returns 1 (equal) or 0 (not equal).
#[no_mangle]
pub extern "C" fn mb_bigint_eq(a_bits: u64, b_bits: u64) -> i64 {
    let a = MbValue::from_bits(a_bits);
    let b = MbValue::from_bits(b_bits);
    unsafe { mb_int_eq(a, b) as i64 }
}

/// JIT extern: hash an integer MbValue.
#[no_mangle]
pub extern "C" fn mb_bigint_hash(val_bits: u64) -> i64 {
    let val = MbValue::from_bits(val_bits);
    unsafe { mb_int_hash(val) }
}

/// JIT extern: retain a BigInt heap object.
#[no_mangle]
pub extern "C" fn mb_bigint_retain(val_bits: u64) {
    let val = MbValue::from_bits(val_bits);
    if let Some(ptr) = val.as_ptr() {
        unsafe { mb_retain(ptr) };
    }
}

/// JIT extern: release a BigInt heap object.
#[no_mangle]
pub extern "C" fn mb_bigint_release(val_bits: u64) {
    let val = MbValue::from_bits(val_bits);
    if let Some(ptr) = val.as_ptr() {
        unsafe { mb_release(ptr) };
    }
}

/// JIT extern: create a heap BigInt from a signed 64-bit value.
/// Returns the NaN-boxed MbValue bits (pointer to heap BigInt).
/// Signature: `fn(v: i64) -> u64`
#[no_mangle]
pub extern "C" fn mb_bigint_from_i64(v: i64) -> u64 {
    bigint_from_i128(v as i128).to_bits()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn inline(i: i64) -> MbValue { MbValue::from_int(i) }

    #[test]
    fn test_fits_inline_boundary() {
        assert!(fits_inline(0));
        assert!(fits_inline(INT48_MAX));
        assert!(fits_inline(INT48_MIN));
        assert!(!fits_inline(INT48_MAX + 1));
        assert!(!fits_inline(INT48_MIN - 1));
        assert!(!fits_inline(i64::MAX));
        assert!(!fits_inline(i64::MIN));
    }

    #[test]
    fn test_add_no_overflow() {
        unsafe {
            let r = mb_int_add(inline(1), inline(2));
            assert_eq!(r.as_int(), Some(3));
        }
    }

    #[test]
    fn test_add_overflow_promotes_to_bigint() {
        unsafe {
            let max = MbValue::from_int(INT48_MAX);
            let one = MbValue::from_int(1);
            let r = mb_int_add(max, one);
            // Must be a BigInt heap pointer, not inline
            assert!(r.is_ptr(), "expected BigInt pointer on overflow");
            let big = extract_bigint(r).expect("should be BigInt");
            assert_eq!(big, BigInt::from(INT48_MAX) + BigInt::from(1));
            // Cleanup
            if let Some(ptr) = r.as_ptr() { mb_release(ptr); }
        }
    }

    #[test]
    fn test_sub_overflow() {
        unsafe {
            let min = MbValue::from_int(INT48_MIN);
            let one = MbValue::from_int(1);
            let r = mb_int_sub(min, one);
            assert!(r.is_ptr());
            let big = extract_bigint(r).expect("should be BigInt");
            assert_eq!(big, BigInt::from(INT48_MIN) - BigInt::from(1));
            if let Some(ptr) = r.as_ptr() { mb_release(ptr); }
        }
    }

    #[test]
    fn test_mul_overflow() {
        unsafe {
            let large = MbValue::from_int(1_000_000_000i64);
            let r = mb_int_mul(large, large);
            // 1e18 exceeds INT48_MAX (~1.4e14) → BigInt
            assert!(r.is_ptr());
            let big = extract_bigint(r).expect("BigInt");
            assert_eq!(big, BigInt::from(1_000_000_000_000_000_000i64));
            if let Some(ptr) = r.as_ptr() { mb_release(ptr); }
        }
    }

    #[test]
    fn test_mul_no_overflow() {
        unsafe {
            let r = mb_int_mul(inline(3), inline(7));
            assert_eq!(r.as_int(), Some(21));
        }
    }

    #[test]
    fn test_bigint_add_bigint() {
        unsafe {
            let big_a = MbObject::new_bigint(BigInt::from(INT48_MAX) + BigInt::from(100));
            let big_b = MbObject::new_bigint(BigInt::from(INT48_MAX) + BigInt::from(200));
            let va = MbValue::from_ptr(big_a);
            let vb = MbValue::from_ptr(big_b);
            let r = mb_int_add(va, vb);
            let expected = (BigInt::from(INT48_MAX) + 100) + (BigInt::from(INT48_MAX) + 200);
            assert_eq!(extract_bigint(r).unwrap(), expected);
            mb_release(big_a);
            mb_release(big_b);
            if let Some(ptr) = r.as_ptr() { mb_release(ptr); }
        }
    }

    #[test]
    fn test_normalize_back_to_inline() {
        unsafe {
            // Big - Big that fits back in inline range
            let big_a = MbObject::new_bigint(BigInt::from(INT48_MAX) + BigInt::from(1));
            let big_b = MbObject::new_bigint(BigInt::from(1));
            let va = MbValue::from_ptr(big_a);
            let vb = MbValue::from_ptr(big_b);
            let r = mb_int_sub(va, vb);
            // (INT48_MAX + 1) - 1 = INT48_MAX, fits inline
            assert_eq!(r.as_int(), Some(INT48_MAX));
            mb_release(big_a);
            mb_release(big_b);
        }
    }

    #[test]
    fn test_cmp_inline() {
        unsafe {
            assert_eq!(mb_int_cmp(inline(3), inline(5)), std::cmp::Ordering::Less);
            assert_eq!(mb_int_cmp(inline(5), inline(5)), std::cmp::Ordering::Equal);
            assert_eq!(mb_int_cmp(inline(7), inline(5)), std::cmp::Ordering::Greater);
        }
    }

    #[test]
    fn test_eq_inline() {
        unsafe {
            assert!(mb_int_eq(inline(42), inline(42)));
            assert!(!mb_int_eq(inline(42), inline(43)));
        }
    }

    #[test]
    fn test_hash_inline() {
        unsafe {
            assert_eq!(mb_int_hash(inline(0)), 0);
            assert_eq!(mb_int_hash(inline(42)), 42);
            assert_eq!(mb_int_hash(inline(-1)), -1);
        }
    }

    #[test]
    fn test_hash_bigint() {
        unsafe {
            let obj = MbObject::new_bigint(BigInt::from(INT48_MAX) + BigInt::from(1));
            let v = MbValue::from_ptr(obj);
            let h = mb_int_hash(v);
            // Must not panic; actual value is deterministic
            let _ = h;
            mb_release(obj);
        }
    }

    #[test]
    fn test_abi_add() {
        let a = MbValue::from_int(10).to_bits();
        let b = MbValue::from_int(20).to_bits();
        let r = MbValue::from_bits(mb_bigint_add(a, b));
        assert_eq!(r.as_int(), Some(30));
    }

    #[test]
    fn test_abi_sub() {
        let a = MbValue::from_int(50).to_bits();
        let b = MbValue::from_int(20).to_bits();
        let r = MbValue::from_bits(mb_bigint_sub(a, b));
        assert_eq!(r.as_int(), Some(30));
    }

    #[test]
    fn test_abi_mul() {
        let a = MbValue::from_int(6).to_bits();
        let b = MbValue::from_int(7).to_bits();
        let r = MbValue::from_bits(mb_bigint_mul(a, b));
        assert_eq!(r.as_int(), Some(42));
    }

    #[test]
    fn test_abi_cmp() {
        let a = MbValue::from_int(3).to_bits();
        let b = MbValue::from_int(5).to_bits();
        assert_eq!(mb_bigint_cmp(a, b), -1);
        assert_eq!(mb_bigint_cmp(b, a),  1);
        assert_eq!(mb_bigint_cmp(a, a),  0);
    }

    #[test]
    fn test_abi_eq() {
        let a = MbValue::from_int(99).to_bits();
        let b = MbValue::from_int(99).to_bits();
        let c = MbValue::from_int(100).to_bits();
        assert_eq!(mb_bigint_eq(a, b), 1);
        assert_eq!(mb_bigint_eq(a, c), 0);
    }
}
