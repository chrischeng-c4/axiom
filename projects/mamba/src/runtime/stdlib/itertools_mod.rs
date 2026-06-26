use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// itertools module for Mamba (#392).
///
/// Provides eager list-returning implementations:
/// chain, islice, zip_longest, product, permutations,
/// combinations, repeat, accumulate
use std::collections::HashMap;

/// Extract a String from an MbValue that wraps a heap Str.
#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// True if `val` is a trailing kwargs dict (a `Dict` object). The mamba call
/// lowering folds keyword arguments into a final positional `dict` argument, so
/// native dispatchers that accept keyword-only parameters (`fillvalue`, `key`,
/// `initial`, `repeat`, `times`, ...) receive `{"name": value}` in the slot a
/// caller would otherwise leave empty.
fn is_kwargs_dict(val: MbValue) -> bool {
    matches!(val.as_ptr(), Some(ptr) if unsafe {
        matches!((*ptr).data, ObjData::Dict(_))
    })
}

/// Read a named entry from a kwargs dict, returning `None` when the value is
/// not a dict or the key is absent.
fn kwargs_get(val: MbValue, name: &str) -> Option<MbValue> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let guard = lock.read().unwrap();
            let key = super::super::dict_ops::DictKey::Str(name.to_string());
            guard.get(&key).copied()
        } else {
            None
        }
    }
}

/// Split a dispatcher arg slice into `(positional, kwargs)` where `kwargs` is
/// the trailing folded keyword dict (if any). Distinguishing a genuine trailing
/// dict argument from a kwargs dict is impossible in general, but every
/// itertools entry point that takes a real positional iterable would iterate it
/// — and these helpers only consult `kwargs` for the specific named keyword
/// they expect, so a real `dict` positional is left in `positional` untouched
/// when none of the expected keywords are present.
fn split_kwargs(a: &[MbValue], expected: &[&str]) -> (usize, MbValue) {
    if let Some(last) = a.last().copied() {
        if is_kwargs_dict(last) && expected.iter().any(|k| kwargs_get(last, k).is_some()) {
            return (a.len() - 1, last);
        }
    }
    (a.len(), MbValue::none())
}

/// Raise a TypeError with `msg` and return None (mamba's native-call error
/// convention — the pending exception is honored by the interpreter once the
/// dispatcher returns).
fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise a ValueError with `msg` and return None.
fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// True when a non-StopIteration exception is currently pending (a source
/// iterable raised mid-drain). StopIteration is the normal end-of-iteration
/// signal and must NOT abort eager materialization.
fn real_exception_pending() -> bool {
    match super::super::exception::current_exception_type() {
        Some(t) => t != "StopIteration",
        None => false,
    }
}

// ── Dispatch wrappers: native ABI (args_ptr, nargs) to match mb_call_spread ──

/// Safe wrapper over `from_raw_parts` — returns `&[]` for `nargs == 0` or null pointer
/// (from_raw_parts requires non-null aligned ptr even when len is 0).
unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

unsafe extern "C" fn dispatch_chain(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let mut items: Vec<MbValue> = Vec::new();
    for arg in a {
        items.extend(extract_list(*arg));
        // A source argument that raises mid-iteration (or a non-iterable
        // element that raises TypeError) leaves a pending exception — stop and
        // propagate it instead of returning a partial list.
        if real_exception_pending() {
            return MbValue::none();
        }
    }
    MbValue::from_ptr(MbObject::new_list(items))
}

/// `itertools.chain.from_iterable(iterables)` — chain over a single iterable
/// whose elements are themselves iterables. Equivalent to `chain(*iterables)`:
/// materialize each sub-iterable in order and concatenate. Registered as a
/// method of the native `chain` class so `itertools.chain.from_iterable`
/// resolves to a callable unbound method via mb_getattr's func->native-class
/// method bridge.
unsafe extern "C" fn dispatch_chain_from_iterable(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let source = a.first().copied().unwrap_or_else(MbValue::none);
    // The argument to chain.from_iterable must itself be iterable; a bare
    // scalar (e.g. chain.from_iterable(123)) raises TypeError like CPython.
    // Scalars carry no pointer; iterator handles / generators are bare ints
    // too, so exempt those before flagging.
    if source.as_ptr().is_none()
        && !super::super::iter::mb_is_iterator_handle(source)
        && !super::super::generator::is_known_generator(source)
    {
        raise_type_error(&format!(
            "'{}' object is not iterable",
            super::super::builtins::value_type_name(source)
        ));
        return MbValue::none();
    }
    super::super::iter::mb_chain_from_iterable_iter(source)
}

unsafe extern "C" fn dispatch_islice(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // CPython islice() index arguments (`start`, `stop`, `step`) must each be a
    // non-negative integer or None; anything else raises ValueError. The 2-arg
    // form `islice(it, stop)` validates `stop`; the 3/4-arg form validates
    // `start`, `stop` and `step` (step must additionally be >= 1).
    // `validate_index` returns Some(error_value) when the argument is invalid.
    let validate_index = |v: MbValue, is_step: bool| -> Option<MbValue> {
        if v.is_none() {
            return None;
        }
        match v.as_int() {
            Some(n) if n >= 0 && !(is_step && n == 0) => None,
            _ => Some(raise_value_error(
                "Indices for islice() must be None or an integer: 0 <= x <= sys.maxsize.",
            )),
        }
    };
    match nargs {
        0 | 1 => {}
        2 => {
            // islice(iterable, stop)
            if let Some(err) = validate_index(a[1], false) {
                return err;
            }
        }
        _ => {
            // islice(iterable, start, stop[, step])
            if let Some(err) = validate_index(a[1], false) {
                return err;
            }
            if let Some(err) = validate_index(a[2], false) {
                return err;
            }
            if let Some(step) = a.get(3).copied() {
                if let Some(err) = validate_index(step, true) {
                    return err;
                }
            }
        }
    }
    mb_itertools_islice(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
        a.get(3).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_zip_longest(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // The mamba call lowering folds keyword args into a final dict positional
    // arg. `zip_longest(*iters, fillvalue=x)` arrives with a trailing
    // `{"fillvalue": x}` dict. CPython also rejects any keyword other than
    // `fillvalue` with TypeError, so a trailing kwargs dict that carries any
    // unexpected key is an error.
    let (npos, kw) = split_kwargs(a, &["fillvalue"]);
    if !kw.is_none() {
        // Reject unknown keywords (anything besides `fillvalue`).
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    for k in guard.keys() {
                        if !matches!(k, super::super::dict_ops::DictKey::Str(s) if s == "fillvalue")
                        {
                            return raise_type_error(
                                "zip_longest() got an unexpected keyword argument",
                            );
                        }
                    }
                }
            }
        }
    }
    let fill = kwargs_get(kw, "fillvalue").unwrap_or_else(MbValue::none);
    let iters: Vec<MbValue> = a[..npos].to_vec();
    mb_itertools_zip_longest_n(&iters, fill)
}

