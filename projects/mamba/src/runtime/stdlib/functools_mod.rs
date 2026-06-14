/// functools module for Mamba (#393).
///
/// Provides: reduce, partial, lru_cache, cache, total_ordering, wraps,
///   cached_property, cmp_to_key, update_wrapper, singledispatch,
///   singledispatchmethod.
/// Note: reduce and partial are stubs — full function-call dispatch
/// is not yet wired in. lru_cache is an identity passthrough.
/// cached_property, singledispatch, singledispatchmethod are MVP stubs.

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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

// ── Dispatch wrappers ──

#[allow(dead_code)]
fn extract_list(val: MbValue) -> Vec<MbValue> {
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

// Native ABI: functools.X(*args) lowers to `mb_call_spread(func, args_list)`
// which invokes native extern functions with (args_ptr, nargs). Using this
// convention means arg count is variadic and signature mismatches are avoided.
unsafe extern "C" fn dispatch_reduce(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_reduce(
        args.get(0).copied().unwrap_or_else(MbValue::none),
        args.get(1).copied().unwrap_or_else(MbValue::none),
        args.get(2).copied().unwrap_or_else(MbValue::none),
    )
}

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_callable_value(v: MbValue) -> bool {
    super::super::builtins::mb_callable(v).as_bool() == Some(true)
}

unsafe extern "C" fn dispatch_partial(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Trailing dict (if present) carries the construction-time keyword args.
    let (positional, kwargs) = split_kwargs(args);
    if positional.is_empty() {
        return raise_exc(
            "TypeError",
            "type 'functools.partial' takes at least one argument",
        );
    }
    let func = positional.first().copied().unwrap_or_else(MbValue::none);
    if !is_callable_value(func) {
        return raise_exc("TypeError", "the first argument must be callable");
    }
    let bound: Vec<MbValue> = positional.iter().skip(1).copied().collect();
    let keywords = kwargs.unwrap_or_else(|| MbValue::from_ptr(MbObject::new_dict()));
    build_partial(func, bound, keywords)
}

/// Build a `functools.partial` Instance. Flattens nested partials: when `func`
/// is itself a partial, CPython concatenates the args (outer-partial args first
/// then the new bound args) and merges keywords (new keywords win). The stored
/// `args` field is a tuple and `keywords` is a dict — matching CPython's
/// introspection surface.
fn build_partial(mut func: MbValue, mut bound: Vec<MbValue>, keywords: MbValue) -> MbValue {
    // Flatten a single level of partial nesting (CPython's __new__ does this).
    if let Some(fp) = func.as_ptr() {
        let nested = unsafe {
            if let ObjData::Instance { ref class_name, ref fields } = (*fp).data {
                if class_name == "functools.partial" {
                    let f = fields.read().unwrap();
                    Some((
                        f.get("func").copied().unwrap_or_else(MbValue::none),
                        f.get("args").copied().unwrap_or_else(MbValue::none),
                        f.get("keywords").copied().unwrap_or_else(MbValue::none),
                    ))
                } else { None }
            } else { None }
        };
        if let Some((inner_func, inner_args, inner_kw)) = nested {
            func = inner_func;
            let mut merged = super::super::builtins::extract_items(inner_args);
            merged.append(&mut bound);
            bound = merged;
            // Merge keywords: inner first, then new (new wins on conflict).
            let merged_kw = merge_dicts(inner_kw, keywords);
            return make_partial_instance(func, bound, merged_kw);
        }
    }
    make_partial_instance(func, bound, keywords)
}

/// Merge two kwargs dicts into a fresh dict (`b` wins on key collisions).
fn merge_dicts(a: MbValue, b: MbValue) -> MbValue {
    let out = MbValue::from_ptr(MbObject::new_dict());
    for src in [a, b] {
        if let Some(sp) = src.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*sp).data {
                    let guard = lock.read().unwrap();
                    if let Some(op) = out.as_ptr() {
                        if let ObjData::Dict(ref olock) = (*op).data {
                            let mut og = olock.write().unwrap();
                            for (k, v) in guard.iter() {
                                super::super::rc::retain_if_ptr(*v);
                                og.insert(k.clone(), *v);
                            }
                        }
                    }
                }
            }
        }
    }
    out
}

/// Construct the `functools.partial` Instance with `func`/`args`/`keywords`.
fn make_partial_instance(func: MbValue, bound: Vec<MbValue>, keywords: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(func); }
    for v in &bound { unsafe { super::super::rc::retain_if_ptr(*v); } }
    let args_tuple = MbValue::from_ptr(MbObject::new_tuple(bound));
    let mut fields = FxHashMap::default();
    fields.insert("func".to_string(), func);
    fields.insert("args".to_string(), args_tuple);
    fields.insert("keywords".to_string(), keywords);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.partial".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

unsafe extern "C" fn dispatch_lru_cache(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    dispatch_lru_cache_impl(args)
}

unsafe extern "C" fn dispatch_cache(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // functools.cache(f) == functools.lru_cache(maxsize=None)(f) — always wraps
    // the callable with unbounded caching.
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    mb_functools_lru_cache_wrap(func, MbValue::none(), MbValue::from_bool(false))
}

/// Decide whether this call is the bare `@lru_cache` form (one callable arg)
/// or the factory form (`@lru_cache()` / `@lru_cache(maxsize=N, typed=True)`).
fn dispatch_lru_cache_impl(args: &[MbValue]) -> MbValue {
    // Split trailing kwargs dict (emitted by the method-kwargs lowering) from
    // positional args so we can read maxsize / typed keyword args.
    let (positional, kwargs) = split_kwargs(args);
    let (maxsize_kw, typed_kw) = parse_lru_kwargs(kwargs);
    // Bare `@lru_cache` form: single callable positional arg, no kwargs.
    if positional.len() == 1 && kwargs.is_none()
        && super::super::builtins::resolve_callable_pub(positional[0]).is_some()
    {
        // Default maxsize = 128 (CPython default).
        let default_max = MbValue::from_int(128);
        return mb_functools_lru_cache_wrap(positional[0], default_max, MbValue::from_bool(false));
    }
    // Factory form: parse maxsize positional or from kwargs.
    let maxsize = if let Some(kw) = maxsize_kw {
        kw
    } else if let Some(&first) = positional.first() {
        // Positional maxsize (rare, but legal: `lru_cache(128)`)
        if super::super::builtins::resolve_callable_pub(first).is_none() {
            first
        } else {
            MbValue::from_int(128)
        }
    } else {
        MbValue::from_int(128)
    };
    let typed = typed_kw.unwrap_or_else(|| MbValue::from_bool(false));
    // An explicit maxsize= keyword must be an int or None — a string like
    // "all" must NOT be resolved as the builtin callable of the same name.
    let from_kwarg = maxsize_kw.is_some();
    let maxsize_ok = maxsize.is_none()
        || maxsize.as_int().is_some()
        || maxsize.as_bool().is_some()
        || (!from_kwarg && is_callable_value(maxsize));
    if !maxsize_ok {
        return raise_exc(
            "TypeError",
            "Expected first argument to be an integer, a callable, or None",
        );
    }
    mb_functools_lru_cache_factory(maxsize, typed)
}

/// Split `args` into positional and trailing-kwargs-dict parts.
fn split_kwargs(args: &[MbValue]) -> (&[MbValue], Option<MbValue>) {
    if let Some(last) = args.last() {
        if let Some(ptr) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(_) = (*ptr).data {
                    return (&args[..args.len() - 1], Some(*last));
                }
            }
        }
    }
    (args, None)
}

/// Extract `maxsize` / `typed` from the kwargs dict if present.
fn parse_lru_kwargs(kwargs: Option<MbValue>) -> (Option<MbValue>, Option<MbValue>) {
    let Some(kw) = kwargs else { return (None, None); };
    let Some(ptr) = kw.as_ptr() else { return (None, None); };
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let guard = lock.read().unwrap();
            let max = guard.get(&super::super::dict_ops::DictKey::Str("maxsize".into())).copied();
            let typed = guard.get(&super::super::dict_ops::DictKey::Str("typed".into())).copied();
            return (max, typed);
        }
    }
    (None, None)
}

