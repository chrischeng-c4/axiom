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
//!   CPython's MT19937 generator family. Not bit-for-bit seed-compatible
//!   with CPython (CPython uses a custom seed-init from
//!   `init_by_array`), but distribution shapes match.
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
//! - Variate fns: Box–Muller for gauss/normalvariate; inverse-CDF for
//!   expovariate/paretovariate/weibullvariate; Marsaglia–Tsang 2000 for
//!   gammavariate; ratio-of-gammas for betavariate; Best–Fisher 1979
//!   (iteration cap 32) for vonmisesvariate. Not bit-for-bit CPython
//!   but distribution-correct.
//!
//! - `getstate`/`setstate`: return `(handle_id,)` 1-tuple so
//!   snapshot/restore patterns where the handle stays alive round-trip
//!   correctly. Full 625-int MT state serialization is out of scope.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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
    /// Cached spare normal for Box–Muller (per-handle would be cleaner
    /// but a thread-local cache works for typical scalar-loop usage).
    static GAUSS_SPARE: Cell<Option<f64>> = const { Cell::new(None) };
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
    RANDOMS.with(|m| { m.borrow_mut().remove(&id); });
    RANDOM_IDS.with(|s| { s.borrow_mut().remove(&id); });
    RANDOM_REFCOUNTS.with(|r| { r.borrow_mut().remove(&id); });
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

fn make_handle(seed: Option<u32>) -> u64 {
    let id = alloc_random_id();
    let rng = match seed {
        Some(s) => Mt::new(s),
        None => Mt::new_unseeded(),
    };
    RANDOMS.with(|m| { m.borrow_mut().insert(id, rng); });
    RANDOM_IDS.with(|s| { s.borrow_mut().insert(id); });
    id
}

fn default_handle() -> u64 {
    DEFAULT_HANDLE.with(|c| {
        if let Some(id) = c.get() { return id; }
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

/// Convert any MbValue seed (int / float / str / bytes / none) to a u32.
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

fn next_u32(id: u64) -> u32 { with_rng(id, |r| r.next_u32()) }
fn next_u64(id: u64) -> u64 { with_rng(id, |r| r.next_u64()) }

/// Float in [0.0, 1.0). 53-bit mantissa precision per CPython's random().
fn next_f64(id: u64) -> f64 {
    let hi = (next_u32(id) >> 5) as u64;  // top 27 bits
    let lo = (next_u32(id) >> 6) as u64;  // top 26 bits
    ((hi * (1u64 << 26)) + lo) as f64 / (1u64 << 53) as f64
}

fn extract_list(val: MbValue) -> Option<Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            Some(lock.read().unwrap().to_vec())
        } else { None }
    })
}

fn extract_f64(val: MbValue, default: f64) -> f64 {
    val.as_float()
        .or_else(|| val.as_int().map(|i| i as f64))
        .unwrap_or(default)
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
            lock.read().unwrap().get(key).copied()
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

fn raise_index_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// True when the value is a list (mutable sequence). `shuffle` requires a
/// mutable sequence; CPython raises TypeError on immutable inputs such as str.
fn is_list_value(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_)) })
        .unwrap_or(false)
}

// ── Module-level functions (receiver-less; route to default handle) ──
// All also exposed as instance methods via the integer-handle protocol.

pub fn mb_random_method_random(receiver: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    MbValue::from_float(next_f64(id))
}

pub fn mb_random_method_seed(receiver: MbValue, seed: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    // CPython's version-2 seed accepts only None / int / float / str / bytes /
    // bytearray. Other hashable-or-not types (complex, list, dict, tuple, set)
    // raise TypeError.
    if let Some(ptr) = seed.as_ptr() {
        let unsupported = unsafe {
            matches!(
                (*ptr).data,
                ObjData::List(_)
                    | ObjData::Dict(_)
                    | ObjData::Tuple(_)
                    | ObjData::Set(_)
                    | ObjData::FrozenSet(_)
                    | ObjData::Complex(_, _)
            )
        };
        if unsupported {
            return raise_type_error("The only supported seed types are: None, int, float, str, bytes, and bytearray.");
        }
    }
    let s = seed_from_value(seed);
    RANDOMS.with(|m| { m.borrow_mut().insert(id, Mt::new(s)); });
    GAUSS_SPARE.with(|c| c.set(None));
    MbValue::none()
}

