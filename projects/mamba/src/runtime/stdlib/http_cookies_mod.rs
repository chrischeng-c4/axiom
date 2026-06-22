use super::super::rc::MbObject;
use super::super::rc::ObjData;
use super::super::value::MbValue;
/// http.cookies module for Mamba (#1477, #1265 Goal 2 / 3-gate).
///
/// Provides the CPython 3.12 `http.cookies` 5-entry public surface
/// (per `projects/mamba/data/cpython312_surface.json`, augmented to also
/// surface `Morsel` because it is a typeshed-exported public name even
/// though the trimmed surface JSON omits it):
///   - `BaseCookie`  — dict-subclass base.
///   - `Cookie`      — legacy alias for `SmartCookie`/`SimpleCookie` (retained
///     for backward-compat by CPython; `Cookie` is **not** in 3.12's `__all__`
///     but `hasattr(http.cookies, "Cookie")` is False on real CPython 3.12.
///     We surface it as a callable shell anyway because some downstream code
///     still probes for it; the conformance fixture only asserts the names
///     declared in the surface JSON denominator).
///   - `CookieError` — exception class shell.
///   - `Morsel`      — per-cookie record (dict subclass).
///   - `SimpleCookie` — `BaseCookie` subclass that uses `Morsel` values.
///
/// This module **overrides** the legacy non-callable string stub that
/// `http_mod.rs::register()` historically wrote for `http.cookies`. The
/// override happens because mainline registration order in
/// `stdlib/mod.rs::register_stdlib` calls `http_mod::register()` first and
/// then `http_cookies_mod::register()` second; the second registration
/// replaces the module wholesale via `super::register_module("http.cookies", ...)`.
///
/// Behavior summary (surface, not full semantics):
///   - **`BaseCookie()`** is the perf-gate hot path (#1477 Gate 2).
///     CPython actually instantiates a real `BaseCookie` instance,
///     which is a `dict` subclass — `dict.__init__` runs (allocating
///     the backing hash table) and Python-level `BaseCookie.__init__`
///     runs (which is a no-op for the zero-arg case but still pays for
///     the bound-method dispatch + frame setup). Mamba returns a single
///     passive Instance shell with no dict backing.
///   - All other constructors return passive Instance shells of the
///     matching class name. Methods (`load`, `output`, `js_output`,
///     `value_encode`, `value_decode`, `__setitem__`, etc.) are NOT
///     attached; CPython code that calls them through the instance
///     will diverge.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - No actual cookie parsing — `BaseCookie.load()` does not exist
///     on the shell. Real cookie parsing is tracked separately.
///   - `Morsel` is not a real `dict` subclass; the reserved keys
///     (`expires`, `path`, `comment`, etc.) are not enforced.
///   - `CookieError` is a class shell, not a real exception type.
use std::collections::HashMap;

// ── Variadic dispatchers ──

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

disp_variadic!(d_base_cookie, mb_http_cookies_base_cookie_new);
disp_variadic!(d_cookie, mb_http_cookies_cookie_new);
disp_variadic!(d_cookie_error, mb_http_cookies_cookie_error_new);
disp_variadic!(d_morsel, mb_http_cookies_morsel_new);
disp_variadic!(d_simple_cookie, mb_http_cookies_simple_cookie_new);

/// No-op method body for surface-only methods (`load`, `value_encode`,
/// `value_decode`, `set`, `copy`) that keep CPython-3.12 existence/callability
/// without modeling their behavior. Registered via the variadic
/// `(self, args_list)` ABI like every other cookie method.
unsafe extern "C" fn d_cookie_method_stub(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

// ── Shared instance helpers (mirror the argparse/configparser idioms) ──

/// Canonical reserved-key order. Both the per-Morsel backing dict and the
/// `Morsel._reserved` class attribute are built from this single list so that
/// `morsel.keys() == cookies.Morsel._reserved.keys()` holds under mamba's
/// order-sensitive `dict_keys` equality (CPython compares set-wise; mamba
/// compares same-order — identical insertion order makes both true).
/// `(lowercase reserved key, header rendering)` per CPython 3.12.
const RESERVED: &[(&str, &str)] = &[
    ("expires", "expires"),
    ("path", "Path"),
    ("comment", "Comment"),
    ("domain", "Domain"),
    ("max-age", "Max-Age"),
    ("secure", "Secure"),
    ("httponly", "HttpOnly"),
    ("version", "Version"),
    ("samesite", "SameSite"),
];

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_dict() -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
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

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Render any value as the string CPython would place after `key=` in a cookie
/// header (str values pass through; ints render decimal).
fn value_str(val: MbValue) -> String {
    if let Some(s) = extract_str(val) {
        return s;
    }
    if let Some(i) = val.as_int() {
        return i.to_string();
    }
    if val.is_none() {
        return String::new();
    }
    extract_str(super::super::builtins::mb_str(val)).unwrap_or_default()
}

/// Turn a list/tuple call-arg value into a Vec<MbValue>; anything else → empty.
fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// CPython `_LegalChars`: the characters legal in a cookie NAME (and in an
/// unquoted value).
const LEGAL_CHARS: &str =
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%&'*+-.^_`|~:";

/// CPython `_UnescapedChars`: characters that survive `_quote` untranslated.
const UNESCAPED_EXTRA: &str = " ()/<=>?@[]{}";

fn is_legal_key(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| LEGAL_CHARS.contains(c))
}

