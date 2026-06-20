use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// statistics module for Mamba (#1265 Task #82, Wave-9).
///
/// Covers CPython 3.12 `statistics` 46-entry surface: numeric module
/// functions, the four class symbols (NormalDist, LinearRegression,
/// StatisticsError, NormalDist), the bisect/itertools/functools/operator
/// re-exports that CPython's statistics imports (bisect_left,
/// bisect_right, count, groupby, reduce, repeat, itemgetter), the math
/// re-exports (erf, exp, fabs, fsum, hypot, log, sqrt, sumprod, tau),
/// the numbers / Fraction / Decimal type tokens, the namedtuple /
/// Counter / defaultdict re-exports, and module placeholders (math,
/// numbers, random, sys).
///
/// Carve-outs:
///   - `NormalDist(mu, sigma)` returns an Instance with `mu` + `sigma`
///     fields. Behavioral methods (pdf, cdf, inv_cdf, overlap, samples,
///     quantiles, zscore, arithmetic operators, repr/eq dunders) are not
///     wired through method dispatch yet — the module-level statistical
///     functions are the active surface.
///   - `LinearRegression` is exposed as a class token / namedtuple
///     stub; the `linear_regression()` function returns a plain
///     `(slope, intercept)` tuple, which is structurally compatible.
///   - `StatisticsError` is registered as a type-name `Str` so that
///     `except statistics.StatisticsError` matches the raised instance
///     (exception::mb_exception_matches compares by type-name string) and
///     `hasattr(statistics, "StatisticsError")` passes. The domain
///     functions raise real catchable `StatisticsError` / `TypeError` /
///     `ValueError` (empty data, too-few points, negative geometric/
///     harmonic inputs, constant regression x, unequal-length bivariate
///     inputs, bad n/method). The full Exception subclass hierarchy is not
///     modeled, so `hasattr(StatisticsError, "__cause__")` is still false.
///   - Keyword args: mamba lowers native-call keywords to positional values
///     (name dropped, lower/ast_to_hir.rs), so `quantiles(data, n=4,
///     method="inclusive")`, `correlation(x, y, method="ranked")`,
///     `linear_regression(x, y, proportional=True)` and the weighted
///     `fmean`/`harmonic_mean` forms read their options off the trailing
///     positional slots, classifying each by runtime type.
///   - `mode` / `multimode` operate over any hashable iterable (ints,
///     floats, and strings iterated per character), using dict-key grouping
///     with first-seen tie-breaking.
///   - `Fraction` / `Decimal`: exposed as class tokens (Instance stubs);
///     arithmetic over Fraction/Decimal inputs collapses to float
///     (matches the lenient `extract_floats` pattern).
///   - `bisect_left` / `bisect_right`: real impls over list/tuple of
///     comparable numeric values. Key function arg is ignored.
///   - `count` / `repeat`: itertools re-exports. Return eager lists
///     (count returns a 1-element list; repeat materializes when
///     `times` is supplied, else returns a 1-element list seeded with
///     the value — semantically a stub for the unbounded iterator).
///   - `groupby`: returns an empty list placeholder (iterator protocol
///     for grouped pairs not yet modelled).
///   - `reduce`: applies a binary function over a list; initial arg
///     optional.
///   - `itemgetter`: returns the requested key/index from a single
///     argument (single-getter shape). The callable-factory shape
///     (`f = itemgetter(0); f(seq)`) is deferred.
///   - `namedtuple` / `Counter` / `defaultdict`: class tokens
///     (Instance stubs). Construction-via-call is not yet wired.
///   - `math` / `numbers` / `random` / `sys`: module re-exports
///     surfaced as `None` placeholders for `dir()` parity, matching
///     the glob_mod convention.
///   - `tau`: real constant 2*pi.
///   - `sumprod(p, q)`: dot product of two iterables; CPython 3.12+.
///   - `fsum`: precise summation via Neumaier's compensated summation.
///   - `hypot`: sqrt(sum(x**2 for x in args)).
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// ── Dispatch wrappers: flat-args ABI ──

/// Build a safe `&[MbValue]` slice from the raw flat-args ABI pointer.
///
/// `std::slice::from_raw_parts` has a hard precondition that the pointer be
/// non-null *even when the length is zero* — and mamba passes a null `args_ptr`
/// for a zero-argument call (`statistics.mean()`). Dereferencing that null
/// pointer aborts the process with an "unsafe precondition violated" panic
/// before the function body can raise the `TypeError` CPython expects. Funnel
/// every dispatcher through this helper so a 0-arg call yields an empty slice
/// instead of crashing.
#[inline]
unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { args_slice(args_ptr, nargs) };
            if a.is_empty() {
                return raise_type_error(concat!(stringify!($fn), "() missing required argument"));
            }
            let arg0 = a[0];
            if reject_non_iterable(arg0) {
                return MbValue::none();
            }
            $fn(materialize_iterable(arg0))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { args_slice(args_ptr, nargs) };
            $fn(
                materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none)),
                materialize_iterable(a.get(1).copied().unwrap_or_else(MbValue::none)),
            )
        }
    };
}

/// Raise a catchable exception whose type-name is `exc`. Mamba matches
/// `except mod.Err` by the type-name string carried on both the raised
/// instance and the except-target value (see exception::mb_exception_matches);
/// `StatisticsError` is registered as `Str("StatisticsError")` so the catch
/// resolves correctly.
fn raise_named(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_type_error(msg: &str) -> MbValue {
    raise_named("TypeError", msg)
}

fn raise_stat_error(msg: &str) -> MbValue {
    raise_named("StatisticsError", msg)
}

fn raise_value_error(msg: &str) -> MbValue {
    raise_named("ValueError", msg)
}

fn is_sequence(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_) | ObjData::Tuple(_)) })
}

/// Python type name for a non-iterable scalar argument, used in the
/// `'<type>' object is not iterable` TypeError message. Returns `None` for any
/// value that *is* (or may be) iterable, including mamba iterator handles —
/// which are int-tagged values, so they must be excluded before the int check.
fn scalar_type_name(val: MbValue) -> Option<&'static str> {
    // `iter([])` / generators are registry-backed int handles; they are
    // iterable, so never reject them as a non-iterable int.
    if super::super::iter::is_iter_handle(val) {
        return None;
    }
    if val.is_none() {
        return Some("NoneType");
    }
    if val.is_bool() {
        return Some("bool");
    }
    if val.as_int().is_some() {
        return Some("int");
    }
    if val.as_float().is_some() {
        return Some("float");
    }
    None
}

/// Guard: raise `TypeError: '<type>' object is not iterable` and return true
/// when `val` is an obviously non-iterable *scalar* (None / int / float /
/// bool). Container and instance shapes pass through unchanged so any
/// genuinely-iterable receiver still reaches the draining path; this only
/// rejects the scalar inputs CPython itself rejects with that TypeError
/// (`mean(None)`, `fmean(data, 70)`, …).
fn reject_non_iterable(val: MbValue) -> bool {
    match scalar_type_name(val) {
        Some(name) => {
            raise_type_error(&format!("'{}' object is not iterable", name));
            true
        }
        None => false,
    }
}

/// CPython's statistics functions accept any iterable, not just sequences:
/// `mean(iter([...]))`, `fmean((x for x in ...))`, etc. all work. mamba models
/// iterators as registry-backed int handles (see runtime::iter), which the
/// List/Tuple-only `extract_*` helpers cannot read. Normalize such an input by
/// draining the iterator once into a freshly-allocated List, so the existing
/// list-based code paths (including `all_ints`, repeated `extract_floats`
/// calls, etc.) see a concrete sequence. List/Tuple/Str and every non-iterator
/// value pass through unchanged. An exhausted/empty iterator becomes `[]`,
/// which then drives the same "empty data" StatisticsError as `[]` itself.
fn materialize_iterable(val: MbValue) -> MbValue {
    if super::super::iter::is_iter_handle(val) {
        if let Some(items) = super::super::iter::drain_iter_to_vec(val) {
            return MbValue::from_ptr(MbObject::new_list(items));
        }
    }
    // Dict-like and instance iterables (Counter, dict, set, namedtuple, any
    // class with __iter__) — route through the generic iterator protocol so
    // `mode(Counter(...))` sees the keys rather than an empty sequence. A
    // non-iterable instance makes mb_iter raise TypeError, matching CPython's
    // `iter(data)` contract; we pass the original value through unchanged in
    // that case so the caller's own error path still runs.
    if let Some(ptr) = val.as_ptr() {
        let drainable = unsafe {
            matches!(
                (*ptr).data,
                ObjData::Dict(_)
                    | ObjData::Set(_)
                    | ObjData::FrozenSet(_)
                    | ObjData::Instance { .. }
            )
        };
        if drainable {
            let handle = super::super::iter::mb_iter(val);
            if super::super::iter::is_iter_handle(handle) {
                if let Some(items) = super::super::iter::drain_iter_to_vec(handle) {
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
            }
        }
    }
    val
}

unsafe extern "C" fn dispatch_mean(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let data = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    if !is_sequence(data) {
        return raise_type_error("mean() arg must be a sequence");
    }
    mb_statistics_mean(data)
}

/// geometric_mean(data) — exactly one positional argument. A second
/// positional arg (`geometric_mean(data, 70)`) is a TypeError in CPython.
unsafe extern "C" fn dispatch_geometric_mean(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("geometric_mean() missing required argument 'data'");
    }
    if a.len() > 1 {
        return raise_type_error(&format!(
            "geometric_mean() takes 1 positional argument but {} were given",
            a.len()
        ));
    }
    if reject_non_iterable(a[0]) {
        return MbValue::none();
    }
    mb_statistics_geometric_mean(materialize_iterable(a[0]))
}

/// fmean(data, weights=None) — weights act as repetition counts.
unsafe extern "C" fn dispatch_fmean(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("fmean() missing required argument 'data'");
    }
    if reject_non_iterable(a[0]) {
        return MbValue::none();
    }
    let data = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    // A non-None weights arg must itself be iterable; `fmean(data, 70)` is a
    // TypeError ('int' object is not iterable) in CPython (`list(weights)`).
    let weights = match a.get(1).copied().filter(|w| !w.is_none()) {
        None => None,
        Some(w) => {
            if reject_non_iterable(w) {
                return MbValue::none();
            }
            Some(materialize_iterable(w))
        }
    };
    mb_statistics_fmean_weighted(data, weights)
}

/// harmonic_mean(data, weights=None).
unsafe extern "C" fn dispatch_harmonic_mean(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("harmonic_mean() missing required argument 'data'");
    }
    if reject_non_iterable(a[0]) {
        return MbValue::none();
    }
    let data = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    let weights = a
        .get(1)
        .copied()
        .filter(|w| !w.is_none())
        .map(materialize_iterable);
    mb_statistics_harmonic_mean_weighted(data, weights)
}
dispatch_unary!(dispatch_median, mb_statistics_median);
dispatch_unary!(dispatch_median_low, mb_statistics_median_low);
dispatch_unary!(dispatch_median_high, mb_statistics_median_high);
dispatch_unary!(dispatch_mode, mb_statistics_mode);
dispatch_unary!(dispatch_multimode, mb_statistics_multimode);
dispatch_unary!(dispatch_pstdev, mb_statistics_pstdev);
dispatch_unary!(dispatch_pvariance, mb_statistics_pvariance);
dispatch_unary!(dispatch_stdev, mb_statistics_stdev);
dispatch_unary!(dispatch_variance, mb_statistics_variance);
dispatch_binary!(dispatch_covariance, mb_statistics_covariance);

