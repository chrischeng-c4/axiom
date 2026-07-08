use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use super::call_named_callable;

/// sorted(iterable, key=None, reverse=False) — sort with key function and reverse flag.
/// Validate that `func` can be called with a single positional argument (the
/// sort/min/max key contract). Raises and returns true when it declares more
/// than one REQUIRED positional parameter — CPython: "<lambda>() missing 1
/// required positional argument: 'y'". A native callable (no recorded params)
/// or a callable with `*args` is left unchecked.
pub fn key_unary_arity_error(func: MbValue) -> bool {
    let params = match super::super::closure::func_params(func) {
        Some(p) => p,
        None => return false,
    };
    if params.iter().any(|p| p.kind == 2) {
        return false; // *args absorbs extra/missing positionals
    }
    let required: Vec<String> = params
        .iter()
        .filter(|p| p.kind <= 1 && !p.has_default)
        .map(|p| p.name.clone())
        .collect();
    if required.len() <= 1 {
        return false;
    }
    let missing = &required[1..];
    let n = missing.len();
    let names = if n == 1 {
        format!("'{}'", missing[0])
    } else {
        let head = missing[..n - 1]
            .iter()
            .map(|x| format!("'{x}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{head} and '{}'", missing[n - 1])
    };
    let fname = super::super::closure::mb_func_get_name(func)
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "<lambda>".to_string());
    super::raise_type_error(format!(
        "{fname}() missing {n} required positional argument{}: {names}",
        if n == 1 { "" } else { "s" }
    ));
    true
}

pub fn mb_sorted_kwargs(iterable: MbValue, key: MbValue, reverse: MbValue) -> MbValue {
    let items = super::extract_items(iterable);
    let do_reverse = reverse.as_bool() == Some(true) || reverse.as_int() == Some(1);
    let has_key = !key.is_none();

    if has_key {
        // The key must be callable (CPython: `sorted(xs, key=42)` →
        // "'int' object is not callable"). Callables are functions, named
        // builtins (Str), or instances with __call__; a bare scalar/container
        // is rejected up front rather than silently producing None keys.
        let key_callable = super::resolve_callable(key).is_some()
            || key.as_ptr().map_or(false, |p| unsafe {
                matches!(&(*p).data, ObjData::Str(_) | ObjData::Instance { .. })
            });
        if !key_callable {
            super::raise_type_error(format!(
                "'{}' object is not callable",
                super::value_type_name(key)
            ));
            return MbValue::none();
        }
        // The key is invoked with exactly one argument; a key declaring >1
        // required positional param raises TypeError before any sorting.
        if key_unary_arity_error(key) {
            return MbValue::none();
        }
    }

    if has_key {
        // Apply key function to each element, sort by key result
        let key_fn_addr = super::resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };

        let mut indexed: Vec<(MbValue, MbValue)> = Vec::with_capacity(items.len());
        for &item in &items {
            let k = if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                // Instance-based callables (unbound method wrappers,
                // functools.partial, @dataclass callables, __call__ protocol,
                // ...) — route through the dynamic 1-arg dispatcher.
                super::super::class::mb_call1_val(key, item)
            } else {
                item
            };
            // A key function that raises aborts the sort (CPython propagates the
            // exception rather than swallowing it and sorting partial keys).
            if super::super::exception::mb_has_exception().as_bool() == Some(true) {
                return MbValue::none();
            }
            indexed.push((item, k));
        }

        indexed.sort_by(|a, b| {
            super::stable_order_for_reverse(super::mb_value_cmp(a.1, b.1), do_reverse)
        });
        let sorted_items: Vec<MbValue> = indexed.into_iter().map(|(v, _)| v).collect();
        // Items borrowed from source container — retain.
        MbValue::from_ptr(MbObject::new_list_borrowed(sorted_items))
    } else {
        let mut sorted_items = items;
        // Type-specialized sort for no-key case (same logic as mb_sorted).
        if !sorted_items.is_empty()
            && sorted_items[0].is_int()
            && sorted_items.iter().all(|v| v.is_int())
        {
            sorted_items.sort_by(|a, b| {
                super::stable_order_for_reverse(
                    a.as_int().unwrap_or(0).cmp(&b.as_int().unwrap_or(0)),
                    do_reverse,
                )
            });
        } else {
            sorted_items.sort_by(|a, b| {
                super::stable_order_for_reverse(super::mb_value_cmp(*a, *b), do_reverse)
            });
        }
        // Items borrowed from source container — retain.
        MbValue::from_ptr(MbObject::new_list_borrowed(sorted_items))
    }
}

/// min(iterable, key=None, default=None) — min with key and default.
pub fn mb_min_kwargs(args: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let items = super::extract_items(args);
    if items.is_empty() {
        return if default.is_none() {
            MbValue::none()
        } else {
            default
        };
    }
    let has_key = !key.is_none();
    let result = if has_key {
        let key_fn_addr = super::resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let apply_key = |item: MbValue| -> MbValue {
            if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                super::super::class::mb_call1_val(key, item)
            } else {
                item
            }
        };
        items
            .into_iter()
            .reduce(|a, b| {
                if super::compare_values(apply_key(a), apply_key(b)) {
                    a
                } else {
                    b
                }
            })
            .unwrap_or(default)
    } else {
        items
            .into_iter()
            .reduce(|a, b| if super::compare_values(a, b) { a } else { b })
            .unwrap_or(default)
    };
    // Honor NEW-contract: returned value must be independently owned —
    // the iterable still holds its own ref, so retain on the way out.
    unsafe {
        super::super::rc::retain_if_ptr(result);
    }
    result
}

/// max(iterable, key=None, default=None) — max with key and default.
pub fn mb_max_kwargs(args: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let items = super::extract_items(args);
    if items.is_empty() {
        return if default.is_none() {
            MbValue::none()
        } else {
            default
        };
    }
    let has_key = !key.is_none();
    let result = if has_key {
        let key_fn_addr = super::resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let apply_key = |item: MbValue| -> MbValue {
            if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                super::super::class::mb_call1_val(key, item)
            } else {
                item
            }
        };
        items
            .into_iter()
            .reduce(|a, b| {
                if super::compare_values(apply_key(b), apply_key(a)) {
                    a
                } else {
                    b
                }
            })
            .unwrap_or(default)
    } else {
        items
            .into_iter()
            .reduce(|a, b| if super::compare_values(b, a) { a } else { b })
            .unwrap_or(default)
    };
    unsafe {
        super::super::rc::retain_if_ptr(result);
    }
    result
}

/// sum(iterable, start) — sum with an initial value.
pub fn mb_sum_with_start(iterable: MbValue, start: MbValue) -> MbValue {
    // CPython rejects text/bytes starts up front with a dedicated message.
    if let Some(ptr) = start.as_ptr() {
        let reject = unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => Some("strings [use ''.join(seq) instead]"),
                ObjData::Bytes(_) => Some("bytes [use b''.join(seq) instead]"),
                ObjData::ByteArray(_) => Some("bytearray [use b''.join(seq) instead]"),
                _ => None,
            }
        };
        if let Some(kind) = reject {
            super::raise_type_error(format!("sum() can't sum {kind}"));
            return MbValue::none();
        }
    }
    super::sum_from(iterable, start)
}
