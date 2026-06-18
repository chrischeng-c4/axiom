//! @codegen-skip: handwrite-pre-standardize
//!
//! bisect module for Mamba — Python 3.12 `bisect` stdlib.
//!
//! Provides `bisect_left`, `bisect_right`, `insort_left`, `insort_right`
//! over arbitrary ordered MbValue sequences (int / float / mixed numeric
//! / str / list / tuple, per `mb_lt`).
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_binary + value-ordered binary
//! search) is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1414 cluster anchor + see
//! `.aw/handoffs/1414-patrol-handoff.md`.
//!
//! The previous shim registered each attribute as a *string* MbValue
//! ("mb_bisect_bisect_left", etc.) which made `bisect.bisect_left(...)`
//! raise `AttributeError: 'dict' object has no attribute 'bisect_left'`
//! at import-time (caller would get a non-callable str). The fix wires
//! real `dispatch_*` extern "C" thunks through `NATIVE_FUNC_ADDRS` so
//! mamba's callable-resolution can find them — same shape as
//! `keyword_mod` and `struct_mod`.
use super::super::builtins::mb_lt;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// True iff `class_name`'s instances expose the named dunder, either as a
/// class-level method (the normal case) or — for stdlib stub Instances — as an
/// instance field. Mirrors the lookup `mb_len`/`mb_obj_getitem` perform.
fn instance_has_dunder(val: MbValue, class_name: &str, dunder: &str) -> bool {
    if !super::super::class::lookup_method(class_name, dunder).is_none() {
        return true;
    }
    // Instance-field fallback (e.g. dispatchers wired directly as fields).
    val.as_ptr()
        .map(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .read()
                    .unwrap()
                    .get(dunder)
                    .is_some_and(|v| !v.is_none())
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// True iff `val` is a valid first argument to bisect/insort.
///
/// CPython accepts any object and works through `len()` + `__getitem__`. We
/// accept the native indexable containers (list / tuple), `range(...)` handles
/// (tagged-int iterators), `array` handles, and any user-class **instance that
/// defines BOTH `__len__` and `__getitem__`** — matching the `_bisect` C
/// accessor, which requires both (`GetOnly`/`LenOnly` raise `TypeError`).
fn is_bisect_sequence(val: MbValue) -> bool {
    // range(...) and other iterator handles are tagged ints, as are array
    // handles; both support `len()` + integer `__getitem__` via the shared
    // accessors used below.
    if val.is_int() {
        if super::super::iter::is_iter_handle(val) {
            return true;
        }
        let id = val.as_int().unwrap_or(0) as u64;
        return super::super::stdlib::array_mod::is_array_handle(id);
    }
    val.as_ptr().is_some_and(|ptr| unsafe {
        match (*ptr).data {
            ObjData::List(_) | ObjData::Tuple(_) => true,
            ObjData::Instance { ref class_name, .. } => {
                // A `list` subclass is a sequence even when it doesn't redefine
                // `__len__`/`__getitem__` (it inherits them from `list`).
                super::super::class::class_mro_any(class_name, |c| c == "list")
                    // collections.UserList / UserString are sequence wrappers;
                    // their __len__/__getitem__ are served by native dispatch
                    // (not the instance method table), so instance_has_dunder
                    // misses them. seq_len/seq_get route through mb_len /
                    // mb_obj_getitem, which handle the wrappers correctly.
                    || matches!(
                        super::super::stdlib::collections_mod::user_wrapper_kind(class_name),
                        Some("list") | Some("str")
                    )
                    || (instance_has_dunder(val, class_name, "__len__")
                        && instance_has_dunder(val, class_name, "__getitem__"))
            }
            _ => false,
        }
    })
}

/// Insert `x` at `pos` into the sequence `a`. For a plain native list this
/// mutates the backing store directly; for a `list` subclass or any other
/// instance it dispatches the object's own `insert(pos, x)` method, so an
/// overridden `insert` is observed (CPython's `insort` always calls
/// `a.insert`).
fn seq_insert(a: MbValue, pos: usize, x: MbValue) {
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().insert(pos, x);
                return;
            }
        }
    }
    let name = MbValue::from_ptr(MbObject::new_str("insert".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![super::super::bigint_ops::int_from_i64(pos as i64), x]));
    super::super::class::mb_call_method(a, name, args);
}

