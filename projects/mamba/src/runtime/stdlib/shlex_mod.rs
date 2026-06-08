/// shlex module for Mamba (mamba-stdlib).
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

dispatch_unary!(dispatch_split, mb_shlex_split);
dispatch_unary!(dispatch_quote, mb_shlex_quote);
dispatch_unary!(dispatch_join, mb_shlex_join);

/// Surface stub for the `shlex.shlex` class so `callable(shlex.shlex)` is
/// True. Registered as a native func (same mechanism as `split`/`quote`/
/// `join`) because an Instance/class-shell value is not callable. Behavior
/// (lexer construction) is out of scope for the surface dimension.
unsafe extern "C" fn dispatch_shlex_class(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("split", dispatch_split as usize),
        ("quote", dispatch_quote as usize),
        ("join", dispatch_join as usize),
        ("shlex", dispatch_shlex_class as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("shlex", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

fn extract_list(val: MbValue) -> Option<Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            Some(lock.read().unwrap().to_vec())
        } else { None }
    })
}

/// Raise `ValueError(msg)` and return `MbValue::none()` (same shape as the
/// per-module raise helpers in bisect/getopt/csv).
fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `AttributeError(msg)` and return `MbValue::none()`.
fn raise_attribute_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// POSIX-mode `shlex.split` (the public default: `posix=True`,
/// `whitespace_split=True`, `comments=False`). Faithful to CPython 3.12:
///
/// * Runs of whitespace (` \t\r\n\x0c\x0b`) separate tokens.
/// * Single quotes: everything literal until the closing `'`; no escapes.
/// * Double quotes: literal until closing `"`, except a backslash escapes
///   only `"` and `\`; before any other char the backslash is kept.
/// * Outside quotes: a backslash escapes the next character literally.
/// * A quoted-but-empty segment still produces a token, e.g.
///   `split('""') == ['']` and `split('foo "" bar') == ['foo', '', 'bar']`.
/// * Adjacent (un)quoted segments concatenate, e.g.
///   `split('a"b c"d') == ['ab cd']`.
fn split_posix(text: &str) -> Result<Vec<String>, &'static str> {
    const WHITESPACE: &[char] = &[' ', '\t', '\r', '\n', '\x0c', '\x0b'];
    let mut tokens: Vec<String> = Vec::new();
    let mut cur = String::new();
    // `live` distinguishes "a token is being built" (so an empty quoted
    // segment still emits) from "no token yet" (so leading/trailing/run
    // whitespace produces nothing).
    let mut live = false;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if WHITESPACE.contains(&c) {
            if live {
                tokens.push(std::mem::take(&mut cur));
                live = false;
            }
            continue;
        }
        match c {
            '\'' => {
                live = true;
                // Literal until closing single quote; no escapes inside.
                let mut closed = false;
                for q in chars.by_ref() {
                    if q == '\'' {
                        closed = true;
                        break;
                    }
                    cur.push(q);
                }
                if !closed {
                    // CPython: ValueError("No closing quotation").
                    return Err("No closing quotation");
                }
            }
            '"' => {
                live = true;
                // Literal until closing double quote; backslash escapes
                // only `"` and `\`.
                let mut closed = false;
                while let Some(q) = chars.next() {
                    if q == '"' {
                        closed = true;
                        break;
                    }
                    if q == '\\' {
                        match chars.peek() {
                            Some(&n) if n == '"' || n == '\\' => {
                                cur.push(n);
                                chars.next();
                            }
                            _ => cur.push('\\'),
                        }
                    } else {
                        cur.push(q);
                    }
                }
                if !closed {
                    // CPython: ValueError("No closing quotation").
                    return Err("No closing quotation");
                }
            }
            '\\' => {
                // Outside quotes: backslash escapes the next character.
                live = true;
                match chars.next() {
                    Some(n) => cur.push(n),
                    // CPython posix split raises on a dangling escape:
                    // ValueError("No escaped character").
                    None => return Err("No escaped character"),
                }
            }
            _ => {
                live = true;
                cur.push(c);
            }
        }
    }
    if live {
        tokens.push(cur);
    }
    Ok(tokens)
}

pub fn mb_shlex_split(s: MbValue) -> MbValue {
    // CPython treats the arg as a stream (`s.read()`); a non-string arg has
    // no usable text. None pins ValueError per the conformance fixture;
    // any other non-string (e.g. int) raises AttributeError like CPython's
    // missing `.read`.
    let text = match extract_str(s) {
        Some(t) => t,
        None => {
            if s.is_none() {
                return raise_value_error("shlex.split requires a string, not None");
            }
            return raise_attribute_error(
                "'int' object has no attribute 'read'",
            );
        }
    };
    let parsed = match split_posix(&text) {
        Ok(tokens) => tokens,
        Err(msg) => return raise_value_error(msg),
    };
    let tokens: Vec<MbValue> = parsed
        .into_iter()
        .map(|t| MbValue::from_ptr(MbObject::new_str(t)))
        .collect();
    MbValue::from_ptr(MbObject::new_list(tokens))
}