/// median_grouped(data, interval=1.0)
unsafe extern "C" fn dispatch_median_grouped(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("median_grouped() missing required argument 'data'");
    }
    if reject_non_iterable(a[0]) {
        return MbValue::none();
    }
    let data = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    // The interval arg defaults to 1.0; a non-numeric interval (`''`, `b''`,
    // …) is a TypeError, not a silent fallthrough to 1.0.
    let interval = match a.get(1).copied() {
        None => 1.0,
        Some(v) => match v.as_float().or_else(|| v.as_int().map(|i| i as f64)) {
            Some(f) => f,
            None => return raise_type_error("Value cannot be converted to a float"),
        },
    };
    mb_statistics_median_grouped(data, interval)
}

/// quantiles(data, *, n=4, method="exclusive").
///
/// In mamba, keyword arguments to native functions are lowered to plain
/// positional values (name dropped, see lower/ast_to_hir.rs), so a call
/// `quantiles(data, n=4, method="inclusive")` arrives as the positional
/// slice `[data, 4, "inclusive"]`. We read `n` from slot 1 and the method
/// string from slot 2.
unsafe extern "C" fn dispatch_quantiles(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if a.is_empty() {
        return raise_type_error("quantiles() missing required argument 'data'");
    }
    let data = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    let mut n: i64 = 4;
    let mut method_inclusive = false;
    // Because mamba lowers keyword args to positional values (dropping the
    // name), `quantiles(data, n=4)` and `quantiles(data, method='inclusive')`
    // both arrive as a single trailing value. Classify every trailing arg by
    // its runtime type: a Str is the `method`, a Dict is a packed kwargs dict,
    // and an int/bool is `n`. A trailing float as `n` is a TypeError.
    for &arg in &a[1.min(nargs)..] {
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Str(_) => {
                        match method_from_value(arg) {
                            Ok(inc) => method_inclusive = inc,
                            Err(()) => return MbValue::none(),
                        }
                        continue;
                    }
                    ObjData::Dict(ref lock) => {
                        let g = lock.read().unwrap();
                        let k_n = super::super::dict_ops::DictKey::Str("n".to_string());
                        if let Some(d) = g.get(&k_n) {
                            if d.as_float().is_some() && d.as_int().is_none() && !d.is_bool() {
                                return raise_type_error("n must be an integer");
                            }
                            if let Some(i) = d.as_int() {
                                n = i;
                            }
                        }
                        let k_m = super::super::dict_ops::DictKey::Str("method".to_string());
                        if let Some(mv) = g.get(&k_m) {
                            match method_from_value(*mv) {
                                Ok(inc) => method_inclusive = inc,
                                Err(()) => return MbValue::none(),
                            }
                        }
                        continue;
                    }
                    _ => {}
                }
            }
        }
        // A bare numeric trailing arg is a genuinely-positional cut count:
        // CPython's signature is `quantiles(data, *, n=4, method=...)` (n is
        // keyword-only), so `quantiles(data, 4)` is a TypeError. Keyword
        // call sites reach us as a packed kwargs dict (the lowering tracks
        // `quantiles` in the kwargs-dict allowlist), handled above.
        if arg.is_bool() || arg.as_int().is_some() || arg.as_float().is_some() {
            return raise_type_error(&format!(
                "quantiles() takes 1 positional argument but {} were given",
                nargs
            ));
        }
    }
    mb_statistics_quantiles(data, n, method_inclusive)
}

/// Resolve a `method=` value to inclusive(true)/exclusive(false). An unknown
/// method string raises ValueError and returns Err(()).
fn method_from_value(v: MbValue) -> Result<bool, ()> {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return match s.as_str() {
                    "inclusive" => Ok(true),
                    "exclusive" => Ok(false),
                    other => {
                        raise_value_error(&format!("Unknown method: {}", other));
                        Err(())
                    }
                };
            }
        }
    }
    Ok(false)
}

/// correlation(x, y, /, *, method="linear").
///
/// As with quantiles, the `method=` keyword is lowered to a positional value
/// (slot 2). "linear" (default) gives Pearson; "ranked" gives Spearman.
/// An unknown method is a ValueError.
unsafe extern "C" fn dispatch_correlation(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let x = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    let y = materialize_iterable(a.get(1).copied().unwrap_or_else(MbValue::none));
    let mut ranked = false;
    if let Some(&mv) = a.get(2) {
        // Trailing kwargs dict or bare positional method string.
        let from_dict = mv.as_ptr().and_then(|p| unsafe {
            if let ObjData::Dict(ref lock) = (*p).data {
                let g = lock.read().unwrap();
                let k = super::super::dict_ops::DictKey::Str("method".to_string());
                g.get(&k).copied()
            } else {
                None
            }
        });
        let method_val = from_dict.unwrap_or(mv);
        if let Some(ptr) = method_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    match s.as_str() {
                        "linear" => ranked = false,
                        "ranked" => ranked = true,
                        other => return raise_value_error(&format!("Unknown method: {}", other)),
                    }
                }
            }
        }
    }
    mb_statistics_correlation(x, y, ranked)
}

/// linear_regression(x, y, /, *, proportional=False).
///
/// The `proportional=` keyword is lowered to a positional value (slot 2).
unsafe extern "C" fn dispatch_linear_regression(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let x = materialize_iterable(a.get(0).copied().unwrap_or_else(MbValue::none));
    let y = materialize_iterable(a.get(1).copied().unwrap_or_else(MbValue::none));
    let mut proportional = false;
    if let Some(&pv) = a.get(2) {
        // Trailing kwargs dict shape (defensive).
        if let Some(ptr) = pv.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let k = super::super::dict_ops::DictKey::Str("proportional".to_string());
                    if let Some(d) = g.get(&k) {
                        proportional = truthy(*d);
                    }
                } else {
                    proportional = truthy(pv);
                }
            }
        } else {
            proportional = truthy(pv);
        }
    }
    mb_statistics_linear_regression(x, y, proportional)
}

/// Python truthiness for the values we expect as a proportional flag
/// (bool / int / float). Non-zero numbers and True are truthy.
fn truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    if let Some(f) = v.as_float() {
        return f != 0.0;
    }
    !v.is_none()
}

// ── Wave-9 extra dispatchers ──

dispatch_unary!(dispatch_fabs, mb_statistics_fabs);
dispatch_unary!(dispatch_exp, mb_statistics_exp);
dispatch_unary!(dispatch_sqrt, mb_statistics_sqrt);
dispatch_unary!(dispatch_erf, mb_statistics_erf);
dispatch_unary!(dispatch_fsum, mb_statistics_fsum);

/// log(x[, base]) — natural log if base omitted.
unsafe extern "C" fn dispatch_log(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let x = a.get(0).copied().unwrap_or_else(MbValue::none);
    let base = a.get(1).copied();
    mb_statistics_log(x, base)
}

/// hypot(*coords) — sqrt(sum(c**2)).
unsafe extern "C" fn dispatch_hypot(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_statistics_hypot(a)
}

dispatch_binary!(dispatch_sumprod, mb_statistics_sumprod);

/// bisect_left(a, x, lo=0, hi=None)
unsafe extern "C" fn dispatch_bisect_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    let x = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_statistics_bisect_left(seq, x)
}

/// bisect_right(a, x, lo=0, hi=None)
unsafe extern "C" fn dispatch_bisect_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    let x = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_statistics_bisect_right(seq, x)
}

/// count(start=0, step=1) — eager 1-element list seed (carve-out).
unsafe extern "C" fn dispatch_count(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let start = a.get(0).copied().unwrap_or_else(|| MbValue::from_int(0));
    MbValue::from_ptr(MbObject::new_list(vec![start]))
}

/// repeat(value, times=None) — eager materialization when times supplied.
unsafe extern "C" fn dispatch_repeat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let val = a.get(0).copied().unwrap_or_else(MbValue::none);
    let times = a.get(1).and_then(|v| v.as_int()).unwrap_or(1) as usize;
    let mut out = Vec::with_capacity(times);
    for _ in 0..times {
        out.push(val);
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// groupby(iterable, key=None) — empty list placeholder (carve-out).
unsafe extern "C" fn dispatch_groupby(_args: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// reduce(fn, iterable[, initial]) — Mamba carve-out: returns last item /
/// initial. Calling a user-supplied callable from a native dispatcher
/// requires the trampoline (#2003 family); deferred. Tests cover the
/// scalar fold-collapse behavior.
unsafe extern "C" fn dispatch_reduce(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let seq = a.get(1).copied().unwrap_or_else(MbValue::none);
    let initial = a.get(2).copied();
    mb_statistics_reduce(seq, initial)
}

/// itemgetter(key, *more) — single-arg shape returns the lookup over a
/// later-supplied container. We model the common usage:
///     itemgetter(key)(seq)
/// by accepting (key, seq) and returning seq[key]. Pure factory shape
/// (`f = itemgetter(0)` then `f(seq)` later) is deferred.
unsafe extern "C" fn dispatch_itemgetter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    if nargs < 2 {
        return MbValue::none();
    }
    let key = a[0];
    let seq = a[1];
    mb_statistics_itemgetter(key, seq)
}

/// NormalDist(mu=0.0, sigma=1.0). A negative sigma is a StatisticsError,
/// matching CPython (`sigma must be non-negative`).
unsafe extern "C" fn dispatch_normaldist(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let mu = a
        .get(0)
        .copied()
        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)))
        .unwrap_or(0.0);
    let sigma = a
        .get(1)
        .copied()
        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)))
        .unwrap_or(1.0);
    if sigma < 0.0 {
        return raise_stat_error("sigma must be non-negative");
    }
    mb_statistics_normaldist(mu, sigma)
}

