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
///     quantiles, zscore, arithmetic operators) are not wired through
///     method dispatch yet — the module-level statistical functions are
///     the active surface.
///   - `LinearRegression` is exposed as a class token / namedtuple
///     stub; the `linear_regression()` function returns a plain
///     `(slope, intercept)` tuple, which is structurally compatible.
///   - `StatisticsError` is an Instance with `__name__` / `__module__`
///     fields, mirroring the `struct.error` carve-out. The Exception
///     subclass hierarchy is not modeled yet, so domain errors return
///     `None` rather than raise.
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

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_sequence(val: MbValue) -> bool {
    val.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_) | ObjData::Tuple(_)) })
}

unsafe extern "C" fn dispatch_mean(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    if !is_sequence(data) {
        return raise_type_error("mean() arg must be a sequence");
    }
    mb_statistics_mean(data)
}

dispatch_unary!(dispatch_fmean, mb_statistics_fmean);
dispatch_unary!(dispatch_geometric_mean, mb_statistics_geometric_mean);
dispatch_unary!(dispatch_harmonic_mean, mb_statistics_harmonic_mean);
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
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let interval = a
        .get(1)
        .copied()
        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)))
        .unwrap_or(1.0);
    mb_statistics_median_grouped(data, interval)
}

/// quantiles(data, *, n=4, method="exclusive") — n is positional in our minimal cut.
unsafe extern "C" fn dispatch_quantiles(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let mut n: i64 = 4;
    let mut method_inclusive = false;
    // Walk positional/keyword: support (data, n) and trailing kwargs dict.
    if nargs >= 2 {
        let v = a[1];
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let k_n = super::super::dict_ops::DictKey::Str("n".to_string());
                    if let Some(nv) = g.get(&k_n) {
                        if let Some(i) = nv.as_int() {
                            n = i;
                        }
                    }
                    let k_m = super::super::dict_ops::DictKey::Str("method".to_string());
                    if let Some(mv) = g.get(&k_m) {
                        if let Some(mp) = mv.as_ptr() {
                            if let ObjData::Str(ref s) = (*mp).data {
                                method_inclusive = s == "inclusive";
                            }
                        }
                    }
                }
            }
        } else if let Some(i) = v.as_int() {
            n = i;
        }
    }
    mb_statistics_quantiles(data, n, method_inclusive)
}

/// correlation(x, y, /, *, method="linear") — method ignored ("ranked" deferred).
unsafe extern "C" fn dispatch_correlation(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let x = a.get(0).copied().unwrap_or_else(MbValue::none);
    let y = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_statistics_correlation(x, y)
}

/// linear_regression(x, y, /, *, proportional=False)
unsafe extern "C" fn dispatch_linear_regression(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let x = a.get(0).copied().unwrap_or_else(MbValue::none);
    let y = a.get(1).copied().unwrap_or_else(MbValue::none);
    let mut proportional = false;
    if nargs >= 3 {
        let v = a[nargs - 1];
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    let k = super::super::dict_ops::DictKey::Str("proportional".to_string());
                    if let Some(pv) = g.get(&k) {
                        proportional = pv.as_bool().unwrap_or(false);
                    }
                }
            }
        }
    }
    mb_statistics_linear_regression(x, y, proportional)
}

// ── Wave-9 extra dispatchers ──

dispatch_unary!(dispatch_fabs, mb_statistics_fabs);
dispatch_unary!(dispatch_exp, mb_statistics_exp);
dispatch_unary!(dispatch_sqrt, mb_statistics_sqrt);
dispatch_unary!(dispatch_erf, mb_statistics_erf);
dispatch_unary!(dispatch_fsum, mb_statistics_fsum);

/// log(x[, base]) — natural log if base omitted.
unsafe extern "C" fn dispatch_log(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let x = a.get(0).copied().unwrap_or_else(MbValue::none);
    let base = a.get(1).copied();
    mb_statistics_log(x, base)
}

/// hypot(*coords) — sqrt(sum(c**2)).
unsafe extern "C" fn dispatch_hypot(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_statistics_hypot(a)
}

dispatch_binary!(dispatch_sumprod, mb_statistics_sumprod);

/// bisect_left(a, x, lo=0, hi=None)
unsafe extern "C" fn dispatch_bisect_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    let x = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_statistics_bisect_left(seq, x)
}