unsafe extern "C" fn dispatch_product(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // `product(*pools, repeat=k)` — the `repeat` keyword (default 1) folds into
    // a trailing kwargs dict. The remaining positionals are the input pools.
    let (npos, kw) = split_kwargs(a, &["repeat"]);
    let repeat = match kwargs_get(kw, "repeat") {
        Some(v) => match v.as_int() {
            Some(n) if n >= 0 => n as usize,
            _ => return raise_value_error("product() repeat argument must be non-negative"),
        },
        None => 1,
    };
    let pools: Vec<MbValue> = a[..npos].to_vec();
    mb_itertools_product_n(&pools, repeat)
}

/// Resolve the `r` argument for the combinatoric functions. `r` may be the
/// second positional, an `r=` keyword (folded into a trailing kwargs dict), or
/// absent (None → "use the full length"). Returns `Err(error_value)` when `r`
/// is a negative integer (CPython raises ValueError) or a non-int non-None.
fn resolve_r(a: &[MbValue]) -> Result<MbValue, MbValue> {
    let (npos, kw) = split_kwargs(a, &["r"]);
    let r = kwargs_get(kw, "r")
        .or_else(|| if npos >= 2 { a.get(1).copied() } else { None })
        .unwrap_or_else(MbValue::none);
    if r.is_none() {
        return Ok(r);
    }
    match r.as_int() {
        Some(n) if n >= 0 => Ok(r),
        Some(_) => Err(raise_value_error("r must be non-negative")),
        None => Err(raise_type_error("Expected a non-negative int as r")),
    }
}

unsafe extern "C" fn dispatch_permutations(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let r = match resolve_r(a) {
        Ok(r) => r,
        Err(e) => return e,
    };
    mb_itertools_permutations(a.get(0).copied().unwrap_or_else(MbValue::none), r)
}

unsafe extern "C" fn dispatch_combinations(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let r = match resolve_r(a) {
        Ok(r) => r,
        Err(e) => return e,
    };
    mb_itertools_combinations(a.get(0).copied().unwrap_or_else(MbValue::none), r)
}

unsafe extern "C" fn dispatch_combinations_with_replacement(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    let r = match resolve_r(a) {
        Ok(r) => r,
        Err(e) => return e,
    };
    mb_itertools_combinations_with_replacement(a.get(0).copied().unwrap_or_else(MbValue::none), r)
}

unsafe extern "C" fn dispatch_repeat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // `repeat(object[, times])`. `times` may be the second positional or a
    // `times=` keyword (folded into a trailing kwargs dict). When present it
    // must be an integer; CPython raises TypeError otherwise (and without this
    // guard a non-int `times` is read as None → infinite repeat → `list()`
    // hangs forever).
    let (npos, kw) = split_kwargs(a, &["times"]);
    let val = a.get(0).copied().unwrap_or_else(MbValue::none);
    let times = kwargs_get(kw, "times")
        .or_else(|| if npos >= 2 { a.get(1).copied() } else { None })
        .unwrap_or_else(MbValue::none);
    if !times.is_none() && times.as_int().is_none() {
        return raise_type_error("'str' object cannot be interpreted as an integer");
    }
    mb_itertools_repeat(val, times)
}

unsafe extern "C" fn dispatch_accumulate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // `accumulate(iterable[, func, *, initial=None])`. `initial` is a
    // keyword-only parameter (folded into a trailing kwargs dict); when
    // supplied it is yielded first and seeds the running fold.
    let (npos, kw) = split_kwargs(a, &["initial", "func"]);
    let iterable = a.get(0).copied().unwrap_or_else(MbValue::none);
    let func = kwargs_get(kw, "func")
        .or_else(|| if npos >= 2 { a.get(1).copied() } else { None })
        .unwrap_or_else(MbValue::none);
    let initial = kwargs_get(kw, "initial").unwrap_or_else(MbValue::none);
    let has_initial = kwargs_get(kw, "initial").is_some();
    mb_itertools_accumulate_full(iterable, func, initial, has_initial)
}