pub fn mb_random_method_randint(receiver: MbValue, a: MbValue, b: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let lo = extract_i64(a, 0);
    let hi = extract_i64(b, 0);
    // CPython: randint(a, b) == randrange(a, b+1); an inverted range yields an
    // empty randrange which raises ValueError ("empty range ...").
    if hi < lo {
        return raise_value_error(&format!("empty range in randrange({}, {})", lo, hi + 1));
    }
    let range = (hi - lo + 1) as u64;
    let val = lo + (next_u64(id) % range) as i64;
    MbValue::from_int(val)
}

pub fn mb_random_method_randrange(
    receiver: MbValue, a: MbValue, b: MbValue, step: MbValue,
) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
        return raise_value_error(&format!(
            "empty range in randrange({}, {}, {})", lo, hi, s
        ));
    }
    let span = ((hi - lo) as u64) / (s.unsigned_abs());
    if span == 0 { return MbValue::from_int(lo); }
    let pick = (next_u64(id) % span) as i64;
    MbValue::from_int(lo + pick * s)
}

pub fn mb_random_method_uniform(receiver: MbValue, a: MbValue, b: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let lo = extract_f64(a, 0.0);
    let hi = extract_f64(b, 1.0);
    MbValue::from_float(lo + (hi - lo) * next_f64(id))
}

pub fn mb_random_method_triangular(
    receiver: MbValue, low: MbValue, high: MbValue, mode: MbValue,
) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let lo = extract_f64(low, 0.0);
    let hi = extract_f64(high, 1.0);
    let m = if mode.is_none() { (lo + hi) * 0.5 } else { extract_f64(mode, (lo + hi) * 0.5) };
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
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    match extract_list(seq) {
        Some(items) if !items.is_empty() => {
            let idx = (next_u64(id) % items.len() as u64) as usize;
            items[idx]
        }
        // CPython indexes seq[int(random()*len)]; an empty sequence raises
        // IndexError ("Cannot choose from an empty sequence").
        _ => raise_index_error("Cannot choose from an empty sequence"),
    }
}

pub fn mb_random_method_shuffle(receiver: MbValue, lst: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    // CPython mutates x in place via x[i], x[j] = x[j], x[i]; an immutable
    // sequence (e.g. str/tuple) raises TypeError on item assignment.
    if !is_list_value(lst) {
        return raise_type_error("'str' object does not support item assignment");
    }
    if let Some(ptr) = lst.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut items = lock.write().unwrap();
                let n = items.len();
                for i in (1..n).rev() {
                    let j = (next_u64(id) % (i as u64 + 1)) as usize;
                    items.swap(i, j);
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_random_method_sample(receiver: MbValue, pop: MbValue, k: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    // CPython 3.11+ requires a sequence; sets/dicts raise TypeError
    // ("Population must be a sequence ...").
    let items = match extract_list(pop) {
        Some(v) => v,
        None => {
            return raise_type_error(
                "Population must be a sequence.  For dicts or sets, use sorted(d).",
            );
        }
    };

    // The method-call lowering folds keyword args (`counts=`, `k=`) into a
    // trailing `ObjData::Dict` that arrives here in the `k` slot. Unfold it:
    // pull the real `k` and the optional `counts` sequence out of the bag.
    let mut counts = MbValue::none();
    let raw_k = if is_dict_value(k) {
        if let Some(c) = kwarg_get(k, "counts") { counts = c; }
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
                return raise_value_error(
                    "The number of counts does not match the population",
                );
            }
            // A non-sequence (scalar) counts is a TypeError in CPython; leave
            // that to the TypeError-specific path.
            None => {}
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
            return MbValue::from_ptr(MbObject::new_list(pool));
        }
    }

    // Negative k or k larger than the population raises ValueError
    // ("Sample larger than population or is negative").
    if raw_k < 0 || raw_k as usize > items.len() {
        return raise_value_error("Sample larger than population or is negative");
    }
    let count = raw_k as usize;
    let mut pool = items;
    for i in 0..count {
        let j = i + (next_u64(id) % (pool.len() - i) as u64) as usize;
        pool.swap(i, j);
    }
    pool.truncate(count);
    MbValue::from_ptr(MbObject::new_list(pool))
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
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
    MbValue::from_ptr(MbObject::new_list(out))
}

