use super::rc::{MbObject, ObjData};
/// String operations for the Mamba runtime (#284).
///
/// Implements Python-compatible string methods as extern-callable functions.
/// All functions operate on MbValue (NaN-boxed) and return MbValue.
use super::value::MbValue;
use std::cell::RefCell;
use std::collections::HashMap;

const SURROGATE_SENTINEL: &str = "\u{e000}";

thread_local! {
    static SURROGATE_STRINGS: RefCell<HashMap<usize, Vec<u32>>> = RefCell::new(HashMap::new());
}

/// Helper: extract string reference from a MbValue pointer.
unsafe fn as_str(val: MbValue) -> Option<&'static str> {
    val.as_ptr().and_then(|ptr| {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.as_str())
        } else {
            None
        }
    })
}

/// Helper: create a new string MbValue from a Rust String.
fn new_str(s: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s))
}

pub(crate) fn new_surrogate_codepoints_str(codepoints: Vec<u32>) -> MbValue {
    let val = new_str(SURROGATE_SENTINEL.to_string());
    if let Some(ptr) = val.as_ptr() {
        SURROGATE_STRINGS.with(|strings| {
            strings.borrow_mut().insert(ptr as usize, codepoints);
        });
    }
    val
}

pub(crate) fn new_lone_surrogate_str(codepoint: u32) -> MbValue {
    debug_assert!((0xD800..=0xDFFF).contains(&codepoint));
    new_surrogate_codepoints_str(vec![codepoint])
}

pub(crate) fn cleanup_all_surrogate_strings() {
    SURROGATE_STRINGS.with(|strings| strings.borrow_mut().clear());
}

pub(crate) fn surrogate_codepoints(val: MbValue) -> Option<Vec<u32>> {
    let ptr = val.as_ptr()?;
    unsafe {
        if !matches!(&(*ptr).data, ObjData::Str(s) if s == SURROGATE_SENTINEL) {
            return None;
        }
    }
    SURROGATE_STRINGS.with(|strings| strings.borrow().get(&(ptr as usize)).cloned())
}

pub(crate) fn surrogate_single_codepoint(val: MbValue) -> Option<u32> {
    let codepoints = surrogate_codepoints(val)?;
    if codepoints.len() == 1 {
        Some(codepoints[0])
    } else {
        None
    }
}

pub(crate) fn surrogate_len(val: MbValue) -> Option<usize> {
    surrogate_codepoints(val).map(|codepoints| codepoints.len())
}

pub(crate) fn string_values_equal_if_surrogate(a: MbValue, b: MbValue) -> Option<bool> {
    let a_surrogate = surrogate_codepoints(a);
    let b_surrogate = surrogate_codepoints(b);
    if a_surrogate.is_none() && b_surrogate.is_none() {
        return None;
    }
    let a_codepoints = a_surrogate.or_else(|| string_codepoints(a))?;
    let b_codepoints = b_surrogate.or_else(|| string_codepoints(b))?;
    Some(a_codepoints == b_codepoints)
}

fn string_codepoints(val: MbValue) -> Option<Vec<u32>> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.chars().map(|c| c as u32).collect()),
            _ => None,
        }
    }
}

pub(crate) fn repr_string_from_codepoints(codepoints: &[u32]) -> String {
    let has_single = codepoints.iter().any(|cp| *cp == '\'' as u32);
    let has_double = codepoints.iter().any(|cp| *cp == '"' as u32);
    let use_double = has_single && !has_double;
    let quote_char = if use_double { '"' } else { '\'' };
    let mut escaped = String::new();
    for &codepoint in codepoints {
        push_escaped_codepoint(&mut escaped, codepoint, quote_char, use_double, false);
    }
    format!("{quote_char}{escaped}{quote_char}")
}

pub(crate) fn ascii_string_from_codepoints(codepoints: &[u32]) -> String {
    format!("'{}'", escape_codepoints_non_ascii(codepoints))
}

pub(crate) fn escape_codepoints_non_ascii(codepoints: &[u32]) -> String {
    let mut escaped = String::new();
    for &codepoint in codepoints {
        push_escaped_codepoint(&mut escaped, codepoint, '\'', false, true);
    }
    escaped
}

fn push_escaped_codepoint(
    out: &mut String,
    codepoint: u32,
    quote_char: char,
    use_double: bool,
    ascii_only: bool,
) {
    match codepoint {
        cp if cp == '\\' as u32 => out.push_str("\\\\"),
        cp if cp == '\'' as u32 && !use_double => out.push_str("\\'"),
        cp if cp == '"' as u32 && use_double => out.push_str("\\\""),
        0x0A => out.push_str("\\n"),
        0x0D => out.push_str("\\r"),
        0x09 => out.push_str("\\t"),
        0x07 => out.push_str("\\a"),
        0x08 => out.push_str("\\b"),
        0x0C => out.push_str("\\f"),
        0x0B => out.push_str("\\v"),
        0xD800..=0xDFFF => out.push_str(&format!("\\u{codepoint:04x}")),
        cp if ascii_only && cp >= 0x80 => push_codepoint_escape(out, cp),
        cp if cp < 0x20 || (0x7F..=0x9F).contains(&cp) => push_codepoint_escape(out, cp),
        cp => {
            if let Some(c) = char::from_u32(cp) {
                if c == quote_char && !use_double {
                    out.push('\\');
                }
                out.push(c);
            } else {
                push_codepoint_escape(out, cp);
            }
        }
    }
}

fn push_codepoint_escape(out: &mut String, codepoint: u32) {
    if codepoint < 0x100 {
        out.push_str(&format!("\\x{codepoint:02x}"));
    } else if codepoint < 0x10000 {
        out.push_str(&format!("\\u{codepoint:04x}"));
    } else {
        out.push_str(&format!("\\U{codepoint:08x}"));
    }
}

fn raise_exception(kind: &str, msg: String) -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(kind.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    raise_exception("TypeError", msg.into())
}

fn raise_index_error(msg: impl Into<String>) -> MbValue {
    raise_exception("IndexError", msg.into())
}

fn raise_lookup_error(msg: impl Into<String>) -> MbValue {
    raise_exception("LookupError", msg.into())
}

fn raise_overflow_error(msg: impl Into<String>) -> MbValue {
    raise_exception("OverflowError", msg.into())
}

fn known_text_codec_fallback(enc: &str) -> bool {
    matches!(
        enc.replace('_', "-").as_str(),
            "idna"
            | "utf-7"
            | "utf7"
            | "euc-jp"
            | "eucjp"
            | "iso-2022-jp"
            | "shift-jis"
            | "sjis"
            | "cp932"
            | "cp1252"
            | "windows-1252"
            | "iso-8859-15"
            | "iso8859-15"
            | "latin-9"
            | "latin9"
            | "big5"
            | "gbk"
            | "gb2312"
            | "gb18030"
    )
}

fn int_digits_for_percent(v: MbValue, radix: u32) -> Option<(bool, String)> {
    if let Some(i) = v.as_int() {
        let abs = i.unsigned_abs();
        let digits = match radix {
            2 => format!("{:b}", abs),
            8 => format!("{:o}", abs),
            16 => format!("{:x}", abs),
            _ => abs.to_string(),
        };
        return Some((i < 0, digits));
    }
    if let Some(b) = v.as_bool() {
        return Some((false, if b { "1" } else { "0" }.to_string()));
    }
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::BigInt(ref big) = (*ptr).data {
            let s = big.to_str_radix(radix);
            if let Some(rest) = s.strip_prefix('-') {
                Some((true, rest.to_string()))
            } else {
                Some((false, s))
            }
        } else {
            None
        }
    }
}

// ── Concatenation and Repeat ──

/// str + str → new string
pub fn mb_str_concat(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_str(a), as_str(b)) {
            (Some(sa), Some(sb)) => new_str(format!("{sa}{sb}")),
            _ => MbValue::none(),
        }
    }
}

/// str * int → repeated string
pub fn mb_str_repeat(s: MbValue, n: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(count)) = (as_str(s), n.as_int()) {
            if count <= 0 {
                return new_str(String::new());
            }
            new_str(st.repeat(count as usize))
        } else {
            MbValue::none()
        }
    }
}

// ── Indexing and Slicing ──

/// str[index] → single character string
pub fn mb_str_getitem(s: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(idx)) = (as_str(s), index.as_int()) {
            let chars: Vec<char> = st.chars().collect();
            let len = chars.len() as i64;
            let actual = if idx < 0 { idx + len } else { idx };
            if actual >= 0 && actual < len {
                new_str(chars[actual as usize].to_string())
            } else {
                raise_index_error("string index out of range")
            }
        } else {
            MbValue::none()
        }
    }
}

/// str[start:stop] → substring
pub fn mb_str_slice(s: MbValue, start: MbValue, stop: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let chars: Vec<char> = st.chars().collect();
            let len = chars.len() as i64;
            let s_idx = normalize_index(start.as_int().unwrap_or(0), len);
            let e_idx = normalize_index(stop.as_int().unwrap_or(len), len);
            if s_idx <= e_idx {
                let result: String = chars[s_idx as usize..e_idx as usize].iter().collect();
                new_str(result)
            } else {
                new_str(String::new())
            }
        } else {
            MbValue::none()
        }
    }
}

fn normalize_index(idx: i64, len: i64) -> i64 {
    let i = if idx < 0 { idx + len } else { idx };
    i.max(0).min(len)
}

fn clamp_rev_str(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(-1).min(len - 1)
}

/// str[start:stop:step] → substring (step support, Unicode codepoints)
pub fn mb_str_slice_full(s: MbValue, start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let chars: Vec<char> = st.chars().collect();
            let len = chars.len() as i64;
            let step_val = step.as_int().unwrap_or(1);
            if step_val == 0 {
                return new_str(String::new());
            }
            let (s_idx, e_idx) = if step_val > 0 {
                let si = normalize_index(start.as_int().unwrap_or(0), len);
                let ei = normalize_index(stop.as_int().unwrap_or(len), len);
                (si, ei)
            } else {
                // For absent start/stop with negative step, use literal defaults
                // without clamping. clamp_rev_str normalizes -1 to len-1 which
                // breaks the loop condition (e.g. s[::-1] would produce empty).
                let si = match start.as_int() {
                    Some(v) => clamp_rev_str(v, len),
                    None => len - 1,
                };
                let ei = match stop.as_int() {
                    Some(v) => clamp_rev_str(v, len),
                    None => -1,
                };
                (si, ei)
            };
            let mut result = String::new();
            let mut i = s_idx;
            if step_val > 0 {
                while i < e_idx {
                    if i >= 0 && i < len {
                        result.push(chars[i as usize]);
                    }
                    i += step_val;
                }
            } else {
                while i > e_idx {
                    if i >= 0 && i < len {
                        result.push(chars[i as usize]);
                    }
                    i += step_val;
                }
            }
            new_str(result)
        } else {
            MbValue::none()
        }
    }
}

// ── Case Methods ──

pub fn mb_str_upper(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            new_str(st.to_uppercase())
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_lower(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            new_str(st.to_lowercase())
        } else {
            MbValue::none()
        }
    }
}

/// str.casefold() — aggressive lowercase for caseless comparison.
/// Python applies the Unicode casefold algorithm, which handles cases like
/// "ß" → "ss". Rust's `to_lowercase()` does not do this, so we apply common
/// German special-case mappings on top.
pub fn mb_str_casefold(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let lowered = st.to_lowercase();
            // Sharp s: both "ß" and "ẞ" (U+1E9E) fold to "ss"
            let folded = lowered.replace('ß', "ss");
            new_str(folded)
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_capitalize(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let mut chars = st.chars();
            let result = match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            };
            new_str(result)
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_title(s: MbValue) -> MbValue {
    // CPython's str.title() upper-cases each cased character that follows a
    // non-cased character. "Cased" = alphabetic (letters with upper/lower
    // forms); digits, punctuation, whitespace, and apostrophes are all
    // non-cased, so e.g. "123abc" -> "123Abc" and "don't" -> "Don'T".
    unsafe {
        if let Some(st) = as_str(s) {
            let mut result = String::with_capacity(st.len());
            let mut prev_cased = false;
            for c in st.chars() {
                let cased = c.is_alphabetic();
                if cased {
                    if prev_cased {
                        result.extend(c.to_lowercase());
                    } else {
                        result.extend(c.to_uppercase());
                    }
                } else {
                    result.push(c);
                }
                prev_cased = cased;
            }
            new_str(result)
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_swapcase(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let result: String = st
                .chars()
                .map(|c| {
                    if c.is_uppercase() {
                        c.to_lowercase().to_string()
                    } else {
                        c.to_uppercase().to_string()
                    }
                })
                .collect();
            new_str(result)
        } else {
            MbValue::none()
        }
    }
}

// ── Strip Methods ──

pub fn mb_str_strip(s: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            if chars.is_none() {
                new_str(st.trim().to_string())
            } else if let Some(ch) = as_str(chars) {
                let cs: Vec<char> = ch.chars().collect();
                new_str(st.trim_matches(|c: char| cs.contains(&c)).to_string())
            } else {
                MbValue::none()
            }
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_lstrip(s: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            if chars.is_none() {
                new_str(st.trim_start().to_string())
            } else if let Some(ch) = as_str(chars) {
                let cs: Vec<char> = ch.chars().collect();
                new_str(st.trim_start_matches(|c: char| cs.contains(&c)).to_string())
            } else {
                MbValue::none()
            }
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_rstrip(s: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            if chars.is_none() {
                new_str(st.trim_end().to_string())
            } else if let Some(ch) = as_str(chars) {
                let cs: Vec<char> = ch.chars().collect();
                new_str(st.trim_end_matches(|c: char| cs.contains(&c)).to_string())
            } else {
                MbValue::none()
            }
        } else {
            MbValue::none()
        }
    }
}

// ── Search Methods ──

pub fn mb_str_find(s: MbValue, sub: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sub)) {
            (Some(st), Some(pat)) => {
                let len = st.len() as i64;
                let s_idx = clamp_index(start, len, 0) as usize;
                let e_idx = clamp_index(end, len, len) as usize;
                if s_idx > e_idx || s_idx > st.len() {
                    return MbValue::from_int(-1);
                }
                let slice = &st[s_idx..e_idx.min(st.len())];
                MbValue::from_int(slice.find(pat).map(|i| (i + s_idx) as i64).unwrap_or(-1))
            }
            _ => MbValue::from_int(-1),
        }
    }
}

pub fn mb_str_index(s: MbValue, sub: MbValue, start: MbValue, end: MbValue) -> MbValue {
    let result = mb_str_find(s, sub, start, end);
    if let Some(idx) = result.as_int() {
        if idx < 0 {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("substring not found".to_string())),
            );
            return MbValue::none();
        }
    }
    result
}

/// str.rindex — like rfind but raises ValueError when the substring is absent.
pub fn mb_str_rindex(s: MbValue, sub: MbValue, start: MbValue, end: MbValue) -> MbValue {
    let result = mb_str_rfind(s, sub, start, end);
    if let Some(idx) = result.as_int() {
        if idx < 0 {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("substring not found".to_string())),
            );
            return MbValue::none();
        }
    }
    result
}

pub fn mb_str_rfind(s: MbValue, sub: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sub)) {
            (Some(st), Some(pat)) => {
                let len = st.len() as i64;
                let s_idx = clamp_index(start, len, 0) as usize;
                let e_idx = clamp_index(end, len, len) as usize;
                if s_idx > e_idx || s_idx > st.len() {
                    return MbValue::from_int(-1);
                }
                let slice = &st[s_idx..e_idx.min(st.len())];
                MbValue::from_int(slice.rfind(pat).map(|i| (i + s_idx) as i64).unwrap_or(-1))
            }
            _ => MbValue::from_int(-1),
        }
    }
}

pub fn mb_str_count(s: MbValue, sub: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sub)) {
            (Some(st), Some(pat)) => {
                let len = st.len() as i64;
                let s_idx = clamp_index(start, len, 0) as usize;
                let e_idx = clamp_index(end, len, len) as usize;
                if s_idx > e_idx || s_idx > st.len() {
                    return MbValue::from_int(0);
                }
                let slice = &st[s_idx..e_idx.min(st.len())];
                MbValue::from_int(slice.matches(pat).count() as i64)
            }
            _ => MbValue::from_int(0),
        }
    }
}

/// Helper: clamp a start/end index (Python-style, allowing negatives).
/// Returns default_val if val is None.
fn clamp_index(val: MbValue, len: i64, default_val: i64) -> i64 {
    if val.is_none() {
        return default_val;
    }
    if let Some(i) = val.as_int() {
        let adj = if i < 0 { (len + i).max(0) } else { i.min(len) };
        adj
    } else {
        default_val
    }
}

pub fn mb_str_startswith(s: MbValue, prefix: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let len = st.len() as i64;
            let s_idx = clamp_index(start, len, 0) as usize;
            let e_idx = clamp_index(end, len, len) as usize;
            if s_idx > e_idx || s_idx > st.len() {
                return MbValue::from_bool(false);
            }
            let slice = &st[s_idx..e_idx.min(st.len())];
            // Check for tuple/list argument (Python accepts tuple of prefixes)
            if let Some(ptr) = prefix.as_ptr() {
                let prefixes: Option<Vec<MbValue>> = match &(*ptr).data {
                    ObjData::Tuple(items) => Some(items.clone()),
                    ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
                    _ => None,
                };
                if let Some(items) = prefixes {
                    for item in &items {
                        if let Some(p) = as_str(*item) {
                            if slice.starts_with(p) {
                                return MbValue::from_bool(true);
                            }
                        }
                    }
                    return MbValue::from_bool(false);
                }
            }
            if let Some(p) = as_str(prefix) {
                MbValue::from_bool(slice.starts_with(p))
            } else {
                raise_type_error(format!(
                    "startswith first arg must be str or a tuple of str, not {}",
                    super::builtins::value_type_name(prefix)
                ))
            }
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_endswith(s: MbValue, suffix: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let len = st.len() as i64;
            let s_idx = clamp_index(start, len, 0) as usize;
            let e_idx = clamp_index(end, len, len) as usize;
            if s_idx > e_idx || s_idx > st.len() {
                return MbValue::from_bool(false);
            }
            let slice = &st[s_idx..e_idx.min(st.len())];
            // Check for tuple/list argument (Python accepts tuple of suffixes)
            if let Some(ptr) = suffix.as_ptr() {
                let suffixes: Option<Vec<MbValue>> = match &(*ptr).data {
                    ObjData::Tuple(items) => Some(items.clone()),
                    ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
                    _ => None,
                };
                if let Some(items) = suffixes {
                    for item in &items {
                        if let Some(p) = as_str(*item) {
                            if slice.ends_with(p) {
                                return MbValue::from_bool(true);
                            }
                        }
                    }
                    return MbValue::from_bool(false);
                }
            }
            if let Some(p) = as_str(suffix) {
                MbValue::from_bool(slice.ends_with(p))
            } else {
                MbValue::from_bool(false)
            }
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_contains(s: MbValue, sub: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sub)) {
            (Some(st), Some(pat)) => MbValue::from_bool(st.contains(pat)),
            _ => MbValue::from_bool(false),
        }
    }
}

// ── Modification Methods ──

pub fn mb_str_replace(s: MbValue, old: MbValue, new_val: MbValue, count: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(old), as_str(new_val)) {
            (Some(st), Some(o), Some(n)) => {
                if let Some(max) = count.as_int() {
                    if max < 0 {
                        new_str(st.replace(o, n))
                    } else {
                        new_str(st.replacen(o, n, max as usize))
                    }
                } else {
                    new_str(st.replace(o, n))
                }
            }
            _ => MbValue::none(),
        }
    }
}

