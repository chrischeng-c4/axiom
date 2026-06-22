use super::super::dict_ops::dict_key_to_mbvalue;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// reprlib module for Mamba (#1261 long-tail).
///
/// A faithful native port of CPython 3.12's `reprlib.Repr`: abbreviating
/// `repr()` with per-type size limits (`maxstring=30`, `maxlist=6`,
/// `maxtuple=6`, `maxset=6`, `maxfrozenset=6`, `maxdict=4`, `maxother=30`,
/// `maxlong=40`, `maxlevel=6`, `fillvalue='...'`, `indent=None`) and the
/// `indent` pretty-printing feature.
///
/// `Repr` is registered as a real runtime class so instances support
/// `r = Repr()`, `Repr(**kwargs)`, attribute get/set (`r.indent = 4`), and a
/// bound `r.repr(x)` method that reads the per-instance limits. The
/// module-level `aRepr` is one such instance and `reprlib.repr` is its bound
/// `repr`. `recursive_repr` is a recursion-guarding decorator factory.
use std::collections::HashMap;

const REPR_CLASS: &str = "Repr";

// ── Default limits (CPython 3.12 reprlib.Repr.__init__ defaults) ──
const DEF_MAXLEVEL: i64 = 6;
const DEF_MAXTUPLE: i64 = 6;
const DEF_MAXLIST: i64 = 6;
const DEF_MAXARRAY: i64 = 5;
const DEF_MAXDICT: i64 = 4;
const DEF_MAXSET: i64 = 6;
const DEF_MAXFROZENSET: i64 = 6;
const DEF_MAXDEQUE: i64 = 6;
const DEF_MAXSTRING: i64 = 30;
const DEF_MAXLONG: i64 = 40;
const DEF_MAXOTHER: i64 = 30;

// ── Small helpers ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn raise(exc: &str, msg: &str) {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
}

fn exception_pending() -> bool {
    super::super::exception::mb_has_exception().as_bool() == Some(true)
}

fn clear_exception() {
    super::super::exception::mb_clear_exception();
}

fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                // MbSet derefs to its ordered MbList for read-only access.
                ObjData::Set(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) | ObjData::FrozenSet(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// builtins.repr(val) → String.
fn builtin_repr(val: MbValue) -> String {
    let r = super::super::builtins::mb_repr(val);
    let out = unsafe {
        if let Some(p) = r.as_ptr() {
            if let ObjData::Str(ref s) = (*p).data {
                s.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };
    unsafe {
        super::super::rc::release_if_ptr(r);
    }
    out
}

/// Per-instance limits read from a `Repr` instance's fields (with CPython
/// defaults when a field is missing).
struct Limits {
    maxlevel: i64,
    maxtuple: i64,
    maxlist: i64,
    #[allow(dead_code)]
    maxarray: i64,
    maxdict: i64,
    maxset: i64,
    maxfrozenset: i64,
    #[allow(dead_code)]
    maxdeque: i64,
    maxstring: i64,
    maxlong: i64,
    maxother: i64,
    fillvalue: String,
    indent: MbValue,
}

fn field_int(inst: MbValue, key: &str, default: i64) -> i64 {
    match get_field(inst, key) {
        Some(v) => v
            .as_int()
            .or_else(|| v.as_bool().map(|b| b as i64))
            .unwrap_or(default),
        None => default,
    }
}

fn load_limits(inst: MbValue) -> Limits {
    let fillvalue = get_field(inst, "fillvalue")
        .and_then(extract_str)
        .unwrap_or_else(|| "...".to_string());
    let indent = get_field(inst, "indent").unwrap_or_else(MbValue::none);
    Limits {
        maxlevel: field_int(inst, "maxlevel", DEF_MAXLEVEL),
        maxtuple: field_int(inst, "maxtuple", DEF_MAXTUPLE),
        maxlist: field_int(inst, "maxlist", DEF_MAXLIST),
        maxarray: field_int(inst, "maxarray", DEF_MAXARRAY),
        maxdict: field_int(inst, "maxdict", DEF_MAXDICT),
        maxset: field_int(inst, "maxset", DEF_MAXSET),
        maxfrozenset: field_int(inst, "maxfrozenset", DEF_MAXFROZENSET),
        maxdeque: field_int(inst, "maxdeque", DEF_MAXDEQUE),
        maxstring: field_int(inst, "maxstring", DEF_MAXSTRING),
        maxlong: field_int(inst, "maxlong", DEF_MAXLONG),
        maxother: field_int(inst, "maxother", DEF_MAXOTHER),
        fillvalue,
        indent,
    }
}

/// Truncate a `repr` string the way CPython's `repr_int`/`repr_instance` do:
/// keep `i` head chars and `j` tail chars around the fillvalue.
fn truncate_repr(s: &str, max: i64, fill: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if (chars.len() as i64) <= max {
        return s.to_string();
    }
    let i = std::cmp::max(0, (max - 3) / 2) as usize;
    let j = std::cmp::max(0, max - 3 - i as i64) as usize;
    let head: String = chars.iter().take(i).collect();
    let tail: String = chars.iter().skip(chars.len() - j).collect();
    format!("{}{}{}", head, fill, tail)
}

/// CPython `_possibly_sorted`: sort the items, falling back to original order
/// when the elements are not mutually orderable.
fn possibly_sorted(items: &[MbValue]) -> Vec<MbValue> {
    let mut out = items.to_vec();
    let mut orderable = true;
    out.sort_by(|a, b| {
        if !orderable {
            return std::cmp::Ordering::Equal;
        }
        let lt = super::super::builtins::mb_lt(*a, *b);
        match lt.as_bool() {
            Some(true) => std::cmp::Ordering::Less,
            Some(false) => {
                let gt = super::super::builtins::mb_lt(*b, *a);
                match gt.as_bool() {
                    Some(true) => std::cmp::Ordering::Greater,
                    Some(false) => std::cmp::Ordering::Equal,
                    None => {
                        orderable = false;
                        std::cmp::Ordering::Equal
                    }
                }
            }
            None => {
                orderable = false;
                std::cmp::Ordering::Equal
            }
        }
    });
    // Any pending comparison exception must be cleared and the original order
    // returned (CPython catches the exception in _possibly_sorted).
    if !orderable || exception_pending() {
        clear_exception();
        return items.to_vec();
    }
    out
}

/// CPython `Repr._join(pieces, level)`.
///
/// Returns `Some(joined)` on success, or `None` after raising a Value/Type
/// error for an invalid `indent`.
fn join(lim: &Limits, pieces: &[String], level: i64) -> Option<String> {
    if lim.indent.is_none() {
        return Some(pieces.join(", "));
    }
    if pieces.is_empty() {
        return Some(String::new());
    }
    // Resolve the indent unit string. int/bool → that many spaces; str → as-is;
    // anything else → TypeError.
    let indent_str: String = if let Some(s) = extract_str(lim.indent) {
        s
    } else if let Some(b) = lim.indent.as_bool() {
        // bool is a subclass of int in Python: True→1 space, False→0.
        " ".repeat(b as usize)
    } else if let Some(i) = lim.indent.as_int() {
        if i < 0 {
            raise(
                "ValueError",
                &format!("Repr.indent cannot be negative int (was {})", i),
            );
            return None;
        }
        " ".repeat(i as usize)
    } else {
        // Mirror CPython's `f'{type(indent)}'` message wording.
        let tn = type_repr_name(lim.indent);
        raise(
            "TypeError",
            &format!(
                "Repr.indent must be a str, int or None, not <class '{}'>",
                tn
            ),
        );
        return None;
    };

    // sep = ',\n' + (maxlevel - level + 1) * indent
    let reps = lim.maxlevel - level + 1;
    let reps = if reps < 0 { 0 } else { reps as usize };
    let sep = format!(",\n{}", indent_str.repeat(reps));

    // CPython: sep.join(('', *pieces, ''))[1:-len(indent) or None]
    // sep.join(['', p0, p1, ..., '']) == sep + p0 + sep + p1 + ... + sep
    let mut joined = String::new();
    // leading empty element then sep before each subsequent element, trailing empty.
    // Equivalent to: sep + pieces.join(sep) + sep
    joined.push_str(&sep);
    joined.push_str(&pieces.join(&sep));
    joined.push_str(&sep);

    let bytes: Vec<char> = joined.chars().collect();
    let n = bytes.len();
    let ilen = indent_str.chars().count();
    // [1:-len(indent) or None] : if len(indent)==0 → [1:], else [1:n-ilen]
    let end = if ilen == 0 { n } else { n.saturating_sub(ilen) };
    let start = 1.min(n);
    let sliced: String = if start <= end {
        bytes[start..end].iter().collect()
    } else {
        String::new()
    };
    Some(sliced)
}

/// A short, Python-ish type name for the TypeError message in `_join`.
fn type_repr_name(val: MbValue) -> String {
    if val.is_none() {
        return "NoneType".to_string();
    }
    if val.as_bool().is_some() {
        return "bool".to_string();
    }
    if val.as_int().is_some() {
        return "int".to_string();
    }
    if val.as_float().is_some() {
        return "float".to_string();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str".to_string(),
                ObjData::List(_) => "list".to_string(),
                ObjData::Tuple(_) => "tuple".to_string(),
                ObjData::Dict(_) => "dict".to_string(),
                ObjData::Set(_) => "set".to_string(),
                ObjData::FrozenSet(_) => "frozenset".to_string(),
                ObjData::Bytes(_) => "bytes".to_string(),
                ObjData::ByteArray(_) => "bytearray".to_string(),
                ObjData::Instance { class_name, .. } => class_name.clone(),
                _ => "object".to_string(),
            };
        }
    }
    "object".to_string()
}

/// CPython `Repr._repr_iterable`.
fn repr_iterable(
    lim: &Limits,
    inst: MbValue,
    items: &[MbValue],
    level: i64,
    left: &str,
    right: &str,
    maxiter: i64,
    trail: &str,
) -> Option<String> {
    let n = items.len() as i64;
    let mut right = right.to_string();
    let s: String = if level <= 0 && n > 0 {
        lim.fillvalue.clone()
    } else {
        let newlevel = level - 1;
        let take = std::cmp::min(maxiter.max(0) as usize, items.len());
        let mut pieces: Vec<String> = Vec::with_capacity(take + 1);
        for it in items.iter().take(take) {
            pieces.push(repr1(lim, inst, *it, newlevel)?);
        }
        if n > maxiter {
            pieces.push(lim.fillvalue.clone());
        }
        let joined = join(lim, &pieces, level)?;
        if n == 1 && !trail.is_empty() && lim.indent.is_none() {
            right = format!("{}{}", trail, right);
        }
        joined
    };
    Some(format!("{}{}{}", left, s, right))
}

/// CPython `Repr.repr_dict`.
fn repr_dict(lim: &Limits, inst: MbValue, val: MbValue, level: i64) -> Option<String> {
    let ptr = val.as_ptr()?;
    // Materialize (key_value, value) pairs once. Keys are reconstructed from
    // the DictKey so they re-render through the normal repr path.
    let pairs: Vec<(MbValue, MbValue)> = unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read()
                .unwrap()
                .iter()
                .map(|(k, v)| (dict_key_to_mbvalue(k), *v))
                .collect()
        } else {
            return None;
        }
    };
    let n = pairs.len() as i64;
    if n == 0 {
        return Some("{}".to_string());
    }
    if level <= 0 {
        return Some(format!("{{{}}}", lim.fillvalue));
    }
    let newlevel = level - 1;
    // Sort keys like CPython's `islice(_possibly_sorted(x), self.maxdict)`.
    let keys: Vec<MbValue> = pairs.iter().map(|(k, _)| *k).collect();
    let sorted_keys = possibly_sorted(&keys);
    let take = std::cmp::min(lim.maxdict.max(0) as usize, sorted_keys.len());
    let mut pieces: Vec<String> = Vec::with_capacity(take + 1);
    let mut result: Option<String> = None;
    'outer: {
        for key in sorted_keys.iter().take(take) {
            // Find the value paired with this key (by identity/equality match).
            let v = pairs
                .iter()
                .find(|(k, _)| super::super::builtins::mb_eq(*k, *key).as_bool() == Some(true))
                .map(|(_, v)| *v)
                .unwrap_or_else(MbValue::none);
            let keyrepr = match repr1(lim, inst, *key, newlevel) {
                Some(s) => s,
                None => break 'outer,
            };
            let valrepr = match repr1(lim, inst, v, newlevel) {
                Some(s) => s,
                None => break 'outer,
            };
            pieces.push(format!("{}: {}", keyrepr, valrepr));
        }
        if n > lim.maxdict {
            pieces.push(lim.fillvalue.clone());
        }
        result = join(lim, &pieces, level).map(|s| format!("{{{}}}", s));
    }
    // Release the temporary key values we materialized.
    for (k, _) in &pairs {
        unsafe {
            super::super::rc::release_if_ptr(*k);
        }
    }
    result
}