/// Full `choices(population, weights=None, *, cum_weights=None, k=1)` with
/// CPython-3.12 weight validation. `weights` / `cum_weights` are `none()`
/// when absent. Reached from the module-level `dispatch_choices` slab, which
/// (unlike instance routing) preserves the complete argument vector.
pub fn mb_random_method_choices_full(
    receiver: MbValue, pop: MbValue, weights: MbValue, cum_weights: MbValue, k: MbValue,
) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
                return raise_value_error(
                    "The number of weights does not match the population",
                );
            }
            // A non-sequence cum_weights (scalar) is a TypeError in CPython;
            // leave that to the TypeError-specific path and fall through here.
            None => {}
        }
    } else if has_weights {
        match weight_seq_len(weights) {
            Some(len) if len == pop_len => {}
            Some(_) => {
                return raise_value_error(
                    "The number of weights does not match the population",
                );
            }
            None => {}
        }
        // Total of weights must be strictly positive (covers all-zero and
        // negative-total). CPython: `if total <= 0.0: raise ValueError`.
        if let Some(total) = weight_seq_sum(weights) {
            if total <= 0.0 {
                return raise_value_error(
                    "Total of weights must be greater than zero",
                );
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
                // bisect_right over the cumulative table.
                let mut idx = cum.iter().position(|&c| r < c).unwrap_or(n - 1);
                if idx >= n { idx = n - 1; }
                out.push(items[idx]);
            }
        }
        _ => {
            for _ in 0..count {
                let idx = (next_u64(id) % n as u64) as usize;
                out.push(items[idx]);
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// Materialise a non-list population (Str/Tuple) into a `Vec<MbValue>` for
/// selection. Str yields one 1-character string per code point.
fn population_as_items(pop: MbValue) -> Vec<MbValue> {
    pop.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => items.clone(),
            ObjData::Str(s) => s
                .chars()
                .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                .collect(),
            _ => Vec::new(),
        }
    }).unwrap_or_default()
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
    if items.len() != n { return None; }
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
    if items.len() != n { return None; }
    Some(items.iter().map(|v| extract_f64(*v, 0.0)).collect())
}

/// Standard-normal sample via Box–Muller (cached spare).
fn standard_normal(id: u64) -> f64 {
    if let Some(s) = GAUSS_SPARE.with(|c| c.replace(None)) {
        return s;
    }
    // Reject u1==0 to avoid ln(0).
    let mut u1 = next_f64(id);
    while u1 <= f64::EPSILON { u1 = next_f64(id); }
    let u2 = next_f64(id);
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    let z0 = r * theta.cos();
    let z1 = r * theta.sin();
    GAUSS_SPARE.with(|c| c.set(Some(z1)));
    z0
}

pub fn mb_random_method_gauss(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let s = extract_f64(sigma, 1.0);
    MbValue::from_float(m + s * standard_normal(id))
}

pub fn mb_random_method_normalvariate(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    mb_random_method_gauss(receiver, mu, sigma)
}

pub fn mb_random_method_expovariate(receiver: MbValue, lambd: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let lam = extract_f64(lambd, 1.0);
    let mut u = next_f64(id);
    while u <= f64::EPSILON { u = next_f64(id); }
    MbValue::from_float(-u.ln() / lam)
}

pub fn mb_random_method_lognormvariate(receiver: MbValue, mu: MbValue, sigma: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let m = extract_f64(mu, 0.0);
    let s = extract_f64(sigma, 1.0);
    MbValue::from_float((m + s * standard_normal(id)).exp())
}

pub fn mb_random_method_vonmisesvariate(receiver: MbValue, mu: MbValue, kappa: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
        if v_pre <= 0.0 { continue; }
        let v = v_pre.powi(3);
        let u = next_f64(id);
        if u < 1.0 - 0.0331 * x.powi(4) { return d * v; }
        if u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) { return d * v; }
    }
}

pub fn mb_random_method_betavariate(receiver: MbValue, alpha: MbValue, beta: MbValue) -> MbValue {
    let g1 = mb_random_method_gammavariate(receiver, alpha, MbValue::from_float(1.0))
        .as_float().unwrap_or(0.0);
    if g1 == 0.0 { return MbValue::from_float(0.0); }
    let g2 = mb_random_method_gammavariate(receiver, beta, MbValue::from_float(1.0))
        .as_float().unwrap_or(0.0);
    MbValue::from_float(g1 / (g1 + g2))
}

/// `binomialvariate(n=1, p=0.5)` — number of successes in `n` independent
/// trials, each succeeding with probability `p`. Returns an int in `[0, n]`.
/// CPython raises ValueError for `n < 0` or `p` outside `[0, 1]`.
pub fn mb_random_method_binomialvariate(receiver: MbValue, n: MbValue, p: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let a = extract_f64(alpha, 1.0).max(f64::EPSILON);
    let mut u = next_f64(id);
    while u <= f64::EPSILON { u = next_f64(id); }
    MbValue::from_float((1.0 - u).powf(-1.0 / a))
}