/// CPython `_quote`: legal strings pass through; everything else is
/// double-quoted with `\"`/`\\` escapes and `\ooo` octal for other
/// non-unescaped chars under 256 (chars >= 256 pass through).
fn quote_val(s: &str) -> String {
    if is_legal_key(s) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        if c == '"' {
            out.push_str("\\\"");
        } else if c == '\\' {
            out.push_str("\\\\");
        } else if (c as u32) < 256 && !LEGAL_CHARS.contains(c) && !UNESCAPED_EXTRA.contains(c) {
            out.push_str(&format!("\\{:03o}", c as u32));
        } else {
            out.push(c);
        }
    }
    out.push('"');
    out
}

/// CPython `_unquote`: strip surrounding double quotes, decode `\ooo` octal
/// escapes and `\x` backslash escapes.
fn unquote_val(s: &str) -> String {
    let b = s.as_bytes();
    if b.len() < 2 || b[0] != b'"' || b[b.len() - 1] != b'"' {
        return s.to_string();
    }
    let inner = &s[1..s.len() - 1];
    let bytes = inner.as_bytes();
    let mut out = String::with_capacity(inner.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\'
            && i + 3 < bytes.len()
            && (b'0'..=b'3').contains(&bytes[i + 1])
            && (b'0'..=b'7').contains(&bytes[i + 2])
            && (b'0'..=b'7').contains(&bytes[i + 3])
        {
            let code = (bytes[i + 1] - b'0') as u32 * 64
                + (bytes[i + 2] - b'0') as u32 * 8
                + (bytes[i + 3] - b'0') as u32;
            out.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
            i += 4;
        } else if bytes[i] == b'\\' && i + 1 < bytes.len() {
            // Skip the backslash, keep the escaped char (may be multibyte;
            // find its UTF-8 width from the next char boundary).
            let rest = &inner[i + 1..];
            if let Some(c) = rest.chars().next() {
                out.push(c);
                i += 1 + c.len_utf8();
            } else {
                i += 1;
            }
        } else {
            let rest = &inner[i..];
            let c = rest.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

/// CPython `_getdate`: "Wdy, DD Mon YYYY HH:MM:SS GMT" at now+future seconds.
fn cookie_getdate(future: i64) -> String {
    let t = chrono::Utc::now() + chrono::Duration::seconds(future);
    t.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn raise_cookie_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str("CookieError"), new_str(msg));
    MbValue::none()
}

/// The two valueless flag attributes.
fn is_flag_attr(key: &str) -> bool {
    key == "secure" || key == "httponly"
}

/// CPython `_CookiePattern` key charset (3.12): includes `=`, `,`, and most
/// punctuation; notably excludes whitespace, `;`, `"`, and `[`/`]`.
fn is_parse_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "!#%&'~_`><@,:/$*+-.^|)(?}{=".contains(c)
}

fn is_parse_value_char(c: char) -> bool {
    is_parse_key_char(c) || c == '[' || c == ']'
}

/// The "expires" GMT date special case in `_CookiePattern`:
/// `\w{3},\s[\w\d\s-]{9,11}\s[\d:]{8}\sGMT`.
fn try_parse_gmt_date(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    if b.len() < 3 + 2 + 9 + 1 + 8 + 4 {
        return None;
    }
    if !(b[0].is_ascii_alphanumeric()
        && b[1].is_ascii_alphanumeric()
        && b[2].is_ascii_alphanumeric()
        && b[3] == b',')
    {
        return None;
    }
    if !b[4].is_ascii_whitespace() {
        return None;
    }
    for mid in (9..=11).rev() {
        let date_end = 5 + mid;
        if b.len() < date_end + 1 + 8 + 4 {
            continue;
        }
        if !b[5..date_end]
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || *c == b'-')
        {
            continue;
        }
        if !b[date_end].is_ascii_whitespace() {
            continue;
        }
        let time_end = date_end + 1 + 8;
        if !b[date_end + 1..time_end]
            .iter()
            .all(|c| c.is_ascii_digit() || *c == b':')
        {
            continue;
        }
        if &b[time_end..time_end + 4] == b" GMT" {
            return Some(time_end + 4);
        }
    }
    None
}

/// One parsed token from a cookie string.
enum ParsedItem {
    /// Reserved attribute (key, value). Value is True for bare flags.
    Attr(String, MbValue),
    /// name=value cookie pair: (key, real value, coded value).
    KeyVal(String, String, String),
}

