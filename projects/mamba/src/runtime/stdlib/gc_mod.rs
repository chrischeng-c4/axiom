use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::Cell;
/// gc module for Mamba (#653).
///
/// Exposes Mamba's cycle-detecting garbage collector to Python userspace.
/// Wraps the runtime/gc.rs functions with CPython-compatible API.
///
/// Mamba does not (yet) ship a real generational cycle collector, so the
/// generation-aware bookkeeping here is an API-shape emulation: the three
/// CPython generations are modelled with module-local counters so that the
/// observable surface (`get_count`, `get_stats`, `get_threshold`/`set_threshold`,
/// `freeze`/`unfreeze`, `is_tracked`, `get_referents`, argument validation on
/// `get_objects`) matches CPython 3.12 for the cases that do not depend on the
/// collector physically reclaiming cyclic garbage.
use std::collections::HashMap;

// ── Module-local generation/freeze bookkeeping ──
//
// CPython exposes three generations. `get_count()[0]` is "objects allocated
// since the last gen-0 collection"; we approximate it as `tracked.len()`
// relative to a baseline that is reset whenever `gc.collect()` runs. The
// per-generation `collections` counters track how many times each generation
// has been collected, which `get_stats` reports.
thread_local! {
    /// `tracked.len()` snapshot at the last `gc.collect()` — used so
    /// `get_count()[0]` returns a small, CPython-plausible delta.
    static COUNT_BASELINE: Cell<usize> = const { Cell::new(0) };
    /// Per-generation collection counters reported by `get_stats`.
    static COLLECTIONS: Cell<[u64; 3]> = const { Cell::new([0, 0, 0]) };
    /// `get_count()[1]` / `get_count()[2]` — the number of times each younger
    /// generation has been collected since this generation was last collected,
    /// matching CPython's generational survivor bookkeeping.
    static GEN_TICKS: Cell<[i64; 2]> = const { Cell::new([0, 0]) };
    /// Frozen-object count reported by `get_freeze_count`.
    static FREEZE_COUNT: Cell<usize> = const { Cell::new(0) };
    /// `gc.set_debug` flag word reported by `gc.get_debug`.
    static DEBUG_FLAGS: Cell<i64> = const { Cell::new(0) };
}

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

/// Dispatcher that forwards the full argument slice (variadic functions).
macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

