use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// reprlib module for Mamba (#1261 long-tail).
///
/// CPython-style abbreviating repr. Mirrors the `reprlib.Repr` default
/// limits — `maxstring=30`, `maxlist=6`, `maxtuple=6`, `maxset=6`,
/// `maxfrozenset=6`, `maxdict=4`, `maxother=30`, `maxlevel=6`. Anything
/// else (numbers, None, functions, etc.) flows through `builtins.repr`
/// with the `maxother` cap.
use std::collections::HashMap;

const MAXLEVEL: u32 = 6;
const MAXSTRING: usize = 30;
const MAXLIST: usize = 6;
const MAXTUPLE: usize = 6;
const MAXSET: usize = 6;
const MAXFROZENSET: usize = 6;
const MAXDICT: usize = 4;
const MAXOTHER: usize = 30;

fn mb_repr_str(val: MbValue) -> String {
    let r = super::super::builtins::mb_repr(val);
    unsafe {
        if let Some(p) = r.as_ptr() {
            if let ObjData::Str(ref s) = (*p).data {
                return s.clone();
            }
        }
    }
    String::new()
}

/// Char-aware middle truncation: keep head and tail, drop the middle.
/// Matches CPython's `_possibly_sorted` slicing on `repr` output.
fn shorten_repr(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    if max < 5 {
        // Degenerate budgets — leave a minimal marker.
        return "...".to_string();
    }
    let half = (max - 3) / 2;
    let extra = (max - 3) % 2;
    let head_n = half + extra;
    let tail_n = half;
    let chars: Vec<char> = s.chars().collect();
    let head: String = chars.iter().take(head_n).collect();
    let tail: String = chars.iter().skip(chars.len() - tail_n).collect();
    format!("{}...{}", head, tail)
}

fn abbreviate_string(s: &str) -> String {
    // CPython truncates the *unquoted* contents and then re-quotes:
    //   repr(reprlib.repr('a'*100)) ==  "'aaaaaaaaaaaaa...aaaaaaaaaaaaaa'"
    // (30-char budget includes both quotes, so 28 inner chars max.)
    let max_inner = MAXSTRING.saturating_sub(2);
    let truncated = if s.chars().count() <= max_inner {
        s.to_string()
    } else {
        shorten_repr(s, max_inner)
    };
    // Re-render through mb_repr so escaping matches CPython quoting.
    let val = MbValue::from_ptr(MbObject::new_str(truncated));
    let out = mb_repr_str(val);
    unsafe {
        super::super::rc::release_if_ptr(val);
    }
    out
}

fn repr_with_level(val: MbValue, level: u32) -> String {
    if level == 0 {
        // CPython renders the empty bracket pair when depth is exhausted.
        if let Some(p) = val.as_ptr() {
            unsafe {
                match &(*p).data {
                    ObjData::List(_) => return "[...]".to_string(),
                    ObjData::Tuple(_) => return "(...)".to_string(),
                    ObjData::Dict(_) => return "{...}".to_string(),
                    ObjData::Set(_) | ObjData::FrozenSet(_) => return "{...}".to_string(),
                    _ => {}
                }
            }
        }
    }

    if let Some(p) = val.as_ptr() {
        unsafe {
            match &(*p).data {
                ObjData::Str(s) => return abbreviate_string(s),
                ObjData::List(lock) => {
                    let items = lock.read().unwrap();
                    return format_sequence(&items, level, "[", "]", MAXLIST);
                }
                ObjData::Tuple(items) => {
                    let n = items.len();
                    let mut out = format_sequence(items, level, "(", ")", MAXTUPLE);
                    if n == 1 {
                        // Single-element tuple: drop the trailing ')' and add ',)'.
                        if out.ends_with(')') {
                            out.pop();
                            out.push_str(",)");
                        }
                    }
                    return out;
                }
                ObjData::Set(lock) => {
                    let items = lock.read().unwrap();
                    if items.is_empty() {
                        return "set()".to_string();
                    }
                    return format_sequence(&items, level, "{", "}", MAXSET);
                }
                ObjData::FrozenSet(items) => {
                    if items.is_empty() {
                        return "frozenset()".to_string();
                    }
                    return format!(
                        "frozenset({})",
                        format_sequence(items, level, "{", "}", MAXFROZENSET)
                    );
                }
                ObjData::Dict(lock) => {
                    let map = lock.read().unwrap();
                    if map.is_empty() {
                        return "{}".to_string();
                    }
                    let total = map.len();
                    let take = MAXDICT.min(total);
                    let mut parts: Vec<String> = Vec::with_capacity(take + 1);
                    for (k, v) in map.iter().take(take) {
                        let key_val = super::super::dict_ops::dict_key_to_mbvalue(k);
                        let key_str = repr_with_level(key_val, level.saturating_sub(1));
                        super::super::rc::release_if_ptr(key_val);
                        let val_str = repr_with_level(*v, level.saturating_sub(1));
                        parts.push(format!("{}: {}", key_str, val_str));
                    }
                    if total > take {
                        parts.push("...".to_string());
                    }
                    return format!("{{{}}}", parts.join(", "));
                }
                _ => {}
            }
        }
    }

    // Fallback: full mb_repr + maxother truncation.
    let s = mb_repr_str(val);
    if s.chars().count() <= MAXOTHER {
        s
    } else {
        shorten_repr(&s, MAXOTHER)
    }
}

