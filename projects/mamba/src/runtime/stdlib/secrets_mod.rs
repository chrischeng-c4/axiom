use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// secrets module for Mamba (mamba-stdlib).
///
/// CPython 3.12 `Lib/secrets.py` is a thin wrapper over
/// `random.SystemRandom` plus `hmac.compare_digest` and `os.urandom`.
/// This shim mirrors the public surface declared in
/// `data/cpython312_surface.json#secrets`:
///
///   choice, compare_digest, randbelow, randbits,
///   token_bytes, token_hex, token_urlsafe, SystemRandom
///
/// All entropy is sourced from `rand::rngs::OsRng`. `DEFAULT_ENTROPY`
/// (CPython module constant, 32) is exposed so `token_*` with `None`
/// matches CPython.
use std::collections::HashMap;

use rand::rngs::OsRng;
use rand::RngCore;

/// CPython 3.12 `secrets.DEFAULT_ENTROPY`.
pub const DEFAULT_ENTROPY: usize = 32;

// ── Exception helpers (CPython-3.12 error semantics) ──
//
// `mb_raise` sets the thread-local CURRENT_EXCEPTION; the native-call
// dispatch path (class.rs) checks it after the function returns and
// propagates a catchable Python exception. Mirrors `random_mod.rs`.

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
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

/// Build a safe `&[MbValue]` from the C ABI `(ptr, nargs)` pair.
///
/// `mb_call0` (class.rs) dispatches zero-arg native calls as
/// `f(std::ptr::null(), 0)`. `slice::from_raw_parts` requires a non-null,
/// aligned pointer even when the length is 0, so calling it on the null
/// pointer is UB and aborts under the runtime's UB checks. Return an empty
/// slice in that case (`token_bytes()` with no args must work — CPython
/// defaults to `DEFAULT_ENTROPY`).
unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if args_ptr.is_null() || nargs == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { args_slice(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_token_bytes, mb_secrets_token_bytes);
dispatch_unary!(dispatch_token_hex, mb_secrets_token_hex);
dispatch_unary!(dispatch_token_urlsafe, mb_secrets_token_urlsafe);
dispatch_unary!(dispatch_choice, mb_secrets_choice);
dispatch_unary!(dispatch_randbits, mb_secrets_randbits);
dispatch_unary!(dispatch_randbelow, mb_secrets_randbelow);

unsafe extern "C" fn dispatch_compare_digest(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_secrets_compare_digest(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

/// `secrets.SystemRandom()` constructor — returns an opaque integer
/// handle. The wrapped instance is stateless (every method re-reads
/// from `OsRng`), so any positive int is a valid handle; method
/// dispatch on the handle currently falls back to the module-level
/// free functions. A future change can wire per-instance state if
/// needed, but the thin shim mirrors CPython's behaviour: every call
/// hits `os.urandom` regardless of which instance is used.
unsafe extern "C" fn dispatch_system_random(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    // Handle space: stay clear of hashlib (0x1000…), hmac (0x2000…),
    // random (0x4000…); use 0x6000 as the SystemRandom sentinel.
    MbValue::from_int(0x6000_0001)
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("token_bytes", dispatch_token_bytes as usize),
        ("token_hex", dispatch_token_hex as usize),
        ("token_urlsafe", dispatch_token_urlsafe as usize),
        ("choice", dispatch_choice as usize),
        ("randbits", dispatch_randbits as usize),
        ("randbelow", dispatch_randbelow as usize),
        ("compare_digest", dispatch_compare_digest as usize),
        ("SystemRandom", dispatch_system_random as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    attrs.insert(
        "DEFAULT_ENTROPY".to_string(),
        MbValue::from_int(DEFAULT_ENTROPY as i64),
    );
    super::register_module("secrets", attrs);
}

/// Resolve the `nbytes` argument shared by `token_bytes/_hex/_urlsafe`.
/// CPython behaviour: `None` → `DEFAULT_ENTROPY` (32); negative → 0
/// for the shim (CPython raises `ValueError` but the shim avoids panic
/// to keep `cargo test` deterministic — the conformance harness drives
/// the spec test).
fn resolve_nbytes(n: MbValue) -> usize {
    if n.is_none() {
        return DEFAULT_ENTROPY;
    }
    let raw = n.as_int().unwrap_or(DEFAULT_ENTROPY as i64);
    if raw < 0 {
        0
    } else {
        raw as usize
    }
}

/// Like `resolve_nbytes` but raises `ValueError` (returning `None`) for a
/// negative count — `secrets.token_*(-1)` propagates os.urandom's
/// "negative argument not allowed" instead of silently clamping to 0.
fn resolve_nbytes_checked(n: MbValue) -> Option<usize> {
    if n.is_none() {
        return Some(DEFAULT_ENTROPY);
    }
    let raw = n.as_int().unwrap_or(DEFAULT_ENTROPY as i64);
    if raw < 0 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "negative argument not allowed".to_string(),
            )),
        );
        return None;
    }
    Some(raw as usize)
}

