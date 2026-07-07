use super::super::rc::ObjData;
use super::super::value::MbValue;
use super::{mb_abs, mb_bool, mb_chr, mb_float, mb_int, mb_len, mb_ord, mb_repr, mb_str};

/// Apply a named builtin type constructor or function to a single value.
/// Returns `Some(result)` if the name is known, `None` otherwise.
/// Used by mb_map/mb_filter when func is a string type-name (e.g. "str", "abs").
pub(crate) fn call_named_callable(name: &str, item: MbValue) -> Option<MbValue> {
    match name {
        "str" => Some(mb_str(item)),
        "int" => Some(mb_int(item)),
        "float" => Some(mb_float(item)),
        "bool" => Some(mb_bool(item)),
        "abs" => Some(mb_abs(item)),
        "len" => Some(mb_len(item)),
        "repr" => Some(mb_repr(item)),
        "chr" => Some(mb_chr(item)),
        "ord" => Some(mb_ord(item)),
        _ => None,
    }
}

/// Public wrapper for cross-module named callable dispatch.
pub fn call_named_callable_pub(name: &str, item: MbValue) -> Option<MbValue> {
    call_named_callable(name, item)
}

/// Extract a builtin type name from a callable value.
///
/// - STRING `"int"` / `"str"` → name (legacy class_syms path)
/// - Type-singleton INSTANCE (`class_name="type"`, `__name__="int"`) → name (new path)
/// - Otherwise → None
pub fn callable_as_type_name(func: MbValue) -> Option<String> {
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(ref s) => Some(s.clone()),
                ObjData::Instance {
                    class_name: ref cn,
                    ref fields,
                } if cn == "type" => fields.read().ok().and_then(|f| {
                    f.get("__name__").and_then(|v| {
                        if let Some(vp) = v.as_ptr() {
                            if let ObjData::Str(ref s) = (*vp).data {
                                Some(s.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                }),
                _ => None,
            }
        }
    } else {
        None
    }
}

/// General callable dispatcher — handles all callable types in a single call.
///
/// Dispatch order:
/// 1. String-named or type-singleton builtins (`int`, `str`, `abs`, …)
///    resolved via `callable_as_type_name` + `call_named_callable`.
/// 2. Fall through to `mb_call1_val` for TAG_FUNC pointers, Instance
///    callables with `__call__`, unbound method wrappers, functools.partial.
///
/// Used by the `IterKind::Map` / `IterKind::Filter` advance paths so that
/// lazy map/filter iterators work for all callable flavours, not just the
/// TAG_FUNC subset that `mb_call_method1` covers.
pub fn call_any_callable(func: MbValue, arg: MbValue) -> MbValue {
    if let Some(type_name) = callable_as_type_name(func) {
        if let Some(result) = call_named_callable(&type_name, arg) {
            return result;
        }
    }
    super::super::class::mb_call1_val(func, arg)
}

/// map(func, iterable) — return a lazy map iterator (not a list).
///
/// Delegates to `iter::mb_map_iter` which stores an `IterKind::Map` handle
/// in the thread-local ITERATORS table. The handle satisfies `hasattr(x,
/// "__next__")` and is consumed lazily by `list()`, `for`-loops, `next()`,
/// etc. — matching CPython's `map` object semantics.
pub fn mb_map(func: MbValue, iterable: MbValue) -> MbValue {
    super::super::iter::mb_map_iter(func, iterable)
}

/// filter(func, iterable) — return a lazy filter iterator (not a list).
///
/// Delegates to `iter::mb_filter_iter` which stores an `IterKind::Filter`
/// handle in ITERATORS. Lazy, like CPython's `filter` object.
pub fn mb_filter(func: MbValue, iterable: MbValue) -> MbValue {
    super::super::iter::mb_filter_iter(func, iterable)
}
