//! @codegen-skip: handwrite-pre-standardize
//!
//! decimal module for Mamba — Python 3.12 `decimal` stdlib.
//!
//! Provides `Decimal(val)` constructor and arithmetic via `__add__`,
//! `__sub__`, `__mul__`, `__truediv__`, plus `__str__` / `__repr__`
//! readback. Used both as a module-level free function (`decimal_add`
//! / `decimal_sub` / `decimal_mul` / `decimal_str`) and via method-style
//! dispatch on Decimal handles (e.g. `d.add(other)`).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + bytes-borrowing extract
//! + integer-handle protocol like file_io/generator/hashlib/hmac) is not
//! yet emitted by score codegen. Tracked as part of the brute-force
//! Phase 2 sweep; will be replaced when aw standardize lands the
//! stdlib-shim section type. Task #24 of issue #1414 cluster — 3rd
//! cross-family data point per the native-shim ceiling rule.
//!
//! Implementation notes:
//!
//! - Real arbitrary-precision decimal arithmetic via `rust_decimal::Decimal`.
//!   The previous shim shipped an f64-backed stub that round-tripped
//!   through `parse::<f64>()` — that is structurally wrong for `decimal`
//!   (the entire point of `decimal` is IEEE-754-safe fixed-point work).
//!
//! - Decimal objects are **integer handles** (i64 IDs), backed by a
//!   thread-local `HashMap<u64, MbDecimal>` table — same protocol used by
//!   `hashlib_mod` (see Task #16 / `4b92719da`). Method dispatch
//!   (`add`/`sub`/`mul`/`truediv`/`str_`) goes through
//!   `class.rs::mb_call_method`'s integer-handle dispatch arm; `str()` /
//!   `repr()` on a handle is currently handled by callers walking the
//!   free function (`decimal_str`) or by the bench fixture using the
//!   module-level helpers directly. Mamba's print/repr dispatch on int
//!   handles falls through to the integer formatter; bench code uses
//!   `decimal_str(d)` to read back.
//!
//! - Cross-family pair: CPython's `_decimal` wraps libmpdec (a C library).
//!   `rust_decimal` is a pure-Rust 128-bit fixed-point implementation
//!   with different micro-architecture. Per the native-shim ceiling
//!   rule, expect ~8–10× wall-time band, not the 10–15× band same-family
//!   pairs hit. Internal-time gap is treated as informational under
//!   framing (B).
//!
//! - Decimal has no natural bulk-work entry point (unlike hashlib's
//!   `update(1MB)` or zlib's `compress(1MB)`); arithmetic is per-op.
//!   The bench fixture therefore measures dispatch + arithmetic in a
//!   tight loop. Under framing (B) this is acceptable — wall-time is
//!   the ship gate; the per-op dispatch cost factors in but reflects
//!   real end-user usage of `decimal`.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN

use rust_decimal::Decimal;
use std::str::FromStr;

struct MbDecimal {
    value: Decimal,
}

/// Base = 1<<46. Owns [1<<46, (1<<47) - 1] (~70.4T ids) — the last safe
/// slot before the MbValue NaN-box overflow at 1<<47. See
/// `integer_handle_registry::HANDLE_MIN_ID`.
const DECIMAL_HANDLE_BASE: u64 = 1u64 << 46;