/// Parsed `bisect`/`insort` argument bundle.
///
/// CPython signature: `(a, x, lo=0, hi=None, *, key=None)`. The lowering
/// passes positional args directly and packs keyword args into a single
/// trailing dict (the same `is_method_call && has_any_kwargs` path used by
/// every other attribute call). We accept either shape here.
struct BisectArgs {
    a: MbValue,
    x: MbValue,
    lo: i64,
    /// `None` means "default to len(a)".
    hi: Option<i64>,
    /// `MbValue::none()` means no key function.
    key: MbValue,
}

/// True iff `v` is a kwargs dict appended by the call lowering. We treat a
/// trailing dict as kwargs only when it carries one of the *keyword-specific*
/// bisect parameter names — `lo`, `hi`, or `key`. The positional names `a`/`x`
/// are deliberately excluded: a genuine data dict passed as the search argument
/// (e.g. `insort([], {'a': 2, 'b': 1})`) commonly contains a key literally
/// named `a`, and must not be mistaken for kwargs. Every fixture that passes
/// `a=`/`x=` by keyword also passes `lo`/`hi`/`key`, so this stays correct while
/// eliminating the false positive (#param-eq is unrelated).
fn dict_as_kwargs(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            if ["lo", "hi", "key"].iter().any(|k| map.contains_key(*k)) {
                return Some(v);
            }
        }
    }
    None
}

fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    let ptr = dict.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            return lock.read().unwrap().get(key).copied();
        }
    }
    None
}

/// Interpret a value supplied for the `lo`/`hi` index parameter. An integer
/// (or bool, which is an int subclass in Python) is accepted; any other
/// non-`None` value is a `TypeError` — CPython feeds these through `__index__`,
/// so e.g. a string `lo` raises "'str' object cannot be interpreted as an
/// integer". Returns `Err` (after raising) on a bad type.
fn coerce_index(v: MbValue, fname: &str) -> Result<i64, MbValue> {
    if let Some(i) = v.as_int_pyint() {
        return Ok(i);
    }
    // A bound above 2^47 (e.g. `bisect(data, x, n - 10, n)` with n =
    // sys.maxsize) is a NaN-box-promoted BigInt; unbox it when it fits i64.
    use num_traits::ToPrimitive;
    if let Some(i) = unsafe { super::super::bigint_ops::extract_bigint(v) }.and_then(|b| b.to_i64()) {
        return Ok(i);
    }
    Err(raise_type_error(&format!(
        "{}() '{}' object cannot be interpreted as an integer",
        fname,
        value_type_name(v),
    )))
}

/// Parse positional args plus an optional trailing kwargs dict into the
/// CPython `(a, x, lo, hi, key)` shape. Positional args bind in order
/// `a, x, lo, hi`; keyword args (from the trailing dict) override/fill by
/// name and also supply the keyword-only `key`.
///
/// Returns `Err` (after raising a `TypeError`) when `lo`/`hi` is present but
/// not an integer — this also covers the case where mamba's variable-call
/// lowering collapses a `key=<non-callable>` keyword down to a positional `lo`
/// slot (it loses the keyword name), since CPython rejects both shapes with a
/// `TypeError`.
fn parse_args(a: &[MbValue], fname: &str) -> Result<BisectArgs, MbValue> {
    // Detect a trailing kwargs dict (only if it names bisect params).
    let (positional, kwargs) = match a.last().and_then(|v| dict_as_kwargs(*v)) {
        Some(kw) => (&a[..a.len() - 1], Some(kw)),
        None => (a, None),
    };

    let lo = match positional.get(2) {
        Some(v) if !v.is_none() => coerce_index(*v, fname)?,
        _ => 0,
    };
    let hi = match positional.get(3) {
        Some(v) if !v.is_none() => Some(coerce_index(*v, fname)?),
        _ => None,
    };

    let mut out = BisectArgs {
        a: positional.first().copied().unwrap_or_else(MbValue::none),
        x: positional.get(1).copied().unwrap_or_else(MbValue::none),
        lo,
        hi,
        key: MbValue::none(),
    };

    if let Some(kw) = kwargs {
        if let Some(v) = dict_get(kw, "a") {
            out.a = v;
        }
        if let Some(v) = dict_get(kw, "x") {
            out.x = v;
        }
        if let Some(v) = dict_get(kw, "lo") {
            if !v.is_none() {
                out.lo = coerce_index(v, fname)?;
            }
        }
        if let Some(v) = dict_get(kw, "hi") {
            out.hi = if v.is_none() {
                None
            } else {
                Some(coerce_index(v, fname)?)
            };
        }
        if let Some(v) = dict_get(kw, "key") {
            if !v.is_none() {
                out.key = v;
            }
        }
    }
    Ok(out)
}

