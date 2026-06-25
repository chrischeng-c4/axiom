/// NaN-boxed value representation (#279).
///
/// Uses the NaN-boxing technique from LuaJIT/SpiderMonkey to pack all Mamba
/// values into a single 64-bit word:
///
/// - Floats: any f64 that is NOT a signaling NaN
/// - Tagged values: encoded in the NaN payload bits
///
/// Layout (IEEE 754 double):
///   [sign:1][exponent:11][mantissa:52]
///
/// A quiet NaN has exponent=0x7FF, bit 51 set. We use sign=1 + exponent=0x7FF
/// + bit 51 set as our tag prefix, leaving 51 bits for payload:
///
///   1_11111111111_1_TTT_PPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPPP
///   ^             ^ ^^^  48 bits of payload
///   sign=1     qNaN tag (3 bits)
///
/// Tags (3 bits):
///   000 = pointer to heap object (48-bit pointer)
///   001 = integer (48-bit signed, covers ±2^47)
///   010 = bool (payload: 0 or 1)
///   011 = None
///   100 = function pointer (48-bit code address)
use super::rc::MbObject;

/// Tag bits within the NaN payload.
const TAG_PTR: u64 = 0;
const TAG_INT: u64 = 1;
const TAG_BOOL: u64 = 2;
const TAG_NONE: u64 = 3;
/// Function pointer — stores a 48-bit code address (JIT or extern).
const TAG_FUNC: u64 = 4;
/// The `NotImplemented` singleton — returned from rich comparison dunders
/// to signal that the reflected operation should be tried.
const TAG_NOTIMPLEMENTED: u64 = 5;
/// StopIteration sentinel — returned by `mb_next_or_stop` when an iterator
/// is exhausted. Distinct from `None` (which is a valid yielded value) and
/// from any other tagged MbValue. Internal-only; never visible to user code.
const TAG_STOP_ITER: u64 = 6;
/// The `Ellipsis` singleton (`...`) — a real interned value so that
/// `Ellipsis is Ellipsis` holds, `repr(...)` renders `Ellipsis`, and
/// `type(...)` reports `ellipsis`.
const TAG_ELLIPSIS: u64 = 7;

/// The NaN prefix: sign=1, exponent=0x7FF, quiet bit=1 → bits 63..51
const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;

/// Negative canonical quiet NaN (sign=1) — the one sign-carrying NaN bit
/// pattern we treat as a genuine float rather than a tagged value, so a
/// negative NaN (e.g. `complex("-nan").real`, whose sign CPython preserves)
/// survives boxing instead of collapsing to the positive canonical NaN.
/// It is bit-identical to `from_ptr(null)` (TAG_PTR, payload 0), but no code
/// ever boxes a null pointer, so reclaiming this slot for a float is safe.
const NEG_CANON_NAN: u64 = NAN_PREFIX;

/// Mask for the tag field (bits 48..50)
const TAG_MASK: u64 = 0x0007_0000_0000_0000;
const TAG_SHIFT: u32 = 48;

/// Mask for the 48-bit payload
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF;

/// A NaN-boxed Mamba value — always 64 bits.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MbValue(u64);

impl MbValue {
    /// Create a float value. If the float happens to be a NaN, we canonicalize it.
    pub fn from_float(f: f64) -> Self {
        let bits = f.to_bits();
        // Check if this is one of our tagged NaNs (has our prefix)
        if bits & NAN_PREFIX == NAN_PREFIX && bits != f64::NAN.to_bits() {
            // A negative NaN canonicalizes to the one sign-carrying NaN slot
            // we keep distinguishable (`NEG_CANON_NAN`), preserving its sign
            // bit (e.g. `complex("-nan").real`); every other tagged-prefix NaN
            // collapses to the standard positive quiet NaN.
            if f.is_nan() && f.is_sign_negative() {
                Self(NEG_CANON_NAN)
            } else {
                Self(f64::NAN.to_bits())
            }
        } else {
            Self(bits)
        }
    }