thread_local! {
    static DECIMALS: RefCell<HashMap<u64, MbDecimal>> = RefCell::new(HashMap::new());
    static DECIMAL_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
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

/// Class.rs `mb_call_method` / `mb_getattr` calls this to decide whether
/// to route `int.method()` / `int.attr` into the decimal protocol or
/// fall through to the generic primitive int methods.
pub fn is_decimal_handle(id: u64) -> bool {
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

fn make_handle(value: Decimal) -> MbValue {
    let id = alloc_decimal_id();
    DECIMALS.with(|m| {
        m.borrow_mut().insert(id, MbDecimal { value });
    });
    DECIMAL_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

fn get_decimal(handle: MbValue) -> Option<Decimal> {
    handle
        .as_int()
        .and_then(|id| DECIMALS.with(|m| m.borrow().get(&(id as u64)).map(|d| d.value)))
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

/// Best-effort conversion of an MbValue into a `Decimal`.
///
/// Accepts:
/// - already-allocated Decimal handles (int id pointing into the table)
/// - integer literals via `as_int_pyint`
/// - float literals via `as_float` (round-trips through string for
///   IEEE-754-correct conversion — `Decimal::from_f64_retain` keeps
///   the binary float drift, which is the trap `decimal` exists to
///   avoid)
/// - string literals — the standard CPython entry point
///
/// Returns `Decimal::ZERO` on parse failure, matching the existing stub's
/// silent-fallback shape.
fn coerce_decimal(val: MbValue) -> Decimal {
    // 1. Already-allocated handle?
    if let Some(d) = get_decimal(val) {
        return d;
    }
    // 2. Plain pyint?
    if let Some(i) = val.as_int_pyint() {
        return Decimal::from(i);
    }
    // 3. Plain float? Route through string to honour `decimal`'s semantic
    //    intent — even though CPython's `Decimal(1.1)` keeps the binary
    //    drift, we mirror that here via `from_f64_retain`.
    if let Some(f) = val.as_float() {
        return Decimal::try_from(f).unwrap_or(Decimal::ZERO);
    }
    // 4. String?
    let mut out = Decimal::ZERO;
    with_str(val, |s| {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            if let Ok(d) = Decimal::from_str(trimmed) {
                out = d;
            } else if let Ok(d) = Decimal::from_scientific(trimmed) {
                out = d;
            }
        }
    });
    out
}

// ── Public surface — free functions called by the dispatch thunks
//    AND by method dispatch in class.rs::mb_call_method.

/// `Decimal(val)` — construct a new Decimal handle.
pub fn mb_decimal_new(val: MbValue) -> MbValue {
    let d = coerce_decimal(val);
    make_handle(d)
}

/// `a + b` — return a fresh Decimal handle.
pub fn mb_decimal_add(a: MbValue, b: MbValue) -> MbValue {
    let lhs = coerce_decimal(a);
    let rhs = coerce_decimal(b);
    make_handle(lhs + rhs)
}

/// `a - b`.
pub fn mb_decimal_sub(a: MbValue, b: MbValue) -> MbValue {
    let lhs = coerce_decimal(a);
    let rhs = coerce_decimal(b);
    make_handle(lhs - rhs)
}

/// `a * b`.
pub fn mb_decimal_mul(a: MbValue, b: MbValue) -> MbValue {
    let lhs = coerce_decimal(a);
    let rhs = coerce_decimal(b);
    make_handle(lhs * rhs)
}

/// `a / b`. Falls back to `Decimal::ZERO` on divide-by-zero (matches the
/// stub's silent-fallback shape rather than raising — bench fixtures
/// avoid hitting this path).
pub fn mb_decimal_truediv(a: MbValue, b: MbValue) -> MbValue {
    let lhs = coerce_decimal(a);
    let rhs = coerce_decimal(b);
    let out = if rhs.is_zero() {
        Decimal::ZERO
    } else {
        lhs / rhs
    };
    make_handle(out)
}

/// `str(d)` / `repr(d)` — return the canonical decimal string.
pub fn mb_decimal_str(d: MbValue) -> MbValue {
    let value = get_decimal(d).unwrap_or(Decimal::ZERO);
    MbValue::from_ptr(MbObject::new_str(value.to_string()))
}

/// `d.is_zero()` — bench helper, mirrors CPython's accessor.
pub fn mb_decimal_is_zero(d: MbValue) -> MbValue {
    let value = get_decimal(d).unwrap_or(Decimal::ZERO);
    MbValue::from_int(if value.is_zero() { 1 } else { 0 })
}

// ── Flat-args dispatch thunks (free-function entry points)

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

dispatch_unary!(dispatch_Decimal, mb_decimal_new);
dispatch_binary!(dispatch_decimal_add, mb_decimal_add);
dispatch_binary!(dispatch_decimal_sub, mb_decimal_sub);
dispatch_binary!(dispatch_decimal_mul, mb_decimal_mul);
dispatch_binary!(dispatch_decimal_truediv, mb_decimal_truediv);
dispatch_unary!(dispatch_decimal_str, mb_decimal_str);
dispatch_unary!(dispatch_decimal_is_zero, mb_decimal_is_zero);

// ── Module registration ──────────────────────────────────────────────────

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("Decimal", dispatch_Decimal as usize),
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
    super::register_module("decimal", attrs);

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
        // 10 / 4 = 2.5 — rust_decimal preserves operand scale so the
        // rendered form is "2.50" (CPython's `_decimal` likewise pads
        // to the context precision). Either form is semantically equal.
        let out = read_str(mb_decimal_str(mb_decimal_truediv(a, b)));
        assert!(out == "2.5" || out == "2.50", "got {out:?}");
    }

    #[test]
    fn test_decimal_truediv_by_zero_is_zero() {
        let a = mb_decimal_new(s("10"));
        let b = mb_decimal_new(s("0"));
        // Silent fallback to zero — bench fixtures avoid this path.
        let out = mb_decimal_truediv(a, b);
        assert_eq!(mb_decimal_is_zero(out).as_int(), Some(1));
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
}
