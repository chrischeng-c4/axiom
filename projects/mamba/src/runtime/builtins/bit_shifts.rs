use super::super::value::MbValue;

/// Left shift `a << b` for dynamically-typed (Any/boxed) integer operands.
/// The static (Int, Int) path is handled inline by codegen; this is the
/// runtime fallback reached when either operand is `Any` (e.g. a list/tuple
/// element or an `enumerate` index), matching `mb_bitand`/`mb_bitor`. Without
/// it, `x << n` on a boxed int silently produced None (no `mb_lshift` was
/// registered, so `binop_to_runtime` returned None and codegen emitted a raw
/// shift over NaN-boxed bits).
pub fn mb_lshift(a: MbValue, b: MbValue) -> MbValue {
    // int-SUBCLASS operand unwrap (#1030) — no other Instance-special-casing
    // here, so this is safe to check first.
    if let Some((na, nb)) = super::int_subclass_numeric_operands(a, b, "__lshift__") {
        return mb_lshift(na, nb);
    }
    let bi = match b.as_int() {
        Some(x) if x >= 0 => x,
        _ => return MbValue::none(),
    };
    // Fast path: inline base whose shift result is recoverable in i64
    // (no bits shifted out). `int_from_i64` still promotes to BigInt when the
    // value exceeds the inline range, so `1 << 48` is exact.
    if let Some(ai) = a.as_int() {
        if bi < 63 {
            let shifted = ai.wrapping_shl(bi as u32);
            if (shifted >> bi) == ai {
                return super::super::bigint_ops::int_from_i64(shifted);
            }
        }
        if ai == 0 {
            return MbValue::from_int(0);
        }
    }
    // General path: arbitrary-precision shift (handles i64 overflow — e.g.
    // `1 << 64` must yield 2**64, not wrap to 1 — and BigInt bases).
    match unsafe { super::super::bigint_ops::to_bigint(a) } {
        Some(big) => super::super::bigint_ops::normalize_bigint(big << (bi as u64)),
        None => MbValue::none(),
    }
}

/// Right shift `a >> b` for dynamically-typed (Any/boxed) integer operands.
/// See `mb_lshift`.
pub fn mb_rshift(a: MbValue, b: MbValue) -> MbValue {
    // int-SUBCLASS operand unwrap (#1030) — no other Instance-special-casing
    // here, so this is safe to check first.
    if let Some((na, nb)) = super::int_subclass_numeric_operands(a, b, "__rshift__") {
        return mb_rshift(na, nb);
    }
    let bi = match b.as_int() {
        Some(x) if x >= 0 => x,
        _ => return MbValue::none(),
    };
    // Fast path: inline base, shift amount within i64's bit width.
    // `wrapping_shr` wraps the shift COUNT modulo 64 in Rust, which is wrong
    // once `bi >= 64` (e.g. `5 >> 64` must be `0`, not `5`) — an inline
    // (small-magnitude) base's arithmetic right shift by >=64 always
    // saturates to 0 (non-negative) or -1 (negative), so handle that
    // directly instead of calling into `wrapping_shr`.
    if let Some(ai) = a.as_int() {
        if bi < 64 {
            return MbValue::from_int(ai.wrapping_shr(bi as u32));
        }
        return MbValue::from_int(if ai < 0 { -1 } else { 0 });
    }
    // General path: BigInt base — arbitrary-precision arithmetic right shift,
    // floor toward -infinity like CPython's `>>` (#1085's
    // `(-9223372036854775808) >> 1` needs this; previously `a.as_int()`
    // returned `None` for the heap BigInt and this fell straight to
    // `MbValue::none()`, whose TAG_NONE bit pattern was then misread as a
    // function object downstream).
    match unsafe { super::super::bigint_ops::to_bigint(a) } {
        Some(big) => super::super::bigint_ops::normalize_bigint(big >> (bi as u64)),
        None => MbValue::none(),
    }
}