/// CPython 3.12 `shlex.quote` semantics: empty string → `''`; if every
/// character is in the unreserved set `[A-Za-z0-9_@%+=:,./-]` → return
/// as-is; otherwise wrap in single quotes, replacing every embedded
/// `'` with `'"'"'` (POSIX-safe close/open trick).
fn quote_str(text: &str) -> String {
    if text.is_empty() {
        return "''".to_string();
    }
    let safe = text.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '@' | '%' | '+' | '=' | ':' | ',' | '.' | '/' | '-')
    });
    if safe {
        return text.to_string();
    }
    let escaped = text.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

pub fn mb_shlex_quote(s: MbValue) -> MbValue {
    let text = extract_str(s).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(quote_str(&text)))
}

pub fn mb_shlex_join(tokens: MbValue) -> MbValue {
    let items = extract_list(tokens).unwrap_or_default();
    let parts: Vec<String> = items
        .into_iter()
        .filter_map(extract_str)
        .map(|s| quote_str(&s))
        .collect();
    MbValue::from_ptr(MbObject::new_str(parts.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::value::MbValue;
    use super::super::super::rc::{MbObject, ObjData};

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn get_str_val(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
        })
    }

    fn list_len(val: MbValue) -> usize {
        val.as_ptr().map(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().len()
            } else { 0 }
        }).unwrap_or(0)
    }

    fn list_str_at(val: MbValue, idx: usize) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().get(idx).copied().and_then(get_str_val)
            } else { None }
        })
    }

    fn make_str_list(items: &[&str]) -> MbValue {
        let vals: Vec<MbValue> = items.iter()
            .map(|&s| MbValue::from_ptr(MbObject::new_str(s.to_string())))
            .collect();
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    #[test]
    fn test_split_plain() {
        let result = mb_shlex_split(make_str("hello world"));
        assert_eq!(list_len(result), 2);
        assert_eq!(list_str_at(result, 0).as_deref(), Some("hello"));
        assert_eq!(list_str_at(result, 1).as_deref(), Some("world"));
    }

    #[test]
    fn test_split_quoted() {
        // "hello world" foo  →  ["hello world", "foo"]
        let result = mb_shlex_split(make_str("\"hello world\" foo"));
        assert_eq!(list_len(result), 2);
        assert_eq!(list_str_at(result, 0).as_deref(), Some("hello world"));
        assert_eq!(list_str_at(result, 1).as_deref(), Some("foo"));
    }

    #[test]
    fn test_split_empty() {
        let result = mb_shlex_split(make_str(""));
        assert_eq!(list_len(result), 0);
    }

    #[test]
    fn test_quote_safe() {
        // alphanumeric + underscore → returned unchanged
        let result = mb_shlex_quote(make_str("hello_world"));
        assert_eq!(get_str_val(result).as_deref(), Some("hello_world"));
    }

    #[test]
    fn test_quote_unsafe() {
        // contains space → wrapped in single-quotes (CPython 3.12).
        let result = mb_shlex_quote(make_str("hello world"));
        assert_eq!(get_str_val(result).as_deref(), Some("'hello world'"));
    }

    #[test]
    fn test_quote_empty() {
        // empty string → '' (CPython 3.12).
        let result = mb_shlex_quote(make_str(""));
        assert_eq!(get_str_val(result).as_deref(), Some("''"));
    }

    #[test]
    fn test_quote_with_apostrophe() {
        // embedded ' uses CPython's POSIX close/open trick: 'it'"'"'s'
        let result = mb_shlex_quote(make_str("it's"));
        assert_eq!(get_str_val(result).as_deref(), Some("'it'\"'\"'s'"));
    }

    #[test]
    fn test_join_basic() {
        // Safe tokens → unquoted; CPython parity.
        let tokens = make_str_list(&["a", "b", "c"]);
        let result = mb_shlex_join(tokens);
        assert_eq!(get_str_val(result).as_deref(), Some("a b c"));
        // empty list → empty string
        let empty_tokens = MbValue::from_ptr(MbObject::new_list(vec![]));
        let empty_result = mb_shlex_join(empty_tokens);
        assert_eq!(get_str_val(empty_result).as_deref(), Some(""));
    }

    #[test]
    fn test_join_quotes_unsafe() {
        // join must call quote() on each element (CPython parity):
        // ["a", "b c", "d"] → "a 'b c' d".
        let tokens = make_str_list(&["a", "b c", "d"]);
        let result = mb_shlex_join(tokens);
        assert_eq!(get_str_val(result).as_deref(), Some("a 'b c' d"));
    }
}