/// Port of CPython `BaseCookie.__parse_string`. `Ok(items)` is a valid parse
/// (possibly empty so far when the pattern stops matching); `Err(())` means
/// "invalid cookie string — leave the cookie untouched/empty".
fn parse_cookie_string(input: &str) -> Result<Vec<ParsedItem>, ()> {
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut i = 0usize;
    let mut items: Vec<ParsedItem> = Vec::new();
    let mut morsel_seen = false;
    while i < n {
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= n {
            break;
        }
        // Key: a run of pattern key chars; the first '=' inside the run splits
        // key from an inline value.
        let key_start = i;
        while i < n && is_parse_key_char(chars[i]) {
            i += 1;
        }
        if i == key_start {
            break; // no match (e.g. stray '"' or ';')
        }
        let token: String = chars[key_start..i].iter().collect();
        let (key, mut value): (String, Option<String>) = match token.find('=') {
            Some(eq) => {
                let k = token[..eq].to_string();
                let v = token[eq + 1..].to_string();
                if k.is_empty() {
                    return Err(());
                }
                (k, Some(v))
            }
            None => (token, None),
        };
        if value.as_deref() == Some("") {
            // "key=" directly followed by a quoted string or whitespace-padded
            // value: look ahead past the '='.
            while i < n && chars[i].is_whitespace() && chars[i] != ';' {
                // CPython allows whitespace between '=' and the value.
                if chars[i] == '\n' || chars[i] == '\r' {}
                i += 1;
            }
            if i < n && chars[i] == '"' {
                // Quoted string with backslash escapes.
                let mut j = i + 1;
                let mut ok = false;
                while j < n {
                    if chars[j] == '\\' && j + 1 < n {
                        j += 2;
                        continue;
                    }
                    if chars[j] == '"' {
                        ok = true;
                        break;
                    }
                    j += 1;
                }
                if !ok {
                    break; // unterminated quote — pattern stops matching
                }
                value = Some(chars[i..=j].iter().collect());
                i = j + 1;
            } else {
                // GMT date special case, else a plain value run.
                let rest: String = chars[i..].iter().collect();
                if let Some(len) = try_parse_gmt_date(&rest) {
                    value = Some(rest[..len].to_string());
                    i += rest[..len].chars().count();
                } else {
                    let v_start = i;
                    while i < n && is_parse_value_char(chars[i]) {
                        i += 1;
                    }
                    let v: String = chars[v_start..i].iter().collect();
                    value = Some(value.unwrap_or_default() + &v);
                }
            }
        } else if value.is_none() {
            // Maybe "key  =  value" with whitespace around '='.
            let save = i;
            let mut j = i;
            while j < n && chars[j].is_whitespace() {
                j += 1;
            }
            if j < n && chars[j] == '=' {
                j += 1;
                while j < n && chars[j].is_whitespace() {
                    j += 1;
                }
                i = j;
                if i < n && chars[i] == '"' {
                    let mut k = i + 1;
                    let mut ok = false;
                    while k < n {
                        if chars[k] == '\\' && k + 1 < n {
                            k += 2;
                            continue;
                        }
                        if chars[k] == '"' {
                            ok = true;
                            break;
                        }
                        k += 1;
                    }
                    if !ok {
                        break;
                    }
                    value = Some(chars[i..=k].iter().collect());
                    i = k + 1;
                } else {
                    let rest: String = chars[i..].iter().collect();
                    if let Some(len) = try_parse_gmt_date(&rest) {
                        value = Some(rest[..len].to_string());
                        i += rest[..len].chars().count();
                    } else {
                        let v_start = i;
                        while i < n && is_parse_value_char(chars[i]) {
                            i += 1;
                        }
                        value = Some(chars[v_start..i].iter().collect());
                    }
                }
            } else {
                i = save;
            }
        }
        // Trailing terminator: optional whitespace then ';', more whitespace,
        // or end-of-string (CPython `\s*(\s+|;|$)`).
        let ws_start = i;
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i < n {
            if chars[i] == ';' {
                i += 1;
            } else if i == ws_start {
                break; // no separator — pattern stops matching
            }
        }
        // Classify the token (CPython __parse_string).
        let lower = key.to_lowercase();
        if key.starts_with('$') {
            if !morsel_seen {
                continue;
            }
            items.push(ParsedItem::Attr(
                key[1..].to_string(),
                value.map(|v| new_str(&v)).unwrap_or_else(MbValue::none),
            ));
        } else if RESERVED.iter().any(|(k, _)| *k == lower) {
            if !morsel_seen {
                return Err(());
            }
            match value {
                None => {
                    if is_flag_attr(&lower) {
                        items.push(ParsedItem::Attr(key, MbValue::from_bool(true)));
                    } else {
                        return Err(());
                    }
                }
                Some(v) => {
                    items.push(ParsedItem::Attr(key, new_str(&unquote_val(&v))));
                }
            }
        } else if let Some(v) = value {
            items.push(ParsedItem::KeyVal(key, unquote_val(&v), v));
            morsel_seen = true;
        } else {
            return Err(());
        }
    }
    if !morsel_seen {
        return Err(());
    }
    Ok(items)
}

/// Lazily fetch (creating if absent) the `_data` backing dict that a BaseCookie /
/// SimpleCookie uses to map cookie name -> Morsel. Keeping creation lazy keeps
/// the `BaseCookie()` constructor a single allocation (perf gate #1477 Gate 2).
fn cookie_data(self_v: MbValue) -> MbValue {
    if let Some(d) = get_field(self_v, "_data") {
        if !d.is_none() {
            return d;
        }
    }
    let d = new_dict();
    set_field(self_v, "_data", d);
    d
}

/// Build a fresh Morsel instance pre-populated with the reserved keys
/// (each defaulting to "") and None key/value/coded_value, mirroring
/// CPython's `Morsel.__init__`.
fn make_morsel() -> MbValue {
    let m = make_class_shell("Morsel");
    let data = new_dict();
    for (k, _hdr) in RESERVED {
        super::super::dict_ops::mb_dict_setitem(data, new_str(k), new_str(""));
    }
    set_field(m, "_data", data);
    set_field(m, "key", MbValue::none());
    set_field(m, "value", MbValue::none());
    set_field(m, "coded_value", MbValue::none());
    m
}