/// The four synthesized comparison methods, one per derived op. Each reads the
/// receiver class's recorded seed, calls it, and derives the target op —
/// propagating NotImplemented exactly like CPython's `_le_from_lt` family.
unsafe extern "C" fn synth_lt(self_v: MbValue, args: MbValue) -> MbValue {
    synth_compare(self_v, first_arg(args), "__lt__")
}
unsafe extern "C" fn synth_le(self_v: MbValue, args: MbValue) -> MbValue {
    synth_compare(self_v, first_arg(args), "__le__")
}
unsafe extern "C" fn synth_gt(self_v: MbValue, args: MbValue) -> MbValue {
    synth_compare(self_v, first_arg(args), "__gt__")
}
unsafe extern "C" fn synth_ge(self_v: MbValue, args: MbValue) -> MbValue {
    synth_compare(self_v, first_arg(args), "__ge__")
}

fn first_arg(args: MbValue) -> MbValue {
    // A comparison dunder is reached two ways: the binop path (`a <= b`) invokes
    // it via invoke_binop_method, which passes the RAW right-hand operand; the
    // method-call path (`a.__le__(b)`) wraps the args in a list. Accept both —
    // a List arg yields its first element, anything else IS the rhs. (Without
    // this, the binop path made `other` None, so the derived op compared the
    // seed against None — TypeError mid-derivation.)
    if let Some(p) = args.as_ptr() {
        if let ObjData::List(ref lk) = unsafe { &(*p).data } {
            return lk.read().unwrap().first().copied().unwrap_or_else(MbValue::none);
        }
    }
    args
}

unsafe extern "C" fn dispatch_total_ordering(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let cls = args.get(0).copied().unwrap_or_else(MbValue::none);
    // Resolve the class name (bare-name string from class-body decoration, or
    // a `type` object from `type("E", (), {...})`).
    let name = extract_str(cls).or_else(|| {
        cls.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                if class_name == "type" {
                    return fields.read().unwrap().get("__name__").copied().and_then(extract_str);
                }
            }
            None
        })
    });
    match name {
        Some(name) => install_total_ordering(&name, cls),
        None => cls,
    }
}

/// Apply total_ordering to a registered class: find the seed op DEFINED on the
/// class itself, install synthesized methods for the missing ops, and raise
/// only when no ordering exists anywhere in the MRO.
fn install_total_ordering(name: &str, cls: MbValue) -> MbValue {
    const OPS: [&str; 4] = ["__lt__", "__le__", "__gt__", "__ge__"];
    // Seed precedence: CPython picks the lexicographic max of the roots
    // (`__gt__` > `__ge__` > `__lt__` > `__le__`); matching that order keeps
    // single-root classes (the common case) seeded by their sole op.
    let own_seed = ["__gt__", "__ge__", "__lt__", "__le__"]
        .into_iter()
        .find(|op| super::super::class::class_defines_own_method(name, op));
    let Some(seed) = own_seed else {
        // No ordering op defined on the class itself. If it inherits one from
        // a base — a user base that defines an op, OR an ordered builtin base
        // (`class MyInt(int)`) — total_ordering is a no-op and must NOT
        // overwrite the inherited ops. Only a class with no ordering anywhere
        // is an error.
        const ORDERED_BUILTINS: [&str; 9] = [
            "int", "float", "bool", "str", "bytes", "bytearray", "tuple", "list", "frozenset",
        ];
        let inherits = super::super::class::class_mro_any(name, |c| {
            c != name
                && (ORDERED_BUILTINS.contains(&c)
                    || OPS.iter().any(|op| super::super::class::class_defines_own_method(c, op)))
        });
        if !inherits {
            return raise_exc(
                "ValueError",
                "must define at least one ordering operation: < > <= >=",
            );
        }
        return cls;
    };
    TOTAL_ORDERING_SEEDS.with(|m| m.borrow_mut().insert(name.to_string(), seed.to_string()));
    // Install synthesized methods for every op the class does NOT define
    // itself (native `(self, args)` dispatchers added to the method table).
    for op in OPS {
        if super::super::class::class_defines_own_method(name, op) {
            continue;
        }
        let addr = match op {
            "__lt__" => synth_lt as *const () as usize,
            "__le__" => synth_le as *const () as usize,
            "__gt__" => synth_gt as *const () as usize,
            _ => synth_ge as *const () as usize,
        };
        super::super::module::register_variadic_func(addr as u64);
        super::super::class::class_replace_method(name, op, MbValue::from_func(addr));
    }
    cls
}

thread_local! {
    /// Class name → the seed ordering dunder it defines, recorded by
    /// total_ordering so the synthesized methods know which op to derive from.
    static TOTAL_ORDERING_SEEDS: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Compute a synthesized comparison `target` for `self_v` against `other`,
/// deriving from the receiver class's recorded seed op. Propagates
/// NotImplemented when the seed call does, matching CPython's _ordering.
fn synth_compare(self_v: MbValue, other: MbValue, target: &str) -> MbValue {
    let class_name = self_v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    });
    let Some(class_name) = class_name else { return MbValue::not_implemented(); };
    let seed = TOTAL_ORDERING_SEEDS.with(|m| m.borrow().get(&class_name).cloned());
    let Some(seed) = seed else { return MbValue::not_implemented(); };
    let seed_m = super::super::class::lookup_method(&class_name, &seed);
    if seed_m.is_none() {
        return MbValue::not_implemented();
    }
    let mname = MbValue::from_ptr(MbObject::new_str(seed.clone()));
    let arglist = MbValue::from_ptr(MbObject::new_list(vec![other]));
    let op_result = super::super::class::mb_call_method(self_v, mname, arglist);
    if op_result.is_not_implemented() {
        return MbValue::not_implemented();
    }
    let opb = truthy(op_result);
    // eq / ne via the equality protocol (handles __eq__ + identity fallback).
    let eq = super::super::builtins::mb_eq(self_v, other).as_bool().unwrap_or(false);
    let ne = !eq;
    let derived = match (seed.as_str(), target) {
        ("__lt__", "__le__") => opb || eq,
        ("__lt__", "__gt__") => !opb && ne,
        ("__lt__", "__ge__") => !opb,
        ("__le__", "__lt__") => opb && ne,
        ("__le__", "__gt__") => !opb,
        ("__le__", "__ge__") => !opb || eq,
        ("__gt__", "__ge__") => opb || eq,
        ("__gt__", "__lt__") => !opb && ne,
        ("__gt__", "__le__") => !opb,
        ("__ge__", "__le__") => !opb || eq,
        ("__ge__", "__lt__") => !opb,
        ("__ge__", "__gt__") => opb && ne,
        _ => return MbValue::not_implemented(),
    };
    MbValue::from_bool(derived)
}

/// Legacy operator hook. total_ordering now installs real synthesized methods
/// on the class (see `install_total_ordering`), which the comparison operators
/// resolve via normal method dispatch *before* reaching this hook. Kept as a
/// no-op so the call sites in builtins.rs continue to compile.
pub fn mb_functools_total_ordering_richcmp(_a: MbValue, _b: MbValue, _op: &str) -> Option<bool> {
    None
}

/// Truthiness helper for dunder results (bool / int / float / None).
fn truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() { b }
    else if let Some(i) = v.as_int() { i != 0 }
    else if let Some(f) = v.as_float() { f != 0.0 }
    else if v.is_none() { false }
    else {
        const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
        const TAG_MASK: u64 = 0x0007_0000_0000_0000;
        let bits = v.to_bits();
        let looks_boxed = (bits & NAN_PREFIX) == NAN_PREFIX
            && bits != f64::NAN.to_bits()
            && ((bits & TAG_MASK) >> 48) <= 6;
        if looks_boxed { true } else { (bits as i64) != 0 }
    }
}

/// True when `v` is an instance of a `total_ordering`-decorated class. The
/// synthesized methods now serve the comparison operators directly, so this is
/// only consulted by the (now-redundant) operator hook.
pub fn is_total_ordering_instance(v: MbValue) -> bool {
    v.as_ptr().map(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            TOTAL_ORDERING_SEEDS.with(|s| s.borrow().contains_key(class_name))
        } else { false }
    }).unwrap_or(false)
}