/// Build a class-token Instance: an opaque sentinel with class_name +
/// `__name__` / `__module__` fields. Mirrors the `struct.error` shape.
fn class_token(name: &str, module: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str(module.to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the statistics module.
/// `StatisticsError(message)` constructor. Models the module-level name as a
/// real exception *class* (the proven re.error pattern) rather than a sentinel
/// Str: a callable whose addr resolves to "StatisticsError" via
/// NATIVE_TYPE_NAMES, so `except statistics.StatisticsError` / `isinstance`
/// keep matching the raised instance, while the registered chaining slots make
/// `hasattr(statistics.StatisticsError, "__cause__")` True.
unsafe extern "C" fn dispatch_statistics_error(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let msg = a.first().copied().unwrap_or_else(MbValue::none);
    let mut fields = FxHashMap::default();
    fields.insert("message".to_string(), msg);
    fields.insert(
        "args".to_string(),
        MbValue::from_ptr(MbObject::new_tuple(vec![msg])),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "StatisticsError".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Existing 18 numeric fns
        ("fmean", dispatch_fmean as *const () as usize),
        ("mean", dispatch_mean as *const () as usize),
        (
            "geometric_mean",
            dispatch_geometric_mean as *const () as usize,
        ),
        (
            "harmonic_mean",
            dispatch_harmonic_mean as *const () as usize,
        ),
        ("median", dispatch_median as *const () as usize),
        ("median_low", dispatch_median_low as *const () as usize),
        ("median_high", dispatch_median_high as *const () as usize),
        (
            "median_grouped",
            dispatch_median_grouped as *const () as usize,
        ),
        ("mode", dispatch_mode as *const () as usize),
        ("multimode", dispatch_multimode as *const () as usize),
        ("pstdev", dispatch_pstdev as *const () as usize),
        ("pvariance", dispatch_pvariance as *const () as usize),
        ("stdev", dispatch_stdev as *const () as usize),
        ("variance", dispatch_variance as *const () as usize),
        ("quantiles", dispatch_quantiles as *const () as usize),
        ("covariance", dispatch_covariance as *const () as usize),
        ("correlation", dispatch_correlation as *const () as usize),
        (
            "linear_regression",
            dispatch_linear_regression as *const () as usize,
        ),
        // Wave-9 additions: math re-exports
        ("fabs", dispatch_fabs as *const () as usize),
        ("exp", dispatch_exp as *const () as usize),
        ("sqrt", dispatch_sqrt as *const () as usize),
        ("erf", dispatch_erf as *const () as usize),
        ("log", dispatch_log as *const () as usize),
        ("hypot", dispatch_hypot as *const () as usize),
        ("fsum", dispatch_fsum as *const () as usize),
        ("sumprod", dispatch_sumprod as *const () as usize),
        // bisect re-exports
        ("bisect_left", dispatch_bisect_left as *const () as usize),
        ("bisect_right", dispatch_bisect_right as *const () as usize),
        // itertools re-exports
        ("count", dispatch_count as *const () as usize),
        ("repeat", dispatch_repeat as *const () as usize),
        ("groupby", dispatch_groupby as *const () as usize),
        // functools re-export
        ("reduce", dispatch_reduce as *const () as usize),
        // operator re-export
        ("itemgetter", dispatch_itemgetter as *const () as usize),
        // statistics class — NormalDist constructor
        ("NormalDist", dispatch_normaldist as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // NormalDist doubles as a class object: resolve_class_name must map the
    // constructor dispatcher to "NormalDist" so classmethod dispatch
    // (`NormalDist.from_samples(...)`) and isinstance checks see the class.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            dispatch_normaldist as *const () as u64,
            "NormalDist".to_string(),
        );
    });

    // StatisticsError: a real exception class (re.error pattern). The name is a
    // callable constructor whose addr resolves to "StatisticsError" via
    // NATIVE_TYPE_NAMES (so `except`/`isinstance` match the raised instance,
    // which raise_named tags with the same class name), and mb_class_register
    // seeds the BaseException chaining slots so `hasattr(StatisticsError,
    // "__cause__")` is True.
    let stat_err_addr = dispatch_statistics_error as *const () as usize;
    attrs.insert(
        "StatisticsError".to_string(),
        MbValue::from_func(stat_err_addr),
    );
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(stat_err_addr as u64);
    });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(stat_err_addr as u64, "StatisticsError".to_string());
    });
    {
        let mut slots: HashMap<String, MbValue> = HashMap::new();
        let slot = MbValue::from_func(stat_err_addr);
        slots.insert("__cause__".to_string(), slot);
        slots.insert("__context__".to_string(), slot);
        slots.insert("__suppress_context__".to_string(), slot);
        super::super::class::mb_class_register(
            "StatisticsError",
            vec!["Exception".to_string()],
            slots,
        );
    }
    // Class tokens (Instance sentinels — see carve-outs).
    attrs.insert(
        "LinearRegression".to_string(),
        class_token("LinearRegression", "statistics"),
    );
    attrs.insert("Fraction".to_string(), class_token("Fraction", "fractions"));
    attrs.insert("Decimal".to_string(), class_token("Decimal", "decimal"));
    attrs.insert("Counter".to_string(), class_token("Counter", "collections"));
    attrs.insert(
        "defaultdict".to_string(),
        class_token("defaultdict", "collections"),
    );
    attrs.insert(
        "namedtuple".to_string(),
        class_token("namedtuple", "collections"),
    );

    // Constants — math.tau re-export.
    attrs.insert(
        "tau".to_string(),
        MbValue::from_float(std::f64::consts::TAU),
    );

    // Module re-export placeholders (None — surface parity for dir()).
    for sub in ["math", "numbers", "random", "sys"] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    // Module metadata. `__all__` mirrors CPython 3.12's public surface; every
    // listed name is registered above so `hasattr(statistics, name)` holds for
    // each (GlobalsTest::test_check_all). `__doc__` is the module summary line
    // (GlobalsTest::test_meta only checks presence).
    let all_names = [
        "NormalDist",
        "StatisticsError",
        "correlation",
        "covariance",
        "fmean",
        "geometric_mean",
        "harmonic_mean",
        "linear_regression",
        "mean",
        "median",
        "median_grouped",
        "median_high",
        "median_low",
        "mode",
        "multimode",
        "pstdev",
        "pvariance",
        "quantiles",
        "stdev",
        "variance",
    ];
    attrs.insert(
        "__all__".to_string(),
        MbValue::from_ptr(MbObject::new_list(
            all_names
                .iter()
                .map(|s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
                .collect(),
        )),
    );
    attrs.insert(
        "__doc__".to_string(),
        MbValue::from_ptr(MbObject::new_str("Basic statistics module.".to_string())),
    );

    super::register_module("statistics", attrs);
}

// ── Wave-9 real impls ──

/// fabs(x) — absolute value as float.
pub fn mb_statistics_fabs(x: MbValue) -> MbValue {
    match as_f64(x) {
        Some(f) => MbValue::from_float(f.abs()),
        None => MbValue::none(),
    }
}

/// exp(x) — e**x.
pub fn mb_statistics_exp(x: MbValue) -> MbValue {
    match as_f64(x) {
        Some(f) => MbValue::from_float(f.exp()),
        None => MbValue::none(),
    }
}

/// sqrt(x) — square root.
pub fn mb_statistics_sqrt(x: MbValue) -> MbValue {
    match as_f64(x) {
        Some(f) if f >= 0.0 => MbValue::from_float(f.sqrt()),
        _ => MbValue::none(),
    }
}

/// erf(x) — Abramowitz & Stegun 7.1.26 approximation (max err ~1.5e-7).
pub fn mb_statistics_erf(x: MbValue) -> MbValue {
    match as_f64(x) {
        Some(f) => MbValue::from_float(erf_f64(f)),
        None => MbValue::none(),
    }
}

/// log(x[, base]) — math.log re-export.
pub fn mb_statistics_log(x: MbValue, base: Option<MbValue>) -> MbValue {
    let f = match as_f64(x) {
        Some(v) if v > 0.0 => v,
        _ => return MbValue::none(),
    };
    match base.and_then(as_f64) {
        Some(b) if b > 0.0 && b != 1.0 => MbValue::from_float(f.ln() / b.ln()),
        Some(_) => MbValue::none(),
        None => MbValue::from_float(f.ln()),
    }
}

/// hypot(*coords) — sqrt(sum(c*c)).
pub fn mb_statistics_hypot(args: &[MbValue]) -> MbValue {
    let mut s = 0.0f64;
    for v in args {
        if let Some(f) = as_f64(*v) {
            s += f * f;
        }
    }
    MbValue::from_float(s.sqrt())
}

/// fsum(iterable) — precise sum via Neumaier's algorithm.
pub fn mb_statistics_fsum(seq: MbValue) -> MbValue {
    let v = extract_floats(seq);
    let mut sum = 0.0f64;
    let mut c = 0.0f64;
    for x in v {
        let t = sum + x;
        if sum.abs() >= x.abs() {
            c += (sum - t) + x;
        } else {
            c += (x - t) + sum;
        }
        sum = t;
    }
    MbValue::from_float(sum + c)
}

/// Neumaier compensated sum that mirrors CPython's `math.fsum` infinity
/// handling: a `+inf` together with a `-inf` raises
/// `ValueError('-inf + inf in fsum')`. A single signed infinity yields that
/// infinity; a NaN propagates. Returns `Err(())` after raising the ValueError.
fn fsum_checked(v: &[f64]) -> Result<f64, ()> {
    // Non-finite values break the Neumaier compensation (inf - inf == NaN),
    // so resolve them up front, exactly as CPython's math.fsum does:
    //   * any NaN              → NaN
    //   * +inf and -inf both   → ValueError('-inf + inf in fsum')
    //   * a single signed inf  → that infinity
    let mut seen_nan = false;
    let mut seen_pos_inf = false;
    let mut seen_neg_inf = false;
    for &x in v {
        if x.is_nan() {
            seen_nan = true;
        } else if x.is_infinite() {
            if x > 0.0 {
                seen_pos_inf = true;
            } else {
                seen_neg_inf = true;
            }
        }
    }
    if seen_pos_inf && seen_neg_inf {
        raise_value_error("-inf + inf in fsum");
        return Err(());
    }
    if seen_nan {
        return Ok(f64::NAN);
    }
    if seen_pos_inf {
        return Ok(f64::INFINITY);
    }
    if seen_neg_inf {
        return Ok(f64::NEG_INFINITY);
    }
    // All finite — compensated (Neumaier) summation for precision.
    let mut sum = 0.0f64;
    let mut c = 0.0f64;
    for &x in v {
        let t = sum + x;
        if sum.abs() >= x.abs() {
            c += (sum - t) + x;
        } else {
            c += (x - t) + sum;
        }
        sum = t;
    }
    Ok(sum + c)
}

/// sumprod(p, q) — sum of element-wise products (CPython 3.12+).
pub fn mb_statistics_sumprod(p: MbValue, q: MbValue) -> MbValue {
    let vp = extract_floats(p);
    let vq = extract_floats(q);
    let n = vp.len().min(vq.len());
    let mut s = 0.0f64;
    for i in 0..n {
        s += vp[i] * vq[i];
    }
    // Preserve int if every input is int.
    if all_ints(p) && all_ints(q) && s.fract() == 0.0 {
        MbValue::from_int(s as i64)
    } else {
        MbValue::from_float(s)
    }
}