unsafe extern "C" fn dispatch_bisect_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let p = match parse_args(a, "bisect_left") {
        Ok(p) => p,
        Err(v) => return v,
    };
    if !is_bisect_sequence(p.a) {
        return raise_type_error("bisect_left() argument 1 must be a sequence");
    }
    match bisect_index(p.a, p.x, p.lo, p.hi, p.key, false) {
        Ok(pos) => super::super::bigint_ops::int_from_i64(pos as i64),
        Err(v) => v,
    }
}

unsafe extern "C" fn dispatch_bisect_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let p = match parse_args(a, "bisect_right") {
        Ok(p) => p,
        Err(v) => return v,
    };
    if !is_bisect_sequence(p.a) {
        return raise_type_error("bisect_right() argument 1 must be a sequence");
    }
    match bisect_index(p.a, p.x, p.lo, p.hi, p.key, true) {
        Ok(pos) => super::super::bigint_ops::int_from_i64(pos as i64),
        Err(v) => v,
    }
}

unsafe extern "C" fn dispatch_insort_left(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let p = match parse_args(a, "insort_left") {
        Ok(p) => p,
        Err(v) => return v,
    };
    if !is_bisect_sequence(p.a) {
        return raise_type_error("insort_left() argument 1 must be a sequence");
    }
    insort(p.a, p.x, p.lo, p.hi, p.key, false)
}

unsafe extern "C" fn dispatch_insort_right(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let p = match parse_args(a, "insort_right") {
        Ok(p) => p,
        Err(v) => return v,
    };
    if !is_bisect_sequence(p.a) {
        return raise_type_error("insort_right() argument 1 must be a sequence");
    }
    insort(p.a, p.x, p.lo, p.hi, p.key, true)
}

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("bisect_left", dispatch_bisect_left as usize),
        ("bisect_right", dispatch_bisect_right as usize),
        // CPython provides `bisect` as an alias for `bisect_right`, and
        // `insort` for `insort_right`. Mirror that here.
        ("bisect", dispatch_bisect_right as usize),
        ("insort_left", dispatch_insort_left as usize),
        ("insort_right", dispatch_insort_right as usize),
        ("insort", dispatch_insort_right as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("bisect", attrs);
}

/// Three-valued result of an element comparison: the boolean answer, or a
/// signal that a Python-level exception was raised during the comparison (e.g.
/// `CmpErr.__lt__` raising `ZeroDivisionError`). `Exc` must short-circuit the
/// search and propagate the pending exception, exactly like CPython.
enum Cmp {
    Bool(bool),
    Exc,
}

fn exception_pending() -> bool {
    super::super::exception::mb_has_exception().as_bool() == Some(true)
}