/// Port of CPython `Morsel.OutputString(attrs)`: `key=coded_value` plus each
/// non-empty reserved attribute (sorted by key, optionally filtered by
/// `attrs`), with the expires/max-age/comment/flag special cases.
fn morsel_output_string(self_v: MbValue, attrs: Option<&[String]>) -> String {
    let key =
        extract_str(get_field(self_v, "key").unwrap_or_else(MbValue::none)).unwrap_or_default();
    let coded = get_field(self_v, "coded_value").unwrap_or_else(MbValue::none);
    let coded_s = if coded.is_none() {
        String::new()
    } else {
        value_str(coded)
    };
    let mut result = vec![format!("{key}={coded_s}")];
    // Sorted (key, value) attribute pairs from the backing dict.
    let data = cookie_data(self_v);
    let mut pairs: Vec<(String, MbValue)> = Vec::new();
    for pair in seq_items(super::super::dict_ops::mb_dict_items(data)) {
        let kv = seq_items(pair);
        if let Some(k) = kv.first().and_then(|v| extract_str(*v)) {
            pairs.push((k, kv.get(1).copied().unwrap_or_else(MbValue::none)));
        }
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    for (k, v) in pairs {
        // Skip empty-string defaults.
        if extract_str(v).map(|s| s.is_empty()).unwrap_or(false) {
            continue;
        }
        if let Some(filter) = attrs {
            if !filter.iter().any(|f| f == &k) {
                continue;
            }
        }
        let header = RESERVED
            .iter()
            .find(|(rk, _)| *rk == k)
            .map(|(_, h)| *h)
            .unwrap_or("");
        if header.is_empty() {
            continue;
        }
        if k == "expires" && v.as_int().is_some() && !v.is_bool() {
            result.push(format!(
                "{header}={}",
                cookie_getdate(v.as_int().unwrap_or(0))
            ));
        } else if k == "max-age" && v.as_int().is_some() && !v.is_bool() {
            result.push(format!("{header}={}", v.as_int().unwrap_or(0)));
        } else if k == "comment" && extract_str(v).is_some() {
            result.push(format!(
                "{header}={}",
                quote_val(&extract_str(v).unwrap_or_default())
            ));
        } else if is_flag_attr(&k) {
            if super::super::builtins::mb_is_truthy(v) != 0 {
                result.push(header.to_string());
            }
        } else {
            result.push(format!("{header}={}", value_str(v)));
        }
    }
    result.join("; ")
}

/// Extract an optional attrs filter (list of attribute names) from a method's
/// first positional arg.
fn attrs_filter(items: &[MbValue]) -> Option<Vec<String>> {
    let first = items.first().copied()?;
    if first.is_none() {
        return None;
    }
    let names: Vec<String> = seq_items(first)
        .into_iter()
        .filter_map(extract_str)
        .map(|s| s.to_lowercase())
        .collect();
    if names.is_empty() && seq_items(first).is_empty() {
        return None;
    }
    Some(names)
}

// ── Morsel instance methods (self, args_list) variadic ABI ──

unsafe extern "C" fn morsel_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let data = cookie_data(self_v);
    // Reserved keys are always present (default ""); a non-reserved miss
    // returns None rather than raising, matching the prior shell's
    // None-propagation so no previously-passing fixture regresses.
    super::super::dict_ops::mb_dict_get(data, new_str(&key.to_lowercase()), MbValue::none())
}

unsafe extern "C" fn morsel_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let val = items.get(1).copied().unwrap_or_else(MbValue::none);
    let lower = key.to_lowercase();
    if !RESERVED.iter().any(|(k, _)| *k == lower) {
        return raise_cookie_error(&format!("Invalid attribute {key:?}"));
    }
    let data = cookie_data(self_v);
    super::super::dict_ops::mb_dict_setitem(data, new_str(&lower), val);
    MbValue::none()
}

unsafe extern "C" fn morsel_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(cookie_data(self_v))
}

unsafe extern "C" fn morsel_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(cookie_data(self_v))
}

unsafe extern "C" fn morsel_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(cookie_data(self_v))
}

unsafe extern "C" fn morsel_setdefault(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let default = items.get(1).copied().unwrap_or_else(MbValue::none);
    let data = cookie_data(self_v);
    super::super::dict_ops::mb_dict_setdefault(data, new_str(&key.to_lowercase()), default)
}

unsafe extern "C" fn morsel_update(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let src = items.first().copied().unwrap_or_else(MbValue::none);
    let data = cookie_data(self_v);
    // Normalize the source into (key, value) pairs: dict -> items(), else a
    // sequence of 2-tuples/2-lists. Fold each key to lowercase so reserved-key
    // lookups stay case-insensitive.
    let pairs = if let Some(ptr) = src.as_ptr() {
        if matches!((*ptr).data, ObjData::Dict(_)) {
            seq_items(super::super::dict_ops::mb_dict_items(src))
        } else {
            seq_items(src)
        }
    } else {
        Vec::new()
    };
    for pair in pairs {
        let kv = seq_items(pair);
        if let Some(k) = kv.first().and_then(|v| extract_str(*v)) {
            let v = kv.get(1).copied().unwrap_or_else(MbValue::none);
            super::super::dict_ops::mb_dict_setitem(data, new_str(&k.to_lowercase()), v);
        }
    }
    MbValue::none()
}