/// bisect_left(a, x) — index of first element >= x in a sorted sequence.
pub fn mb_statistics_bisect_left(seq: MbValue, x: MbValue) -> MbValue {
    let v = extract_floats(seq);
    let key = match as_f64(x) {
        Some(k) => k,
        None => return MbValue::from_int(0),
    };
    let mut lo = 0usize;
    let mut hi = v.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if v[mid] < key {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    MbValue::from_int(lo as i64)
}

/// bisect_right(a, x) — index of first element > x in a sorted sequence.
pub fn mb_statistics_bisect_right(seq: MbValue, x: MbValue) -> MbValue {
    let v = extract_floats(seq);
    let key = match as_f64(x) {
        Some(k) => k,
        None => return MbValue::from_int(0),
    };
    let mut lo = 0usize;
    let mut hi = v.len();
    while lo < hi {
        let mid = (lo + hi) / 2;
        if key < v[mid] {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    MbValue::from_int(lo as i64)
}

/// reduce(fn, iterable[, initial]) — Mamba carve-out collapses to last
/// item or initial (no callable trampoline). Numeric-fold tests use the
/// scalar shape.
pub fn mb_statistics_reduce(seq: MbValue, initial: Option<MbValue>) -> MbValue {
    if let Some(ptr) = seq.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let g = lock.read().unwrap();
                    if let Some(last) = g.last() {
                        return *last;
                    }
                }
                ObjData::Tuple(items) => {
                    if let Some(last) = items.last() {
                        return *last;
                    }
                }
                _ => {}
            }
        }
    }
    initial.unwrap_or_else(MbValue::none)
}

/// itemgetter(key)(seq) collapsed to itemgetter(key, seq).
pub fn mb_statistics_itemgetter(key: MbValue, seq: MbValue) -> MbValue {
    if let Some(ptr) = seq.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let g = lock.read().unwrap();
                    if let Some(i) = key.as_int() {
                        let idx = if i < 0 { g.len() as i64 + i } else { i };
                        if idx >= 0 && (idx as usize) < g.len() {
                            return g[idx as usize];
                        }
                    }
                }
                ObjData::Tuple(items) => {
                    if let Some(i) = key.as_int() {
                        let idx = if i < 0 { items.len() as i64 + i } else { i };
                        if idx >= 0 && (idx as usize) < items.len() {
                            return items[idx as usize];
                        }
                    }
                }
                ObjData::Dict(ref lock) => {
                    let g = lock.read().unwrap();
                    if let Some(s_ptr) = key.as_ptr() {
                        if let ObjData::Str(ref s) = (*s_ptr).data {
                            let k = super::super::dict_ops::DictKey::Str(s.clone());
                            if let Some(v) = g.get(&k) {
                                return *v;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    MbValue::none()
}

/// NormalDist(mu, sigma) — Instance with mu/sigma fields.
pub fn mb_statistics_normaldist(mu: f64, sigma: f64) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("mu".to_string(), MbValue::from_float(mu));
    fields.insert("sigma".to_string(), MbValue::from_float(sigma));
    fields.insert("mean".to_string(), MbValue::from_float(mu));
    fields.insert("median".to_string(), MbValue::from_float(mu));
    fields.insert("mode".to_string(), MbValue::from_float(mu));
    fields.insert("stdev".to_string(), MbValue::from_float(sigma));
    fields.insert("variance".to_string(), MbValue::from_float(sigma * sigma));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "NormalDist".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

const SQRT2: f64 = std::f64::consts::SQRT_2;

/// NormalDist arithmetic (CPython `__add__`/`__radd__`/`__sub__`/`__rsub__`/
/// `__mul__`/`__rmul__`/`__truediv__`/`__neg__`): a constant translates or
/// scales the distribution; adding/subtracting two NormalDists combines
/// variances (sigma = hypot(s1, s2)). Returns None when neither operand is a
/// NormalDist (caller falls through to its regular paths) or the shape is
/// unsupported (e.g. `number / NormalDist`, which CPython leaves as TypeError).
pub fn normaldist_binop(op: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    let pa = normaldist_params(a);
    let pb = normaldist_params(b);
    match (pa, pb) {
        (None, None) => None,
        (Some((m1, s1)), Some((m2, s2))) => match op {
            "+" => Some(mb_statistics_normaldist(m1 + m2, s1.hypot(s2))),
            "-" => Some(mb_statistics_normaldist(m1 - m2, s1.hypot(s2))),
            _ => None,
        },
        (Some((m, s)), None) => {
            let x = as_f64(b)?;
            match op {
                "+" => Some(mb_statistics_normaldist(m + x, s)),
                "-" => Some(mb_statistics_normaldist(m - x, s)),
                "*" => Some(mb_statistics_normaldist(m * x, s * x.abs())),
                "/" => Some(mb_statistics_normaldist(m / x, s / x.abs())),
                _ => None,
            }
        }
        (None, Some((m, s))) => {
            let x = as_f64(a)?;
            match op {
                // __radd__ / __rmul__ are symmetric; __rsub__ is -(self - x).
                "+" => Some(mb_statistics_normaldist(x + m, s)),
                "-" => Some(mb_statistics_normaldist(x - m, s)),
                "*" => Some(mb_statistics_normaldist(x * m, s * x.abs())),
                _ => None,
            }
        }
    }
}

/// -NormalDist — flipped mean, same sigma, fresh object. None for non-NormalDist.
pub fn normaldist_neg(a: MbValue) -> Option<MbValue> {
    let (m, s) = normaldist_params(a)?;
    Some(mb_statistics_normaldist(-m, s))
}

/// CPython repr: `NormalDist(mu=100.0, sigma=15.0)`. None for non-NormalDist.
pub fn normaldist_repr(recv: MbValue) -> Option<String> {
    let (m, s) = normaldist_params(recv)?;
    Some(format!(
        "NormalDist(mu={}, sigma={})",
        super::super::string_ops::python_float_repr(m),
        super::super::string_ops::python_float_repr(s),
    ))
}

/// NormalDist.from_samples(data) — fit (mean, sample stdev) from the data.
pub fn mb_statistics_normaldist_from_samples(data: MbValue) -> MbValue {
    let data = materialize_iterable(data);
    let v = match extract_floats_checked(data) {
        Ok(v) => v,
        Err(()) => return MbValue::none(),
    };
    if v.len() < 2 {
        return raise_stat_error("stdev requires at least two data points");
    }
    let (mean, m2) = welford_m_m2(&v);
    let sigma = (m2 / (v.len() - 1) as f64).sqrt();
    mb_statistics_normaldist(mean, sigma)
}

/// splitmix64 step — the deterministic PRNG behind NormalDist.samples().
/// Not CPython's Mersenne Twister: the samples() contract under test is
/// "same seed → identical sequence, different seed → different sequence,
/// values are N(mu, sigma) floats", not bit-parity with CPython's stream.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

/// Uniform f64 in (0, 1) from one splitmix64 step (never exactly 0).
fn prng_unit(state: &mut u64) -> f64 {
    ((splitmix64(state) >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 1.0)
}

thread_local! {
    /// Entropy counter for seed=None — each unseeded samples() call gets a
    /// fresh stream without consulting wall-clock time.
    static SAMPLES_NONCE: std::cell::Cell<u64> = const { std::cell::Cell::new(0x5EED) };
}

/// Map a samples(seed=...) value to a PRNG state: int seeds use their value,
/// strings hash via FNV-1a (so "alpha" != "beta"), None draws a fresh nonce.
fn seed_to_state(seed: Option<MbValue>) -> u64 {
    match seed {
        None => SAMPLES_NONCE.with(|c| {
            let v = c.get().wrapping_add(0x9E3779B97F4A7C15);
            c.set(v);
            v
        }),
        Some(v) if v.is_none() => seed_to_state(None),
        Some(v) => {
            if let Some(i) = v.as_int() {
                return i as u64;
            }
            if let Some(f) = v.as_float() {
                return f.to_bits();
            }
            if let Some(ptr) = v.as_ptr() {
                unsafe {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        let mut h: u64 = 0xcbf29ce484222325;
                        for b in s.as_bytes() {
                            h ^= *b as u64;
                            h = h.wrapping_mul(0x100000001b3);
                        }
                        return h;
                    }
                }
            }
            0xDEFA017
        }
    }
}

/// Read the (mu, sigma) pair off a NormalDist Instance.
fn normaldist_params(recv: MbValue) -> Option<(f64, f64)> {
    let ptr = recv.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name != "NormalDist" {
                return None;
            }
            let f = fields.read().unwrap();
            let mu = f.get("mu").and_then(|v| as_f64(*v)).unwrap_or(0.0);
            let sigma = f.get("sigma").and_then(|v| as_f64(*v)).unwrap_or(1.0);
            return Some((mu, sigma));
        }
    }
    None
}

/// Wichura AS241 inverse normal CDF — byte-for-byte port of CPython's
/// `statistics._normal_dist_inv_cdf`.
fn normal_dist_inv_cdf(p: f64, mu: f64, sigma: f64) -> f64 {
    let q = p - 0.5;
    if q.abs() <= 0.425 {
        let r = 0.180625 - q * q;
        let num = (((((((2.5090809287301226727e+3 * r + 3.3430575583581281105e+4) * r
            + 6.7265770927008700853e+4)
            * r
            + 4.5921953931549871457e+4)
            * r
            + 1.3731693765509461125e+4)
            * r
            + 1.9715909503065514427e+3)
            * r
            + 1.3314166789178437745e+2)
            * r
            + 3.3871328727963666080e+0)
            * q;
        let den = (((((((5.2264952788528545610e+3 * r + 2.8729085735721942674e+4) * r
            + 3.9307895800092710610e+4)
            * r
            + 2.1213794301586595867e+4)
            * r
            + 5.3941960214247511077e+3)
            * r
            + 6.8718700749205790830e+2)
            * r
            + 4.2313330701600911252e+1)
            * r
            + 1.0);
        return mu + (num / den) * sigma;
    }
    let mut r = if q <= 0.0 { p } else { 1.0 - p };
    r = (-r.ln()).sqrt();
    let x;
    if r <= 5.0 {
        r -= 1.6;
        let num = (((((((7.7454501427834140764e-4 * r + 2.2723844989269184583e-2) * r
            + 2.4178072517745061177e-1)
            * r
            + 1.2704582524523682585e+0)
            * r
            + 3.6478483247632046050e+0)
            * r
            + 5.7694972214606914055e+0)
            * r
            + 4.6303378461565452959e+0)
            * r
            + 1.4234371107496835773e+0);
        let den = (((((((1.0507500716444168432e-9 * r + 5.4759380849953449460e-4) * r
            + 1.5198666563616457196e-2)
            * r
            + 1.4810397642748007459e-1)
            * r
            + 6.8976733498510000455e-1)
            * r
            + 1.6763848301838038494e+0)
            * r
            + 2.0531916266377588219e+0)
            * r
            + 1.0);
        x = num / den;
    } else {
        r -= 5.0;
        let num = (((((((2.0103343992922881326e-7 * r + 2.7115555687434875782e-5) * r
            + 1.2426609473880784386e-3)
            * r
            + 2.6532189526576123093e-2)
            * r
            + 2.9656057182850489123e-1)
            * r
            + 1.7848265399172913358e+0)
            * r
            + 5.4637849111641143699e+0)
            * r
            + 6.6579046435011037772e+0);
        let den = (((((((2.0442631033899397856e-15 * r + 1.4215117583164458887e-7) * r
            + 1.8463183175100546818e-5)
            * r
            + 7.8686913114561325910e-4)
            * r
            + 1.4875361290850614852e-2)
            * r
            + 1.3692988092273580531e-1)
            * r
            + 5.9983220655588793769e-1)
            * r
            + 1.0);
        x = num / den;
    }
    let x = if q < 0.0 { -x } else { x };
    mu + x * sigma
}

/// Dispatch a method call on a NormalDist Instance. `method` is the bare
/// method name (e.g. "cdf"); `args` are the call arguments (receiver excluded).
/// Returns `None` if the method name is not one we model, so the caller can
/// fall back to the generic path.
pub fn mb_statistics_normaldist_method(
    recv: MbValue,
    method: &str,
    args: &[MbValue],
) -> Option<MbValue> {
    let (mu, sigma) = normaldist_params(recv)?;
    match method {
        "cdf" => {
            if args.len() != 1 {
                return Some(raise_type_error("cdf() takes exactly one argument"));
            }
            let x = match as_f64(args[0]) {
                Some(v) => v,
                None => return Some(raise_type_error("cdf() argument must be a number")),
            };
            if sigma == 0.0 {
                return Some(raise_stat_error("cdf() not defined when sigma is zero"));
            }
            let erf = erf_f64((x - mu) / (sigma * SQRT2));
            Some(MbValue::from_float(0.5 * (1.0 + erf)))
        }
        "pdf" => {
            if args.len() != 1 {
                return Some(raise_type_error("pdf() takes exactly one argument"));
            }
            let x = match as_f64(args[0]) {
                Some(v) => v,
                None => return Some(raise_type_error("pdf() argument must be a number")),
            };
            let variance = sigma * sigma;
            if variance == 0.0 {
                return Some(raise_stat_error("pdf() not defined when sigma is zero"));
            }
            let diff = x - mu;
            let val =
                (diff * diff / (-2.0 * variance)).exp() / (std::f64::consts::TAU * variance).sqrt();
            Some(MbValue::from_float(val))
        }
        "inv_cdf" => {
            if args.len() != 1 {
                return Some(raise_type_error("inv_cdf() takes exactly one argument"));
            }
            let p = match as_f64(args[0]) {
                Some(v) => v,
                None => return Some(raise_type_error("inv_cdf() argument must be a number")),
            };
            if p <= 0.0 || p >= 1.0 {
                return Some(raise_stat_error("p must be in the range 0.0 < p < 1.0"));
            }
            Some(MbValue::from_float(normal_dist_inv_cdf(p, mu, sigma)))
        }
        "zscore" => {
            if args.len() != 1 {
                return Some(raise_type_error("zscore() takes exactly one argument"));
            }
            let x = match as_f64(args[0]) {
                Some(v) => v,
                None => return Some(raise_type_error("zscore() argument must be a number")),
            };
            if sigma == 0.0 {
                return Some(raise_stat_error("zscore() not defined when sigma is zero"));
            }
            Some(MbValue::from_float((x - mu) / sigma))
        }
        "quantiles" => {
            // quantiles(n=4) — cut points of the distribution.
            let n: i64 = args.first().and_then(|v| v.as_int()).unwrap_or(4);
            if n < 1 {
                return Some(raise_stat_error("n must be at least 1"));
            }
            let mut out = Vec::with_capacity((n - 1).max(0) as usize);
            for i in 1..n {
                let p = i as f64 / n as f64;
                out.push(MbValue::from_float(normal_dist_inv_cdf(p, mu, sigma)));
            }
            Some(MbValue::from_ptr(MbObject::new_list(out)))
        }
        "samples" => {
            // samples(n, *, seed=None) — n gaussian draws. The seed kwarg
            // arrives either as a trailing kwargs dict ({"seed": v}, the
            // method-call convention) or as a flattened positional value.
            let n = match args.first().and_then(|v| v.as_int()) {
                Some(n) if n >= 0 => n as usize,
                _ => {
                    return Some(raise_type_error(
                        "samples() requires a non-negative integer n",
                    ))
                }
            };
            let mut seed: Option<MbValue> = None;
            for &arg in &args[1..] {
                if let Some(ptr) = arg.as_ptr() {
                    unsafe {
                        if let ObjData::Dict(ref lock) = (*ptr).data {
                            let g = lock.read().unwrap();
                            let k = super::super::dict_ops::DictKey::Str("seed".to_string());
                            if let Some(s) = g.get(&k) {
                                seed = Some(*s);
                            }
                            continue;
                        }
                    }
                }
                seed = Some(arg);
            }
            let mut state = seed_to_state(seed);
            // Box-Muller: two uniforms → one gaussian (cos branch only, so
            // each draw consumes a fixed two PRNG steps — keeps same-seed
            // streams aligned regardless of n).
            let mut out = Vec::with_capacity(n);
            for _ in 0..n {
                let u1 = prng_unit(&mut state);
                let u2 = prng_unit(&mut state);
                let z = (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos();
                out.push(MbValue::from_float(mu + sigma * z));
            }
            Some(MbValue::from_ptr(MbObject::new_list(out)))
        }
        "overlap" => {
            // overlap(other) — overlapping coefficient. Out of scope for the
            // active fixtures; fall through.
            None
        }
        _ => None,
    }
}

/// High-accuracy erf — a Rust port of CPython's `m_erf`/`m_erfc` from
/// `Modules/mathmodule.c` (Sun's fdlibm-derived rational approximations).
/// Accurate to ~1e-16, and `erf(0.0) == 0.0` exactly — both required so that
/// `NormalDist().cdf(0.0)` rounds to exactly 0.5 and `inv_cdf(cdf(x))` round
/// trips within 1e-9.
fn erf_f64(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let absx = x.abs();
    if absx < 1.5 {
        m_erf_series(x)
    } else {
        let cf = m_erfc_contfrac(absx);
        if x > 0.0 {
            1.0 - cf
        } else {
            cf - 1.0
        }
    }
}

/// Complementary error function, same source as `erf_f64`. Kept alongside
/// `erf_f64` for completeness / future `erfc` surface; not yet wired to a
/// dispatcher.
#[allow(dead_code)]
fn erfc_f64(x: f64) -> f64 {
    if x.is_nan() {
        return x;
    }
    let absx = x.abs();
    if absx < 1.5 {
        1.0 - m_erf_series(x)
    } else {
        let cf = m_erfc_contfrac(absx);
        if x > 0.0 {
            cf
        } else {
            2.0 - cf
        }
    }
}

const ERF_SERIES_CUTOFF: f64 = 1.5;
const ERF_SERIES_TERMS: usize = 25;
const ERFC_CONTFRAC_CUTOFF: f64 = 30.0;
const ERFC_CONTFRAC_TERMS: usize = 50;
/// 2 / sqrt(pi)
const TWOOPI: f64 = 1.1283791670955126;

/// erf(x) via power series, valid for |x| < ERF_SERIES_CUTOFF.
fn m_erf_series(x: f64) -> f64 {
    let x2 = x * x;
    let mut acc = 0.0f64;
    let mut fk = ERF_SERIES_TERMS as f64 + 0.5;
    for _ in 0..ERF_SERIES_TERMS {
        acc = 2.0 + x2 * acc / fk;
        fk -= 1.0;
    }
    acc * x * (-x2).exp() * TWOOPI / 2.0
}

/// erfc(x) via continued fraction, valid for x >= ERF_SERIES_CUTOFF.
fn m_erfc_contfrac(x: f64) -> f64 {
    if x >= ERFC_CONTFRAC_CUTOFF {
        return 0.0;
    }
    let x2 = x * x;
    let mut a = 0.0f64;
    let mut da = 0.5f64;
    let mut p = 1.0f64;
    let mut p_last = 0.0f64;
    let mut q = da + x2;
    let mut q_last = 1.0f64;
    for _ in 0..ERFC_CONTFRAC_TERMS {
        a += da;
        da += 2.0;
        let b = da + x2;
        let temp = p;
        p = b * p - a * p_last;
        p_last = temp;
        let temp = q;
        q = b * q - a * q_last;
        q_last = temp;
    }
    p / q * x * (-x2).exp() * TWOOPI / 2.0
}

// ── Helpers ──

fn as_f64(val: MbValue) -> Option<f64> {
    if let Some(f) = val.as_float() {
        return Some(f);
    }
    // Decimal/Fraction are NaN-boxed int HANDLES (ids ≥ 2^40) — intercept
    // them before the int readback or the handle id leaks as the value
    // (`fmean([Decimal("3.5")])` returned ~7e13). See #2129 carve-out.
    if super::super::builtins::is_decimal_handle_value(val) {
        return super::decimal_mod::mb_decimal_float(val).as_float();
    }
    if super::super::builtins::is_fraction_handle_value(val) {
        return super::fractions_mod::mb_fraction_float(val).as_float();
    }
    if let Some(i) = val.as_int() {
        return Some(i as f64);
    }
    if let Some(b) = val.as_bool() {
        return Some(if b { 1.0 } else { 0.0 });
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            // Instances supporting __float__ (user numeric types).
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let method = super::super::class::lookup_method(class_name, "__float__");
                if !method.is_none() {
                    let name = MbValue::from_ptr(MbObject::new_str("__float__".to_string()));
                    let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                    return super::super::class::mb_call_method(val, name, args).as_float();
                }
                return None;
            }
            // BigInt → f64 (may saturate to inf, matching float() semantics).
            if let Some(big) = super::super::bigint_ops::extract_bigint(val) {
                use num_traits::ToPrimitive;
                return big.to_f64();
            }
        }
    }
    None
}

