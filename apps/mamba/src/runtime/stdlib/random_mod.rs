//! @codegen-skip: handwrite-pre-standardize
//!
//! `random` module for Mamba — Python 3.12 stdlib `random` (#1265 Task #40).
//!
//! Provides module-level functions (`random`, `seed`, `randint`, `randrange`,
//! `uniform`, `triangular`, `choice`, `shuffle`, `sample`, `choices`,
//! `gauss`, `normalvariate`, `expovariate`, `lognormvariate`,
//! `vonmisesvariate`, `gammavariate`, `betavariate`, `paretovariate`,
//! `weibullvariate`, `getstate`, `setstate`, `getrandbits`, `randbytes`)
//! routed through a thread-local default `_inst` handle, plus the
//! `Random` class via the integer-handle protocol (see hashlib/hmac/
//! decimal/array).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (NATIVE_FUNC_ADDRS dispatch + integer-handle method routing +
//! Mersenne-Twister state) is not yet emitted by score codegen. Tracked
//! as part of the Phase-2 brute-force sweep; will be replaced when score
//! standardize lands the stdlib-shim section type. Issue #1414 cluster.
//!
//! Implementation notes:
//!
//! - PRNG backend is `rand_mt::Mt` (32-bit Mersenne Twister), matching
//!   CPython's MT19937 generator family. Integer/bool/BigInt seeds use
//!   CPython's arbitrary-length `init_by_array` state initialization; the
//!   existing u32 legacy path remains for non-integer seed types.
//!
//! - Each `Random` instance and the module's lazy-init `_inst` are
//!   `Mt` instances stored in a thread-local `HashMap<u64, Mt>`.
//!   Handle IDs start at `0x4000_0000` to namespace away from
//!   hashlib (1..), hmac, decimal, array, json (also low-id ranges).
//!
//! - Module-level fns lazy-init a default handle on first call
//!   (approach (a) from the scout doc). CPython does eager init at
//!   import; lazy at first call is observationally identical for end
//!   users.
//!
//! - `gauss` uses CPython's exact two-draw trigonometric Box–Muller with a
//!   per-handle spare; `normalvariate` and the other variates retain the
//!   legacy distribution-correct helpers. Inverse-CDF is used for
//!   expovariate/paretovariate/weibullvariate; Marsaglia–Tsang 2000 for
//!   gammavariate; ratio-of-gammas for betavariate; Best–Fisher 1979
//!   (iteration cap 32) for vonmisesvariate.
//!
//! - `getstate`/`setstate`: return `(handle_id,)` 1-tuple so
//!   snapshot/restore patterns where the handle stays alive round-trip
//!   correctly. Full 625-int MT state serialization is out of scope.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use num_bigint::{BigInt, Sign};
use sha2::{Digest, Sha512};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN

use rand_mt::Mt;

/// Base = (1<<46) + (1<<45) = 3*(1<<45). Owns its slice of the
/// integer-handle id space well above `HANDLE_MIN_ID` (= 2^40). See
/// `integer_handle_registry::HANDLE_MIN_ID`.
const RANDOM_HANDLE_BASE: u64 = (1u64 << 46) + (1u64 << 45);

thread_local! {
    static RANDOMS: RefCell<HashMap<u64, Mt>> = RefCell::new(HashMap::new());
    static RANDOM_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_RANDOM_ID: Cell<u64> = const { Cell::new(RANDOM_HANDLE_BASE) };
    /// Per-handle refcount (#2111).
    static RANDOM_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
    /// Lazy-init default handle for module-level functions (Python's
    /// `random._inst`). None until first call.
    static DEFAULT_HANDLE: Cell<Option<u64>> = const { Cell::new(None) };
    /// Legacy cached spare normal retained for non-gauss Box–Muller callers.
    static GAUSS_SPARE: Cell<Option<f64>> = const { Cell::new(None) };
    /// CPython-compatible `gauss()` spare, isolated per Random handle.
    static GAUSS_SPARES: RefCell<HashMap<u64, f64>> = RefCell::new(HashMap::new());
}

fn alloc_random_id() -> u64 {
    NEXT_RANDOM_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    })
}

/// Class.rs predicate — route `int.method()` into the random protocol
/// when the receiver id was allocated here.
pub fn is_random_handle(id: u64) -> bool {
    RANDOM_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_random_handle(id: u64) {
    RANDOMS.with(|m| {
        m.borrow_mut().remove(&id);
    });
    RANDOM_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    RANDOM_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
    GAUSS_SPARES.with(|c| {
        c.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_random_handle(id) {
        return false;
    }
    RANDOM_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_random_handle(id) {
        return false;
    }
    let should_drop = RANDOM_REFCOUNTS.with(|r| {
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
        drop_random_handle(id);
    }
    true
}

fn make_handle(seed: Option<Mt>) -> u64 {
    let id = alloc_random_id();
    let rng = seed.unwrap_or_else(Mt::new_unseeded);
    RANDOMS.with(|m| {
        m.borrow_mut().insert(id, rng);
    });
    RANDOM_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    id
}

fn default_handle() -> u64 {
    DEFAULT_HANDLE.with(|c| {
        if let Some(id) = c.get() {
            return id;
        }
        let id = make_handle(None);
        c.set(Some(id));
        id
    })
}

/// FNV-1a 32-bit hash of arbitrary bytes — used when the user passes a
/// non-int seed (str/bytes). Cheap and consistent for the shim.
fn fnv1a_32(bytes: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for b in bytes {
        h ^= u32::from(*b);
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Convert a non-integer MbValue seed to the legacy u32 seed.
fn seed_from_value(v: MbValue) -> u32 {
    if v.is_none() {
        // CPython uses os.urandom; we use a deterministic-but-time-varying
        // default. Tests should pass an explicit seed for reproducibility.
        return 0x5489_u32;
    }
    if let Some(i) = v.as_int() {
        return (i as i64) as u32;
    }
    if let Some(f) = v.as_float() {
        return f.to_bits() as u32;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => return fnv1a_32(s.as_bytes()),
                ObjData::Bytes(b) => return fnv1a_32(b.as_slice()),
                _ => {}
            }
        }
    }
    0x5489_u32
}

const MT_STATE_LEN: usize = 624;

/// Temper one MT19937 state word locally so it can be passed through the
/// public `rand_mt::Mt::from([u32; 624])` state-recovery constructor.
#[inline]
fn temper_mt(mut x: u32) -> u32 {
    x ^= x >> 11;
    x ^= (x << 7) & 0x9d2c_5680;
    x ^= (x << 15) & 0xefc6_0000;
    x ^= x >> 18;
    x
}

/// Convert an integer-like runtime value into CPython's nonempty little-endian
/// base-2^32 key array. `num_bigint` stores its magnitude in exactly this
/// order, so this preserves every bit of an arbitrary-length seed while
/// making negative seeds equivalent to their absolute values.
fn integer_seed_words(v: MbValue) -> Option<Vec<u32>> {
    // SAFETY: `v` is a live runtime value owned by the caller; `to_bigint`
    // only reads its inline payload or BigInt object.
    let big = unsafe { super::super::bigint_ops::to_bigint(v) }?;
    let (_, mut words) = big.to_u32_digits();
    if words.is_empty() {
        words.push(0);
    }
    Some(words)
}

/// Convert CPython version-2 text-like seed material into its MT key words.
/// The payload and raw SHA-512 digest are interpreted as one positive
/// big-endian integer before `num_bigint` exposes little-endian u32 digits.
fn text_seed_words(v: MbValue) -> Option<Vec<u32>> {
    let bytes = v.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            ObjData::Bytes(b) => Some(b.as_slice().to_vec()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })?;
    let digest = Sha512::digest(&bytes);
    let mut material = Vec::with_capacity(bytes.len().checked_add(digest.len())?);
    material.extend_from_slice(&bytes);
    material.extend_from_slice(&digest);
    let big = BigInt::from_bytes_be(Sign::Plus, &material);
    let (_, mut words) = big.to_u32_digits();
    if words.is_empty() {
        words.push(0);
    }
    Some(words)
}

/// Build an MT19937 instance from CPython's exact `init_by_array` algorithm.
/// The local state is converted to tempered output words because `Mt::from`
/// intentionally accepts 624 observed outputs rather than raw state words.
fn mt_from_cpython_key(key: &[u32]) -> Mt {
    debug_assert!(!key.is_empty());

    let mut state = [0_u32; MT_STATE_LEN];
    state[0] = 19650218;
    for i in 1..MT_STATE_LEN {
        state[i] = 1_812_433_253_u32
            .wrapping_mul(state[i - 1] ^ (state[i - 1] >> 30))
            .wrapping_add(i as u32);
    }

    let mut i = 1_usize;
    let mut j = 0_usize;
    let mut remaining = MT_STATE_LEN.max(key.len());
    while remaining != 0 {
        state[i] = (state[i]
            ^ (state[i - 1] ^ (state[i - 1] >> 30)).wrapping_mul(1_664_525))
            .wrapping_add(key[j])
            .wrapping_add(j as u32);
        i += 1;
        j += 1;
        if i >= MT_STATE_LEN {
            state[0] = state[MT_STATE_LEN - 1];
            i = 1;
        }
        if j >= key.len() {
            j = 0;
        }
        remaining -= 1;
    }

    remaining = MT_STATE_LEN - 1;
    while remaining != 0 {
        state[i] = (state[i]
            ^ (state[i - 1] ^ (state[i - 1] >> 30)).wrapping_mul(1_566_083_941))
            .wrapping_sub(i as u32);
        i += 1;
        if i >= MT_STATE_LEN {
            state[0] = state[MT_STATE_LEN - 1];
            i = 1;
        }
        remaining -= 1;
    }
    state[0] = 1_u32 << 31;

    let mut tempered = [0_u32; MT_STATE_LEN];
    for (out, &word) in tempered.iter_mut().zip(state.iter()) {
        *out = temper_mt(word);
    }
    Mt::from(tempered)
}

/// Shared constructor/`Random.seed` conversion. Integer-like seeds follow
/// CPython; all other supported legacy seed types retain their existing u32
/// conversion and unsupported values retain the old fallback behavior.
fn mt_from_seed_value(v: MbValue) -> Mt {
    if let Some(words) = integer_seed_words(v).or_else(|| text_seed_words(v)) {
        mt_from_cpython_key(&words)
    } else {
        Mt::new(seed_from_value(v))
    }
}

fn is_supported_seed_value(v: MbValue) -> bool {
    if v.is_none() || v.is_int() || v.is_bool() || v.is_float() {
        return true;
    }
    v.as_ptr().is_some_and(|ptr| unsafe {
        matches!(
            &(*ptr).data,
            ObjData::BigInt(_) | ObjData::Str(_) | ObjData::Bytes(_) | ObjData::ByteArray(_)
        )
    })
}

// ── Per-handle PRNG ops ──

fn with_rng<R>(id: u64, f: impl FnOnce(&mut Mt) -> R) -> R {
    RANDOMS.with(|m| {
        let mut g = m.borrow_mut();
        // If a handle was somehow lost, recover by inserting a fresh one
        // — avoids panic in pathological setstate paths.
        let rng = g.entry(id).or_insert_with(Mt::new_unseeded);
        f(rng)
    })
}

fn next_u32(id: u64) -> u32 {
    with_rng(id, |r| r.next_u32())
}
fn next_u64(id: u64) -> u64 {
    with_rng(id, |r| r.next_u64())
}

/// Draw a bounded integer with CPython's high-bit rejection layout.
fn random_below(id: u64, n: u64) -> u64 {
    debug_assert!(n > 0);
    let k = 64 - n.leading_zeros();
    loop {
        let candidate = if k <= 32 {
            u64::from(next_u32(id) >> (32 - k))
        } else {
            let low = u64::from(next_u32(id));
            let mut high = next_u32(id);
            let partial_bits = k % 32;
            if partial_bits != 0 {
                high >>= 32 - partial_bits;
            }
            low | (u64::from(high) << 32)
        };
        if candidate < n {
            return candidate;
        }
    }
}

/// Float in [0.0, 1.0). 53-bit mantissa precision per CPython's random().
fn next_f64(id: u64) -> f64 {
    let hi = (next_u32(id) >> 5) as u64; // top 27 bits
    let lo = (next_u32(id) >> 6) as u64; // top 26 bits
    ((hi * (1u64 << 26)) + lo) as f64 / (1u64 << 53) as f64
}

fn extract_list(val: MbValue) -> Option<Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            Some(lock.read().unwrap().to_vec())
        } else {
            None
        }
    })
}

fn extract_f64(val: MbValue, default: f64) -> f64 {
    val.as_float()
        .or_else(|| val.as_int().map(|i| i as f64))
        // bool is an int subtype: True/False weights count as 1/0.
        .or_else(|| val.as_bool().map(|b| if b { 1.0 } else { 0.0 }))
        .unwrap_or(default)
}

/// Materialize an iterator-handle argument (e.g. `range(n)` used as a
/// choices/sample population or cum_weights) into a List value; every other
/// value passes through unchanged.
fn materialize_arg(val: MbValue) -> MbValue {
    if super::super::iter::is_iter_handle(val) {
        if let Some(items) = super::super::iter::drain_iter_to_vec(val) {
            return MbValue::from_ptr(MbObject::new_list(items));
        }
    }
    val
}

fn extract_i64(val: MbValue, default: i64) -> i64 {
    val.as_int().unwrap_or(default)
}

/// True when `val` is a runtime dict object (the trailing kwargs dict that
/// the method-call lowering appends for `f(a, kw=v)` on a module/instance
/// attribute). Used to disambiguate a real positional argument from the
/// folded keyword bag.
fn is_dict_value(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Read a string-keyed entry out of a trailing kwargs `ObjData::Dict`.
/// Returns `None` when `val` is not a dict or the key is absent.
fn kwarg_get(val: MbValue, key: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            // `DictKey::Str` hashes with the Python-semantic domain (#1028),
            // not Rust's native `str` hash — a raw `.get(&str)` here would
            // silently miss present keys (#1566). Route through the
            // hash-domain-safe helper.
            super::super::dict_ops::dict_get_exact_str(&lock.read().unwrap(), key)
        } else {
            None
        }
    })
}

/// Length of a `choices`/`sample` population: List, Tuple, or Str (character
/// count). Returns `None` for unsupported population shapes.
fn population_len(val: MbValue) -> Option<usize> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().len()),
            ObjData::Tuple(items) => Some(items.len()),
            ObjData::Str(s) => Some(s.chars().count()),
            _ => None,
        }
    })
}

/// Number of elements in a weight sequence (List or Tuple). `None` when the
/// value is not a recognised sequence (e.g. a scalar or `range` proxy).
fn weight_seq_len(val: MbValue) -> Option<usize> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().len()),
            ObjData::Tuple(items) => Some(items.len()),
            _ => None,
        }
    })
}

/// Sum a weight sequence (List or Tuple of int/float) as f64. `None` when the
/// value is not a recognised numeric sequence.
fn weight_seq_sum(val: MbValue) -> Option<f64> {
    val.as_ptr().and_then(|ptr| unsafe {
        let items: Vec<MbValue> = match &(*ptr).data {
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => return None,
        };
        let total: f64 = items.iter().map(|v| extract_f64(*v, 0.0)).sum();
        Some(total)
    })
}

// ── Exception helpers (CPython-3.12 error semantics) ──