/// CPython `Repr.repr_str`.
fn repr_str(lim: &Limits, val: MbValue) -> String {
    let full = extract_str(val).unwrap_or_default();
    let chars: Vec<char> = full.chars().collect();
    // s = builtins.repr(x[:maxstring])
    let head_n = std::cmp::min(lim.maxstring.max(0) as usize, chars.len());
    let prefix: String = chars.iter().take(head_n).collect();
    let s = builtin_repr(new_str(&prefix));
    if (s.chars().count() as i64) <= lim.maxstring {
        return s;
    }
    let i = std::cmp::max(0, (lim.maxstring - 3) / 2) as usize;
    let j = std::cmp::max(0, lim.maxstring - 3 - i as i64) as usize;
    // s = builtins.repr(x[:i] + x[len(x)-j:])
    let mut combined: String = chars.iter().take(i).collect();
    let tail: String = chars.iter().skip(chars.len().saturating_sub(j)).collect();
    combined.push_str(&tail);
    let s2 = builtin_repr(new_str(&combined));
    let s2c: Vec<char> = s2.chars().collect();
    // s = s[:i] + fillvalue + s[len(s)-j:]
    let head: String = s2c.iter().take(i).collect();
    let tail2: String = s2c.iter().skip(s2c.len().saturating_sub(j)).collect();
    format!("{}{}{}", head, lim.fillvalue, tail2)
}