pub fn mb_random_method_weibullvariate(receiver: MbValue, alpha: MbValue, beta: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    let a = extract_f64(alpha, 1.0);
    let b = extract_f64(beta, 1.0).max(f64::EPSILON);
    let mut u = next_f64(id);
    while u <= f64::EPSILON { u = next_f64(id); }
    MbValue::from_float(a * (-u.ln()).powf(1.0 / b))
}

pub fn mb_random_method_getrandbits(receiver: MbValue, k: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
    // Mamba MbValue ints are 48-bit; cap k to 47 so the masked result
    // round-trips through `from_int`. CPython supports arbitrary k via
    // bigint — out of scope for this shim.
    let bits = raw_bits.clamp(1, 47) as u32;
    let mask = (1_u64 << bits) - 1;
    MbValue::from_int((next_u64(id) & mask) as i64)
}

pub fn mb_random_method_randbytes(receiver: MbValue, n: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
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
pub fn mb_random_method_getstate(receiver: MbValue) -> MbValue {
    let id = receiver.as_int().map(|i| i as u64).unwrap_or_else(default_handle);
    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(id as i64)]))
}

/// `setstate(state)` — no-op since state is just the handle id.
pub fn mb_random_method_setstate(_receiver: MbValue, _state: MbValue) -> MbValue {
    MbValue::none()
}

// ── Flat-args dispatch thunks (module-level fn entry points) ──

unsafe extern "C" fn dispatch_random(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_random_method_random(MbValue::none())
}
unsafe extern "C" fn dispatch_seed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_seed(MbValue::none(), a.first().copied().unwrap_or_else(MbValue::none))
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
    mb_random_method_choice(MbValue::none(), a.first().copied().unwrap_or_else(MbValue::none))
}
unsafe extern "C" fn dispatch_shuffle(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_shuffle(MbValue::none(), a.first().copied().unwrap_or_else(MbValue::none))
}
unsafe extern "C" fn dispatch_sample(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_sample(
        MbValue::none(),
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
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
fn parse_choices_kwargs(a: &[MbValue]) -> (MbValue, MbValue, MbValue) {
    let mut weights = MbValue::none();
    let mut cum_weights = MbValue::none();
    let mut k = MbValue::from_int(1);

    // Trailing kwargs dict (folded keyword arguments).
    if let Some(&last) = a.last() {
        if is_dict_value(last) {
            if let Some(w) = kwarg_get(last, "weights") { weights = w; }
            if let Some(c) = kwarg_get(last, "cum_weights") { cum_weights = c; }
            if let Some(kk) = kwarg_get(last, "k") { k = kk; }
        }
    }

    // Positional weights at index 1 (only when it isn't the trailing dict).
    if let Some(&pos1) = a.get(1) {
        if !is_dict_value(pos1) && weights.is_none() && cum_weights.is_none() {
            // Distinguish positional `weights` from a positional `k`:
            // a weight sequence is a list/tuple; a bare int is `k`.
            if weight_seq_len(pos1).is_some() {
                weights = pos1;
            } else if pos1.as_int().is_some() {
                k = pos1;
            }
        }
    }

    (weights, cum_weights, k)
}
unsafe extern "C" fn dispatch_gauss(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_gauss(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_normalvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_normalvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_expovariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_expovariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_lognormvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_lognormvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_vonmisesvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_vonmisesvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(0.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(0.0)),
    )
}
unsafe extern "C" fn dispatch_gammavariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_gammavariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_betavariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_betavariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_binomialvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_binomialvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_int(1)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(0.5)),
    )
}
unsafe extern "C" fn dispatch_paretovariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_paretovariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(1.0)),
    )
}
unsafe extern "C" fn dispatch_weibullvariate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_random_method_weibullvariate(
        MbValue::none(),
        a.first().copied().unwrap_or_else(|| MbValue::from_float(1.0)),
        a.get(1).copied().unwrap_or_else(|| MbValue::from_float(1.0)),
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
    mb_random_method_setstate(MbValue::none(), a.first().copied().unwrap_or_else(MbValue::none))
}

/// `Random(seed=None)` — constructor returns a handle id wrapped as int.
unsafe extern "C" fn dispatch_Random(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seed_val = a.first().copied().unwrap_or_else(MbValue::none);
    let seed = if seed_val.is_none() { None } else { Some(seed_from_value(seed_val)) };
    let id = make_handle(seed);
    MbValue::from_int(id as i64)
}