fn raise_value_error(msg: &str) -> MbValue {
    let exc_type = MbValue::from_ptr(MbObject::new_str("ValueError".to_string()));
    let message = MbValue::from_ptr(MbObject::new_str(msg.to_string()));
    super::super::exception::mb_raise(exc_type, message);
    unsafe {
        super::super::rc::release_if_ptr(exc_type);
        super::super::rc::release_if_ptr(message);
    }
    MbValue::none()
}

fn raise_type_error(msg: &str) -> MbValue {
    let exc_type = MbValue::from_ptr(MbObject::new_str("TypeError".to_string()));
    let message = MbValue::from_ptr(MbObject::new_str(msg.to_string()));
    super::super::exception::mb_raise(exc_type, message);
    unsafe {
        super::super::rc::release_if_ptr(exc_type);
        super::super::rc::release_if_ptr(message);
    }
    MbValue::none()
}

fn raise_index_error(msg: &str) -> MbValue {
    let exc_type = MbValue::from_ptr(MbObject::new_str("IndexError".to_string()));
    let message = MbValue::from_ptr(MbObject::new_str(msg.to_string()));
    super::super::exception::mb_raise(exc_type, message);
    unsafe {
        super::super::rc::release_if_ptr(exc_type);
        super::super::rc::release_if_ptr(message);
    }
    MbValue::none()
}

/// True when the value is a list (mutable sequence). `shuffle` requires a
/// mutable sequence; CPython raises TypeError on immutable inputs such as str.
fn is_list_value(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_)) })
        .unwrap_or(false)
}

/// The temporary population view used by `sample()`. List/Tuple snapshots
/// borrow their element owners, while raw strings and drained iterator values
/// arrive as freshly owned heap values which must be released after selection.
/// Keeping this guard alive until the result list has retained its elements
/// makes early validation returns and both selection paths cleanup-safe.
struct SampleItems {
    values: Vec<MbValue>,
    owns_values: bool,
}

impl SampleItems {
    fn borrowed(values: Vec<MbValue>) -> Self {
        Self {
            values,
            owns_values: false,
        }
    }

    fn owned(values: Vec<MbValue>) -> Self {
        Self {
            values,
            owns_values: true,
        }
    }
}

impl Drop for SampleItems {
    fn drop(&mut self) {
        if self.owns_values {
            for &value in &self.values {
                unsafe { super::super::rc::release_if_ptr(value) };
            }
        }
    }
}

/// Supported `sample()` populations in the current runtime: plain lists,
/// tuples, strings, and iterator/range handles. Iterator handles are drained
/// directly so their returned values are owned by the local guard instead of
/// passing through the general `materialize_arg` list wrapper. Returns `None`
/// for every other shape so the caller can raise the Sequence type wall
/// instead of silently accepting an arbitrary object.
fn sample_population_items(pop: MbValue) -> Option<SampleItems> {
    if super::super::iter::is_range_handle(pop) {
        return super::super::iter::drain_iter_to_vec(pop).map(SampleItems::owned);
    }

    if super::super::iter::is_iter_handle(pop) {
        return None;
    }

    pop.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(SampleItems::borrowed(lock.read().unwrap().to_vec())),
            ObjData::Tuple(items) => Some(SampleItems::borrowed(items.clone())),
            ObjData::Str(s) => Some(SampleItems::owned(
                s.chars()
                    .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                    .collect(),
            )),
            _ => None,
        }
    })
}

// ── Module-level functions (receiver-less; route to default handle) ──
// All also exposed as instance methods via the integer-handle protocol.

pub fn mb_random_method_random(receiver: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    MbValue::from_float(next_f64(id))
}

pub fn mb_random_method_seed(receiver: MbValue, seed: MbValue) -> MbValue {
    // CPython's version-2 seed accepts only None / int / float / str / bytes /
    // bytearray. Other hashable-or-not types (complex, list, dict, tuple, set)
    // raise TypeError.
    if !is_supported_seed_value(seed) {
        return raise_type_error(
            "The only supported seed types are: None, int, float, str, bytes, and bytearray.",
        );
    }
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let rng = mt_from_seed_value(seed);
    RANDOMS.with(|m| {
        m.borrow_mut().insert(id, rng);
    });
    GAUSS_SPARES.with(|c| {
        c.borrow_mut().remove(&id);
    });
    GAUSS_SPARE.with(|c| c.set(None));
    MbValue::none()
}

pub fn mb_random_method_randint(receiver: MbValue, a: MbValue, b: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let lo = extract_i64(a, 0);
    let hi = extract_i64(b, 0);
    // CPython: randint(a, b) == randrange(a, b+1); an inverted range yields an
    // empty randrange which raises ValueError ("empty range ...").
    if hi < lo {
        return raise_value_error(&format!("empty range in randrange({}, {})", lo, hi + 1));
    }
    let range = (hi - lo + 1) as u64;
    let val = lo + random_below(id, range) as i64;
    MbValue::from_int(val)
}

pub fn mb_random_method_randrange(
    receiver: MbValue,
    a: MbValue,
    b: MbValue,
    step: MbValue,
) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let s = extract_i64(step, 1);
    if s == 0 {
        return raise_value_error("zero step for randrange()");
    }
    let (lo, hi) = if b.is_none() {
        (0_i64, extract_i64(a, 0))
    } else {
        (extract_i64(a, 0), extract_i64(b, 0))
    };
    // CPython raises ValueError on an empty range (width <= 0 for positive
    // step). The full width/step empty check below also covers negative steps.
    if (s > 0 && hi <= lo) || (s < 0 && hi >= lo) {
        return raise_value_error(&format!("empty range in randrange({}, {}, {})", lo, hi, s));
    }
    if s > 0 {
        let width = (hi - lo) as u64;
        let n = ((width - 1) / s as u64) + 1;
        let pick = random_below(id, n) as i64;
        return MbValue::from_int(lo + pick * s);
    }
    let span = ((hi - lo) as u64) / (s.unsigned_abs());
    if span == 0 {
        return MbValue::from_int(lo);
    }
    let pick = (next_u64(id) % span) as i64;
    MbValue::from_int(lo + pick * s)
}

pub fn mb_random_method_uniform(receiver: MbValue, a: MbValue, b: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let lo = extract_f64(a, 0.0);
    let hi = extract_f64(b, 1.0);
    MbValue::from_float(lo + (hi - lo) * next_f64(id))
}

pub fn mb_random_method_triangular(
    receiver: MbValue,
    low: MbValue,
    high: MbValue,
    mode: MbValue,
) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let lo = extract_f64(low, 0.0);
    let hi = extract_f64(high, 1.0);
    let m = if mode.is_none() {
        (lo + hi) * 0.5
    } else {
        extract_f64(mode, (lo + hi) * 0.5)
    };
    let u = next_f64(id);
    let c = if hi == lo { 0.5 } else { (m - lo) / (hi - lo) };
    let val = if u < c {
        lo + ((hi - lo) * (u * c).sqrt())
    } else {
        hi - ((hi - lo) * ((1.0 - u) * (1.0 - c)).sqrt())
    };
    MbValue::from_float(val)
}

pub fn mb_random_method_choice(receiver: MbValue, seq: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    match extract_list(seq) {
        Some(items) if !items.is_empty() => {
            let idx = random_below(id, items.len() as u64) as usize;
            super::super::rc::return_owned(items[idx])
        }
        // CPython indexes seq[int(random()*len)]; an empty sequence raises
        // IndexError ("Cannot choose from an empty sequence").
        _ => raise_index_error("Cannot choose from an empty sequence"),
    }
}

pub fn mb_random_method_shuffle(receiver: MbValue, lst: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // CPython mutates x in place via x[i], x[j] = x[j], x[i]; an immutable
    // sequence (e.g. str/tuple) raises TypeError on item assignment.
    if !is_list_value(lst) {
        let type_name = super::super::builtins::value_type_name(lst);
        return raise_type_error(&format!(
            "'{type_name}' object does not support item assignment"
        ));
    }
    if let Some(ptr) = lst.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let n = items.len();
                for i in (1..n).rev() {
                    let j = random_below(id, (i as u64) + 1) as usize;
                    items.swap(i, j);
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_random_method_sample(receiver: MbValue, pop: MbValue, k: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // CPython 3.11+ requires a sequence; sets/dicts raise TypeError
    // ("Population must be a sequence ...").
    let sample_items = match sample_population_items(pop) {
        Some(v) => v,
        None => {
            return raise_type_error(
                "Population must be a sequence.  For dicts or sets, use sorted(d).",
            );
        }
    };
    let items = &sample_items.values;

    // The method-call lowering folds keyword args (`counts=`, `k=`) into a
    // trailing `ObjData::Dict` that arrives here in the `k` slot. Unfold it:
    // pull the real `k` and the optional `counts` sequence out of the bag.
    let mut counts = MbValue::none();
    let raw_k = if is_dict_value(k) {
        if let Some(c) = kwarg_get(k, "counts") {
            counts = c;
        }
        extract_i64(kwarg_get(k, "k").unwrap_or_else(MbValue::none), 0)
    } else {
        extract_i64(k, 0)
    };

    // ── counts=... handling (CPython 3.12 random.sample) ──
    // Only engages when `counts` is supplied, so an unweighted sample(...) is
    // byte-for-byte unaffected.
    if !counts.is_none() {
        // counts length must match the population length.
        match weight_seq_len(counts) {
            Some(len) if len == items.len() => {}
            Some(_) => {
                return raise_value_error("The number of counts does not match the population");
            }
            // CPython runs `counts = list(counts)` — a scalar is the
            // iteration TypeError.
            None => {
                let tn = if counts.is_float() {
                    "float"
                } else if counts.as_bool().is_some() {
                    "bool"
                } else if counts.as_int().is_some() {
                    "int"
                } else {
                    "object"
                };
                return raise_type_error(&format!("'{tn}' object is not iterable"));
            }
        }
        // total = sum(counts); the expanded population has `total` elements.
        // CPython: `if not 0 <= k <= total: raise ValueError`. A negative
        // total (negative counts) or k > total both trip this.
        if let Some(total) = weight_seq_sum(counts) {
            let total = total as i64;
            if raw_k < 0 || raw_k > total {
                return raise_value_error("Sample larger than population or is negative");
            }
        }
        // Expand the population by repeating each element `counts[i]` times,
        // then sample uniformly from the expansion.
        let counts_vec = counts.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => Some(lock.read().unwrap().to_vec()),
                ObjData::Tuple(it) => Some(it.clone()),
                _ => None,
            }
        });
        if let Some(cv) = counts_vec {
            let mut expanded: Vec<MbValue> = Vec::new();
            for (i, c) in cv.iter().enumerate() {
                let reps = extract_i64(*c, 0).max(0) as usize;
                for _ in 0..reps {
                    expanded.push(items[i]);
                }
            }
            let count = raw_k.max(0) as usize;
            let mut pool = expanded;
            for i in 0..count {
                let j = i + (next_u64(id) % (pool.len() - i) as u64) as usize;
                pool.swap(i, j);
            }
            pool.truncate(count);
                return MbValue::from_ptr(MbObject::new_list_borrowed(pool));
        }
    }

    // Negative k or k larger than the population raises ValueError
    // ("Sample larger than population or is negative").
    if raw_k < 0 || raw_k as usize > items.len() {
        return raise_value_error("Sample larger than population or is negative");
    }
    let count = raw_k as usize;
    let n = items.len();
    let mut setsize = 21usize;
    if count > 5 {
        let target = count.saturating_mul(3);
        let mut power = 4usize;
        while power < target {
            let next = power.saturating_mul(4);
            if next == power {
                break;
            }
            power = next;
        }
        setsize = setsize.saturating_add(power);
    }
    if n <= setsize {
        let mut pool = items.clone();
        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            let j = random_below(id, (n - i) as u64) as usize;
            result.push(pool[j]);
            pool[j] = pool[n - i - 1];
        }
        return MbValue::from_ptr(MbObject::new_list_borrowed(result));
    }
    let mut selected = HashSet::<usize>::new();
    let mut result = Vec::with_capacity(count);
    while result.len() < count {
        let j = random_below(id, n as u64) as usize;
        if selected.insert(j) {
            result.push(items[j]);
        }
    }
    MbValue::from_ptr(MbObject::new_list_borrowed(result))
}