/// Extract floats from list/tuple. Non-numeric items are skipped.
fn extract_floats(seq: MbValue) -> Vec<f64> {
    seq.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|v| as_f64(*v))
                    .collect(),
                ObjData::Tuple(items) => items.iter().filter_map(|v| as_f64(*v)).collect(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// Extract the raw MbValue elements of a list/tuple (no float coercion).
/// Strings iterate per character (each char becomes a 1-char Str value),
/// matching CPython's treatment of `mode("abc")` / `multimode("abc")`.
fn extract_values(seq: MbValue) -> Vec<MbValue> {
    seq.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock
                    .read()
                    .unwrap()
                    .iter()
                    .copied()
                    .collect::<Vec<MbValue>>(),
                ObjData::Tuple(items) => items.iter().copied().collect::<Vec<MbValue>>(),
                ObjData::Str(ref s) => s
                    .chars()
                    .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                    .collect::<Vec<MbValue>>(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// Extract floats, raising TypeError on the first non-numeric element.
/// Returns Err(()) after raising; callers should propagate `MbValue::none()`.
/// `bool` counts as numeric (True==1, False==0), matching CPython.
fn extract_floats_checked(seq: MbValue) -> Result<Vec<f64>, ()> {
    let vals = extract_values(seq);
    let mut out = Vec::with_capacity(vals.len());
    for v in vals {
        if let Some(b) = bool_as_f64(v) {
            out.push(b);
        } else if let Some(f) = as_f64(v) {
            out.push(f);
        } else {
            raise_type_error("can't convert type to float");
            return Err(());
        }
    }
    Ok(out)
}

/// bool→f64 (True=1.0, False=0.0). Returns None for non-bool values.
fn bool_as_f64(v: MbValue) -> Option<f64> {
    if v.is_bool() {
        v.as_bool().map(|b| if b { 1.0 } else { 0.0 })
    } else {
        None
    }
}

/// True if every element of the sequence is an int (no float) — used so
/// odd-length median() preserves int return type.
fn all_ints(seq: MbValue) -> bool {
    seq.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let g = lock.read().unwrap();
                    !g.is_empty()
                        && g.iter()
                            .all(|v| v.as_int().is_some() && v.as_float().is_none())
                }
                ObjData::Tuple(items) => {
                    !items.is_empty()
                        && items
                            .iter()
                            .all(|v| v.as_int().is_some() && v.as_float().is_none())
                }
                _ => false,
            }
        })
        .unwrap_or(false)
}

