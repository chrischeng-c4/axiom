use super::*;

/// Format a byte slice as a Python bytes/bytearray literal body: picks quote,
/// escapes `\`, the chosen quote, `\n`/`\r`/`\t`, and bytes outside 0x20..=0x7E as `\xHH`.
pub(super) fn format_bytes_inner(data: &[u8]) -> String {
    let has_single = data.contains(&b'\'');
    let has_double = data.contains(&b'"');
    let use_double = has_single && !has_double;
    let quote = if use_double { b'"' } else { b'\'' };
    let mut out = String::with_capacity(data.len() + 2);
    out.push(quote as char);
    for &b in data {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c as char);
            }
            0x20..=0x7E => out.push(b as char),
            c => out.push_str(&format!("\\x{c:02x}")),
        }
    }
    out.push(quote as char);
    out
}

#[inline]
pub(crate) fn box_raw_i64_or_bigint(raw: i64) -> MbValue {
    if raw >= -(1i64 << 47) && raw < (1i64 << 47) {
        MbValue::from_int(raw)
    } else {
        crate::runtime::bigint_ops::bigint_from_i128(raw as i128)
    }
}

#[inline]
fn is_registered_func_value(candidate: MbValue) -> bool {
    let Some(addr) = candidate.as_func() else {
        return false;
    };
    crate::runtime::module::is_native_func(addr as u64)
        || crate::runtime::closure::func_params(candidate).is_some()
}

#[inline]
fn is_runtime_ptr_value(candidate: MbValue) -> bool {
    let Some(ptr) = candidate.as_ptr() else {
        return false;
    };
    let addr = ptr as usize;
    addr >= 0x1_0000_0000 && addr % std::mem::align_of::<MbObject>() == 0
}

#[inline]
pub(crate) fn passthrough_boxed_int_candidate(raw: i64, allow_func: bool) -> Option<MbValue> {
    let candidate = MbValue::from_bits(raw as u64);
    if is_runtime_ptr_value(candidate)
        || candidate.is_int()
        || candidate.is_bool()
        || candidate.is_none()
    {
        return Some(candidate);
    }
    if allow_func && is_registered_func_value(candidate) {
        return Some(candidate);
    }
    None
}

/// Box a raw i64 into a NaN-boxed MbValue integer.
/// Used by JIT to convert primitive int results before passing to runtime fns.
///
/// Caller convention: raw values may be inline i64s, out-of-range i64s that
/// must promote to BigInt, or already-boxed runtime values. Preserve the
/// runtime-pointer/int/bool/None tags directly, and preserve TAG_FUNC only
/// when the payload is a registered runtime callable. Pointer passthrough is
/// intentionally narrower than `MbValue::is_ptr()`: the raw integer aliases
/// around `-2**51` have tiny payloads that would otherwise be retained as
/// invalid pointers. This keeps #1084's real-function passthrough alive while
/// rejecting #1136's raw `-2**50` TAG_FUNC and adjacent TAG_PTR aliases.
pub fn mb_box_int(raw: i64) -> MbValue {
    if let Some(out) = passthrough_boxed_int_candidate(raw, true) {
        // Refcount: mb_box_int is classified as NEW (caller owns the
        // result, rc=1 — no post-call retain). When the input is a
        // heap pointer we pass the bits through unchanged, which would
        // violate the NEW contract (caller would release a borrowed
        // reference, causing a double-free when the original owner
        // also releases it — #tuple_return_double_call_unpack UAF).
        // Retain the passthrough pointer so the result is genuinely
        // owned by the caller.
        unsafe {
            crate::runtime::rc::retain_if_ptr(out);
        }
        return out;
    }
    box_raw_i64_or_bigint(raw)
}

/// Box a raw i64 for a **rich comparison** operand only — used solely by
/// `emit_checked_int_compare`'s slow path (#1133). Do not reuse for general
/// re-boxing; that stays `mb_box_int` (e.g. #1084's decorator-return
/// passthrough, which genuinely needs a live TAG_FUNC value to survive).
///
/// `mb_unbox_int_if_boxed` deliberately hands back the *exact* raw i64 of an
/// i64-fitting BigInt (#99), so ordinary comparisons stay value-correct
/// across distinct BigInt allocations. Once `|raw|` exceeds the 48-bit
/// inline range, that raw bit pattern can coincidentally collide with the
/// NaN-boxing prefix at a tag `mb_box_int`'s generic `tag <= 4` check wrongly
/// accepts as "already boxed" — e.g. `-2**50` collides with TAG_FUNC(4),
/// `-2**51` collides with TAG_PTR(0) — corrupting the comparison instead of
/// promoting to BigInt (#1133: a spurious `TypeError` comparing `function`/
/// `int`, or worse).
///
/// A comparison operand is never legitimately a real TAG_FUNC value (nor
/// TAG_NOTIMPLEMENTED/TAG_STOP_ITER/TAG_ELLIPSIS), so the accepted tag set
/// here is enumerated at this consuming site: only genuinely-boxed
/// pointer/int/bool/None values pass through. Uses `MbValue`'s own tag
/// predicates (which already exclude the two reserved canonical-NaN bit
/// patterns) rather than a hand-rolled bit check, so a raw value that
/// happens to alias one of those reserved patterns (e.g. `-2**51`, which is
/// bit-identical to `from_ptr(null)`) is also correctly rejected here and
/// falls through to fresh BigInt promotion.
pub fn mb_box_int_for_compare(raw: i64) -> MbValue {
    if let Some(candidate) = passthrough_boxed_int_candidate(raw, false) {
        // Same refcount contract as `mb_box_int`: retain the passthrough
        // pointer so the result is genuinely owned by the caller.
        unsafe {
            crate::runtime::rc::retain_if_ptr(candidate);
        }
        return candidate;
    }
    box_raw_i64_or_bigint(raw)
}