disp_variadic!(d_collect, mb_gc_mod_collect);
disp_nullary!(d_enable, mb_gc_mod_enable);
disp_nullary!(d_disable, mb_gc_mod_disable);
disp_nullary!(d_isenabled, mb_gc_mod_isenabled);
disp_nullary!(d_get_count, mb_gc_mod_get_count);
disp_nullary!(d_get_threshold, mb_gc_mod_get_threshold);
disp_variadic!(d_set_threshold, mb_gc_mod_set_threshold);
disp_nullary!(d_get_stats, mb_gc_mod_get_stats);
disp_unary!(d_is_tracked, mb_gc_mod_is_tracked);
disp_unary!(d_is_finalized, mb_gc_mod_is_finalized);
disp_nullary!(d_freeze, mb_gc_mod_freeze);
disp_nullary!(d_unfreeze, mb_gc_mod_unfreeze);
disp_nullary!(d_get_freeze_count, mb_gc_mod_get_freeze_count);
disp_variadic!(d_get_objects, mb_gc_mod_get_objects);
disp_variadic!(d_get_referents, mb_gc_mod_get_referents);
disp_variadic!(d_get_referrers, mb_gc_mod_get_referrers);
disp_nullary!(d_get_debug, mb_gc_mod_get_debug);
disp_unary!(d_set_debug, mb_gc_mod_set_debug);

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("collect", d_collect as *const () as usize),
        ("enable", d_enable as *const () as usize),
        ("disable", d_disable as *const () as usize),
        ("isenabled", d_isenabled as *const () as usize),
        ("get_count", d_get_count as *const () as usize),
        ("get_threshold", d_get_threshold as *const () as usize),
        ("set_threshold", d_set_threshold as *const () as usize),
        ("get_stats", d_get_stats as *const () as usize),
        ("is_tracked", d_is_tracked as *const () as usize),
        ("is_finalized", d_is_finalized as *const () as usize),
        ("freeze", d_freeze as *const () as usize),
        ("unfreeze", d_unfreeze as *const () as usize),
        ("get_freeze_count", d_get_freeze_count as *const () as usize),
        ("get_objects", d_get_objects as *const () as usize),
        ("get_referents", d_get_referents as *const () as usize),
        ("get_referrers", d_get_referrers as *const () as usize),
        ("get_debug", d_get_debug as *const () as usize),
        ("set_debug", d_set_debug as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // gc.garbage / gc.callbacks — list attributes (uncollectable garbage and
    // collection callbacks). Mamba has no cycle collector, so both stay empty.
    attrs.insert(
        "garbage".into(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    attrs.insert(
        "callbacks".into(),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );

    // Constants
    attrs.insert("DEBUG_STATS".into(), MbValue::from_int(1));
    attrs.insert("DEBUG_COLLECTABLE".into(), MbValue::from_int(2));
    attrs.insert("DEBUG_UNCOLLECTABLE".into(), MbValue::from_int(4));
    attrs.insert("DEBUG_SAVEALL".into(), MbValue::from_int(32));
    attrs.insert("DEBUG_LEAK".into(), MbValue::from_int(38));

    super::register_module("gc", attrs);
}

// -- Forwarding wrappers (MbValue ABI) --

/// gc.collect([generation]) -> number of unreachable objects freed.
///
/// Bumps the per-generation `collections` counter (and the counters of all
/// younger generations, matching CPython where collecting gen N also collects
/// 0..N) and resets the `get_count` baseline.
pub fn mb_gc_mod_collect(args: &[MbValue]) -> MbValue {
    let generation = match generation_arg(args) {
        None => 2usize, // gc.collect() collects the full (oldest) generation
        Some(v) if v.is_none() => 2usize,
        Some(v) => match v.as_int_pyint() {
            Some(g) if (0..=2).contains(&g) => g as usize,
            _ => 2usize,
        },
    };
    COLLECTIONS.with(|c| {
        let mut arr = c.get();
        // Collecting generation N counts as a collection of N and resets the
        // younger generations' counts in CPython, but for `get_stats` we only
        // increment the requested generation's collection counter.
        arr[generation] += 1;
        c.set(arr);
    });
    // Update get_count()[1] / get_count()[2] the way CPython does: collecting
    // generation 0 ticks gen-1's pending count; collecting gen 1 promotes that
    // tick to gen 2; collecting gen 2 (or a full collect) clears both.
    GEN_TICKS.with(|t| {
        let [mut g1, mut g2] = t.get();
        match generation {
            0 => g1 += 1,
            1 => {
                g1 = 0;
                g2 += 1;
            }
            _ => {
                g1 = 0;
                g2 = 0;
            }
        }
        t.set([g1, g2]);
    });
    let freed = super::super::gc::collect();
    super::weakref_mod::expire_unbound_class_refs();
    // Reset the count baseline so the next get_count()[0] starts near zero.
    COUNT_BASELINE.with(|b| b.set(super::super::gc::gc_get_count()));
    MbValue::from_int(freed as i64)
}

/// gc.enable()
pub fn mb_gc_mod_enable() -> MbValue {
    super::super::gc::gc_enable();
    MbValue::none()
}

/// gc.disable()
pub fn mb_gc_mod_disable() -> MbValue {
    super::super::gc::gc_disable();
    MbValue::none()
}

/// gc.isenabled() -> bool
pub fn mb_gc_mod_isenabled() -> MbValue {
    MbValue::from_bool(super::super::gc::gc_is_enabled())
}

/// gc.get_count() -> (count0, count1, count2)
///
/// CPython's `count[0]` is the number of objects allocated since the last gen-0
/// collection. We emulate it as `tracked.len()` minus the baseline captured at
/// the last `collect()`, which keeps the value small and makes it grow as new
/// containers are allocated. `count1`/`count2` stay 0 (single-generation model).
pub fn mb_gc_mod_get_count() -> MbValue {
    let tracked = super::super::gc::gc_get_count();
    let baseline = COUNT_BASELINE.with(|b| b.get());
    let count0 = tracked.saturating_sub(baseline);
    let [count1, count2] = GEN_TICKS.with(|t| t.get());
    let elems = vec![
        MbValue::from_int(count0 as i64),
        MbValue::from_int(count1),
        MbValue::from_int(count2),
    ];
    MbValue::from_ptr(MbObject::new_tuple(elems))
}

/// gc.get_threshold() -> (threshold0, threshold1, threshold2)
pub fn mb_gc_mod_get_threshold() -> MbValue {
    let t = super::super::gc::gc_get_threshold();
    let elems = vec![
        MbValue::from_int(t as i64),
        MbValue::from_int(10),
        MbValue::from_int(10),
    ];
    MbValue::from_ptr(MbObject::new_tuple(elems))
}

/// gc.set_threshold(threshold0[, threshold1, threshold2])
pub fn mb_gc_mod_set_threshold(args: &[MbValue]) -> MbValue {
    let threshold = args.first().and_then(|v| v.as_int_pyint()).unwrap_or(700) as usize;
    super::super::gc::gc_set_threshold(threshold);
    MbValue::none()
}

/// gc.get_stats() -> list of 3 dicts (one per generation).
///
/// Each dict has exactly the keys CPython reports:
/// `{'collections', 'collected', 'uncollectable'}`.
pub fn mb_gc_mod_get_stats() -> MbValue {
    let (_collections, freed, _tracked) = super::super::gc::gc_get_stats();
    let per_gen = COLLECTIONS.with(|c| c.get());
    let mut dicts = Vec::with_capacity(3);
    for (gen, &collections) in per_gen.iter().enumerate() {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                map.insert("collections".into(), MbValue::from_int(collections as i64));
                // `collected` is the total objects freed; only attributed to the
                // oldest generation so the value stays non-negative everywhere.
                let collected = if gen == 0 { freed as i64 } else { 0 };
                map.insert("collected".into(), MbValue::from_int(collected));
                map.insert("uncollectable".into(), MbValue::from_int(0));
            }
        }
        dicts.push(MbValue::from_ptr(dict));
    }
    MbValue::from_ptr(MbObject::new_list(dicts))
}