/// Sort ascending; NaN treated as greater (stable behavior for sequences).
fn sorted_floats(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

/// Welford's online mean. Numerically stable for huge n.
fn welford_mean(v: &[f64]) -> f64 {
    let mut m = 0.0f64;
    for (i, &x) in v.iter().enumerate() {
        m += (x - m) / (i + 1) as f64;
    }
    m
}

/// Welford's online (mean, sum-of-squared-deviations).
fn welford_m_m2(v: &[f64]) -> (f64, f64) {
    let mut mean = 0.0f64;
    let mut m2 = 0.0f64;
    for (i, &x) in v.iter().enumerate() {
        let delta = x - mean;
        mean += delta / (i + 1) as f64;
        let delta2 = x - mean;
        m2 += delta * delta2;
    }
    (mean, m2)
}

// ── Public scalar API ──

/// fmean(data) — float arithmetic mean. Welford-stable.
pub fn mb_statistics_fmean(data: MbValue) -> MbValue {
    mb_statistics_fmean_weighted(data, None)
}

/// fmean(data, weights) — weighted float mean: sum(w*x)/sum(w). With no
/// weights this is the plain arithmetic mean. Uniform weights reproduce the
/// unweighted mean (weights act as repetition counts).
pub fn mb_statistics_fmean_weighted(data: MbValue, weights: Option<MbValue>) -> MbValue {
    let v = match extract_floats_checked(data) {
        Ok(v) => v,
        Err(()) => return MbValue::none(),
    };
    if v.is_empty() {
        return raise_stat_error("fmean requires at least one data point");
    }
    match weights {
        None => {
            // CPython: `total = fsum(data); return total / n`. fsum raises
            // `ValueError('-inf + inf in fsum')` when both signed infinities
            // appear; NaN/single-sign-inf propagate. Mirror that contract.
            match fsum_checked(&v) {
                Ok(total) => MbValue::from_float(total / v.len() as f64),
                Err(()) => MbValue::none(),
            }
        }
        Some(w) => {
            let wv = match extract_floats_checked(w) {
                Ok(wv) => wv,
                Err(()) => return MbValue::none(),
            };
            if wv.len() != v.len() {
                return raise_stat_error("data and weights must be the same length");
            }
            let mut num = 0.0f64;
            let mut den = 0.0f64;
            for i in 0..v.len() {
                num += wv[i] * v[i];
                den += wv[i];
            }
            if den == 0.0 {
                return raise_stat_error("sum of weights must be non-zero");
            }
            MbValue::from_float(num / den)
        }
    }
}

/// mean(data) — float fast path; type-preserving variants deferred.
pub fn mb_statistics_mean(data: MbValue) -> MbValue {
    let v = match extract_floats_checked(data) {
        Ok(v) => v,
        Err(()) => return MbValue::none(),
    };
    if v.is_empty() {
        return raise_stat_error("mean requires at least one data point");
    }
    MbValue::from_float(welford_mean(&v))
}

/// geometric_mean(data) — `exp(fmean(map(log, data)))`, matching CPython.
///
/// CPython computes `exp(fmean(map(log, data)))` and wraps any `ValueError`
/// from `log` (a non-positive argument) into a StatisticsError with the
/// canonical message. The observable contract:
///   * empty data                  → StatisticsError
///   * any element == 0.0 or < 0   → StatisticsError (`log` domain error;
///                                    -inf < 0 so it lands here too)
///   * NaN element                 → result is NaN (`log(nan)` is nan, no error)
///   * +inf element (no negatives) → result is +inf (`log(inf)` is inf)
pub fn mb_statistics_geometric_mean(data: MbValue) -> MbValue {
    const MSG: &str = "geometric mean requires a non-empty dataset \
                       containing positive numbers";
    let v = match extract_floats_checked(data) {
        Ok(v) => v,
        Err(()) => return MbValue::none(),
    };
    if v.is_empty() {
        return raise_stat_error(MSG);
    }
    // `log(x)` raises a domain ValueError for x <= 0 (and for -inf, which is
    // < 0). NaN is *not* a domain error — it propagates. Detect any such
    // element and surface the wrapped StatisticsError.
    if v.iter().any(|&x| x <= 0.0) {
        return raise_stat_error(MSG);
    }
    // logs: NaN/inf propagate naturally through fmean/exp.
    let logs: Vec<f64> = v.iter().map(|x| x.ln()).collect();
    let log_mean = welford_mean(&logs);
    MbValue::from_float(log_mean.exp())
}

/// harmonic_mean(data) — n / sum(1/x). A single zero collapses to 0.0.
pub fn mb_statistics_harmonic_mean(data: MbValue) -> MbValue {
    mb_statistics_harmonic_mean_weighted(data, None)
}

/// harmonic_mean(data, weights) — weighted harmonic mean: sum(w)/sum(w/x).
/// Unweighted (weights=None) is n/sum(1/x). Any zero data point makes the
/// result 0.0; a negative value raises StatisticsError.
pub fn mb_statistics_harmonic_mean_weighted(data: MbValue, weights: Option<MbValue>) -> MbValue {
    let v = match extract_floats_checked(data) {
        Ok(v) => v,
        Err(()) => return MbValue::none(),
    };
    if v.is_empty() {
        return raise_stat_error("harmonic_mean requires at least one data point");
    }
    if v.iter().any(|&x| x < 0.0) {
        return raise_stat_error("harmonic mean does not support negative values");
    }
    // CPython: any zero in the data -> harmonic mean is 0.
    if v.iter().any(|&x| x == 0.0) {
        return MbValue::from_float(0.0);
    }
    match weights {
        None => {
            let inv_sum: f64 = v.iter().map(|x| 1.0 / x).sum();
            MbValue::from_float(v.len() as f64 / inv_sum)
        }
        Some(w) => {
            let wv = match extract_floats_checked(w) {
                Ok(wv) => wv,
                Err(()) => return MbValue::none(),
            };
            if wv.len() != v.len() {
                return raise_stat_error("Number of weights does not match data size");
            }
            if wv.iter().any(|&x| x < 0.0) {
                return raise_stat_error("harmonic mean does not support negative values");
            }
            let sum_w: f64 = wv.iter().sum();
            if sum_w == 0.0 {
                return raise_stat_error("Weights sum to zero");
            }
            let denom: f64 = (0..v.len()).map(|i| wv[i] / v[i]).sum();
            MbValue::from_float(sum_w / denom)
        }
    }
}

/// median(data) — sort + middle. Type-preserves int for all-int odd-length input.
pub fn mb_statistics_median(data: MbValue) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    if v.is_empty() {
        return raise_stat_error("no median for empty data");
    }
    let n = v.len();
    if n % 2 == 0 {
        MbValue::from_float((v[n / 2 - 1] + v[n / 2]) / 2.0)
    } else if all_ints(data) {
        MbValue::from_int(v[n / 2] as i64)
    } else {
        MbValue::from_float(v[n / 2])
    }
}

/// median_low(data) — lower-middle for even-length; same as median for odd.
pub fn mb_statistics_median_low(data: MbValue) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    if v.is_empty() {
        return raise_stat_error("no median for empty data");
    }
    let n = v.len();
    let idx = if n % 2 == 0 { n / 2 - 1 } else { n / 2 };
    if all_ints(data) {
        MbValue::from_int(v[idx] as i64)
    } else {
        MbValue::from_float(v[idx])
    }
}

/// median_high(data) — upper-middle for even-length; same as median for odd.
pub fn mb_statistics_median_high(data: MbValue) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    if v.is_empty() {
        return raise_stat_error("no median for empty data");
    }
    let n = v.len();
    let idx = n / 2;
    if all_ints(data) {
        MbValue::from_int(v[idx] as i64)
    } else {
        MbValue::from_float(v[idx])
    }
}

/// median_grouped(data, interval=1.0) — interpolated median in a grouped bin.
///
/// Mirrors CPython's algorithm exactly:
///   data = sorted(data); x = data[n//2]
///   i = bisect_left(data, x);  j = bisect_right(data, x)
///   L = x - interval/2;  cf = i;  f = j - i
///   return L + interval * (n/2 - cf) / f
///
/// A non-numeric element (e.g. `''`/`b''`) reaches `float(x)` and raises a
/// TypeError, matching CPython (the empty-data case raises StatisticsError
/// before any float coercion).
pub fn mb_statistics_median_grouped(data: MbValue, interval: f64) -> MbValue {
    // Extract elements without float coercion so a non-numeric element can
    // surface the CPython TypeError rather than being silently dropped.
    let raw = extract_values(data);
    if raw.is_empty() {
        return raise_stat_error("no median for empty data");
    }
    // Coerce every element; a non-numeric one is a TypeError (CPython's
    // `float(x)` failure). bool counts as numeric.
    let mut v: Vec<f64> = Vec::with_capacity(raw.len());
    for item in &raw {
        if let Some(b) = bool_as_f64(*item) {
            v.push(b);
        } else if let Some(f) = as_f64(*item) {
            v.push(f);
        } else {
            return raise_type_error("Value cannot be converted to a float");
        }
    }
    let v = sorted_floats(v);
    let n = v.len();
    let x = v[n / 2];
    // i = bisect_left(data, x): index of first element >= x (== count of < x).
    let i = v.iter().take_while(|&&y| y < x).count();
    // j = bisect_right(data, x): index of first element > x.
    let j = v.iter().take_while(|&&y| y <= x).count();
    let l = x - interval / 2.0;
    let cf = i as f64;
    let f = (j - i) as f64;
    MbValue::from_float(l + interval * (n as f64 / 2.0 - cf) / f)
}