unsafe extern "C" fn dispatch_wraps(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let wrapped = args.get(0).copied().unwrap_or_else(MbValue::none);
    // wraps(f) returns a `functools.wraps` Instance carrying the wrapped
    // function. When this Instance is used as a decorator (called via
    // mb_call1_val), it copies __name__ from the wrapped function to the
    // wrapper and returns the wrapper.
    let mut fields = FxHashMap::default();
    fields.insert("_wrapped".to_string(), wrapped);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.wraps".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// REQ: R6
unsafe extern "C" fn dispatch_cached_property(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_cached_property(args.get(0).copied().unwrap_or_else(MbValue::none))
}

// REQ: R7
unsafe extern "C" fn dispatch_cmp_to_key(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_cmp_to_key(args.get(0).copied().unwrap_or_else(MbValue::none))
}

// REQ: R8
unsafe extern "C" fn dispatch_update_wrapper(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let wrapper = args.get(0).copied().unwrap_or_else(MbValue::none);
    let wrapped = args.get(1).copied().unwrap_or_else(MbValue::none);
    // updated=("name",...) entries must exist on the wrapper — CPython does
    // getattr(wrapper, attr).update(...) and lets the AttributeError out.
    let (_, kwargs) = split_kwargs(args);
    if let Some(kw) = kwargs {
        if let Some(kptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*kptr).data {
                    let updated = lock
                        .read()
                        .unwrap()
                        .get(&super::super::dict_ops::DictKey::Str("updated".into()))
                        .copied();
                    if let Some(upd) = updated {
                        if let Some(uptr) = upd.as_ptr() {
                            let names: Vec<String> = match &(*uptr).data {
                                ObjData::Tuple(items) => {
                                    items.iter().filter_map(|v| extract_str(*v)).collect()
                                }
                                ObjData::List(lock) => lock
                                    .read()
                                    .unwrap()
                                    .iter()
                                    .filter_map(|v| extract_str(*v))
                                    .collect(),
                                _ => Vec::new(),
                            };
                            for name in names {
                                if name != "__dict__" {
                                    return raise_exc(
                                        "AttributeError",
                                        &format!(
                                            "'function' object has no attribute '{name}'"
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    mb_functools_update_wrapper(wrapper, wrapped)
}

// REQ: R9
unsafe extern "C" fn dispatch_singledispatch(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_singledispatch(args.get(0).copied().unwrap_or_else(MbValue::none))
}

// REQ: R10
unsafe extern "C" fn dispatch_singledispatchmethod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_singledispatchmethod(args.get(0).copied().unwrap_or_else(MbValue::none))
}

// partialmethod is the descriptor-bound sibling of `partial`. Hot path
// for `partialmethod` is one-shot at class-definition (binds func +
// fixed args to a class method); same MVP shape as `partial`.
unsafe extern "C" fn dispatch_partialmethod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    if !is_callable_value(func) {
        return raise_exc(
            "TypeError",
            "the first argument must be a callable or a descriptor",
        );
    }
    let bound: Vec<MbValue> = args.iter().skip(1).copied().collect();
    let bound_list = MbValue::from_ptr(MbObject::new_list(bound));
    let mut fields = FxHashMap::default();
    fields.insert("func".to_string(), func);
    fields.insert("args".to_string(), bound_list);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.partialmethod".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// ── recursive_repr / get_cache_token / namedtuple / GenericAlias
//    stubs (#1265 Task #77, Wave-7 ship #4) ──
//
// These four entries close the surface gap against CPython 3.12's
// `dir(functools)`. RLock is the 5th name in that gap but is
// deliberately skipped — it has nontrivial thread-lock semantics
// that can't be cleanly stubbed and is rarely accessed via the
// `functools.RLock` re-export (callers use `threading.RLock`).

/// `functools.recursive_repr(fillvalue='...')` — decorator factory.
///
/// CPython returns a decorator that, when applied to `__repr__`,
/// detects re-entrant `repr()` calls on the same object and returns
/// `fillvalue` instead of recursing. **Carve-out**: the recursion-
/// detection table requires thread-local frame plumbing that mamba
/// doesn't yet expose to stdlib code. This stub returns the user's
/// `__repr__` function unchanged — non-recursive cases work
/// transparently; recursive `__repr__` will still hit the runtime's
/// own recursion guard (a recursion-limit exception) rather than
/// the fillvalue. Tracked under #1451 conformance.
/// Stub impl — see module-level carve-out doc above.
pub fn mb_functools_recursive_repr(arg: MbValue) -> MbValue {
    // Passthrough: when used as `@recursive_repr` directly on a fn,
    // the fn is returned unchanged. When used as `@recursive_repr()`
    // (no fillvalue), the factory call returns None — the user then
    // would need to apply the result as a decorator, which our stub
    // doesn't support cleanly. The common idiom `@recursive_repr` is
    // covered; `@recursive_repr('...')` is not.
    arg
}

unsafe extern "C" fn dispatch_recursive_repr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_recursive_repr(a.first().copied().unwrap_or_else(MbValue::none))
}

/// `functools.get_cache_token()` — abc cache validity token.
///
/// CPython re-exports `abc.get_cache_token` here. The token is an
/// opaque int that monotonically increments whenever the ABC
/// registry is mutated; callers compare consecutive tokens to detect
/// invalidation. **Carve-out**: mamba has no ABC registry yet, so
/// the token never invalidates — this stub returns a constant 0.
/// Callers that compare two values for equality will (correctly)
/// see the cache as always-valid. Tracked under #1451 conformance.
pub fn mb_functools_get_cache_token() -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_get_cache_token(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_functools_get_cache_token()
}

/// `functools.namedtuple(name, fields)` — re-export of
/// `collections.namedtuple`.
unsafe extern "C" fn dispatch_namedtuple(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    super::collections_mod::mb_namedtuple(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}

/// `functools.GenericAlias(origin, args)` — generic-type alias.
///
/// **Carve-out**: stores `_origin` + `_args` for inspection; subscript
/// / indexing semantics are not wired.
pub fn mb_functools_generic_alias(origin: MbValue, type_args: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_origin".to_string(), origin);
    fields.insert("_args".to_string(), type_args);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "types.GenericAlias".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

unsafe extern "C" fn dispatch_generic_alias(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_functools_generic_alias(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

/// WRAPPER_ASSIGNMENTS — names copied by `functools.wraps` from the
/// wrapped fn onto the wrapper. CPython value:
/// `('__module__', '__name__', '__qualname__', '__annotations__',
///   '__doc__', '__type_params__')`.
fn wrapper_assignments_tuple() -> MbValue {
    let names = [
        "__module__",
        "__name__",
        "__qualname__",
        "__annotations__",
        "__doc__",
        "__type_params__",
    ];
    let items: Vec<MbValue> = names
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_tuple(items))
}

/// WRAPPER_UPDATES — single-element `('__dict__',)`.
fn wrapper_updates_tuple() -> MbValue {
    let items = vec![MbValue::from_ptr(MbObject::new_str("__dict__".to_string()))];
    MbValue::from_ptr(MbObject::new_tuple(items))
}

/// Register the functools module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Register each dispatcher as a TAG_FUNC value AND mark it native so
    // mb_call_spread uses the (args_ptr, nargs) ABI.
    let dispatchers: [(&str, usize); 16] = [
        ("reduce", dispatch_reduce as *const () as usize),
        ("partial", dispatch_partial as *const () as usize),
        ("partialmethod", dispatch_partialmethod as *const () as usize),
        ("lru_cache", dispatch_lru_cache as *const () as usize),
        ("cache", dispatch_cache as *const () as usize),
        ("total_ordering", dispatch_total_ordering as *const () as usize),
        ("wraps", dispatch_wraps as *const () as usize),
        ("cached_property", dispatch_cached_property as *const () as usize),
        ("cmp_to_key", dispatch_cmp_to_key as *const () as usize),
        ("update_wrapper", dispatch_update_wrapper as *const () as usize),
        ("singledispatch", dispatch_singledispatch as *const () as usize),
        ("singledispatchmethod", dispatch_singledispatchmethod as *const () as usize),
        ("recursive_repr", dispatch_recursive_repr as *const () as usize),
        ("get_cache_token", dispatch_get_cache_token as *const () as usize),
        ("namedtuple", dispatch_namedtuple as *const () as usize),
        ("GenericAlias", dispatch_generic_alias as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Module-level tuple constants used by `functools.wraps`.
    attrs.insert("WRAPPER_ASSIGNMENTS".to_string(), wrapper_assignments_tuple());
    attrs.insert("WRAPPER_UPDATES".to_string(), wrapper_updates_tuple());

    super::register_module("functools", attrs);
}

// ── Runtime functions ──

/// functools.reduce(func, iterable, initial) -> value
///
/// Fold `iterable` with `func`, starting from `initial`.
///
/// **Stub**: Since function-call dispatch is not yet wired into the
/// stdlib layer, this performs a basic sum-fold when `func` is the
/// string "add", and returns `initial` otherwise.
/// Legacy sequence-protocol drain for `reduce`: if `iterable` is a user
/// Instance that defines `__getitem__` but not `__iter__`, materialize it by
/// calling `__getitem__(0)`, `__getitem__(1)`, ... until an IndexError is
/// raised. Returns `None` when the object is not a __getitem__-only sequence
/// (caller falls back to the normal iterator protocol).
fn reduce_getitem_sequence(iterable: MbValue) -> Option<Vec<MbValue>> {
    let ptr = iterable.as_ptr()?;
    let class_name = unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            class_name.clone()
        } else {
            return None;
        }
    };
    // Only take this path for __getitem__-only instances. If __iter__ exists,
    // defer to the normal iterator protocol (it is the preferred path).
    if !super::super::class::lookup_method(&class_name, "__iter__").is_none() {
        return None;
    }
    if super::super::class::lookup_method(&class_name, "__getitem__").is_none() {
        return None;
    }
    let mut out = Vec::new();
    let mut i: i64 = 0;
    loop {
        let item = super::super::class::mb_obj_getitem(iterable, MbValue::from_int(i));
        // __getitem__ signalled the end of the sequence by raising IndexError.
        if let Some(ty) = super::super::exception::current_exception_type() {
            if ty == "IndexError" || ty == "StopIteration" {
                super::super::exception::mb_clear_exception();
                break;
            }
            // A different, genuine exception — propagate it.
            return Some(out);
        }
        out.push(item);
        i += 1;
        // Defensive bound to avoid runaway loops on a misbehaving sequence.
        if i > 100_000_000 {
            break;
        }
    }
    Some(out)
}

pub fn mb_functools_reduce(
    func: MbValue,
    iterable: MbValue,
    initial: MbValue,
) -> MbValue {
    // Materialize iterable. CPython's reduce uses PyObject_GetIter, which
    // accepts the legacy sequence protocol: an object with __getitem__ (and no
    // __iter__) is iterated by calling __getitem__(0), __getitem__(1), ... until
    // IndexError. mamba's mb_iter only honors __iter__, so detect the
    // __getitem__-only case here and drive it manually.
    let items: Vec<MbValue> = if let Some(seq) = reduce_getitem_sequence(iterable) {
        seq
    } else {
        let iter_handle = super::super::iter::mb_iter(iterable);
        if iter_handle.is_none() {
            return initial;
        }
        let mut acc = Vec::new();
        loop {
            if super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
                break;
            }
            let item = super::super::iter::mb_next(iter_handle);
            if item.is_none() && super::super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
                break;
            }
            acc.push(item);
        }
        acc
    };

    // Determine initial accumulator. CPython: with no initializer and an empty
    // iterable, raise TypeError; with a single element and no initializer, that
    // element is returned without ever calling func.
    let (mut acc, start) = if initial.is_none() {
        if items.is_empty() {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "reduce() of empty iterable with no initial value".to_string(),
                )),
            );
            return MbValue::none();
        }
        (items[0], 1)
    } else {
        (initial, 0)
    };
    // Short-circuit: nothing left to fold (single element / empty+initial) →
    // return the accumulator without resolving or calling func at all. This
    // matches CPython even when `func` is not callable (e.g. reduce(42, "1")).
    if start >= items.len() {
        unsafe { super::super::rc::retain_if_ptr(acc); }
        return acc;
    }

    // Try to resolve func as a callable (lambda, function, closure).
    if let Some(raw_addr) = super::super::builtins::resolve_callable_pub(func) {
        // Fast path (mitigation 3A for #2100): when `func` is a native
        // extern "C" fn(*const MbValue, usize) -> MbValue, call it directly
        // with stack-allocated `[MbValue; 2]`. Skips per-iter
        // `MbObject::new_list(vec![acc, *item])` allocation — that List is
        // cycle-capable and dominates GC pressure in callback-bound loops
        // (operator.add/mul/max/min, math.* unaries, etc.).
        if super::super::module::is_native_func(raw_addr as u64) {
            let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                unsafe { std::mem::transmute(raw_addr) };
            for item in &items[start..] {
                let args = [acc, *item];
                acc = unsafe { f(args.as_ptr(), 2) };
            }
            return acc;
        }

        // Slow path: closure / lambda / Python-defined fn. Use mb_call_spread
        // so JIT raw-i64 returns are re-boxed correctly (CheckedAdd returns
        // unboxed i64 for perf).
        for item in &items[start..] {
            let pair = MbValue::from_ptr(MbObject::new_list(vec![acc, *item]));
            acc = super::super::builtins::mb_call_spread(func, pair);
        }
        return acc;
    }

    // Fallback: known builtin string like "add"/"mul"
    let func_name = extract_str(func);
    match func_name.as_deref() {
        Some("add") => reduce_add(&items, initial),
        Some("mul") => reduce_mul(&items, initial),
        _ => initial,
    }
}

/// Built-in reduce: addition fold.
fn reduce_add(items: &[MbValue], initial: MbValue) -> MbValue {
    let mut acc = initial.as_int().unwrap_or(0);
    for item in items {
        if let Some(v) = item.as_int() {
            acc += v;
        } else if let Some(f) = item.as_float() {
            // Switch to float accumulation
            let mut facc = acc as f64 + f;
            for remaining in &items[items.len()..] {
                if let Some(v2) = remaining.as_int() {
                    facc += v2 as f64;
                } else if let Some(f2) = remaining.as_float() {
                    facc += f2;
                }
            }
            return MbValue::from_float(facc);
        }
    }
    MbValue::from_int(acc)
}

/// Built-in reduce: multiplication fold.
fn reduce_mul(items: &[MbValue], initial: MbValue) -> MbValue {
    let mut acc = initial.as_int().unwrap_or(1);
    for item in items {
        if let Some(v) = item.as_int() {
            acc *= v;
        }
    }
    MbValue::from_int(acc)
}

/// functools.partial(func, *args) -> callable
///
/// Returns an Instance-backed partial application. `func` is the underlying
/// callable, `args` is a list of already-bound positional arguments. The
/// runtime's dynamic-dispatch path (`mb_call_spread`) detects the
/// `functools.partial` class name and prepends the bound args to the call.
pub fn mb_functools_partial(
    func: MbValue,
    args: MbValue,
) -> MbValue {
    // Normalize `args` to a list of bound positional args. When called via
    // the native dispatch (`*args` spread), `args` is the caller's list.
    // When called via `mb_functools_partial(f, x)` directly (the 2-arg form),
    // wrap the single value as a one-element list.
    let bound = if args.as_ptr().map(|p| unsafe {
        matches!(&(*p).data, ObjData::List(_))
    }).unwrap_or(false) {
        args
    } else {
        MbValue::from_ptr(MbObject::new_list(vec![args]))
    };
    let mut fields = FxHashMap::default();
    fields.insert("func".to_string(), func);
    fields.insert("args".to_string(), bound);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.partial".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// repr(functools.partial(...)) →
/// `functools.partial(<func>, <pos>..., <key>=<val>...)`, matching CPython's
/// `partial.__repr__`. Positional args and keyword values are rendered with
/// `repr()`; keyword keys are bare identifiers.
pub fn mb_functools_partial_repr(partial: MbValue) -> MbValue {
    let Some(ptr) = partial.as_ptr() else {
        return MbValue::from_ptr(MbObject::new_str("functools.partial()".to_string()));
    };
    let (func, args_val, kw_val) = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            (
                f.get("func").copied().unwrap_or_else(MbValue::none),
                f.get("args").copied().unwrap_or_else(MbValue::none),
                f.get("keywords").copied().unwrap_or_else(MbValue::none),
            )
        } else {
            return MbValue::from_ptr(MbObject::new_str("functools.partial()".to_string()));
        }
    };
    let repr_of = |v: MbValue| -> String {
        let r = super::super::builtins::mb_repr(v);
        extract_str(r).unwrap_or_else(|| "None".to_string())
    };
    let mut parts: Vec<String> = Vec::new();
    parts.push(repr_of(func));
    for a in super::super::builtins::extract_items(args_val) {
        parts.push(repr_of(a));
    }
    if let Some(kp) = kw_val.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*kp).data {
                let guard = lock.read().unwrap();
                for (k, v) in guard.iter() {
                    if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                        parts.push(format!("{}={}", ks, repr_of(*v)));
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(format!(
        "functools.partial({})",
        parts.join(", ")
    )))
}

/// functools.lru_cache(func) -> wrapper  (legacy 1-arg entry point)
///
/// Wraps `func` with a default-capacity (maxsize=128) LRU cache.
pub fn mb_functools_lru_cache(func: MbValue) -> MbValue {
    mb_functools_lru_cache_wrap(func, MbValue::from_int(128), MbValue::from_bool(false))
}

/// Build the lru_cache *factory* instance (returned when called with kwargs or
/// empty parens). When subsequently called with a function, mb_call_spread
/// detects this class and produces the real wrapper.
pub fn mb_functools_lru_cache_factory(maxsize: MbValue, typed: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_maxsize".to_string(), maxsize);
    fields.insert("_typed".to_string(), typed);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.lru_cache_factory".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Build the lru_cache wrapper instance that `mb_call_spread` routes through.
pub fn mb_functools_lru_cache_wrap(func: MbValue, maxsize: MbValue, typed: MbValue) -> MbValue {
    // Retain the wrapped callable so the wrapper owns its own rc.
    unsafe { super::super::rc::retain_if_ptr(func); }
    let mut fields = FxHashMap::default();
    fields.insert("_func".to_string(), func);
    fields.insert("_maxsize".to_string(), maxsize);
    fields.insert("_typed".to_string(), typed);
    fields.insert("_cache".to_string(),
        MbValue::from_ptr(super::super::rc::MbObject::new_dict()));
    fields.insert("_order".to_string(),
        MbValue::from_ptr(super::super::rc::MbObject::new_list(Vec::new())));
    fields.insert("_hits".to_string(), MbValue::from_int(0));
    fields.insert("_misses".to_string(), MbValue::from_int(0));
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.lru_cache_wrapper".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Internal: build a stable cache key string from positional args. Uses the
/// DictKey's Display impl for each arg so primitives (int, str, bool, None)
/// and Instance values with defined __hash__ all produce deterministic keys.
/// Honor `typed` by mixing each value's type name into the key so `f(1)` and
/// `f(1.0)` live in separate cache slots. Returns a `DictKey::Str` directly
/// so callers don't need another `to_dict_key` round-trip.
fn lru_build_cache_key_dk(items: &[MbValue], typed: bool) -> super::super::dict_ops::DictKey {
    use super::super::dict_ops::DictKey;
    let mut out = String::new();
    out.push('(');
    for (i, v) in items.iter().enumerate() {
        if i > 0 { out.push('|'); }
        if typed {
            // Tag each arg with its type name so f(1) / f(1.0) separate.
            if v.is_int() { out.push_str("i:"); }
            else if v.is_float() { out.push_str("f:"); }
            else if v.is_bool() { out.push_str("b:"); }
            else if v.is_none() { out.push_str("n:"); }
            else if let Some(p) = v.as_ptr() {
                let tag: String = unsafe {
                    match &(*p).data {
                        ObjData::Str(_) => "s:".to_string(),
                        ObjData::List(_) => "l:".to_string(),
                        ObjData::Dict(_) => "d:".to_string(),
                        ObjData::Tuple(_) => "t:".to_string(),
                        ObjData::Set(_) => "e:".to_string(),
                        ObjData::FrozenSet(_) => "fs:".to_string(),
                        ObjData::Instance { class_name, .. } => {
                            format!("{class_name}:")
                        }
                        _ => "?:".to_string(),
                    }
                };
                out.push_str(&tag);
            }
        }
        lru_encode_value_key(*v, &mut out);
    }
    out.push(')');
    DictKey::Str(out)
}

/// Append a value-faithful encoding of `v` into `out`. Unlike a bare
/// `hash_val` stringification, this folds container *element* keys so that
/// two distinct args that merely share a hash never collide into the same
/// cache slot. The encoding mirrors `DictKey`'s eq semantics:
///   * Tuple — ordered fold of element keys (`mb_tuple_eq` is element-wise).
///   * FrozenSet — order-independent fold (element keys sorted), so two
///     frozensets that compare equal (any build order) produce the same key,
///     and unequal frozensets with a colliding hash produce distinct keys.
///   * Instance — hash plus pointer identity, so distinct instances with an
///     equal `__hash__` get distinct keys. (Two `__eq__`-equal-but-distinct
///     instances over-separate into different slots, which is a benign cache
///     miss — never a wrong hit.)
/// Recursion is depth-bounded implicitly by the value tree; nested containers
/// fold recursively.
fn lru_encode_value_key(v: MbValue, out: &mut String) {
    use super::super::dict_ops::{to_dict_key, DictKey};
    let dk = to_dict_key(v);
    match &dk {
        DictKey::Int(i) => out.push_str(&i.to_string()),
        DictKey::Float(bits) => { out.push('f'); out.push_str(&bits.to_string()); }
        DictKey::Str(s) => { out.push('"'); out.push_str(s); out.push('"'); }
        DictKey::Bool(b) => out.push_str(if *b { "T" } else { "F" }),
        DictKey::None => out.push_str("None"),
        DictKey::Instance { hash_val, ptr, .. } => {
            // Hash buckets, pointer identity disambiguates collisions. We
            // cannot fully encode user `__eq__` into a string, so we lean on
            // identity: never a wrong hit, at worst an extra miss.
            out.push('@');
            out.push_str(&hash_val.to_string());
            out.push('#');
            out.push_str(&format!("{ptr:x}"));
        }
        DictKey::Other(s) => out.push_str(s),
        DictKey::Func(addr) => {
            out.push('@');
            out.push_str(&format!("{addr:x}"));
        }
        DictKey::Tuple { .. } => {
            // Ordered fold of element keys — matches mb_tuple_eq.
            out.push('T');
            out.push('[');
            if let Some(p) = v.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(ref items) = (*p).data {
                        for (i, el) in items.iter().enumerate() {
                            if i > 0 { out.push(','); }
                            lru_encode_value_key(*el, out);
                        }
                    }
                }
            }
            out.push(']');
        }
        DictKey::FrozenSet { hash_val, .. } => {
            // Order-independent fold: encode each element, sort, then join so
            // equal frozensets (any build order) match and unequal ones with a
            // colliding hash stay distinct.
            out.push('S');
            out.push('{');
            let mut parts: Vec<String> = Vec::new();
            if let Some(p) = v.as_ptr() {
                unsafe {
                    if let ObjData::FrozenSet(ref items) = (*p).data {
                        for el in items.iter() {
                            let mut s = String::new();
                            lru_encode_value_key(*el, &mut s);
                            parts.push(s);
                        }
                    }
                }
            }
            parts.sort();
            // Fall back to mixing in the content hash so two frozensets whose
            // element encodings happen to sort identically (they shouldn't for
            // distinct sets) still carry their structural hash signal.
            out.push_str(&parts.join(","));
            out.push(';');
            out.push_str(&hash_val.to_string());
            out.push('}');
        }
    }
}

/// Called by `mb_call_spread` when `func` is a `functools.lru_cache_wrapper`
/// Instance. Looks up the cache by args, invokes the inner func on miss,
/// honors maxsize (0 = disabled, None = unbounded, positive = LRU).
pub fn mb_functools_lru_cache_invoke(wrapper: MbValue, items: Vec<MbValue>) -> MbValue {
    use super::super::rc::{MbObject, ObjData};
    let Some(ptr) = wrapper.as_ptr() else { return MbValue::none(); };
    // Fetch cache fields under a read lock, cloned so we can drop the lock
    // before calling the inner function (which might reentrantly invoke us).
    let (inner_func, maxsize_v, typed_v, cache_val, order_val) = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            (
                f.get("_func").copied().unwrap_or_else(MbValue::none),
                f.get("_maxsize").copied().unwrap_or_else(MbValue::none),
                f.get("_typed").copied().unwrap_or_else(MbValue::none),
                f.get("_cache").copied().unwrap_or_else(MbValue::none),
                f.get("_order").copied().unwrap_or_else(MbValue::none),
            )
        } else {
            return MbValue::none();
        }
    };
    let typed = typed_v.as_bool().unwrap_or(false);
    let maxsize_opt: Option<i64> = maxsize_v.as_int();
    // maxsize == 0 → caching disabled, always call through.
    if matches!(maxsize_opt, Some(0)) {
        bump_lru_counter(wrapper, "_misses");
        let args_list = MbValue::from_ptr(MbObject::new_list(items));
        return super::super::builtins::mb_call_spread(inner_func, args_list);
    }
    // Compute cache key.
    let key_dk = lru_build_cache_key_dk(&items, typed);
    // Cache lookup.
    if let Some(cache_ptr) = cache_val.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*cache_ptr).data {
                if let Some(&hit) = lock.read().unwrap().get(&key_dk) {
                    super::super::rc::retain_if_ptr(hit);
                    bump_lru_counter(wrapper, "_hits");
                    // Touch the order list: move key to the end (LRU).
                    lru_touch_order(order_val, &key_dk);
                    return hit;
                }
            }
        }
    }
    // Miss — call inner, store result.
    bump_lru_counter(wrapper, "_misses");
    let args_list = MbValue::from_ptr(MbObject::new_list(items));
    let result = super::super::builtins::mb_call_spread(inner_func, args_list);
    // Insert into cache.
    if let Some(cache_ptr) = cache_val.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*cache_ptr).data {
                let mut guard = lock.write().unwrap();
                super::super::rc::retain_if_ptr(result);
                guard.insert(key_dk.clone(), result);
            }
        }
    }
    // Append to order list.
    if let Some(order_ptr) = order_val.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*order_ptr).data {
                let mut guard = lock.write().unwrap();
                // Use the tuple-key MbValue (already built) as the order entry;
                // re-wrap for uniqueness.
                guard.push(dict_key_to_stable_mbvalue(&key_dk));
            }
        }
    }
    // Evict if over maxsize.
    if let Some(limit) = maxsize_opt {
        if limit > 0 {
            lru_evict_to_limit(cache_val, order_val, limit as usize);
        }
    }
    result
}