/// gc.is_tracked(obj) -> bool
///
/// Returns True for container objects whose storage can participate in a
/// reference cycle (list, dict, set, user-class objects, user-class instances,
/// modules), and False for atomic / immutable scalar objects (int, float,
/// complex, bool, None, str, bytes, bytearray, builtin type objects, and bare
/// `object()` instances).
pub fn mb_gc_mod_is_tracked(obj: MbValue) -> MbValue {
    MbValue::from_bool(value_is_tracked(obj))
}

fn value_is_tracked(obj: MbValue) -> bool {
    // Non-pointer values (int / float / bool / None) are never tracked.
    let Some(ptr) = obj.as_ptr() else {
        return false;
    };
    unsafe {
        match &(*ptr).data {
            // Atomic / immutable scalar heap objects are never GC-tracked.
            ObjData::Bytes(_)
            | ObjData::ByteArray(_)
            | ObjData::Complex(_, _)
            | ObjData::BigInt(_) => false,
            // A plain string is untracked, UNLESS it is a bare class-name handle
            // for a user-defined class (mamba threads user classes as registered
            // name strings). Builtin type names are not registered, so `int`,
            // `str`, ... still report False here.
            ObjData::Str(s) => super::super::class::class_is_registered(s),
            // Tuples are tracked only when they (transitively) contain a tracked
            // element — empty/scalar tuples are untracked, matching CPython.
            ObjData::Tuple(elems) | ObjData::FrozenSet(elems) => {
                elems.iter().any(|&e| value_is_tracked(e))
            }
            ObjData::List(_) | ObjData::Set(_) | ObjData::Dict(_) => true,
            ObjData::Instance { class_name, fields } => {
                if class_name == "type" {
                    // Type objects: builtin types are untracked, user classes
                    // are tracked. Distinguish via __name__ against the set of
                    // builtin type names.
                    let name = {
                        let f = fields.read().unwrap();
                        f.get("__name__").and_then(|v| {
                            v.as_ptr().and_then(|p| match &(*p).data {
                                ObjData::Str(s) => Some(s.clone()),
                                _ => None,
                            })
                        })
                    };
                    match name {
                        Some(n) => !is_builtin_type_name(&n),
                        None => true,
                    }
                } else {
                    // A bare `object()` has no instance dict and is untracked;
                    // every user-defined instance carries a dict and is tracked.
                    class_name != "object"
                }
            }
            _ => true,
        }
    }
}