/// split(sep, maxsplit) → list of strings
pub fn mb_str_split(s: MbValue, sep: MbValue, maxsplit: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let max = maxsplit.as_int().unwrap_or(-1);
            let parts: Vec<MbValue> = if sep.is_none() {
                if max < 0 {
                    st.split_whitespace()
                        .map(|p| new_str(p.to_string()))
                        .collect()
                } else {
                    st.split_whitespace()
                        .take(max as usize + 1)
                        .collect::<Vec<_>>()
                        .into_iter()
                        .enumerate()
                        .map(|(i, p)| {
                            if i == max as usize {
                                // Last element gets the remainder
                                let rest_start = st.find(p).unwrap_or(0);
                                new_str(st[rest_start..].to_string())
                            } else {
                                new_str(p.to_string())
                            }
                        })
                        .collect()
                }
            } else if let Some(sep_str) = as_str(sep) {
                if sep_str.is_empty() {
                    // Python raises ValueError for empty separator
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("empty separator".to_string())),
                    );
                    return MbValue::none();
                }
                if max < 0 {
                    st.split(sep_str).map(|p| new_str(p.to_string())).collect()
                } else {
                    st.splitn(max as usize + 1, sep_str)
                        .map(|p| new_str(p.to_string()))
                        .collect()
                }
            } else {
                return MbValue::none();
            };
            MbValue::from_ptr(MbObject::new_list(parts))
        } else {
            MbValue::none()
        }
    }
}

/// rsplit(sep=None, maxsplit=-1) → list[str]
/// Splits from the right. With maxsplit=-1 the result matches split(sep).
pub fn mb_str_rsplit(s: MbValue, sep: MbValue, maxsplit: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let max = maxsplit.as_int().unwrap_or(-1);
            let parts: Vec<MbValue> = if sep.is_none() {
                // CPython: no sep + no limit → rsplit == split on whitespace.
                if max < 0 {
                    st.split_whitespace()
                        .map(|p| new_str(p.to_string()))
                        .collect()
                } else {
                    // Walk from the right, taking up to `max` words and preserving
                    // the head substring exactly (internal whitespace kept, only
                    // trailing whitespace next to the split is dropped).
                    let bytes = st.as_bytes();
                    let mut end = bytes.len();
                    let mut tail: Vec<String> = Vec::new();
                    let mut i = bytes.len();
                    let mut taken: i64 = 0;
                    while taken < max && i > 0 {
                        // Skip trailing whitespace.
                        while i > 0 && (bytes[i - 1] as char).is_whitespace() {
                            i -= 1;
                        }
                        if i == 0 {
                            break;
                        }
                        let word_end = i;
                        while i > 0 && !(bytes[i - 1] as char).is_whitespace() {
                            i -= 1;
                        }
                        tail.push(st[i..word_end].to_string());
                        end = i;
                        taken += 1;
                    }
                    // Trim trailing whitespace from the head.
                    let head = &st[..end];
                    let head = head.trim_end();
                    let mut out: Vec<MbValue> = Vec::new();
                    if !head.is_empty() || taken == 0 {
                        out.push(new_str(head.to_string()));
                    }
                    tail.reverse();
                    out.extend(tail.into_iter().map(new_str));
                    out
                }
            } else if let Some(sep_str) = as_str(sep) {
                if sep_str.is_empty() {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("empty separator".to_string())),
                    );
                    return MbValue::none();
                }
                if max < 0 {
                    st.split(sep_str).map(|p| new_str(p.to_string())).collect()
                } else {
                    // Rust's rsplitn yields from the right; reverse to match Python's order.
                    let mut pieces: Vec<&str> = st.rsplitn(max as usize + 1, sep_str).collect();
                    pieces.reverse();
                    pieces.into_iter().map(|p| new_str(p.to_string())).collect()
                }
            } else {
                return MbValue::none();
            };
            MbValue::from_ptr(MbObject::new_list(parts))
        } else {
            MbValue::none()
        }
    }
}

/// join(iterable) → string
pub fn mb_str_join(sep: MbValue, items: MbValue) -> MbValue {
    unsafe {
        let Some(sep_str) = as_str(sep) else {
            return MbValue::none();
        };
        // Fast-path: direct container access without consuming an iterator.
        // For List/Tuple/FrozenSet we walk the storage in-place. The classic
        // implementation collected a Vec<&str> then called Vec::join, which
        // does a Vec allocation + a String allocation. Build the result
        // String directly with a single pre-sized allocation — saves one
        // malloc per call (the bench's string_concat hot path).
        #[inline(always)]
        fn join_slice(values: &[MbValue], sep_str: &str) -> MbValue {
            unsafe {
                let mut total: usize = 0;
                let mut count: usize = 0;
                for (idx, v) in values.iter().enumerate() {
                    if let Some(s) = as_str(*v) {
                        total += s.len();
                        count += 1;
                    } else {
                        return raise_type_error(format!(
                            "sequence item {idx}: expected str instance, {} found",
                            super::builtins::value_type_name(*v)
                        ));
                    }
                }
                if count == 0 {
                    return new_str(String::new());
                }
                total += sep_str.len() * (count - 1);
                let mut out = String::with_capacity(total);
                let mut first = true;
                for v in values.iter() {
                    if let Some(s) = as_str(*v) {
                        if !first {
                            out.push_str(sep_str);
                        }
                        out.push_str(s);
                        first = false;
                    }
                }
                new_str(out)
            }
        }
        if let Some(ptr) = items.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let guard = lock.read().unwrap();
                    return join_slice(&guard, sep_str);
                }
                ObjData::Tuple(ref t) => {
                    return join_slice(t, sep_str);
                }
                ObjData::FrozenSet(ref items) => {
                    return join_slice(items, sep_str);
                }
                ObjData::Set(ref lock) => {
                    let guard = lock.read().unwrap();
                    return join_slice(&guard, sep_str);
                }
                ObjData::Str(ref s) => {
                    // Joining over a string yields per-char joining.
                    if sep_str.is_empty() {
                        // sep="" over a string is just the string itself.
                        return new_str(s.clone());
                    }
                    let mut out = String::with_capacity(
                        s.len() + sep_str.len() * s.chars().count().saturating_sub(1),
                    );
                    let mut first = true;
                    for c in s.chars() {
                        if !first {
                            out.push_str(sep_str);
                        }
                        out.push(c);
                        first = false;
                    }
                    return new_str(out);
                }
                _ => {}
            }
        }
        // Iterator protocol fallback (reversed, map, filter, zip, generators, ...)
        let iter_handle = super::iter::mb_iter(items);
        if iter_handle.is_none() {
            return MbValue::none();
        }
        let mut parts: Vec<String> = Vec::new();
        let mut idx = 0usize;
        loop {
            let v = super::iter::mb_next_raise(iter_handle);
            if super::exception::mb_has_exception().as_bool() == Some(true) {
                super::exception::mb_clear_exception();
                break;
            }
            if let Some(st) = as_str(v) {
                parts.push(st.to_string());
            } else {
                return raise_type_error(format!(
                    "sequence item {idx}: expected str instance, {} found",
                    super::builtins::value_type_name(v)
                ));
            }
            idx += 1;
        }
        new_str(parts.join(sep_str))
    }
}

// ── Predicate Methods ──

/// Returns true for code points with Unicode `Numeric_Type=Digit` that are
/// NOT already in `General_Category=Nd`. These are the "decoration" digits
/// (superscripts, subscripts, single-glyph circled/parenthesised digits)
/// where CPython's `str.isdigit()` is True but `isdecimal()` is False.
/// The list mirrors the UCD's Nt=Di set excluding Nd; ranges large enough
/// to be ergonomic, items beyond 9 (e.g. ⑩) carry Nt=Nu and are skipped.
fn is_unicode_digit_no(c: char) -> bool {
    matches!(c as u32,
        0x00B2..=0x00B3 |   // SUPERSCRIPT TWO, THREE
        0x00B9 |             // SUPERSCRIPT ONE
        0x2070 |             // SUPERSCRIPT ZERO
        0x2074..=0x2079 |   // SUPERSCRIPT FOUR..NINE
        0x2080..=0x2089 |   // SUBSCRIPT ZERO..NINE
        0x2460..=0x2468 |   // CIRCLED DIGIT ONE..NINE
        0x2474..=0x247C |   // PARENTHESIZED DIGIT ONE..NINE
        0x2488..=0x2490 |   // DIGIT ONE FULL STOP..NINE FULL STOP
        0x24EA |             // CIRCLED DIGIT ZERO
        0x24F5..=0x24FD |   // DOUBLE CIRCLED DIGIT ONE..NINE
        0x24FF |             // NEGATIVE CIRCLED DIGIT ZERO
        0x2776..=0x277E |   // DINGBAT NEGATIVE CIRCLED DIGIT ONE..NINE
        0x2780..=0x2788 |   // DINGBAT CIRCLED SANS-SERIF ONE..NINE
        0x278A..=0x2792) // DINGBAT NEGATIVE CIRCLED SANS-SERIF ONE..NINE
}

pub fn mb_str_isdigit(s: MbValue) -> MbValue {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                !st.is_empty()
                    && st.chars().all(|c| {
                        c.general_category() == GeneralCategory::DecimalNumber
                            || is_unicode_digit_no(c)
                    }),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_isalpha(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(!st.is_empty() && st.chars().all(|c| c.is_alphabetic()))
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_isalnum(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(!st.is_empty() && st.chars().all(|c| c.is_alphanumeric()))
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_isspace(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(!st.is_empty() && st.chars().all(|c| c.is_whitespace()))
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_isupper(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                st.chars().any(|c| c.is_uppercase()) && st.chars().all(|c| !c.is_lowercase()),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_islower(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                st.chars().any(|c| c.is_lowercase()) && st.chars().all(|c| !c.is_uppercase()),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

pub fn mb_str_istitle(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            if st.is_empty() {
                return MbValue::from_bool(false);
            }
            let mut cased = false;
            let mut prev_cased = false;
            for ch in st.chars() {
                if ch.is_uppercase() {
                    if prev_cased {
                        return MbValue::from_bool(false);
                    }
                    cased = true;
                    prev_cased = true;
                } else if ch.is_lowercase() {
                    if !prev_cased {
                        return MbValue::from_bool(false);
                    }
                    cased = true;
                    prev_cased = true;
                } else {
                    prev_cased = false;
                }
            }
            MbValue::from_bool(cased)
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// str.isascii() — True if string is empty or all chars are ASCII (CPython 3.7+).
pub fn mb_str_isascii(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(st.is_ascii())
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// str.isidentifier() — True if string is a valid Python identifier.
/// Per PEP 3131: first char `XID_Start` (a letter or underscore), rest `XID_Continue`.
/// We approximate with ASCII rules + unicode_xid logic: first must be alphabetic or '_',
/// rest must be alphanumeric or '_'. Empty string returns False.
pub fn mb_str_isidentifier(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let mut chars = st.chars();
            let first = match chars.next() {
                Some(c) => c,
                None => return MbValue::from_bool(false),
            };
            if !(first == '_' || first.is_alphabetic()) {
                return MbValue::from_bool(false);
            }
            for c in chars {
                if !(c == '_' || c.is_alphanumeric()) {
                    return MbValue::from_bool(false);
                }
            }
            MbValue::from_bool(true)
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// str.isnumeric() — True iff every char has `General_Category` in
/// {Nd, Nl, No}: decimal digits, letter-number (Roman numerals), or
/// other-number (fractions, superscripts, circled digits, ...).
pub fn mb_str_isnumeric(s: MbValue) -> MbValue {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                !st.is_empty()
                    && st.chars().all(|c| {
                        matches!(
                            c.general_category(),
                            GeneralCategory::DecimalNumber
                                | GeneralCategory::LetterNumber
                                | GeneralCategory::OtherNumber
                        )
                    }),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// str.isdecimal() — True iff every char has `General_Category=Nd`
/// (Decimal_Number). This matches CPython exactly: ASCII `0-9`, Arabic-Indic
/// digits `٠-٩`, Devanagari `०-९`, fullwidth `０-９`, etc., but excludes
/// superscripts, fractions, Roman numerals.
pub fn mb_str_isdecimal(s: MbValue) -> MbValue {
    use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                !st.is_empty()
                    && st
                        .chars()
                        .all(|c| c.general_category() == GeneralCategory::DecimalNumber),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// str.isprintable() — True if all chars are printable. Empty string is True.
/// CPython treats space (U+0020) as printable but other whitespace as not.
pub fn mb_str_isprintable(s: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_bool(
                st.chars()
                    .all(|c| c == ' ' || (!c.is_whitespace() && !c.is_control())),
            )
        } else {
            MbValue::from_bool(false)
        }
    }
}

// ── Padding Methods ──

pub fn mb_str_center(s: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(w)) = (as_str(s), width.as_int()) {
            let fillchar = match fill_char(fill) {
                Some(c) => c,
                None => return MbValue::none(),
            };
            let w = w as usize;
            let char_len = st.chars().count();
            if char_len >= w {
                return new_str(st.to_string());
            }
            let pad = w - char_len;
            let left = pad / 2 + (pad & w & 1);
            let right = pad - left;
            let result = format!(
                "{}{}{}",
                fillchar.to_string().repeat(left),
                st,
                fillchar.to_string().repeat(right)
            );
            new_str(result)
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_ljust(s: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(w)) = (as_str(s), width.as_int()) {
            let fillchar = match fill_char(fill) {
                Some(c) => c,
                None => return MbValue::none(),
            };
            let w = w as usize;
            let char_len = st.chars().count();
            if char_len >= w {
                return new_str(st.to_string());
            }
            new_str(format!(
                "{}{}",
                st,
                fillchar.to_string().repeat(w - char_len)
            ))
        } else {
            MbValue::none()
        }
    }
}

pub fn mb_str_rjust(s: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(w)) = (as_str(s), width.as_int()) {
            let fillchar = match fill_char(fill) {
                Some(c) => c,
                None => return MbValue::none(),
            };
            let w = w as usize;
            let char_len = st.chars().count();
            if char_len >= w {
                return new_str(st.to_string());
            }
            new_str(format!(
                "{}{}",
                fillchar.to_string().repeat(w - char_len),
                st
            ))
        } else {
            MbValue::none()
        }
    }
}

fn fill_char(fill: MbValue) -> Option<char> {
    if fill.is_none() {
        return Some(' ');
    }
    unsafe {
        if let Some(f) = as_str(fill) {
            let mut chars = f.chars();
            let Some(ch) = chars.next() else {
                raise_type_error("The fill character must be exactly one character long");
                return None;
            };
            if chars.next().is_some() {
                raise_type_error("The fill character must be exactly one character long");
                return None;
            }
            Some(ch)
        } else {
            Some(' ')
        }
    }
}

pub fn mb_str_zfill(s: MbValue, width: MbValue) -> MbValue {
    unsafe {
        if let (Some(st), Some(w)) = (as_str(s), width.as_int()) {
            let w = w as usize;
            let char_len = st.chars().count();
            if char_len >= w {
                return new_str(st.to_string());
            }
            let (sign, num) = if st.starts_with('-') || st.starts_with('+') {
                (&st[..1], &st[1..])
            } else {
                ("", st)
            };
            new_str(format!("{}{}{}", sign, "0".repeat(w - char_len), num))
        } else {
            MbValue::none()
        }
    }
}

// ── Encoding / Hashing ──

pub fn mb_str_encode(s: MbValue) -> MbValue {
    // Returns the string as UTF-8 bytes (a bytes object, not a list).
    // Default encoding/errors path — `mb_str_encode_with` covers the
    // explicit-args form so the dispatcher can route both calls.
    unsafe {
        if let Some(st) = as_str(s) {
            MbValue::from_ptr(MbObject::new_bytes(st.as_bytes().to_vec()))
        } else {
            MbValue::none()
        }
    }
}

/// `str.encode(encoding="utf-8", errors="strict")` — full form.
///
/// Supported encodings: `utf-8` / `utf8` (no errors needed) and `ascii`
/// (errors may be `strict` / `ignore` / `replace`). Other encodings fall
/// back to UTF-8 to keep behaviour permissive rather than panicking.
/// Canonical name of a known CPython *non-text* codec (bytes-transform codecs
/// like base64/rot_13/zlib), normalised across `-`/`_` and the `_codec`
/// suffix. `str.encode` / `bytes.decode` reject these with LookupError ("not a
/// text encoding"); they are only usable via `codecs.encode()`/`decode()`.
pub(crate) fn nontext_codec_name(enc: &str) -> Option<&'static str> {
    let norm = enc.replace('-', "_");
    let norm = norm.strip_suffix("_codec").unwrap_or(&norm);
    Some(match norm {
        "base64" | "base_64" => "base64",
        "bz2" => "bz2",
        "hex" => "hex",
        "quopri" => "quopri",
        "rot13" | "rot_13" => "rot_13",
        "uu" => "uu",
        "zlib" => "zlib",
        _ => return None,
    })
}

/// Full set of CPython codec names + aliases (the names `codecs.lookup`
/// resolves on this platform), normalised by lowercasing and stripping
/// `-`/`_`/space (dots preserved, matching CPython's `normalize_encoding`
/// punctuation handling for membership). Used by `str.encode` / `bytes.decode`
/// to tell a recognised-but-not-enumerated codec (which keeps the lenient
/// fallback) from a genuinely unknown name (which must raise `LookupError:
/// unknown encoding: <name>`, matching CPython).
const KNOWN_CODECS: &[&str] = &[
    "037", "1026", "1125", "1140", "1250", "1251", "1252", "1253", "1254",
    "1255", "1256", "1257", "1258", "273", "424", "437", "500", "646", "775",
    "850", "852", "855", "857", "858", "860", "861", "862", "863", "864",
    "865", "866", "869", "8859", "932", "936", "949", "950", "ansix3.41968",
    "ansix3.41986", "ansix341968", "arabic", "ascii", "asmo708", "base64",
    "base64codec", "big5", "big5hkscs", "big5tw", "bz2", "bz2codec", "charmap",
    "chinese", "cp037", "cp1006", "cp1026", "cp1051", "cp1125", "cp1140",
    "cp1250", "cp1251", "cp1252", "cp1253", "cp1254", "cp1255", "cp1256",
    "cp1257", "cp1258", "cp1361", "cp154", "cp273", "cp367", "cp424", "cp437",
    "cp500", "cp65001", "cp720", "cp737", "cp775", "cp819", "cp850", "cp852",
    "cp855", "cp856", "cp857", "cp858", "cp860", "cp861", "cp862", "cp863",
    "cp864", "cp865", "cp866", "cp866u", "cp869", "cp874", "cp875", "cp932",
    "cp936", "cp949", "cp950", "cpgr", "cpis", "csascii", "csbig5", "csibm037",
    "csibm1026", "csibm273", "csibm424", "csibm500", "csibm855", "csibm857",
    "csibm858", "csibm860", "csibm861", "csibm863", "csibm864", "csibm865",
    "csibm866", "csibm869", "csiso2022jp", "csiso2022kr", "csiso58gb231280",
    "csisolatin1", "csisolatin2", "csisolatin3", "csisolatin4", "csisolatin5",
    "csisolatin6", "csisolatinarabic", "csisolatincyrillic", "csisolatingreek",
    "csisolatinhebrew", "cskoi8r", "cspc775baltic", "cspc850multilingual",
    "cspc862latinhebrew", "cspc8codepage437", "cspcp852", "csptcp154",
    "csshiftjis", "cyrillic", "cyrillicasian", "ebcdiccpbe", "ebcdiccpca",
    "ebcdiccpch", "ebcdiccphe", "ebcdiccpnl", "ebcdiccpus", "ebcdiccpwt",
    "ecma114", "ecma118", "elot928", "euccn", "eucgb2312cn", "eucjis2004",
    "eucjisx0213", "eucjp", "euckr", "gb18030", "gb180302000", "gb2312",
    "gb23121980", "gb231280", "gbk", "greek", "greek8", "hebrew", "hex",
    "hexcodec", "hkscs", "hproman8", "hz", "hzgb", "hzgb2312", "ibm037",
    "ibm039", "ibm1026", "ibm1051", "ibm1125", "ibm1140", "ibm273", "ibm367",
    "ibm424", "ibm437", "ibm500", "ibm775", "ibm819", "ibm850", "ibm852",
    "ibm855", "ibm857", "ibm858", "ibm860", "ibm861", "ibm862", "ibm863",
    "ibm864", "ibm865", "ibm866", "ibm869", "idna", "iso2022jp", "iso2022jp1",
    "iso2022jp2", "iso2022jp2004", "iso2022jp3", "iso2022jpext", "iso2022kr",
    "iso646.irv1991", "iso646us", "iso8859", "iso88591", "iso885910",
    "iso8859101992", "iso885911", "iso8859112001", "iso885911987", "iso885913",
    "iso885914", "iso8859141998", "iso885915", "iso885916", "iso8859162001",
    "iso88592", "iso885921987", "iso88593", "iso885931988", "iso88594",
    "iso885941988", "iso88595", "iso885951988", "iso88596", "iso885961987",
    "iso88597", "iso885971987", "iso88598", "iso885981988", "iso88599",
    "iso885991989", "isoceltic", "isoir100", "isoir101", "isoir109",
    "isoir110", "isoir126", "isoir127", "isoir138", "isoir144", "isoir148",
    "isoir157", "isoir166", "isoir199", "isoir226", "isoir58", "isoir6",
    "jisx0213", "johab", "koi8r", "koi8t", "koi8u", "korean", "ksc5601",
    "ksc56011987", "ksx1001", "kz1048", "l1", "l10", "l2", "l3", "l4", "l5",
    "l6", "l7", "l8", "l9", "latin", "latin1", "latin10", "latin2", "latin3",
    "latin4", "latin5", "latin6", "latin7", "latin8", "latin9", "macarabic",
    "maccenteuro", "maccentraleurope", "maccroatian", "maccyrillic", "macfarsi",
    "macgreek", "maciceland", "macintosh", "maclatin2", "macroman",
    "macromanian", "macturkish", "ms1361", "ms932", "ms936", "ms949", "ms950",
    "mskanji", "palmos", "pt154", "ptcp154", "punycode", "quopri",
    "quopricodec", "quotedprintable", "r8", "rawunicodeescape", "rk1048",
    "roman8", "rot13", "ruscii", "shiftjis", "shiftjis2004", "shiftjisx0213",
    "sjis", "sjis2004", "sjisx0213", "strk10482002", "thai", "tis620",
    "tis6200", "tis62025290", "tis62025291", "u16", "u32", "u7", "u8", "uhc",
    "ujis", "undefined", "unicode11utf7", "unicodebigunmarked", "unicodeescape",
    "unicodelittleunmarked", "us", "usascii", "utf", "utf16", "utf16be",
    "utf16le", "utf32", "utf32be", "utf32le", "utf7", "utf8", "utf8sig",
    "utf8ucs2", "utf8ucs4", "uu", "uucodec", "windows1250", "windows1251",
    "windows1252", "windows1253", "windows1254", "windows1255", "windows1256",
    "windows1257", "windows1258", "xmacjapanese", "xmackorean",
    "xmacsimpchinese", "xmactradchinese", "zip", "zlib", "zlibcodec",
];

/// True when `name` is a codec CPython's `codecs.lookup` resolves. Mirrors the
/// `KNOWN_CODECS` membership after the same normalisation (lowercase + strip
/// `-`/`_`/space, dots kept). A name that is *not* known is rejected with
/// `LookupError: unknown encoding: <name>` by `str.encode`/`bytes.decode`.
pub(crate) fn is_known_codec(name: &str) -> bool {
    let norm: String = name
        .chars()
        .filter(|c| !matches!(c, '-' | '_' | ' '))
        .flat_map(|c| c.to_lowercase())
        .collect();
    KNOWN_CODECS.binary_search(&norm.as_str()).is_ok()
}

pub fn mb_str_encode_with(s: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    unsafe {
        let st = match as_str(s) {
            Some(t) => t,
            None => return MbValue::none(),
        };
        let enc_orig = as_str(encoding).unwrap_or("utf-8").to_string();
        let enc = enc_orig.to_ascii_lowercase();
        let err = as_str(errors).unwrap_or("strict").to_string();
        let raise_uee = |enc_name: &str, ch: char, pos: usize| {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("UnicodeEncodeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'{}' codec can't encode character '\\u{:04x}' in position {}: ordinal not in range",
                    enc_name, ch as u32, pos
                ))),
            );
        };
        let bytes = match enc.as_str() {
            "utf-8-sig" | "utf_8_sig" => {
                let mut out = vec![0xEF, 0xBB, 0xBF];
                out.extend_from_slice(st.as_bytes());
                out
            }
            "utf-8" | "utf8" | "u8" => st.as_bytes().to_vec(),
            "ascii" | "us-ascii" => {
                let mut out: Vec<u8> = Vec::with_capacity(st.len());
                for (pos, ch) in st.chars().enumerate() {
                    if (ch as u32) < 0x80 {
                        out.push(ch as u8);
                    } else {
                        match err.as_str() {
                            "ignore" => continue,
                            "replace" => out.push(b'?'),
                            _ => {
                                raise_uee("ascii", ch, pos);
                                return MbValue::none();
                            }
                        }
                    }
                }
                out
            }
            "latin-1" | "latin_1" | "iso-8859-1" | "8859" => {
                let mut out: Vec<u8> = Vec::with_capacity(st.len());
                for (pos, ch) in st.chars().enumerate() {
                    if (ch as u32) < 0x100 {
                        out.push(ch as u8);
                    } else {
                        match err.as_str() {
                            "ignore" => continue,
                            "replace" => out.push(b'?'),
                            _ => {
                                raise_uee("latin-1", ch, pos);
                                return MbValue::none();
                            }
                        }
                    }
                }
                out
            }
            // UTF-16 / UTF-32 families. The "-be"/"-le" suffixed forms write
            // no BOM; the bare "utf-16"/"utf-32" forms prepend a little-endian
            // BOM (matching CPython's native default). Needed by plistlib's
            // binary writer (unicode strings → utf-16be) and test_xml_encodings.
            "utf-16be" | "utf-16-be" | "utf_16_be" => {
                let mut out = Vec::with_capacity(st.len() * 2);
                for u in st.encode_utf16() {
                    out.extend_from_slice(&u.to_be_bytes());
                }
                out
            }
            "utf-16le" | "utf-16-le" | "utf_16_le" => {
                let mut out = Vec::with_capacity(st.len() * 2);
                for u in st.encode_utf16() {
                    out.extend_from_slice(&u.to_le_bytes());
                }
                out
            }
            "utf-16" | "utf16" => {
                let mut out = vec![0xFF, 0xFE];
                for u in st.encode_utf16() {
                    out.extend_from_slice(&u.to_le_bytes());
                }
                out
            }
            "utf-32be" | "utf-32-be" | "utf_32_be" => {
                let mut out = Vec::with_capacity(st.len() * 4);
                for ch in st.chars() {
                    out.extend_from_slice(&(ch as u32).to_be_bytes());
                }
                out
            }
            "utf-32le" | "utf-32-le" | "utf_32_le" => {
                let mut out = Vec::with_capacity(st.len() * 4);
                for ch in st.chars() {
                    out.extend_from_slice(&(ch as u32).to_le_bytes());
                }
                out
            }
            "utf-32" | "utf32" => {
                let mut out = vec![0xFF, 0xFE, 0x00, 0x00];
                for ch in st.chars() {
                    out.extend_from_slice(&(ch as u32).to_le_bytes());
                }
                out
            }
            _ => {
                // A known non-text codec (rot_13, base64, ...) is a LookupError
                // via str.encode ("not a text encoding").
                if let Some(canon) = nontext_codec_name(&enc) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("LookupError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'{canon}' is not a text encoding; use codecs.encode() to handle arbitrary codecs"
                        ))),
                    );
                    return MbValue::none();
                }
                // A recognised-but-not-enumerated text codec (utf-7, idna,
                // punycode, cp125x, ...) keeps the lenient utf-8 fallback. A
                // name CPython's codecs.lookup does not resolve is rejected.
                if !is_known_codec(&enc) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("LookupError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "unknown encoding: {enc_orig}"
                        ))),
                    );
                    return MbValue::none();
                }
                st.as_bytes().to_vec()
            }
        };
        MbValue::from_ptr(MbObject::new_bytes(bytes))
    }
}