/// Move an entry to the end of the LRU order list (mark as most recent).
fn lru_touch_order(order_val: MbValue, key_dk: &super::super::dict_ops::DictKey) {
    let key_str = match key_dk {
        super::super::dict_ops::DictKey::Str(s) => s.clone(),
        _ => return,
    };
    if let Some(ptr) = order_val.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let mut guard = lock.write().unwrap();
                let pos = guard.iter().position(|v| {
                    v.as_ptr().map(|p| {
                        if let ObjData::Str(ref s) = (*p).data {
                            s == &key_str
                        } else { false }
                    }).unwrap_or(false)
                });
                if let Some(i) = pos {
                    let entry = guard.remove(i);
                    guard.push(entry);
                }
            }
        }
    }
}

/// Evict oldest entries from the cache until length <= limit.
fn lru_evict_to_limit(cache_val: MbValue, order_val: MbValue, limit: usize) {
    let Some(cache_ptr) = cache_val.as_ptr() else { return; };
    let Some(order_ptr) = order_val.as_ptr() else { return; };
    unsafe {
        if let (ObjData::Dict(ref cache_lock), ObjData::List(ref order_lock)) =
            (&(*cache_ptr).data, &(*order_ptr).data)
        {
            loop {
                let len = cache_lock.read().unwrap().len();
                if len <= limit { break; }
                let oldest = {
                    let mut order = order_lock.write().unwrap();
                    if order.is_empty() { break; }
                    order.remove(0)
                };
                let oldest_key = super::super::dict_ops::to_dict_key(oldest);
                let mut cache = cache_lock.write().unwrap();
                if let Some(prev) = cache.shift_remove(&oldest_key) {
                    super::super::rc::release_if_ptr(prev);
                }
            }
        }
    }
}