unsafe extern "C" fn dispatch_takewhile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_takewhile(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_dropwhile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_dropwhile(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_filterfalse(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_filterfalse(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_compress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_compress(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_starmap(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_starmap(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_pairwise(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_pairwise(a.get(0).copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_batched(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_batched(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_groupby(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    // `groupby(iterable, key=None)`. `key` may be the second positional or a
    // `key=` keyword (folded into a trailing kwargs dict). A non-None key that
    // is not callable raises TypeError in CPython (e.g. `groupby('abc', [])`).
    let (npos, kw) = split_kwargs(a, &["key"]);
    let iterable = a.get(0).copied().unwrap_or_else(MbValue::none);
    let key = kwargs_get(kw, "key")
        .or_else(|| if npos >= 2 { a.get(1).copied() } else { None })
        .unwrap_or_else(MbValue::none);
    if !key.is_none() && super::super::builtins::mb_callable(key).as_bool() != Some(true) {
        return raise_type_error("groupby() key argument must be callable or None");
    }
    mb_itertools_groupby(iterable, key)
}

unsafe extern "C" fn dispatch_tee(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_tee(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_count(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_count(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_cycle(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { args_slice(args_ptr, nargs) };
    mb_itertools_cycle(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

/// Register the itertools module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: [(&str, usize); 20] = [
        ("chain", dispatch_chain as *const () as usize),
        ("islice", dispatch_islice as *const () as usize),
        ("zip_longest", dispatch_zip_longest as *const () as usize),
        ("product", dispatch_product as *const () as usize),
        ("permutations", dispatch_permutations as *const () as usize),
        ("combinations", dispatch_combinations as *const () as usize),
        (
            "combinations_with_replacement",
            dispatch_combinations_with_replacement as *const () as usize,
        ),
        ("repeat", dispatch_repeat as *const () as usize),
        ("accumulate", dispatch_accumulate as *const () as usize),
        ("takewhile", dispatch_takewhile as *const () as usize),
        ("dropwhile", dispatch_dropwhile as *const () as usize),
        ("filterfalse", dispatch_filterfalse as *const () as usize),
        ("compress", dispatch_compress as *const () as usize),
        ("starmap", dispatch_starmap as *const () as usize),
        ("pairwise", dispatch_pairwise as *const () as usize),
        ("batched", dispatch_batched as *const () as usize),
        ("groupby", dispatch_groupby as *const () as usize),
        ("tee", dispatch_tee as *const () as usize),
        ("count", dispatch_count as *const () as usize),
        ("cycle", dispatch_cycle as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // `itertools.chain` is a type whose classmethod `from_iterable` must be a
    // callable attribute on the class object (`itertools.chain.from_iterable`).
    // The `chain` name is bound to `dispatch_chain` as a func value, so register
    // a native `chain` class carrying the `from_iterable` method and map the
    // constructor func addr -> "chain" in NATIVE_TYPE_NAMES. mb_getattr's
    // func->native-class method bridge then resolves the attribute to a callable
    // unbound method (lookup_method against the table mb_class_register fills).
    let from_iterable_addr = dispatch_chain_from_iterable as *const () as usize;
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(from_iterable_addr as u64);
    });
    let mut chain_methods: HashMap<String, MbValue> = HashMap::new();
    chain_methods.insert(
        "from_iterable".to_string(),
        MbValue::from_func(from_iterable_addr),
    );
    super::super::class::mb_class_register("chain", Vec::new(), chain_methods);
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            dispatch_chain as *const () as usize as u64,
            "chain".to_string(),
        );
    });

    super::register_module("itertools", attrs);
}

// ── Helpers ──

/// Extract items from any iterable (list, tuple, str, set, generator, custom __iter__).
/// Goes through the iterator protocol for non-sequence types.
fn extract_list(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                ObjData::Set(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::FrozenSet(items) => return items.clone(),
                ObjData::Str(s) => {
                    return s
                        .chars()
                        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                        .collect();
                }
                _ => {}
            }
        }
    }
    // A range handle is a re-iterable sequence, not a one-shot iterator:
    // materialize it from its (current, stop, step) params WITHOUT consuming,
    // so the same handle can be extracted more than once. This matters when a
    // single range object is aliased across pools, e.g.
    // `product(*[range(n)] * 2)` (which shares one object) or `chain(r, r)`.
    // The generic mb_iter/mb_next fallback below would drain it on the first
    // extract and yield an empty list on the second.
    if let Some((cur, stop, step)) = super::super::iter::mb_iter_range_params(val) {
        let mut out = Vec::new();
        let mut c = cur;
        if step > 0 {
            while c < stop {
                out.push(MbValue::from_int(c));
                c += step;
            }
        } else if step < 0 {
            while c > stop {
                out.push(MbValue::from_int(c));
                c += step;
            }
        }
        return out;
    }
    // Fall back to iterator protocol (generators, iterator handles, custom iter).
    let iter_handle = super::super::iter::mb_iter(val);
    if iter_handle.is_none() {
        return vec![];
    }
    let mut items = Vec::new();
    loop {
        if super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        let item = super::super::iter::mb_next(iter_handle);
        // A source iterable that raises mid-drain (e.g. a generator that
        // `raise ValueError` after yielding) leaves a pending non-StopIteration
        // exception. Stop materializing and leave the exception pending so the
        // itertools call (chain, zip_longest, product, ...) propagates it to
        // the caller, matching CPython's lazy error-propagation semantics.
        if real_exception_pending() {
            break;
        }
        if item.is_none() && super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        items.push(item);
    }
    items
}

#[allow(dead_code)]
fn _unused_extract_list_original(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

// ── Runtime functions ──

/// itertools.chain(a, b) -> concatenation of two lists
pub fn mb_itertools_chain(a: MbValue, b: MbValue) -> MbValue {
    let mut items_a = extract_list(a);
    let items_b = extract_list(b);
    items_a.extend(items_b);
    MbValue::from_ptr(MbObject::new_list(items_a))
}

/// itertools.islice(iterable, start, stop) -> list[start..stop]
pub fn mb_itertools_islice(
    iterable: MbValue,
    start: MbValue,
    stop: MbValue,
    step: MbValue,
) -> MbValue {
    // The maximum index we ever need is `stop` (the 2nd positional in the
    // 2-arg form lives in `start`). Consuming only that many elements keeps
    // islice finite over infinite sources like `count()` / `cycle()` — a full
    // drain would hang. `None` stop means "until exhausted" (finite sources).
    let consume_cap: Option<usize> = if stop.is_none() && step.is_none() {
        start.as_int().map(|v| v.max(0) as usize)
    } else {
        stop.as_int().map(|v| v.max(0) as usize)
    };
    // Materialize any iterable (list, tuple, generator, ...) via iterator protocol.
    let items: Vec<MbValue> = {
        let iter_handle = super::super::iter::mb_iter(iterable);
        if iter_handle.is_none() {
            extract_list(iterable)
        } else {
            let mut acc = Vec::new();
            loop {
                if consume_cap.map_or(false, |cap| acc.len() >= cap) {
                    break;
                }
                if super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
                    break;
                }
                let item = super::super::iter::mb_next(iter_handle);
                if item.is_none()
                    && super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false)
                {
                    break;
                }
                acc.push(item);
            }
            acc
        }
    };
    // islice(iterable, stop)                       — start=0, step=1
    // islice(iterable, start, stop)                — step=1
    // islice(iterable, start, stop, step)
    // We always receive 4 args from the dispatcher (None-padded). Disambiguate
    // by checking how many args were actually provided: if `stop` and `step`
    // are None, treat `start` as the stop.
    let s: usize;
    let e: usize;
    let st: usize;
    if stop.is_none() && step.is_none() {
        // islice(iterable, stop)
        s = 0;
        e = start.as_int().unwrap_or(items.len() as i64).max(0) as usize;
        st = 1;
    } else {
        s = start.as_int().unwrap_or(0).max(0) as usize;
        e = stop.as_int().unwrap_or(items.len() as i64).max(0) as usize;
        st = step.as_int().unwrap_or(1).max(1) as usize;
    }
    let e = e.min(items.len());
    let s = s.min(e);
    let sliced: Vec<MbValue> = items[s..e].iter().step_by(st).copied().collect();
    MbValue::from_ptr(MbObject::new_list(sliced))
}

/// itertools.zip_longest(a, b) -> list of 2-element tuples,
/// padding shorter list with None.
pub fn mb_itertools_zip_longest(a: MbValue, b: MbValue) -> MbValue {
    mb_itertools_zip_longest_fill(a, b, MbValue::none())
}

/// itertools.zip_longest(a, b, fillvalue=...)
pub fn mb_itertools_zip_longest_fill(a: MbValue, b: MbValue, fill: MbValue) -> MbValue {
    let items_a = extract_list(a);
    let items_b = extract_list(b);
    let len = items_a.len().max(items_b.len());

    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let va = items_a.get(i).copied().unwrap_or(fill);
        let vb = items_b.get(i).copied().unwrap_or(fill);
        let pair = MbObject::new_tuple(vec![va, vb]);
        result.push(MbValue::from_ptr(pair));
    }

    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.product(a, b) -> cartesian product as list of 2-tuples
pub fn mb_itertools_product(a: MbValue, b: MbValue) -> MbValue {
    let items_a = extract_list(a);
    let items_b = extract_list(b);

    let mut result = Vec::with_capacity(items_a.len() * items_b.len());
    for va in &items_a {
        for vb in &items_b {
            let pair = MbObject::new_tuple(vec![*va, *vb]);
            result.push(MbValue::from_ptr(pair));
        }
    }

    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.product(*pools, repeat=k) — N-ary cartesian product.
///
/// CPython rules reproduced exactly:
///   - `product()` (no pools) yields a single empty tuple: `[()]`.
///   - any empty pool collapses the whole product to `[]`.
///   - `repeat=k` repeats the supplied pools `k` times (`repeat=0` → `[()]`).
/// Each result row is a tuple whose length is `len(pools) * repeat`.
fn mb_itertools_product_n(pools_in: &[MbValue], repeat: usize) -> MbValue {
    // Materialize every pool once; a non-iterable pool raises through
    // `extract_list`/`mb_iter`, leaving a pending exception.
    let mut pools: Vec<Vec<MbValue>> = Vec::with_capacity(pools_in.len());
    for p in pools_in {
        let items = extract_list(*p);
        if real_exception_pending() {
            return MbValue::none();
        }
        pools.push(items);
    }
    // `repeat` concatenates `repeat` copies of the pool list.
    let mut expanded: Vec<&Vec<MbValue>> = Vec::with_capacity(pools.len() * repeat);
    for _ in 0..repeat {
        for p in &pools {
            expanded.push(p);
        }
    }

    // Start from a single empty tuple and fan out over each pool.
    let mut result: Vec<Vec<MbValue>> = vec![Vec::new()];
    for pool in &expanded {
        if pool.is_empty() {
            // An empty pool collapses the cartesian product.
            return MbValue::from_ptr(MbObject::new_list(Vec::new()));
        }
        let mut next: Vec<Vec<MbValue>> = Vec::with_capacity(result.len() * pool.len());
        for prefix in &result {
            for &item in pool.iter() {
                let mut row = prefix.clone();
                row.push(item);
                next.push(row);
            }
        }
        result = next;
    }

    let tuples: Vec<MbValue> = result
        .into_iter()
        .map(|row| MbValue::from_ptr(MbObject::new_tuple(row)))
        .collect();
    MbValue::from_ptr(MbObject::new_list(tuples))
}

/// itertools.zip_longest(*iterables, fillvalue=None) — N-ary parallel zip that
/// pads every short column up to the longest length with `fill`.
fn mb_itertools_zip_longest_n(iters: &[MbValue], fill: MbValue) -> MbValue {
    if iters.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    // Materialize each source in turn, stopping at the first that raises a
    // real (non-StopIteration) error so it propagates instead of being padded
    // over — and so a later source isn't drained after an upstream failure.
    let mut cols: Vec<Vec<MbValue>> = Vec::with_capacity(iters.len());
    for it in iters {
        let col = extract_list(*it);
        if real_exception_pending() {
            return MbValue::none();
        }
        cols.push(col);
    }
    let len = cols.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let row: Vec<MbValue> = cols
            .iter()
            .map(|c| c.get(i).copied().unwrap_or(fill))
            .collect();
        result.push(MbValue::from_ptr(MbObject::new_tuple(row)));
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.permutations(iterable, r) -> r-length permutations
pub fn mb_itertools_permutations(iterable: MbValue, r: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let r_val = r.as_int().unwrap_or(items.len() as i64) as usize;

    if r_val > items.len() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let mut result = Vec::new();
    let indices: Vec<usize> = (0..items.len()).collect();
    permute_helper(&items, &indices, r_val, &mut vec![], &mut result);

    MbValue::from_ptr(MbObject::new_list(result))
}

fn permute_helper(
    items: &[MbValue],
    available: &[usize],
    r: usize,
    current: &mut Vec<MbValue>,
    result: &mut Vec<MbValue>,
) {
    if current.len() == r {
        let tuple = MbObject::new_tuple_borrowed(current.clone());
        result.push(MbValue::from_ptr(tuple));
        return;
    }
    for &idx in available {
        if current.len() < r {
            current.push(items[idx]);
            let remaining: Vec<usize> = available.iter().copied().filter(|&i| i != idx).collect();
            permute_helper(items, &remaining, r, current, result);
            current.pop();
        }
    }
}

/// itertools.combinations(iterable, r) -> r-length combinations
pub fn mb_itertools_combinations(iterable: MbValue, r: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let r_val = r.as_int().unwrap_or(items.len() as i64) as usize;

    if r_val > items.len() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let mut result = Vec::new();
    combine_helper(&items, r_val, 0, &mut vec![], &mut result);

    MbValue::from_ptr(MbObject::new_list(result))
}

fn combine_helper(
    items: &[MbValue],
    r: usize,
    start: usize,
    current: &mut Vec<MbValue>,
    result: &mut Vec<MbValue>,
) {
    if current.len() == r {
        let tuple = MbObject::new_tuple_borrowed(current.clone());
        result.push(MbValue::from_ptr(tuple));
        return;
    }
    for i in start..items.len() {
        current.push(items[i]);
        combine_helper(items, r, i + 1, current, result);
        current.pop();
    }
}

/// itertools.combinations_with_replacement(iterable, r)
/// — r-length combinations allowing the same element to be picked repeatedly.
pub fn mb_itertools_combinations_with_replacement(iterable: MbValue, r: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let r_val = r.as_int().unwrap_or(0).max(0) as usize;

    if items.is_empty() {
        if r_val == 0 {
            let empty_tuple = MbObject::new_tuple(vec![]);
            return MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(empty_tuple)]));
        }
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let mut result = Vec::new();
    combine_with_replacement_helper(&items, r_val, 0, &mut vec![], &mut result);
    MbValue::from_ptr(MbObject::new_list(result))
}

fn combine_with_replacement_helper(
    items: &[MbValue],
    r: usize,
    start: usize,
    current: &mut Vec<MbValue>,
    result: &mut Vec<MbValue>,
) {
    if current.len() == r {
        let tuple = MbObject::new_tuple_borrowed(current.clone());
        result.push(MbValue::from_ptr(tuple));
        return;
    }
    for i in start..items.len() {
        current.push(items[i]);
        combine_with_replacement_helper(items, r, i, current, result);
        current.pop();
    }
}

/// itertools.repeat(val, n) -> list of val repeated n times
pub fn mb_itertools_repeat(val: MbValue, n: MbValue) -> MbValue {
    // Lazy handle covers both forms: `repeat(v)` (n is None → infinite) and
    // `repeat(v, k)` (bounded). `list(repeat(v, k))` drains to k copies;
    // `next(repeat(v))` yields v forever.
    super::super::iter::mb_repeat_iter(val, n)
}

/// itertools.count(start, step, limit) -> bounded arithmetic sequence
///
/// **Carve-out**: CPython's `count(start=0, step=1)` is an infinite
/// iterator. Mamba materializes eagerly (matching the existing
/// `repeat(val, n)` / `accumulate(iterable)` family policy), so a
/// third `limit` argument is required to bound the result. Without
/// it the call returns an empty list. The common CPython idiom
/// `list(islice(count(0, 2), 10))` maps directly to
/// `count(0, 2, 10)` under this shim.
///
/// Tracked as a follow-up under #1452 conformance — true infinite
/// iterator support is blocked on iterator-handle plumbing
/// ([[project_mamba_integer_handle_pattern]] applied to lazy iters).
pub fn mb_itertools_count(start: MbValue, step: MbValue, limit: MbValue) -> MbValue {
    let Some(n) = limit.as_int() else {
        // No bound supplied: the true CPython form — a lazy infinite counter
        // backed by an iterator handle that `next()` / `zip()` consume.
        return super::super::iter::mb_count_iter(start, step);
    };
    if n <= 0 {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    // Promote to float if either start or step is float; otherwise stay int.
    let start_int = start.as_int();
    let step_int = step.as_int();
    let start_f = start.as_float();
    let step_f = step.as_float();
    let use_float = start_f.is_some() || step_f.is_some();

    let mut result = Vec::with_capacity(n as usize);
    if use_float {
        let s = start_f
            .or_else(|| start_int.map(|v| v as f64))
            .unwrap_or(0.0);
        let d = step_f.or_else(|| step_int.map(|v| v as f64)).unwrap_or(1.0);
        for i in 0..n {
            result.push(MbValue::from_float(s + d * i as f64));
        }
    } else {
        let s = start_int.unwrap_or(0);
        let d = step_int.unwrap_or(1);
        for i in 0..n {
            result.push(MbValue::from_int(s + d * i));
        }
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.cycle(iterable, n_cycles) -> iterable repeated n_cycles times
///
/// **Carve-out**: CPython's `cycle(iterable)` is an infinite iterator
/// (it caches the iterable then re-yields forever). Mamba materializes
/// eagerly, so a second `n_cycles` argument bounds the result. Without
/// it the call returns an empty list. `list(islice(cycle([1,2,3]), 7))`
/// becomes `cycle([1,2,3], 3)[:7]` in idiomatic mamba.
///
/// Tracked as a follow-up under #1452 conformance.
pub fn mb_itertools_cycle(iterable: MbValue, n_cycles: MbValue) -> MbValue {
    // Bounded internal form `cycle(iterable, n)` (used by call sites that pass a
    // synthetic repeat count) stays eager. The true CPython form `cycle(iterable)`
    // is infinite and backed by a lazy handle that next()/islice consume.
    let Some(n) = n_cycles.as_int() else {
        return super::super::iter::mb_cycle_iter(iterable);
    };
    if n <= 0 {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let items = extract_list(iterable);
    if items.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let mut result = Vec::with_capacity(items.len() * n as usize);
    for _ in 0..n {
        result.extend_from_slice(&items);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.accumulate(iterable) -> running sum list
///
/// Performs running sum over a list of ints/floats.
pub fn mb_itertools_accumulate(iterable: MbValue) -> MbValue {
    let items = extract_list(iterable);

    if items.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let mut result = Vec::with_capacity(items.len());
    let mut acc_int: i64 = 0;
    let mut acc_float: f64 = 0.0;
    let mut use_float = false;

    for (i, item) in items.iter().enumerate() {
        if let Some(v) = item.as_int() {
            if use_float {
                acc_float += v as f64;
            } else {
                acc_int += v;
            }
        } else if let Some(v) = item.as_float() {
            if !use_float && i > 0 {
                acc_float = acc_int as f64;
            }
            use_float = true;
            acc_float += v;
        }

        if use_float {
            result.push(MbValue::from_float(acc_float));
        } else {
            result.push(MbValue::from_int(acc_int));
        }
    }

    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.accumulate(iterable, func) -> running application of binary func
pub fn mb_itertools_accumulate_func(iterable: MbValue, func: MbValue) -> MbValue {
    let items = extract_list(iterable);
    if items.is_empty() {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }
    let mut result = Vec::with_capacity(items.len());
    let mut acc = items[0];
    result.push(acc);
    for item in items.iter().skip(1) {
        let args = MbValue::from_ptr(MbObject::new_list(vec![acc, *item]));
        acc = super::super::builtins::mb_call_spread(func, args);
        result.push(acc);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.accumulate(iterable, func=None, *, initial=None)
///
/// Unified entry that honors the optional binary `func` and the keyword-only
/// `initial` seed. When `initial` is supplied it is yielded first and seeds the
/// running fold (so the output is one element longer than the input). Falls
/// back to the numeric-sum / func-fold helpers when no seed is given.
pub fn mb_itertools_accumulate_full(
    iterable: MbValue,
    func: MbValue,
    initial: MbValue,
    has_initial: bool,
) -> MbValue {
    if !has_initial {
        // No seed: preserve the existing numeric-sum / func-fold behavior.
        return if !func.is_none() {
            mb_itertools_accumulate_func(iterable, func)
        } else {
            mb_itertools_accumulate(iterable)
        };
    }

    // With an initial seed CPython yields `initial`, then folds the rest with
    // either `func` or `+`. The numeric `+` path is handled via mb_add so that
    // `accumulate([1,2,3,4], initial=0)` produces `[0, 1, 3, 6, 10]`.
    let items = extract_list(iterable);
    let mut result = Vec::with_capacity(items.len() + 1);
    let mut acc = initial;
    result.push(acc);
    for item in items {
        if !func.is_none() {
            let args = MbValue::from_ptr(MbObject::new_list(vec![acc, item]));
            acc = super::super::builtins::mb_call_spread(func, args);
        } else {
            acc = super::super::builtins::mb_add(acc, item);
        }
        result.push(acc);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.takewhile(pred, iterable) -> list of elements while pred is true
// REQ: R2
pub fn mb_itertools_takewhile(pred: MbValue, iterable: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let mut result = Vec::new();
    for item in items {
        let args = MbValue::from_ptr(MbObject::new_list(vec![item]));
        let ok = super::super::builtins::mb_call_spread(pred, args);
        // CPython: any truthy return passes — use Python truthiness, not
        // strict-bool tag matching. `as_bool()` only matches TAG_BOOL,
        // so `lambda x: x % 2` (returning int) was treated as false and
        // takewhile returned `[]` for non-bool predicates.
        if super::super::builtins::mb_bool(ok).as_bool() != Some(true) {
            break;
        }
        result.push(item);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.dropwhile(pred, iterable) -> list of remaining elements after pred becomes false
// REQ: R2
pub fn mb_itertools_dropwhile(pred: MbValue, iterable: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let mut dropping = true;
    let mut result = Vec::new();
    for item in items {
        if dropping {
            let args = MbValue::from_ptr(MbObject::new_list(vec![item]));
            let ok = super::super::builtins::mb_call_spread(pred, args);
            if super::super::builtins::mb_bool(ok).as_bool() == Some(true) {
                continue;
            }
            dropping = false;
        }
        result.push(item);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.filterfalse(pred, iterable) -> list of elements where pred is false
// REQ: R2
pub fn mb_itertools_filterfalse(pred: MbValue, iterable: MbValue) -> MbValue {
    let items = extract_list(iterable);
    let mut result = Vec::new();
    for item in items {
        let args = MbValue::from_ptr(MbObject::new_list(vec![item]));
        let ok = super::super::builtins::mb_call_spread(pred, args);
        if super::super::builtins::mb_bool(ok).as_bool() != Some(true) {
            result.push(item);
        }
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.compress(data, selectors) -> list of `data` items where the
/// parallel selector is truthy. Stops at the shorter of the two inputs
/// (CPython rule).
pub fn mb_itertools_compress(data: MbValue, selectors: MbValue) -> MbValue {
    let data_items = extract_list(data);
    let sel_items = extract_list(selectors);
    let mut result = Vec::with_capacity(data_items.len().min(sel_items.len()));
    for (d, s) in data_items.iter().zip(sel_items.iter()) {
        if super::super::builtins::mb_bool(*s).as_bool() == Some(true) {
            result.push(*d);
        }
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.starmap(func, iterable) -> list of `func(*args)` for each
/// `args` tuple in `iterable`. Equivalent to
/// `[func(*args) for args in iterable]`.
pub fn mb_itertools_starmap(func: MbValue, iterable: MbValue) -> MbValue {
    let rows = extract_list(iterable);
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let args_list = match row.as_ptr() {
            Some(ptr) => unsafe {
                match &(*ptr).data {
                    ObjData::Tuple(items) => MbValue::from_ptr(MbObject::new_list(items.clone())),
                    ObjData::List(ref lock) => {
                        MbValue::from_ptr(MbObject::new_list(lock.read().unwrap().to_vec()))
                    }
                    _ => MbValue::from_ptr(MbObject::new_list(vec![row])),
                }
            },
            None => MbValue::from_ptr(MbObject::new_list(vec![row])),
        };
        let val = super::super::builtins::mb_call_spread(func, args_list);
        result.push(val);
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.pairwise(iterable) -> list of consecutive (prev, curr) pairs
/// (Py3.10+, PEP 618-adjacent). Returns `[]` for inputs shorter than 2.
pub fn mb_itertools_pairwise(iterable: MbValue) -> MbValue {
    let items = extract_list(iterable);
    if items.len() < 2 {
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    let mut result = Vec::with_capacity(items.len() - 1);
    for w in items.windows(2) {
        result.push(MbValue::from_ptr(MbObject::new_tuple(vec![w[0], w[1]])));
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.batched(iterable, n) -> list of tuples of size `n`
///
/// CPython 3.12+. Successive tuples of size `n`; the last may be shorter.
/// Raises ValueError if `n < 1`.
pub fn mb_itertools_batched(iterable: MbValue, n: MbValue) -> MbValue {
    let n_int = n.as_int().unwrap_or(0);
    if n_int < 1 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str("n must be at least one".to_string())),
        );
        return MbValue::from_ptr(MbObject::new_list(Vec::new()));
    }
    let n_usz = n_int as usize;
    let items = extract_list(iterable);
    let mut result = Vec::with_capacity(items.len().div_ceil(n_usz));
    for chunk in items.chunks(n_usz) {
        result.push(MbValue::from_ptr(MbObject::new_tuple(chunk.to_vec())));
    }
    MbValue::from_ptr(MbObject::new_list(result))
}

/// itertools.groupby(iterable, key=None) -> lazy `(key, group_iterator)` pairs.
///
/// Consecutive elements that share the same `key(elem)` value (or the
/// element itself when `key` is None) are represented as group boundaries.
/// The outer/group iterator pair share state so advancing the outer invalidates
/// the previously active group like CPython's `_grouper`.
pub fn mb_itertools_groupby(iterable: MbValue, key: MbValue) -> MbValue {
    let items = extract_list(iterable);
    if items.is_empty() {
        return super::super::iter::mb_groupby_iter(Vec::new(), Vec::new());
    }
    let has_key = !key.is_none();
    let key_of = |v: MbValue| -> MbValue {
        if has_key {
            let args = MbValue::from_ptr(MbObject::new_list(vec![v]));
            super::super::builtins::mb_call_spread(key, args)
        } else {
            v
        }
    };

    let mut groups: Vec<super::super::iter::GroupByGroupSpec> = Vec::new();
    let mut cur_key = key_of(items[0]);
    let mut start = 0usize;
    for (idx, item) in items.iter().enumerate().skip(1) {
        let k = key_of(*item);
        let same = super::super::builtins::mb_eq(cur_key, k).as_bool() == Some(true);
        if same {
            continue;
        } else {
            groups.push(super::super::iter::GroupByGroupSpec { key: cur_key, start, end: idx });
            cur_key = k;
            start = idx;
        }
    }
    groups.push(super::super::iter::GroupByGroupSpec {
        key: cur_key,
        start,
        end: items.len(),
    });
    super::super::iter::mb_groupby_iter(items, groups)
}

/// itertools.tee(iterable, n=2) -> tuple of `n` independent iterators.
///
/// Mamba returns `n` shallow-copied lists — observably indistinguishable
/// since each is independently consumable. Defaults to n=2 when the
/// argument is missing or not an int.
pub fn mb_itertools_tee(iterable: MbValue, n: MbValue) -> MbValue {
    let n_int = n.as_int().unwrap_or(2);
    if n_int <= 0 {
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    }
    let items = extract_list(iterable);
    let mut copies = Vec::with_capacity(n_int as usize);
    for _ in 0..n_int {
        copies.push(MbValue::from_ptr(MbObject::new_list(items.clone())));
    }
    MbValue::from_ptr(MbObject::new_tuple(copies))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_list(vals: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    #[test]
    fn test_chain() {
        let a = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = make_list(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        let result = mb_itertools_chain(a, b);
        let items = extract_list(result);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[3].as_int(), Some(4));
    }

    #[test]
    fn test_islice() {
        let list = make_list(vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
            MbValue::from_int(40),
        ]);
        let result = mb_itertools_islice(
            list,
            MbValue::from_int(1),
            MbValue::from_int(3),
            MbValue::from_int(1),
        );
        let items = extract_list(result);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_int(), Some(20));
        assert_eq!(items[1].as_int(), Some(30));
    }

    #[test]
    fn test_product() {
        let a = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = make_list(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        let result = mb_itertools_product(a, b);
        let items = extract_list(result);
        // 2 x 2 = 4 pairs
        assert_eq!(items.len(), 4);
    }

    #[test]
    fn test_combinations() {
        let list = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let result = mb_itertools_combinations(list, MbValue::from_int(2));
        let items = extract_list(result);
        // C(3,2) = 3
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_repeat() {
        let result = mb_itertools_repeat(MbValue::from_int(7), MbValue::from_int(4));
        let items = extract_list(result);
        assert_eq!(items.len(), 4);
        for item in &items {
            assert_eq!(item.as_int(), Some(7));
        }
    }

    #[test]
    fn test_accumulate() {
        let list = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let result = mb_itertools_accumulate(list);
        let items = extract_list(result);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(3));
        assert_eq!(items[2].as_int(), Some(6));
        assert_eq!(items[3].as_int(), Some(10));
    }

    #[test]
    fn test_zip_longest() {
        let a = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let b = make_list(vec![MbValue::from_int(10)]);
        let result = mb_itertools_zip_longest(a, b);
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        // Third tuple should have None for second element
        unsafe {
            let third = items[2].as_ptr().unwrap();
            if let ObjData::Tuple(ref elems) = (*third).data {
                assert_eq!(elems[0].as_int(), Some(3));
                assert!(elems[1].is_none());
            } else {
                panic!("expected Tuple");
            }
        }
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_chain_combines_lists() {
        let a = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = make_list(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        let result = mb_itertools_chain(a, b);
        let items = extract_list(result);
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[3].as_int(), Some(4));
    }

    #[test]
    fn test_py312_islice_start_stop() {
        let lst = make_list(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let result = mb_itertools_islice(
            lst,
            MbValue::from_int(1),
            MbValue::from_int(4),
            MbValue::from_int(1),
        );
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[2].as_int(), Some(3));
    }

    #[test]
    fn test_py312_repeat_n_times() {
        let result = mb_itertools_repeat(MbValue::from_int(42), MbValue::from_int(3));
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        for item in &items {
            assert_eq!(item.as_int(), Some(42));
        }
    }

    #[test]
    fn test_py312_accumulate_prefix_sums() {
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let result = mb_itertools_accumulate(lst);
        let items = extract_list(result);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(3));
        assert_eq!(items[2].as_int(), Some(6));
    }

    #[test]
    fn test_py312_combinations_count() {
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let result = mb_itertools_combinations(lst, MbValue::from_int(2));
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_py312_product_two_lists() {
        let a = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let b = make_list(vec![MbValue::from_int(3), MbValue::from_int(4)]);
        let result = mb_itertools_product(a, b);
        let items = extract_list(result);
        assert_eq!(items.len(), 4);
    }

    // Helper: native predicate "x < 3" using the native ABI
    // Returns MbValue::bool(true) when the single int argument is < 3.
    unsafe extern "C" fn pred_lt3(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        let a = unsafe { args_slice(args_ptr, nargs) };
        let v = a.get(0).and_then(|x| x.as_int()).unwrap_or(0);
        MbValue::from_bool(v < 3)
    }

    fn make_pred_lt3() -> MbValue {
        let addr = pred_lt3 as *const () as usize;
        super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        MbValue::from_func(addr)
    }

    // Helper: native predicate "x % 2 != 0" (truthy for odd numbers)
    unsafe extern "C" fn pred_is_odd(args_ptr: *const MbValue, nargs: usize) -> MbValue {
        let a = unsafe { args_slice(args_ptr, nargs) };
        let v = a.get(0).and_then(|x| x.as_int()).unwrap_or(0);
        MbValue::from_bool(v % 2 != 0)
    }

    fn make_pred_is_odd() -> MbValue {
        let addr = pred_is_odd as *const () as usize;
        super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        MbValue::from_func(addr)
    }

    // REQ: R2
    #[test]
    fn test_takewhile_stops_at_first_false() {
        // takewhile(x < 3, [1,2,3,4]) == [1,2]
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let pred = make_pred_lt3();
        let result = mb_itertools_takewhile(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(2));
    }

    // REQ: R2
    #[test]
    fn test_takewhile_empty_when_pred_false_from_start() {
        // takewhile(x < 3, [5,6,7]) == []
        let lst = make_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(6),
            MbValue::from_int(7),
        ]);
        let pred = make_pred_lt3();
        let result = mb_itertools_takewhile(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 0);
    }

    // REQ: R2
    #[test]
    fn test_dropwhile_skips_leading_true() {
        // dropwhile(x < 3, [1,2,3,4]) == [3,4]
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let pred = make_pred_lt3();
        let result = mb_itertools_dropwhile(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].as_int(), Some(3));
        assert_eq!(items[1].as_int(), Some(4));
    }

    // REQ: R2
    #[test]
    fn test_dropwhile_keeps_all_when_pred_false_from_start() {
        // dropwhile(x < 3, [5,6,7]) == [5,6,7]
        let lst = make_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(6),
            MbValue::from_int(7),
        ]);
        let pred = make_pred_lt3();
        let result = mb_itertools_dropwhile(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_int(), Some(5));
    }

    // REQ: R2
    #[test]
    fn test_filterfalse_keeps_even_numbers() {
        // filterfalse(is_odd, [0,1,2,3,4]) == [0,2,4]
        let lst = make_list(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let pred = make_pred_is_odd();
        let result = mb_itertools_filterfalse(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_int(), Some(0));
        assert_eq!(items[1].as_int(), Some(2));
        assert_eq!(items[2].as_int(), Some(4));
    }

    // REQ: R2
    #[test]
    fn test_filterfalse_empty_when_all_true() {
        // filterfalse(is_odd, [1,3,5]) == []
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(3),
            MbValue::from_int(5),
        ]);
        let pred = make_pred_is_odd();
        let result = mb_itertools_filterfalse(pred, lst);
        let items = extract_list(result);
        assert_eq!(items.len(), 0);
    }

    // -- count tests (Wave-7 ship #2, #1265 Task #75) --

    #[test]
    fn test_count_default_int_step1() {
        // count(0, 1, 5) == [0, 1, 2, 3, 4]
        let r = mb_itertools_count(
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(5),
        );
        let items = extract_list(r);
        assert_eq!(items.len(), 5);
        assert_eq!(items[0].as_int(), Some(0));
        assert_eq!(items[4].as_int(), Some(4));
    }

    #[test]
    fn test_count_custom_start_step() {
        // count(10, 3, 4) == [10, 13, 16, 19]
        let r = mb_itertools_count(
            MbValue::from_int(10),
            MbValue::from_int(3),
            MbValue::from_int(4),
        );
        let items = extract_list(r);
        assert_eq!(
            items
                .iter()
                .map(|v| v.as_int().unwrap())
                .collect::<Vec<_>>(),
            vec![10, 13, 16, 19]
        );
    }

    #[test]
    fn test_count_negative_step() {
        // count(5, -1, 6) == [5, 4, 3, 2, 1, 0]
        let r = mb_itertools_count(
            MbValue::from_int(5),
            MbValue::from_int(-1),
            MbValue::from_int(6),
        );
        let items = extract_list(r);
        assert_eq!(
            items
                .iter()
                .map(|v| v.as_int().unwrap())
                .collect::<Vec<_>>(),
            vec![5, 4, 3, 2, 1, 0]
        );
    }

    #[test]
    fn test_count_float_promotion() {
        // count(0.0, 0.5, 4) == [0.0, 0.5, 1.0, 1.5]
        let r = mb_itertools_count(
            MbValue::from_float(0.0),
            MbValue::from_float(0.5),
            MbValue::from_int(4),
        );
        let items = extract_list(r);
        assert_eq!(items.len(), 4);
        assert_eq!(items[2].as_float(), Some(1.0));
        assert_eq!(items[3].as_float(), Some(1.5));
    }

    #[test]
    fn test_count_missing_limit_is_lazy_infinite() {
        // CPython 3.12: `itertools.count(0, 1)` with no bound is a LAZY
        // INFINITE iterator, not an empty sequence. mamba returns an iterator
        // handle (int id) that next() drives forever; verify a bounded prefix
        // 0,1,2,... . (Materializing it via extract_list would loop forever by
        // design, which is why this no longer asserts len()==0.)
        let r = mb_itertools_count(MbValue::from_int(0), MbValue::from_int(1), MbValue::none());
        assert!(
            r.as_int().is_some(),
            "count() with no bound must be a lazy iterator handle"
        );
        for expected in 0..10i64 {
            assert_eq!(
                super::super::super::iter::mb_next(r).as_int(),
                Some(expected)
            );
        }
    }

    #[test]
    fn test_count_zero_limit() {
        let r = mb_itertools_count(
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(0),
        );
        assert_eq!(extract_list(r).len(), 0);
    }

    // -- cycle tests --

    #[test]
    fn test_cycle_basic() {
        // cycle([1, 2, 3], 2) == [1, 2, 3, 1, 2, 3]
        let lst = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let r = mb_itertools_cycle(lst, MbValue::from_int(2));
        let items = extract_list(r);
        assert_eq!(
            items
                .iter()
                .map(|v| v.as_int().unwrap())
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 1, 2, 3]
        );
    }

    #[test]
    fn test_cycle_zero_cycles_returns_empty() {
        let lst = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let r = mb_itertools_cycle(lst, MbValue::from_int(0));
        assert_eq!(extract_list(r).len(), 0);
    }

    #[test]
    fn test_cycle_missing_n_is_lazy_infinite() {
        // CPython 3.12: `itertools.cycle([1, 2])` with no bound is a LAZY
        // INFINITE iterator that re-yields the cached pass forever, not an
        // empty sequence. mamba returns an iterator handle (int id); verify a
        // bounded prefix repeats 1,2,1,2,... .
        let lst = make_list(vec![MbValue::from_int(1), MbValue::from_int(2)]);
        let r = mb_itertools_cycle(lst, MbValue::none());
        assert!(
            r.as_int().is_some(),
            "cycle() with no bound must be a lazy iterator handle"
        );
        for i in 0..10usize {
            let expected = if i % 2 == 0 { 1 } else { 2 };
            assert_eq!(
                super::super::super::iter::mb_next(r).as_int(),
                Some(expected)
            );
        }
    }

    #[test]
    fn test_cycle_empty_iterable() {
        // cycle([], n) == [] regardless of n.
        let lst = make_list(vec![]);
        let r = mb_itertools_cycle(lst, MbValue::from_int(100));
        assert_eq!(extract_list(r).len(), 0);
    }

    #[test]
    fn test_cycle_single_element() {
        let lst = make_list(vec![MbValue::from_int(7)]);
        let r = mb_itertools_cycle(lst, MbValue::from_int(3));
        let items = extract_list(r);
        assert_eq!(
            items
                .iter()
                .map(|v| v.as_int().unwrap())
                .collect::<Vec<_>>(),
            vec![7, 7, 7]
        );
    }
}