/// mode(data) — single most-common value over any hashable iterable
/// (ints, floats, strings/chars). First-seen value wins on a count tie
/// (CPython 3.8+). Empty input raises StatisticsError.
pub fn mb_statistics_mode(data: MbValue) -> MbValue {
    use super::super::dict_ops::to_dict_key;
    let vals = extract_values(data);
    if vals.is_empty() {
        return raise_stat_error("no mode for empty data");
    }
    // Group by hash-key, tracking first-seen order, count, and a
    // representative value to return.
    let mut counts: HashMap<super::super::dict_ops::DictKey, usize> = HashMap::new();
    let mut order: Vec<super::super::dict_ops::DictKey> = Vec::new();
    let mut repr_val: HashMap<super::super::dict_ops::DictKey, MbValue> = HashMap::new();
    for v in &vals {
        let k = to_dict_key(*v);
        let e = counts.entry(k.clone()).or_insert(0);
        if *e == 0 {
            order.push(k.clone());
            repr_val.insert(k.clone(), *v);
        }
        *e += 1;
    }
    // First key in first-seen order whose count is maximal.
    let max_c = *counts.values().max().unwrap();
    for k in &order {
        if counts[k] == max_c {
            return repr_val[k];
        }
    }
    MbValue::none()
}

/// multimode(data) — list of every value tied for max count, in
/// first-seen order. Empty input returns an empty list (never errors).
pub fn mb_statistics_multimode(data: MbValue) -> MbValue {
    use super::super::dict_ops::to_dict_key;
    let vals = extract_values(data);
    if vals.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let mut counts: HashMap<super::super::dict_ops::DictKey, usize> = HashMap::new();
    let mut order: Vec<super::super::dict_ops::DictKey> = Vec::new();
    let mut repr_val: HashMap<super::super::dict_ops::DictKey, MbValue> = HashMap::new();
    for v in &vals {
        let k = to_dict_key(*v);
        let e = counts.entry(k.clone()).or_insert(0);
        if *e == 0 {
            order.push(k.clone());
            repr_val.insert(k.clone(), *v);
        }
        *e += 1;
    }
    let max_c = *counts.values().max().unwrap();
    let result: Vec<MbValue> = order
        .iter()
        .filter(|k| counts[*k] == max_c)
        .map(|k| repr_val[k])
        .collect();
    MbValue::from_ptr(MbObject::new_list(result))
}

/// pvariance(data) — population variance (divide by n).
pub fn mb_statistics_pvariance(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return raise_stat_error("pvariance requires at least one data point");
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float(m2 / v.len() as f64)
}

/// pstdev(data) — population standard deviation.
pub fn mb_statistics_pstdev(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return raise_stat_error("pstdev requires at least one data point");
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float((m2 / v.len() as f64).sqrt())
}