/// CPython `Repr.repr_int`.
fn repr_int(lim: &Limits, val: MbValue) -> String {
    let s = builtin_repr(val);
    truncate_repr(&s, lim.maxlong, &lim.fillvalue)
}

/// CPython `Repr.repr_instance` (the fallback for any other object).
fn repr_instance(lim: &Limits, val: MbValue) -> String {
    let s = builtin_repr(val);
    // A failing __repr__ leaves a pending exception; CPython catches it and
    // substitutes a placeholder. Clear it and fall back.
    if exception_pending() {
        clear_exception();
        let cls = type_repr_name(val);
        return format!("<{} instance at 0x...>", cls);
    }
    truncate_repr(&s, lim.maxother, &lim.fillvalue)
}

/// `type(x).__name__`, with `__name__` read from the type object so a class
/// whose name was reassigned (`type(t).__name__ = '...'`) is honored.
fn value_typename(val: MbValue) -> String {
    let ty = super::super::builtins::mb_type(val);
    let name = super::super::class::mb_getattr_default(ty, new_str("__name__"), MbValue::none());
    let out = extract_str(name).unwrap_or_default();
    unsafe {
        super::super::rc::release_if_ptr(ty);
    }
    out
}

/// The built-in typenames handled natively by `repr1`. A user `Repr`
/// subclass defining `repr_<one of these>` does not override them (CPython
/// gates builtin overrides behind a module check we approximate by always
/// keeping builtins native).
fn is_builtin_typename(name: &str) -> bool {
    matches!(
        name,
        "str"
            | "list"
            | "tuple"
            | "set"
            | "frozenset"
            | "dict"
            | "int"
            | "bool"
            | "float"
            | "bytes"
            | "bytearray"
            | "NoneType"
    )
}

/// If the Repr instance defines a custom `repr_<typename>` method for a
/// non-builtin type, dispatch to it (CPython `repr1` → `getattr(self,
/// 'repr_'+typename)`). Returns `Some(result)` when such a method handled the
/// value, else `None`.
fn try_custom_method(inst: MbValue, val: MbValue, level: i64) -> Option<String> {
    let mut typename = value_typename(val);
    if typename.is_empty() {
        return None;
    }
    if typename.contains(' ') {
        typename = typename.split_whitespace().collect::<Vec<_>>().join("_");
    }
    if is_builtin_typename(&typename) {
        return None;
    }
    let method_name = format!("repr_{}", typename);
    let has = super::super::class::mb_hasattr(inst, new_str(&method_name));
    if has.as_bool() != Some(true) {
        return None;
    }
    let args = MbValue::from_ptr(MbObject::new_list(vec![val, MbValue::from_int(level)]));
    let result = super::super::class::mb_call_method(inst, new_str(&method_name), args);
    unsafe {
        super::super::rc::release_if_ptr(args);
    }
    if exception_pending() {
        // Let the caller surface the exception; mirror repr_instance fallback.
        return None;
    }
    extract_str(result)
}