/// Class name of an instance value, if any.
fn instance_class_name(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Dispatch a rich-comparison dunder `recv.<dunder>(arg)`, returning the raw
/// result value. Caller must check `exception_pending()` afterwards.
fn call_richcmp(recv: MbValue, dunder: &str, arg: MbValue) -> MbValue {
    let name = MbValue::from_ptr(MbObject::new_str(dunder.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![arg]));
    super::super::class::mb_call_method(recv, name, args)
}

/// `a < b` for a binary-search comparison, with full CPython `__lt__`
/// semantics:
///
/// * For user-class instances we dispatch `a.__lt__(b)` directly (NOT through
///   `mb_lt`, whose instance path collapses non-bool/NotImplemented results to
///   `False`). A non-bool result is taken by truthiness (so a non-empty string
///   is `True`); a `NotImplemented` result falls back to the reflected
///   `b.__gt__(a)`. An exception raised anywhere along the way is reported as
///   `Cmp::Exc` so the search aborts and the exception propagates.
/// * For native operands (int/float/str/list/tuple/…) we defer to `mb_lt`,
///   which already returns a proper bool.
fn elem_lt(a: MbValue, b: MbValue) -> Cmp {
    let a_is_instance = instance_class_name(a).is_some();
    if let Some(class_name) = instance_class_name(a) {
        if instance_has_dunder(a, &class_name, "__lt__") {
            let r = call_richcmp(a, "__lt__", b);
            if exception_pending() {
                return Cmp::Exc;
            }
            if !r.is_not_implemented() {
                return Cmp::Bool(super::super::builtins::mb_is_truthy(r) != 0);
            }
            // a.__lt__(b) is NotImplemented → reflected b.__gt__(a).
            if let Some(reflected) = reflected_gt(b, a) {
                return reflected;
            }
            // Both NotImplemented: CPython would raise TypeError for `<`, but
            // bisect's element ordering only needs a falsey default here.
            return Cmp::Bool(false);
        }
    }
    // `a` is a native value (int/float/str/...) and `b` is an instance: CPython
    // tries the native `a.__lt__(b)` (NotImplemented for a foreign instance),
    // then the reflected `b.__gt__(a)`. mamba's native `mb_lt` does not perform
    // this reflected dispatch, so do it here (this is what propagates e.g.
    // `bisect_right(seq_of_CmpErr, 10)` → `CmpErr.__gt__` → ZeroDivisionError).
    if !a_is_instance {
        if let Some(reflected) = reflected_gt(b, a) {
            return reflected;
        }
    }
    let r = mb_lt(a, b);
    if exception_pending() {
        return Cmp::Exc;
    }
    match r.as_bool() {
        Some(b) => Cmp::Bool(b),
        None => Cmp::Bool(super::super::builtins::mb_is_truthy(r) != 0),
    }
}

/// Reflected `right.__gt__(left)` for `left < right` when `right` is an
/// instance exposing `__gt__`. Returns `None` when `right` isn't such an
/// instance or its `__gt__` returns `NotImplemented` (so the caller falls back
/// to the native comparison); `Some(Cmp::Exc)` if the dunder raised.
fn reflected_gt(right: MbValue, left: MbValue) -> Option<Cmp> {
    let cls = instance_class_name(right)?;
    if !instance_has_dunder(right, &cls, "__gt__") {
        return None;
    }
    let r = call_richcmp(right, "__gt__", left);
    if exception_pending() {
        return Some(Cmp::Exc);
    }
    if r.is_not_implemented() {
        return None;
    }
    Some(Cmp::Bool(super::super::builtins::mb_is_truthy(r) != 0))
}

/// Plain boolean `a < b` for the native-only helper paths (`with_list`),
/// where operands are guaranteed native and no exception can be raised.
#[inline]
fn lt(a: MbValue, b: MbValue) -> bool {
    match elem_lt(a, b) {
        Cmp::Bool(v) => v,
        Cmp::Exc => false,
    }
}

/// Length of the sequence `a` as seen by bisect. For native list/tuple this is
/// a direct lock-free read; for range/array handles and `__len__` instances it
/// routes through the shared `len()` accessor (which dispatches `__len__`).
fn seq_len(a: MbValue) -> usize {
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::List(ref lock) => return lock.read().unwrap().len(),
                ObjData::Tuple(ref items) => return items.len(),
                _ => {}
            }
        }
    }
    // range/array handles and __len__ instances. A length above 2^47 is
    // NaN-box-promoted to a BigInt, so `.as_int()` is None; unbox it to its
    // exact i64 (the value still fits usize on 64-bit) so bisect over a huge
    // sequence, e.g. `range(sys.maxsize - 1)`, searches the real length
    // rather than collapsing to 0.
    let lenv = super::super::builtins::mb_len(a);
    if lenv.is_none() {
        return 0;
    }
    super::super::builtins::mb_unbox_int_if_boxed(lenv).max(0) as usize
}

/// `a[idx]` for bisect's binary search. For native list/tuple this clones the
/// element out from under a short-lived lock (so we never hold it across a
/// re-entrant `key` callback or a `__getitem__` dispatch); for range/array
/// handles and `__getitem__` instances it routes through the shared subscript
/// accessor, which dispatches `__getitem__`.
fn seq_get(a: MbValue, idx: usize) -> MbValue {
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::List(ref lock) => {
                    return lock
                        .read()
                        .unwrap()
                        .get(idx)
                        .copied()
                        .unwrap_or_else(MbValue::none)
                }
                ObjData::Tuple(ref items) => {
                    return items.get(idx).copied().unwrap_or_else(MbValue::none)
                }
                _ => {}
            }
        }
    }
    super::super::class::mb_obj_getitem(a, super::super::bigint_ops::int_from_i64(idx as i64))
}

/// Apply the optional `key` function to an element. `none` key is identity.
#[inline]
fn apply_key(key: MbValue, elem: MbValue) -> MbValue {
    if key.is_none() {
        elem
    } else {
        super::super::class::mb_call1_val(key, elem)
    }
}