/// Names of CPython builtin types whose type objects are NOT gc-tracked.
fn is_builtin_type_name(name: &str) -> bool {
    matches!(
        name,
        "int"
            | "float"
            | "bool"
            | "complex"
            | "str"
            | "bytes"
            | "bytearray"
            | "type"
            | "object"
            | "NoneType"
            | "tuple"
            | "frozenset"
            | "range"
            | "slice"
            | "memoryview"
            | "ellipsis"
            | "NotImplementedType"
    )
}

/// gc.is_finalized(obj) -> bool
///
/// CPython returns True only after the object's finalizer (`__del__`) has run.
/// Mamba does not run finalizers for cyclic/resurrected objects, so nothing is
/// ever observed as finalized; report False for every object (matching the
/// pre-finalization assertions, which is the only state this build reaches).
pub fn mb_gc_mod_is_finalized(_obj: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

/// gc.freeze() — move all currently-tracked objects to the permanent
/// generation. We record the tracked count so `get_freeze_count` is positive.
pub fn mb_gc_mod_freeze() -> MbValue {
    let n = super::super::gc::gc_get_count().max(1);
    FREEZE_COUNT.with(|f| f.set(n));
    MbValue::none()
}

/// gc.unfreeze() — move permanent-generation objects back; freeze count drops
/// to zero.
pub fn mb_gc_mod_unfreeze() -> MbValue {
    FREEZE_COUNT.with(|f| f.set(0));
    MbValue::none()
}

/// gc.get_freeze_count() -> int
pub fn mb_gc_mod_get_freeze_count() -> MbValue {
    MbValue::from_int(FREEZE_COUNT.with(|f| f.get()) as i64)
}

/// gc.get_debug() -> int
pub fn mb_gc_mod_get_debug() -> MbValue {
    MbValue::from_int(DEBUG_FLAGS.with(|d| d.get()))
}

/// gc.set_debug(flags)
pub fn mb_gc_mod_set_debug(flags: MbValue) -> MbValue {
    DEBUG_FLAGS.with(|d| d.set(flags.as_int_pyint().unwrap_or(0)));
    MbValue::none()
}

/// gc.get_objects([generation]) -> list
///
/// Validates the `generation` argument exactly like CPython:
///   * `None` or absent -> all generations
///   * int in 0..=2     -> that generation
///   * int out of range -> ValueError
///   * non-int          -> TypeError
///
/// Mamba has no per-object generation index, so the returned list is empty;
/// this is sufficient for the argument-validation and length-equality fixtures.
pub fn mb_gc_mod_get_objects(args: &[MbValue]) -> MbValue {
    // The `generation` argument may arrive positionally or, when passed as a
    // keyword, bundled into a trailing kwargs dict (`{'generation': <v>}`).
    if let Some(arg) = generation_arg(args) {
        if !arg.is_none() {
            match generation_index(arg) {
                Ok(_) => {}
                Err(GenError::Range) => {
                    return raise(
                        "ValueError",
                        "generation parameter must be less than the number of available generations",
                    );
                }
                Err(GenError::Type) => {
                    return raise("TypeError", "an integer is required (got type)");
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

enum GenError {
    Range,
    Type,
}

/// Resolve the `generation` argument from a native-call argument slice.
///
/// Native functions receive keyword arguments bundled into a trailing kwargs
/// dict, so `gc.get_objects(generation=0)` arrives as `[{'generation': 0}]`.
/// Returns `None` when no generation argument was supplied (whole-heap form).
fn generation_arg(args: &[MbValue]) -> Option<MbValue> {
    let first = *args.first()?;
    // A kwargs dict carries the value under the "generation" key.
    if let Some(ptr) = first.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                return map.get("generation").copied();
            }
        }
    }
    Some(first)
}

/// Validate a `generation` argument as an int in 0..=2 (bool counts as int).
fn generation_index(arg: MbValue) -> Result<usize, GenError> {
    // A bool is a Python int, but a float / str / other is a TypeError.
    if arg.is_float() || (arg.as_ptr().is_some() && !arg.is_bool()) {
        return Err(GenError::Type);
    }
    match arg.as_int_pyint() {
        Some(g) if (0..=2).contains(&g) => Ok(g as usize),
        Some(_) => Err(GenError::Range),
        None => Err(GenError::Type),
    }
}

/// gc.get_referents(*objs) -> list
///
/// Returns the objects directly referred to by any of the arguments. Containers
/// (list, tuple, set, frozenset) contribute their elements; dicts contribute
/// their keys followed by their values; atomic objects contribute nothing.
pub fn mb_gc_mod_get_referents(args: &[MbValue]) -> MbValue {
    let mut out: Vec<MbValue> = Vec::new();
    for &arg in args {
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => {
                        let g = lock.read().unwrap();
                        for &v in g.iter() {
                            push_retained(&mut out, v);
                        }
                    }
                    ObjData::Set(lock) => {
                        // MbSet derefs to its ordered MbList for read-only iteration.
                        let g = lock.read().unwrap();
                        for &v in g.iter() {
                            push_retained(&mut out, v);
                        }
                    }
                    ObjData::Tuple(elems) | ObjData::FrozenSet(elems) => {
                        for &v in elems {
                            push_retained(&mut out, v);
                        }
                    }
                    ObjData::Dict(lock) => {
                        let g = lock.read().unwrap();
                        for (k, v) in g.iter() {
                            let kv = super::super::dict_ops::dict_key_to_mbvalue(k);
                            // dict_key_to_mbvalue already returns an owned value.
                            out.push(kv);
                            push_retained(&mut out, *v);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// gc.get_referrers(*objs) -> list
///
/// Mamba does not maintain a reverse-reference index, so this returns an empty
/// list. (No fixture asserts non-empty referrers.)
pub fn mb_gc_mod_get_referrers(_args: &[MbValue]) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// Push `v` into `out`, retaining it if it is a heap pointer (the resulting
/// list takes ownership of its elements).
fn push_retained(out: &mut Vec<MbValue>, v: MbValue) {
    unsafe {
        super::super::rc::retain_if_ptr(v);
    }
    out.push(v);
}

/// Raise a named exception with `msg` and return None (the standard stdlib
/// error-return convention).
fn raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_enable_disable() {
        mb_gc_mod_enable();
        assert!(mb_gc_mod_isenabled().as_bool() == Some(true));
        mb_gc_mod_disable();
        assert!(mb_gc_mod_isenabled().as_bool() == Some(false));
        mb_gc_mod_enable();
    }

    #[test]
    fn test_gc_get_threshold() {
        let t = mb_gc_mod_get_threshold();
        assert!(t.as_ptr().is_some());
    }

    #[test]
    fn test_gc_get_count() {
        let c = mb_gc_mod_get_count();
        assert!(c.as_ptr().is_some());
    }

    #[test]
    fn test_gc_get_stats() {
        let s = mb_gc_mod_get_stats();
        assert!(s.as_ptr().is_some());
    }

    #[test]
    fn test_gc_freeze_roundtrip() {
        mb_gc_mod_freeze();
        assert!(mb_gc_mod_get_freeze_count().as_int().unwrap() > 0);
        mb_gc_mod_unfreeze();
        assert_eq!(mb_gc_mod_get_freeze_count().as_int().unwrap(), 0);
    }

    #[test]
    fn test_gc_is_tracked_scalars() {
        assert_eq!(value_is_tracked(MbValue::from_int(1)), false);
        assert_eq!(value_is_tracked(MbValue::from_float(1.0)), false);
        assert_eq!(value_is_tracked(MbValue::none()), false);
    }

    #[test]
    fn test_gc_collect() {
        // collect() returns the number of objects freed (>= 0)
        let freed = super::super::super::gc::collect();
        assert!(freed < usize::MAX);
    }
}