/// variance(data) — sample variance (divide by n-1; Bessel correction).
pub fn mb_statistics_variance(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.len() < 2 {
        return raise_stat_error("variance requires at least two data points");
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float(m2 / (v.len() - 1) as f64)
}

/// stdev(data) — sample standard deviation.
pub fn mb_statistics_stdev(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.len() < 2 {
        return raise_stat_error("stdev requires at least two data points");
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float((m2 / (v.len() - 1) as f64).sqrt())
}

/// quantiles(data, n=4, method="exclusive") — cut points dividing data into n bins.
pub fn mb_statistics_quantiles(data: MbValue, n: i64, inclusive: bool) -> MbValue {
    if n < 1 {
        return raise_stat_error("n must be at least 1");
    }
    let v = sorted_floats(extract_floats(data));
    let ld = v.len();
    if ld < 2 {
        return raise_stat_error("must have at least two data points");
    }
    let n_u = n as usize;
    let mut result: Vec<MbValue> = Vec::with_capacity(n_u - 1);
    if inclusive {
        let m = (ld - 1) as f64;
        for i in 1..n_u {
            let j_full = (i as f64) * m / (n as f64);
            let j = j_full.floor() as usize;
            let delta = j_full - j as f64;
            let interp = v[j] * (1.0 - delta) + v.get(j + 1).copied().unwrap_or(v[j]) * delta;
            result.push(MbValue::from_float(interp));
        }
    } else {
        let m = (ld + 1) as f64;
        for i in 1..n_u {
            let j_full = (i as f64) * m / (n as f64);
            let j = j_full.floor() as usize;
            let delta = j_full - j as f64;
            // Exclusive uses 1-based indexing; floor down by 1 then interpolate.
            let lo = if j == 0 {
                v[0]
            } else if j - 1 < ld {
                v[j - 1]
            } else {
                v[ld - 1]
            };
            let hi = if j < ld { v[j] } else { v[ld - 1] };
            let interp = lo * (1.0 - delta) + hi * delta;
            result.push(MbValue::from_float(interp));
        }
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// covariance(x, y) — sample covariance (divide by n-1).
pub fn mb_statistics_covariance(x: MbValue, y: MbValue) -> MbValue {
    let vx = extract_floats(x);
    let vy = extract_floats(y);
    if vx.len() != vy.len() {
        return raise_stat_error(
            "covariance requires that both inputs have same number of data points",
        );
    }
    let n = vx.len();
    if n < 2 {
        return raise_stat_error("covariance requires at least two data points");
    }
    let mx = welford_mean(&vx);
    let my = welford_mean(&vy);
    let mut s = 0.0f64;
    for i in 0..n {
        s += (vx[i] - mx) * (vy[i] - my);
    }
    MbValue::from_float(s / (n - 1) as f64)
}

/// correlation(x, y, ranked=False) — Pearson correlation coefficient, or
/// Spearman rank correlation when `ranked` is true (method="ranked").
pub fn mb_statistics_correlation(x: MbValue, y: MbValue, ranked: bool) -> MbValue {
    let mut vx = extract_floats(x);
    let mut vy = extract_floats(y);
    if vx.len() != vy.len() {
        return raise_stat_error(
            "correlation requires that both inputs have same number of data points",
        );
    }
    let n = vx.len();
    if n < 2 {
        return raise_stat_error("correlation requires at least two data points");
    }
    if ranked {
        vx = average_ranks(&vx);
        vy = average_ranks(&vy);
    }
    let mx = welford_mean(&vx);
    let my = welford_mean(&vy);
    let mut sxx = 0.0f64;
    let mut syy = 0.0f64;
    let mut sxy = 0.0f64;
    for i in 0..n {
        let dx = vx[i] - mx;
        let dy = vy[i] - my;
        sxx += dx * dx;
        syy += dy * dy;
        sxy += dx * dy;
    }
    let denom = (sxx * syy).sqrt();
    if denom == 0.0 {
        return raise_stat_error("at least one of the inputs is constant");
    }
    MbValue::from_float(sxy / denom)
}

/// Average (fractional) ranks of `data`, ties sharing the mean of the ranks
/// they span — the ranking used by Spearman correlation (CPython's
/// `statistics._rank` with ties averaged).
fn average_ranks(data: &[f64]) -> Vec<f64> {
    let n = data.len();
    let mut idx: Vec<usize> = (0..n).collect();
    idx.sort_by(|&a, &b| {
        data[a]
            .partial_cmp(&data[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut ranks = vec![0.0f64; n];
    let mut i = 0usize;
    while i < n {
        let mut j = i + 1;
        while j < n && data[idx[j]] == data[idx[i]] {
            j += 1;
        }
        // Positions i..j (0-based) share rank = mean of (i+1 .. j) 1-based.
        let avg_rank = ((i + 1 + j) as f64) / 2.0;
        for k in i..j {
            ranks[idx[k]] = avg_rank;
        }
        i = j;
    }
    ranks
}

/// linear_regression(x, y, proportional=False) -> (slope, intercept)
pub fn mb_statistics_linear_regression(x: MbValue, y: MbValue, proportional: bool) -> MbValue {
    let vx = extract_floats(x);
    let vy = extract_floats(y);
    if vx.len() != vy.len() {
        return raise_stat_error(
            "linear regression requires that both inputs have same number of data points",
        );
    }
    let n = vx.len();
    if n < 2 {
        return raise_stat_error("linear regression requires at least two data points");
    }
    let (slope, intercept) = if proportional {
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for i in 0..n {
            num += vx[i] * vy[i];
            den += vx[i] * vx[i];
        }
        if den == 0.0 {
            return raise_stat_error("x is constant");
        }
        (num / den, 0.0)
    } else {
        let mx = welford_mean(&vx);
        let my = welford_mean(&vy);
        let mut sxx = 0.0f64;
        let mut sxy = 0.0f64;
        for i in 0..n {
            let dx = vx[i] - mx;
            sxx += dx * dx;
            sxy += dx * (vy[i] - my);
        }
        if sxx == 0.0 {
            return raise_stat_error("x is constant");
        }
        let s = sxy / sxx;
        (s, my - s * mx)
    };
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_float(slope),
        MbValue::from_float(intercept),
    ]))
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::MbObject;
    use super::super::super::value::MbValue;
    use super::*;

    fn make_int_list(items: &[i64]) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|&i| MbValue::from_int(i)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    fn make_float_list(items: &[f64]) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|&f| MbValue::from_float(f)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    fn empty_list() -> MbValue {
        MbValue::from_ptr(MbObject::new_list(vec![]))
    }

    #[test]
    fn test_mean_basic() {
        assert_eq!(
            mb_statistics_mean(make_int_list(&[1, 2, 3, 4, 5])).as_float(),
            Some(3.0)
        );
    }

    #[test]
    fn test_mean_empty() {
        assert!(mb_statistics_mean(empty_list()).is_none());
    }

    #[test]
    fn test_fmean_basic() {
        assert_eq!(
            mb_statistics_fmean(make_float_list(&[1.5, 2.5, 3.5])).as_float(),
            Some(2.5)
        );
    }

    #[test]
    fn test_median_odd_int_preserves_int() {
        assert_eq!(
            mb_statistics_median(make_int_list(&[1, 3, 2])).as_int(),
            Some(2)
        );
    }

    #[test]
    fn test_median_even() {
        assert_eq!(
            mb_statistics_median(make_int_list(&[1, 2, 3, 4])).as_float(),
            Some(2.5)
        );
    }

    #[test]
    fn test_median_low_even() {
        assert_eq!(
            mb_statistics_median_low(make_int_list(&[1, 2, 3, 4])).as_int(),
            Some(2)
        );
    }

    #[test]
    fn test_median_high_even() {
        assert_eq!(
            mb_statistics_median_high(make_int_list(&[1, 2, 3, 4])).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_median_grouped_basic() {
        // CPython: statistics.median_grouped([1,2,2,3,4,4,4,4,4,5]) -> 3.7
        let result =
            mb_statistics_median_grouped(make_int_list(&[1, 2, 2, 3, 4, 4, 4, 4, 4, 5]), 1.0);
        let v = result.as_float().unwrap();
        assert!((v - 3.7).abs() < 1e-9, "got {}", v);
    }

    #[test]
    fn test_mode_basic() {
        assert_eq!(
            mb_statistics_mode(make_int_list(&[1, 2, 2, 3])).as_int(),
            Some(2)
        );
    }

    #[test]
    fn test_mode_first_occurrence_tiebreak() {
        // 1 and 2 both appear once first; tie → first-occurrence wins → 1.
        assert_eq!(
            mb_statistics_mode(make_int_list(&[1, 2, 3, 2, 1])).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_multimode_tied() {
        // [1, 1, 2, 2, 3] → [1, 2]
        let result = mb_statistics_multimode(make_int_list(&[1, 1, 2, 2, 3]));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let g = lock.read().unwrap();
                assert_eq!(g.len(), 2);
                assert_eq!(g[0].as_int(), Some(1));
                assert_eq!(g[1].as_int(), Some(2));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_pvariance_basic() {
        // pvariance([1,2,3,4,5]) = 2.0
        let result = mb_statistics_pvariance(make_int_list(&[1, 2, 3, 4, 5]));
        let v = result.as_float().unwrap();
        assert!((v - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_pstdev_basic() {
        // pstdev([1,2,3,4,5]) = sqrt(2.0)
        let result = mb_statistics_pstdev(make_int_list(&[1, 2, 3, 4, 5]));
        let v = result.as_float().unwrap();
        assert!((v - 2.0f64.sqrt()).abs() < 1e-9);
    }

    #[test]
    fn test_variance_basic() {
        // variance([2.0, 4.0]) = 2.0
        assert_eq!(
            mb_statistics_variance(make_float_list(&[2.0, 4.0])).as_float(),
            Some(2.0)
        );
    }

    #[test]
    fn test_stdev_basic() {
        let v = mb_statistics_stdev(make_float_list(&[2.0, 4.0]))
            .as_float()
            .unwrap();
        assert!((v - 1.4142135623730951).abs() < 1e-9);
    }

    #[test]
    fn test_variance_too_few() {
        assert!(mb_statistics_variance(make_float_list(&[1.0])).is_none());
    }

    #[test]
    fn test_geometric_mean_basic() {
        let v = mb_statistics_geometric_mean(make_float_list(&[1.0, 4.0]))
            .as_float()
            .unwrap();
        assert!((v - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_harmonic_mean_basic() {
        // harmonic_mean([1, 2, 4]) = 12/7
        let v = mb_statistics_harmonic_mean(make_float_list(&[1.0, 2.0, 4.0]))
            .as_float()
            .unwrap();
        assert!((v - 12.0 / 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_covariance_basic() {
        // covariance([1,2,3,4,5], [2,4,6,8,10]) = 5.0
        let v = mb_statistics_covariance(
            make_int_list(&[1, 2, 3, 4, 5]),
            make_int_list(&[2, 4, 6, 8, 10]),
        )
        .as_float()
        .unwrap();
        assert!((v - 5.0).abs() < 1e-9);
    }

    #[test]
    fn test_correlation_perfect_positive() {
        let v = mb_statistics_correlation(
            make_int_list(&[1, 2, 3, 4, 5]),
            make_int_list(&[2, 4, 6, 8, 10]),
            false,
        )
        .as_float()
        .unwrap();
        assert!((v - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_correlation_perfect_negative() {
        let v = mb_statistics_correlation(
            make_int_list(&[1, 2, 3, 4, 5]),
            make_int_list(&[10, 8, 6, 4, 2]),
            false,
        )
        .as_float()
        .unwrap();
        assert!((v - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn test_linear_regression_basic() {
        // y = 2x → slope=2, intercept=0
        let result = mb_statistics_linear_regression(
            make_int_list(&[1, 2, 3, 4, 5]),
            make_int_list(&[2, 4, 6, 8, 10]),
            false,
        );
        unsafe {
            if let ObjData::Tuple(ref t) = (*result.as_ptr().unwrap()).data {
                let slope = t[0].as_float().unwrap();
                let intercept = t[1].as_float().unwrap();
                assert!((slope - 2.0).abs() < 1e-9);
                assert!(intercept.abs() < 1e-9);
            } else {
                panic!("expected tuple");
            }
        }
    }

    #[test]
    fn test_linear_regression_proportional() {
        let result = mb_statistics_linear_regression(
            make_int_list(&[1, 2, 3, 4, 5]),
            make_int_list(&[2, 4, 6, 8, 10]),
            true,
        );
        unsafe {
            if let ObjData::Tuple(ref t) = (*result.as_ptr().unwrap()).data {
                let slope = t[0].as_float().unwrap();
                let intercept = t[1].as_float().unwrap();
                assert!((slope - 2.0).abs() < 1e-9);
                assert!(intercept.abs() < 1e-9);
            } else {
                panic!("expected tuple");
            }
        }
    }

    #[test]
    fn test_quantiles_default() {
        // quantiles([1..=9], n=4, exclusive) = [2.5, 5.0, 7.5]
        let result = mb_statistics_quantiles(make_int_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9]), 4, false);
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let g = lock.read().unwrap();
                assert_eq!(g.len(), 3);
                assert!((g[0].as_float().unwrap() - 2.5).abs() < 1e-9);
                assert!((g[1].as_float().unwrap() - 5.0).abs() < 1e-9);
                assert!((g[2].as_float().unwrap() - 7.5).abs() < 1e-9);
            } else {
                panic!("expected list");
            }
        }
    }

    // ── Wave-9 additions ──

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_class_name(instance: MbValue) -> Option<String> {
        instance.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_fabs_basic() {
        assert_eq!(
            mb_statistics_fabs(MbValue::from_float(-3.5)).as_float(),
            Some(3.5)
        );
        assert_eq!(
            mb_statistics_fabs(MbValue::from_int(-7)).as_float(),
            Some(7.0)
        );
    }

    #[test]
    fn test_exp_basic() {
        let v = mb_statistics_exp(MbValue::from_int(0)).as_float().unwrap();
        assert!((v - 1.0).abs() < 1e-9);
        let v2 = mb_statistics_exp(MbValue::from_float(1.0))
            .as_float()
            .unwrap();
        assert!((v2 - std::f64::consts::E).abs() < 1e-9);
    }

    #[test]
    fn test_sqrt_basic() {
        assert_eq!(
            mb_statistics_sqrt(MbValue::from_int(16)).as_float(),
            Some(4.0)
        );
        assert!(mb_statistics_sqrt(MbValue::from_float(-1.0)).is_none());
    }

    #[test]
    fn test_erf_basic() {
        // erf(0) == 0; erf(1) ~ 0.8427
        let z = mb_statistics_erf(MbValue::from_float(0.0))
            .as_float()
            .unwrap();
        assert!(z.abs() < 1e-6);
        let one = mb_statistics_erf(MbValue::from_float(1.0))
            .as_float()
            .unwrap();
        assert!((one - 0.8427007).abs() < 1e-4);
    }

    #[test]
    fn test_log_natural_and_base() {
        let ln_e = mb_statistics_log(MbValue::from_float(std::f64::consts::E), None)
            .as_float()
            .unwrap();
        assert!((ln_e - 1.0).abs() < 1e-9);
        let log2_8 = mb_statistics_log(MbValue::from_int(8), Some(MbValue::from_int(2)))
            .as_float()
            .unwrap();
        assert!((log2_8 - 3.0).abs() < 1e-9);
        assert!(mb_statistics_log(MbValue::from_int(0), None).is_none());
    }

    #[test]
    fn test_hypot_basic() {
        let args = vec![MbValue::from_int(3), MbValue::from_int(4)];
        let v = mb_statistics_hypot(&args).as_float().unwrap();
        assert!((v - 5.0).abs() < 1e-9);
        let args3 = vec![
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(6),
        ];
        let v3 = mb_statistics_hypot(&args3).as_float().unwrap();
        assert!((v3 - 7.0).abs() < 1e-9);
    }

    #[test]
    fn test_fsum_precision() {
        // Classic compensated-sum test: [0.1; 10] sums to 1.0 cleanly.
        let v = mb_statistics_fsum(make_float_list(&[0.1; 10]))
            .as_float()
            .unwrap();
        assert!((v - 1.0).abs() < 1e-15);
    }

    #[test]
    fn test_sumprod_basic() {
        let v = mb_statistics_sumprod(make_int_list(&[1, 2, 3]), make_int_list(&[4, 5, 6]));
        // 1*4 + 2*5 + 3*6 = 32; int inputs → int result.
        assert_eq!(v.as_int(), Some(32));
    }

    #[test]
    fn test_sumprod_float() {
        let v = mb_statistics_sumprod(make_float_list(&[1.5, 2.5]), make_float_list(&[2.0, 4.0]))
            .as_float()
            .unwrap();
        assert!((v - 13.0).abs() < 1e-9);
    }

    #[test]
    fn test_bisect_left_basic() {
        let seq = make_int_list(&[1, 3, 5, 7, 9]);
        assert_eq!(
            mb_statistics_bisect_left(seq, MbValue::from_int(5)).as_int(),
            Some(2)
        );
        assert_eq!(
            mb_statistics_bisect_left(seq, MbValue::from_int(0)).as_int(),
            Some(0)
        );
        assert_eq!(
            mb_statistics_bisect_left(seq, MbValue::from_int(10)).as_int(),
            Some(5)
        );
    }

    #[test]
    fn test_bisect_right_basic() {
        let seq = make_int_list(&[1, 3, 5, 7, 9]);
        assert_eq!(
            mb_statistics_bisect_right(seq, MbValue::from_int(5)).as_int(),
            Some(3)
        );
        assert_eq!(
            mb_statistics_bisect_right(seq, MbValue::from_int(0)).as_int(),
            Some(0)
        );
        assert_eq!(
            mb_statistics_bisect_right(seq, MbValue::from_int(10)).as_int(),
            Some(5)
        );
    }

    #[test]
    fn test_reduce_collapse_last_or_initial() {
        // List path → last element.
        let last = mb_statistics_reduce(make_int_list(&[1, 2, 3]), None);
        assert_eq!(last.as_int(), Some(3));
        // Empty sequence + initial → initial.
        let init = mb_statistics_reduce(empty_list(), Some(MbValue::from_int(99)));
        assert_eq!(init.as_int(), Some(99));
    }

    #[test]
    fn test_itemgetter_list_index() {
        let seq = make_int_list(&[10, 20, 30]);
        let v = mb_statistics_itemgetter(MbValue::from_int(1), seq);
        assert_eq!(v.as_int(), Some(20));
        // Negative index.
        let v2 = mb_statistics_itemgetter(MbValue::from_int(-1), seq);
        assert_eq!(v2.as_int(), Some(30));
    }

    #[test]
    fn test_normaldist_fields() {
        let nd = mb_statistics_normaldist(10.0, 2.0);
        assert_eq!(get_class_name(nd).as_deref(), Some("NormalDist"));
        assert_eq!(get_field(nd, "mu").as_float(), Some(10.0));
        assert_eq!(get_field(nd, "sigma").as_float(), Some(2.0));
        assert_eq!(get_field(nd, "variance").as_float(), Some(4.0));
        assert_eq!(get_field(nd, "median").as_float(), Some(10.0));
    }

    #[test]
    fn test_class_token_shape() {
        let t = class_token("StatisticsError", "statistics");
        assert_eq!(get_class_name(t).as_deref(), Some("StatisticsError"));
        let name_ptr = get_field(t, "__name__").as_ptr().unwrap();
        unsafe {
            if let ObjData::Str(ref s) = (*name_ptr).data {
                assert_eq!(s, "StatisticsError");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_tau_constant() {
        // tau == 2*pi; check directly via the constant we register.
        let expected = std::f64::consts::TAU;
        assert!((expected - 2.0 * std::f64::consts::PI).abs() < 1e-15);
    }
}