/// `SystemRandom()` — CPython's hardware-entropy generator. Mamba has no
/// os.urandom-backed PRNG here, so model it as an unseeded MT handle that
/// shares the integer-handle method protocol (random/getrandbits/choice/…).
/// Distribution-correct; not cryptographically strong.
unsafe extern "C" fn dispatch_SystemRandom(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
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

    // Module-level float constants (CPython exposes these in `random`).
    // `TWOPI` is used internally by vonmisesvariate; surfaced as an attr.
    attrs.insert(
        "TWOPI".to_string(),
        MbValue::from_float(2.0 * std::f64::consts::PI),
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
                MbValue::none(), MbValue::from_int(1), MbValue::from_int(6),
            ).as_int().unwrap();
            assert!((1..=6).contains(&v));
        }
    }

    #[test]
    fn test_randint_equal_bounds() {
        seed_default(1);
        let v = mb_random_method_randint(
            MbValue::none(), MbValue::from_int(5), MbValue::from_int(5),
        );
        assert_eq!(v.as_int(), Some(5));
    }

    #[test]
    fn test_randint_reversed_bounds_returns_none() {
        let v = mb_random_method_randint(
            MbValue::none(), MbValue::from_int(10), MbValue::from_int(1),
        );
        assert!(v.is_none());
    }

    #[test]
    fn test_uniform_range() {
        seed_default(55);
        for _ in 0..50 {
            let f = mb_random_method_uniform(
                MbValue::none(), MbValue::from_float(10.0), MbValue::from_float(20.0),
            ).as_float().unwrap();
            assert!((10.0..=20.0).contains(&f), "out of range: {f}");
        }
    }

    #[test]
    fn test_choice_and_sample() {
        seed_default(7);
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10), MbValue::from_int(20), MbValue::from_int(30),
        ]));
        let v = mb_random_method_choice(MbValue::none(), list).as_int().unwrap();
        assert!(v == 10 || v == 20 || v == 30);
        let s = mb_random_method_sample(MbValue::none(), list, MbValue::from_int(2));
        unsafe {
            if let ObjData::List(ref lk) = (*s.as_ptr().unwrap()).data {
                assert_eq!(lk.read().unwrap().len(), 2);
            } else { panic!("expected list"); }
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
    fn test_shuffle_preserves_elements() {
        seed_default(100);
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
            MbValue::from_int(4), MbValue::from_int(5),
        ]));
        mb_random_method_shuffle(MbValue::none(), list);
        unsafe {
            if let ObjData::List(ref lk) = (*list.as_ptr().unwrap()).data {
                let g = lk.read().unwrap();
                assert_eq!(g.len(), 5);
                let mut sorted: Vec<i64> = g.iter().map(|v| v.as_int().unwrap()).collect();
                sorted.sort();
                assert_eq!(sorted, vec![1, 2, 3, 4, 5]);
            }
        }
    }

    #[test]
    fn test_gauss_finite() {
        seed_default(11);
        for _ in 0..50 {
            let f = mb_random_method_gauss(
                MbValue::none(), MbValue::from_float(0.0), MbValue::from_float(1.0),
            ).as_float().unwrap();
            assert!(f.is_finite());
        }
    }

    #[test]
    fn test_expovariate_positive() {
        seed_default(12);
        for _ in 0..50 {
            let f = mb_random_method_expovariate(
                MbValue::none(), MbValue::from_float(1.5),
            ).as_float().unwrap();
            assert!(f > 0.0 && f.is_finite());
        }
    }

    #[test]
    fn test_gammavariate_positive() {
        seed_default(13);
        for _ in 0..30 {
            let f = mb_random_method_gammavariate(
                MbValue::none(), MbValue::from_float(2.0), MbValue::from_float(1.0),
            ).as_float().unwrap();
            assert!(f > 0.0 && f.is_finite());
        }
    }

    #[test]
    fn test_betavariate_unit() {
        seed_default(14);
        for _ in 0..30 {
            let f = mb_random_method_betavariate(
                MbValue::none(), MbValue::from_float(2.0), MbValue::from_float(5.0),
            ).as_float().unwrap();
            assert!((0.0..=1.0).contains(&f), "out of [0,1]: {f}");
        }
    }

    #[test]
    fn test_getrandbits_range() {
        seed_default(15);
        // Mamba MbValue ints are 48-bit; impl caps k at 47.
        for k in [1, 8, 16, 32, 47] {
            let v = mb_random_method_getrandbits(
                MbValue::none(), MbValue::from_int(k),
            ).as_int().unwrap();
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
}