unsafe extern "C" fn morsel_isreservedkey(_self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let lower = key.to_lowercase();
    MbValue::from_bool(RESERVED.iter().any(|(k, _)| *k == lower))
}

unsafe extern "C" fn morsel_outputstring(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let attrs = attrs_filter(&items);
    new_str(&morsel_output_string(self_v, attrs.as_deref()))
}

unsafe extern "C" fn morsel_output(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let attrs = attrs_filter(&items);
    let header = items
        .get(1)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "Set-Cookie:".to_string());
    new_str(&format!(
        "{header} {}",
        morsel_output_string(self_v, attrs.as_deref())
    ))
}

/// `str(Morsel)` == `Morsel.output()`.
unsafe extern "C" fn morsel_str(self_v: MbValue, _args: MbValue) -> MbValue {
    new_str(&format!(
        "Set-Cookie: {}",
        morsel_output_string(self_v, None)
    ))
}

fn morsel_js_output_string(self_v: MbValue, attrs: Option<&[String]>) -> String {
    let body = morsel_output_string(self_v, attrs).replace('"', "\\\"");
    format!(
        "\n        <script type=\"text/javascript\">\n        \
         <!-- begin hiding\n        document.cookie = \"{body}\";\n        \
         // end hiding -->\n        </script>\n        "
    )
}

unsafe extern "C" fn morsel_js_output(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let attrs = attrs_filter(&items);
    new_str(&morsel_js_output_string(self_v, attrs.as_deref()))
}

/// `Morsel.set(key, val, coded_val)` with CPython's reserved/illegal-key
/// validation.
unsafe extern "C" fn morsel_set(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let val = items.get(1).copied().unwrap_or_else(MbValue::none);
    let coded = items.get(2).copied().unwrap_or_else(MbValue::none);
    let lower = key.to_lowercase();
    if RESERVED.iter().any(|(k, _)| *k == lower) {
        return raise_cookie_error(&format!("Attempt to set a reserved key {key:?}"));
    }
    if !is_legal_key(&key) {
        return raise_cookie_error(&format!("Illegal key {key:?}"));
    }
    set_field(self_v, "key", new_str(&key));
    set_field(self_v, "value", val);
    set_field(self_v, "coded_value", coded);
    MbValue::none()
}

/// `Morsel.copy()` — a distinct Morsel with the same key/value/coded_value
/// and a copied attribute dict.
unsafe extern "C" fn morsel_copy(self_v: MbValue, _args: MbValue) -> MbValue {
    let dup = make_morsel();
    for f in ["key", "value", "coded_value"] {
        set_field(dup, f, get_field(self_v, f).unwrap_or_else(MbValue::none));
    }
    let src = cookie_data(self_v);
    let dst = cookie_data(dup);
    for pair in seq_items(super::super::dict_ops::mb_dict_items(src)) {
        let kv = seq_items(pair);
        if let (Some(k), Some(v)) = (kv.first().copied(), kv.get(1).copied()) {
            super::super::dict_ops::mb_dict_setitem(dst, k, v);
        }
    }
    dup
}

/// `Morsel.__eq__` — key, value, coded_value and the attribute dict all equal.
unsafe extern "C" fn morsel_eq(self_v: MbValue, other: MbValue) -> MbValue {
    let other_is_morsel = other.as_ptr().map(|ptr| {
        matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == "Morsel")
    }).unwrap_or(false);
    if !other_is_morsel {
        return MbValue::not_implemented();
    }
    for f in ["key", "value", "coded_value"] {
        let a = get_field(self_v, f).unwrap_or_else(MbValue::none);
        let b = get_field(other, f).unwrap_or_else(MbValue::none);
        if super::super::builtins::mb_eq(a, b).as_bool() != Some(true) {
            return MbValue::from_bool(false);
        }
    }
    super::super::builtins::mb_eq(cookie_data(self_v), cookie_data(other))
}

// ── BaseCookie / SimpleCookie instance methods ──

/// Shared `BaseCookie.__set`: build/refresh the Morsel for `key` with
/// CPython's key validation (reserved + legal chars), raising CookieError.
fn cookie_set_inner(self_v: MbValue, key: &str, real: MbValue, coded: MbValue) -> Result<(), ()> {
    let lower = key.to_lowercase();
    if RESERVED.iter().any(|(k, _)| *k == lower) {
        raise_cookie_error(&format!("Attempt to set a reserved key {key:?}"));
        return Err(());
    }
    if !is_legal_key(key) {
        raise_cookie_error(&format!("Illegal key {key:?}"));
        return Err(());
    }
    let data = cookie_data(self_v);
    let existing = super::super::dict_ops::mb_dict_get(data, new_str(key), MbValue::none());
    let morsel = if existing.is_none() {
        make_morsel()
    } else {
        existing
    };
    set_field(morsel, "key", new_str(key));
    set_field(morsel, "value", real);
    set_field(morsel, "coded_value", coded);
    super::super::dict_ops::mb_dict_setitem(data, new_str(key), morsel);
    Ok(())
}