/// Round-trip a DictKey back through the tuple form for the order list.
/// For Int/Str/Bool keys this emits a matching MbValue; for Instance/Other
/// we fall back to re-constructing from the Instance pointer stored inside.
fn dict_key_to_stable_mbvalue(key: &super::super::dict_ops::DictKey) -> MbValue {
    super::super::dict_ops::dict_key_to_mbvalue(key)
}

/// Increment a counter field ("_hits" or "_misses") on the wrapper.
fn bump_lru_counter(wrapper: MbValue, field: &str) {
    let Some(ptr) = wrapper.as_ptr() else { return; };
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let mut f = fields.write().unwrap();
            let cur = f.get(field).and_then(|v| v.as_int()).unwrap_or(0);
            f.insert(field.to_string(), MbValue::from_int(cur + 1));
        }
    }
}

/// Called by `mb_call_spread` when `func` is the factory Instance produced by
/// `@lru_cache(maxsize=N)` form. The single arg is the wrapped callable.
pub fn mb_functools_lru_cache_factory_apply(factory: MbValue, items: Vec<MbValue>) -> MbValue {
    let Some(ptr) = factory.as_ptr() else { return MbValue::none(); };
    let (maxsize, typed) = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            (
                f.get("_maxsize").copied().unwrap_or_else(|| MbValue::from_int(128)),
                f.get("_typed").copied().unwrap_or_else(|| MbValue::from_bool(false)),
            )
        } else {
            return MbValue::none();
        }
    };
    let func = items.first().copied().unwrap_or_else(MbValue::none);
    mb_functools_lru_cache_wrap(func, maxsize, typed)
}