pub fn mb_str_hash(s: MbValue) -> MbValue {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    unsafe {
        if let Some(codepoints) = surrogate_codepoints(s) {
            let mut hasher = DefaultHasher::new();
            codepoints.hash(&mut hasher);
            let h = (hasher.finish() >> 17) as i64;
            return MbValue::from_int(h);
        }
        if let Some(st) = as_str(s) {
            // CPython: hash of the empty string is 0.
            if st.is_empty() {
                return MbValue::from_int(0);
            }
            let mut hasher = DefaultHasher::new();
            st.hash(&mut hasher);
            let h = (hasher.finish() >> 17) as i64;
            MbValue::from_int(h)
        } else {
            MbValue::from_int(0)
        }
    }
}

// ── Comparison ──

pub fn mb_str_eq(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_str(a), as_str(b)) {
            (Some(sa), Some(sb)) => MbValue::from_bool(sa == sb),
            _ => MbValue::from_bool(false),
        }
    }
}

pub fn mb_str_lt(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        match (as_str(a), as_str(b)) {
            (Some(sa), Some(sb)) => MbValue::from_bool(sa < sb),
            _ => MbValue::from_bool(false),
        }
    }
}

// ── Additional String Methods ──

/// splitlines(keepends=False) → list of lines.
/// Splits on \n, \r, and \r\n (Python semantics).
pub fn mb_str_splitlines(s: MbValue, keepends: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let keep = if let Some(b) = keepends.as_bool() {
                b
            } else if let Some(i) = keepends.as_int() {
                i != 0
            } else {
                false
            };
            let mut out: Vec<MbValue> = Vec::new();
            let bytes = st.as_bytes();
            let mut start = 0usize;
            let mut i = 0usize;
            while i < bytes.len() {
                let b = bytes[i];
                if b == b'\n' || b == b'\r' {
                    // Line terminator length: \r\n = 2, else 1.
                    let term_len = if b == b'\r' && i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                        2
                    } else {
                        1
                    };
                    let end = if keep { i + term_len } else { i };
                    out.push(new_str(
                        String::from_utf8_lossy(&bytes[start..end]).into_owned(),
                    ));
                    i += term_len;
                    start = i;
                } else {
                    i += 1;
                }
            }
            if start < bytes.len() {
                out.push(new_str(
                    String::from_utf8_lossy(&bytes[start..]).into_owned(),
                ));
            }
            MbValue::from_ptr(MbObject::new_list(out))
        } else {
            MbValue::none()
        }
    }
}

/// expandtabs(tabsize=8) → str with tab characters expanded.
pub fn mb_str_expandtabs(s: MbValue, tabsize: MbValue) -> MbValue {
    unsafe {
        if let Some(st) = as_str(s) {
            let bigint_tabsize = tabsize.as_ptr().map_or(false, |p| {
                matches!(&(*p).data, ObjData::BigInt(_))
            });
            if bigint_tabsize
                || tabsize
                    .as_int()
                    .map(|i| i > 1_000_000_000)
                    .unwrap_or(false)
            {
                return raise_overflow_error("Python int too large to convert to C int");
            }
            let size: usize = tabsize
                .as_int()
                .map(|i| if i < 0 { 0 } else { i as usize })
                .unwrap_or(8);
            let mut out = String::with_capacity(st.len());
            let mut col: usize = 0;
            for c in st.chars() {
                match c {
                    '\t' => {
                        if size == 0 { /* drop tab */
                        } else {
                            let pad = size - (col % size);
                            for _ in 0..pad {
                                out.push(' ');
                            }
                            col += pad;
                        }
                    }
                    '\n' | '\r' => {
                        out.push(c);
                        col = 0;
                    }
                    _ => {
                        out.push(c);
                        col += 1;
                    }
                }
            }
            new_str(out)
        } else {
            MbValue::none()
        }
    }
}

/// partition(sep) → (before, sep, after) or (str, "", "") if not found
pub fn mb_str_partition(s: MbValue, sep: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sep)) {
            (Some(st), Some(pat)) => {
                if let Some(pos) = st.find(pat) {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(st[..pos].to_string()),
                        new_str(pat.to_string()),
                        new_str(st[pos + pat.len()..].to_string()),
                    ]))
                } else {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(st.to_string()),
                        new_str(String::new()),
                        new_str(String::new()),
                    ]))
                }
            }
            _ => MbValue::none(),
        }
    }
}

/// rpartition(sep) → (before, sep, after) from the right
pub fn mb_str_rpartition(s: MbValue, sep: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(sep)) {
            (Some(st), Some(pat)) => {
                if let Some(pos) = st.rfind(pat) {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(st[..pos].to_string()),
                        new_str(pat.to_string()),
                        new_str(st[pos + pat.len()..].to_string()),
                    ]))
                } else {
                    MbValue::from_ptr(MbObject::new_tuple(vec![
                        new_str(String::new()),
                        new_str(String::new()),
                        new_str(st.to_string()),
                    ]))
                }
            }
            _ => MbValue::none(),
        }
    }
}

/// removeprefix(prefix) → str without prefix (Python 3.9+)
pub fn mb_str_removeprefix(s: MbValue, prefix: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(prefix)) {
            (Some(st), Some(p)) => {
                if st.starts_with(p) {
                    new_str(st[p.len()..].to_string())
                } else {
                    new_str(st.to_string())
                }
            }
            _ => MbValue::none(),
        }
    }
}

/// removesuffix(suffix) → str without suffix (Python 3.9+)
pub fn mb_str_removesuffix(s: MbValue, suffix: MbValue) -> MbValue {
    unsafe {
        match (as_str(s), as_str(suffix)) {
            (Some(st), Some(p)) => {
                if st.ends_with(p) {
                    new_str(st[..st.len() - p.len()].to_string())
                } else {
                    new_str(st.to_string())
                }
            }
            _ => MbValue::none(),
        }
    }
}

// ── F-String Format Specifiers ──

/// Format a single value according to a Python format spec.
/// Supports: fill, align (<>^), width, precision, type (d,f,s,e,x,o,b).
pub fn mb_format_value(val: MbValue, spec: MbValue) -> MbValue {
    unsafe {
        // An Instance whose class registers __format__ formats itself
        // (ipaddress addresses, user classes); everything else goes through
        // the built-in spec formatter.
        if let Some(ptr) = val.as_ptr() {
            if let super::rc::ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let method = super::class::lookup_method(class_name, "__format__");
                if !method.is_none() {
                    // Direct method-value call: format() dispatches on the
                    // TYPE, ignoring a per-instance __format__ attribute.
                    return super::class::call_method_value2(method, val, spec);
                }
            }
        }
        let spec_str = as_str(spec).unwrap_or("");
        // Decimal / Fraction integer handles carry their own __format__
        // pipeline (#2129) — without this, the raw handle id is formatted.
        if let Some(out) = super::stdlib::decimal_mod::mb_numeric_handle_format(val, spec_str) {
            return out;
        }
        let formatted = format_with_spec(val, spec_str);
        new_str(formatted)
    }
}

/// Internal: apply a format spec to a value.
///
/// Delegates to the more complete `apply_format_spec` parser so f-strings and
/// `format(val, spec)` see the same feature set as `"{}".format(...)` (the
/// `#` alt-form flag, sign flag, `_` separator, `%` percent type, etc.).
/// Kept as a thin wrapper for call-site compatibility.
fn format_with_spec(val: MbValue, spec: &str) -> String {
    apply_format_spec(val, spec)
}

/// Spec-less f-string field (`f"{x}"`): CPython calls format(x, "") which
/// dispatches type-level `__format__`; objects without one fall back to
/// str(). Keeps the historical mb_str fast path for every non-instance.
pub fn mb_fstring_value(val: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = val.as_ptr() {
            if let super::rc::ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let method = super::class::lookup_method(class_name, "__format__");
                if !method.is_none() {
                    let empty = new_str(String::new());
                    return super::class::call_method_value2(method, val, empty);
                }
            }
        }
    }
    super::builtins::mb_str(val)
}

// ── Formatting ──

/// CPython-style `repr(float)` / `str(float)`. Lowercase `nan` / `inf` for
/// non-finite values; scientific notation when `|x| < 1e-4` or `|x| >= 1e16`
/// (Python's threshold); otherwise Rust's shortest-round-trip decimal with
/// `.0` appended when no fractional part is present.
pub fn python_float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".to_string();
    }
    if f.is_infinite() {
        return if f < 0.0 {
            "-inf".to_string()
        } else {
            "inf".to_string()
        };
    }
    let abs = f.abs();
    if abs != 0.0 && (abs < 1e-4 || abs >= 1e16) {
        let raw = format!("{:e}", f);
        return pythonize_exponent(&raw, false);
    }
    let s = format!("{}", f);
    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
        format!("{}.0", s)
    } else {
        s
    }
}

/// Reformat Rust's `{:e}` exponent (`1.23e3`, `1.23e-4`) into CPython style
/// (`1.23e+03`, `1.23e-04`): explicit sign, minimum two exponent digits.
fn pythonize_exponent(raw: &str, upper: bool) -> String {
    let marker = if upper { 'E' } else { 'e' };
    let Some(idx) = raw.rfind(marker) else {
        return raw.to_string();
    };
    let (mantissa, exp_part) = raw.split_at(idx);
    let exp_str = &exp_part[1..]; // skip the marker char
    let (sign, digits) = if let Some(rest) = exp_str.strip_prefix('-') {
        ('-', rest)
    } else if let Some(rest) = exp_str.strip_prefix('+') {
        ('+', rest)
    } else {
        ('+', exp_str)
    };
    let exp_num: i32 = digits.parse().unwrap_or(0);
    format!("{}{}{}{:02}", mantissa, marker, sign, exp_num)
}

/// Strip trailing zeros (and a trailing `.`) from a `g`-style result, unless
/// the alternate (`#`) form is in effect (which preserves the decimal point).
fn strip_g_trailing_zeros(s: &str, alternate: bool) -> String {
    if alternate {
        return s.to_string();
    }
    // Split off any exponent portion (e/E + sign/digits).
    let exp_idx = s.find(|c: char| c == 'e' || c == 'E');
    let (head, tail) = match exp_idx {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };
    let head_stripped = if head.contains('.') {
        let trimmed = head.trim_end_matches('0');
        trimmed.trim_end_matches('.').to_string()
    } else {
        head.to_string()
    };
    format!("{}{}", head_stripped, tail)
}

/// Format the magnitude of a finite float in Python 'g'-family style:
/// `precision` significant digits, switching to exponential past a threshold.
/// `none_type=false` is the 'g'/'G' presentation type (scientific when
/// `exp >= precision`); `none_type=true` is the empty presentation type with an
/// explicit precision, which switches one decade earlier (`exp >= precision-1`).
/// Returns the unsigned body only (no sign prefix). The decimal exponent is read
/// from the rounded 'e' form so a round-up across a power of ten (e.g. 99.0 at
/// precision 1 → "1e+02") picks the right branch, matching CPython.
fn format_g_magnitude(
    f_val: f64,
    precision: Option<usize>,
    upper: bool,
    alternate: bool,
    none_type: bool,
) -> String {
    // Python: precision 0 is treated as 1; default is 6.
    let p = precision.unwrap_or(6).max(1);
    let abs_v = f_val.abs();
    let e_prec = p - 1;
    let e_str = if upper {
        format!("{:.prec$E}", abs_v, prec = e_prec)
    } else {
        format!("{:.prec$e}", abs_v, prec = e_prec)
    };
    // Rust prints "<mantissa>e<exp>" with no '+' and no leading zeros.
    let exp: i32 = e_str
        .rsplit(|c| c == 'e' || c == 'E')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let sci_threshold = if none_type { p as i32 - 1 } else { p as i32 };
    if exp < -4 || exp >= sci_threshold {
        let r = strip_g_trailing_zeros(&e_str, alternate);
        pythonize_exponent(&r, upper)
    } else {
        let f_prec = (p as i32 - 1 - exp).max(0) as usize;
        let r = format!("{:.prec$}", abs_v, prec = f_prec);
        strip_g_trailing_zeros(&r, alternate)
    }
}