    /// Create an integer value. Integers are stored as 48-bit signed values.
    /// Panics if the value doesn't fit in 48 bits.
    pub fn from_int(i: i64) -> Self {
        debug_assert!(
            i >= -(1i64 << 47) && i < (1i64 << 47),
            "integer {i} out of 48-bit range"
        );
        let payload = (i as u64) & PAYLOAD_MASK;
        Self(NAN_PREFIX | (TAG_INT << TAG_SHIFT) | payload)
    }

    /// Create a boolean value.
    pub fn from_bool(b: bool) -> Self {
        Self(NAN_PREFIX | (TAG_BOOL << TAG_SHIFT) | (b as u64))
    }

    /// The singleton None value.
    pub fn none() -> Self {
        Self(NAN_PREFIX | (TAG_NONE << TAG_SHIFT))
    }

    /// The singleton `NotImplemented` value.
    pub fn not_implemented() -> Self {
        Self(NAN_PREFIX | (TAG_NOTIMPLEMENTED << TAG_SHIFT))
    }

    /// The singleton `Ellipsis` value (`...`).
    pub fn ellipsis() -> Self {
        Self(NAN_PREFIX | (TAG_ELLIPSIS << TAG_SHIFT))
    }

    /// The StopIteration sentinel — returned by `mb_next_or_stop` to signal
    /// iterator exhaustion. Internal use only; must never reach user code.
    #[inline(always)]
    pub fn stop_iter_sentinel() -> Self {
        Self(NAN_PREFIX | (TAG_STOP_ITER << TAG_SHIFT))
    }

    /// Identity check for the StopIteration sentinel — pure bit-equality.
    #[inline(always)]
    pub fn is_stop_iter_sentinel(self) -> bool {
        self.0 == NAN_PREFIX | (TAG_STOP_ITER << TAG_SHIFT)
    }

    /// Create a pointer value to a heap-allocated object.
    /// The pointer must fit in 48 bits (standard on x86-64/ARM64).
    pub fn from_ptr(ptr: *mut MbObject) -> Self {
        let addr = ptr as u64;
        debug_assert!(addr & !PAYLOAD_MASK == 0, "pointer exceeds 48 bits");
        Self(NAN_PREFIX | (TAG_PTR << TAG_SHIFT) | (addr & PAYLOAD_MASK))
    }

    // ── Type queries ──

    pub fn is_float(self) -> bool {
        // A value is a float if it does NOT have our NaN prefix, or it is one
        // of the two canonical-NaN slots (positive `f64::NAN`, or the
        // sign-carrying `NEG_CANON_NAN`) we reserve as genuine floats.
        (self.0 & NAN_PREFIX) != NAN_PREFIX
            || self.0 == f64::NAN.to_bits()
            || self.0 == NEG_CANON_NAN
    }

    pub fn is_int(self) -> bool {
        self.tag() == Some(TAG_INT)
    }

    pub fn is_bool(self) -> bool {
        self.tag() == Some(TAG_BOOL)
    }

    pub fn is_none(self) -> bool {
        self.tag() == Some(TAG_NONE)
    }

    pub fn is_not_implemented(self) -> bool {
        self.tag() == Some(TAG_NOTIMPLEMENTED)
    }

    pub fn is_ellipsis(self) -> bool {
        self.tag() == Some(TAG_ELLIPSIS)
    }

    pub fn is_ptr(self) -> bool {
        self.tag() == Some(TAG_PTR)
    }

    fn tag(self) -> Option<u64> {
        if (self.0 & NAN_PREFIX) == NAN_PREFIX
            && self.0 != f64::NAN.to_bits()
            && self.0 != NEG_CANON_NAN
        {
            Some((self.0 & TAG_MASK) >> TAG_SHIFT)
        } else {
            None
        }
    }

    // ── Value extraction ──

    pub fn as_float(self) -> Option<f64> {
        if self.is_float() {
            Some(f64::from_bits(self.0))
        } else {
            None
        }
    }

    pub fn as_int(self) -> Option<i64> {
        if self.is_int() {
            let raw = (self.0 & PAYLOAD_MASK) as i64;
            // Sign-extend from 48 bits
            Some((raw << 16) >> 16)
        } else {
            None
        }
    }