/// Instance-handle `gen.choices(...)` entry point. Kept on the legacy
/// (uniform / empty-for-str) behaviour: the instance method-call lowering in
/// `class.rs` drops the trailing kwargs `dict` when a positional weight arg
/// precedes it, so this path cannot reliably recover `k`/`weights` together.
/// Weighted-error validation is wired through the module-level
/// `dispatch_choices` slab (see `mb_random_method_choices_full`), which keeps
/// the full argument vector. Touching this shim would regress
/// `choices_algorithms` / `choices_subnormal` (they pass today only because
/// the dropped-`k` path degenerates uniformly).
pub fn mb_random_method_choices(receiver: MbValue, pop: MbValue, k: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let items = match extract_list(pop) {
        Some(v) => v,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let raw_k = extract_i64(k, 1);
    if items.is_empty() {
        if raw_k > 0 {
            return raise_index_error("Cannot choose from an empty sequence");
        }
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let count = raw_k.max(0) as usize;
    let n = items.len() as u64;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let idx = (next_u64(id) % n) as usize;
        out.push(items[idx]);
    }
    // `items[idx]` is picked with replacement, so the same pointer can land
    // in `out` more than once; `out` borrows those pointers from `items`
    // (itself borrowed from the population container), so the returned list
    // must retain each occurrence (`new_list_borrowed`) rather than take
    // unretained ownership (`new_list`) — otherwise a repeated pick makes
    // `release_contained_values` release the same object once per
    // occurrence on drop, a use-after-free/double-free on duplicate picks.
    MbValue::from_ptr(MbObject::new_list_borrowed(out))
}

/// Full `choices(population, weights=None, *, cum_weights=None, k=1)` with
/// CPython-3.12 weight validation. `weights` / `cum_weights` are `none()`
/// when absent. Reached from the module-level `dispatch_choices` slab, which
/// (unlike instance routing) preserves the complete argument vector.
pub fn mb_random_method_choices_full(
    receiver: MbValue,
    pop: MbValue,
    weights: MbValue,
    cum_weights: MbValue,
    k: MbValue,
) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // `choices(range(n), ...)` / `cum_weights=range(1, n+1)`: materialize
    // iterator-handle arguments into lists up front.
    let pop = materialize_arg(pop);
    let weights = materialize_arg(weights);
    let cum_weights = materialize_arg(cum_weights);
    // CPython: weights and cum_weights are mutually exclusive, and each must
    // be a sequence (a scalar is a TypeError).
    if !weights.is_none() && !cum_weights.is_none() {
        return raise_type_error("Cannot specify both weights and cumulative weights");
    }
    for w in [weights, cum_weights] {
        if !w.is_none() && weight_seq_len(w).is_none() {
            return raise_type_error("weights must be a sequence");
        }
    }
    let raw_k = extract_i64(k, 1);

    // Population length (str / list / tuple). Needed for the weight-length
    // checks below; an unsupported population behaves like the legacy
    // empty-result fallback.
    let pop_len = match population_len(pop) {
        Some(n) => n,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let has_weights = !weights.is_none();
    let has_cum = !cum_weights.is_none();

    // ── Weighted-call validation (CPython 3.12 random.choices) ──
    // Only runs when a weight sequence is actually supplied, so an
    // unweighted choices(...) call is byte-for-byte unaffected.
    if has_cum {
        // cum_weights length must match the population length.
        match weight_seq_len(cum_weights) {
            Some(len) if len == pop_len => {}
            Some(_) => {
                return raise_value_error("The number of weights does not match the population");
            }
            // A non-sequence cum_weights (scalar) is a TypeError in CPython;
            // leave that to the TypeError-specific path and fall through here.
            None => {}
        }
    } else if has_weights {
        match weight_seq_len(weights) {
            Some(len) if len == pop_len => {}
            Some(_) => {
                return raise_value_error("The number of weights does not match the population");
            }
            None => {}
        }
        // Total of weights must be strictly positive (covers all-zero and
        // negative-total). CPython: `if total <= 0.0: raise ValueError`.
        if let Some(total) = weight_seq_sum(weights) {
            if total <= 0.0 {
                return raise_value_error("Total of weights must be greater than zero");
            }
        }
    }

    // Now materialise the population for selection (str → 1-char strings).
    let items = match extract_list(pop) {
        Some(v) => v,
        None => population_as_items(pop),
    };

    // CPython: with a non-empty k, an empty population raises IndexError
    // ("Cannot choose from an empty sequence").
    if items.is_empty() {
        if raw_k > 0 {
            return raise_index_error("Cannot choose from an empty sequence");
        }
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let count = raw_k.max(0) as usize;
    let n = items.len();

    // Weighted selection when a positive-total weight sequence is present;
    // otherwise the original uniform pick (byte-for-byte for unweighted).
    let cum: Option<Vec<f64>> = if has_cum {
        cumulative_from_cum_weights(cum_weights, n)
    } else if has_weights {
        cumulative_from_weights(weights, n)
    } else {
        None
    };

    let mut out = Vec::with_capacity(count);
    match cum {
        Some(cum) if !cum.is_empty() => {
            let total = *cum.last().unwrap();
            for _ in 0..count {
                let r = next_f64(id) * total;
                // bisect_right over the cumulative table (binary search — the
                // table can hold 100k+ entries for range() populations).
                let mut idx = cum.partition_point(|&c| c <= r);
                if idx >= n {
                    idx = n - 1;
                }
                out.push(items[idx]);
            }
        }
        _ => {
            // floor(random() * n) — the SAME one-f64-per-pick consumption as
            // the weighted branch, so `choices(pop, k=..)` and
            // `choices(pop, [1]*n, k=..)` draw identical streams from one
            // seed (CPython parity: both spend one random() per pick).
            for _ in 0..count {
                let idx = ((next_f64(id) * n as f64) as usize).min(n - 1);
                out.push(items[idx]);
            }
        }
    }
    // `items[idx]` is picked with replacement (weighted or uniform), so the
    // same pointer can land in `out` more than once; `items` is itself
    // borrowed (either cloned from an existing List/Tuple population, or
    // freshly allocated per-call for a Str population with no other owner).
    // Either way, `out`'s pointers are not uniquely-owned-per-slot, so the
    // returned list must retain each occurrence (`new_list_borrowed`)
    // instead of taking unretained ownership (`new_list`) — otherwise a
    // repeated pick makes `release_contained_values` release the same
    // object once per occurrence on drop, a use-after-free/double-free on
    // duplicate picks (the Cranelift-JIT-opt1 malloc-corruption crash of
    // #2539).
    MbValue::from_ptr(MbObject::new_list_borrowed(out))
}

/// Materialise a non-list population (Str/Tuple) into a `Vec<MbValue>` for
/// selection. Str yields one 1-character string per code point.
fn population_as_items(pop: MbValue) -> Vec<MbValue> {
    pop.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => items.clone(),
                ObjData::Str(s) => s
                    .chars()
                    .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                    .collect(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// Build a cumulative table from a `weights` sequence of length `n`.
/// `None` when the sequence length mismatches (validated earlier) or the
/// value is not a numeric sequence.
fn cumulative_from_weights(weights: MbValue, n: usize) -> Option<Vec<f64>> {
    let items = weights.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().to_vec()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    })?;
    if items.len() != n {
        return None;
    }
    let mut acc = 0.0;
    let mut cum = Vec::with_capacity(n);
    for v in &items {
        acc += extract_f64(*v, 0.0);
        cum.push(acc);
    }
    Some(cum)
}

/// Build a cumulative table directly from a `cum_weights` sequence.
fn cumulative_from_cum_weights(cum_weights: MbValue, n: usize) -> Option<Vec<f64>> {
    let items = cum_weights.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => Some(lock.read().unwrap().to_vec()),
            ObjData::Tuple(items) => Some(items.clone()),
            _ => None,
        }
    })?;
    if items.len() != n {
        return None;
    }
    Some(items.iter().map(|v| extract_f64(*v, 0.0)).collect())
}

/// Standard-normal sample via Box–Muller (cached spare).
fn standard_normal(id: u64) -> f64 {
    if let Some(s) = GAUSS_SPARE.with(|c| c.replace(None)) {
        return s;
    }
    // Reject u1==0 to avoid ln(0).
    let mut u1 = next_f64(id);
    while u1 <= f64::EPSILON {
        u1 = next_f64(id);
    }
    let u2 = next_f64(id);
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    let z0 = r * theta.cos();
    let z1 = r * theta.sin();
    GAUSS_SPARE.with(|c| c.set(Some(z1)));
    z0
}

/// CPython's `Random.gauss`: two-draw trigonometric Box–Muller with a spare
/// cached on the owning handle. This is intentionally separate from
/// `standard_normal`, whose legacy process-local cache serves non-gauss APIs.
fn gauss_standard_normal(id: u64) -> f64 {
    if let Some(spare) = GAUSS_SPARES.with(|c| c.borrow_mut().remove(&id)) {
        return spare;
    }
    let x2pi = next_f64(id) * std::f64::consts::TAU;
    let g2rad = (-2.0 * (1.0 - next_f64(id)).ln()).sqrt();
    let z = x2pi.cos() * g2rad;
    let spare = x2pi.sin() * g2rad;
    GAUSS_SPARES.with(|c| {
        c.borrow_mut().insert(id, spare);
    });
    z
}

pub fn mb_random_method_gauss(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let s = extract_f64(sigma, 1.0);
    MbValue::from_float(m + s * gauss_standard_normal(id))
}

pub fn mb_random_method_normalvariate(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let s = extract_f64(sigma, 1.0);
    MbValue::from_float(m + s * standard_normal(id))
}

pub fn mb_random_method_expovariate(receiver: MbValue, lambd: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let lam = extract_f64(lambd, 1.0);
    let mut u = next_f64(id);
    while u <= f64::EPSILON {
        u = next_f64(id);
    }
    MbValue::from_float(-u.ln() / lam)
}

pub fn mb_random_method_lognormvariate(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let s = extract_f64(sigma, 1.0);
    MbValue::from_float((m + s * standard_normal(id)).exp())
}

pub fn mb_random_method_vonmisesvariate(receiver: MbValue, mu: MbValue, kappa: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let k = extract_f64(kappa, 0.0);
    if k < 1e-6 {
        return MbValue::from_float(2.0 * std::f64::consts::PI * next_f64(id));
    }
    // Best–Fisher 1979 — iteration cap to stay deterministic.
    let s = 0.5 / k;
    let r = s + (1.0 + s * s).sqrt();
    let mut theta = 0.0_f64;
    for _ in 0..32 {
        let u1 = next_f64(id);
        let z = (std::f64::consts::PI * u1).cos();
        let f = (1.0 + r * z) / (r + z);
        let c = k * (r - f);
        let u2 = next_f64(id);
        if u2 < c * (2.0 - c) || u2 <= c * (1.0_f64 - c).exp() {
            let u3 = next_f64(id);
            let sign = if u3 > 0.5 { 1.0 } else { -1.0 };
            theta = (m + sign * f.acos()).rem_euclid(2.0 * std::f64::consts::PI);
            break;
        }
        theta = m;
    }
    MbValue::from_float(theta)
}

pub fn mb_random_method_gammavariate(receiver: MbValue, alpha: MbValue, beta: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // CPython: gammavariate requires alpha > 0 and beta > 0.
    let raw_a = extract_f64(alpha, 1.0);
    let raw_b = extract_f64(beta, 1.0);
    if raw_a <= 0.0 || raw_b <= 0.0 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "gammavariate: alpha and beta must be > 0.0".to_string(),
            )),
        );
        return MbValue::none();
    }
    let a = raw_a.max(f64::EPSILON);
    let b = raw_b;
    // Marsaglia–Tsang 2000, with α<1 handled via boost trick.
    let val = if a < 1.0 {
        let g = sample_gamma(id, a + 1.0);
        let u = next_f64(id).max(f64::EPSILON);
        g * u.powf(1.0 / a)
    } else {
        sample_gamma(id, a)
    };
    MbValue::from_float(val * b)
}

fn sample_gamma(id: u64, alpha: f64) -> f64 {
    let d = alpha - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let x = standard_normal(id);
        let v_pre = 1.0 + c * x;
        if v_pre <= 0.0 {
            continue;
        }
        let v = v_pre.powi(3);
        let u = next_f64(id);
        if u < 1.0 - 0.0331 * x.powi(4) {
            return d * v;
        }
        if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
            return d * v;
        }
    }
}

pub fn mb_random_method_betavariate(receiver: MbValue, alpha: MbValue, beta: MbValue) -> MbValue {
    let g1 = mb_random_method_gammavariate(receiver, alpha, MbValue::from_float(1.0))
        .as_float()
        .unwrap_or(0.0);
    if g1 == 0.0 {
        return MbValue::from_float(0.0);
    }
    let g2 = mb_random_method_gammavariate(receiver, beta, MbValue::from_float(1.0))
        .as_float()
        .unwrap_or(0.0);
    MbValue::from_float(g1 / (g1 + g2))
}

/// `binomialvariate(n=1, p=0.5)` — number of successes in `n` independent
/// trials, each succeeding with probability `p`. Returns an int in `[0, n]`.
/// CPython raises ValueError for `n < 0` or `p` outside `[0, 1]`.
pub fn mb_random_method_binomialvariate(receiver: MbValue, n: MbValue, p: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // Keyword calls (`binomialvariate(**kwargs)` / `binomialvariate(n, p=x)`)
    // pack the keywords into a dict occupying a positional slot.
    let mut n = n;
    let mut p = p;
    for slot in [n, p] {
        if is_dict_value(slot) {
            if slot.to_bits() == n.to_bits() {
                n = MbValue::none();
            }
            if slot.to_bits() == p.to_bits() {
                p = MbValue::none();
            }
            if let Some(x) = kwarg_get(slot, "n") {
                n = x;
            }
            if let Some(x) = kwarg_get(slot, "p") {
                p = x;
            }
        }
    }
    let n_trials = extract_i64(n, 1);
    let prob = extract_f64(p, 0.5);
    if n_trials < 0 {
        return raise_value_error("n must be non-negative");
    }
    if !(0.0..=1.0).contains(&prob) {
        return raise_value_error("p must be in the range 0.0 <= p <= 1.0");
    }
    // Simple inversion: count Bernoulli successes. n is small in practice for
    // these fixtures; distribution-correct though not CPython's BTPE path.
    if prob <= 0.0 {
        return MbValue::from_int(0);
    }
    if prob >= 1.0 {
        return MbValue::from_int(n_trials);
    }
    let mut successes: i64 = 0;
    for _ in 0..n_trials {
        if next_f64(id) < prob {
            successes += 1;
        }
    }
    MbValue::from_int(successes)
}

pub fn mb_random_method_paretovariate(receiver: MbValue, alpha: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let a = extract_f64(alpha, 1.0).max(f64::EPSILON);
    let mut u = next_f64(id);
    while u <= f64::EPSILON {
        u = next_f64(id);
    }
    MbValue::from_float((1.0 - u).powf(-1.0 / a))
}

pub fn mb_random_method_weibullvariate(
    receiver: MbValue,
    alpha: MbValue,
    beta: MbValue,
) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let a = extract_f64(alpha, 1.0);
    let b = extract_f64(beta, 1.0).max(f64::EPSILON);
    let mut u = next_f64(id);
    while u <= f64::EPSILON {
        u = next_f64(id);
    }
    MbValue::from_float(a * (-u.ln()).powf(1.0 / b))
}

pub fn mb_random_method_getrandbits(receiver: MbValue, k: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // CPython requires an integer argument: a float raises TypeError
    // ("'float' object cannot be interpreted as an integer"); a negative
    // count raises ValueError ("number of bits must be non-negative").
    if k.is_float() {
        return raise_type_error("'float' object cannot be interpreted as an integer");
    }
    let raw_bits = extract_i64(k, 32);
    if raw_bits < 0 {
        return raise_value_error("number of bits must be non-negative");
    }
    if raw_bits == 0 {
        return MbValue::from_int(0);
    }
    let Some(bits) = usize::try_from(raw_bits).ok() else {
        return raise_value_error("number of bits is too large");
    };
    if bits <= 32 {
        return MbValue::from_int((next_u32(id) >> (32 - bits)) as i64);
    }
    let word_count = ((bits - 1) / 32) + 1;
    let partial_bits = bits % 32;
    let mut words = Vec::with_capacity(word_count);
    for index in 0..word_count {
        let mut word = next_u32(id);
        if index + 1 == word_count && partial_bits != 0 {
            word >>= 32 - partial_bits;
        }
        words.push(word);
    }
    super::super::bigint_ops::normalize_bigint(BigInt::from_slice(Sign::Plus, &words))
}

pub fn mb_random_method_randbytes(receiver: MbValue, n: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let raw_n = extract_i64(n, 0);
    // CPython: randbytes(n) maps to getrandbits(n*8); a negative count
    // raises ValueError ("number of bytes must be non-negative").
    if raw_n < 0 {
        return raise_value_error("number of bytes must be non-negative");
    }
    let count = raw_n as usize;
    let mut buf = Vec::with_capacity(count);
    let mut remaining = count;
    while remaining >= 4 {
        buf.extend_from_slice(&next_u32(id).to_le_bytes());
        remaining -= 4;
    }
    if remaining > 0 {
        let bytes = next_u32(id).to_le_bytes();
        buf.extend_from_slice(&bytes[..remaining]);
    }
    MbValue::from_ptr(MbObject::new_bytes(buf))
}

/// `getstate()` → `(handle_id,)` 1-tuple — opaque token. Restore via
/// `setstate`, valid for the lifetime of the handle.
thread_local! {
    /// getstate() snapshots: cloned generator states keyed by snapshot id.
    static SAVED_STATES: std::cell::RefCell<HashMap<u64, Mt>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_STATE_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

pub fn mb_random_method_getstate(receiver: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    // Snapshot the live generator so setstate() can rewind exactly.
    let snapshot = RANDOMS.with(|m| m.borrow().get(&id).cloned());
    let state_id = NEXT_STATE_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    if let Some(rng) = snapshot {
        SAVED_STATES.with(|m| {
            m.borrow_mut().insert(state_id, rng);
        });
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(3),
        MbValue::from_int(state_id as i64),
    ]))
}