/// Apply a Python format spec (e.g., ".2f", ">10", "<10", "05d") to a value.
fn apply_format_spec(val: MbValue, spec: &str) -> String {
    if spec.is_empty() {
        return value_to_string(val);
    }
    let chars: Vec<char> = spec.chars().collect();
    let mut i = 0;

    // Parse fill and align
    let (fill, align) = if chars.len() >= 2 && matches!(chars[1], '<' | '>' | '^') {
        i = 2;
        (chars[0], chars[1])
    } else if !chars.is_empty() && matches!(chars[0], '<' | '>' | '^') {
        i = 1;
        (' ', chars[0])
    } else {
        (' ', '\0') // no alignment specified
    };

    // Parse sign (only for numeric)
    let sign = if i < chars.len() && matches!(chars[i], '+' | '-' | ' ') {
        i += 1;
        chars[i - 1]
    } else {
        '-'
    };

    // Parse "#" alternate form (adds 0b / 0o / 0x prefix for int radix formats).
    let alternate = if i < chars.len() && chars[i] == '#' {
        i += 1;
        true
    } else {
        false
    };

    // Parse zero-fill: "0" before width
    let zero_fill = if i < chars.len() && chars[i] == '0' {
        i += 1;
        true
    } else {
        false
    };

    // Parse width
    let width_start = i;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    let width: usize = if i > width_start {
        chars[width_start..i]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    // Parse thousands separator (`,` or `_`) before the precision / type char.
    let thousands = if i < chars.len() && (chars[i] == ',' || chars[i] == '_') {
        let sep = chars[i];
        i += 1;
        Some(sep)
    } else {
        None
    };

    // Parse precision (.N)
    let precision: Option<usize> = if i < chars.len() && chars[i] == '.' {
        i += 1;
        let prec_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        Some(
            chars[prec_start..i]
                .iter()
                .collect::<String>()
                .parse()
                .unwrap_or(0),
        )
    } else {
        None
    };

    // Parse type char (d, f, s, etc.)
    let type_char = if i < chars.len() { chars[i] } else { '\0' };

    // CPython format-code/type validation: a known code applied to the wrong
    // scalar type (or an unknown code) raises ValueError instead of silently
    // coercing. Only scalar values participate — container/instance handling
    // keeps its existing behavior.
    {
        let is_str_val = unsafe { as_str(val).is_some() };
        let is_bool_val = val.as_bool().is_some();
        let is_int_val = val.as_int().is_some() || is_bool_val;
        let is_float_val = !is_int_val && val.as_float().is_some();
        if is_str_val || is_int_val || is_float_val {
            let type_name = if is_str_val {
                "str"
            } else if is_float_val {
                "float"
            } else if is_bool_val {
                "bool"
            } else {
                "int"
            };
            let bad = match type_char {
                'b' | 'c' | 'd' | 'o' | 'x' | 'X' => !is_int_val,
                'e' | 'E' | 'f' | 'F' | 'g' | 'G' | '%' => is_str_val,
                'n' => is_str_val,
                's' => !is_str_val,
                // A second grouping separator after `,`/`_` (e.g. `,_`).
                ',' | '_' => true,
                '\0' => false,
                c => c.is_ascii_alphabetic(),
            };
            if bad {
                super::exception::mb_raise(
                    new_str("ValueError".to_string()),
                    new_str(format!(
                        "Unknown format code '{type_char}' for object of type '{type_name}'"
                    )),
                );
                return String::new();
            }
        }
    }

    // Build the sign/magnitude pieces separately so `+`/` ` prefix survives
    // zero-padding (CPython: `"{:+05d}".format(3)` → `"+0003"`).
    let extract_int = || -> i64 {
        val.as_int()
            .unwrap_or_else(|| val.as_float().map(|f| f as i64).unwrap_or(0))
    };
    let apply_thousands = |digits: &str, sep: char| -> String {
        let (minus, num) = digits
            .strip_prefix('-')
            .map(|r| (true, r))
            .unwrap_or((false, digits));
        let bytes = num.as_bytes();
        let mut out = Vec::with_capacity(num.len() + num.len() / 3);
        let first_group = bytes.len() % 3;
        for (idx, b) in bytes.iter().enumerate() {
            if idx > 0 && idx >= first_group && (idx - first_group) % 3 == 0 {
                out.push(sep as u8);
            }
            out.push(*b);
        }
        let with_sep = String::from_utf8(out).unwrap_or_else(|_| num.to_string());
        if minus {
            format!("-{with_sep}")
        } else {
            with_sep
        }
    };
    // Non-finite floats render as canonical inf/nan, with case following the
    // presentation letter (CPython): lowercase 'f'/'e'/'g'/'%'/None → inf/nan,
    // uppercase 'F'/'E'/'G' → INF/NAN. Rust's own `{}` prints "NaN"/"inf", so
    // intercept before the per-type formatting. Sign flags still apply; numeric
    // zero-padding does not (handled at the width step below).
    let nonfinite_float = val.as_float().filter(|fv| !fv.is_finite()).and_then(|fv| {
        if !matches!(type_char, 'f' | 'F' | 'e' | 'E' | 'g' | 'G' | '%' | '\0') {
            return None;
        }
        let upper = matches!(type_char, 'F' | 'E' | 'G');
        let word = if fv.is_nan() {
            if upper {
                "NAN"
            } else {
                "nan"
            }
        } else if upper {
            "INF"
        } else {
            "inf"
        };
        let prefix = if fv.is_sign_negative() {
            "-"
        } else if sign == '+' {
            "+"
        } else if sign == ' ' {
            " "
        } else {
            ""
        };
        Some((prefix.to_string(), word.to_string()))
    });
    let is_nonfinite_float = nonfinite_float.is_some();
    let (sign_prefix, body) = if let Some(nf) = nonfinite_float {
        nf
    } else {
        match type_char {
            '%' => {
                // Percent type: multiply by 100, format as fixed-point, append '%'.
                let f_val = val
                    .as_float()
                    .or_else(|| val.as_int().map(|i| i as f64))
                    .unwrap_or(0.0);
                let scaled = f_val * 100.0;
                let prec = precision.unwrap_or(6);
                let raw = format!("{:.prec$}", scaled.abs(), prec = prec);
                let prefix = if f_val.is_sign_negative() {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, format!("{raw}%"))
            }
            'e' | 'E' => {
                let f_val = val
                    .as_float()
                    .or_else(|| val.as_int().map(|i| i as f64))
                    .unwrap_or(0.0);
                let prec = precision.unwrap_or(6);
                let raw = if type_char == 'e' {
                    format!("{:.prec$e}", f_val.abs(), prec = prec)
                } else {
                    format!("{:.prec$E}", f_val.abs(), prec = prec)
                };
                let raw = pythonize_exponent(&raw, type_char == 'E');
                let prefix = if f_val.is_sign_negative() {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, raw)
            }
            'g' | 'G' => {
                let f_val = val
                    .as_float()
                    .or_else(|| val.as_int().map(|i| i as f64))
                    .unwrap_or(0.0);
                let raw = format_g_magnitude(f_val, precision, type_char == 'G', alternate, false);
                let prefix = if f_val.is_sign_negative() {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, raw)
            }
            'f' | 'F' => {
                let f_val = val
                    .as_float()
                    .or_else(|| val.as_int().map(|i| i as f64))
                    .unwrap_or(0.0);
                let prec = precision.unwrap_or(6);
                let raw = format!("{:.prec$}", f_val.abs(), prec = prec);
                let digits_body = if let Some(sep) = thousands {
                    // Only the integer part is grouped.
                    if let Some((int_part, frac)) = raw.split_once('.') {
                        format!("{}.{}", apply_thousands(int_part, sep), frac)
                    } else {
                        apply_thousands(&raw, sep)
                    }
                } else {
                    raw
                };
                let prefix = if f_val.is_sign_negative() {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, digits_body)
            }
            'd' => {
                let v = extract_int();
                let digits = if let Some(sep) = thousands {
                    apply_thousands(&v.abs().to_string(), sep)
                } else {
                    v.abs().to_string()
                };
                let prefix = if v < 0 {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, digits)
            }
            'b' | 'o' | 'x' | 'X' => {
                let v = extract_int();
                let abs_val = v.unsigned_abs();
                let digits = match type_char {
                    'b' => format!("{:b}", abs_val),
                    'o' => format!("{:o}", abs_val),
                    'x' => format!("{:x}", abs_val),
                    'X' => format!("{:X}", abs_val),
                    _ => unreachable!(),
                };
                let alt_prefix = if alternate {
                    match type_char {
                        'b' => "0b",
                        'o' => "0o",
                        'x' => "0x",
                        'X' => "0X",
                        _ => "",
                    }
                } else {
                    ""
                };
                let sign_part = if v < 0 {
                    "-".to_string()
                } else if sign == '+' {
                    "+".to_string()
                } else if sign == ' ' {
                    " ".to_string()
                } else {
                    String::new()
                };
                (format!("{sign_part}{alt_prefix}"), digits)
            }
            's' | '\0' => {
                // None-type float with an explicit precision is 'g'-like (significant
                // digits, exponential past the threshold) but keeps a trailing ".0"
                // when the result would otherwise look integral (CPython's
                // Py_DTSF_ADD_DOT_0). The `str`-style char truncation below only
                // applies to actual strings — a float here always has type '\0'
                // ('s' on a float already raised ValueError in the validation block).
                if thousands.is_none()
                    && precision.is_some()
                    && val.as_float().is_some()
                    && val.as_int().is_none()
                {
                    let f_val = val.as_float().unwrap();
                    let mut body = format_g_magnitude(f_val, precision, false, alternate, true);
                    if !body.contains(|c| c == '.' || c == 'e' || c == 'E') {
                        body.push_str(".0");
                    }
                    let prefix = if f_val.is_sign_negative() {
                        "-".to_string()
                    } else if sign == '+' {
                        "+".to_string()
                    } else if sign == ' ' {
                        " ".to_string()
                    } else {
                        String::new()
                    };
                    (prefix, body)
                } else
                // When no type is given but thousands was requested and the value
                // is numeric, behave like `d` / `f` so `"{:,}".format(1234567)` →
                // `"1,234,567"`.
                if thousands.is_some()
                    && (val.as_int().is_some() || val.as_float().is_some())
                {
                    let sep = thousands.unwrap();
                    if let Some(f) = val.as_float() {
                        let raw = format!("{:.6}", f.abs());
                        let body = if let Some((int_part, frac)) = raw.split_once('.') {
                            format!("{}.{}", apply_thousands(int_part, sep), frac)
                        } else {
                            apply_thousands(&raw, sep)
                        };
                        let prefix = if f.is_sign_negative() {
                            "-".to_string()
                        } else if sign == '+' {
                            "+".to_string()
                        } else if sign == ' ' {
                            " ".to_string()
                        } else {
                            String::new()
                        };
                        (prefix, body)
                    } else {
                        let v = val.as_int().unwrap_or(0);
                        let digits = apply_thousands(&v.abs().to_string(), sep);
                        let prefix = if v < 0 {
                            "-".to_string()
                        } else if sign == '+' {
                            "+".to_string()
                        } else if sign == ' ' {
                            " ".to_string()
                        } else {
                            String::new()
                        };
                        (prefix, digits)
                    }
                } else if matches!(sign, '+' | ' ')
                    && (val.as_int().is_some() || val.as_float().is_some())
                {
                    // Sign flag with no explicit type: treat numerics like `d` / `f`
                    // so `f"{42:+}"` → `"+42"`, `f"{-3.14: }"` → `"-3.14"`.
                    // Use `value_to_string` so floats keep CPython's repr (`0.0` not
                    // `0`, `1e100` not `10000…`); just split off any leading minus.
                    let raw = value_to_string(val);
                    let (is_neg, body) = match raw.strip_prefix('-') {
                        Some(rest) => (true, rest.to_string()),
                        None => (false, raw),
                    };
                    let prefix = if is_neg {
                        "-".to_string()
                    } else if sign == '+' {
                        "+".to_string()
                    } else {
                        " ".to_string()
                    };
                    (prefix, body)
                } else {
                    let s = if let Some(prec) = precision {
                        value_to_string(val).chars().take(prec).collect()
                    } else {
                        value_to_string(val)
                    };
                    (String::new(), s)
                }
            }
            _ => (String::new(), value_to_string(val)),
        }
    };
    let formatted = format!("{sign_prefix}{body}");

    // Apply width and alignment
    if width > formatted.len() {
        let padding = width - formatted.len();
        // Non-finite floats use space fill even when '0' was requested
        // (CPython: `format(inf, "010")` → `"       inf"`).
        let zero_fill = zero_fill && !is_nonfinite_float;
        let actual_fill = if zero_fill && align == '\0' {
            '0'
        } else {
            fill
        };
        let actual_align = if align == '\0' {
            if zero_fill {
                '>'
            } else if val.as_ptr().is_some() {
                '<'
            } else {
                '>'
            }
        } else {
            align
        };
        // Zero-padding with a sign prefix goes between sign and digits
        // (CPython: `"{:+05d}".format(3)` → `"+0003"`, not `"000+3"`).
        if zero_fill && align == '\0' && !sign_prefix.is_empty() {
            let zeros: String = std::iter::repeat('0').take(padding).collect();
            return format!("{sign_prefix}{zeros}{body}");
        }
        match actual_align {
            '<' => format!(
                "{}{}",
                formatted,
                std::iter::repeat(actual_fill)
                    .take(padding)
                    .collect::<String>()
            ),
            '>' => format!(
                "{}{}",
                std::iter::repeat(actual_fill)
                    .take(padding)
                    .collect::<String>(),
                formatted
            ),
            '^' => {
                let left = padding / 2;
                let right = padding - left;
                format!(
                    "{}{}{}",
                    std::iter::repeat(actual_fill)
                        .take(left)
                        .collect::<String>(),
                    formatted,
                    std::iter::repeat(actual_fill)
                        .take(right)
                        .collect::<String>()
                )
            }
            _ => formatted,
        }
    } else {
        formatted
    }
}

/// `"template" % args` — CPython's printf-style formatter. `args` is either
/// a single value (for templates with exactly one `%` conversion) or a tuple
/// of values. Supports a practical subset of CPython conversions — %d, %i,
/// %s, %r, %f, %x, %X, %o, %c, %%, plus width / precision / left-align flag
/// / zero-pad flag / sign flag. Doesn't aim for full CPython fidelity; covers
/// the common cases that tests and everyday code exercise.
pub fn mb_str_percent_format(tmpl: String, args: MbValue) -> MbValue {
    // Flatten args: single value → [value], tuple → tuple items, dict →
    // mapping (used by `%(key)s` conversions), other → [value].
    let mut arg_slots: Vec<MbValue> = Vec::new();
    let mut mapping: Option<MbValue> = None;
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => arg_slots.extend(items.iter().copied()),
                ObjData::Dict(_) => mapping = Some(args),
                _ => arg_slots.push(args),
            }
        }
    } else {
        arg_slots.push(args);
    }
    let mut arg_idx = 0usize;
    let mut out = String::new();
    let mut chars = tmpl.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        // %% literal
        if chars.peek() == Some(&'%') {
            chars.next();
            out.push('%');
            continue;
        }

        // Mapping key: `%(name)s` looks up `name` in the dict argument.
        let mut mapping_value: Option<MbValue> = None;
        if chars.peek() == Some(&'(') {
            chars.next();
            let mut key = String::new();
            let mut depth = 1usize;
            for ch in chars.by_ref() {
                if ch == '(' {
                    depth += 1;
                    key.push(ch);
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    key.push(ch);
                } else {
                    key.push(ch);
                }
            }
            if let Some(map) = mapping {
                let key_val = MbValue::from_ptr(super::rc::MbObject::new_str(key));
                mapping_value = Some(super::dict_ops::mb_dict_getitem(map, key_val));
            }
        }

        // Flags: -+ #0 space
        let mut left_align = false;
        let mut sign_plus = false;
        let mut sign_space = false;
        let mut alternate = false;
        let mut zero_pad = false;
        loop {
            match chars.peek() {
                Some('-') => {
                    left_align = true;
                    chars.next();
                }
                Some('+') => {
                    sign_plus = true;
                    chars.next();
                }
                Some(' ') => {
                    sign_space = true;
                    chars.next();
                }
                Some('#') => {
                    alternate = true;
                    chars.next();
                }
                Some('0') => {
                    zero_pad = true;
                    chars.next();
                }
                _ => break,
            }
        }

        // Width
        let mut width: usize = 0;
        if chars.peek() == Some(&'*') {
            chars.next();
            if let Some(v) = arg_slots.get(arg_idx) {
                width = v.as_int().unwrap_or(0).max(0) as usize;
                arg_idx += 1;
            }
        } else {
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    width = width * 10 + (d as usize - '0' as usize);
                    chars.next();
                } else {
                    break;
                }
            }
        }

        // Precision
        let mut precision: Option<usize> = None;
        if chars.peek() == Some(&'.') {
            chars.next();
            let mut p: usize = 0;
            if chars.peek() == Some(&'*') {
                chars.next();
                if let Some(v) = arg_slots.get(arg_idx) {
                    p = v.as_int().unwrap_or(0).max(0) as usize;
                    arg_idx += 1;
                }
            } else {
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        p = p * 10 + (d as usize - '0' as usize);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            precision = Some(p);
        }

        // Length modifiers (h/l/L) — accept and ignore.
        while let Some(&d) = chars.peek() {
            if matches!(d, 'h' | 'l' | 'L') {
                chars.next();
            } else {
                break;
            }
        }

        let conv = match chars.next() {
            Some(ch) => ch,
            None => {
                out.push('%');
                break;
            }
        };

        let val = if mapping_value.is_some() {
            mapping_value
        } else {
            arg_slots.get(arg_idx).copied()
        };
        if mapping_value.is_none() && !matches!(conv, '%') {
            arg_idx += 1;
        }

        let (mut sign_prefix, body) = match conv {
            'd' | 'i' => {
                if let Some((negative, digits)) =
                    val.and_then(|a| int_digits_for_percent(a, 10))
                {
                    let prefix = if negative {
                        "-".to_string()
                    } else if sign_plus {
                        "+".to_string()
                    } else if sign_space {
                        " ".to_string()
                    } else {
                        String::new()
                    };
                    (prefix, digits)
                } else {
                    let v = val.and_then(|a| a.as_float()).map(|f| f as i64).unwrap_or(0);
                    let prefix = if v < 0 {
                        "-".to_string()
                    } else if sign_plus {
                        "+".to_string()
                    } else if sign_space {
                        " ".to_string()
                    } else {
                        String::new()
                    };
                    (prefix, v.unsigned_abs().to_string())
                }
            }
            'f' | 'F' => {
                let v = val
                    .and_then(|a| a.as_float())
                    .or_else(|| val.and_then(|a| a.as_int()).map(|i| i as f64))
                    .unwrap_or(0.0);
                let prec = precision.unwrap_or(6);
                let body = format!("{:.prec$}", v.abs(), prec = prec);
                let prefix = if v.is_sign_negative() {
                    "-".to_string()
                } else if sign_plus {
                    "+".to_string()
                } else if sign_space {
                    " ".to_string()
                } else {
                    String::new()
                };
                (prefix, body)
            }
            's' => {
                let s = val.map(value_to_string).unwrap_or_default();
                let body = if let Some(p) = precision {
                    s.chars().take(p).collect()
                } else {
                    s
                };
                (String::new(), body)
            }
            'r' => {
                let s = val
                    .map(|v| {
                        let r = super::builtins::mb_repr(v);
                        r.as_ptr()
                            .and_then(|p| unsafe {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();
                (String::new(), s)
            }
            'x' | 'X' | 'o' | 'b' => {
                let radix = match conv {
                    'o' => 8,
                    'b' => 2,
                    _ => 16,
                };
                let (negative, mut body) = val
                    .and_then(|a| int_digits_for_percent(a, radix))
                    .unwrap_or_else(|| (false, "0".to_string()));
                if conv == 'X' {
                    body = body.to_ascii_uppercase();
                }
                let sign_part = if negative {
                    "-".to_string()
                } else if sign_plus {
                    "+".to_string()
                } else if sign_space {
                    " ".to_string()
                } else {
                    String::new()
                };
                let alt = if alternate {
                    match conv {
                        'x' => "0x",
                        'X' => "0X",
                        'o' => "0o",
                        'b' => "0b",
                        _ => "",
                    }
                } else {
                    ""
                };
                (format!("{sign_part}{alt}"), body)
            }
            'c' => {
                let s = val
                    .and_then(|a| a.as_int())
                    .and_then(|i| char::from_u32(i as u32).map(|c| c.to_string()))
                    .or_else(|| val.map(value_to_string))
                    .unwrap_or_default();
                (String::new(), s)
            }
            '%' => {
                out.push('%');
                continue;
            }
            _ => {
                // Unknown conversion — emit as-is and consume no arg.
                if !matches!(conv, '%') {
                    arg_idx -= 1;
                }
                out.push('%');
                out.push(conv);
                continue;
            }
        };

        // Apply width / alignment. Zero-pad lives between sign and body when
        // align is default (right) and zero flag is set.
        let total_len = sign_prefix.chars().count() + body.chars().count();
        if total_len < width {
            let pad = width - total_len;
            if left_align {
                out.push_str(&sign_prefix);
                out.push_str(&body);
                for _ in 0..pad {
                    out.push(' ');
                }
            } else if zero_pad && matches!(conv, 'd' | 'i' | 'f' | 'F' | 'x' | 'X' | 'o' | 'b') {
                out.push_str(&sign_prefix);
                for _ in 0..pad {
                    out.push('0');
                }
                out.push_str(&body);
            } else {
                for _ in 0..pad {
                    out.push(' ');
                }
                out.push_str(&sign_prefix);
                out.push_str(&body);
            }
        } else {
            out.push_str(&sign_prefix);
            out.push_str(&body);
        }
        let _ = (&mut sign_prefix,); // quiet mut-only-when-fused warning
    }
    new_str(out)
}

/// Resolve nested `{...}` inside a format spec. `"{:{}}".format("hi", 10)`
/// pulls the next positional arg (10) to build the actual spec (`"10"`), then
/// re-parses that as the real spec. Supports `{}` (auto index) and `{N}`.
fn resolve_nested_spec(spec: &str, arg_list: &[MbValue], auto_idx: &mut usize) -> String {
    if !spec.contains('{') {
        return spec.to_string();
    }
    let mut out = String::new();
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut inner = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
            }
            let val = if inner.is_empty() {
                let v = arg_list
                    .get(*auto_idx)
                    .copied()
                    .unwrap_or_else(MbValue::none);
                *auto_idx += 1;
                v
            } else if let Ok(idx) = inner.parse::<usize>() {
                arg_list.get(idx).copied().unwrap_or_else(MbValue::none)
            } else {
                MbValue::none()
            };
            out.push_str(&value_to_string(val));
        } else {
            out.push(c);
        }
    }
    out
}