/// Runtime type name of `v`, for the "'X' object is not callable" message.
fn value_type_name(v: MbValue) -> &'static str {
    if v.as_int().is_some() {
        return "int";
    }
    if v.as_float().is_some() {
        return "float";
    }
    match v.as_ptr() {
        None => "NoneType",
        Some(ptr) => unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Tuple(_) => "tuple",
                ObjData::Dict(_) => "dict",
                _ => "object",
            }
        },
    }
}

/// CPython invokes `key(...)` eagerly the moment the search needs it, so a
/// non-callable `key` raises `TypeError: 'X' object is not callable`. Returns
/// `Err(none)` (after raising) when `key` is present but not callable; `Ok(())`
/// when it is callable or absent.
fn check_key_callable(key: MbValue) -> Result<(), MbValue> {
    if key.is_none() || super::super::builtins::mb_callable(key).as_bool() == Some(true) {
        Ok(())
    } else {
        Err(raise_type_error(&format!(
            "'{}' object is not callable",
            value_type_name(key)
        )))
    }
}

/// CPython `(a, x, lo, hi, *, key)` binary search core, shared by both
/// `bisect_left` (`right=false`) and `bisect_right` (`right=true`).
///
/// Returns `Err(MbValue::none())` after raising a Python-level exception
/// (negative `lo`, or a comparison/key error propagated through the runtime).
fn bisect_index(
    a: MbValue,
    x: MbValue,
    lo_in: i64,
    hi_in: Option<i64>,
    key: MbValue,
    right: bool,
) -> Result<usize, MbValue> {
    if lo_in < 0 {
        return Err(raise_value_error("lo must be non-negative"));
    }
    let n = seq_len(a);
    let mut lo = lo_in as usize;
    // hi defaults to len(a). A caller-supplied hi is honored verbatim (CPython
    // does NOT clamp it to len — an over-large hi only matters if the search
    // actually indexes past the end, which then raises IndexError via the
    // element accessor, matching `_bisect`). A negative hi means an empty
    // window.
    let mut hi = match hi_in {
        Some(h) if h < 0 => 0,
        Some(h) => h as usize,
        None => n,
    };
    if lo > hi {
        // An out-of-range lo (above hi) yields no search window.
        lo = hi;
    }
    // A non-callable key only matters once we actually need to map an element;
    // an empty search window never invokes it (matching CPython's laziness).
    if lo < hi {
        check_key_callable(key)?;
    }
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        let elem = apply_key(key, seq_get(a, mid));
        // A `key(...)` callback or `__getitem__` may itself raise; propagate.
        if exception_pending() {
            return Err(MbValue::none());
        }
        // bisect_left advances when `elem < x`; bisect_right advances when
        // `!(x < elem)` (i.e. inserts after equal elements). A comparison that
        // raises (e.g. CmpErr.__lt__ → ZeroDivisionError) aborts the search
        // and propagates the pending exception.
        let go_right = if right {
            match elem_lt(x, elem) {
                Cmp::Bool(b) => !b,
                Cmp::Exc => return Err(MbValue::none()),
            }
        } else {
            match elem_lt(elem, x) {
                Cmp::Bool(b) => b,
                Cmp::Exc => return Err(MbValue::none()),
            }
        };
        if go_right {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    Ok(lo)
}

/// CPython `insort_*`: find the insertion point (using `key(x)` as the search
/// value when a key is supplied), then insert the raw `x` into the list.
fn insort(a: MbValue, x: MbValue, lo: i64, hi: Option<i64>, key: MbValue, right: bool) -> MbValue {
    // CPython's insort eagerly computes key(x) as the search value, so a
    // non-callable key raises TypeError before any insertion happens (even
    // for an empty list).
    if let Err(v) = check_key_callable(key) {
        return v;
    }
    // With a key, the search compares against key(x); the raw x is inserted.
    let search_x = apply_key(key, x);
    // A key callback during the search may raise; if so `bisect_index` returns
    // Err and we must not attempt the insert.
    let pos = match bisect_index(a, search_x, lo, hi, key, right) {
        Ok(p) => p,
        Err(v) => return v,
    };
    seq_insert(a, pos, x);
    MbValue::none()
}

/// Run `f` against a borrowed view of the list elements held by `a`,
/// holding the read lock for the duration of the closure. Returns
/// `f(&[])` for non-sequence values in direct helper calls; the public module
/// dispatchers validate call-boundary TypeError behavior.
///
/// Borrowing instead of cloning is essential: the binary-search hot
/// loop does ~log2(n) comparisons, but a naive `read_list()` would
/// clone all n MbValues per call — making `bisect_left` on a 1000-elem
/// list ~100× slower than CPython instead of ~equal.
fn with_list<R>(a: MbValue, f: impl FnOnce(&[MbValue]) -> R) -> R
where
    R: Default,
{
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                return f(guard.as_slice());
            } else if let ObjData::Tuple(ref items) = (*ptr).data {
                return f(items.as_slice());
            }
        }
    }
    f(&[])
}