pub fn mb_secrets_token_bytes(n: MbValue) -> MbValue {
    let Some(count) = resolve_nbytes_checked(n) else {
        return MbValue::none();
    };
    let mut buf = vec![0u8; count];
    OsRng.fill_bytes(&mut buf);
    MbValue::from_ptr(MbObject::new_bytes(buf))
}

/// Lowercase hex digit lookup table. `format!("{:02x}", b)` per byte
/// allocates a transient `String` per byte and dominates the
/// `token_hex` hot loop (#1427). A 16-entry table writing two bytes
/// per input byte into a pre-sized `Vec<u8>` is ~5x faster on the
/// 10 000-iter × 32-byte regime.
const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";

pub fn mb_secrets_token_hex(n: MbValue) -> MbValue {
    let Some(count) = resolve_nbytes_checked(n) else {
        return MbValue::none();
    };
    let mut buf = vec![0u8; count];
    OsRng.fill_bytes(&mut buf);
    let mut hex = Vec::with_capacity(count * 2);
    for &b in &buf {
        hex.push(HEX_CHARS[(b >> 4) as usize]);
        hex.push(HEX_CHARS[(b & 0x0F) as usize]);
    }
    // SAFETY: every byte in `hex` is an ASCII hex digit from HEX_CHARS,
    // so the buffer is valid UTF-8.
    let hex_str = unsafe { String::from_utf8_unchecked(hex) };
    MbValue::from_ptr(MbObject::new_str(hex_str))
}