/// Resolve a nested format spec for the kwargs-aware path: like
/// `resolve_nested_spec` but the inner `{...}` references may be positional
/// (`{}` auto, `{N}`) OR keyword (`{name}`). Used by `mb_str_format_kwargs`
/// so `"{x:>{w}}".format(x='hi', w=5)` and `"{:>{}}".format(*['hi', 5])`
/// match CPython instead of leaking the unexpanded spec.
fn resolve_nested_spec_kwargs(
    spec: &str,
    pos_list: &[MbValue],
    kw_map: &std::collections::HashMap<String, MbValue>,
    auto_idx: &mut usize,
) -> String {
    if !spec.contains('{') {
        return spec.to_string();
    }
    let mut out = String::new();
    let mut chars = spec.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut inner = String::new();
            for ch in chars.by_ref() {
                if ch == '}' {
                    break;
                }
                inner.push(ch);
            }
            let val = if inner.is_empty() {
                let v = pos_list
                    .get(*auto_idx)
                    .copied()
                    .unwrap_or_else(MbValue::none);
                *auto_idx += 1;
                v
            } else if let Ok(idx) = inner.parse::<usize>() {
                pos_list.get(idx).copied().unwrap_or_else(MbValue::none)
            } else {
                kw_map
                    .get(inner.as_str())
                    .copied()
                    .unwrap_or_else(MbValue::none)
            };
            out.push_str(&value_to_string(val));
        } else {
            out.push(c);
        }
    }
    out
}

/// str.maketrans(x, y=None, z=None) — build a translation table for translate().
///
/// Supported forms:
/// * maketrans({ord_or_char: ord_or_char_or_str_or_None}) — dict form
/// * maketrans(x, y) — x and y are equal-length strings; char[i] in x maps to char[i] in y
/// * maketrans(x, y, z) — like above, plus chars in z map to None (deleted)
///
/// Returns a dict keyed by the source codepoint (int) → replacement MbValue.
pub fn mb_str_translate_dict_from_pairs(
    from_chars: &str,
    to_chars: &str,
    delete_chars: &str,
) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut guard = lock.write().unwrap();
            for (f, t) in from_chars.chars().zip(to_chars.chars()) {
                let k = super::dict_ops::DictKey::Int(f as i64);
                let v = MbValue::from_int(t as i64);
                guard.insert(k, v);
            }
            for c in delete_chars.chars() {
                let k = super::dict_ops::DictKey::Int(c as i64);
                guard.insert(k, MbValue::none());
            }
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_str_maketrans(x: MbValue, y: MbValue, z: MbValue) -> MbValue {
    // 1-arg form: dict input — normalize keys to codepoints.
    if y.is_none() {
        if let Some(ptr) = x.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let src = lock.read().unwrap();
                    let out = MbObject::new_dict();
                    if let ObjData::Dict(ref out_lock) = (*out).data {
                        let mut guard = out_lock.write().unwrap();
                        for (k, v) in src.iter() {
                            let key_int = match k {
                                super::dict_ops::DictKey::Int(i) => *i,
                                super::dict_ops::DictKey::Str(s) if s.chars().count() == 1 => {
                                    s.chars().next().unwrap() as i64
                                }
                                _ => continue,
                            };
                            guard.insert(super::dict_ops::DictKey::Int(key_int), *v);
                        }
                    }
                    return MbValue::from_ptr(out);
                }
            }
        }
        return MbValue::from_ptr(MbObject::new_dict());
    }
    // 2/3-arg form: equal-length strings x/y, optional delete string z.
    let x_s = as_str_owned(x).unwrap_or_default();
    let y_s = as_str_owned(y).unwrap_or_default();
    let z_s = as_str_owned(z).unwrap_or_default();
    mb_str_translate_dict_from_pairs(&x_s, &y_s, &z_s)
}

/// str.translate(table) — apply a codepoint-indexed translation dict.
pub fn mb_str_translate(s: MbValue, table: MbValue) -> MbValue {
    let src = match as_str_owned(s) {
        Some(x) => x,
        None => return MbValue::none(),
    };
    let tbl_ptr = match table.as_ptr() {
        Some(p) => p,
        None => return new_str(src),
    };
    let mut out = String::with_capacity(src.len());
    unsafe {
        if let ObjData::Dict(ref lock) = (*tbl_ptr).data {
            let guard = lock.read().unwrap();
            for c in src.chars() {
                let k = super::dict_ops::DictKey::Int(c as i64);
                match guard.get(&k) {
                    Some(v) if v.is_none() => {} // delete
                    Some(v) => {
                        if let Some(i) = v.as_int() {
                            if let Some(ch) = char::from_u32(i as u32) {
                                out.push(ch);
                            }
                        } else if let Some(vp) = v.as_ptr() {
                            if let ObjData::Str(ref rep) = (*vp).data {
                                out.push_str(rep);
                            }
                        }
                    }
                    None => out.push(c),
                }
            }
        } else {
            return new_str(src);
        }
    }
    new_str(out)
}

/// Helper: clone an MbValue's string contents if it's a Str, else None.
fn as_str_owned(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// str.format_map(mapping) — like `format(**mapping)` but uses the mapping
/// directly; `{name}` placeholders look up via __getitem__ semantics.
pub fn mb_str_format_map(s: MbValue, mapping: MbValue) -> MbValue {
    let template = match as_str_owned(s) {
        Some(t) => t,
        None => return MbValue::none(),
    };
    let mapping_ptr = mapping.as_ptr();
    let lookup = |name: &str| -> Option<MbValue> {
        let ptr = mapping_ptr?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                let k = super::dict_ops::DictKey::Str(name.to_string());
                return guard.get(&k).copied();
            }
            if matches!(&(*ptr).data, ObjData::List(_)) {
                raise_type_error("list indices must be integers or slices, not str");
                return None;
            }
            // Mapping-protocol objects (e.g. re.Match) look up via
            // __getitem__ semantics.
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "re.Match" {
                    return fields
                        .read()
                        .unwrap()
                        .get(&format!("group_name_{name}"))
                        .copied();
                }
            }
        }
        None
    };
    let mut out = String::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            if chars.peek() == Some(&'{') {
                chars.next();
                out.push('{');
                continue;
            }
            let mut field = String::new();
            let mut depth = 1u32;
            for c in chars.by_ref() {
                if c == '{' {
                    depth += 1;
                    field.push(c);
                    continue;
                }
                if c == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    field.push(c);
                    continue;
                }
                field.push(c);
            }
            let (field_name, fmt_spec) = if let Some(colon_pos) = field.find(':') {
                (&field[..colon_pos], &field[colon_pos + 1..])
            } else {
                (field.as_str(), "")
            };
            match lookup(field_name) {
                Some(val) => out.push_str(&apply_format_spec(val, fmt_spec)),
                None => {
                    if super::exception::mb_has_exception().as_bool() == Some(true) {
                        return MbValue::none();
                    }
                    // Pass the bare key — KeyError.__str__ applies repr-once
                    // when printed. Pre-quoting here doubles the escapes
                    // (e.g. `caught: '\'missing\''` instead of `caught: 'missing'`).
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(field_name.to_string())),
                    );
                    return MbValue::none();
                }
            }
        } else if ch == '}' {
            if chars.peek() == Some(&'}') {
                chars.next();
            }
            out.push('}');
        } else {
            out.push(ch);
        }
    }
    new_str(out)
}

/// Split a format-spec field name into a head (before any `.` / `[`) and the
/// trailing path. Returns `(head, path_suffix)` where `path_suffix` starts at
/// the first `.` or `[`. CPython lexer rule, simplified for our subset.
fn split_field_head(name: &str) -> (&str, &str) {
    let bytes = name.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'.' || b == b'[' {
            return (&name[..i], &name[i..]);
        }
    }
    (name, "")
}

/// Apply a field-name path (`.attr` and `[key]` segments) to a base value.
/// Returns `None` if any step fails — caller decides whether to surface a
/// literal placeholder or `None`-sub. Mirrors CPython's `Formatter.get_field`
/// resolution for the cases we already support: list/tuple int-index, dict
/// str-index, and instance attribute lookup.
unsafe fn resolve_field_path(mut val: MbValue, mut path: &str) -> Option<MbValue> {
    while !path.is_empty() {
        let bytes = path.as_bytes();
        if bytes[0] == b'.' {
            // .attr — read until next `.` or `[`.
            let rest = &path[1..];
            let (attr, after) = split_field_head(rest);
            if attr.is_empty() {
                return None;
            }
            let attr_val = MbValue::from_ptr(MbObject::new_str(attr.to_string()));
            let got = super::class::mb_getattr(val, attr_val);
            if got.is_none() {
                // Attribute field-access miss is an error (CPython raises;
                // mamba previously left the field literal). Raise AttributeError
                // unless mb_getattr already set a pending exception.
                if super::exception::mb_has_exception().as_bool() != Some(true) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "object has no attribute '{attr}'"
                        ))),
                    );
                }
                return None;
            }
            val = got;
            path = after;
        } else if bytes[0] == b'[' {
            // [key] — read until matching ']'. CPython doesn't allow nested
            // brackets here, so a flat scan is enough.
            let close = path.find(']')?;
            let key_str = &path[1..close];
            let after = &path[close + 1..];
            // CPython's format-spec lexer treats `[N]` as int only when N is
            // ascii-digits-only (no `-`, `+`, leading whitespace). `[-1]` on a
            // list raises TypeError in CPython because it's parsed as a string
            // key. Mirror that: digits-only → int, anything else → str.
            let is_int_key = !key_str.is_empty() && key_str.bytes().all(|b| b.is_ascii_digit());
            if is_int_key {
                let idx: usize = key_str.parse().ok()?;
                let ptr = val.as_ptr()?;
                let next = match &(*ptr).data {
                    ObjData::List(lock) => {
                        let v = lock.read().unwrap();
                        if idx >= v.len() {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "list index out of range".to_string(),
                                )),
                            );
                            return None;
                        }
                        Some(v[idx])
                    }
                    ObjData::Tuple(items) => {
                        if idx >= items.len() {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "tuple index out of range".to_string(),
                                )),
                            );
                            return None;
                        }
                        Some(items[idx])
                    }
                    // A digit key on a mapping is an integer key lookup, not a
                    // sequence index (`'{0[2]}'.format({})` → KeyError 2).
                    ObjData::Dict(lock) => {
                        let map = lock.read().unwrap();
                        let got = map.get(&super::dict_ops::DictKey::Int(idx as i64)).copied();
                        if got.is_none() {
                            drop(map);
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!("{idx}"))),
                            );
                        }
                        got
                    }
                    _ => None,
                };
                val = next?;
            } else {
                let ptr = val.as_ptr()?;
                let next = match &(*ptr).data {
                    ObjData::Dict(lock) => {
                        let map = lock.read().unwrap();
                        let got = map.get(key_str).copied();
                        if got.is_none() {
                            drop(map);
                            // Missing mapping key in a field access raises
                            // KeyError (CPython), not a literal passthrough.
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!("'{key_str}'"))),
                            );
                        }
                        got
                    }
                    _ => None,
                };
                val = next?;
            }
            path = after;
        } else {
            return None;
        }
    }
    Some(val)
}

/// str.format(*args, **kwargs) — positional ({}, {0}), keyword ({name}).
pub fn mb_str_format(s: MbValue, args: MbValue) -> MbValue {
    unsafe {
        let template = match as_str(s) {
            Some(t) => t,
            None => return MbValue::none(),
        };
        let arg_list = match args.as_ptr() {
            Some(ptr) => match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => return MbValue::none(),
            },
            None => return MbValue::none(),
        };
        let mut result = String::new();
        let mut auto_idx = 0usize;
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    result.push('{');
                    continue;
                }
                // Read the field — balance nested `{...}` so `{:{}}` captures
                // the whole inner spec rather than cutting at the first `}`.
                let mut field = String::new();
                let mut depth = 1u32;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                        field.push(c);
                        continue;
                    }
                    if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        field.push(c);
                        continue;
                    }
                    field.push(c);
                }
                // Split field into name/index and format spec at ':'
                let (field_name, fmt_spec) = if let Some(colon_pos) = field.find(':') {
                    (&field[..colon_pos], &field[colon_pos + 1..])
                } else {
                    (field.as_str(), "")
                };
                // CPython: the outer field's value comes from auto_idx first,
                // THEN the nested `{}` inside the spec bumps auto_idx further.
                // Capture the value slot before resolving the inner spec.
                let (head, path) = split_field_head(field_name);
                if head.is_empty() {
                    if auto_idx < arg_list.len() {
                        let mut value = arg_list[auto_idx];
                        auto_idx += 1;
                        if !path.is_empty() {
                            value = match resolve_field_path(value, path) {
                                Some(v) => v,
                                None => {
                                    // A raised field-access error (IndexError /
                                    // AttributeError) propagates; otherwise keep
                                    // the legacy literal passthrough.
                                    if super::exception::mb_has_exception().as_bool() == Some(true)
                                    {
                                        return MbValue::none();
                                    }
                                    result.push('{');
                                    result.push_str(&field);
                                    result.push('}');
                                    continue;
                                }
                            };
                        }
                        let resolved_spec = resolve_nested_spec(fmt_spec, &arg_list, &mut auto_idx);
                        result.push_str(&apply_format_spec(value, &resolved_spec));
                    }
                } else if let Ok(idx) = head.parse::<usize>() {
                    let resolved_spec = resolve_nested_spec(fmt_spec, &arg_list, &mut auto_idx);
                    if idx < arg_list.len() {
                        let mut value = arg_list[idx];
                        if !path.is_empty() {
                            value = match resolve_field_path(value, path) {
                                Some(v) => v,
                                None => {
                                    // A raised field-access error (IndexError /
                                    // AttributeError) propagates; otherwise keep
                                    // the legacy literal passthrough.
                                    if super::exception::mb_has_exception().as_bool() == Some(true)
                                    {
                                        return MbValue::none();
                                    }
                                    result.push('{');
                                    result.push_str(&field);
                                    result.push('}');
                                    continue;
                                }
                            };
                        }
                        result.push_str(&apply_format_spec(value, &resolved_spec));
                    }
                } else {
                    if head.starts_with('!') {
                        result.push('{');
                        result.push_str(&field);
                        result.push('}');
                        continue;
                    }
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(head.to_string())),
                    );
                    return MbValue::none();
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                result.push('}');
            } else {
                result.push(ch);
            }
        }
        new_str(result)
    }
}

/// str.format(*args, **kwargs) — with keyword argument support.
/// Takes the template string, a positional args list, and a kwargs dict.
pub fn mb_str_format_kwargs(s: MbValue, pos_args: MbValue, kwargs: MbValue) -> MbValue {
    // `.format(...)` is lowered to mb_str_format_kwargs regardless of receiver
    // type. When the receiver is a string.Formatter (or subclass) instance,
    // delegate to the real Formatter engine so `Formatter().format(fmt, kw=..)`
    // and subclass overrides work instead of treating the instance as a
    // template string.
    // logging.Formatter shares the "Formatter" class name with string.Formatter
    // but has a totally different `.format(record)` contract — route it to the
    // logging engine first.
    if super::stdlib::logging_mod::value_is_logging_formatter(s) {
        let items = super::builtins::extract_items(pos_args);
        let record = items.first().copied().unwrap_or_else(MbValue::none);
        return super::stdlib::logging_mod::logging_formatter_format(s, record);
    }
    if super::stdlib::string_constants_mod::value_is_formatter(s) {
        return super::stdlib::string_constants_mod::formatter_format_from_kwargs(
            s, pos_args, kwargs,
        );
    }
    unsafe {
        let template = match as_str(s) {
            Some(t) => t,
            None => return MbValue::none(),
        };
        let pos_list: Vec<MbValue> = match pos_args.as_ptr() {
            Some(ptr) => match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => vec![],
            },
            None => vec![],
        };
        // Build keyword map from dict
        let kw_map: std::collections::HashMap<String, MbValue> = match kwargs.as_ptr() {
            Some(ptr) => match &(*ptr).data {
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    map.iter().map(|(k, &v)| (k.to_string(), v)).collect()
                }
                _ => std::collections::HashMap::new(),
            },
            None => std::collections::HashMap::new(),
        };
        let mut result = String::new();
        let mut auto_idx = 0usize;
        let mut chars = template.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    result.push('{');
                    continue;
                }
                // Balance nested `{...}` so `{x:>{w}}` captures the whole inner
                // spec rather than cutting at the first `}` (matches the
                // positional `mb_str_format` field reader).
                let mut field = String::new();
                let mut depth = 1u32;
                for c in chars.by_ref() {
                    if c == '{' {
                        depth += 1;
                        field.push(c);
                        continue;
                    }
                    if c == '}' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                        field.push(c);
                        continue;
                    }
                    field.push(c);
                }
                // Split field into name/index and format spec at ':'
                let (field_name, fmt_spec) = if let Some(colon_pos) = field.find(':') {
                    (&field[..colon_pos], &field[colon_pos + 1..])
                } else {
                    (field.as_str(), "")
                };
                let (head, path) = split_field_head(field_name);
                let base: Option<MbValue> = if head.is_empty() {
                    if auto_idx < pos_list.len() {
                        let v = pos_list[auto_idx];
                        auto_idx += 1;
                        Some(v)
                    } else {
                        None
                    }
                } else if let Ok(idx) = head.parse::<usize>() {
                    if idx < pos_list.len() {
                        Some(pos_list[idx])
                    } else {
                        None
                    }
                } else {
                    kw_map.get(head).copied()
                };
                match base {
                    Some(mut value) => {
                        if !path.is_empty() {
                            match resolve_field_path(value, path) {
                                Some(v) => value = v,
                                None => {
                                    result.push('{');
                                    result.push_str(&field);
                                    result.push('}');
                                    continue;
                                }
                            }
                        }
                        // Resolve any nested `{...}` inside the spec AFTER the
                        // outer field claimed its auto-index slot, mirroring
                        // CPython's evaluation order.
                        let resolved_spec =
                            resolve_nested_spec_kwargs(fmt_spec, &pos_list, &kw_map, &mut auto_idx);
                        result.push_str(&apply_format_spec(value, &resolved_spec));
                    }
                    None => {
                        result.push('{');
                        result.push_str(&field);
                        result.push('}');
                    }
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                result.push('}');
            } else {
                result.push(ch);
            }
        }
        new_str(result)
    }
}

/// Format a byte slice as a Python bytes literal body: picks quote, escapes
/// non-printable bytes as \\xHH, using CPython repr rules.
fn format_bytes_inline(data: &[u8]) -> String {
    let has_single = data.contains(&b'\'');
    let has_double = data.contains(&b'"');
    let use_double = has_single && !has_double;
    let quote = if use_double { b'"' } else { b'\'' };
    let mut out = String::with_capacity(data.len() + 2);
    out.push(quote as char);
    for &b in data {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c as char);
            }
            0x20..=0x7E => out.push(b as char),
            c => out.push_str(&format!("\\x{c:02x}")),
        }
    }
    out.push(quote as char);
    out
}