/// Float power: base ** exp (for JIT use with raw f64 operands).
/// Direct `f64::powf` so JIT-typed float**float doesn't have to box, call
/// `mb_pow`, and unbox the NaN-boxed result. Without this the
/// `(MirBinOp::Pow, Ty::Float)` codegen path fell into the default arm and
/// produced garbage (#1885).
pub fn mb_pow_float(base: f64, exp: f64) -> f64 {
    base.powf(exp)
}

/// Integer power: base ** exp (for JIT use).
/// Returns raw i64 if result fits, or NaN-boxed BigInt bits if it overflows (#833).
pub fn mb_pow_int(base: i64, exp: i64) -> i64 {
    if exp < 0 {
        return 0; // Python returns float for negative exponents; int approx = 0
    }
    // Use BigInt for reliable arbitrary-precision power
    use num_bigint::BigInt;
    let result = BigInt::from(base).pow(exp as u32);
    let fits_inline = result >= BigInt::from(-(1i64 << 47)) && result < BigInt::from(1i64 << 47);
    if fits_inline {
        // Safe to extract as i64
        use num_traits::ToPrimitive;
        result.to_i64().unwrap_or(0)
    } else {
        // Return NaN-boxed BigInt pointer
        crate::runtime::bigint_ops::bigint_from_big(result).to_bits() as i64
    }
}

/// Box a raw i64 (0/1) into a NaN-boxed MbValue bool.
///
/// Idempotent: if `raw` is already a NaN-boxed MbValue (any tag), return it
/// unchanged. Re-boxing an already-boxed value as `MbValue::from_bool(raw != 0)`
/// would always yield `True` because every NaN-boxed value has the NAN_PREFIX
/// high bits set. This shows up when a Bool-typed HIR expression was lowered
/// through a runtime call (e.g. `mb_lt` for Float-Float comparison) and the
/// caller still treats the result as a raw 0/1 needing boxing.
pub fn mb_box_bool(raw: i64) -> MbValue {
    let v = MbValue::from_bits(raw as u64);
    if v.is_bool() {
        return v;
    }
    MbValue::from_bool(raw != 0)
}

/// Box a raw f64 into a NaN-boxed MbValue float.
pub fn mb_box_float(f: f64) -> MbValue {
    MbValue::from_float(f)
}

/// Unbox a NaN-boxed MbValue integer to a raw i64 (#827 nested capture fix).
/// Used when a capture binding from a container element (sequence/mapping/class)
/// must be stored as a primitive i64 so that arithmetic BinOps work correctly.
pub fn mb_unbox_int(val: MbValue) -> i64 {
    val.as_int().unwrap_or(0)
}

/// Unbox a NaN-boxed MbValue bool to a raw i64 (0 or 1) (#827 nested capture fix).
pub fn mb_unbox_bool(val: MbValue) -> i64 {
    val.as_bool().map(|b| b as i64).unwrap_or(0)
}

/// Unbox a NaN-boxed MbValue float to a raw f64 (#827 nested capture fix).
pub fn mb_unbox_float(val: MbValue) -> f64 {
    val.as_float().unwrap_or(0.0)
}

/// Unbox a NaN-boxed int if the bits carry the NAN_INT_PREFIX tag;
/// otherwise treat the input as already-raw and pass it through.
/// Used in entry-body lowering's typed-return path: a typed-int VReg
/// captured from a top-level `f()` call may hold either a raw i64
/// (literal arms) or a boxed MbValue (e.g. IfExpr / getattr return),
/// and the JIT entry caller expects a raw i64.
pub fn mb_unbox_int_if_boxed(val: MbValue) -> i64 {
    if let Some(i) = val.as_int() {
        return i;
    }
    // A BigInt that fits i64 unboxes to its exact value, so raw integer
    // comparisons (==, <, …) value-compare correctly across distinct
    // allocations — without this, `-9223372036854775807 == -9223372036854775807`
    // compared two BigInt pointers and was False (#99). A BigInt larger than
    // i64 cannot be a register int, so fall back to the boxed bits.
    use num_traits::ToPrimitive;
    if let Some(i) =
        unsafe { crate::runtime::bigint_ops::extract_bigint(val) }.and_then(|b| b.to_i64())
    {
        return i;
    }
    val.to_bits() as i64
}

/// Unbox only inline tagged ints; keep raw i64 values and boxed BigInt bits
/// unchanged. Used by the JIT entry-body typed-return path so a top-level
/// `f()` call still returns raw `42`, but an overflowing `f()` preserves its
/// boxed BigInt sentinel instead of collapsing to a plain i64.
pub fn mb_unbox_inline_int_if_boxed(val: MbValue) -> i64 {
    if let Some(i) = val.as_int() {
        i
    } else {
        val.to_bits() as i64
    }
}

/// Unbox a NaN-boxed bool if tagged; otherwise pass through. See
/// `mb_unbox_int_if_boxed` for the entry-body return-path use case.
pub fn mb_unbox_bool_if_boxed(val: MbValue) -> i64 {
    if let Some(b) = val.as_bool() {
        b as i64
    } else {
        val.to_bits() as i64
    }
}

/// Unbox a NaN-boxed float if it is one; otherwise reinterpret the
/// bits as a raw f64. See `mb_unbox_int_if_boxed` for context.
pub fn mb_unbox_float_if_boxed(val: MbValue) -> f64 {
    val.as_float()
        .unwrap_or_else(|| f64::from_bits(val.to_bits()))
}