    /// Bool-tolerant integer coercion (#1680).
    ///
    /// Python defines `bool` as a subclass of `int` — `True == 1`, `False == 0`.
    /// Use this in positions that semantically expect a Python int (range bounds,
    /// list indices, slice components, `int(b)` etc.); plain `as_int` stays
    /// strict so display paths don't lose the bool→"True"/"False" distinction.
    pub fn as_int_pyint(self) -> Option<i64> {
        if let Some(i) = self.as_int() {
            Some(i)
        } else if self.is_bool() {
            Some((self.0 & 1) as i64)
        } else {
            None
        }
    }

    /// Extract the integer payload without checking the tag.
    /// Caller must ensure `self.is_int()` is true.
    /// Used in hot loops where the type has already been verified (e.g., all-int sort).
    #[inline(always)]
    pub fn as_int_unchecked(self) -> i64 {
        let raw = (self.0 & PAYLOAD_MASK) as i64;
        (raw << 16) >> 16
    }

    pub fn as_bool(self) -> Option<bool> {
        if self.is_bool() {
            Some((self.0 & 1) != 0)
        } else {
            None
        }
    }

    pub fn as_ptr(self) -> Option<*mut MbObject> {
        if self.is_ptr() {
            Some((self.0 & PAYLOAD_MASK) as *mut MbObject)
        } else {
            None
        }
    }

    /// Create a function-pointer value (TAG_FUNC = 4). Stores a 48-bit code address.
    /// Used when JIT/extern function addresses are passed as first-class values
    /// (e.g. `map(abs, [...])` or compiled lambda bodies).
    pub fn from_func(addr: usize) -> Self {
        let payload = addr as u64 & PAYLOAD_MASK;
        Self(NAN_PREFIX | (TAG_FUNC << TAG_SHIFT) | payload)
    }

    /// Extract the raw code address from a function-pointer value.
    pub fn as_func(self) -> Option<usize> {
        if self.tag() == Some(TAG_FUNC) {
            Some((self.0 & PAYLOAD_MASK) as usize)
        } else {
            None
        }
    }

    /// Raw bits for codegen.
    pub fn to_bits(self) -> u64 {
        self.0
    }

    /// Construct from raw bits.
    pub fn from_bits(bits: u64) -> Self {
        Self(bits)
    }
}

impl std::fmt::Debug for MbValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_none() {
            write!(f, "None")
        } else if self.is_not_implemented() {
            write!(f, "NotImplemented")
        } else if self.is_ellipsis() {
            write!(f, "Ellipsis")
        } else if let Some(i) = self.as_int() {
            write!(f, "{i}")
        } else if let Some(b) = self.as_bool() {
            write!(f, "{b}")
        } else if let Some(fl) = self.as_float() {
            write!(f, "{fl}")
        } else if self.is_ptr() {
            write!(f, "<object@{:#x}>", self.0 & PAYLOAD_MASK)
        } else {
            write!(f, "<unknown:{:#018x}>", self.0)
        }
    }
}

impl PartialEq for MbValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for MbValue {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mbvalue_has_u64_abi() {
        assert_eq!(std::mem::size_of::<MbValue>(), std::mem::size_of::<u64>());
        assert_eq!(std::mem::align_of::<MbValue>(), std::mem::align_of::<u64>());
    }

    #[test]
    fn test_int_roundtrip() {
        for i in [
            0,
            1,
            -1,
            42,
            -42,
            1000000,
            -1000000,
            (1i64 << 47) - 1,
            -(1i64 << 47),
        ] {
            let v = MbValue::from_int(i);
            assert!(v.is_int(), "expected int for {i}");
            assert_eq!(v.as_int(), Some(i), "roundtrip failed for {i}");
            assert!(!v.is_float());
            assert!(!v.is_bool());
            assert!(!v.is_none());
        }
    }

    #[test]
    fn test_float_roundtrip() {
        for f in [0.0, 1.0, -1.0, 3.14, f64::INFINITY, f64::NEG_INFINITY] {
            let v = MbValue::from_float(f);
            assert!(v.is_float(), "expected float for {f}");
            assert_eq!(v.as_float(), Some(f));
            assert!(!v.is_int());
        }
    }