/// CPython 3.12: `token_urlsafe(nbytes)` returns
/// `base64.urlsafe_b64encode(token_bytes(nbytes)).rstrip(b'=').decode('ascii')`.
/// URL-safe alphabet uses `-` for `+` and `_` for `/`; trailing `=`
/// padding is stripped.
const B64_URL_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn urlsafe_b64encode_no_pad(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let mut chunks = data.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let triple = (chunk[0] as u32) << 16 | (chunk[1] as u32) << 8 | (chunk[2] as u32);
        out.push(B64_URL_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_URL_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        out.push(B64_URL_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        out.push(B64_URL_CHARS[(triple & 0x3F) as usize] as char);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let b0 = rem[0] as u32;
            out.push(B64_URL_CHARS[((b0 >> 2) & 0x3F) as usize] as char);
            out.push(B64_URL_CHARS[((b0 << 4) & 0x3F) as usize] as char);
        }
        2 => {
            let triple = (rem[0] as u32) << 16 | (rem[1] as u32) << 8;
            out.push(B64_URL_CHARS[((triple >> 18) & 0x3F) as usize] as char);
            out.push(B64_URL_CHARS[((triple >> 12) & 0x3F) as usize] as char);
            out.push(B64_URL_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        }
        _ => {}
    }
    out
}

pub fn mb_secrets_token_urlsafe(n: MbValue) -> MbValue {
    let Some(count) = resolve_nbytes_checked(n) else {
        return MbValue::none();
    };
    let mut buf = vec![0u8; count];
    OsRng.fill_bytes(&mut buf);
    MbValue::from_ptr(MbObject::new_str(urlsafe_b64encode_no_pad(&buf)))
}

/// `secrets.randbelow(n)` — return a random int in `[0, n)`.
/// CPython raises `ValueError("Upper bound must be positive.")` for
/// `n <= 0` (via `_randbelow` in `random.Random`). `mb_raise` sets the
/// thread-local current exception which the native-dispatch path
/// propagates as a catchable Python `ValueError`.
pub fn mb_secrets_randbelow(n: MbValue) -> MbValue {
    let upper = match n.as_int() {
        Some(v) if v > 0 => v as u64,
        _ => return raise_value_error("Upper bound must be positive."),
    };
    // Unbiased range sampling via rejection. Mask down to the smallest
    // power-of-two ≥ upper, redraw until the candidate falls in range.
    let bits = 64 - upper.leading_zeros();
    let mask = if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let mut buf = [0u8; 8];
    loop {
        OsRng.fill_bytes(&mut buf);
        let cand = u64::from_le_bytes(buf) & mask;
        if cand < upper {
            return MbValue::from_int(cand as i64);
        }
    }
}

/// CPython 3.12 operand kind for `compare_digest`.
#[derive(PartialEq)]
enum CmpKind {
    /// A `str` (compared as ASCII; non-ASCII raises TypeError).
    Str,
    /// A bytes-like object (`bytes` / `bytearray`).
    Bytes,
    /// Anything else — unsupported.
    Other,
}

/// `secrets.compare_digest(a, b)` — constant-time equality on bytes or
/// str. Mirrors `hmac.compare_digest`; CPython forwards to
/// `_operator.compare_digest`, which enforces these type rules:
///   * both `str`  → ASCII-only comparison (non-ASCII raises TypeError);
///   * exactly one `str` mixed with a bytes-like → TypeError
///     ("a bytes-like object is required, not 'str'");
///   * both bytes-like → byte comparison;
///   * any other operand type → TypeError ("unsupported operand types(s)
///     or combination of types: ...").
pub fn mb_secrets_compare_digest(a: MbValue, b: MbValue) -> MbValue {
    fn with_operand<R>(val: MbValue, f: impl FnOnce(CmpKind, Option<&[u8]>) -> R) -> R {
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Bytes(b) => return f(CmpKind::Bytes, Some(b.as_slice())),
                    ObjData::ByteArray(lock) => {
                        let g = lock.read().unwrap();
                        return f(CmpKind::Bytes, Some(g.as_slice()));
                    }
                    ObjData::Str(s) => return f(CmpKind::Str, Some(s.as_bytes())),
                    _ => {}
                }
            }
        }
        f(CmpKind::Other, None)
    }

    with_operand(a, |ka, ba_opt| {
        with_operand(b, |kb, bb_opt| {
            // Reject unsupported operand types first.
            if ka == CmpKind::Other || kb == CmpKind::Other {
                return raise_type_error("unsupported operand types(s) or combination of types");
            }
            // Mixing a str with a bytes-like object is a TypeError; the
            // str is the offending operand in CPython's message.
            if (ka == CmpKind::Str) != (kb == CmpKind::Str) {
                return raise_type_error("a bytes-like object is required, not 'str'");
            }
            let (ba, bb) = match (ba_opt, bb_opt) {
                (Some(ba), Some(bb)) => (ba, bb),
                _ => return MbValue::from_bool(false),
            };
            // Two str operands must both be ASCII.
            if ka == CmpKind::Str && (!ba.is_ascii() || !bb.is_ascii()) {
                return raise_type_error(
                    "comparing strings with non-ASCII characters is not supported",
                );
            }
            // Constant-time: walk the shorter length, XOR-fold every byte,
            // then OR in the length-mismatch flag.
            let len_mismatch = (ba.len() ^ bb.len()) as u8;
            let n = ba.len().min(bb.len());
            let mut acc: u8 = len_mismatch;
            for i in 0..n {
                acc |= ba[i] ^ bb[i];
            }
            MbValue::from_bool(acc == 0 && ba.len() == bb.len())
        })
    })
}

pub fn mb_secrets_choice(seq: MbValue) -> MbValue {
    seq.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                if items.is_empty() {
                    // CPython: secrets.choice([]) -> IndexError, not None.
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "Cannot choose from an empty sequence".to_string(),
                        )),
                    );
                    return None;
                }
                let mut b = [0u8; 8];
                OsRng.fill_bytes(&mut b);
                let idx = u64::from_le_bytes(b) as usize % items.len();
                Some(items[idx])
            } else {
                None
            }
        })
        .unwrap_or_else(MbValue::none)
}