/// cache_info() → (hits, misses, maxsize, currsize)
/// Returns a 4-tuple. Displayed via `__repr__` as CacheInfo(...), but the
/// attribute access .hits / .misses / .maxsize / .currsize must also work —
/// so we ship it as an Instance with those fields.
pub fn mb_functools_lru_cache_info(wrapper: MbValue) -> MbValue {
    use super::super::rc::{MbObject, ObjData};
    let Some(ptr) = wrapper.as_ptr() else { return MbValue::none(); };
    let (hits, misses, maxsize, currsize) = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            let h = f.get("_hits").copied().unwrap_or_else(|| MbValue::from_int(0));
            let m = f.get("_misses").copied().unwrap_or_else(|| MbValue::from_int(0));
            let ms = f.get("_maxsize").copied().unwrap_or_else(MbValue::none);
            let c = f.get("_cache").copied().unwrap_or_else(MbValue::none);
            let sz = c.as_ptr().map(|cp| {
                if let ObjData::Dict(ref lock) = (*cp).data {
                    lock.read().unwrap().len() as i64
                } else { 0 }
            }).unwrap_or(0);
            (h, m, ms, MbValue::from_int(sz))
        } else {
            return MbValue::none();
        }
    };
    let mut fields = FxHashMap::default();
    fields.insert("hits".to_string(), hits);
    fields.insert("misses".to_string(), misses);
    fields.insert("maxsize".to_string(), maxsize);
    fields.insert("currsize".to_string(), currsize);
    let obj = Box::new(MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.CacheInfo".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// cache_clear() — reset counters and empty the cache dict + order list.
pub fn mb_functools_lru_cache_clear(wrapper: MbValue) -> MbValue {
    let Some(ptr) = wrapper.as_ptr() else { return MbValue::none(); };
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let mut f = fields.write().unwrap();
            f.insert("_hits".to_string(), MbValue::from_int(0));
            f.insert("_misses".to_string(), MbValue::from_int(0));
            if let Some(cache_ptr) = f.get("_cache").and_then(|v| v.as_ptr()) {
                if let ObjData::Dict(ref lock) = (*cache_ptr).data {
                    let mut guard = lock.write().unwrap();
                    for (_, v) in guard.drain(..) {
                        super::super::rc::release_if_ptr(v);
                    }
                }
            }
            if let Some(order_ptr) = f.get("_order").and_then(|v| v.as_ptr()) {
                if let ObjData::List(ref lock) = (*order_ptr).data {
                    lock.write().unwrap().clear();
                }
            }
        }
    }
    MbValue::none()
}

/// functools.cached_property(func) -> func  (R6)
///
/// MVP: identity passthrough. The descriptor protocol is not yet wired.
/// Returns the function unchanged so that `from functools import cached_property`
/// does not crash.
pub fn mb_functools_cached_property(func: MbValue) -> MbValue {
    func
}