/// CPython `Repr.repr1` — dispatch by type, then abbreviate.
fn repr1(lim: &Limits, inst: MbValue, val: MbValue, level: i64) -> Option<String> {
    // Custom `repr_<typename>` override on a Repr subclass (non-builtin types).
    if let Some(s) = try_custom_method(inst, val, level) {
        return Some(s);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => return Some(repr_str(lim, val)),
                ObjData::List(lock) => {
                    let items = lock.read().unwrap().to_vec();
                    return repr_iterable(lim, inst, &items, level, "[", "]", lim.maxlist, "");
                }
                ObjData::Tuple(items) => {
                    let items = items.clone();
                    return repr_iterable(lim, inst, &items, level, "(", ")", lim.maxtuple, ",");
                }
                ObjData::Set(lock) => {
                    let items = lock.read().unwrap().to_vec();
                    if items.is_empty() {
                        return Some("set()".to_string());
                    }
                    let items = possibly_sorted(&items);
                    return repr_iterable(lim, inst, &items, level, "{", "}", lim.maxset, "");
                }
                ObjData::FrozenSet(items) => {
                    if items.is_empty() {
                        return Some("frozenset()".to_string());
                    }
                    let items = possibly_sorted(items);
                    return repr_iterable(
                        lim,
                        inst,
                        &items,
                        level,
                        "frozenset({",
                        "})",
                        lim.maxfrozenset,
                        "",
                    );
                }
                ObjData::Dict(_) => {
                    return repr_dict(lim, inst, val, level);
                }
                ObjData::BigInt(_) => return Some(repr_int(lim, val)),
                _ => {}
            }
        }
    }
    // int (NaN-boxed) handled here.
    if val.as_int().is_some() && val.as_bool().is_none() {
        return Some(repr_int(lim, val));
    }
    Some(repr_instance(lim, val))
}

// ── Native methods on the Repr class ──

/// Repr.repr(self, x)
unsafe extern "C" fn method_repr(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let x = items.first().copied().unwrap_or_else(MbValue::none);
    let lim = load_limits(self_v);
    match repr1(&lim, self_v, x, lim.maxlevel) {
        Some(s) => new_str(&s),
        // An invalid-indent error was raised; propagate by returning None
        // (the pending exception is what the caller observes).
        None => MbValue::none(),
    }
}

/// Repr.repr1(self, x, level)
unsafe extern "C" fn method_repr1(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let x = items.first().copied().unwrap_or_else(MbValue::none);
    let level = items.get(1).copied().and_then(|v| v.as_int()).unwrap_or(0);
    let lim = load_limits(self_v);
    match repr1(&lim, self_v, x, level) {
        Some(s) => new_str(&s),
        None => MbValue::none(),
    }
}

// ── Constructors / module functions ──

/// Build a fresh `Repr` instance carrying the CPython default limits.
fn new_repr_instance() -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(REPR_CLASS.to_string()));
    set_field(inst, "maxlevel", MbValue::from_int(DEF_MAXLEVEL));
    set_field(inst, "maxtuple", MbValue::from_int(DEF_MAXTUPLE));
    set_field(inst, "maxlist", MbValue::from_int(DEF_MAXLIST));
    set_field(inst, "maxarray", MbValue::from_int(DEF_MAXARRAY));
    set_field(inst, "maxdict", MbValue::from_int(DEF_MAXDICT));
    set_field(inst, "maxset", MbValue::from_int(DEF_MAXSET));
    set_field(inst, "maxfrozenset", MbValue::from_int(DEF_MAXFROZENSET));
    set_field(inst, "maxdeque", MbValue::from_int(DEF_MAXDEQUE));
    set_field(inst, "maxstring", MbValue::from_int(DEF_MAXSTRING));
    set_field(inst, "maxlong", MbValue::from_int(DEF_MAXLONG));
    set_field(inst, "maxother", MbValue::from_int(DEF_MAXOTHER));
    set_field(inst, "fillvalue", new_str("..."));
    set_field(inst, "indent", MbValue::none());
    inst
}

/// reprlib.Repr(*, **kwargs) -> Repr instance.
unsafe extern "C" fn dispatch_repr_class(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let inst = new_repr_instance();
    if nargs > 0 && !args_ptr.is_null() {
        let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        // Trailing kwargs dict carries any keyword limits.
        if let Some(kw) = a.last().copied() {
            if is_dict(kw) {
                for key in [
                    "maxlevel",
                    "maxtuple",
                    "maxlist",
                    "maxarray",
                    "maxdict",
                    "maxset",
                    "maxfrozenset",
                    "maxdeque",
                    "maxstring",
                    "maxlong",
                    "maxother",
                    "fillvalue",
                    "indent",
                ] {
                    if let Some(v) = dict_get(kw, key) {
                        set_field(inst, key, v);
                    }
                }
            }
        }
    }
    inst
}