/// Convert a MbValue to its string representation.
/// Format a value as a repr-like string for use inside containers (lists, tuples, etc.).
/// Strings get single-quoted with Python escape sequences (\\n, \\r, \\t, etc.).
fn repr_in_container(val: MbValue) -> String {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    let has_single = s.contains('\'');
                    let has_double = s.contains('"');
                    let use_double = has_single && !has_double;
                    let quote_char = if use_double { '"' } else { '\'' };
                    let mut escaped = String::with_capacity(s.len() + 2);
                    for c in s.chars() {
                        match c {
                            '\\' => escaped.push_str("\\\\"),
                            '\'' if !use_double => escaped.push_str("\\'"),
                            '"' if use_double => escaped.push_str("\\\""),
                            '\n' => escaped.push_str("\\n"),
                            '\r' => escaped.push_str("\\r"),
                            '\t' => escaped.push_str("\\t"),
                            c if c.is_control() => {
                                let cp = c as u32;
                                if cp < 0x100 {
                                    escaped.push_str(&format!("\\x{:02x}", cp));
                                } else {
                                    escaped.push_str(&format!("\\u{:04x}", cp));
                                }
                            }
                            c => escaped.push(c),
                        }
                    }
                    return format!("{quote_char}{escaped}{quote_char}");
                }
                ObjData::Instance { class_name, .. } => {
                    // In container context Python uses repr, not str — prefer
                    // __repr__ so `print([obj])` shows obj.__repr__() for each
                    // element rather than falling back to __str__.
                    let repr_method = super::class::lookup_method(class_name, "__repr__");
                    if !repr_method.is_none() {
                        let result = super::class::mb_call_method1(repr_method, val);
                        if let Some(rp) = result.as_ptr() {
                            if let ObjData::Str(ref s) = (*rp).data {
                                return s.clone();
                            }
                        }
                        return value_to_string(result);
                    }
                }
                _ => {}
            }
        }
    }
    value_to_string(val)
}

pub fn value_to_string(val: MbValue) -> String {
    // TAG_FUNC user-defined functions render as `<function NAME at 0xADDR>`
    // (CPython parity). Closure handles share TAG_INT with low-value ints
    // (closure IDs start at 1), so we restrict detection to TAG_FUNC only
    // to avoid corrupting integer rendering.
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        return format!("<function {name} at 0x{addr:x}>");
    }
    if let Some(i) = val.as_int() {
        format!("{i}")
    } else if let Some(f) = val.as_float() {
        python_float_repr(f)
    } else if let Some(b) = val.as_bool() {
        (if b { "True" } else { "False" }).to_string()
    } else if val.is_none() {
        "None".to_string()
    } else if val.is_not_implemented() {
        "NotImplemented".to_string()
    } else if val.is_ellipsis() {
        "Ellipsis".to_string()
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => s.clone(),
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let parts: Vec<String> = items.iter().map(|v| repr_in_container(*v)).collect();
                    format!("[{}]", parts.join(", "))
                }
                ObjData::Dict(ref lock) => {
                    let items = lock.read().unwrap();
                    let parts: Vec<String> = items
                        .iter()
                        .map(|(k, v)| {
                            format!(
                                "{}: {}",
                                super::dict_ops::dict_key_display(k),
                                repr_in_container(*v)
                            )
                        })
                        .collect();
                    format!("{{{}}}", parts.join(", "))
                }
                ObjData::Tuple(items) => {
                    let parts: Vec<String> = items.iter().map(|v| repr_in_container(*v)).collect();
                    if items.len() == 1 {
                        format!("({},)", parts[0])
                    } else {
                        format!("({})", parts.join(", "))
                    }
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                } => {
                    // Type objects: str(type) -> "<class 'name'>", matching repr
                    // and CPython's type.__str__ behavior.
                    if class_name == "type" {
                        let name = fields
                            .read()
                            .ok()
                            .and_then(|f| f.get("__name__").copied())
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| if let ObjData::Str(ref s) = (*p).data {
                                Some(s.clone())
                            } else {
                                None
                            });
                        if let Some(name) = name {
                            return format!("<class '{name}'>");
                        }
                    }
                    if class_name == "UnionType" {
                        return super::builtins::union_type_repr(val);
                    }
                    // Class-body enum member without a user __str__:
                    // str(Color.RED) → "Color.RED".
                    if let Some(s) = super::stdlib::enum_class::member_str(val) {
                        return s;
                    }
                    // slice repr matches CPython's "slice(start, stop, step)"
                    // surface — keep print() / str() / repr() consistent. (#1256)
                    if class_name == "slice" {
                        let f = fields.read().unwrap();
                        let s = f.get("start").copied().unwrap_or(MbValue::none());
                        let e = f.get("stop").copied().unwrap_or(MbValue::none());
                        let st = f.get("step").copied().unwrap_or(MbValue::none());
                        drop(f);
                        return format!(
                            "slice({}, {}, {})",
                            repr_in_container(s),
                            repr_in_container(e),
                            repr_in_container(st)
                        );
                    }
                    // memoryview repr matches CPython's "<memory at 0x...>"
                    // surface. (#1256)
                    if class_name == "memoryview" {
                        return format!("<memory at 0x{:x}>", ptr as usize);
                    }
                    // weakref.ref: str(r) falls back to repr, naming the
                    // referent's class (gh-99184).
                    if class_name == "ReferenceType" {
                        let r = super::stdlib::weakref_mod::reference_repr(val);
                        if let Some(rp) = r.as_ptr() {
                            if let ObjData::Str(ref s) = (*rp).data {
                                return s.clone();
                            }
                        }
                    }
                    // Counter has its own CPython-style repr; print(c) and
                    // str(c) both go through it. (#1638)
                    if class_name == "collections.Counter" {
                        return super::stdlib::collections_mod::counter_repr(val);
                    }
                    // defaultdict / deque also need CPython-style repr. (#1640)
                    if class_name == "collections.defaultdict" {
                        return super::stdlib::collections_mod::defaultdict_repr(val);
                    }
                    if class_name == "collections.deque" {
                        return super::stdlib::collections_mod::deque_repr(val);
                    }
                    if class_name == "collections.OrderedDict" {
                        return super::stdlib::collections_mod::ordereddict_repr(val);
                    }
                    // re.Match / re.Pattern CPython-style repr. (#1642)
                    if class_name == "re.Match" {
                        return super::stdlib::re_mod::match_repr(val);
                    }
                    if class_name == "re.Pattern" {
                        return super::stdlib::re_mod::pattern_repr(val);
                    }
                    // datetime / timedelta str form: ISO datetime / [D days,] H:MM:SS. (#1644)
                    if class_name == "datetime.datetime" {
                        return super::stdlib::datetime_mod::datetime_str(val);
                    }
                    if class_name == "datetime.timedelta" {
                        return super::stdlib::datetime_mod::timedelta_str(val);
                    }
                    if class_name == "datetime.time" {
                        return super::stdlib::datetime_mod::time_str(val);
                    }
                    if class_name == "datetime.timezone" {
                        return super::stdlib::datetime_mod::timezone_str(val);
                    }
                    // zoneinfo.ZoneInfo str is its key (e.g. "America/New_York").
                    if class_name == "ZoneInfo" {
                        if let Some(k) = fields.read().ok()
                            .and_then(|f| f.get("key").copied())
                        {
                            if let Some(p) = k.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    return s.clone();
                                }
                            }
                        }
                    }
                    // namedtuple: dynamic class_name → marker-field dispatch. (#1648)
                    if let Some(s) = super::stdlib::collections_mod::namedtuple_repr(val) {
                        return s;
                    }
                    // __str__ dunder dispatch — Python's str(obj) calls obj.__str__()
                    let str_method = super::class::lookup_method(class_name, "__str__");
                    if !str_method.is_none() {
                        let result = super::class::mb_call_method1(str_method, val);
                        if let Some(ptr) = result.as_ptr() {
                            if let ObjData::Str(ref s) = (*ptr).data {
                                return s.clone();
                            }
                        }
                        return value_to_string(result);
                    }
                    // Fallback: __repr__ dunder
                    let repr_method = super::class::lookup_method(class_name, "__repr__");
                    if !repr_method.is_none() {
                        let result = super::class::mb_call_method1(repr_method, val);
                        if let Some(ptr) = result.as_ptr() {
                            if let ObjData::Str(ref s) = (*ptr).data {
                                return s.clone();
                            }
                        }
                        return value_to_string(result);
                    }
                    if let Some((_base, payload)) =
                        super::class::builtin_data_payload_if_unoverridden(val, "__str__")
                    {
                        return value_to_string(payload);
                    }
                    // PEP 654 ExceptionGroup str: "message (N sub-exceptions)".
                    if super::exception::is_subclass_of(class_name, "BaseExceptionGroup")
                        || super::exception::is_subclass_of(class_name, "ExceptionGroup")
                        || class_name == "BaseExceptionGroup"
                        || class_name == "ExceptionGroup"
                    {
                        let g = fields.read().unwrap();
                        let msg_v = g.get("message").copied();
                        let exc_v = g.get("exceptions").copied();
                        drop(g);
                        if let (Some(msg_v), Some(exc_v)) = (msg_v, exc_v) {
                            let n = exc_v.as_ptr().map(|p| unsafe {
                                match &(*p).data {
                                    ObjData::Tuple(ref t) => t.len(),
                                    ObjData::List(ref l) => l.read().unwrap().len(),
                                    _ => 0,
                                }
                            });
                            let m = msg_v.as_ptr().and_then(|p| unsafe {
                                if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
                            });
                            if let (Some(n), Some(m)) = (n, m) {
                                return format!(
                                    "{m} ({n} sub-exception{})",
                                    if n == 1 { "" } else { "s" }
                                );
                            }
                        }
                    }
                    // Exception str: 0 args → ""; 1 arg → str(arg0);
                    // ≥2 args → repr of the args tuple. KeyError keeps its
                    // quirk where __str__ is repr(args[0]). (#1652)
                    let fields_guard = fields.read().unwrap();
                    if let Some(args_v) = fields_guard.get("args").copied() {
                        let items: Option<Vec<MbValue>> = args_v.as_ptr().and_then(|p| {
                            if let ObjData::Tuple(ref it) = (*p).data {
                                Some(it.clone())
                            } else {
                                None
                            }
                        });
                        if let Some(items) = items {
                            drop(fields_guard);
                            return match items.len() {
                                0 => String::new(),
                                1 => {
                                    let a0 = items[0];
                                    if class_name == "KeyError" {
                                        // KeyError.__str__ is repr(key); use the
                                        // shared repr for CPython-matching quote
                                        // selection (a `'`-containing key is
                                        // double-quoted, not single-escaped).
                                        let r = super::builtins::mb_repr(a0);
                                        if let Some(p) = r.as_ptr() {
                                            if let ObjData::Str(ref rs) = (*p).data {
                                                return rs.clone();
                                            }
                                        }
                                    }
                                    value_to_string(a0)
                                }
                                _ => {
                                    let tuple_val =
                                        MbValue::from_ptr(super::rc::MbObject::new_tuple(items));
                                    let r = super::builtins::mb_repr(tuple_val);
                                    r.as_ptr()
                                        .and_then(|p| {
                                            if let ObjData::Str(ref s) = (*p).data {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_default()
                                }
                            };
                        }
                    }
                    if let Some(msg_val) = fields_guard.get("message") {
                        if let Some(msg_ptr) = msg_val.as_ptr() {
                            if let ObjData::Str(ref s) = (*msg_ptr).data {
                                if class_name == "KeyError" {
                                    let r = super::builtins::mb_repr(*msg_val);
                                    if let Some(p) = r.as_ptr() {
                                        if let ObjData::Str(ref rs) = (*p).data {
                                            return rs.clone();
                                        }
                                    }
                                }
                                return s.clone();
                            }
                        }
                        return String::new();
                    }
                    format!("<{class_name} instance>")
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap();
                    if items.is_empty() {
                        "set()".to_string()
                    } else {
                        let parts: Vec<String> =
                            items.iter().map(|v| value_to_string(*v)).collect();
                        format!("{{{}}}", parts.join(", "))
                    }
                }
                ObjData::FrozenSet(items) => {
                    if items.is_empty() {
                        "frozenset()".to_string()
                    } else {
                        let parts: Vec<String> =
                            items.iter().map(|v| value_to_string(*v)).collect();
                        format!("frozenset({{{}}})", parts.join(", "))
                    }
                }
                ObjData::Bytes(data) => format!("b{}", format_bytes_inline(data)),
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    format!("bytearray(b{})", format_bytes_inline(&data))
                }
                ObjData::BigInt(big) => big.to_string(),
                ObjData::Complex(re, im) => complex_repr_string(*re, *im),
                ObjData::CodeObject { filename, mode, .. } => {
                    format!("<code object <module> at {filename} mode={mode}>")
                }
            }
        }
    } else {
        String::new()
    }
}

// ── Method Dispatch ──

/// repr() of a complex value, matching CPython: `{im}j` when the real part is
/// +0.0, else `(re±imj)`. NaN components render lowercase `nan` (Rust's `{}`
/// gives `NaN`); inf/-inf and integer-valued floats already match via `{}`.
pub fn complex_repr_string(re: f64, im: f64) -> String {
    fn part(f: f64) -> String {
        if f.is_nan() { "nan".to_string() } else { format!("{f}") }
    }
    if re == 0.0 && re.is_sign_positive() {
        format!("{}j", part(im))
    } else if im >= 0.0 {
        format!("({}+{}j)", part(re), part(im))
    } else {
        format!("({}{}j)", part(re), part(im))
    }
}

/// Parse a C99 hexadecimal floating-point string (CPython `float.fromhex`):
/// optional sign, optional `0x`/`0X` prefix, hex mantissa with an optional
/// fractional `.`, optional binary exponent `p±ddd` (decimal); plus the
/// case-insensitive specials `inf`/`infinity`/`nan`. Returns `None` for any
/// malformed input so the caller can raise ValueError.
fn parse_hex_float(input: &str) -> Option<f64> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }
    let (sign, rest) = match s.as_bytes()[0] {
        b'+' => (1.0_f64, &s[1..]),
        b'-' => (-1.0_f64, &s[1..]),
        _ => (1.0_f64, s),
    };
    let lower = rest.to_ascii_lowercase();
    match lower.as_str() {
        "inf" | "infinity" => return Some(sign * f64::INFINITY),
        "nan" => return Some(f64::NAN),
        _ => {}
    }
    let body = lower.strip_prefix("0x").unwrap_or(&lower);
    // Split off the optional binary exponent.
    let (mantissa, exp) = match body.split_once('p') {
        Some((m, e)) => {
            // Exponent must be a (possibly signed) decimal integer.
            if e.is_empty() {
                return None;
            }
            (m, e.parse::<i32>().ok()?)
        }
        None => (body, 0),
    };
    let (int_part, frac_part) = match mantissa.split_once('.') {
        Some((i, f)) => (i, f),
        None => (mantissa, ""),
    };
    if int_part.is_empty() && frac_part.is_empty() {
        return None;
    }
    let mut value = 0.0_f64;
    for c in int_part.chars() {
        value = value * 16.0 + c.to_digit(16)? as f64;
    }
    let mut scale = 1.0_f64 / 16.0;
    for c in frac_part.chars() {
        value += c.to_digit(16)? as f64 * scale;
        scale /= 16.0;
    }
    Some(sign * value * 2f64.powi(exp))
}