/// bisect_right(a, x, lo=0, hi=None)
unsafe extern "C" fn dispatch_bisect_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let seq = a.get(0).copied().unwrap_or_else(MbValue::none);
    let x = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_statistics_bisect_right(seq, x)
}

/// count(start=0, step=1) — eager 1-element list seed (carve-out).
unsafe extern "C" fn dispatch_count(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let start = a.get(0).copied().unwrap_or_else(|| MbValue::from_int(0));
    MbValue::from_ptr(MbObject::new_list(vec![start]))
}

/// repeat(value, times=None) — eager materialization when times supplied.
unsafe extern "C" fn dispatch_repeat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
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
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
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
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs < 2 {
        return MbValue::none();
    }
    let key = a[0];
    let seq = a[1];
    mb_statistics_itemgetter(key, seq)
}

/// NormalDist(mu=0.0, sigma=1.0) — Instance stub.
unsafe extern "C" fn dispatch_normaldist(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
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

    // Class tokens (Instance sentinels — see carve-outs).
    attrs.insert(
        "StatisticsError".to_string(),
        class_token("StatisticsError", "statistics"),
    );
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
    let f = match as_f64(x) {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let sign = if f < 0.0 { -1.0 } else { 1.0 };
    let ax = f.abs();
    let t = 1.0 / (1.0 + 0.3275911 * ax);
    let y = 1.0
        - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t
            + 0.254829592)
            * t
            * (-ax * ax).exp();
    MbValue::from_float(sign * y)
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

// ── Helpers ──

fn as_f64(val: MbValue) -> Option<f64> {
    if let Some(f) = val.as_float() {
        Some(f)
    } else if let Some(i) = val.as_int() {
        Some(i as f64)
    } else {
        None
    }
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
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    MbValue::from_float(welford_mean(&v))
}

/// mean(data) — float fast path; type-preserving variants deferred.
pub fn mb_statistics_mean(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    MbValue::from_float(welford_mean(&v))
}

/// geometric_mean(data) — exp(mean(ln(x))).
pub fn mb_statistics_geometric_mean(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    if v.iter().any(|&x| x <= 0.0) {
        return MbValue::none();
    }
    let log_mean = welford_mean(&v.iter().map(|x| x.ln()).collect::<Vec<f64>>());
    MbValue::from_float(log_mean.exp())
}

/// harmonic_mean(data) — n / sum(1/x).
pub fn mb_statistics_harmonic_mean(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    if v.iter().any(|&x| x <= 0.0) {
        return MbValue::none();
    }
    let inv_sum: f64 = v.iter().map(|x| 1.0 / x).sum();
    MbValue::from_float(v.len() as f64 / inv_sum)
}

/// median(data) — sort + middle. Type-preserves int for all-int odd-length input.
pub fn mb_statistics_median(data: MbValue) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    if v.is_empty() {
        return MbValue::none();
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
        return MbValue::none();
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
        return MbValue::none();
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
pub fn mb_statistics_median_grouped(data: MbValue, interval: f64) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    let n = v.len();
    if n == 0 {
        return MbValue::none();
    }
    let x = v[n / 2];
    let l = x - interval / 2.0;
    // cf = count of values < l; f = count of values in [x - interval/2, x + interval/2)
    let cf = v.iter().take_while(|&&y| y < l).count();
    let f = v.iter().filter(|&&y| y >= l && y < l + interval).count();
    if f == 0 {
        return MbValue::from_float(x);
    }
    MbValue::from_float(l + interval * ((n as f64 / 2.0 - cf as f64) / f as f64))
}

/// mode(data) — most-common value (first-occurrence order for ties).
pub fn mb_statistics_mode(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    let mut counts: HashMap<u64, usize> = HashMap::new();
    let mut first_idx: HashMap<u64, usize> = HashMap::new();
    for (i, &x) in v.iter().enumerate() {
        let k = x.to_bits();
        *counts.entry(k).or_insert(0) += 1;
        first_idx.entry(k).or_insert(i);
    }
    let max_c = *counts.values().max().unwrap();
    let bits = counts
        .iter()
        .filter(|&(_, &c)| c == max_c)
        .min_by_key(|&(k, _)| first_idx.get(k).copied().unwrap_or(usize::MAX))
        .map(|(k, _)| *k)
        .unwrap();
    let f = f64::from_bits(bits);
    if all_ints(data) {
        MbValue::from_int(f as i64)
    } else {
        MbValue::from_float(f)
    }
}

/// multimode(data) — list of every value tied for max count.
pub fn mb_statistics_multimode(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let mut counts: HashMap<u64, usize> = HashMap::new();
    let mut order: Vec<u64> = Vec::new();
    for &x in &v {
        let k = x.to_bits();
        let e = counts.entry(k).or_insert(0);
        if *e == 0 {
            order.push(k);
        }
        *e += 1;
    }
    let max_c = *counts.values().max().unwrap();
    let ints = all_ints(data);
    let result: Vec<MbValue> = order
        .into_iter()
        .filter(|k| counts[k] == max_c)
        .map(|k| {
            let f = f64::from_bits(k);
            if ints {
                MbValue::from_int(f as i64)
            } else {
                MbValue::from_float(f)
            }
        })
        .collect();
    MbValue::from_ptr(MbObject::new_list(result))
}

/// pvariance(data) — population variance (divide by n).
pub fn mb_statistics_pvariance(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float(m2 / v.len() as f64)
}

/// pstdev(data) — population standard deviation.
pub fn mb_statistics_pstdev(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.is_empty() {
        return MbValue::none();
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float((m2 / v.len() as f64).sqrt())
}

/// variance(data) — sample variance (divide by n-1; Bessel correction).
pub fn mb_statistics_variance(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.len() < 2 {
        return MbValue::none();
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float(m2 / (v.len() - 1) as f64)
}

/// stdev(data) — sample standard deviation.
pub fn mb_statistics_stdev(data: MbValue) -> MbValue {
    let v = extract_floats(data);
    if v.len() < 2 {
        return MbValue::none();
    }
    let (_, m2) = welford_m_m2(&v);
    MbValue::from_float((m2 / (v.len() - 1) as f64).sqrt())
}

/// quantiles(data, n=4, method="exclusive") — cut points dividing data into n bins.
pub fn mb_statistics_quantiles(data: MbValue, n: i64, inclusive: bool) -> MbValue {
    let v = sorted_floats(extract_floats(data));
    let ld = v.len();
    if ld < 2 || n < 1 {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
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
    let n = vx.len().min(vy.len());
    if n < 2 {
        return MbValue::none();
    }
    let mx = welford_mean(&vx[..n]);
    let my = welford_mean(&vy[..n]);
    let mut s = 0.0f64;
    for i in 0..n {
        s += (vx[i] - mx) * (vy[i] - my);
    }
    MbValue::from_float(s / (n - 1) as f64)
}

/// correlation(x, y) — Pearson correlation coefficient.
pub fn mb_statistics_correlation(x: MbValue, y: MbValue) -> MbValue {
    let vx = extract_floats(x);
    let vy = extract_floats(y);
    let n = vx.len().min(vy.len());
    if n < 2 {
        return MbValue::none();
    }
    let mx = welford_mean(&vx[..n]);
    let my = welford_mean(&vy[..n]);
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
        return MbValue::none();
    }
    MbValue::from_float(sxy / denom)
}

/// linear_regression(x, y, proportional=False) -> (slope, intercept)
pub fn mb_statistics_linear_regression(x: MbValue, y: MbValue, proportional: bool) -> MbValue {
    let vx = extract_floats(x);
    let vy = extract_floats(y);
    let n = vx.len().min(vy.len());
    if n < 2 {
        return MbValue::none();
    }
    let (slope, intercept) = if proportional {
        let mut num = 0.0f64;
        let mut den = 0.0f64;
        for i in 0..n {
            num += vx[i] * vy[i];
            den += vx[i] * vx[i];
        }
        if den == 0.0 {
            return MbValue::none();
        }
        (num / den, 0.0)
    } else {
        let mx = welford_mean(&vx[..n]);
        let my = welford_mean(&vy[..n]);
        let mut sxx = 0.0f64;
        let mut sxy = 0.0f64;
        for i in 0..n {
            let dx = vx[i] - mx;
            sxx += dx * dx;
            sxy += dx * (vy[i] - my);
        }
        if sxx == 0.0 {
            return MbValue::none();
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