/// Pickle bridge — dumps(): snapshot the handle's live generator and return
/// an opaque state id (same registry getstate() uses). Same-process loads()
/// rehydrates from it; cross-process pickles are out of scope for this shim.
pub fn pickle_snapshot(id: u64) -> Option<u64> {
    let snapshot = RANDOMS.with(|m| m.borrow().get(&id).cloned())?;
    let state_id = NEXT_STATE_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    SAVED_STATES.with(|m| {
        m.borrow_mut().insert(state_id, snapshot);
    });
    Some(state_id)
}

/// Pickle bridge — loads(): build a fresh handle whose generator continues
/// from the snapshot. The snapshot stays registered so repeated loads() of
/// the same blob each get an identical stream.
pub fn pickle_restore(state_id: u64) -> Option<MbValue> {
    let saved = SAVED_STATES.with(|m| m.borrow().get(&state_id).cloned())?;
    let id = make_handle(None);
    RANDOMS.with(|m| {
        m.borrow_mut().insert(id, saved);
    });
    Some(MbValue::from_int(id as i64))
}

/// `setstate(state)` — restore the generator snapshotted by getstate().
pub fn mb_random_method_setstate(receiver: MbValue, state: MbValue) -> MbValue {
    let id = receiver
        .as_int()
        .map(|i| i as u64)
        .unwrap_or_else(default_handle);
    let state_id = state.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Tuple(items) => items.get(1).and_then(|v| v.as_int()),
            ObjData::List(lock) => lock
                .read()
                .ok()
                .and_then(|g| g.get(1).and_then(|v| v.as_int())),
            _ => None,
        }
    });
    if let Some(sid) = state_id {
        let saved = SAVED_STATES.with(|m| m.borrow().get(&(sid as u64)).cloned());
        if let Some(rng) = saved {
            RANDOMS.with(|m| {
                m.borrow_mut().insert(id, rng);
            });
        }
    }
    MbValue::none()
}

// ── Flat-args dispatch thunks (module-level fn entry points) ──

unsafe extern "C" fn dispatch_random(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_random_method_random(MbValue::none())
}
unsafe extern "C" fn dispatch_seed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // seed(a=None, version=2): more than two positionals is a TypeError.
    let positional = a.iter().filter(|v| !is_dict_value(**v)).count();
    if positional > 2 {
        return raise_type_error(&format!(
            "seed() takes from 1 to 3 positional arguments but {} were given",
            positional + 1
        ));
    }
    mb_random_method_seed(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_randint(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_randint(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_randrange(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_randrange(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(|| MbValue::from_int(1)),
    )
}
unsafe extern "C" fn dispatch_uniform(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_uniform(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_triangular(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_triangular(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_choice(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_choice(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_shuffle(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_shuffle(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_sample(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_sample(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
extern "C" fn random_instance_shuffle(receiver: MbValue, lst: MbValue) -> MbValue {
    mb_random_method_shuffle(handle_for_instance(receiver), lst)
}
extern "C" fn random_instance_sample(receiver: MbValue, pop: MbValue, k: MbValue) -> MbValue {
    mb_random_method_sample(handle_for_instance(receiver), pop, k)
}
unsafe extern "C" fn dispatch_choices(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (weights, cum_weights, k) = parse_choices_kwargs(a);
    mb_random_method_choices_full(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        weights,
        cum_weights,
        k,
    )
}

/// Resolve `weights` / `cum_weights` / `k` for a module-level
/// `random.choices(population, weights=None, *, cum_weights=None, k=1)` call.
/// The lowering folds keyword args into a trailing `ObjData::Dict`; a bare
/// `choices(pop, weights_seq)` supplies the weight sequence positionally at
/// index 1. Returns `(weights, cum_weights, k)` with `none()` for absent
/// optionals.
pub(crate) fn parse_choices_kwargs(a: &[MbValue]) -> (MbValue, MbValue, MbValue) {
    let mut weights = MbValue::none();
    let mut cum_weights = MbValue::none();
    let mut k = MbValue::from_int(1);

    // Trailing kwargs dict (folded keyword arguments).
    if let Some(&last) = a.last() {
        if is_dict_value(last) {
            if let Some(w) = kwarg_get(last, "weights") {
                weights = w;
            }
            if let Some(c) = kwarg_get(last, "cum_weights") {
                cum_weights = c;
            }
            if let Some(kk) = kwarg_get(last, "k") {
                k = kk;
            }
        }
    }

    // Positional index 1 is `weights` in the CPython signature — including a
    // scalar, which the validation downstream rejects with TypeError.
    if let Some(&pos1) = a.get(1) {
        if !is_dict_value(pos1) && weights.is_none() && cum_weights.is_none() {
            weights = pos1;
        }
    }

    (weights, cum_weights, k)
}
unsafe extern "C" fn dispatch_gauss(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_gauss(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_normalvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_normalvariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_expovariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_expovariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_lognormvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_lognormvariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_vonmisesvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_vonmisesvariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.0)),
    )
}
unsafe extern "C" fn dispatch_gammavariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_gammavariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_betavariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_betavariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_binomialvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_binomialvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_int(1)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(0.5)),
    )
}
unsafe extern "C" fn dispatch_paretovariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_paretovariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_weibullvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_weibullvariate(
        MbValue::none(),
        a.first()
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1)
            .copied()
            .unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_getrandbits(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_getrandbits(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_int(32)),
    )
}
unsafe extern "C" fn dispatch_randbytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_randbytes(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_int(0)),
    )
}
unsafe extern "C" fn dispatch_getstate(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_random_method_getstate(MbValue::none())
}
unsafe extern "C" fn dispatch_setstate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_setstate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
    )
}

// ── User subclasses of random.Random ──
//
// A `class T(random.Random)` instance is a plain Instance with no generator
// state; mb_call_method routes its method calls here. CPython's
// `__init_subclass__` picks the `_randbelow` strategy from the FIRST class in
// the MRO that defines `getrandbits` or `random`; we re-derive that per call.

/// The 24 instance methods the native handle protocol understands.
pub fn is_random_method_name(name: &str) -> bool {
    matches!(
        name,
        "random"
            | "seed"
            | "randint"
            | "randrange"
            | "uniform"
            | "triangular"
            | "choice"
            | "shuffle"
            | "sample"
            | "choices"
            | "gauss"
            | "normalvariate"
            | "expovariate"
            | "lognormvariate"
            | "vonmisesvariate"
            | "gammavariate"
            | "betavariate"
            | "paretovariate"
            | "weibullvariate"
            | "getrandbits"
            | "randbytes"
            | "getstate"
            | "setstate"
            | "binomialvariate"
    )
}

/// Lazily allocate (and cache on the instance) a native generator handle for
/// a user `random.Random` subclass instance.
pub fn handle_for_instance(recv: MbValue) -> MbValue {
    if let Some(ptr) = recv.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(h) = fields.read().unwrap().get("__random_handle__") {
                    return *h;
                }
                let id = make_handle(None);
                let h = MbValue::from_int(id as i64);
                fields
                    .write()
                    .unwrap()
                    .insert("__random_handle__".to_string(), h);
                return h;
            }
        }
    }
    recv
}

enum RandBelow {
    Getrandbits,
    Random,
    Native,
}

/// CPython `Random.__init_subclass__`: the first MRO class (before the native
/// base) defining `getrandbits` or `random` decides the `_randbelow` route.
fn randbelow_kind(class_name: &str) -> RandBelow {
    for cls in super::super::class::class_mro_list(class_name) {
        if cls == "Random" || cls == "SystemRandom" {
            break;
        }
        if super::super::class::class_defines_own_method(&cls, "getrandbits") {
            return RandBelow::Getrandbits;
        }
        if super::super::class::class_defines_own_method(&cls, "random") {
            return RandBelow::Random;
        }
    }
    RandBelow::Native
}

/// Is `method` user-defined anywhere in the MRO before the native base?
fn user_overrides(class_name: &str, method: &str) -> bool {
    for cls in super::super::class::class_mro_list(class_name) {
        if cls == "Random" || cls == "SystemRandom" {
            break;
        }
        if super::super::class::class_defines_own_method(&cls, method) {
            return true;
        }
    }
    false
}

fn call_self_method1(recv: MbValue, name: &str, arg: MbValue) -> MbValue {
    let n = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![arg]));
    super::super::class::mb_call_method(recv, n, args)
}

/// `_randbelow(n)` honoring user overrides (the whole point of the CPython
/// dispatch contract: an overridden getrandbits/random must be exercised).
fn randbelow_subclass(recv: MbValue, class_name: &str, n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    match randbelow_kind(class_name) {
        RandBelow::Getrandbits => {
            let k = 64 - (n as u64).leading_zeros() as i64; // n.bit_length()
            for _ in 0..10_000 {
                let r = call_self_method1(recv, "getrandbits", MbValue::from_int(k));
                let r = r.as_int_pyint().unwrap_or(0);
                if r < n {
                    return r;
                }
            }
            0
        }
        RandBelow::Random => {
            let zero_args = MbValue::from_ptr(MbObject::new_list(vec![]));
            let nm = MbValue::from_ptr(MbObject::new_str("random".to_string()));
            let f = super::super::class::mb_call_method(recv, nm, zero_args)
                .as_float()
                .unwrap_or(0.0);
            ((f * n as f64) as i64).clamp(0, n - 1)
        }
        RandBelow::Native => {
            let handle = handle_for_instance(recv);
            let id = handle
                .as_int()
                .map(|i| i as u64)
                .unwrap_or_else(default_handle);
            (next_u64(id) % n as u64) as i64
        }
    }
}

/// Method dispatch for user subclass instances of random.Random. Returns
/// None when the method is user-overridden (the generic path must call the
/// user code) or unknown.
pub fn random_subclass_method(recv: MbValue, method: &str, args: &[MbValue]) -> Option<MbValue> {
    let class_name = recv.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })?;
    if !is_random_method_name(method) {
        return None;
    }
    // A user override wins for direct calls — fall through to generic dispatch.
    if user_overrides(&class_name, method) {
        return None;
    }
    match method {
        "randrange" => {
            // randrange(stop) / randrange(start, stop[, step])
            let a0 = args.first().and_then(|v| v.as_int_pyint());
            let a1 = args.get(1).and_then(|v| v.as_int_pyint());
            let step = args.get(2).and_then(|v| v.as_int_pyint()).unwrap_or(1);
            let (start, width) = match (a0, a1) {
                (Some(stop), None) => (0, stop),
                (Some(start), Some(stop)) => (start, stop - start),
                _ => (0, 0),
            };
            if width <= 0 || step == 0 {
                return Some(raise_value_error("empty range for randrange()"));
            }
            let slots = if step == 1 {
                width
            } else {
                (width + step - 1) / step
            };
            Some(MbValue::from_int(
                start + step * randbelow_subclass(recv, &class_name, slots),
            ))
        }
        "randint" => {
            let a = args.first().and_then(|v| v.as_int_pyint()).unwrap_or(0);
            let b = args.get(1).and_then(|v| v.as_int_pyint()).unwrap_or(0);
            if b < a {
                return Some(raise_value_error("empty range for randrange()"));
            }
            Some(MbValue::from_int(
                a + randbelow_subclass(recv, &class_name, b - a + 1),
            ))
        }
        _ => {
            // Everything else: delegate to the native handle protocol (the
            // class.rs random-handle arm) through the instance's handle.
            let handle = handle_for_instance(recv);
            if !handle.is_int() {
                return None;
            }
            let nm = MbValue::from_ptr(MbObject::new_str(method.to_string()));
            let rest = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
            Some(super::super::class::mb_call_method(handle, nm, rest))
        }
    }
}

/// `Random(seed=None)` — constructor returns a handle id wrapped as int.
unsafe extern "C" fn dispatch_Random(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seed_val = a.first().copied().unwrap_or_else(MbValue::none);
    let seed = if seed_val.is_none() {
        None
    } else {
        Some(mt_from_seed_value(seed_val))
    };
    let id = make_handle(seed);
    MbValue::from_int(id as i64)
}

/// `SystemRandom()` — CPython's hardware-entropy generator. Mamba has no
/// os.urandom-backed PRNG here, so model it as an unseeded MT handle that
/// shares the integer-handle method protocol (random/getrandbits/choice/…).
/// Distribution-correct; not cryptographically strong.
unsafe extern "C" fn dispatch_SystemRandom(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    crate::icf_guard!();
    let id = make_handle(None);
    MbValue::from_int(id as i64)
}