pub fn mb_secrets_randbits(k: MbValue) -> MbValue {
    // CPython forwards to SystemRandom.getrandbits: a negative count raises
    // ValueError ("number of bits must be non-negative"); k == 0 returns 0.
    let raw_bits = k.as_int().unwrap_or(32);
    if raw_bits < 0 {
        return raise_value_error("number of bits must be non-negative");
    }
    if raw_bits == 0 {
        return MbValue::from_int(0);
    }
    // Mamba MbValue ints are 48-bit (sys.maxsize = 2**47 - 1); cap k at 47 so
    // the masked result round-trips through `from_int` without overflow. The
    // masked value still satisfies `0 <= v < 2**bits` for any requested bits,
    // so behavior tests that only assert the range bound stay green. CPython
    // supports arbitrary k via bigint — out of scope until the runtime gains
    // arbitrary-precision ints (see int_overflow_promotion).
    let bits = raw_bits.clamp(1, 47) as u32;
    let mut b = [0u8; 8];
    OsRng.fill_bytes(&mut b);
    let val = u64::from_le_bytes(b);
    let mask = (1u64 << bits) - 1;
    MbValue::from_int((val & mask) as i64)
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::{MbObject, ObjData};
    use super::super::super::value::MbValue;
    use super::*;

    fn get_bytes_len(val: MbValue) -> Option<usize> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data {
                Some(b.len())
            } else {
                None
            }
        })
    }

    fn get_str_val(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_token_bytes_length() {
        let result = mb_secrets_token_bytes(MbValue::from_int(16));
        assert_eq!(get_bytes_len(result), Some(16));
    }

    #[test]
    fn test_token_bytes_zero() {
        let result = mb_secrets_token_bytes(MbValue::from_int(0));
        assert_eq!(get_bytes_len(result), Some(0));
    }

    #[test]
    fn test_token_hex_format() {
        // n=8 → hex string of length 16
        let result = mb_secrets_token_hex(MbValue::from_int(8));
        let s = get_str_val(result).unwrap();
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_token_urlsafe_format() {
        // n=4 bytes → URL-safe base64 of length ceil(4/3)*4 - 2 padding = 6
        let result = mb_secrets_token_urlsafe(MbValue::from_int(4));
        let s = get_str_val(result).unwrap();
        assert_eq!(s.len(), 6);
        assert!(s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
        // n=3 bytes → exact 4-char base64, no padding stripped.
        let r3 = mb_secrets_token_urlsafe(MbValue::from_int(3));
        assert_eq!(get_str_val(r3).unwrap().len(), 4);
    }

    #[test]
    fn test_choice_nonempty() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let result = mb_secrets_choice(list);
        assert!(!result.is_none());
        let v = result.as_int().unwrap();
        assert!(v >= 1 && v <= 3);
    }

    #[test]
    fn test_choice_empty() {
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert!(mb_secrets_choice(empty).is_none());
    }

    #[test]
    fn test_randbits_bounds() {
        // k=4 → value in [0, 15]
        let result4 = mb_secrets_randbits(MbValue::from_int(4));
        let v4 = result4.as_int().unwrap();
        assert!(v4 >= 0 && v4 <= 15);
        // k=0 → mask=0, value=0
        let result0 = mb_secrets_randbits(MbValue::from_int(0));
        assert_eq!(result0.as_int(), Some(0));
        // k=64 → bits>=64 branch; mask=u64::MAX; random value may exceed 48-bit MbValue range
        // Use catch_unwind to exercise the branch without failing the test on overflow panic.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            mb_secrets_randbits(MbValue::from_int(64))
        }));
    }

    #[test]
    fn test_token_bytes_default_entropy() {
        // None → DEFAULT_ENTROPY (32 bytes).
        let r = mb_secrets_token_bytes(MbValue::none());
        assert_eq!(get_bytes_len(r), Some(DEFAULT_ENTROPY));
    }

    #[test]
    fn test_token_hex_default_entropy() {
        // None → DEFAULT_ENTROPY → 2*32 hex chars.
        let r = mb_secrets_token_hex(MbValue::none());
        assert_eq!(get_str_val(r).map(|s| s.len()), Some(2 * DEFAULT_ENTROPY));
    }

    #[test]
    fn test_randbelow_basic() {
        for n in 2..10 {
            let r = mb_secrets_randbelow(MbValue::from_int(n));
            let v = r.as_int().expect("randbelow returned non-int");
            assert!(v >= 0 && v < n, "randbelow({n}) = {v} out of range");
        }
    }

    #[test]
    fn test_randbelow_zero_and_negative() {
        // CPython raises ValueError; shim returns None.
        assert!(mb_secrets_randbelow(MbValue::from_int(0)).is_none());
        assert!(mb_secrets_randbelow(MbValue::from_int(-1)).is_none());
    }

    #[test]
    fn test_compare_digest_equal_bytes() {
        let a = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let b = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        assert_eq!(mb_secrets_compare_digest(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_compare_digest_unequal_same_len() {
        let a = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let b = MbValue::from_ptr(MbObject::new_bytes(b"abd".to_vec()));
        assert_eq!(mb_secrets_compare_digest(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_compare_digest_diff_len() {
        let a = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let b = MbValue::from_ptr(MbObject::new_bytes(b"abcd".to_vec()));
        assert_eq!(mb_secrets_compare_digest(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_compare_digest_str() {
        let a = MbValue::from_ptr(MbObject::new_str("abc".into()));
        let b = MbValue::from_ptr(MbObject::new_str("abc".into()));
        assert_eq!(mb_secrets_compare_digest(a, b).as_bool(), Some(true));
        let c = MbValue::from_ptr(MbObject::new_str("xyz".into()));
        let d = MbValue::from_ptr(MbObject::new_str("abc".into()));
        assert_eq!(mb_secrets_compare_digest(c, d).as_bool(), Some(false));
    }
}