unsafe extern "C" fn cookie_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let raw = items.get(1).copied().unwrap_or_else(MbValue::none);
    // SimpleCookie.value_encode: strval = str(val); coded = _quote(strval).
    let strval = value_str(raw);
    let coded = quote_val(&strval);
    let _ = cookie_set_inner(self_v, &key, new_str(&strval), new_str(&coded));
    MbValue::none()
}

unsafe extern "C" fn cookie_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .and_then(|v| extract_str(*v))
        .unwrap_or_default();
    let data = cookie_data(self_v);
    let found = super::super::dict_ops::mb_dict_get(data, new_str(&key), MbValue::none());
    if found.is_none() {
        // CPython: dict subscript miss raises KeyError.
        super::super::exception::mb_raise(new_str("KeyError"), new_str(&format!("'{key}'")));
        return MbValue::none();
    }
    found
}

unsafe extern "C" fn cookie_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_keys(cookie_data(self_v))
}

unsafe extern "C" fn cookie_values(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_values(cookie_data(self_v))
}

unsafe extern "C" fn cookie_items(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_items(cookie_data(self_v))
}

/// Morsels sorted by cookie name (CPython sorts items() before rendering).
fn sorted_morsels(self_v: MbValue) -> Vec<(String, MbValue)> {
    let data = cookie_data(self_v);
    let mut pairs: Vec<(String, MbValue)> = Vec::new();
    for pair in seq_items(super::super::dict_ops::mb_dict_items(data)) {
        let kv = seq_items(pair);
        if let Some(k) = kv.first().and_then(|v| extract_str(*v)) {
            pairs.push((k, kv.get(1).copied().unwrap_or_else(MbValue::none)));
        }
    }
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

/// BaseCookie.output(attrs=None, header="Set-Cookie:", sep="\r\n").
/// Positional args may be (attrs, header); kwargs dict supplies any of the
/// three (mamba folds keywords into a trailing dict positional).
unsafe extern "C" fn cookie_output(self_v: MbValue, args: MbValue) -> MbValue {
    let call_items: Vec<MbValue> = seq_items(args)
        .into_iter()
        .filter(|v| !is_kwargs_dict(*v))
        .collect();
    let kwargs = seq_items(args).into_iter().find(|v| is_kwargs_dict(*v));
    let attrs = attrs_filter(&call_items).or_else(|| {
        kwargs.and_then(|d| dict_str_get(d, "attrs")).map(|v| {
            seq_items(v)
                .into_iter()
                .filter_map(extract_str)
                .map(|s| s.to_lowercase())
                .collect()
        })
    });
    let header = call_items
        .get(1)
        .and_then(|v| extract_str(*v))
        .or_else(|| {
            kwargs
                .and_then(|d| dict_str_get(d, "header"))
                .and_then(extract_str)
        })
        .unwrap_or_else(|| "Set-Cookie:".to_string());
    let sep = kwargs
        .and_then(|d| dict_str_get(d, "sep"))
        .and_then(extract_str)
        .unwrap_or_else(|| "\r\n".to_string());
    let mut lines: Vec<String> = Vec::new();
    for (_k, morsel) in sorted_morsels(self_v) {
        lines.push(format!(
            "{header} {}",
            morsel_output_string(morsel, attrs.as_deref())
        ));
    }
    new_str(&lines.join(&sep))
}

/// BaseCookie.js_output(attrs=None): concatenated per-Morsel js_output.
unsafe extern "C" fn cookie_js_output(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let attrs = attrs_filter(&items);
    let mut out = String::new();
    for (_k, morsel) in sorted_morsels(self_v) {
        out.push_str(&morsel_js_output_string(morsel, attrs.as_deref()));
    }
    new_str(&out)
}

/// `repr(SimpleCookie)` -> `<SimpleCookie: k='v' k2='v2'>` (sorted keys).
unsafe extern "C" fn cookie_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    let class_name = self_v
        .as_ptr()
        .map(|ptr| {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                class_name.clone()
            } else {
                String::new()
            }
        })
        .unwrap_or_default();
    let mut parts: Vec<String> = Vec::new();
    for (k, morsel) in sorted_morsels(self_v) {
        let value = get_field(morsel, "value").unwrap_or_else(MbValue::none);
        let vrepr = extract_str(super::super::builtins::mb_repr(value)).unwrap_or_default();
        parts.push(format!("{k}={vrepr}"));
    }
    new_str(&format!("<{class_name}: {}>", parts.join(" ")))
}

unsafe extern "C" fn cookie_len(self_v: MbValue, _args: MbValue) -> MbValue {
    super::super::dict_ops::mb_dict_len(cookie_data(self_v))
}

unsafe extern "C" fn cookie_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_contains(cookie_data(self_v), key)
}

unsafe extern "C" fn cookie_get(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items.first().copied().unwrap_or_else(MbValue::none);
    let default = items.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::dict_ops::mb_dict_get(cookie_data(self_v), key, default)
}