fn format_sequence(
    items: &[MbValue],
    level: u32,
    open: &str,
    close: &str,
    max_items: usize,
) -> String {
    let total = items.len();
    let take = max_items.min(total);
    let mut parts: Vec<String> = Vec::with_capacity(take + 1);
    for v in items.iter().take(take) {
        parts.push(repr_with_level(*v, level.saturating_sub(1)));
    }
    if total > take {
        parts.push("...".to_string());
    }
    format!("{}{}{}", open, parts.join(", "), close)
}

unsafe extern "C" fn dispatch_repr(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 || a.is_null() {
        return MbValue::from_ptr(MbObject::new_str("".to_string()));
    }
    let v = unsafe { *a };
    let s = repr_with_level(v, MAXLEVEL);
    MbValue::from_ptr(MbObject::new_str(s))
}

unsafe extern "C" fn dispatch_recursive_repr(a: *const MbValue, _n: usize) -> MbValue {
    // reprlib.recursive_repr(fillvalue="...") returns a decorator. In Mamba
    // we don't model decorators here, so return the input verbatim — calls
    // like `@reprlib.recursive_repr()` would just no-op.
    unsafe { *a }
}

unsafe extern "C" fn dispatch_repr_class(_a: *const MbValue, _n: usize) -> MbValue {
    // reprlib.Repr() — return an empty dict carrying the class shape.
    let d = MbObject::new_dict();
    MbValue::from_ptr(d)
}

pub fn register() {
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
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("reprlib", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abbreviates_long_string() {
        let s = "a".repeat(100);
        let v = MbValue::from_ptr(MbObject::new_str(s));
        let out = repr_with_level(v, MAXLEVEL);
        // 28-char inner budget + 2 quotes = 30; abbreviated, contains '...'.
        assert!(out.starts_with('\''));
        assert!(out.ends_with('\''));
        assert!(out.contains("..."));
        assert!(out.chars().count() <= MAXSTRING);
    }

    #[test]
    fn short_string_unchanged() {
        let v = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let out = repr_with_level(v, MAXLEVEL);
        assert_eq!(out, "'hello'");
    }

    #[test]
    fn list_truncated_to_maxlist() {
        let items: Vec<MbValue> = (0..100).map(MbValue::from_int).collect();
        let v = MbValue::from_ptr(MbObject::new_list(items));
        let out = repr_with_level(v, MAXLEVEL);
        assert_eq!(out, "[0, 1, 2, 3, 4, 5, ...]");
    }

    #[test]
    fn short_list_unchanged() {
        let items: Vec<MbValue> = (0..3).map(MbValue::from_int).collect();
        let v = MbValue::from_ptr(MbObject::new_list(items));
        let out = repr_with_level(v, MAXLEVEL);
        assert_eq!(out, "[0, 1, 2]");
    }

    #[test]
    fn dict_truncated_to_maxdict() {
        use super::super::super::dict_ops::DictKey;
        let dict = MbObject::new_dict_with_capacity(20);
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                for i in 0..20i64 {
                    map.insert(DictKey::Int(i), MbValue::from_int(i));
                }
            }
        }
        let v = MbValue::from_ptr(dict);
        let out = repr_with_level(v, MAXLEVEL);
        // first 4 entries + ', ...'
        assert_eq!(out, "{0: 0, 1: 1, 2: 2, 3: 3, ...}");
    }

    #[test]
    fn tuple_one_element_keeps_trailing_comma() {
        let v = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(7)]));
        let out = repr_with_level(v, MAXLEVEL);
        assert_eq!(out, "(7,)");
    }
}