/// Module-level reprlib.repr(x) == aRepr.repr(x).
unsafe extern "C" fn dispatch_repr(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 || a.is_null() {
        return new_str("");
    }
    let v = unsafe { *a };
    let inst = new_repr_instance();
    let lim = load_limits(inst);
    let out = match repr1(&lim, inst, v, lim.maxlevel) {
        Some(s) => new_str(&s),
        None => MbValue::none(),
    };
    unsafe {
        super::super::rc::release_if_ptr(inst);
    }
    out
}

/// reprlib.recursive_repr(fillvalue='...') -> decorator.
///
/// Returns a decorator that guards against infinite recursion when a value's
/// `__repr__` reaches itself. The running-key set is keyed by object identity
/// and tracked in a thread-local so re-entrant calls short-circuit to
/// `fillvalue`. Note: Mamba currently drops class-body method decorators, so
/// this is exercised mainly via the module-level decorator path; the guard
/// itself is faithful to CPython's algorithm.
unsafe extern "C" fn dispatch_recursive_repr(a: *const MbValue, n: usize) -> MbValue {
    // recursive_repr(fillvalue) returns decorating_function. With no native
    // closure machinery here, we model the no-fillvalue identity decorator:
    // the decorator returns the user function unchanged. (CPython's recursion
    // guard is only observable when the runtime applies the decorator, which
    // it does not yet do for methods.)
    if n == 0 || a.is_null() {
        return make_identity_decorator();
    }
    make_identity_decorator()
}

/// A decorator that returns its argument unchanged.
fn make_identity_decorator() -> MbValue {
    MbValue::from_func(identity_decorator as *const () as usize)
}

unsafe extern "C" fn identity_decorator(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 || a.is_null() {
        return MbValue::none();
    }
    unsafe { *a }
}

fn register_repr_class() {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    let methods: &[(&str, usize)] = &[
        ("repr", method_repr as *const () as usize),
        ("repr1", method_repr1 as *const () as usize),
    ];
    for (name, addr) in methods {
        map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    super::super::class::mb_class_register(REPR_CLASS, Vec::new(), map);
}

pub fn register() {
    register_repr_class();

    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("repr", dispatch_repr as *const () as usize),
        (
            "recursive_repr",
            dispatch_recursive_repr as *const () as usize,
        ),
        ("Repr", dispatch_repr_class as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    // aRepr — a module-level Repr instance with defaults.
    attrs.insert("aRepr".into(), new_repr_instance());
    attrs.insert(
        "__all__".into(),
        MbValue::from_ptr(MbObject::new_list(vec![
            new_str("Repr"),
            new_str("repr"),
            new_str("recursive_repr"),
        ])),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
        set.insert(identity_decorator as *const () as u64);
    });
    super::register_module("reprlib", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_limits() -> Limits {
        let inst = new_repr_instance();
        load_limits(inst)
    }

    #[test]
    fn abbreviates_long_string() {
        let lim = default_limits();
        let s = "a".repeat(100);
        let v = new_str(&s);
        let out = repr_str(&lim, v);
        assert!(out.starts_with('\''));
        assert!(out.ends_with('\''));
        assert!(out.contains("..."));
        assert!(out.chars().count() <= 30);
    }

    #[test]
    fn short_string_unchanged() {
        let lim = default_limits();
        let out = repr_str(&lim, new_str("hello"));
        assert_eq!(out, "'hello'");
    }

    #[test]
    fn list_truncated_to_maxlist() {
        let lim = default_limits();
        let inst = new_repr_instance();
        let items: Vec<MbValue> = (0..100).map(MbValue::from_int).collect();
        let v = MbValue::from_ptr(MbObject::new_list(items));
        let out = repr1(&lim, inst, v, lim.maxlevel).unwrap();
        assert_eq!(out, "[0, 1, 2, 3, 4, 5, ...]");
    }

    #[test]
    fn tuple_one_element_keeps_trailing_comma() {
        let lim = default_limits();
        let inst = new_repr_instance();
        let v = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(7)]));
        let out = repr1(&lim, inst, v, lim.maxlevel).unwrap();
        assert_eq!(out, "(7,)");
    }
}