/// Dispatch a method call on a string object.
/// `name` is the method name, `receiver` is the string, `args` is a list of arguments.
pub fn dispatch_str_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    return items.get(i).copied().unwrap_or(MbValue::none());
                }
            }
            MbValue::none()
        }
    };
    let argc = || -> usize {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
            0
        }
    };
    match name {
        // Membership dunder: `s.__contains__(sub)` mirrors the `sub in s`
        // operator. CPython requires exactly one argument, so the no-arg form
        // raises TypeError rather than silently returning False.
        "__contains__" => {
            if argc() == 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "__contains__() takes exactly one argument (0 given)".to_string(),
                    )),
                );
                return MbValue::none();
            }
            mb_str_contains(receiver, arg(0))
        }
        // Iterator dunder: `s.__iter__()` yields the same character iterator as
        // `iter(s)` / a `for` loop, walkable by `next()`.
        "__iter__" => super::iter::mb_iter(receiver),
        // Case methods
        "upper" => mb_str_upper(receiver),
        "lower" => mb_str_lower(receiver),
        "casefold" => mb_str_casefold(receiver),
        "capitalize" => mb_str_capitalize(receiver),
        "title" => mb_str_title(receiver),
        "swapcase" => mb_str_swapcase(receiver),
        // Strip methods
        "strip" => {
            let chars = if argc() > 0 { arg(0) } else { MbValue::none() };
            mb_str_strip(receiver, chars)
        }
        "lstrip" => {
            let chars = if argc() > 0 { arg(0) } else { MbValue::none() };
            mb_str_lstrip(receiver, chars)
        }
        "rstrip" => {
            let chars = if argc() > 0 { arg(0) } else { MbValue::none() };
            mb_str_rstrip(receiver, chars)
        }
        // Search methods
        "find" => {
            let sub = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_find(receiver, sub, start, end)
        }
        "rfind" => {
            let sub = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_rfind(receiver, sub, start, end)
        }
        "index" => {
            let sub = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_index(receiver, sub, start, end)
        }
        "rindex" => {
            let sub = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_rindex(receiver, sub, start, end)
        }
        "count" => {
            let sub = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_count(receiver, sub, start, end)
        }
        "startswith" => {
            let pfx = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_startswith(receiver, pfx, start, end)
        }
        "endswith" => {
            let sfx = arg(0);
            let start = if argc() > 1 { arg(1) } else { MbValue::none() };
            let end = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_endswith(receiver, sfx, start, end)
        }
        // Modification methods
        "replace" => {
            let count = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_str_replace(receiver, arg(0), arg(1), count)
        }
        "split" => {
            let sep = if argc() > 0 { arg(0) } else { MbValue::none() };
            let maxsplit = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_str_split(receiver, sep, maxsplit)
        }
        "rsplit" => {
            let sep = if argc() > 0 { arg(0) } else { MbValue::none() };
            let maxsplit = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_str_rsplit(receiver, sep, maxsplit)
        }
        "join" => mb_str_join(receiver, arg(0)),
        // Predicate methods
        "isdigit" => mb_str_isdigit(receiver),
        "isalpha" => mb_str_isalpha(receiver),
        "isalnum" => mb_str_isalnum(receiver),
        "isspace" => mb_str_isspace(receiver),
        "isupper" => mb_str_isupper(receiver),
        "islower" => mb_str_islower(receiver),
        "istitle" => mb_str_istitle(receiver),
        "isascii" => mb_str_isascii(receiver),
        "isidentifier" => mb_str_isidentifier(receiver),
        "isnumeric" => mb_str_isnumeric(receiver),
        "isdecimal" => mb_str_isdecimal(receiver),
        "isprintable" => mb_str_isprintable(receiver),
        // Padding methods
        "center" => mb_str_center(
            receiver,
            arg(0),
            if argc() > 1 { arg(1) } else { MbValue::none() },
        ),
        "ljust" => mb_str_ljust(
            receiver,
            arg(0),
            if argc() > 1 { arg(1) } else { MbValue::none() },
        ),
        "rjust" => mb_str_rjust(
            receiver,
            arg(0),
            if argc() > 1 { arg(1) } else { MbValue::none() },
        ),
        "zfill" => mb_str_zfill(receiver, arg(0)),
        // Other methods
        "encode" => {
            // `s.encode()` — default UTF-8.
            // `s.encode("ascii", "ignore")` — positional encoding + errors.
            // `s.encode("ascii", errors="replace")` — trailing kwargs dict.
            let mut encoding = MbValue::none();
            let mut errors = MbValue::none();
            // Inspect the last positional slot for a trailing kwargs dict.
            let last_is_kwargs = if argc() > 0 {
                unsafe {
                    if let Some(ptr) = arg(argc() - 1).as_ptr() {
                        matches!((*ptr).data, ObjData::Dict(_))
                    } else {
                        false
                    }
                }
            } else {
                false
            };
            let positional_count = if last_is_kwargs { argc() - 1 } else { argc() };
            if positional_count >= 1 {
                encoding = arg(0);
            }
            if positional_count >= 2 {
                errors = arg(1);
            }
            if last_is_kwargs {
                unsafe {
                    if let Some(ptr) = arg(argc() - 1).as_ptr() {
                        if let ObjData::Dict(ref lock) = (*ptr).data {
                            let guard = lock.read().unwrap();
                            if encoding.is_none() {
                                if let Some(&v) =
                                    guard.get(&super::dict_ops::DictKey::Str("encoding".into()))
                                {
                                    encoding = v;
                                }
                            }
                            if errors.is_none() {
                                if let Some(&v) =
                                    guard.get(&super::dict_ops::DictKey::Str("errors".into()))
                                {
                                    errors = v;
                                }
                            }
                        }
                    }
                }
            }
            if encoding.is_none() && errors.is_none() {
                mb_str_encode(receiver)
            } else {
                mb_str_encode_with(receiver, encoding, errors)
            }
        }
        "splitlines" => {
            // Accept either `splitlines(True)` (positional bool) or
            // `splitlines(keepends=True)` (trailing kwargs dict).
            let raw = if argc() > 0 { arg(0) } else { MbValue::none() };
            let keepends = unsafe {
                if let Some(ptr) = raw.as_ptr() {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        let guard = lock.read().unwrap();
                        guard
                            .get(&super::dict_ops::DictKey::Str("keepends".into()))
                            .copied()
                            .unwrap_or(MbValue::none())
                    } else {
                        raw
                    }
                } else {
                    raw
                }
            };
            mb_str_splitlines(receiver, keepends)
        }
        "expandtabs" => {
            mb_str_expandtabs(receiver, if argc() > 0 { arg(0) } else { MbValue::none() })
        }
        "partition" => mb_str_partition(receiver, arg(0)),
        "rpartition" => mb_str_rpartition(receiver, arg(0)),
        "removeprefix" => mb_str_removeprefix(receiver, arg(0)),
        "removesuffix" => mb_str_removesuffix(receiver, arg(0)),
        "format" => mb_str_format(receiver, args),
        "format_map" => mb_str_format_map(receiver, arg(0)),
        "translate" => mb_str_translate(receiver, arg(0)),
        "maketrans" => mb_str_maketrans(
            arg(0),
            if argc() > 1 { arg(1) } else { MbValue::none() },
            if argc() > 2 { arg(2) } else { MbValue::none() },
        ),
        // bytes.fromhex("...") / bytearray.fromhex("...") — classmethod on type string
        "fromhex" => {
            let recv_str = unsafe {
                if let Some(ptr) = receiver.as_ptr() {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        s.clone()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            };
            let hex_val = arg(0);
            let hex_s = unsafe {
                if let Some(ptr) = hex_val.as_ptr() {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        s.clone()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            };
            // float.fromhex parses a C99 hexadecimal floating-point string and
            // raises ValueError on malformed input — distinct from the
            // bytes/bytearray hex-pair decoder below.
            if recv_str == "float" {
                return match parse_hex_float(&hex_s) {
                    Some(f) => MbValue::from_float(f),
                    None => {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "invalid hexadecimal floating-point string".to_string(),
                            )),
                        );
                        MbValue::none()
                    }
                };
            }
            let mut bytes_data: Vec<u8> = Vec::new();
            let mut high_nibble: Option<(u8, usize)> = None;
            let char_count = hex_s.chars().count();
            for (pos, ch) in hex_s.chars().enumerate() {
                if ch.is_ascii_whitespace() {
                    continue;
                }
                let Some(nibble) = ch.to_digit(16).map(|n| n as u8) else {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "non-hexadecimal number found in fromhex() arg at position {pos}"
                        ))),
                    );
                    return MbValue::none();
                };
                if let Some((high, _)) = high_nibble.take() {
                    bytes_data.push((high << 4) | nibble);
                } else {
                    high_nibble = Some((nibble, pos));
                }
            }
            if high_nibble.is_some() {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "non-hexadecimal number found in fromhex() arg at position {char_count}"
                    ))),
                );
                return MbValue::none();
            }
            if recv_str == "bytearray" {
                MbValue::from_ptr(MbObject::new_bytearray(bytes_data))
            } else {
                MbValue::from_ptr(MbObject::new_bytes(bytes_data))
            }
        }
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'str' object has no attribute '{name}'"
                ))),
            );
            MbValue::none()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        new_str(val.to_string())
    }

    #[test]
    fn test_concat() {
        let result = mb_str_concat(s("hello"), s(" world"));
        unsafe {
            assert_eq!(as_str(result), Some("hello world"));
        }
    }

    #[test]
    fn test_repeat() {
        let result = mb_str_repeat(s("ab"), MbValue::from_int(3));
        unsafe {
            assert_eq!(as_str(result), Some("ababab"));
        }
    }

    #[test]
    fn test_upper_lower() {
        unsafe {
            assert_eq!(as_str(mb_str_upper(s("hello"))), Some("HELLO"));
            assert_eq!(as_str(mb_str_lower(s("HELLO"))), Some("hello"));
        }
    }

    #[test]
    fn test_strip() {
        unsafe {
            let n = MbValue::none();
            assert_eq!(as_str(mb_str_strip(s("  hi  "), n)), Some("hi"));
            assert_eq!(as_str(mb_str_lstrip(s("  hi  "), n)), Some("hi  "));
            assert_eq!(as_str(mb_str_rstrip(s("  hi  "), n)), Some("  hi"));
        }
    }

    #[test]
    fn test_find() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_find(s("hello world"), s("world"), n, n).as_int(),
            Some(6)
        );
        assert_eq!(mb_str_find(s("hello"), s("xyz"), n, n).as_int(), Some(-1));
    }

    #[test]
    fn test_startswith_endswith() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_startswith(s("hello"), s("hel"), n, n).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_str_endswith(s("hello"), s("llo"), n, n).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_replace() {
        unsafe {
            assert_eq!(
                as_str(mb_str_replace(
                    s("hello world"),
                    s("world"),
                    s("rust"),
                    MbValue::none()
                )),
                Some("hello rust"),
            );
        }
    }

    #[test]
    fn test_split() {
        let result = mb_str_split(s("a,b,c"), s(","), MbValue::none());
        assert!(result.is_ptr());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("a"));
                assert_eq!(as_str(items[1]), Some("b"));
                assert_eq!(as_str(items[2]), Some("c"));
            }
            super::super::rc::mb_release(ptr);
        }
    }

    #[test]
    fn test_getitem() {
        unsafe {
            assert_eq!(
                as_str(mb_str_getitem(s("hello"), MbValue::from_int(0))),
                Some("h")
            );
            assert_eq!(
                as_str(mb_str_getitem(s("hello"), MbValue::from_int(-1))),
                Some("o")
            );
        }
    }

    #[test]
    fn test_splitlines() {
        let result = mb_str_splitlines(s("hello\nworld\nfoo"), MbValue::none());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("hello"));
                assert_eq!(as_str(items[2]), Some("foo"));
            }
        }
    }

    #[test]
    fn test_partition() {
        let result = mb_str_partition(s("hello-world-foo"), s("-"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("hello"));
                assert_eq!(as_str(items[1]), Some("-"));
                assert_eq!(as_str(items[2]), Some("world-foo"));
            }
        }
    }

    #[test]
    fn test_format_spec() {
        unsafe {
            // Integer formatting
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s("05d"))),
                Some("00042")
            );
            // Float formatting
            assert_eq!(
                as_str(mb_format_value(MbValue::from_float(3.14159), s(".2f"))),
                Some("3.14")
            );
            // Hex formatting
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(255), s("x"))),
                Some("ff")
            );
            // Alignment
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s("<5d"))),
                Some("42   ")
            );
        }
    }

    #[test]
    fn test_predicates() {
        assert_eq!(mb_str_isdigit(s("123")).as_bool(), Some(true));
        assert_eq!(mb_str_isdigit(s("12a")).as_bool(), Some(false));
        assert_eq!(mb_str_isalpha(s("abc")).as_bool(), Some(true));
        assert_eq!(mb_str_isspace(s("  \t")).as_bool(), Some(true));
    }

    #[test]
    fn test_removeprefix_removesuffix() {
        unsafe {
            assert_eq!(
                as_str(mb_str_removeprefix(s("hello_world"), s("hello_"))),
                Some("world")
            );
            assert_eq!(
                as_str(mb_str_removeprefix(s("hello"), s("xyz"))),
                Some("hello")
            );
            assert_eq!(
                as_str(mb_str_removesuffix(s("file.py"), s(".py"))),
                Some("file")
            );
            assert_eq!(
                as_str(mb_str_removesuffix(s("file.py"), s(".rs"))),
                Some("file.py")
            );
        }
    }

    #[test]
    fn test_rpartition() {
        let result = mb_str_rpartition(s("a-b-c"), s("-"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("a-b"));
                assert_eq!(as_str(items[1]), Some("-"));
                assert_eq!(as_str(items[2]), Some("c"));
            }
        }
    }

    // ── rfind ──

    #[test]
    fn test_rfind_found() {
        let n = MbValue::none();
        assert_eq!(mb_str_rfind(s("abcabc"), s("bc"), n, n).as_int(), Some(4));
    }

    #[test]
    fn test_rfind_not_found() {
        let n = MbValue::none();
        assert_eq!(mb_str_rfind(s("hello"), s("xyz"), n, n).as_int(), Some(-1));
    }

    #[test]
    fn test_rfind_empty_sub() {
        let n = MbValue::none();
        // Python: "hello".rfind("") == 5 (len of string)
        assert_eq!(mb_str_rfind(s("hello"), s(""), n, n).as_int(), Some(5));
    }

    // ── count ──

    #[test]
    fn test_count_multiple() {
        let n = MbValue::none();
        assert_eq!(mb_str_count(s("banana"), s("an"), n, n).as_int(), Some(2));
    }

    #[test]
    fn test_count_none() {
        let n = MbValue::none();
        assert_eq!(mb_str_count(s("hello"), s("xyz"), n, n).as_int(), Some(0));
    }

    #[test]
    fn test_count_empty_string() {
        let n = MbValue::none();
        assert_eq!(mb_str_count(s(""), s("a"), n, n).as_int(), Some(0));
    }

    // ── contains ──

    #[test]
    fn test_contains_true() {
        assert_eq!(
            mb_str_contains(s("hello world"), s("world")).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_contains_false() {
        assert_eq!(mb_str_contains(s("hello"), s("xyz")).as_bool(), Some(false));
    }

    #[test]
    fn test_contains_empty_sub() {
        assert_eq!(mb_str_contains(s("hello"), s("")).as_bool(), Some(true));
    }

    // ── join ──

    #[test]
    fn test_join() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b"), s("c")]));
        let result = mb_str_join(s(", "), items);
        unsafe {
            assert_eq!(as_str(result), Some("a, b, c"));
        }
    }

    #[test]
    fn test_join_empty_sep() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![s("x"), s("y")]));
        let result = mb_str_join(s(""), items);
        unsafe {
            assert_eq!(as_str(result), Some("xy"));
        }
    }

    #[test]
    fn test_join_single_item() {
        let items = MbValue::from_ptr(MbObject::new_list(vec![s("only")]));
        let result = mb_str_join(s("-"), items);
        unsafe {
            assert_eq!(as_str(result), Some("only"));
        }
    }

    // ── capitalize ──

    #[test]
    fn test_capitalize() {
        unsafe {
            assert_eq!(
                as_str(mb_str_capitalize(s("hello world"))),
                Some("Hello world")
            );
        }
    }

    #[test]
    fn test_capitalize_already_upper() {
        unsafe {
            assert_eq!(as_str(mb_str_capitalize(s("HELLO"))), Some("Hello"));
        }
    }

    #[test]
    fn test_capitalize_empty() {
        unsafe {
            assert_eq!(as_str(mb_str_capitalize(s(""))), Some(""));
        }
    }

    // ── title ──

    #[test]
    fn test_title() {
        unsafe {
            assert_eq!(as_str(mb_str_title(s("hello world"))), Some("Hello World"));
        }
    }

    #[test]
    fn test_title_mixed() {
        unsafe {
            assert_eq!(as_str(mb_str_title(s("hELLO wORLD"))), Some("Hello World"));
        }
    }

    #[test]
    fn test_title_with_punctuation() {
        unsafe {
            assert_eq!(as_str(mb_str_title(s("it's a test"))), Some("It'S A Test"));
        }
    }

    // ── swapcase ──

    #[test]
    fn test_swapcase() {
        unsafe {
            assert_eq!(
                as_str(mb_str_swapcase(s("Hello World"))),
                Some("hELLO wORLD")
            );
        }
    }

    #[test]
    fn test_swapcase_all_lower() {
        unsafe {
            assert_eq!(as_str(mb_str_swapcase(s("abc"))), Some("ABC"));
        }
    }

    // ── isalnum ──

    #[test]
    fn test_isalnum_true() {
        assert_eq!(mb_str_isalnum(s("abc123")).as_bool(), Some(true));
    }

    #[test]
    fn test_isalnum_false() {
        assert_eq!(mb_str_isalnum(s("abc 123")).as_bool(), Some(false));
    }

    #[test]
    fn test_isalnum_empty() {
        assert_eq!(mb_str_isalnum(s("")).as_bool(), Some(false));
    }

    // ── isupper / islower ──

    #[test]
    fn test_isupper_true() {
        assert_eq!(mb_str_isupper(s("HELLO")).as_bool(), Some(true));
    }

    #[test]
    fn test_isupper_false() {
        assert_eq!(mb_str_isupper(s("Hello")).as_bool(), Some(false));
    }

    #[test]
    fn test_isupper_with_digits() {
        // "HELLO123" — has uppercase, no lowercase → true (Python behavior)
        assert_eq!(mb_str_isupper(s("HELLO123")).as_bool(), Some(true));
    }

    #[test]
    fn test_islower_true() {
        assert_eq!(mb_str_islower(s("hello")).as_bool(), Some(true));
    }

    #[test]
    fn test_islower_false() {
        assert_eq!(mb_str_islower(s("Hello")).as_bool(), Some(false));
    }

    #[test]
    fn test_islower_with_digits() {
        assert_eq!(mb_str_islower(s("hello123")).as_bool(), Some(true));
    }

    // ── center ──

    #[test]
    fn test_center_default_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_center(
                    s("hi"),
                    MbValue::from_int(6),
                    MbValue::none()
                )),
                Some("  hi  ")
            );
        }
    }

    #[test]
    fn test_center_custom_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_center(s("hi"), MbValue::from_int(6), s("*"))),
                Some("**hi**")
            );
        }
    }

    #[test]
    fn test_center_no_padding_needed() {
        unsafe {
            assert_eq!(
                as_str(mb_str_center(
                    s("hello"),
                    MbValue::from_int(3),
                    MbValue::none()
                )),
                Some("hello")
            );
        }
    }

    // ── ljust ──

    #[test]
    fn test_ljust_default_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_ljust(s("hi"), MbValue::from_int(5), MbValue::none())),
                Some("hi   ")
            );
        }
    }

    #[test]
    fn test_ljust_custom_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_ljust(s("hi"), MbValue::from_int(5), s("-"))),
                Some("hi---")
            );
        }
    }

    #[test]
    fn test_ljust_no_padding_needed() {
        unsafe {
            assert_eq!(
                as_str(mb_str_ljust(
                    s("hello"),
                    MbValue::from_int(3),
                    MbValue::none()
                )),
                Some("hello")
            );
        }
    }

    // ── rjust ──

    #[test]
    fn test_rjust_default_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_rjust(s("hi"), MbValue::from_int(5), MbValue::none())),
                Some("   hi")
            );
        }
    }

    #[test]
    fn test_rjust_custom_fill() {
        unsafe {
            assert_eq!(
                as_str(mb_str_rjust(s("hi"), MbValue::from_int(5), s("0"))),
                Some("000hi")
            );
        }
    }

    // ── zfill ──

    #[test]
    fn test_zfill_positive() {
        unsafe {
            assert_eq!(
                as_str(mb_str_zfill(s("42"), MbValue::from_int(5))),
                Some("00042")
            );
        }
    }

    #[test]
    fn test_zfill_negative() {
        unsafe {
            assert_eq!(
                as_str(mb_str_zfill(s("-42"), MbValue::from_int(5))),
                Some("-0042")
            );
        }
    }

    #[test]
    fn test_zfill_no_padding_needed() {
        unsafe {
            assert_eq!(
                as_str(mb_str_zfill(s("12345"), MbValue::from_int(3))),
                Some("12345")
            );
        }
    }

    #[test]
    fn test_zfill_with_plus() {
        unsafe {
            assert_eq!(
                as_str(mb_str_zfill(s("+5"), MbValue::from_int(5))),
                Some("+0005")
            );
        }
    }

    // ── encode ──

    #[test]
    fn test_encode_ascii() {
        let result = mb_str_encode(s("hi"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Bytes(ref data) = (*ptr).data {
                assert_eq!(data.as_slice(), b"hi");
            } else {
                panic!("expected bytes");
            }
        }
    }

    #[test]
    fn test_encode_empty() {
        let result = mb_str_encode(s(""));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Bytes(ref data) = (*ptr).data {
                assert_eq!(data.len(), 0);
            } else {
                panic!("expected bytes");
            }
        }
    }

    // ── hash ──

    #[test]
    fn test_hash_string() {
        let result = mb_str_hash(s("hello"));
        assert!(result.as_int().is_some());
    }

    #[test]
    fn test_hash_same_string_same_result() {
        let a = mb_str_hash(s("test"));
        let b = mb_str_hash(s("test"));
        assert_eq!(a.as_int(), b.as_int());
    }

    #[test]
    fn test_hash_different_strings() {
        let a = mb_str_hash(s("foo"));
        let b = mb_str_hash(s("bar"));
        // Different strings should (usually) have different hashes
        assert_ne!(a.as_int(), b.as_int());
    }

    #[test]
    fn test_hash_non_string_returns_zero() {
        let result = mb_str_hash(MbValue::from_int(42));
        assert_eq!(result.as_int(), Some(0));
    }

    // ── eq ──

    #[test]
    fn test_eq_true() {
        assert_eq!(mb_str_eq(s("hello"), s("hello")).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_false() {
        assert_eq!(mb_str_eq(s("hello"), s("world")).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_empty() {
        assert_eq!(mb_str_eq(s(""), s("")).as_bool(), Some(true));
    }

    // ── lt ──

    #[test]
    fn test_lt_true() {
        assert_eq!(mb_str_lt(s("abc"), s("abd")).as_bool(), Some(true));
    }

    #[test]
    fn test_lt_false() {
        assert_eq!(mb_str_lt(s("xyz"), s("abc")).as_bool(), Some(false));
    }

    #[test]
    fn test_lt_equal_is_false() {
        assert_eq!(mb_str_lt(s("abc"), s("abc")).as_bool(), Some(false));
    }

    // ── slice ──

    #[test]
    fn test_slice_basic() {
        unsafe {
            assert_eq!(
                as_str(mb_str_slice(
                    s("hello"),
                    MbValue::from_int(1),
                    MbValue::from_int(4)
                )),
                Some("ell"),
            );
        }
    }

    #[test]
    fn test_slice_from_start() {
        unsafe {
            assert_eq!(
                as_str(mb_str_slice(
                    s("hello"),
                    MbValue::from_int(0),
                    MbValue::from_int(2)
                )),
                Some("he"),
            );
        }
    }

    #[test]
    fn test_slice_negative_start() {
        unsafe {
            assert_eq!(
                as_str(mb_str_slice(
                    s("hello"),
                    MbValue::from_int(-3),
                    MbValue::from_int(5)
                )),
                Some("llo"),
            );
        }
    }

    #[test]
    fn test_slice_empty_result() {
        unsafe {
            assert_eq!(
                as_str(mb_str_slice(
                    s("hello"),
                    MbValue::from_int(3),
                    MbValue::from_int(1)
                )),
                Some(""),
            );
        }
    }

    // ── format ──

    #[test]
    fn test_format_basic() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("world")]));
        let result = mb_str_format(s("hello {}"), args);
        unsafe {
            assert_eq!(as_str(result), Some("hello world"));
        }
    }

    #[test]
    fn test_format_multiple_args() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("a"), s("b")]));
        let result = mb_str_format(s("{} and {}"), args);
        unsafe {
            assert_eq!(as_str(result), Some("a and b"));
        }
    }

    #[test]
    fn test_format_int_arg() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(42)]));
        let result = mb_str_format(s("value: {}"), args);
        unsafe {
            assert_eq!(as_str(result), Some("value: 42"));
        }
    }

    // ── value_to_string ──

    #[test]
    fn test_value_to_string_int() {
        assert_eq!(value_to_string(MbValue::from_int(42)), "42");
    }

    #[test]
    fn test_value_to_string_float() {
        assert_eq!(value_to_string(MbValue::from_float(3.14)), "3.14");
    }

    #[test]
    fn test_value_to_string_float_whole() {
        assert_eq!(value_to_string(MbValue::from_float(5.0)), "5.0");
    }

    #[test]
    fn test_value_to_string_bool() {
        assert_eq!(value_to_string(MbValue::from_bool(true)), "True");
        assert_eq!(value_to_string(MbValue::from_bool(false)), "False");
    }

    #[test]
    fn test_value_to_string_none() {
        assert_eq!(value_to_string(MbValue::none()), "None");
    }

    #[test]
    fn test_value_to_string_str() {
        assert_eq!(value_to_string(s("hello")), "hello");
    }

    // ── format_value additional specs ──

    #[test]
    fn test_format_value_right_align() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s(">6d"))),
                Some("    42")
            );
        }
    }

    #[test]
    fn test_format_value_center_align() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s("^6d"))),
                Some("  42  ")
            );
        }
    }

    #[test]
    fn test_format_value_fill_align() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s("*>5d"))),
                Some("***42")
            );
        }
    }

    #[test]
    fn test_format_value_binary() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(10), s("b"))),
                Some("1010")
            );
        }
    }

    #[test]
    fn test_format_value_octal() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(8), s("o"))),
                Some("10")
            );
        }
    }

    #[test]
    fn test_format_value_upper_hex() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(255), s("X"))),
                Some("FF")
            );
        }
    }

    #[test]
    fn test_format_value_string_precision() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(s("hello world"), s(".5s"))),
                Some("hello")
            );
        }
    }

    #[test]
    fn test_format_value_empty_spec() {
        unsafe {
            assert_eq!(
                as_str(mb_format_value(MbValue::from_int(42), s(""))),
                Some("42")
            );
        }
    }

    #[test]
    fn test_format_value_scientific() {
        unsafe {
            let result = as_str(mb_format_value(MbValue::from_float(1234.5), s(".2e")));
            assert_eq!(result, Some("1.23e+03"));
        }
    }

    // ── repeat edge cases ──

    #[test]
    fn test_repeat_zero() {
        unsafe {
            assert_eq!(
                as_str(mb_str_repeat(s("ab"), MbValue::from_int(0))),
                Some("")
            );
        }
    }

    #[test]
    fn test_repeat_negative() {
        unsafe {
            assert_eq!(
                as_str(mb_str_repeat(s("ab"), MbValue::from_int(-1))),
                Some("")
            );
        }
    }

    // ── getitem edge cases ──

    #[test]
    fn test_getitem_out_of_bounds() {
        let result = mb_str_getitem(s("hi"), MbValue::from_int(10));
        assert!(result.is_none());
    }

    #[test]
    fn test_getitem_negative_out_of_bounds() {
        let result = mb_str_getitem(s("hi"), MbValue::from_int(-10));
        assert!(result.is_none());
    }

    // ── split edge cases ──

    #[test]
    fn test_split_whitespace() {
        let result = mb_str_split(s("  hello  world  "), MbValue::none(), MbValue::none());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                assert_eq!(as_str(items[0]), Some("hello"));
                assert_eq!(as_str(items[1]), Some("world"));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_split_empty_result_parts() {
        let result = mb_str_split(s("a,,b"), s(","), MbValue::none());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("a"));
                assert_eq!(as_str(items[1]), Some(""));
                assert_eq!(as_str(items[2]), Some("b"));
            } else {
                panic!("expected list");
            }
        }
    }

    // ── partition edge cases ──

    #[test]
    fn test_partition_not_found() {
        let result = mb_str_partition(s("hello"), s("xyz"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("hello"));
                assert_eq!(as_str(items[1]), Some(""));
                assert_eq!(as_str(items[2]), Some(""));
            }
        }
    }

    #[test]
    fn test_rpartition_not_found() {
        let result = mb_str_rpartition(s("hello"), s("xyz"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some(""));
                assert_eq!(as_str(items[1]), Some(""));
                assert_eq!(as_str(items[2]), Some("hello"));
            }
        }
    }

    // ── predicate edge cases ──

    #[test]
    fn test_isdigit_empty() {
        assert_eq!(mb_str_isdigit(s("")).as_bool(), Some(false));
    }

    #[test]
    fn test_isalpha_empty() {
        assert_eq!(mb_str_isalpha(s("")).as_bool(), Some(false));
    }

    #[test]
    fn test_isalpha_with_space() {
        assert_eq!(mb_str_isalpha(s("hello world")).as_bool(), Some(false));
    }

    #[test]
    fn test_isspace_empty() {
        assert_eq!(mb_str_isspace(s("")).as_bool(), Some(false));
    }

    #[test]
    fn test_isspace_non_space() {
        assert_eq!(mb_str_isspace(s("abc")).as_bool(), Some(false));
    }

    // ── concat edge cases ──

    #[test]
    fn test_concat_empty() {
        unsafe {
            assert_eq!(as_str(mb_str_concat(s("hello"), s(""))), Some("hello"));
            assert_eq!(as_str(mb_str_concat(s(""), s("world"))), Some("world"));
            assert_eq!(as_str(mb_str_concat(s(""), s(""))), Some(""));
        }
    }

    // ── find edge cases ──

    #[test]
    fn test_find_empty_sub() {
        let n = MbValue::none();
        assert_eq!(mb_str_find(s("hello"), s(""), n, n).as_int(), Some(0));
    }

    #[test]
    fn test_find_in_empty_string() {
        let n = MbValue::none();
        assert_eq!(mb_str_find(s(""), s("a"), n, n).as_int(), Some(-1));
    }

    // ── startswith/endswith edge cases ──

    #[test]
    fn test_startswith_false() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_startswith(s("hello"), s("xyz"), n, n).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_endswith_false() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_endswith(s("hello"), s("xyz"), n, n).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_startswith_empty_prefix() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_startswith(s("hello"), s(""), n, n).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_endswith_empty_suffix() {
        let n = MbValue::none();
        assert_eq!(
            mb_str_endswith(s("hello"), s(""), n, n).as_bool(),
            Some(true)
        );
    }

    // ── splitlines edge cases ──

    #[test]
    fn test_splitlines_empty() {
        let result = mb_str_splitlines(s(""), MbValue::none());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 0);
            }
        }
    }

    // ── dispatch_str_method ──

    #[test]
    fn test_dispatch_upper() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("upper", s("hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("HELLO"));
        }
    }

    #[test]
    fn test_dispatch_lower() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("lower", s("HELLO"), args);
        unsafe {
            assert_eq!(as_str(result), Some("hello"));
        }
    }

    #[test]
    fn test_dispatch_strip() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("strip", s("  hi  "), args);
        unsafe {
            assert_eq!(as_str(result), Some("hi"));
        }
    }

    #[test]
    fn test_dispatch_find() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("lo")]));
        let result = dispatch_str_method("find", s("hello"), args);
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_dispatch_replace() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("l"), s("r")]));
        let result = dispatch_str_method("replace", s("hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("herro"));
        }
    }

    #[test]
    fn test_dispatch_split_no_args() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("split", s("a b c"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(as_str(items[0]), Some("a"));
            }
        }
    }

    #[test]
    fn test_dispatch_split_with_sep() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s(",")]));
        let result = dispatch_str_method("split", s("a,b"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
            }
        }
    }

    #[test]
    fn test_dispatch_join() {
        let items_list = MbValue::from_ptr(MbObject::new_list(vec![s("x"), s("y")]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![items_list]));
        let result = dispatch_str_method("join", s("-"), args);
        unsafe {
            assert_eq!(as_str(result), Some("x-y"));
        }
    }

    #[test]
    fn test_dispatch_startswith() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("he")]));
        let result = dispatch_str_method("startswith", s("hello"), args);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_endswith() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("lo")]));
        let result = dispatch_str_method("endswith", s("hello"), args);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_isdigit() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("isdigit", s("123"), args);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_capitalize() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("capitalize", s("hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("Hello"));
        }
    }

    #[test]
    fn test_dispatch_title() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("title", s("hello world"), args);
        unsafe {
            assert_eq!(as_str(result), Some("Hello World"));
        }
    }

    #[test]
    fn test_dispatch_swapcase() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("swapcase", s("Hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("hELLO"));
        }
    }

    #[test]
    fn test_dispatch_zfill() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(5)]));
        let result = dispatch_str_method("zfill", s("42"), args);
        unsafe {
            assert_eq!(as_str(result), Some("00042"));
        }
    }

    #[test]
    fn test_dispatch_center_default_fill() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(7)]));
        let result = dispatch_str_method("center", s("hi"), args);
        unsafe {
            let result_str = as_str(result).unwrap();
            assert_eq!(result_str.len(), 7);
            assert!(result_str.contains("hi"));
        }
    }

    #[test]
    fn test_dispatch_encode() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("encode", s("A"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Bytes(ref data) = (*ptr).data {
                assert_eq!(data.as_slice(), b"A");
            }
        }
    }

    #[test]
    fn test_dispatch_removeprefix() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("he")]));
        let result = dispatch_str_method("removeprefix", s("hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("llo"));
        }
    }

    #[test]
    fn test_dispatch_removesuffix() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("lo")]));
        let result = dispatch_str_method("removesuffix", s("hello"), args);
        unsafe {
            assert_eq!(as_str(result), Some("hel"));
        }
    }

    #[test]
    fn test_dispatch_unknown_method() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("nonexistent", s("hello"), args);
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_count() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("l")]));
        let result = dispatch_str_method("count", s("hello"), args);
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_dispatch_rfind() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("l")]));
        let result = dispatch_str_method("rfind", s("hello"), args);
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_dispatch_partition() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("-")]));
        let result = dispatch_str_method("partition", s("a-b"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(as_str(items[0]), Some("a"));
                assert_eq!(as_str(items[1]), Some("-"));
                assert_eq!(as_str(items[2]), Some("b"));
            }
        }
    }

    #[test]
    fn test_dispatch_rpartition() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("-")]));
        let result = dispatch_str_method("rpartition", s("a-b-c"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(as_str(items[0]), Some("a-b"));
                assert_eq!(as_str(items[1]), Some("-"));
                assert_eq!(as_str(items[2]), Some("c"));
            }
        }
    }

    #[test]
    fn test_dispatch_splitlines() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("splitlines", s("a\nb"), args);
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
            }
        }
    }

    #[test]
    fn test_dispatch_isalpha() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("isalpha", s("abc"), args).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_isalnum() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("isalnum", s("abc123"), args).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_isspace() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("isspace", s("  "), args).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_isupper() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("isupper", s("ABC"), args).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_islower() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("islower", s("abc"), args).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dispatch_lstrip() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("lstrip", s("  hi  "), args);
        unsafe {
            assert_eq!(as_str(result), Some("hi  "));
        }
    }

    #[test]
    fn test_dispatch_rstrip() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = dispatch_str_method("rstrip", s("  hi  "), args);
        unsafe {
            assert_eq!(as_str(result), Some("  hi"));
        }
    }

    #[test]
    fn test_dispatch_ljust() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(5)]));
        let result = dispatch_str_method("ljust", s("hi"), args);
        unsafe {
            assert_eq!(as_str(result), Some("hi   "));
        }
    }

    #[test]
    fn test_dispatch_rjust() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(5)]));
        let result = dispatch_str_method("rjust", s("hi"), args);
        unsafe {
            assert_eq!(as_str(result), Some("   hi"));
        }
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_str_removeprefix() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("hello ")]));
        let result = dispatch_str_method("removeprefix", s("hello world"), args);
        unsafe { assert_eq!(as_str(result), Some("world")) };
    }

    #[test]
    fn test_py312_str_removesuffix() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s(" world")]));
        let result = dispatch_str_method("removesuffix", s("hello world"), args);
        unsafe { assert_eq!(as_str(result), Some("hello")) };
    }

    #[test]
    fn test_py312_str_removeprefix_no_match() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("xyz")]));
        let result = dispatch_str_method("removeprefix", s("hello"), args);
        unsafe { assert_eq!(as_str(result), Some("hello")) };
    }

    #[test]
    fn test_py312_str_zfill_sign_preserved() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(5)]));
        let result = dispatch_str_method("zfill", s("-42"), args);
        unsafe { assert_eq!(as_str(result), Some("-0042")) };
    }

    #[test]
    fn test_py312_str_encode_utf8() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![s("utf-8")]));
        let result = dispatch_str_method("encode", s("hello"), args);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_py312_str_istitle() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("istitle", s("Hello World"), args).as_bool(),
            Some(true)
        );
        let args2 = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(
            dispatch_str_method("istitle", s("hello world"), args2).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_py312_str_split_maxsplit() {
        let sep = s(",");
        let args = MbValue::from_ptr(MbObject::new_list(vec![sep, MbValue::from_int(1)]));
        let result = dispatch_str_method("split", s("a,b,c"), args);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_py312_str_center_with_fill() {
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(11), s("*")]));
        let result = dispatch_str_method("center", s("hello"), args);
        unsafe { assert_eq!(as_str(result), Some("***hello***")) };
    }

    // ── String reverse slice tests (string-reverse-slice-fix) ──

    /// S1: Full reverse slice s[::-1] produces reversed string (R1, R2, R6)
    #[test]
    fn test_str_slice_full_reverse() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(), // start absent
            MbValue::none(), // stop absent
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("fedcba"));
        }
    }

    /// S2: Full reverse of single-char string (R1, R2, R6)
    #[test]
    fn test_str_slice_full_reverse_single_char() {
        let result = mb_str_slice_full(
            s("x"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("x"));
        }
    }

    /// S3: Full reverse of empty string (R1, R2, R6)
    #[test]
    fn test_str_slice_full_reverse_empty() {
        let result = mb_str_slice_full(
            s(""),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some(""));
        }
    }

    /// S4: Partial reverse with explicit negative start (R3)
    /// 'abcdef'[-2::-1] → 'edcba'
    #[test]
    fn test_str_slice_partial_reverse_explicit_start() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::from_int(-2), // explicit start → goes through clamp_rev_str
            MbValue::none(),       // stop absent → literal -1
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("edcba"));
        }
    }

    /// S5: Partial reverse with explicit start and stop (R4)
    /// 'abcdef'[4:1:-1] → 'edc'
    #[test]
    fn test_str_slice_partial_reverse_explicit_start_stop() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::from_int(4), // explicit start
            MbValue::from_int(1), // explicit stop
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("edc"));
        }
    }

    /// S6: Positive step slicing unaffected (R5)
    /// 'abcdef'[1:4] → 'bcd'
    #[test]
    fn test_str_slice_positive_step_unaffected() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::from_int(1),
            MbValue::from_int(4),
            MbValue::from_int(1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("bcd"));
        }
    }

    /// S7: Step=-2 skipping reverse (R1, R2)
    /// 'abcdef'[::-2] → 'fdb'
    #[test]
    fn test_str_slice_reverse_step_minus_2() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(-2),
        );
        unsafe {
            assert_eq!(as_str(result), Some("fdb"));
        }
    }

    /// S8: Unicode string reverse (R6)
    /// '你好世界'[::-1] → '界世好你'
    #[test]
    fn test_str_slice_full_reverse_unicode() {
        let result = mb_str_slice_full(
            s("你好世界"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("界世好你"));
        }
    }

    /// S9: step=0 returns empty string
    #[test]
    fn test_str_slice_step_zero_returns_empty() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(0),
        );
        unsafe {
            assert_eq!(as_str(result), Some(""));
        }
    }

    /// S10: Reverse with explicit start=0, absent stop → 'abcdef'[0::-1] → 'a'
    /// Exercises: explicit start goes through clamp_rev_str, absent stop uses literal -1
    #[test]
    fn test_str_slice_reverse_explicit_start_zero() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::from_int(0), // explicit start → clamp_rev_str(0, 6) → 0
            MbValue::none(),      // absent stop → literal -1
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("a"));
        }
    }

    /// S11: Reverse with absent start, explicit stop=0 → 'abcdef'[:0:-1] → 'fedcb'
    /// Exercises: absent start uses literal len-1, explicit stop goes through clamp_rev_str
    #[test]
    fn test_str_slice_reverse_explicit_stop_zero() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(),      // absent start → literal len-1 = 5
            MbValue::from_int(0), // explicit stop → clamp_rev_str(0, 6) → 0
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("fedcb"));
        }
    }

    /// S12: Forward with absent start/stop, step=1 → 'abcdef'[::1] → 'abcdef'
    #[test]
    fn test_str_slice_forward_absent_start_stop() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("abcdef"));
        }
    }

    /// S13: Forward with step=2, absent start/stop → 'abcdef'[::2] → 'ace'
    #[test]
    fn test_str_slice_forward_step_two() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::none(),
            MbValue::none(),
            MbValue::from_int(2),
        );
        unsafe {
            assert_eq!(as_str(result), Some("ace"));
        }
    }

    /// S14: Reverse with explicit negative stop → 'abcdef'[5:-4:-1] → 'fed'
    /// start=5 (f), stop=-4 → clamp_rev_str(-4, 6) → 2, iterates 5→3 producing 'fed'
    #[test]
    fn test_str_slice_reverse_explicit_negative_stop() {
        let result = mb_str_slice_full(
            s("abcdef"),
            MbValue::from_int(5),
            MbValue::from_int(-4),
            MbValue::from_int(-1),
        );
        unsafe {
            assert_eq!(as_str(result), Some("fed"));
        }
    }

    // ── R8: mb_str_format_kwargs tests ──

    #[test]
    fn test_format_kwargs_single() {
        use super::super::dict_ops::mb_dict_setitem;
        let template = s("{name}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        mb_dict_setitem(kwargs, s("name"), s("world"));
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some("world"));
        }
    }

    #[test]
    fn test_format_kwargs_multiple() {
        use super::super::dict_ops::mb_dict_setitem;
        let template = s("{name} is {age}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        mb_dict_setitem(kwargs, s("name"), s("Alice"));
        mb_dict_setitem(kwargs, s("age"), MbValue::from_int(30));
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some("Alice is 30"));
        }
    }

    #[test]
    fn test_format_kwargs_with_positional() {
        use super::super::dict_ops::mb_dict_setitem;
        let template = s("{} says {greeting}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![s("Bob")]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        mb_dict_setitem(kwargs, s("greeting"), s("hello"));
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some("Bob says hello"));
        }
    }

    #[test]
    fn test_format_kwargs_indexed_positional() {
        let template = s("{0} and {1}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![s("x"), s("y")]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some("x and y"));
        }
    }

    #[test]
    fn test_format_kwargs_escaped_braces() {
        let template = s("{{literal}} {name}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        super::super::dict_ops::mb_dict_setitem(kwargs, s("name"), s("test"));
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some("{literal} test"));
        }
    }

    #[test]
    fn test_format_kwargs_missing_key() {
        let template = s("{missing}");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        // Unknown key preserved as-is
        unsafe {
            assert_eq!(as_str(result), Some("{missing}"));
        }
    }

    #[test]
    fn test_format_kwargs_empty_template() {
        let template = s("");
        let pos_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let kwargs = MbValue::from_ptr(MbObject::new_dict());
        let result = mb_str_format_kwargs(template, pos_args, kwargs);
        unsafe {
            assert_eq!(as_str(result), Some(""));
        }
    }

    #[test]
    fn test_istitle_true() {
        assert_eq!(mb_str_istitle(s("Hello World")).as_bool(), Some(true));
    }

    #[test]
    fn test_istitle_false_all_lower() {
        assert_eq!(mb_str_istitle(s("hello world")).as_bool(), Some(false));
    }

    #[test]
    fn test_istitle_empty() {
        assert_eq!(mb_str_istitle(s("")).as_bool(), Some(false));
    }
}