// ── Module registration ──

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("random", dispatch_random as usize),
        ("seed", dispatch_seed as usize),
        ("randint", dispatch_randint as usize),
        ("randrange", dispatch_randrange as usize),
        ("uniform", dispatch_uniform as usize),
        ("triangular", dispatch_triangular as usize),
        ("choice", dispatch_choice as usize),
        ("shuffle", dispatch_shuffle as usize),
        ("sample", dispatch_sample as usize),
        ("choices", dispatch_choices as usize),
        ("gauss", dispatch_gauss as usize),
        ("normalvariate", dispatch_normalvariate as usize),
        ("expovariate", dispatch_expovariate as usize),
        ("lognormvariate", dispatch_lognormvariate as usize),
        ("vonmisesvariate", dispatch_vonmisesvariate as usize),
        ("gammavariate", dispatch_gammavariate as usize),
        ("betavariate", dispatch_betavariate as usize),
        ("binomialvariate", dispatch_binomialvariate as usize),
        ("paretovariate", dispatch_paretovariate as usize),
        ("weibullvariate", dispatch_weibullvariate as usize),
        ("getrandbits", dispatch_getrandbits as usize),
        ("randbytes", dispatch_randbytes as usize),
        ("getstate", dispatch_getstate as usize),
        ("setstate", dispatch_setstate as usize),
        ("Random", dispatch_Random as usize),
        ("SystemRandom", dispatch_SystemRandom as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // random.Random doubles as a subclassable base: map the constructor to
    // its class name and register a method-name table so unbound access
    // (`random.Random.getrandbits`) resolves to an __unbound_method__ wrapper
    // and user subclasses inherit the surface. The stub values are never
    // invoked — unbound dispatch routes by NAME through the handle protocol.
    for (cls, addr) in [
        ("Random", dispatch_Random as usize),
        ("SystemRandom", dispatch_SystemRandom as usize),
    ] {
        super::super::module::register_native_type_name(addr as u64, cls.to_string());
        let stub = MbValue::from_func(addr);
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for name in [
            "random",
            "seed",
            "randint",
            "randrange",
            "uniform",
            "triangular",
            "choice",
            "shuffle",
            "sample",
            "choices",
            "gauss",
            "normalvariate",
            "expovariate",
            "lognormvariate",
            "vonmisesvariate",
            "gammavariate",
            "betavariate",
            "paretovariate",
            "weibullvariate",
            "getrandbits",
            "randbytes",
            "getstate",
            "setstate",
            "binomialvariate",
        ] {
            let func = match name {
                // `object.__new__(Random)` builds a bare instance, so exact-base
                // bound method calls must resolve through the instance handle
                // instead of reusing the constructor stub.
                "shuffle" => MbValue::from_func(random_instance_shuffle as usize),
                "sample" => MbValue::from_func(random_instance_sample as usize),
                _ => stub,
            };
            methods.insert(name.to_string(), func);
        }
        super::super::class::mb_class_register(cls, vec![], methods);
    }

    // Module-level float constants (CPython exposes these in `random`).
    // `TWOPI` is used internally by vonmisesvariate; surfaced as an attr.
    attrs.insert(
        "TWOPI".to_string(),
        MbValue::from_float(2.0 * std::f64::consts::PI),
    );
    // CPython's module-level magic constants (test_random::testMagicConstants).
    attrs.insert(
        "NV_MAGICCONST".to_string(),
        MbValue::from_float(4.0 * (-0.5f64).exp() / 2.0f64.sqrt()),
    );
    attrs.insert("LOG4".to_string(), MbValue::from_float(4.0f64.ln()));
    attrs.insert(
        "SG_MAGICCONST".to_string(),
        MbValue::from_float(1.0 + 4.5f64.ln()),
    );
    attrs.insert(
        "RECIP_BPF".to_string(),
        MbValue::from_float((-53.0f64).exp2()),
    );

    // surface: missing CPython module constants (auto-added)
    attrs.insert("BPF".into(), MbValue::from_int(53));
    super::register_module("random", attrs);

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

    fn seed_default(s: u32) {
        // Reset default handle for deterministic tests.
        DEFAULT_HANDLE.with(|c| c.set(None));
        GAUSS_SPARE.with(|c| c.set(None));
        mb_random_method_seed(MbValue::none(), MbValue::from_int(s as i64));
    }

    fn random_registry_keys() -> (std::collections::HashSet<u64>, std::collections::HashSet<u64>) {
        let random_ids = RANDOM_IDS.with(|s| s.borrow().clone());
        let rng_ids = RANDOMS.with(|m| m.borrow().keys().copied().collect());
        (random_ids, rng_ids)
    }

    fn restore_random_registry(
        before: &(std::collections::HashSet<u64>, std::collections::HashSet<u64>),
    ) {
        let extra_random_ids = RANDOM_IDS.with(|s| {
            s.borrow()
                .difference(&before.0)
                .copied()
                .collect::<Vec<_>>()
        });
        for id in extra_random_ids {
            drop_random_handle(id);
        }

        let extra_rng_ids = RANDOMS.with(|m| {
            m.borrow()
                .keys()
                .filter(|id| !before.1.contains(id))
                .copied()
                .collect::<Vec<_>>()
        });
        for id in extra_rng_ids {
            RANDOMS.with(|m| {
                m.borrow_mut().remove(&id);
            });
        }
    }

    #[test]
    fn test_random_range() {
        seed_default(42);
        for _ in 0..100 {
            let f = mb_random_method_random(MbValue::none()).as_float().unwrap();
            assert!((0.0..1.0).contains(&f), "out of range: {f}");
        }
    }

    #[test]
    fn test_seed_determinism() {
        seed_default(42);
        let a = mb_random_method_random(MbValue::none()).as_float().unwrap();
        seed_default(42);
        let b = mb_random_method_random(MbValue::none()).as_float().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn test_randint_bounds() {
        seed_default(99);
        for _ in 0..100 {
            let v = mb_random_method_randint(
                MbValue::none(),
                MbValue::from_int(1),
                MbValue::from_int(6),
            )
            .as_int()
            .unwrap();
            assert!((1..=6).contains(&v));
        }
    }

    #[test]
    fn test_randbelow_uses_bit_length_rejection() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let observed = (0..6)
            .map(|_| {
                mb_random_method_randint(handle, MbValue::from_int(1), MbValue::from_int(100))
                    .as_int()
            })
            .collect::<Vec<_>>();
        let next = next_u32(id);
        drop_random_handle(id);

        assert_eq!(observed, vec![Some(100), Some(57), Some(15), Some(1), Some(12), Some(75)]);
        assert_eq!(next, 150006740);
    }

    #[test]
    fn test_randrange_step_uses_ceiling_cardinality() {
        let handle_a = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id_a = handle_a.as_int().unwrap() as u64;
        let observed_a = (0..6)
            .map(|_| {
                mb_random_method_randrange(
                    handle_a,
                    MbValue::from_int(10),
                    MbValue::from_int(100),
                    MbValue::from_int(3),
                )
                .as_int()
            })
            .collect::<Vec<_>>();
        let next_a = next_u32(id_a);

        let handle_b = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id_b = handle_b.as_int().unwrap() as u64;
        let observed_b = (0..20)
            .map(|_| {
                mb_random_method_randrange(
                    handle_b,
                    MbValue::from_int(0),
                    MbValue::from_int(5),
                    MbValue::from_int(2),
                )
                .as_int()
            })
            .collect::<Vec<_>>();
        let mut unique_b = observed_b.clone();
        unique_b.sort_unstable();
        unique_b.dedup();
        let next_b = next_u32(id_b);

        drop_random_handle(id_a);
        drop_random_handle(id_b);

        assert_eq!(
            observed_a,
            vec![Some(82), Some(52), Some(19), Some(10), Some(16), Some(97)],
        );
        assert_eq!(next_a, 4048155970);
        assert_eq!(
            observed_b,
            vec![
                Some(2),
                Some(0),
                Some(0),
                Some(0),
                Some(4),
                Some(0),
                Some(4),
                Some(4),
                Some(0),
                Some(0),
                Some(2),
                Some(0),
                Some(0),
                Some(0),
                Some(0),
                Some(2),
                Some(4),
                Some(4),
                Some(2),
                Some(4),
            ],
        );
        assert_eq!(unique_b, vec![Some(0), Some(2), Some(4)]);
        assert_eq!(next_b, 1990923381);
    }

    #[test]
    fn test_randint_equal_bounds() {
        seed_default(1);
        let v =
            mb_random_method_randint(MbValue::none(), MbValue::from_int(5), MbValue::from_int(5));
        assert_eq!(v.as_int(), Some(5));
    }

    #[test]
    fn test_randint_reversed_bounds_returns_none() {
        let v =
            mb_random_method_randint(MbValue::none(), MbValue::from_int(10), MbValue::from_int(1));
        assert!(v.is_none());
    }

    #[test]
    fn test_uniform_range() {
        seed_default(55);
        for _ in 0..50 {
            let f = mb_random_method_uniform(
                MbValue::none(),
                MbValue::from_float(10.0),
                MbValue::from_float(20.0),
            )
            .as_float()
            .unwrap();
            assert!((10.0..=20.0).contains(&f), "out of range: {f}");
        }
    }

    #[test]
    fn test_choice_and_sample() {
        seed_default(7);
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ]));
        let v = mb_random_method_choice(MbValue::none(), list)
            .as_int()
            .unwrap();
        assert!(v == 10 || v == 20 || v == 30);
        let s = mb_random_method_sample(MbValue::none(), list, MbValue::from_int(2));
        unsafe {
            if let ObjData::List(ref lk) = (*s.as_ptr().unwrap()).data {
                assert_eq!(lk.read().unwrap().len(), 2);
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_choice_empty_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        let v = mb_random_method_choice(MbValue::none(), list);
        assert!(v.is_none());
    }

    #[test]
    fn test_sample_exceeds_length() {
        // CPython: random.sample(pop, k) with k > len(pop) raises ValueError
        // ("Sample larger than population or is negative") — it does NOT return
        // an empty list. The impl is already CPython-correct (raises); this test
        // previously asserted the wrong empty-list behavior and unwrap-panicked
        // on the (correct) none return. Assert the rejection instead.
        super::super::super::exception::mb_clear_exception();
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let r = mb_random_method_sample(MbValue::none(), list, MbValue::from_int(5));
        assert!(r.is_none(), "over-sized sample must not return a value");
        assert_eq!(
            super::super::super::exception::current_exception_type().as_deref(),
            Some("ValueError"),
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_sample_rejects_non_sequence_instance() {
        super::super::super::exception::mb_clear_exception();
        let not_sequence = MbValue::from_ptr(MbObject::new_instance("_W".to_string()));
        let r = mb_random_method_sample(MbValue::none(), not_sequence, MbValue::from_int(0));
        assert!(
            r.is_none(),
            "wrong-typed sample population must not return a value"
        );
        assert_eq!(
            super::super::super::exception::current_exception_type().as_deref(),
            Some("TypeError"),
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_sample_reversed_iterator_rejects_without_draw_and_preserves_tuple_owners() {
        super::super::super::exception::mb_clear_exception();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let left_ptr = MbObject::new_str("left".to_string());
        let right_ptr = MbObject::new_str("right".to_string());
        let population = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(left_ptr),
            MbValue::from_ptr(right_ptr),
        ]));

        // The tuple owns one reference to each string.  Keep one independent
        // observer reference so a red iterator-drain path cannot make either
        // pointer uninspectable before the ownership snapshot.
        unsafe {
            super::super::super::rc::retain_if_ptr(MbValue::from_ptr(left_ptr));
            super::super::super::rc::retain_if_ptr(MbValue::from_ptr(right_ptr));
        }
        assert_eq!(
            unsafe {
                (
                    super::super::super::rc::mb_refcount(left_ptr),
                    super::super::super::rc::mb_refcount(right_ptr),
                )
            },
            (2, 2),
        );

        let reversed = super::super::super::iter::mb_reversed(population);
        let returned = mb_random_method_sample(handle, reversed, MbValue::from_int(1));
        let returned_none = returned.is_none();
        let exception_type = super::super::super::exception::current_exception_type();

        // All snapshots happen while the tuple and observer references remain
        // live.  In particular, do not inspect a returned list after releasing
        // its selected element's source owners.
        let returned_items = returned
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => Some(
                        lock.read()
                            .unwrap()
                            .iter()
                            .filter_map(MbValue::as_ptr)
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                }
            })
            .unwrap_or_default();
        let strings = unsafe {
            (
                match &(*left_ptr).data {
                    ObjData::Str(value) => Some(value.clone()),
                    _ => None,
                },
                match &(*right_ptr).data {
                    ObjData::Str(value) => Some(value.clone()),
                    _ => None,
                },
            )
        };
        let refcounts = unsafe {
            (
                super::super::super::rc::mb_refcount(left_ptr),
                super::super::super::rc::mb_refcount(right_ptr),
            )
        };
        let sentinel = next_u32(id);

        // The known-red candidate may return normally and release the
        // reversed iterator's borrowed tuple items as if they were owned.  Add
        // only cleanup-only retains after recording the raw snapshot, then
        // compensate for any returned-list element before releasing that list.
        // This keeps both the red and fixed paths free of UAF/double-release.
        unsafe {
            for (ptr, rc) in [(left_ptr, refcounts.0), (right_ptr, refcounts.1)] {
                for _ in rc..2 {
                    super::super::super::rc::retain_if_ptr(MbValue::from_ptr(ptr));
                }
            }
            for ptr in &returned_items {
                super::super::super::rc::retain_if_ptr(MbValue::from_ptr(*ptr));
            }
            super::super::super::exception::mb_clear_exception();
            super::super::super::iter::mb_iter_release(reversed);
            super::super::super::rc::release_if_ptr(returned);
            super::super::super::rc::release_if_ptr(population);
            super::super::super::rc::release_if_ptr(MbValue::from_ptr(left_ptr));
            super::super::super::rc::release_if_ptr(MbValue::from_ptr(right_ptr));
        }
        drop_random_handle(id);

        assert!(
            returned_none,
            "sample(reversed_tuple, 1) returned normally; fingerprint: exception={exception_type:?}, sentinel={sentinel}, strings={strings:?}, refcounts={refcounts:?}",
        );
        assert_eq!(exception_type.as_deref(), Some("TypeError"));
        assert_eq!(sentinel, 4_150_886_329);
        assert_eq!(strings, (Some("left".to_string()), Some("right".to_string())));
        assert_eq!(refcounts, (2, 2));
    }

    #[test]
    fn test_sample_accepts_range_handle_population() {
        seed_default(17);
        let range =
            super::super::super::builtins::mb_range_2(MbValue::from_int(0), MbValue::from_int(5));
        let s = mb_random_method_sample(MbValue::none(), range, MbValue::from_int(2));
        unsafe {
            if let ObjData::List(ref lk) = (*s.as_ptr().unwrap()).data {
                assert_eq!(lk.read().unwrap().len(), 2);
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_shuffle_preserves_elements() {
        seed_default(100);
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
            MbValue::from_int(5),
        ]));
        mb_random_method_shuffle(MbValue::none(), list);
        unsafe {
            if let ObjData::List(ref lk) = (*list.as_ptr().unwrap()).data {
                let g = lk.read().unwrap();
                assert_eq!(g.len(), 5);
                let mut sorted: Vec<i64> = g.to_vec().into_iter().map(|v| v.as_int().unwrap()).collect();
                sorted.sort();
                assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
            }
        }
    }

    #[test]
    fn test_shuffle_rejects_non_mutable_sequence_instance() {
        super::super::super::exception::mb_clear_exception();
        let not_mutable_sequence = MbValue::from_ptr(MbObject::new_instance("_W".to_string()));
        let r = mb_random_method_shuffle(MbValue::none(), not_mutable_sequence);
        assert!(
            r.is_none(),
            "wrong-typed shuffle input must not return a value"
        );
        assert_eq!(
            super::super::super::exception::current_exception_type().as_deref(),
            Some("TypeError"),
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_gauss_finite() {
        seed_default(11);
        for _ in 0..50 {
            let f = mb_random_method_gauss(
                MbValue::none(),
                MbValue::from_float(0.0),
                MbValue::from_float(1.0),
            )
            .as_float()
            .unwrap();
            assert!(f.is_finite());
        }
    }

    #[test]
    fn test_expovariate_positive() {
        seed_default(12);
        for _ in 0..50 {
            let f = mb_random_method_expovariate(MbValue::none(), MbValue::from_float(1.5))
                .as_float()
                .unwrap();
            assert!(f > 0.0 && f.is_finite());
        }
    }

    #[test]
    fn test_gammavariate_positive() {
        seed_default(13);
        for _ in 0..30 {
            let f = mb_random_method_gammavariate(
                MbValue::none(),
                MbValue::from_float(2.0),
                MbValue::from_float(1.0),
            )
            .as_float()
            .unwrap();
            assert!(f > 0.0 && f.is_finite());
        }
    }

    #[test]
    fn test_betavariate_unit() {
        seed_default(14);
        for _ in 0..30 {
            let f = mb_random_method_betavariate(
                MbValue::none(),
                MbValue::from_float(2.0),
                MbValue::from_float(5.0),
            )
            .as_float()
            .unwrap();
            assert!((0.0..=1.0).contains(&f), "out of [0,1]: {f}");
        }
    }

    #[test]
    fn test_getrandbits_range() {
        seed_default(15);
        // Mamba MbValue ints are 48-bit; impl caps k at 47.
        for k in [1, 8, 16, 32, 47] {
            let v = mb_random_method_getrandbits(MbValue::none(), MbValue::from_int(k))
                .as_int()
                .unwrap();
            assert!(v >= 0);
            assert!((v as u64) < (1_u64 << k));
        }
    }

    #[test]
    fn test_randbytes_length() {
        seed_default(16);
        let b = mb_random_method_randbytes(MbValue::none(), MbValue::from_int(10));
        unsafe {
            if let ObjData::Bytes(ref bs) = (*b.as_ptr().unwrap()).data {
                assert_eq!(bs.len(), 10);
            }
        }
    }

    #[test]
    fn test_random_class_handle_is_distinct() {
        let h1 = unsafe { dispatch_Random([MbValue::from_int(42)].as_ptr(), 1) };
        let h2 = unsafe { dispatch_Random([MbValue::from_int(43)].as_ptr(), 1) };
        let id1 = h1.as_int().unwrap() as u64;
        let id2 = h2.as_int().unwrap() as u64;
        assert!(is_random_handle(id1));
        assert!(is_random_handle(id2));
        assert_ne!(id1, id2);
        // Distinct seeds produce distinct streams.
        let a = mb_random_method_random(h1).as_float().unwrap();
        let b = mb_random_method_random(h2).as_float().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_random_constructor_rejects_user_instance_without_handle_allocation() {
        super::super::super::exception::mb_clear_exception();
        let saved_default = DEFAULT_HANDLE.with(|c| c.get());
        let before = random_registry_keys();
        let instance = MbValue::from_ptr(MbObject::new_instance("UnsupportedSeed".to_string()));

        let returned = unsafe { dispatch_Random([instance].as_ptr(), 1) };
        let returned_none = returned.is_none();
        let exception_type = super::super::super::exception::current_exception_type();
        let after = random_registry_keys();
        let returned_handle = returned
            .as_int()
            .map(|id| id as u64)
            .filter(|id| is_random_handle(*id));

        restore_random_registry(&before);
        DEFAULT_HANDLE.with(|c| c.set(saved_default));
        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(instance) };

        assert!(returned_none, "Random(instance) returned a value");
        assert_eq!(exception_type.as_deref(), Some("TypeError"));
        assert_eq!(after, before, "constructor allocated a random handle");
        assert!(returned_handle.is_none() || !is_random_handle(returned_handle.unwrap()));
    }

    #[test]
    fn test_random_constructor_rejects_range_and_range_iterator_without_allocation_or_consumption() {
        super::super::super::exception::mb_clear_exception();
        let saved_default = DEFAULT_HANDLE.with(|c| c.get());
        let before = random_registry_keys();
        let range = super::super::super::builtins::mb_range_2(
            MbValue::from_int(0),
            MbValue::from_int(3),
        );
        let range_iter = super::super::super::iter::mb_iter(range);
        let range_is_genuine = super::super::super::iter::is_range_handle(range);
        let range_iter_is_iterator = super::super::super::iter::is_iter_handle(range_iter);
        let observations = [range, range_iter]
            .into_iter()
            .map(|seed| {
                super::super::super::exception::mb_clear_exception();
                let returned = unsafe { dispatch_Random([seed].as_ptr(), 1) };
                let returned_none = returned.is_none();
                let exception_type = super::super::super::exception::current_exception_type();
                let after = random_registry_keys();
                restore_random_registry(&before);
                super::super::super::exception::mb_clear_exception();
                (returned_none, exception_type, after)
            })
            .collect::<Vec<_>>();
        let range_next = super::super::super::iter::mb_next(range).as_int();
        let range_iter_next = super::super::super::iter::mb_next(range_iter).as_int();

        super::super::super::iter::mb_iter_release(range_iter);
        super::super::super::iter::mb_iter_release(range);
        DEFAULT_HANDLE.with(|c| c.set(saved_default));
        super::super::super::exception::mb_clear_exception();

        assert!(range_is_genuine);
        assert!(range_iter_is_iterator);
        assert_eq!(
            observations
                .iter()
                .map(|(returned_none, exception_type, after)| {
                    (*returned_none, exception_type.as_deref(), after == &before)
                })
                .collect::<Vec<_>>(),
            vec![(true, Some("TypeError"), true), (true, Some("TypeError"), true)],
        );
        assert_eq!((range_next, range_iter_next), (Some(0), Some(0)));
    }

    #[test]
    fn test_random_seed_rejects_range_and_range_iterator_without_state_change_or_consumption() {
        let saved_default = DEFAULT_HANDLE.with(|c| c.get());
        let observations = (0..2)
            .map(|case| {
                super::super::super::exception::mb_clear_exception();
                let range = super::super::super::builtins::mb_range_2(
                    MbValue::from_int(0),
                    MbValue::from_int(3),
                );
                let range_iter = super::super::super::iter::mb_iter(range);
                let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
                let id = handle.as_int().expect("Random(1234) must return a handle") as u64;
                let before = random_registry_keys();
                let seed = if case == 0 { range } else { range_iter };
                let returned = mb_random_method_seed(handle, seed);
                let returned_none = returned.is_none();
                let exception_type = super::super::super::exception::current_exception_type();
                let after = random_registry_keys();
                let following = next_u32(id);
                let range_next = super::super::super::iter::mb_next(range).as_int();
                let range_iter_next = super::super::super::iter::mb_next(range_iter).as_int();

                restore_random_registry(&before);
                drop_random_handle(id);
                super::super::super::iter::mb_iter_release(range_iter);
                super::super::super::iter::mb_iter_release(range);
                super::super::super::exception::mb_clear_exception();

                (
                    returned_none,
                    exception_type,
                    after == before,
                    following,
                    range_next,
                    range_iter_next,
                )
            })
            .collect::<Vec<_>>();
        DEFAULT_HANDLE.with(|c| c.set(saved_default));
        super::super::super::exception::mb_clear_exception();

        assert_eq!(
            observations
                .iter()
                .map(|(returned_none, exception_type, no_registry_change, following, range_next, range_iter_next)| {
                    (
                        *returned_none,
                        exception_type.as_deref(),
                        *no_registry_change,
                        *following,
                        *range_next,
                        *range_iter_next,
                    )
                })
                .collect::<Vec<_>>(),
            vec![
                (true, Some("TypeError"), true, 4_150_886_329, Some(0), Some(0)),
                (true, Some("TypeError"), true, 4_150_886_329, Some(0), Some(0)),
            ],
        );
    }

    #[test]
    fn test_random_constructor_accepts_heap_bigint_seed_control() {
        super::super::super::exception::mb_clear_exception();
        let saved_default = DEFAULT_HANDLE.with(|c| c.get());
        let before = random_registry_keys();
        let seed = super::super::super::bigint_ops::bigint_from_literal(
            "100000000000000000000",
        );

        let returned = unsafe { dispatch_Random([seed].as_ptr(), 1) };
        let exception_type = super::super::super::exception::current_exception_type();
        let returned_id = returned.as_int().map(|id| id as u64);
        let returned_is_handle = returned_id.is_some_and(is_random_handle);
        let observed = returned_id.map(|id| (next_u32(id), next_u32(id)));
        let after = random_registry_keys();

        restore_random_registry(&before);
        DEFAULT_HANDLE.with(|c| c.set(saved_default));
        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(seed) };

        assert!(exception_type.is_none(), "BigInt seed raised {exception_type:?}");
        assert!(returned_is_handle, "BigInt constructor did not return a handle");
        assert_eq!(
            observed,
            Some((78_147_056, 1_939_873_431)),
        );
        assert_ne!(after, before, "BigInt control did not allocate a handle");
        assert_eq!(random_registry_keys(), before);
    }

    #[test]
    fn test_seed_int_uses_cpython_init_by_array_key() {
        let mut observed = Vec::new();
        let sample_inline_seed = |seed: i64, observed: &mut Vec<(u32, u32)>| {
            let handle = unsafe { dispatch_Random([MbValue::from_int(seed)].as_ptr(), 1) };
            let id = handle.as_int().unwrap() as u64;
            observed.push((next_u32(id), next_u32(id)));
            drop_random_handle(id);
        };

        sample_inline_seed(0, &mut observed);
        sample_inline_seed(42, &mut observed);
        sample_inline_seed(1234, &mut observed);
        sample_inline_seed(-1, &mut observed);
        sample_inline_seed(1_i64 << 40, &mut observed);

        let big_seed = super::super::super::bigint_ops::bigint_from_literal(
            "100000000000000000000",
        );
        let big_handle = unsafe { dispatch_Random([big_seed].as_ptr(), 1) };
        let big_id = big_handle.as_int().unwrap() as u64;
        observed.push((next_u32(big_id), next_u32(big_id)));
        drop_random_handle(big_id);
        unsafe { super::super::super::rc::release_if_ptr(big_seed) };

        let mut long_key_words = [0_u32; 625];
        long_key_words[0] = 1;
        long_key_words[624] = 1;
        let long_seed = super::super::super::bigint_ops::bigint_from_big(
            num_bigint::BigInt::from_slice(num_bigint::Sign::Plus, &long_key_words),
        );
        let long_handle = unsafe { dispatch_Random([long_seed].as_ptr(), 1) };
        let long_id = long_handle.as_int().unwrap() as u64;
        let long_observed = (next_u32(long_id), next_u32(long_id));
        drop_random_handle(long_id);
        unsafe { super::super::super::rc::release_if_ptr(long_seed) };

        let live_handle = unsafe { dispatch_Random([MbValue::none()].as_ptr(), 1) };
        let live_id = live_handle.as_int().unwrap() as u64;
        mb_random_method_seed(live_handle, MbValue::from_int(1234));
        let live_observed = (next_u32(live_id), next_u32(live_id));
        drop_random_handle(live_id);

        assert_eq!(
            observed,
            vec![
                (3626764237, 1654615998),
                (2746317213, 478163327),
                (4150886329, 3342196574),
                (577090037, 2444712010),
                (445128065, 1517081360),
                (78147056, 1939873431),
            ],
        );
        assert_eq!(long_observed, (893496774, 3888348697));
        assert_eq!(live_observed, (4150886329, 3342196574));
    }

    #[test]
    fn test_seed_rejects_user_instance_without_replacing_state() {
        let accepted_seed = |seed: MbValue, release_seed: bool| {
            super::super::super::exception::mb_clear_exception();
            let handle = unsafe { dispatch_Random([MbValue::none()].as_ptr(), 1) };
            let id = handle.as_int().unwrap() as u64;
            let result = mb_random_method_seed(handle, seed);
            let returned_none = result.is_none();
            let exception_type = super::super::super::exception::current_exception_type();
            super::super::super::exception::mb_clear_exception();
            if release_seed {
                unsafe { super::super::super::rc::release_if_ptr(seed) };
            }
            drop_random_handle(id);
            assert!(returned_none, "accepted seed returned a value");
            assert!(
                exception_type.is_none(),
                "accepted seed raised {exception_type:?}"
            );
        };

        accepted_seed(MbValue::none(), false);
        accepted_seed(MbValue::from_int(1234), false);
        accepted_seed(
            super::super::super::bigint_ops::bigint_from_literal("100000000000000000000"),
            true,
        );
        accepted_seed(MbValue::from_bool(true), false);
        accepted_seed(MbValue::from_float(1.25), false);
        accepted_seed(
            MbValue::from_ptr(MbObject::new_str("accepted".to_string())),
            true,
        );
        accepted_seed(
            MbValue::from_ptr(MbObject::new_bytes(b"accepted".to_vec())),
            true,
        );
        accepted_seed(
            MbValue::from_ptr(MbObject::new_bytearray(b"accepted".to_vec())),
            true,
        );

        super::super::super::exception::mb_clear_exception();
        let saved_default = DEFAULT_HANDLE.with(|c| c.get());
        let saved_ids = RANDOM_IDS.with(|s| s.borrow().clone());
        let saved_randoms_len = RANDOMS.with(|m| m.borrow().len());
        let saved_random_ids_len = saved_ids.len();
        DEFAULT_HANDLE.with(|c| c.set(None));
        let module_instance =
            MbValue::from_ptr(MbObject::new_instance("UnsupportedModuleSeed".to_string()));
        let module_result = mb_random_method_seed(MbValue::none(), module_instance);
        let module_exception = super::super::super::exception::current_exception_type();
        let module_default = DEFAULT_HANDLE.with(|c| c.get());
        let module_randoms_len = RANDOMS.with(|m| m.borrow().len());
        let module_random_ids = RANDOM_IDS.with(|s| s.borrow().clone());
        let module_random_ids_len = module_random_ids.len();
        let new_ids: Vec<u64> = module_random_ids.difference(&saved_ids).copied().collect();
        for id in new_ids {
            drop_random_handle(id);
        }
        DEFAULT_HANDLE.with(|c| c.set(saved_default));
        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(module_instance) };

        assert!(module_result.is_none(), "unsupported module seed returned a value");
        assert_eq!(module_exception.as_deref(), Some("TypeError"));
        assert_eq!(module_default, None, "invalid module seed allocated default handle");
        assert_eq!(module_randoms_len, saved_randoms_len);
        assert_eq!(module_random_ids_len, saved_random_ids_len);
        assert_eq!(module_random_ids, saved_ids);
        assert_eq!(DEFAULT_HANDLE.with(|c| c.get()), saved_default);

        super::super::super::exception::mb_clear_exception();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let instance = MbValue::from_ptr(MbObject::new_instance("UnsupportedSeed".to_string()));
        let result = mb_random_method_seed(handle, instance);
        let returned_none = result.is_none();
        let exception_type = super::super::super::exception::current_exception_type();
        let following = next_u32(id);

        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(instance) };
        drop_random_handle(id);

        assert!(returned_none, "unsupported user instance returned a value");
        assert_eq!(
            (exception_type.as_deref(), following),
            (Some("TypeError"), 4_150_886_329),
        );
    }

    #[test]
    fn test_random_uses_cpython_53_bit_word_composition() {
        let fixed_id = alloc_random_id();
        RANDOMS.with(|m| {
            m.borrow_mut().insert(
                fixed_id,
                Mt::new_with_key([0x123, 0x234, 0x345, 0x456]),
            );
        });
        RANDOM_IDS.with(|s| {
            s.borrow_mut().insert(fixed_id);
        });
        let fixed_observed = [next_f64(fixed_id), next_f64(fixed_id)];
        drop_random_handle(fixed_id);

        let live_handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let live_id = live_handle.as_int().unwrap() as u64;
        let live_observed = next_f64(live_id);
        drop_random_handle(live_id);

        assert_eq!(fixed_observed, [0.24856890158782508, 0.11112762955044497]);
        assert_eq!(live_observed, 0.9664535356921388);
    }

    #[test]
    fn test_seed_text_uses_sha512_v2_key_material() {
        let frozen_words = [
            2773263519_u32,
            714787151,
            1681713166,
            1162691619,
            2751392701,
            918174755,
            659538344,
            563255594,
            1263915930,
            178188006,
            2309586594,
            317127246,
            2921349425,
            3426841417,
            2472639162,
            3719247265,
            6382179,
        ];
        let expected = vec![
            3315820543_u32,
            4246262336,
            2397318194,
            1976556168,
            3047419019,
            3939654856,
            1521820844,
            4110710813,
        ];

        let mut reference_mt = mt_from_cpython_key(&frozen_words);
        let reference = (0..8)
            .map(|_| reference_mt.next_u32())
            .collect::<Vec<_>>();

        let str_seed = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        let str_handle = unsafe { dispatch_Random([str_seed].as_ptr(), 1) };
        let str_id = str_handle.as_int().unwrap() as u64;
        let str_observed = (0..8).map(|_| next_u32(str_id)).collect::<Vec<_>>();
        drop_random_handle(str_id);

        let bytes_seed = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let bytes_handle = unsafe { dispatch_Random([bytes_seed].as_ptr(), 1) };
        let bytes_id = bytes_handle.as_int().unwrap() as u64;
        let bytes_observed = (0..8).map(|_| next_u32(bytes_id)).collect::<Vec<_>>();
        drop_random_handle(bytes_id);

        let bytearray_seed = MbValue::from_ptr(MbObject::new_bytearray(b"abc".to_vec()));
        let bytearray_handle = unsafe { dispatch_Random([bytearray_seed].as_ptr(), 1) };
        let bytearray_id = bytearray_handle.as_int().unwrap() as u64;
        let bytearray_observed = (0..8)
            .map(|_| next_u32(bytearray_id))
            .collect::<Vec<_>>();
        let bytearray_after = unsafe {
            bytearray_seed.as_ptr().and_then(|ptr| match &(*ptr).data {
                ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
                _ => None,
            })
        };
        drop_random_handle(bytearray_id);

        let reseed_handle = unsafe { dispatch_Random([MbValue::none()].as_ptr(), 1) };
        let reseed_id = reseed_handle.as_int().unwrap() as u64;
        let reseed_seed = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        mb_random_method_seed(reseed_handle, reseed_seed);
        let reseed_observed = (0..8).map(|_| next_u32(reseed_id)).collect::<Vec<_>>();
        drop_random_handle(reseed_id);

        unsafe {
            super::super::super::rc::release_if_ptr(str_seed);
            super::super::super::rc::release_if_ptr(bytes_seed);
            super::super::super::rc::release_if_ptr(bytearray_seed);
            super::super::super::rc::release_if_ptr(reseed_seed);
        }

        assert_eq!(reference, expected);
        assert_eq!(str_observed, expected);
        assert_eq!(bytes_observed, expected);
        assert_eq!(bytearray_observed, expected);
        assert_eq!(reseed_observed, expected);
        assert_eq!(bytearray_after, Some(b"abc".to_vec()));
    }

    #[test]
    fn test_getrandbits_uses_cpython_word_shift_and_endian_assembly() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let mut snapshots = Vec::new();
        let mut is_64_bit_heap_bigint = false;

        for width in [1_i64, 8, 16, 32, 64] {
            let result = mb_random_method_getrandbits(handle, MbValue::from_int(width));
            if width == 64 {
                is_64_bit_heap_bigint = result.as_ptr().is_some_and(|ptr| unsafe {
                    matches!((*ptr).data, ObjData::BigInt(_))
                });
            }
            snapshots.push(unsafe { super::super::super::bigint_ops::to_bigint(result) });
            unsafe { super::super::super::rc::release_if_ptr(result) };
        }
        drop_random_handle(id);

        let expected = vec![
            Some(num_bigint::BigInt::from(1_u32)),
            Some(num_bigint::BigInt::from(199_u32)),
            Some(num_bigint::BigInt::from(28883_u32)),
            Some(num_bigint::BigInt::from(501869158_u32)),
            Some(num_bigint::BigInt::from(1672079305790387732_u64)),
        ];
        assert_eq!(snapshots, expected);
        assert!(is_64_bit_heap_bigint);
    }

    #[test]
    fn test_gauss_uses_cpython_box_muller_and_per_handle_cache() {
        let left = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let right = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let left_id = left.as_int().unwrap() as u64;
        let right_id = right.as_int().unwrap() as u64;

        mb_random_method_seed(left, MbValue::from_int(1234));
        mb_random_method_seed(right, MbValue::from_int(1234));

        let left_first = mb_random_method_gauss(
            left,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);
        let right_first = mb_random_method_gauss(
            right,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);
        let left_second = mb_random_method_gauss(
            left,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);
        let right_second = mb_random_method_gauss(
            right,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);
        let left_third = mb_random_method_gauss(
            left,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);
        let left_fourth = mb_random_method_gauss(
            left,
            MbValue::from_float(0.0),
            MbValue::from_float(1.0),
        )
        .as_float()
        .unwrap_or(f64::NAN);

        drop_random_handle(left_id);
        drop_random_handle(right_id);

        assert_eq!(
            (left_first, right_first, left_second, right_second),
            (
                1.0542196419272387,
                1.0542196419272387,
                -0.22555725575068641,
                -0.22555725575068641,
            ),
        );
        assert_eq!((left_third, left_fourth), (2.1970405483761803, 0.1034917897273693));
    }

    #[test]
    fn test_choice_consumes_cpython_randbelow_stream() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let list = MbValue::from_ptr(MbObject::new_list(
            (0..6).map(MbValue::from_int).collect(),
        ));

        let choices = (0..6)
            .map(|_| mb_random_method_choice(handle, list).as_int())
            .collect::<Vec<_>>();
        let sentinel = next_u32(id);

        unsafe { super::super::super::rc::release_if_ptr(list) };
        drop_random_handle(id);

        assert_eq!(
            (choices, sentinel),
            (vec![Some(3), Some(0), Some(0), Some(0), Some(4), Some(0)], 2_884_343_186),
        );
    }

    #[test]
    fn test_choice_heap_string_survives_population_release() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let string_ptr = MbObject::new_str("choice-owned-string".to_string());
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(string_ptr)]));
        let before = unsafe { super::super::super::rc::mb_refcount(string_ptr) };

        let returned = mb_random_method_choice(handle, list);
        let returned_ptr = returned.as_ptr();
        // The population still owns the string, so it is safe to inspect the
        // source object's refcount before releasing the list.  The production
        // fix must turn this exact 1 -> 2 transition into an owned return.
        let after = unsafe { super::super::super::rc::mb_refcount(string_ptr) };

        // Known-red pre-fix path: after == 1 means the returned pointer is
        // borrowed.  Release the list, but never dereference or release the
        // now-dangling returned value.  Only the after == 2 path owns it.
        let observed = if after == 2 && returned_ptr == Some(string_ptr) {
            unsafe { super::super::super::rc::release_if_ptr(list) };
            let value = unsafe {
                match &(*string_ptr).data {
                    ObjData::Str(value) => Some(value.clone()),
                    _ => None,
                }
            };
            unsafe { super::super::super::rc::release_if_ptr(returned) };
            value
        } else {
            unsafe { super::super::super::rc::release_if_ptr(list) };
            None
        };

        drop_random_handle(id);

        assert_eq!(returned_ptr, Some(string_ptr));
        assert_eq!((before, after), (1, 2));
        assert_eq!(observed.as_deref(), Some("choice-owned-string"));
    }

    #[test]
    fn test_sample_heap_string_survives_population_release() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let string_ptr = MbObject::new_str("sample-owned-string".to_string());
        let population = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(string_ptr)]));
        let before = unsafe { super::super::super::rc::mb_refcount(string_ptr) };

        let result = mb_random_method_sample(handle, population, MbValue::from_int(1));
        // Capture the selected pointer while both the population and result
        // are live.  The pointer identity must be preserved, not copied.
        let selected_ptr = result.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock
                    .read()
                    .unwrap()
                    .first()
                    .and_then(MbValue::as_ptr),
                _ => None,
            }
        });
        let after = unsafe { super::super::super::rc::mb_refcount(string_ptr) };

        // Known-red pre-fix path: sample() returns a borrowed element and the
        // result list does not own it.  Add exactly one temporary retain only
        // after taking the refcount snapshot, so cleanup remains balanced
        // while still making the post-population string snapshot safe.
        if after == 1 {
            unsafe {
                super::super::super::rc::retain_if_ptr(MbValue::from_ptr(string_ptr));
            }
        }

        unsafe { super::super::super::rc::release_if_ptr(population) };
        let observed = result.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().first().and_then(|value| {
                    value.as_ptr().and_then(|item_ptr| match &(*item_ptr).data {
                        ObjData::Str(value) => Some(value.clone()),
                        _ => None,
                    })
                }),
                _ => None,
            }
        });
        unsafe { super::super::super::rc::release_if_ptr(result) };
        drop_random_handle(id);

        assert_eq!(selected_ptr, Some(string_ptr));
        assert_eq!((before, after), (1, 2));
        assert_eq!(observed.as_deref(), Some("sample-owned-string"));
    }

    #[test]
    fn test_shuffle_uses_cpython_reverse_fisher_yates_consumption() {
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let list = MbValue::from_ptr(MbObject::new_list(
            (0..8).map(MbValue::from_int).collect(),
        ));

        let returned = mb_random_method_shuffle(handle, list);
        let mutated = list.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => Some(
                    lock.read()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_int())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            }
        });
        let sentinel = next_u32(id);

        unsafe { super::super::super::rc::release_if_ptr(list) };
        drop_random_handle(id);

        assert_eq!(
            (returned.is_none(), mutated, sentinel),
            (
                true,
                Some(vec![Some(1), Some(3), Some(2), Some(4), Some(5), Some(6), Some(0), Some(7)]),
                422_719_469,
            ),
        );
    }

    #[test]
    fn test_sample_uses_cpython_pool_or_selected_set_algorithm() {
        let pool_handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let pool_id = pool_handle.as_int().unwrap() as u64;
        let pool_population = MbValue::from_ptr(MbObject::new_list(
            (0..10).map(MbValue::from_int).collect(),
        ));
        let pool_first = mb_random_method_sample(pool_handle, pool_population, MbValue::from_int(4));
        let pool_first_snapshot = pool_first.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => Some(
                    lock.read()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_int())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            }
        });
        unsafe { super::super::super::rc::release_if_ptr(pool_first) };

        let pool_second = mb_random_method_sample(pool_handle, pool_population, MbValue::from_int(4));
        let pool_second_snapshot = pool_second.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => Some(
                    lock.read()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_int())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            }
        });
        unsafe { super::super::super::rc::release_if_ptr(pool_second) };
        let pool_sentinel = next_u32(pool_id);

        let selected_handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let selected_id = selected_handle.as_int().unwrap() as u64;
        let selected_population = MbValue::from_ptr(MbObject::new_list(
            (0..22).map(MbValue::from_int).collect(),
        ));
        let selected = mb_random_method_sample(
            selected_handle,
            selected_population,
            MbValue::from_int(5),
        );
        let selected_snapshot = selected.as_ptr().and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => Some(
                    lock.read()
                        .unwrap()
                        .iter()
                        .map(|value| value.as_int())
                        .collect::<Vec<_>>(),
                ),
                _ => None,
            }
        });
        let selected_sentinel = next_u32(selected_id);
        unsafe {
            super::super::super::rc::release_if_ptr(selected);
            super::super::super::rc::release_if_ptr(pool_population);
            super::super::super::rc::release_if_ptr(selected_population);
        }
        drop_random_handle(pool_id);
        drop_random_handle(selected_id);

        assert_eq!(
            (
                pool_first_snapshot,
                pool_second_snapshot,
                pool_sentinel,
                selected_snapshot,
                selected_sentinel,
            ),
            (
                Some(vec![Some(7), Some(1), Some(0), Some(9)]),
                Some(vec![Some(9), Some(0), Some(1), Some(8)]),
                3_292_010_550,
                Some(vec![Some(14), Some(3), Some(0), Some(2), Some(18)]),
                150_006_740,
            ),
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_sample_raw_string_materialization_is_leak_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let guard = super::super::super::rc::start_leak_balance_scope(
            "sample_raw_string_materialization",
        )
        .expect("start leak scope");
        let source = MbValue::from_ptr(MbObject::new_str("aβcδ".to_string()));
        let result = mb_random_method_sample(handle, source, MbValue::from_int(4));

        // Observe every returned character while both the source and result
        // are live.  No pointer captured here is dereferenced after result
        // cleanup; a red candidate may leave the materialized characters
        // allocated until this short-lived test process exits.
        let (observed_chars, observed_ptrs, observed_refcounts) = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => {
                        let values = lock.read().unwrap();
                        let chars = values
                            .iter()
                            .filter_map(|value| value.as_ptr())
                            .filter_map(|ptr| match &(*ptr).data {
                                ObjData::Str(value) => Some(value.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>();
                        let mut ptrs = Vec::new();
                        let mut refcounts = Vec::new();
                        for value in values.iter() {
                            if let Some(item_ptr) = value.as_ptr() {
                                if !ptrs.contains(&item_ptr) {
                                    ptrs.push(item_ptr);
                                    refcounts.push(super::super::super::rc::mb_refcount(item_ptr));
                                }
                            }
                        }
                        Some((chars, ptrs, refcounts))
                    }
                    _ => None,
                }
            })
            .unwrap_or_default();

        unsafe {
            super::super::super::rc::release_if_ptr(result);
            // The unfixed candidate leaves one extra source owner on every
            // materialized character (raw rc=2 at the snapshot).  Release
            // only those recorded extra-owner paths after the result list is
            // gone; the fixed rc=1 path has no still-live original owner and
            // must not be released again.
            for (ptr, rc) in observed_ptrs.iter().zip(observed_refcounts.iter()) {
                if *rc > 1 {
                    super::super::super::rc::release_if_ptr(MbValue::from_ptr(*ptr));
                }
            }
            super::super::super::rc::release_if_ptr(source);
        }
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced raw-string sample scope");
        } else {
            // The expected-red candidate intentionally retains the tiny
            // materialized-character leak.  Reset before the assertion and
            // before dropping the guard so no stale scope poisons later tests.
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        let mut sorted = observed_chars;
        sorted.sort();
        assert_eq!(observed_ptrs.len(), 4);
        assert_eq!(sorted, vec!["a", "c", "β", "δ"]);
        assert_eq!(delta, 0, "raw-string sample leak delta: {delta}");
        assert_eq!(observed_refcounts, vec![1, 1, 1, 1]);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_sample_counts_repeated_heap_element_is_leak_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let guard = super::super::super::rc::start_leak_balance_scope(
            "sample_counts_repeated_heap_element",
        )
        .expect("start leak scope");
        // A raw one-character string forces sample_population_items to create
        // one heap character.  The counts branch then repeats that same
        // pointer three times in the result.
        let source = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let counts = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(3)]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        let key = MbValue::from_ptr(MbObject::new_str("counts".to_string()));
        super::super::super::dict_ops::mb_dict_setitem(kwargs, key, counts);
        let k_key = MbValue::from_ptr(MbObject::new_str("k".to_string()));
        super::super::super::dict_ops::mb_dict_setitem(kwargs, k_key, MbValue::from_int(3));
        unsafe {
            // String keys are copied into DictKey::Str, so the temporary key
            // object remains the caller's responsibility.
            super::super::super::rc::release_if_ptr(key);
            super::super::super::rc::release_if_ptr(k_key);
        }

        let result = mb_random_method_sample(handle, source, kwargs);
        let (result_ptrs, all_strings) = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => {
                        let values = lock.read().unwrap();
                        let ptrs = values.iter().filter_map(MbValue::as_ptr).collect::<Vec<_>>();
                        let strings = ptrs.iter().all(|ptr| matches!((*(*ptr)).data, ObjData::Str(_)));
                        Some((ptrs, strings))
                    }
                    _ => None,
                }
            })
            .unwrap_or_default();
        let repeated_ptr = result_ptrs.first().copied();
        let repeated = !result_ptrs.is_empty() && result_ptrs.iter().all(|ptr| Some(*ptr) == repeated_ptr);
        let observed_rc = repeated_ptr
            .map(|ptr| unsafe { super::super::super::rc::mb_refcount(ptr) })
            .unwrap_or(0);

        // The result list releases one reference per slot.  On the current
        // red candidate the source character has only rc=1, so add exactly
        // the missing cleanup-only retains after recording the raw snapshot.
        if let Some(ptr) = repeated_ptr {
            for _ in observed_rc..3 {
                unsafe { super::super::super::rc::retain_if_ptr(MbValue::from_ptr(ptr)) };
            }
        }
        unsafe {
            super::super::super::rc::release_if_ptr(result);
            super::super::super::rc::release_if_ptr(source);
            super::super::super::rc::release_if_ptr(counts);
            super::super::super::rc::release_if_ptr(kwargs);
        }
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced repeated-count scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert_eq!(result_ptrs.len(), 3);
        assert!(all_strings, "counts result must contain strings");
        assert!(repeated, "counts result must repeat one source pointer");
        assert_eq!(delta, 0, "repeated-count sample leak delta: {delta}");
        assert_eq!(observed_rc, 3, "raw repeated-element rc: {observed_rc}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_sample_owned_population_early_errors_are_leak_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;

        super::super::super::exception::mb_clear_exception();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "sample_owned_population_k_error",
        )
        .expect("start leak scope");
        let source = MbValue::from_ptr(MbObject::new_str("aβc".to_string()));
        let result = mb_random_method_sample(handle, source, MbValue::from_int(4));
        let k_returned_none = result.is_none();
        let k_exception = super::super::super::exception::current_exception_type();
        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(source) };
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let k_delta = snap.delta();
        if k_delta == 0 {
            guard.finish().expect("balanced k-error scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        super::super::super::exception::mb_clear_exception();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "sample_owned_population_counts_error",
        )
        .expect("start leak scope");
        let source = MbValue::from_ptr(MbObject::new_str("aβc".to_string()));
        let counts = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        let key = MbValue::from_ptr(MbObject::new_str("counts".to_string()));
        super::super::super::dict_ops::mb_dict_setitem(kwargs, key, counts);
        unsafe { super::super::super::rc::release_if_ptr(key) };
        let result = mb_random_method_sample(handle, source, kwargs);
        let counts_returned_none = result.is_none();
        let counts_exception = super::super::super::exception::current_exception_type();
        super::super::super::exception::mb_clear_exception();
        unsafe {
            super::super::super::rc::release_if_ptr(source);
            super::super::super::rc::release_if_ptr(counts);
            super::super::super::rc::release_if_ptr(kwargs);
        }
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let counts_delta = snap.delta();
        if counts_delta == 0 {
            guard.finish().expect("balanced counts-error scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert!(k_returned_none);
        assert_eq!(k_exception.as_deref(), Some("ValueError"));
        assert_eq!(k_delta, 0, "raw-string k error leak delta: {k_delta}");
        assert!(counts_returned_none);
        assert_eq!(counts_exception.as_deref(), Some("ValueError"));
        assert_eq!(counts_delta, 0, "raw-string counts error leak delta: {counts_delta}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_sample_drained_heap_range_items_are_leak_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let guard = super::super::super::rc::start_leak_balance_scope(
            "sample_drained_heap_range_items",
        )
        .expect("start leak scope");
        let base = 1_i64 << 60;
        let start = MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(base)));
        let stop = MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(base + 22)));
        let range = super::super::super::builtins::mb_range_2(start, stop);
        unsafe {
            // mb_range_iter clones the endpoints into its iterator state.
            super::super::super::rc::release_if_ptr(start);
            super::super::super::rc::release_if_ptr(stop);
        }
        let result = mb_random_method_sample(handle, range, MbValue::from_int(5));
        let (result_len, all_heap_ints) = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => {
                        let values = lock.read().unwrap();
                        let all_heap = values.iter().all(|value| {
                            value.as_ptr().is_some_and(|item| matches!((*item).data, ObjData::BigInt(_)))
                        });
                        Some((values.len(), all_heap))
                    }
                    _ => None,
                }
            })
            .unwrap_or((0, false));
        unsafe { super::super::super::rc::release_if_ptr(result) };
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced range sample scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert_eq!(result_len, 5);
        assert!(all_heap_ints, "range sample must return heap-backed integers");
        assert_eq!(delta, 0, "drained range sample leak delta: {delta}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_choices_raw_string_repeated_result_ownership_is_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "choices_raw_string_repeated_result_ownership",
        )
        .expect("start choices raw-string leak scope");
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let source = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let result = mb_random_method_choices_full(
            handle,
            source,
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(3),
        );
        let result_ptrs = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => Some(
                        lock.read()
                            .unwrap()
                            .iter()
                            .filter_map(MbValue::as_ptr)
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                }
            })
            .unwrap_or_default();
        let selected_ptr = result_ptrs.first().copied();
        let selected_text = selected_ptr.and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Str(value) => Some(value.clone()),
                _ => None,
            }
        });
        let observed_rc = selected_ptr
            .map(|ptr| unsafe { super::super::super::rc::mb_refcount(ptr) })
            .unwrap_or(0);

        unsafe {
            if let Some(ptr) = selected_ptr {
                // Keep the allocation alive while releasing the result's
                // three owners and any still-present source owner.
                super::super::super::rc::retain_if_ptr(MbValue::from_ptr(ptr));
            }
            super::super::super::rc::release_if_ptr(result);
            if let Some(ptr) = selected_ptr {
                // The observer is one owner.  Remove only extra owners so
                // both the unfixed and fixed paths remain UAF-safe.
                while super::super::super::rc::mb_refcount(ptr) > 1 {
                    super::super::super::rc::release_if_ptr(MbValue::from_ptr(ptr));
                }
                super::super::super::rc::release_if_ptr(MbValue::from_ptr(ptr));
            }
            super::super::super::rc::release_if_ptr(source);
        }
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced raw-string choices scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert_eq!(result_ptrs.len(), 3);
        assert!(result_ptrs.iter().all(|ptr| Some(*ptr) == selected_ptr));
        assert_eq!(selected_text.as_deref(), Some("x"));
        assert_eq!(observed_rc, 3, "raw choices result ownership rc: {observed_rc}");
        assert_eq!(delta, 0, "raw choices repeated result leak delta: {delta}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_choices_raw_string_zero_k_releases_unselected_materialization() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "choices_raw_string_zero_k_materialization",
        )
        .expect("start choices zero-k leak scope");
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let source = MbValue::from_ptr(MbObject::new_str("aβc".to_string()));
        let result = mb_random_method_choices_full(
            handle,
            source,
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(0),
        );
        unsafe {
            super::super::super::rc::release_if_ptr(result);
            super::super::super::rc::release_if_ptr(source);
        }
        // The current helper leaves the unselected materialized characters
        // inaccessible.  Snapshot the explicit red delta, reset the verifier
        // safely, then assert the post-fix contract.
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced zero-k choices scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert_eq!(delta, 0, "raw choices zero-k leak delta: {delta}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_choices_drained_heap_range_materialization_is_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "choices_drained_heap_range_materialization",
        )
        .expect("start choices range leak scope");
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let base = 1_i64 << 60;
        let start = MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(base)));
        let stop = MbValue::from_ptr(MbObject::new_bigint(num_bigint::BigInt::from(base + 4)));
        let range = super::super::super::builtins::mb_range_2(start, stop);
        unsafe {
            super::super::super::rc::release_if_ptr(start);
            super::super::super::rc::release_if_ptr(stop);
        }
        let result = mb_random_method_choices_full(
            handle,
            range,
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(6),
        );
        let result_ptrs = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => Some(
                        lock.read()
                            .unwrap()
                            .iter()
                            .filter_map(MbValue::as_ptr)
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                }
            })
            .unwrap_or_default();
        let mut occurrences = Vec::<(*mut MbObject, usize)>::new();
        let mut all_heap_bigints = true;
        for &ptr in &result_ptrs {
            if let Some((_, count)) = occurrences.iter_mut().find(|(seen, _)| *seen == ptr) {
                *count += 1;
            } else {
                occurrences.push((ptr, 1));
            }
            unsafe {
                all_heap_bigints &= matches!((*ptr).data, ObjData::BigInt(_));
            }
        }
        let refcounts = occurrences
            .iter()
            .map(|(ptr, _)| unsafe { super::super::super::rc::mb_refcount(*ptr) })
            .collect::<Vec<_>>();
        let owner_counts_match = occurrences
            .iter()
            .zip(refcounts.iter())
            .all(|((_, count), rc)| *rc == *count as u32);
        unsafe { super::super::super::rc::release_if_ptr(result) };
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced range choices scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert_eq!(result_ptrs.len(), 6);
        assert!(all_heap_bigints, "range choices must return heap BigInts");
        assert!(occurrences.len() < result_ptrs.len(), "range choices must repeat a value");
        assert!(
            owner_counts_match && delta == 0,
            "range choices owners: expected rc=occurrences and delta=0, refcounts={refcounts:?}, occurrences={occurrences:?}, delta={delta}",
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_choices_owned_population_early_error_is_balanced() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        super::super::super::exception::mb_clear_exception();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "choices_owned_population_early_error",
        )
        .expect("start choices early-error leak scope");
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let source = MbValue::from_ptr(MbObject::new_str("aβc".to_string()));
        // Drain a raw-string iterator through the owned materialization path
        // before the deliberate weights-length error.
        let source_iter = super::super::super::iter::mb_iter(source);
        // The weights iterator is also materialized before validation, so the
        // early return covers both temporary owner roots (population + weights).
        let weights = super::super::super::builtins::mb_range_2(
            MbValue::from_int(0),
            MbValue::from_int(2),
        );
        let result = mb_random_method_choices_full(
            handle,
            source_iter,
            weights,
            MbValue::none(),
            MbValue::from_int(1),
        );
        let exception_type = super::super::super::exception::current_exception_type();
        let exception_message = super::super::super::exception::current_exception_message();
        super::super::super::exception::mb_clear_exception();
        unsafe { super::super::super::rc::release_if_ptr(source) };
        let snap = super::super::super::rc::get_leak_balance_snapshot().expect("active scope");
        let delta = snap.delta();
        if delta == 0 {
            guard.finish().expect("balanced early-error choices scope");
        } else {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
        }
        drop_random_handle(id);

        assert!(result.is_none());
        assert_eq!(exception_type.as_deref(), Some("ValueError"));
        assert_eq!(
            exception_message.as_deref(),
            Some("The number of weights does not match the population"),
        );
        assert_eq!(delta, 0, "early-error choices leak delta: {delta}");
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_choices_borrowed_heap_element_retains_each_occurrence() {
        super::super::super::rc::reset_leak_balance_scope_for_testing();
        let guard = super::super::super::rc::start_leak_balance_scope(
            "choices_borrowed_heap_element_occurrences",
        )
        .expect("start borrowed choices leak scope");
        let handle = unsafe { dispatch_Random([MbValue::from_int(1234)].as_ptr(), 1) };
        let id = handle.as_int().unwrap() as u64;
        let string_ptr = MbObject::new_str("borrowed-choice".to_string());
        let source = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(string_ptr)]));
        let result = mb_random_method_choices_full(
            handle,
            source,
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(3),
        );
        let result_ptrs = result
            .as_ptr()
            .and_then(|ptr| unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => Some(
                        lock.read()
                            .unwrap()
                            .iter()
                            .filter_map(MbValue::as_ptr)
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                }
            })
            .unwrap_or_default();
        let before_source_release = unsafe { super::super::super::rc::mb_refcount(string_ptr) };
        unsafe { super::super::super::rc::release_if_ptr(source) };
        let after_source_release = unsafe { super::super::super::rc::mb_refcount(string_ptr) };
        let content = unsafe {
            match &(*string_ptr).data {
                ObjData::Str(value) => Some(value.clone()),
                _ => None,
            }
        };
        unsafe {
            super::super::super::rc::retain_if_ptr(MbValue::from_ptr(string_ptr));
            super::super::super::rc::release_if_ptr(result);
            super::super::super::rc::release_if_ptr(MbValue::from_ptr(string_ptr));
        }
        let snap = guard.finish().expect("balanced borrowed choices scope");
        drop_random_handle(id);

        assert_eq!(result_ptrs, vec![string_ptr; 3]);
        assert_eq!(before_source_release, 4);
        assert_eq!(after_source_release, 3);
        assert_eq!(content.as_deref(), Some("borrowed-choice"));
        assert_eq!(snap.delta(), 0, "borrowed choices leak delta: {}", snap.delta());
    }

    #[test]
    #[cfg(debug_assertions)]
    fn test_random_exception_helpers_release_temporary_roots() {
        let (value_type, value_message, value_delta) = {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
            let guard = super::super::super::rc::start_leak_balance_scope(
                "random_raise_value_error",
            )
            .expect("start ValueError leak scope");
            let returned = raise_value_error("random ValueError helper message");
            let exception_type = super::super::super::exception::current_exception_type();
            let exception_message = super::super::super::exception::current_exception_message();
            super::super::super::exception::mb_clear_exception();
            let snapshot = super::super::super::rc::get_leak_balance_snapshot()
                .expect("active ValueError leak scope");
            let delta = snapshot.delta();
            if delta == 0 {
                guard.finish().expect("balanced ValueError leak scope");
            } else {
                super::super::super::rc::reset_leak_balance_scope_for_testing();
                drop(guard);
            }
            assert!(returned.is_none());
            (exception_type, exception_message, delta)
        };

        let (type_type, type_message, type_delta) = {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
            let guard = super::super::super::rc::start_leak_balance_scope(
                "random_raise_type_error",
            )
            .expect("start TypeError leak scope");
            let returned = raise_type_error("random TypeError helper message");
            let exception_type = super::super::super::exception::current_exception_type();
            let exception_message = super::super::super::exception::current_exception_message();
            super::super::super::exception::mb_clear_exception();
            let snapshot = super::super::super::rc::get_leak_balance_snapshot()
                .expect("active TypeError leak scope");
            let delta = snapshot.delta();
            if delta == 0 {
                guard.finish().expect("balanced TypeError leak scope");
            } else {
                super::super::super::rc::reset_leak_balance_scope_for_testing();
                drop(guard);
            }
            assert!(returned.is_none());
            (exception_type, exception_message, delta)
        };

        let (index_type, index_message, index_delta) = {
            super::super::super::rc::reset_leak_balance_scope_for_testing();
            let guard = super::super::super::rc::start_leak_balance_scope(
                "random_raise_index_error",
            )
            .expect("start IndexError leak scope");
            let returned = raise_index_error("random IndexError helper message");
            let exception_type = super::super::super::exception::current_exception_type();
            let exception_message = super::super::super::exception::current_exception_message();
            super::super::super::exception::mb_clear_exception();
            let snapshot = super::super::super::rc::get_leak_balance_snapshot()
                .expect("active IndexError leak scope");
            let delta = snapshot.delta();
            if delta == 0 {
                guard.finish().expect("balanced IndexError leak scope");
            } else {
                super::super::super::rc::reset_leak_balance_scope_for_testing();
                drop(guard);
            }
            assert!(returned.is_none());
            (exception_type, exception_message, delta)
        };

        assert_eq!(value_type.as_deref(), Some("ValueError"));
        assert_eq!(value_message.as_deref(), Some("random ValueError helper message"));
        assert_eq!(type_type.as_deref(), Some("TypeError"));
        assert_eq!(type_message.as_deref(), Some("random TypeError helper message"));
        assert_eq!(index_type.as_deref(), Some("IndexError"));
        assert_eq!(index_message.as_deref(), Some("random IndexError helper message"));
        assert_eq!(
            (value_delta, type_delta, index_delta),
            (0, 0, 0),
            "random exception helper leak deltas",
        );
    }
}