    #[test]
    fn test_bool_roundtrip() {
        let t = MbValue::from_bool(true);
        let f = MbValue::from_bool(false);
        assert!(t.is_bool());
        assert_eq!(t.as_bool(), Some(true));
        assert!(f.is_bool());
        assert_eq!(f.as_bool(), Some(false));
    }

    #[test]
    fn test_none() {
        let n = MbValue::none();
        assert!(n.is_none());
        assert!(!n.is_int());
        assert!(!n.is_float());
        assert!(!n.is_bool());
    }

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<MbValue>(), 8);
    }

    // ── Additional tests ──

    #[test]
    fn test_negative_int_roundtrip() {
        let v = MbValue::from_int(-1);
        assert_eq!(v.as_int(), Some(-1));
        let v2 = MbValue::from_int(-999999);
        assert_eq!(v2.as_int(), Some(-999999));
    }

    #[test]
    fn test_max_48bit_int() {
        let max = (1i64 << 47) - 1; // 140737488355327
        let min = -(1i64 << 47); // -140737488355328
        assert_eq!(MbValue::from_int(max).as_int(), Some(max));
        assert_eq!(MbValue::from_int(min).as_int(), Some(min));
    }

    #[test]
    fn test_zero_int() {
        let v = MbValue::from_int(0);
        assert!(v.is_int());
        assert_eq!(v.as_int(), Some(0));
    }

    #[test]
    fn test_nan_float_canonicalized() {
        let v = MbValue::from_float(f64::NAN);
        assert!(v.is_float());
        let extracted = v.as_float().unwrap();
        assert!(extracted.is_nan());
    }

    #[test]
    fn test_from_ptr_roundtrip() {
        let obj = MbObject::new_str("test".to_string());
        let v = MbValue::from_ptr(obj);
        assert!(v.is_ptr());
        assert!(!v.is_int());
        assert!(!v.is_float());
        assert!(!v.is_bool());
        assert!(!v.is_none());
        let recovered = v.as_ptr().unwrap();
        assert_eq!(recovered, obj);
        unsafe {
            super::super::rc::mb_release(obj);
        }
    }

    #[test]
    fn test_as_int_on_non_int_returns_none() {
        assert_eq!(MbValue::from_float(1.0).as_int(), None);
        assert_eq!(MbValue::from_bool(true).as_int(), None);
        assert_eq!(MbValue::none().as_int(), None);
    }

    #[test]
    fn test_as_float_on_non_float_returns_none() {
        assert_eq!(MbValue::from_int(1).as_float(), None);
        assert_eq!(MbValue::from_bool(true).as_float(), None);
        assert_eq!(MbValue::none().as_float(), None);
    }

    #[test]
    fn test_as_bool_on_non_bool_returns_none() {
        assert_eq!(MbValue::from_int(1).as_bool(), None);
        assert_eq!(MbValue::from_float(1.0).as_bool(), None);
        assert_eq!(MbValue::none().as_bool(), None);
    }

    #[test]
    fn test_as_ptr_on_non_ptr_returns_none() {
        assert_eq!(MbValue::from_int(1).as_ptr(), None);
        assert_eq!(MbValue::from_float(1.0).as_ptr(), None);
        assert_eq!(MbValue::from_bool(true).as_ptr(), None);
        assert_eq!(MbValue::none().as_ptr(), None);
    }

    #[test]
    fn test_to_bits_from_bits_roundtrip() {
        let vals = [
            MbValue::from_int(42),
            MbValue::from_float(3.14),
            MbValue::from_bool(true),
            MbValue::none(),
        ];
        for v in vals {
            let bits = v.to_bits();
            let recovered = MbValue::from_bits(bits);
            assert_eq!(v, recovered);
        }
    }

    #[test]
    fn test_equality() {
        assert_eq!(MbValue::from_int(42), MbValue::from_int(42));
        assert_ne!(MbValue::from_int(42), MbValue::from_int(43));
        assert_eq!(MbValue::from_bool(true), MbValue::from_bool(true));
        assert_ne!(MbValue::from_bool(true), MbValue::from_bool(false));
        assert_eq!(MbValue::none(), MbValue::none());
        assert_ne!(MbValue::from_int(0), MbValue::from_bool(false));
        assert_ne!(MbValue::from_int(0), MbValue::none());
    }

    #[test]
    fn test_debug_format_int() {
        let v = MbValue::from_int(42);
        assert_eq!(format!("{:?}", v), "42");
    }

    #[test]
    fn test_debug_format_bool() {
        assert_eq!(format!("{:?}", MbValue::from_bool(true)), "true");
        assert_eq!(format!("{:?}", MbValue::from_bool(false)), "false");
    }

    #[test]
    fn test_debug_format_none() {
        assert_eq!(format!("{:?}", MbValue::none()), "None");
    }

    #[test]
    fn test_debug_format_float() {
        let v = MbValue::from_float(3.14);
        assert_eq!(format!("{:?}", v), "3.14");
    }

    #[test]
    fn test_debug_format_ptr() {
        let obj = MbObject::new_str("x".to_string());
        let v = MbValue::from_ptr(obj);
        let dbg = format!("{:?}", v);
        assert!(dbg.starts_with("<object@0x"));
        unsafe {
            super::super::rc::mb_release(obj);
        }
    }

    #[test]
    fn test_float_negative_zero() {
        let v = MbValue::from_float(-0.0);
        assert!(v.is_float());
        let f = v.as_float().unwrap();
        assert!(f.is_sign_negative());
    }

    #[test]
    fn test_is_none_exclusive() {
        let n = MbValue::none();
        assert!(n.is_none());
        assert!(!n.is_ptr());
    }

    #[test]
    fn test_copy_semantics() {
        let a = MbValue::from_int(42);
        let b = a; // Copy
        assert_eq!(a, b);
        assert_eq!(a.as_int(), Some(42));
        assert_eq!(b.as_int(), Some(42));
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_int_zero_differs_from_bool_false() {
        let iz = MbValue::from_int(0);
        let bf = MbValue::from_bool(false);
        assert_ne!(iz.to_bits(), bf.to_bits());
        assert!(iz.as_bool().is_none());
        assert!(bf.as_int().is_none());
    }

    #[test]
    fn test_py312_none_distinct() {
        let n = MbValue::none();
        assert_ne!(n, MbValue::from_int(0));
        assert_ne!(n, MbValue::from_bool(false));
        assert_ne!(n, MbValue::from_float(0.0));
    }

    #[test]
    fn test_py312_nan_canonical() {
        let n1 = MbValue::from_float(f64::NAN);
        let n2 = MbValue::from_float(f64::NAN);
        assert_eq!(n1.to_bits(), n2.to_bits());
        assert!(n1.as_float().unwrap().is_nan());
    }

    #[test]
    fn test_py312_neg_zero() {
        let neg = MbValue::from_float(-0.0_f64);
        assert!(neg.is_float());
        assert!(neg.as_float().unwrap().is_sign_negative());
    }

    #[test]
    fn test_py312_infinity() {
        let inf = MbValue::from_float(f64::INFINITY);
        assert!(inf.is_float());
        assert!(!inf.is_int());
        assert!(inf.as_float().unwrap().is_infinite());
    }

    #[test]
    fn test_py312_int_48bit_extremes() {
        let max48: i64 = (1i64 << 47) - 1;
        let min48: i64 = -(1i64 << 47);
        assert_eq!(MbValue::from_int(max48).as_int(), Some(max48));
        assert_eq!(MbValue::from_int(min48).as_int(), Some(min48));
    }

    // ── Additional tag coverage ──

    #[test]
    fn test_func_roundtrip() {
        let addr: usize = 0x12345678;
        let v = MbValue::from_func(addr);
        assert!(!v.is_int());
        assert!(!v.is_float());
        assert!(!v.is_bool());
        assert!(!v.is_none());
        assert!(!v.is_ptr());
        assert_eq!(v.as_func(), Some(addr));
    }

    #[test]
    fn test_func_addr_zero() {
        let v = MbValue::from_func(0);
        assert_eq!(v.as_func(), Some(0));
    }

    #[test]
    fn test_func_max_48bit_addr() {
        let max_addr: usize = (1 << 48) - 1;
        let v = MbValue::from_func(max_addr);
        assert_eq!(v.as_func(), Some(max_addr));
    }

    #[test]
    fn test_as_func_on_non_func_returns_none() {
        assert_eq!(MbValue::from_int(1).as_func(), None);
        assert_eq!(MbValue::from_float(1.0).as_func(), None);
        assert_eq!(MbValue::from_bool(true).as_func(), None);
        assert_eq!(MbValue::none().as_func(), None);
    }

    // ── NaN, Infinity, -0.0 edge cases (R1) ──

    #[test]
    fn test_neg_infinity_roundtrip() {
        let v = MbValue::from_float(f64::NEG_INFINITY);
        assert!(v.is_float());
        let f = v.as_float().unwrap();
        assert!(f.is_infinite());
        assert!(f.is_sign_negative());
    }

    #[test]
    fn test_positive_infinity_roundtrip() {
        let v = MbValue::from_float(f64::INFINITY);
        assert!(v.is_float());
        let f = v.as_float().unwrap();
        assert!(f.is_infinite());
        assert!(f.is_sign_positive());
    }

    #[test]
    fn test_negative_zero_is_float_not_int() {
        let v = MbValue::from_float(-0.0_f64);
        assert!(v.is_float());
        assert!(!v.is_int());
        assert_eq!(v.as_int(), None);
    }

    #[test]
    fn test_negative_zero_sign_preserved() {
        let v = MbValue::from_float(-0.0_f64);
        let f = v.as_float().unwrap();
        // -0.0 == 0.0 in IEEE 754, but sign bit is set
        assert_eq!(f, 0.0_f64);
        assert!(f.is_sign_negative());
    }

    #[test]
    fn test_nan_is_float() {
        let v = MbValue::from_float(f64::NAN);
        assert!(v.is_float());
        assert!(!v.is_int());
        assert!(!v.is_bool());
        assert!(!v.is_none());
    }

    #[test]
    fn test_nan_as_float_is_nan() {
        let v = MbValue::from_float(f64::NAN);
        let f = v.as_float().unwrap();
        assert!(f.is_nan());
    }

    #[test]
    fn test_infinity_not_nan() {
        let v = MbValue::from_float(f64::INFINITY);
        let f = v.as_float().unwrap();
        assert!(!f.is_nan());
        assert!(f.is_infinite());
    }

    // ── from_bits/to_bits invariants ──

    #[test]
    fn test_from_bits_to_bits_int() {
        let v = MbValue::from_int(12345);
        assert_eq!(MbValue::from_bits(v.to_bits()).as_int(), Some(12345));
    }

    #[test]
    fn test_from_bits_to_bits_bool() {
        let t = MbValue::from_bool(true);
        let f = MbValue::from_bool(false);
        assert_eq!(MbValue::from_bits(t.to_bits()).as_bool(), Some(true));
        assert_eq!(MbValue::from_bits(f.to_bits()).as_bool(), Some(false));
    }

    #[test]
    fn test_from_bits_to_bits_none() {
        let n = MbValue::none();
        let recovered = MbValue::from_bits(n.to_bits());
        assert!(recovered.is_none());
    }

    // ── INT range boundary tests ──

    #[test]
    fn test_int_value_one() {
        let v = MbValue::from_int(1);
        assert!(v.is_int());
        assert_eq!(v.as_int(), Some(1));
    }

    #[test]
    fn test_int_value_minus_one() {
        let v = MbValue::from_int(-1);
        assert!(v.is_int());
        assert_eq!(v.as_int(), Some(-1));
    }

    #[test]
    fn test_int_large_positive() {
        let val = 1_000_000_000_i64;
        assert_eq!(MbValue::from_int(val).as_int(), Some(val));
    }

    #[test]
    fn test_int_large_negative() {
        let val = -1_000_000_000_i64;
        assert_eq!(MbValue::from_int(val).as_int(), Some(val));
    }

    // ── Type exclusivity ──

    #[test]
    fn test_int_not_func() {
        assert_eq!(MbValue::from_int(0).as_func(), None);
    }

    #[test]
    fn test_bool_not_func() {
        assert_eq!(MbValue::from_bool(false).as_func(), None);
    }

    #[test]
    fn test_none_not_func() {
        assert_eq!(MbValue::none().as_func(), None);
    }

    // ── float subnormal / denormal ──

    #[test]
    fn test_float_subnormal() {
        let tiny = f64::MIN_POSITIVE / 2.0; // subnormal
        let v = MbValue::from_float(tiny);
        assert!(v.is_float());
        let f = v.as_float().unwrap();
        assert!(f > 0.0);
    }

    // ── from_bits/to_bits for float and func ──

    #[test]
    fn test_from_bits_to_bits_float() {
        let v = MbValue::from_float(2.71828);
        assert_eq!(MbValue::from_bits(v.to_bits()).as_float(), Some(2.71828));
    }

    #[test]
    fn test_from_bits_to_bits_func() {
        let addr: usize = 0xABCDEF;
        let v = MbValue::from_func(addr);
        assert_eq!(MbValue::from_bits(v.to_bits()).as_func(), Some(addr));
    }

    // ── Debug format edge cases ──

    #[test]
    fn test_debug_format_func() {
        let v = MbValue::from_func(0x1000);
        let dbg = format!("{:?}", v);
        // Falls through to the unknown branch since func has no named debug arm
        assert!(!dbg.is_empty());
    }

    #[test]
    fn test_debug_format_nan() {
        let v = MbValue::from_float(f64::NAN);
        let dbg = format!("{:?}", v);
        assert!(dbg.to_lowercase().contains("nan"));
    }

    #[test]
    fn test_debug_format_infinity() {
        let inf = MbValue::from_float(f64::INFINITY);
        let dbg = format!("{:?}", inf);
        assert!(dbg.contains("inf"));
    }

    #[test]
    fn test_debug_format_negative_float() {
        let v = MbValue::from_float(-3.14);
        let dbg = format!("{:?}", v);
        assert!(dbg.starts_with('-'));
    }

    // ── Type exclusivity: float ──

    #[test]
    fn test_float_not_int_bool_none() {
        let v = MbValue::from_float(1.5);
        assert!(!v.is_int());
        assert!(!v.is_bool());
        assert!(!v.is_none());
        assert!(!v.is_ptr());
    }

    #[test]
    fn test_float_as_int_none() {
        assert_eq!(MbValue::from_float(0.0).as_int(), None);
        assert_eq!(MbValue::from_float(-1.5).as_int(), None);
    }

    #[test]
    fn test_float_as_bool_none() {
        assert_eq!(MbValue::from_float(1.0).as_bool(), None);
    }

    #[test]
    fn test_float_as_func_none() {
        assert_eq!(MbValue::from_float(1.0).as_func(), None);
    }

    // ── PTR: different object kinds ──

    #[test]
    fn test_ptr_with_list_object() {
        let list = MbObject::new_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let v = MbValue::from_ptr(list);
        assert!(v.is_ptr());
        assert!(!v.is_int());
        assert!(!v.is_float());
        assert!(!v.is_bool());
        assert!(!v.is_none());
        let recovered = v.as_ptr().unwrap();
        assert_eq!(recovered, list);
        unsafe {
            super::super::rc::mb_release(list);
        }
    }

    #[test]
    fn test_ptr_with_dict_object() {
        let dict = MbObject::new_dict();
        let v = MbValue::from_ptr(dict);
        assert!(v.is_ptr());
        assert_eq!(v.as_func(), None);
        unsafe {
            super::super::rc::mb_release(dict);
        }
    }

    #[test]
    fn test_ptr_as_func_none() {
        let obj = MbObject::new_str("hello".to_string());
        let v = MbValue::from_ptr(obj);
        assert_eq!(v.as_func(), None);
        unsafe {
            super::super::rc::mb_release(obj);
        }
    }

    // ── FUNC exclusivity ──

    #[test]
    fn test_func_is_not_ptr() {
        let v = MbValue::from_func(0x5555);
        assert!(!v.is_ptr());
        assert!(v.as_ptr().is_none());
    }

    #[test]
    fn test_func_is_not_int() {
        let v = MbValue::from_func(1234);
        assert!(!v.is_int());
    }

    #[test]
    fn test_func_is_not_float() {
        let v = MbValue::from_func(0xDEAD);
        assert!(!v.is_float());
    }

    #[test]
    fn test_func_is_not_bool() {
        let v = MbValue::from_func(42);
        assert!(!v.is_bool());
        assert_eq!(v.as_bool(), None);
    }

    #[test]
    fn test_func_is_not_none() {
        let v = MbValue::from_func(0);
        assert!(!v.is_none());
    }

    // ── INT: various values ──

    #[test]
    fn test_int_two_hundred() {
        let v = MbValue::from_int(200);
        assert_eq!(v.as_int(), Some(200));
    }

    #[test]
    fn test_int_negative_hundred() {
        let v = MbValue::from_int(-100);
        assert_eq!(v.as_int(), Some(-100));
    }

    #[test]
    fn test_int_exclusive_of_ptr() {
        let v = MbValue::from_int(42);
        assert!(!v.is_ptr());
        assert_eq!(v.as_ptr(), None);
    }

    // ── Equality and hash invariants ──

    #[test]
    fn test_two_floats_same_value_equal() {
        let a = MbValue::from_float(1.23456789);
        let b = MbValue::from_float(1.23456789);
        assert_eq!(a, b);
    }

    #[test]
    fn test_two_ints_different_not_equal() {
        let a = MbValue::from_int(100);
        let b = MbValue::from_int(101);
        assert_ne!(a, b);
    }

    #[test]
    fn test_int_vs_float_not_equal() {
        // from_int(1) and from_float(1.0) have different bit patterns
        let i = MbValue::from_int(1);
        let f = MbValue::from_float(1.0);
        assert_ne!(i, f);
    }

    #[test]
    fn test_func_addr_equality() {
        let a = MbValue::from_func(0x99887766);
        let b = MbValue::from_func(0x99887766);
        assert_eq!(a, b);
    }

    #[test]
    fn test_func_different_addrs_not_equal() {
        let a = MbValue::from_func(1);
        let b = MbValue::from_func(2);
        assert_ne!(a, b);
    }

    // ── Bit-pattern known-value tests ──

    #[test]
    fn test_none_bits_roundtrip() {
        let n = MbValue::none();
        let bits = n.to_bits();
        // Reconstruct from bits and verify identity
        let n2 = MbValue::from_bits(bits);
        assert!(n2.is_none());
        assert_eq!(n, n2);
    }

    #[test]
    fn test_bool_false_bits_roundtrip() {
        let f = MbValue::from_bool(false);
        let bits = f.to_bits();
        let f2 = MbValue::from_bits(bits);
        assert_eq!(f2.as_bool(), Some(false));
    }

    #[test]
    fn test_bool_true_bits_roundtrip() {
        let t = MbValue::from_bool(true);
        let bits = t.to_bits();
        let t2 = MbValue::from_bits(bits);
        assert_eq!(t2.as_bool(), Some(true));
    }

    // ── float large values ──

    #[test]
    fn test_float_max_finite() {
        let v = MbValue::from_float(f64::MAX);
        assert!(v.is_float());
        assert_eq!(v.as_float(), Some(f64::MAX));
    }

    #[test]
    fn test_float_min_positive() {
        let v = MbValue::from_float(f64::MIN_POSITIVE);
        assert!(v.is_float());
        assert_eq!(v.as_float(), Some(f64::MIN_POSITIVE));
    }

    #[test]
    fn test_float_negative_large() {
        let v = MbValue::from_float(-1e300);
        assert!(v.is_float());
        assert_eq!(v.as_float(), Some(-1e300));
    }
}