/// functools.cmp_to_key(mycmp) -> key-factory Instance  (R7)
///
/// Returns a `functools.cmp_to_key` factory Instance carrying the comparison
/// callable in `_cmp`. Calling the factory with a value produces a key object
/// (`functools.cmp_to_key_obj`) whose rich-comparison operators delegate to the
/// stored cmp. The factory dispatch is wired in `mb_call_spread`; the key
/// comparisons are wired in the runtime comparison operators.
pub fn mb_functools_cmp_to_key(mycmp: MbValue) -> MbValue {
    unsafe { super::super::rc::retain_if_ptr(mycmp); }
    let mut fields = FxHashMap::default();
    fields.insert("_cmp".to_string(), mycmp);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.cmp_to_key".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Apply a `functools.cmp_to_key` factory to a value: build a key object that
/// wraps `obj` and carries the comparison callable. Called by `mb_call_spread`
/// when the factory Instance is invoked.
pub fn mb_functools_cmp_to_key_apply(factory: MbValue, items: Vec<MbValue>) -> MbValue {
    let cmp = factory.as_ptr().map(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().unwrap().get("_cmp").copied().unwrap_or_else(MbValue::none)
        } else {
            MbValue::none()
        }
    }).unwrap_or_else(MbValue::none);
    let obj_val = items.first().copied().unwrap_or_else(MbValue::none);
    unsafe {
        super::super::rc::retain_if_ptr(cmp);
        super::super::rc::retain_if_ptr(obj_val);
    }
    let mut fields = FxHashMap::default();
    fields.insert("_cmp".to_string(), cmp);
    fields.insert("obj".to_string(), obj_val);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.cmp_to_key_obj".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Run the stored cmp on two key objects' wrapped values, returning the sign
/// (`< 0`, `0`, `> 0`) as an i64. Used by every rich-comparison operator.
fn cmp_to_key_compare(a: MbValue, b: MbValue) -> Option<i64> {
    let (cmp, lhs) = a.as_ptr().map(|p| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*p).data {
            if class_name == "functools.cmp_to_key_obj" {
                let f = fields.read().unwrap();
                (
                    Some(f.get("_cmp").copied().unwrap_or_else(MbValue::none)),
                    f.get("obj").copied().unwrap_or_else(MbValue::none),
                )
            } else { (None, MbValue::none()) }
        } else { (None, MbValue::none()) }
    }).unwrap_or((None, MbValue::none()));
    let cmp = cmp?;
    let rhs = b.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*p).data {
            if class_name == "functools.cmp_to_key_obj" {
                Some(fields.read().unwrap().get("obj").copied().unwrap_or_else(MbValue::none))
            } else { None }
        } else { None }
    })?;
    let pair = MbValue::from_ptr(MbObject::new_list(vec![lhs, rhs]));
    let result = super::super::builtins::mb_call_spread(cmp, pair);
    Some(cmp_result_to_sign(result))
}

/// Coerce a comparison callable's return value to a CPython-style sign int.
///
/// A Python-defined cmp whose body computes the sign with bool subtraction
/// (`(x>y) - (x<y)`) is JIT-compiled to return a *raw, unboxed* i64 rather
/// than a NaN-boxed MbValue. When that raw value is negative (e.g. -1) every
/// high bit is set, which is bit-indistinguishable from a NaN-boxed value with
/// the otherwise-unused tag 7 — so `mb_call_spread`'s rebox guard forwards it
/// untouched and the standard `as_int()` decode fails. Detect that case (an
/// invalid tag) and recover the raw i64; otherwise decode normally.
fn cmp_result_to_sign(result: MbValue) -> i64 {
    if let Some(i) = result.as_int() {
        i
    } else if let Some(f) = result.as_float() {
        if f < 0.0 { -1 } else if f > 0.0 { 1 } else { 0 }
    } else if let Some(bv) = result.as_bool() {
        if bv { 1 } else { 0 }
    } else if result.is_none() {
        0
    } else {
        const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
        const TAG_MASK: u64 = 0x0007_0000_0000_0000;
        let bits = result.to_bits();
        let looks_boxed = (bits & NAN_PREFIX) == NAN_PREFIX
            && bits != f64::NAN.to_bits()
            && ((bits & TAG_MASK) >> 48) <= 6; // valid tags are 0..=6
        if looks_boxed {
            // A genuine boxed value we don't otherwise handle — treat as 0.
            0
        } else {
            // Raw unboxed i64 returned by a JIT-compiled int function.
            bits as i64
        }
    }
}

/// True when `v` is a `functools.cmp_to_key_obj` key wrapper.
pub fn is_cmp_to_key_obj(v: MbValue) -> bool {
    v.as_ptr().map(|p| unsafe {
        matches!(&(*p).data, ObjData::Instance { class_name, .. }
            if class_name == "functools.cmp_to_key_obj")
    }).unwrap_or(false)
}

/// Rich-comparison entry for cmp_to_key key objects. `op` is one of
/// "lt","le","gt","ge","eq","ne". Returns the boolean result, or `None` when
/// `a` is not a key object (caller falls back to default comparison).
pub fn mb_functools_cmp_to_key_richcmp(a: MbValue, b: MbValue, op: &str) -> Option<bool> {
    let sign = cmp_to_key_compare(a, b)?;
    Some(match op {
        "lt" => sign < 0,
        "le" => sign <= 0,
        "gt" => sign > 0,
        "ge" => sign >= 0,
        "eq" => sign == 0,
        "ne" => sign != 0,
        _ => false,
    })
}