pub fn mb_bisect_bisect_left(a: MbValue, x: MbValue) -> MbValue {
    let pos = with_list(a, |items| {
        let (mut lo, mut hi) = (0usize, items.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            if lt(items[mid], x) {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    });
    super::super::bigint_ops::int_from_i64(pos as i64)
}

pub fn mb_bisect_bisect_right(a: MbValue, x: MbValue) -> MbValue {
    let pos = with_list(a, |items| {
        let (mut lo, mut hi) = (0usize, items.len());
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            // CPython's bisect_right inserts AFTER any equal elements, i.e.
            // the loop advances `lo` whenever `x !< items[mid]`. With a
            // strict-less-than primitive, that condition is `!lt(x, items[mid])`.
            if lt(x, items[mid]) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    });
    super::super::bigint_ops::int_from_i64(pos as i64)
}

pub fn mb_bisect_insort_left(a: MbValue, x: MbValue) -> MbValue {
    let pos = mb_bisect_bisect_left(a, x).as_int().unwrap_or(0) as usize;
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().insert(pos, x);
            }
        }
    }
    MbValue::none()
}

pub fn mb_bisect_insort_right(a: MbValue, x: MbValue) -> MbValue {
    let pos = mb_bisect_bisect_right(a, x).as_int().unwrap_or(0) as usize;
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.write().unwrap().insert(pos, x);
            }
        }
    }
    MbValue::none()
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::super::super::rc::{MbObject, ObjData};
    use super::super::super::value::MbValue;
    use super::*;

    fn make_int_list(items: &[i64]) -> MbValue {
        let vals: Vec<MbValue> = items.iter().map(|&i| MbValue::from_int(i)).collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    fn list_int_at(val: MbValue, idx: usize) -> Option<i64> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).and_then(|v| v.as_int())
            } else {
                None
            }
        })
    }

    fn list_len(val: MbValue) -> usize {
        val.as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read().unwrap().len()
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    #[test]
    fn test_bisect_left_duplicates() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(2)).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_bisect_right_duplicates() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a, MbValue::from_int(2)).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_bisect_boundary_before() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(0)).as_int(),
            Some(0)
        );
        let a2 = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a2, MbValue::from_int(0)).as_int(),
            Some(0)
        );
    }

    #[test]
    fn test_bisect_boundary_after() {
        let a = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_int(4)).as_int(),
            Some(4)
        );
        let a2 = make_int_list(&[1, 2, 2, 3]);
        assert_eq!(
            mb_bisect_bisect_right(a2, MbValue::from_int(4)).as_int(),
            Some(4)
        );
    }

    #[test]
    fn test_insort_left() {
        let a = make_int_list(&[1, 3]);
        mb_bisect_insort_left(a, MbValue::from_int(2));
        assert_eq!(list_len(a), 3);
        assert_eq!(list_int_at(a, 0), Some(1));
        assert_eq!(list_int_at(a, 1), Some(2));
        assert_eq!(list_int_at(a, 2), Some(3));
    }

    #[test]
    fn test_insort_right() {
        let a = make_int_list(&[1, 2, 3]);
        mb_bisect_insort_right(a, MbValue::from_int(2));
        assert_eq!(list_len(a), 4);
        assert_eq!(list_int_at(a, 1), Some(2));
        assert_eq!(list_int_at(a, 2), Some(2));
        mb_bisect_insort_left(MbValue::none(), MbValue::from_int(1));
        mb_bisect_insort_right(MbValue::none(), MbValue::from_int(1));
    }

    #[test]
    fn test_float_ordering_no_truncation() {
        // Regression: old shim mapped float→i64 via item_key, so 1.4 was
        // treated as equal to 1, producing wrong positions. With strict
        // `lt`, 1.4 sorts between 1 and 2 — bisect_left must return 1.
        let a = make_int_list(&[1, 2, 3]);
        // Insert a float value
        assert_eq!(
            mb_bisect_bisect_left(a, MbValue::from_float(1.4)).as_int(),
            Some(1),
        );
    }
}