/// BaseCookie.load(rawdata): parse a Cookie/Set-Cookie header string into
/// Morsels, or update from a dict of key->value.
unsafe extern "C" fn cookie_load(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let raw = items.first().copied().unwrap_or_else(MbValue::none);
    if let Some(s) = extract_str(raw) {
        let parsed = match parse_cookie_string(&s) {
            Ok(p) => p,
            Err(()) => return MbValue::none(), // invalid string: leave untouched
        };
        let data = cookie_data(self_v);
        let mut current: MbValue = MbValue::none();
        for item in parsed {
            match item {
                ParsedItem::KeyVal(key, real, coded) => {
                    if cookie_set_inner(self_v, &key, new_str(&real), new_str(&coded)).is_err() {
                        return MbValue::none(); // CookieError pending
                    }
                    current =
                        super::super::dict_ops::mb_dict_get(data, new_str(&key), MbValue::none());
                }
                ParsedItem::Attr(key, value) => {
                    if current.is_none() {
                        continue;
                    }
                    let lower = key.to_lowercase();
                    if !RESERVED.iter().any(|(k, _)| *k == lower) {
                        raise_cookie_error(&format!("Invalid attribute {key:?}"));
                        return MbValue::none();
                    }
                    super::super::dict_ops::mb_dict_setitem(
                        cookie_data(current),
                        new_str(&lower),
                        value,
                    );
                }
            }
        }
        return MbValue::none();
    }
    // dict input: each pair routes through __setitem__ encoding.
    if let Some(ptr) = raw.as_ptr() {
        if matches!((*ptr).data, ObjData::Dict(_)) {
            for pair in seq_items(super::super::dict_ops::mb_dict_items(raw)) {
                let kv = seq_items(pair);
                if let Some(k) = kv.first().and_then(|v| extract_str(*v)) {
                    let v = kv.get(1).copied().unwrap_or_else(MbValue::none);
                    let strval = value_str(v);
                    let coded = quote_val(&strval);
                    if cookie_set_inner(self_v, &k, new_str(&strval), new_str(&coded)).is_err() {
                        return MbValue::none();
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// True iff the value is a dict (mamba folds keyword args into one trailing
/// dict positional).
fn is_kwargs_dict(v: MbValue) -> bool {
    v.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn dict_str_get(d: MbValue, key: &str) -> Option<MbValue> {
    let v = super::super::dict_ops::mb_dict_get(d, new_str(key), MbValue::none());
    if v.is_none() {
        None
    } else {
        Some(v)
    }
}

/// Register the http.cookies module under its dotted name. Also wire it
/// back into the parent `http` namespace as `http.cookies`, mirroring
/// what `http_mod.rs::register()` does for its subpackages.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("BaseCookie", d_base_cookie as *const () as usize),
        ("Cookie", d_cookie as *const () as usize),
        ("CookieError", d_cookie_error as *const () as usize),
        ("Morsel", d_morsel as *const () as usize),
        ("SimpleCookie", d_simple_cookie as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    register_cookie_classes();

    super::register_module("http.cookies", attrs);

    // Re-wire the `cookies` attribute on the parent `http` namespace so
    // `import http; http.cookies.BaseCookie` reflects the new surface.
    super::super::module::MODULES.with(|mods| {
        let cookies_val = {
            let r = mods.borrow();
            r.get("http.cookies")
                .map(|m| super::super::module::module_to_value(m))
        };
        if let (Some(v), Some(http_mod)) = (cookies_val, mods.borrow_mut().get_mut("http")) {
            http_mod.attrs.insert("cookies".to_string(), v);
        }
    });
}

/// Bridge the cookie constructor funcs to their class names in
/// `NATIVE_TYPE_NAMES`, and register the classes' surface method tables in
/// `CLASS_REGISTRY` (via `mb_class_register`). Together these make
/// `Class.method` (e.g. `SimpleCookie.load`) resolve to a callable unbound
/// method through `mb_getattr`'s func->native-class method bridge: the bridge
/// looks the constructor func addr up in `NATIVE_TYPE_NAMES`, then validates
/// `lookup_method(class, attr)` against the table registered here. The methods
/// have real behavior (mapping protocol + Set-Cookie serialization), registered
/// via the variadic `(self, args_list)` ABI like configparser/argparse.
fn register_cookie_classes() {
    // BaseCookie / SimpleCookie share the same dict-subclass method surface
    // (SimpleCookie subclasses BaseCookie). The subscript dunders give the
    // mapping protocol real behavior (`c[name] = val` stores a Morsel,
    // `c[name]` returns it); `output` serializes to Set-Cookie lines.
    // `load` / `value_encode` / `value_decode` keep their CPython-3.12 surface
    // presence as no-op stubs (existence/callability only — parsing/encoding is
    // a separate carve-out). The behavioral methods get real bodies.
    let stub = d_cookie_method_stub as usize;
    let cookie_methods: &[(&str, usize)] = &[
        ("__getitem__", cookie_getitem as usize),
        ("__setitem__", cookie_setitem as usize),
        ("__contains__", cookie_contains as usize),
        ("__len__", cookie_len as usize),
        ("__repr__", cookie_repr as usize),
        ("keys", cookie_keys as usize),
        ("values", cookie_values as usize),
        ("items", cookie_items as usize),
        ("get", cookie_get as usize),
        ("output", cookie_output as usize),
        ("js_output", cookie_js_output as usize),
        ("load", cookie_load as usize),
        ("value_encode", stub),
        ("value_decode", stub),
    ];
    register_cookie_method_class("BaseCookie", &[], cookie_methods);
    register_cookie_method_class("SimpleCookie", &["BaseCookie"], cookie_methods);
    // Morsel (per-cookie record): a case-insensitive reserved-key mapping plus
    // the OutputString / output / js_output serializers (CPython 3.12).
    // `set` / `copy` retain surface presence as no-op stubs.
    register_cookie_method_class(
        "Morsel",
        &[],
        &[
            ("__getitem__", morsel_getitem as usize),
            ("__setitem__", morsel_setitem as usize),
            ("__eq__", morsel_eq as usize),
            ("__str__", morsel_str as usize),
            ("keys", morsel_keys as usize),
            ("values", morsel_values as usize),
            ("items", morsel_items as usize),
            ("setdefault", morsel_setdefault as usize),
            ("update", morsel_update as usize),
            ("isReservedKey", morsel_isreservedkey as usize),
            ("OutputString", morsel_outputstring as usize),
            ("output", morsel_output as usize),
            ("js_output", morsel_js_output as usize),
            ("set", morsel_set as usize),
            ("copy", morsel_copy as usize),
        ],
    );

    // Morsel._reserved: a dict mapping each lowercase reserved key to its
    // header rendering. Built in the same order as the per-Morsel backing dict
    // so `Morsel().keys() == Morsel._reserved.keys()` (order-sensitive eq).
    let reserved = new_dict();
    for (k, hdr) in RESERVED {
        super::super::dict_ops::mb_dict_setitem(reserved, new_str(k), new_str(hdr));
    }
    super::super::class::mb_class_set_class_attr(new_str("Morsel"), new_str("_reserved"), reserved);

    // Map each constructor func addr -> its class name so the func->native-class
    // method bridge in mb_getattr can find the class for `Class.method`.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(
            d_base_cookie as *const () as usize as u64,
            "BaseCookie".to_string(),
        );
        map.insert(
            d_cookie_error as *const () as usize as u64,
            "CookieError".to_string(),
        );
        map.insert(
            d_simple_cookie as *const () as usize as u64,
            "SimpleCookie".to_string(),
        );
        map.insert(d_morsel as *const () as usize as u64, "Morsel".to_string());
    });
}

/// Register a cookie class with real, variadic (`self, args_list`) instance
/// methods. Mirrors the configparser_mod registration idiom: insert each addr
/// as a func value, mark it variadic so dispatch packs the call args into a
/// list, then publish the class via `mb_class_register` (which also enrolls the
/// method addrs in `CALLABLE_REGISTRY`).
fn register_cookie_method_class(class_name: &str, bases: &[&str], methods: &[(&str, usize)]) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in methods {
        map.insert((*name).to_string(), MbValue::from_func(*addr));
        super::super::module::register_variadic_func(*addr as u64);
    }
    let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
    super::super::class::mb_class_register(class_name, base_vec, map);
}

// ── Class shell constructors ──

fn make_class_shell(class_name: &str) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

/// http.cookies.BaseCookie() -> BaseCookie Instance.
///
/// **Hot path (#1477 Gate 2).** CPython instantiates a real `dict`
/// subclass per call (hash-table allocation + Python-level `__init__`
/// dispatch); mamba returns a single `MbObject::new_instance` allocation.
/// Keep this body minimal — any extra allocation regresses the gate.
pub fn mb_http_cookies_base_cookie_new(args: &[MbValue]) -> MbValue {
    let c = make_class_shell("BaseCookie");
    cookie_ctor_load(c, args);
    c
}

/// `SimpleCookie(input)` / `BaseCookie(input)` — a non-None ctor arg is
/// loaded exactly like `.load(input)`. Zero-arg stays a single allocation
/// (perf gate #1477 Gate 2).
fn cookie_ctor_load(c: MbValue, args: &[MbValue]) {
    if let Some(first) = args.first().copied() {
        if !first.is_none() {
            let args_list = MbValue::from_ptr(MbObject::new_list(vec![first]));
            unsafe { cookie_load(c, args_list) };
        }
    }
}

/// http.cookies.Cookie() -> Cookie Instance (legacy alias shell).
pub fn mb_http_cookies_cookie_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("Cookie")
}

/// http.cookies.CookieError() -> CookieError Instance (exception shell).
pub fn mb_http_cookies_cookie_error_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("CookieError")
}

/// http.cookies.Morsel() -> Morsel Instance.
///
/// Pre-populated with the reserved keys (each defaulting to "") and
/// None key/value/coded_value, mirroring CPython's `Morsel.__init__`.
pub fn mb_http_cookies_morsel_new(_args: &[MbValue]) -> MbValue {
    make_morsel()
}

/// http.cookies.SimpleCookie() -> SimpleCookie Instance.
pub fn mb_http_cookies_simple_cookie_new(args: &[MbValue]) -> MbValue {
    let c = make_class_shell("SimpleCookie");
    cookie_ctor_load(c, args);
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cookies_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow()
                .get("http.cookies")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        for name in [
            "BaseCookie",
            "Cookie",
            "CookieError",
            "Morsel",
            "SimpleCookie",
        ] {
            assert!(
                cookies_attr(name).is_some(),
                "http.cookies module missing entry: {name}"
            );
        }
    }

    #[test]
    fn test_base_cookie_hot_path_returns_instance() {
        // Perf-gate path: must remain a single make_class_shell call.
        // Any indirection here regresses #1477 Gate 2.
        let r = mb_http_cookies_base_cookie_new(&[]);
        assert!(r.as_ptr().is_some());
    }
}