thread_local! {
    /// `__wrapped__` back-references set by `functools.wraps` /
    /// `update_wrapper`, keyed by the wrapper function's NaN-boxed bits.
    static FUNC_WRAPPED: std::cell::RefCell<HashMap<u64, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Record `wrapper.__wrapped__ = wrapped`.
pub fn set_func_wrapped(wrapper: MbValue, wrapped: MbValue) {
    unsafe { super::super::rc::retain_if_ptr(wrapped); }
    FUNC_WRAPPED.with(|m| { m.borrow_mut().insert(wrapper.to_bits(), wrapped); });
}

/// Read `wrapper.__wrapped__`, or a None-MbValue when unset. Consulted by the
/// runtime attribute-access path for the `__wrapped__` dunder.
pub fn get_func_wrapped(wrapper: MbValue) -> MbValue {
    FUNC_WRAPPED.with(|m| m.borrow().get(&wrapper.to_bits()).copied().unwrap_or_else(MbValue::none))
}

/// Apply a `functools.wraps`/`update_wrapper` copy from `wrapped` onto
/// `wrapper` (both function values): copy `__name__`, `__qualname__`,
/// `__module__`, `__doc__` and set `__wrapped__ = wrapped`. Returns `wrapper`.
pub fn mb_functools_wraps_apply(wrapper: MbValue, wrapped: MbValue) -> MbValue {
    let name = super::super::closure::mb_func_get_name(wrapped);
    if !name.is_none() {
        super::super::closure::mb_func_set_name(wrapper, name);
    }
    let doc = super::super::closure::mb_func_get_doc(wrapped);
    if !doc.is_none() {
        super::super::closure::mb_func_set_doc(wrapper, doc);
    }
    let module = super::super::closure::mb_func_get_module(wrapped);
    if !module.is_none() {
        super::super::closure::mb_func_set_module(wrapper, module);
    }
    set_func_wrapped(wrapper, wrapped);
    unsafe { super::super::rc::retain_if_ptr(wrapper); }
    wrapper
}

/// functools.update_wrapper(wrapper, wrapped) -> wrapper  (R8)
///
/// Copies `__name__` and `__doc__` fields from `wrapped` to `wrapper` when
/// both are Instance objects. Otherwise returns `wrapper` unchanged (identity
/// passthrough for non-Instance values).
pub fn mb_functools_update_wrapper(wrapper: MbValue, wrapped: MbValue) -> MbValue {
    // Function-valued wrapper/wrapped (the common `@wraps`/`update_wrapper`
    // shape): copy name/doc/module and set __wrapped__ via the shared helper.
    let wrapper_is_fn = wrapper.as_func().is_some()
        || (wrapper.as_int().is_some()
            && !super::super::closure::mb_func_get_name(wrapper).is_none());
    let wrapped_is_fn = wrapped.as_func().is_some()
        || (wrapped.as_int().is_some()
            && !super::super::closure::mb_func_get_name(wrapped).is_none());
    if wrapper_is_fn || wrapped_is_fn {
        return mb_functools_wraps_apply(wrapper, wrapped);
    }
    // Only copy fields when both values are heap-allocated Instance objects.
    let wrapper_ptr = wrapper.as_ptr();
    let wrapped_ptr = wrapped.as_ptr();
    if let (Some(wp), Some(dp)) = (wrapper_ptr, wrapped_ptr) {
        unsafe {
            let is_wrapper_instance = matches!(&(*wp).data, ObjData::Instance { .. });
            let is_wrapped_instance = matches!(&(*dp).data, ObjData::Instance { .. });
            if is_wrapper_instance && is_wrapped_instance {
                if let ObjData::Instance { ref fields, .. } = (*dp).data {
                    let src = fields.read().unwrap();
                    if let ObjData::Instance { fields: ref dst_fields, .. } = (*wp).data {
                        let mut dst = dst_fields.write().unwrap();
                        for key in &["__name__", "__doc__", "__module__", "__qualname__"] {
                            if let Some(&val) = src.get(*key) {
                                dst.insert(key.to_string(), val);
                            }
                        }
                    }
                }
            }
        }
    }
    wrapper
}

/// functools.singledispatch(func) -> Instance  (R9)
///
/// MVP: wraps the function in a `functools.singledispatch` Instance.
/// The dispatch registry and `@f.register` mechanism are future work.
pub fn mb_functools_singledispatch(func: MbValue) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("_func".to_string(), func);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.singledispatch".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// functools.singledispatchmethod(func) -> func  (R10)
///
/// MVP: identity passthrough matching the total_ordering pattern.
/// Returns the first argument unchanged.
pub fn mb_functools_singledispatchmethod(func: MbValue) -> MbValue {
    func
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn make_list(vals: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    #[test]
    fn test_reduce_add() {
        let items = make_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]);
        let result =
            mb_functools_reduce(s("add"), items, MbValue::from_int(0));
        assert_eq!(result.as_int(), Some(6));
    }

    #[test]
    fn test_reduce_mul() {
        let items = make_list(vec![
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(4),
        ]);
        let result =
            mb_functools_reduce(s("mul"), items, MbValue::from_int(1));
        assert_eq!(result.as_int(), Some(24));
    }

    #[test]
    fn test_reduce_unknown_func_returns_initial() {
        let items = make_list(vec![MbValue::from_int(10)]);
        let result = mb_functools_reduce(
            s("unknown"),
            items,
            MbValue::from_int(42),
        );
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_partial() {
        let func = s("my_func");
        let args = make_list(vec![MbValue::from_int(1)]);
        let result = mb_functools_partial(func, args);
        // Result should be a functools.partial Instance with func + args fields
        unsafe {
            let ptr = result.as_ptr().expect("expected instance");
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                assert_eq!(class_name, "functools.partial");
                let f = fields.read().unwrap();
                let stored_func = f.get("func").copied().unwrap();
                let name = extract_str(stored_func).unwrap();
                assert_eq!(name, "my_func");
            } else {
                panic!("expected functools.partial Instance");
            }
        }
    }

    #[test]
    fn test_lru_cache_wraps_func() {
        // lru_cache used to be an identity passthrough; it is now a real wrapper
        // that returns a `functools.lru_cache_wrapper` Instance retaining the
        // wrapped callable in `_func`.
        let func = MbValue::from_int(999);
        let result = mb_functools_lru_cache(func);
        unsafe {
            let ptr = result.as_ptr().expect("expected wrapper Instance");
            if let ObjData::Instance { class_name, fields } = &(*ptr).data {
                assert_eq!(class_name, "functools.lru_cache_wrapper");
                let f = fields.read().unwrap();
                assert_eq!(f.get("_func").and_then(|v| v.as_int()), Some(999));
            } else {
                panic!("expected Instance, got non-Instance");
            }
        }
    }

    // REQ: R6
    #[test]
    fn test_cached_property_passthrough() {
        // cached_property is an identity passthrough at MVP.
        let func = MbValue::from_int(42);
        let result = mb_functools_cached_property(func);
        assert_eq!(result.as_int(), Some(42));
    }

    // REQ: R7
    #[test]
    fn test_cmp_to_key_creates_instance() {
        // cmp_to_key should return an Instance with class_name "functools.cmp_to_key"
        // and a "_cmp" field holding the comparison function.
        let cmp_fn = MbValue::from_int(7); // stand-in for a callable
        let result = mb_functools_cmp_to_key(cmp_fn);
        unsafe {
            let ptr = result.as_ptr().expect("expected instance ptr");
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                assert_eq!(class_name, "functools.cmp_to_key");
                let f = fields.read().unwrap();
                let stored = f.get("_cmp").copied().expect("_cmp field missing");
                assert_eq!(stored.as_int(), Some(7));
            } else {
                panic!("expected functools.cmp_to_key Instance");
            }
        }
    }

    // REQ: R8
    #[test]
    fn test_update_wrapper_identity_non_instance() {
        // When args are not Instance objects, update_wrapper returns wrapper unchanged.
        let wrapper = MbValue::from_int(100);
        let wrapped = MbValue::from_int(200);
        let result = mb_functools_update_wrapper(wrapper, wrapped);
        assert_eq!(result.as_int(), Some(100));
    }

    // REQ: R8
    #[test]
    fn test_update_wrapper_copies_fields() {
        use super::super::super::rc::{MbObject as Obj, ObjData as OD};
        // When both are Instance objects, __name__ and __doc__ are copied.
        // Build a "wrapped" instance with __name__ = "my_func".
        let wrapped_ptr = Obj::new_instance("target.func".to_string());
        unsafe {
            if let OD::Instance { ref fields, .. } = (*wrapped_ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("__name__".to_string(), s("my_func"));
            }
        }
        let wrapped = MbValue::from_ptr(wrapped_ptr);

        // Build an empty "wrapper" Instance.
        let wrapper_ptr = Obj::new_instance("wrapper.func".to_string());
        let wrapper = MbValue::from_ptr(wrapper_ptr);

        let result = mb_functools_update_wrapper(wrapper, wrapped);
        // result should be wrapper with __name__ copied.
        unsafe {
            let ptr = result.as_ptr().expect("expected instance ptr");
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let name = f.get("__name__").copied().expect("__name__ missing after update_wrapper");
                assert_eq!(extract_str(name).as_deref(), Some("my_func"));
            } else {
                panic!("expected Instance");
            }
        }
    }

    // REQ: R9
    #[test]
    fn test_singledispatch_creates_instance() {
        let func = MbValue::from_int(55);
        let result = mb_functools_singledispatch(func);
        unsafe {
            let ptr = result.as_ptr().expect("expected instance ptr");
            if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                assert_eq!(class_name, "functools.singledispatch");
                let f = fields.read().unwrap();
                let stored = f.get("_func").copied().expect("_func field missing");
                assert_eq!(stored.as_int(), Some(55));
            } else {
                panic!("expected functools.singledispatch Instance");
            }
        }
    }

    // REQ: R10
    #[test]
    fn test_singledispatchmethod_passthrough() {
        // singledispatchmethod is an identity passthrough at MVP.
        let func = MbValue::from_int(77);
        let result = mb_functools_singledispatchmethod(func);
        assert_eq!(result.as_int(), Some(77));
    }

    // -- recursive_repr / get_cache_token / GenericAlias stubs
    //    (Wave-7 ship #4, #1265 Task #77) --

    #[test]
    fn test_recursive_repr_passthrough() {
        let func = MbValue::from_int(42);
        let r = mb_functools_recursive_repr(func);
        assert_eq!(r.as_int(), Some(42));
    }

    #[test]
    fn test_get_cache_token_returns_zero() {
        // Cache token is a stable opaque value for now.
        assert_eq!(mb_functools_get_cache_token().as_int(), Some(0));
        assert_eq!(mb_functools_get_cache_token().as_int(), Some(0));
    }

    #[test]
    fn test_generic_alias_stores_origin_and_args() {
        let origin = MbValue::from_int(123);
        let type_args = MbValue::from_int(456);
        let r = mb_functools_generic_alias(origin, type_args);
        unsafe {
            let ptr = r.as_ptr().expect("GenericAlias should be ptr");
            if let ObjData::Instance { ref class_name, ref fields, .. } = (*ptr).data {
                assert_eq!(class_name, "types.GenericAlias");
                let f = fields.read().unwrap();
                assert_eq!(f.get("_origin").and_then(|v| v.as_int()), Some(123));
                assert_eq!(f.get("_args").and_then(|v| v.as_int()), Some(456));
            } else {
                panic!("expected Instance");
            }
        }
    }
}
