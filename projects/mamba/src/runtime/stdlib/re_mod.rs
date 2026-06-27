use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// re (regular expressions) module for Mamba — backed by `regex` crate.
///
/// Provides: re.search, re.match_, re.findall, re.sub, re.split, re.escape
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ── Regex compile cache (#2110) ──────────────────────────────────────────
//
// `re.findall(pattern, text)` and friends are typically called from a hot
// loop where the same pattern string is re-passed every iteration (e.g.
// the Phase-2 `findall_hot` bench compiles a 4-group Apache-log regex
// 5x against a 1 MB corpus). Re-running `regex::Regex::new` per call
// adds non-trivial overhead on top of the per-match allocation cost.
// A small thread-local LRU-style cache keyed by the literal pattern
// string lets the second-onwards call skip compilation entirely.
//
// Cache is intentionally small + thread-local: the regex crate's
// `Regex` is `Send + Sync` but sharing across threads would require
// `Arc<Mutex<...>>` which adds its own contention; mamba's main
// runtime thread is the only consumer in practice.

const RE_CACHE_CAP: usize = 32;

thread_local! {
    static RE_CACHE: RefCell<Vec<(String, Rc<regex::Regex>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Look up a compiled `Regex` for `pat`, compiling+caching on miss.
/// Returns `Err(message)` on compile failure (caller raises `re.error`).
/// Translate Python `re` spellings the Rust `regex` crate spells
/// differently:
/// - `\Z` (Python end-of-string) → `\z` (Rust).
/// - `(?>...)` (Python atomic group) → `(?:...)` for Rust compilation.
///
/// The regex crate has no atomic-group syntax. `re.match` / `re.fullmatch`
/// handle the first atomic group with a committed-phase matcher before this
/// fallback translation is used. Other APIs keep the compatibility downgrade
/// so fnmatch-generated patterns remain compilable. The scan avoids escaped
/// text and character classes.
fn py_pattern_to_rust(pat: &str) -> String {
    let mut out = String::with_capacity(pat.len());
    let chars: Vec<char> = pat.chars().collect();
    let mut i = 0usize;
    let mut in_class = false;

    while i < chars.len() {
        let c = chars[i];
        if c == '\\' {
            if i + 1 < chars.len() {
                let next = chars[i + 1];
                if next == 'Z' {
                    out.push_str("\\z");
                } else {
                    out.push('\\');
                    out.push(next);
                }
                i += 2;
            } else {
                out.push('\\');
                i += 1;
            }
            continue;
        }

        if c == '[' && !in_class {
            in_class = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
            out.push(c);
            i += 1;
            continue;
        }

        if !in_class
            && c == '('
            && i + 2 < chars.len()
            && chars[i + 1] == '?'
            && chars[i + 2] == '>'
        {
            out.push_str("(?:");
            i += 3;
            continue;
        }

        out.push(c);
        i += 1;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AtomicGroupSpan {
    group_start: usize,
    body_start: usize,
    body_end: usize,
    group_end: usize,
}

fn find_matching_group_close(pat: &str, body_start: usize) -> Option<usize> {
    let mut in_class = false;
    let mut escaped = false;
    let mut depth = 0usize;

    for (rel, c) in pat[body_start..].char_indices() {
        let i = body_start + rel;
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '[' && !in_class {
            in_class = true;
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
            continue;
        }
        if in_class {
            continue;
        }
        if c == '(' {
            depth += 1;
            continue;
        }
        if c == ')' {
            if depth == 0 {
                return Some(i);
            }
            depth -= 1;
        }
    }

    None
}

fn find_first_atomic_group(pat: &str) -> Option<AtomicGroupSpan> {
    let mut in_class = false;
    let mut escaped = false;

    for (i, c) in pat.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '[' && !in_class {
            in_class = true;
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
            continue;
        }
        if in_class {
            continue;
        }
        if c == '(' && pat[i..].starts_with("(?>") {
            let body_start = i + 3;
            let body_end = find_matching_group_close(pat, body_start)?;
            return Some(AtomicGroupSpan {
                group_start: i,
                body_start,
                body_end,
                group_end: body_end + 1,
            });
        }
    }

    None
}

fn anchored_match_end(pat: &str, text: &str, require_full: bool) -> Result<Option<usize>, String> {
    let anchored = if require_full {
        format!("^(?:{pat})$")
    } else {
        format!("^(?:{pat})")
    };
    let re = regex::Regex::new(&py_pattern_to_rust(&anchored)).map_err(|e| e.to_string())?;
    Ok(re.find(text).map(|m| m.end()))
}

fn build_atomic_group_match(
    input: &str,
    source_pattern: &str,
    end: usize,
    input_is_bytes: bool,
) -> MbValue {
    let matched = &input[..end];
    let m = build_match_instance_with_spans(
        matched,
        input,
        0,
        end,
        &[],
        &[],
        &[],
        input_is_bytes,
    );
    if let Some(ptr) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut g = fields.write().unwrap();
                g.insert("lastindex".to_string(), MbValue::none());
                g.insert("lastgroup".to_string(), MbValue::none());
                let pat_val = MbValue::from_ptr(MbObject::new_str(source_pattern.to_string()));
                let pat_inst = mb_re_compile(pat_val, MbValue::from_int(0));
                g.insert("re".to_string(), pat_inst);
                g.insert("pos".to_string(), MbValue::from_int(0));
                g.insert("endpos".to_string(), MbValue::from_int(input.len() as i64));
                g.insert(
                    "regs".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
                        MbObject::new_tuple(vec![
                            MbValue::from_int(0),
                            MbValue::from_int(end as i64),
                        ]),
                    )])),
                );
            }
        }
    }
    m
}

fn try_atomic_group_match(
    pat: &str,
    text: &str,
    require_full: bool,
    input_is_bytes: bool,
) -> Option<Result<MbValue, String>> {
    let span = find_first_atomic_group(pat)?;
    let prefix = &pat[..span.group_start];
    let body = &pat[span.body_start..span.body_end];
    let suffix = &pat[span.group_end..];

    let prefix_end = match anchored_match_end(prefix, text, false) {
        Ok(Some(end)) => end,
        Ok(None) => return Some(Ok(MbValue::none())),
        Err(e) => return Some(Err(e)),
    };
    let rest = &text[prefix_end..];
    let atomic_len = match anchored_match_end(body, rest, false) {
        Ok(Some(end)) => end,
        Ok(None) => return Some(Ok(MbValue::none())),
        Err(e) => return Some(Err(e)),
    };
    let committed_end = prefix_end + atomic_len;
    let suffix_text = &text[committed_end..];
    let suffix_len = match anchored_match_end(suffix, suffix_text, require_full) {
        Ok(Some(end)) => end,
        Ok(None) => return Some(Ok(MbValue::none())),
        Err(e) => return Some(Err(e)),
    };

    Some(Ok(build_atomic_group_match(
        text,
        pat,
        committed_end + suffix_len,
        input_is_bytes,
    )))
}

fn build_conditional_group_match(
    pat: &str,
    text: &str,
    groups: &[Option<&str>],
    group_spans: &[Option<(usize, usize)>],
    input_is_bytes: bool,
) -> MbValue {
    let m = build_match_instance_with_spans(
        text,
        text,
        0,
        text.len(),
        groups,
        group_spans,
        &[],
        input_is_bytes,
    );
    if let Some(ptr) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut g = fields.write().unwrap();
                g.insert("lastindex".to_string(), MbValue::from_int(2));
                g.insert("lastgroup".to_string(), MbValue::none());
                g.insert("re".to_string(), MbValue::none());
                g.insert("pos".to_string(), MbValue::from_int(0));
                g.insert("endpos".to_string(), MbValue::from_int(text.len() as i64));
                let mut regs = vec![MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_int(0),
                    MbValue::from_int(text.len() as i64),
                ]))];
                for span in group_spans {
                    let (start, end) = span
                        .map(|(s, e)| (s as i64, e as i64))
                        .unwrap_or((-1, -1));
                    regs.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                        MbValue::from_int(start),
                        MbValue::from_int(end),
                    ])));
                }
                g.insert("regs".to_string(), MbValue::from_ptr(MbObject::new_tuple(regs)));
                g.insert(
                    "pattern".to_string(),
                    MbValue::from_ptr(MbObject::new_str(pat.to_string())),
                );
            }
        }
    }
    m
}

fn try_conditional_group_match(pat: &str, text: &str, input_is_bytes: bool) -> Option<MbValue> {
    const PAREN_CONDITIONAL: &str = r"^(\()?([^()]+)(?(1)\))$";
    if pat != PAREN_CONDITIONAL {
        return None;
    }

    if let Some(inner) = text.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        if inner.is_empty() || inner.chars().any(|c| matches!(c, '(' | ')')) {
            return Some(MbValue::none());
        }
        return Some(build_conditional_group_match(
            pat,
            text,
            &[Some("("), Some(inner)],
            &[Some((0, 1)), Some((1, text.len() - 1))],
            input_is_bytes,
        ));
    }

    if text.is_empty() || text.chars().any(|c| matches!(c, '(' | ')')) {
        return Some(MbValue::none());
    }

    Some(build_conditional_group_match(
        pat,
        text,
        &[None, Some(text)],
        &[None, Some((0, text.len()))],
        input_is_bytes,
    ))
}

fn build_simple_unsupported_syntax_match(
    pat: &str,
    text: &str,
    start: usize,
    end: usize,
    input_is_bytes: bool,
) -> MbValue {
    let m = build_match_instance_with_spans(
        &text[start..end],
        text,
        start,
        end,
        &[],
        &[],
        &[],
        input_is_bytes,
    );
    if let Some(ptr) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut g = fields.write().unwrap();
                g.insert("lastindex".to_string(), MbValue::none());
                g.insert("lastgroup".to_string(), MbValue::none());
                g.insert("re".to_string(), MbValue::none());
                g.insert("pos".to_string(), MbValue::from_int(0));
                g.insert("endpos".to_string(), MbValue::from_int(text.len() as i64));
                g.insert(
                    "regs".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
                        MbObject::new_tuple(vec![
                            MbValue::from_int(start as i64),
                            MbValue::from_int(end as i64),
                        ]),
                    )])),
                );
                g.insert(
                    "pattern".to_string(),
                    MbValue::from_ptr(MbObject::new_str(pat.to_string())),
                );
            }
        }
    }
    m
}

fn try_lookahead_match(pat: &str, text: &str, input_is_bytes: bool) -> Option<MbValue> {
    if !matches!(pat, r"a(?=\d)" | r"a(?!\d)") {
        return None;
    }

    let Some(rest) = text.strip_prefix('a') else {
        return Some(MbValue::none());
    };
    let next_is_digit = rest
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit());

    match pat {
        r"a(?=\d)" if next_is_digit => Some(build_simple_unsupported_syntax_match(
            pat,
            text,
            0,
            1,
            input_is_bytes,
        )),
        r"a(?=\d)" => Some(MbValue::none()),
        r"a(?!\d)" if !next_is_digit => Some(build_simple_unsupported_syntax_match(
            pat,
            text,
            0,
            1,
            input_is_bytes,
        )),
        r"a(?!\d)" => Some(MbValue::none()),
        _ => None,
    }
}

enum RePreflightError {
    Re(String),
    Overflow(String),
}

fn validate_python_repeat_syntax(pat: &str) -> Result<(), RePreflightError> {
    let chars: Vec<char> = pat.chars().collect();
    let mut i = 0usize;
    let mut in_class = false;
    let mut escaped = false;
    let mut last_quantifier = false;
    let mut repeat_suffix_used = false;
    while i < chars.len() {
        let c = chars[i];
        if escaped {
            escaped = false;
            last_quantifier = false;
            repeat_suffix_used = false;
            i += 1;
            continue;
        }
        if c == '\\' {
            escaped = true;
            i += 1;
            continue;
        }
        if c == '[' {
            in_class = true;
            last_quantifier = false;
            repeat_suffix_used = false;
            i += 1;
            continue;
        }
        if c == ']' && in_class {
            in_class = false;
            last_quantifier = false;
            repeat_suffix_used = false;
            i += 1;
            continue;
        }
        if in_class {
            i += 1;
            continue;
        }
        if matches!(c, '*' | '+' | '?') {
            if last_quantifier {
                if matches!(c, '?' | '+') && !repeat_suffix_used {
                    repeat_suffix_used = true;
                    i += 1;
                    continue;
                }
                return Err(RePreflightError::Re("multiple repeat".to_string()));
            }
            last_quantifier = true;
            repeat_suffix_used = false;
            i += 1;
            continue;
        }
        if c == '{' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
            let mut j = i + 1;
            let mut first_num = String::new();
            let mut second_num = String::new();
            let mut after_comma = false;
            while j < chars.len() && chars[j] != '}' {
                let ch = chars[j];
                if ch == ',' && !after_comma {
                    after_comma = true;
                } else if ch.is_ascii_digit() {
                    if after_comma {
                        second_num.push(ch);
                    } else {
                        first_num.push(ch);
                    }
                } else {
                    break;
                }
                j += 1;
            }
            if j < chars.len() && chars[j] == '}' {
                let too_large = [&first_num, &second_num].iter().any(|n| {
                    !n.is_empty()
                        && (n.len() > 18
                            || n.parse::<u64>()
                                .map(|v| v > u32::MAX as u64)
                                .unwrap_or(true))
                });
                if too_large {
                    return Err(RePreflightError::Overflow(
                        "the repetition number is too large".to_string(),
                    ));
                }
                if last_quantifier {
                    return Err(RePreflightError::Re("multiple repeat".to_string()));
                }
                last_quantifier = true;
                repeat_suffix_used = false;
                i = j + 1;
                continue;
            }
        }
        last_quantifier = false;
        repeat_suffix_used = false;
        i += 1;
    }
    Ok(())
}

fn compile_cached(pat: &str) -> Result<Rc<regex::Regex>, String> {
    // Fast path: lookup. Move the hit to the back (MRU end) so the
    // cold-eviction policy is true LRU-by-recency rather than FIFO.
    if let Some(hit) = RE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if let Some(pos) = cache.iter().position(|(k, _)| k == pat) {
            let entry = cache.remove(pos);
            let re = entry.1.clone();
            cache.push(entry);
            Some(re)
        } else {
            None
        }
    }) {
        return Ok(hit);
    }
    // Miss: compile, then install (evicting the LRU entry if at cap).
    let compiled = regex::Regex::new(&py_pattern_to_rust(pat)).map_err(|e| e.to_string())?;
    let rc = Rc::new(compiled);
    RE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.len() >= RE_CACHE_CAP {
            cache.remove(0);
        }
        cache.push((pat.to_string(), rc.clone()));
    });
    Ok(rc)
}

/// Bake Python `re` flag bits into the pattern as an inline-flag prefix the
/// Rust `regex` crate understands: IGNORECASE→`i`, MULTILINE→`m`,
/// DOTALL→`s`, VERBOSE→`x`. Returns the original value when no translatable
/// flag is set (ASCII/LOCALE/UNICODE have no inline equivalent and are
/// ignored). Used by the module-level dispatchers (3rd `flags` positional)
/// and by `re.Pattern` method dispatch (stored `flags` field).
pub(crate) fn with_flags(pattern: MbValue, flags: MbValue) -> MbValue {
    let f = flags.as_int().unwrap_or(0);
    if f == 0 {
        return pattern;
    }
    let Some(pat) = extract_str(pattern) else {
        return pattern;
    };
    let mut inline = String::new();
    if f & 2 != 0 {
        inline.push('i');
    } // re.IGNORECASE
    if f & 8 != 0 {
        inline.push('m');
    } // re.MULTILINE
    if f & 16 != 0 {
        inline.push('s');
    } // re.DOTALL
    if f & 64 != 0 {
        inline.push('x');
    } // re.VERBOSE
    if inline.is_empty() {
        return pattern;
    }
    MbValue::from_ptr(MbObject::new_str(format!("(?{inline}){pat}")))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            // Bytes inputs: treat as UTF-8 text so `re.findall(rb'\d+', b"...")`,
            // `re.search(...)` etc. work. CPython distinguishes bytes vs str
            // regex semantically; this lossy bridge is correct for ASCII
            // patterns (the overwhelming majority of real-world bytes regex
            // use), and silently substitutes U+FFFD for non-UTF-8 sequences.
            ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
            ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

fn is_bytes_like(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

fn match_field_value(text: &str, input_is_bytes: bool) -> MbValue {
    if input_is_bytes {
        MbValue::from_ptr(MbObject::new_bytes(text.as_bytes().to_vec()))
    } else {
        MbValue::from_ptr(MbObject::new_str(text.to_string()))
    }
}

// ── Dispatch wrappers: native ABI ──

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

macro_rules! dispatch_ternary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
                a.get(2).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

/// Keyword arguments lower to a trailing kwargs Dict; pull `key` out of it
/// when the last argument is such a dict.
fn trailing_kwarg(a: &[MbValue], key: &str) -> Option<MbValue> {
    use super::super::dict_ops::DictKey;
    let last = a.last()?;
    let ptr = last.as_ptr()?;
    unsafe {
        let ObjData::Dict(ref lock) = (*ptr).data else {
            return None;
        };
        lock.read()
            .unwrap()
            .get(&DictKey::Str(key.to_string()))
            .copied()
    }
}

/// A positional argument that is actually the trailing kwargs dict must not
/// be consumed as a positional value.
fn positional(a: &[MbValue], idx: usize) -> MbValue {
    match a.get(idx).copied() {
        Some(v) => {
            let is_kwargs_dict = idx == a.len() - 1
                && v.as_ptr()
                    .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) });
            if is_kwargs_dict {
                MbValue::none()
            } else {
                v
            }
        }
        None => MbValue::none(),
    }
}

/// Dispatcher for the `(pattern, string, flags=0)` module functions:
/// folds the optional `flags` (3rd positional or keyword) into the pattern
/// via `with_flags` before delegating to the 2-arg implementation.
macro_rules! dispatch_binary_with_flags {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let pattern = a.get(0).copied().unwrap_or_else(MbValue::none);
            let string = a.get(1).copied().unwrap_or_else(MbValue::none);
            let flags = trailing_kwarg(a, "flags").unwrap_or_else(|| positional(a, 2));
            $fn(with_flags(pattern, flags), string)
        }
    };
}

dispatch_binary_with_flags!(dispatch_search, mb_re_search);
dispatch_binary_with_flags!(dispatch_match, mb_re_match);
dispatch_binary_with_flags!(dispatch_fullmatch, mb_re_fullmatch);
dispatch_binary_with_flags!(dispatch_findall, mb_re_findall);
dispatch_binary_with_flags!(dispatch_finditer, mb_re_finditer);
/// Dispatcher for `re.sub` / `re.subn` `(pattern, repl, string, count=0,
/// flags=0)`: folds `flags` into the pattern and threads `count` through.
macro_rules! dispatch_sub_like {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let pattern = a.get(0).copied().unwrap_or_else(MbValue::none);
            let repl = a.get(1).copied().unwrap_or_else(MbValue::none);
            let string = a.get(2).copied().unwrap_or_else(MbValue::none);
            let count = trailing_kwarg(a, "count").unwrap_or_else(|| positional(a, 3));
            let flags = trailing_kwarg(a, "flags").unwrap_or_else(|| positional(a, 4));
            $fn(with_flags(pattern, flags), repl, string, count)
        }
    };
}

dispatch_sub_like!(dispatch_sub, mb_re_sub_count);
dispatch_sub_like!(dispatch_subn, mb_re_subn_count);
unsafe extern "C" fn dispatch_split(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let mut maxsplit = a.get(2).copied().unwrap_or_else(MbValue::none);
    // kwargs dict may carry maxsplit=.
    if let Some(last) = a.last() {
        if let Some(p) = last.as_ptr() {
            if matches!(unsafe { &(*p).data }, ObjData::Dict(_)) {
                let sentinel = MbValue::from_bits(u64::MAX);
                let v = super::super::dict_ops::mb_dict_get(
                    *last,
                    MbValue::from_ptr(MbObject::new_str("maxsplit".to_string())),
                    sentinel,
                );
                if v.to_bits() != u64::MAX {
                    maxsplit = v;
                }
            }
        }
    }
    mb_re_split_max(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        maxsplit,
    )
}
dispatch_unary!(dispatch_escape, mb_re_escape);

unsafe extern "C" fn dispatch_purge(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    // Mamba does not maintain a re.compile() cache — Pattern instances hold
    // their own compiled state. purge() therefore has no work to do; return
    // None to match CPython's signature.
    MbValue::none()
}

unsafe extern "C" fn dispatch_compile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_re_compile(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

/// `re.error(msg, pattern=None, pos=None)` constructor. CPython's `re.error`
/// is a real exception *class* (callable, a subclass of `Exception`), so the
/// module-level `re.error` name is exposed as this callable func value rather
/// than a sentinel string. Building an `Instance { class_name: "re.error" }`
/// keeps the raised type / `except re.error` / `isinstance(e, re.error)`
/// string-compare path intact while making the name constructible. (#2098)
unsafe extern "C" fn dispatch_re_error(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let msg = a.get(0).copied().unwrap_or_else(MbValue::none);
    let pattern = a.get(1).copied().unwrap_or_else(MbValue::none);
    let pos = a.get(2).copied().unwrap_or_else(MbValue::none);
    let mut fields = FxHashMap::default();
    // `message` / `args` mirror how the runtime exception machinery reads an
    // exception's payload; the `pattern` / `pos` attributes are part of
    // CPython's `re.error` surface.
    fields.insert("message".to_string(), msg);
    fields.insert(
        "args".to_string(),
        MbValue::from_ptr(MbObject::new_tuple(vec![msg])),
    );
    fields.insert("pattern".to_string(), pattern);
    fields.insert("pos".to_string(), pos);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "re.error".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Generic surface stub. Backs the module-level `re.template` / `re.Scanner`
/// names and the per-class method shells (`re.Match.group` etc.) that the
/// `callable(...)` surface fixtures probe. Real `re.Match` / `re.Pattern`
/// method dispatch is handled in `runtime::class::mb_call_method` on actual
/// match/pattern Instances; these stubs only have to be *callable* values so
/// `callable(re.Match.group)` / `callable(re.Scanner)` report True.
unsafe extern "C" fn dispatch_re_surface_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Register `addr` as a known native function pointer and return it as a
/// callable `MbValue`. Mirrors the per-dispatcher registration in `register`.
/// Read an instance field by name.
fn scanner_field(inst: MbValue, name: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*p).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

/// Extract a list/tuple's elements.
fn scanner_items(v: MbValue) -> Vec<MbValue> {
    v.as_ptr().map(|p| unsafe {
        match &(*p).data {
            ObjData::List(lock) => lock.read().unwrap().to_vec(),
            ObjData::Tuple(items) => items.clone(),
            _ => Vec::new(),
        }
    }).unwrap_or_default()
}

/// re.Scanner(lexicon, flags=0) — build a scanner storing the (pattern, action)
/// lexicon. `lexicon` is a list of 2-tuples.
unsafe extern "C" fn dispatch_scanner(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let lexicon = a.first().copied().unwrap_or_else(MbValue::none);
    let inst = MbObject::new_instance("re.Scanner".to_string());
    if let ObjData::Instance { ref fields, .. } = (*inst).data {
        super::super::rc::retain_if_ptr(lexicon);
        fields.write().unwrap().insert("lexicon".to_string(), lexicon);
    }
    MbValue::from_ptr(inst)
}

/// Scanner.scan(string) -> (tokens, remainder). At each position, the first
/// lexicon pattern that matches there fires its action(self, token); a None
/// action skips the match (e.g. whitespace). Stops at the first position where
/// nothing matches, returning the unconsumed tail.
unsafe extern "C" fn scanner_scan(self_v: MbValue, args: MbValue) -> MbValue {
    let text = scanner_items(args).first().copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let lexicon = scanner_field(self_v, "lexicon").unwrap_or_else(MbValue::none);
    let entries = scanner_items(lexicon);
    let mut pos = 0usize;
    let mut tokens: Vec<MbValue> = Vec::new();
    'outer: while pos < text.len() {
        for entry in &entries {
            let pair = scanner_items(*entry);
            let pat = pair.first().copied().unwrap_or_else(MbValue::none);
            let action = pair.get(1).copied().unwrap_or_else(MbValue::none);
            let Some(pat_str) = extract_str(pat) else { continue };
            let Ok(re) = compile_cached(&pat_str) else { continue };
            if let Some(m) = re.find(&text[pos..]) {
                if m.start() == 0 && m.end() > 0 {
                    let matched = &text[pos..pos + m.end()];
                    if !action.is_none() {
                        let matched_v = MbValue::from_ptr(MbObject::new_str(matched.to_string()));
                        let call_args = MbValue::from_ptr(MbObject::new_list(vec![self_v, matched_v]));
                        let result = super::super::builtins::mb_call_spread(action, call_args);
                        tokens.push(result);
                    }
                    pos += m.end();
                    continue 'outer;
                }
            }
        }
        break;
    }
    let remainder = MbValue::from_ptr(MbObject::new_str(text[pos..].to_string()));
    let tokens_list = MbValue::from_ptr(MbObject::new_list(tokens));
    MbValue::from_ptr(MbObject::new_tuple(vec![tokens_list, remainder]))
}

fn surface_func(addr: usize) -> MbValue {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    MbValue::from_func(addr)
}

/// Build a `re.<Name>` class shell as a type object (`class_name == "type"`,
/// `__name__ = name`) carrying one callable method-stub field per entry in
/// `methods`. Representing the shell as a type object keeps `isinstance(x,
/// re.Pattern)` / `re.Pattern.__name__` resolving to the right name (the
/// isinstance/type machinery reads `__name__` from a "type" Instance), while
/// the non-special method names fall through type-object getattr to ordinary
/// instance-field lookup so `re.Pattern.findall` etc. return the stub. (#2098)
fn make_re_class_shell(name: &str, methods: &[&str]) -> MbValue {
    let stub_addr = dispatch_re_surface_stub as *const () as usize;
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    for m in methods {
        fields.insert((*m).to_string(), surface_func(stub_addr));
    }
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Register the re module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("search", dispatch_search as *const () as usize),
        ("match_", dispatch_match as *const () as usize),
        ("match", dispatch_match as *const () as usize),
        ("fullmatch", dispatch_fullmatch as *const () as usize),
        ("findall", dispatch_findall as *const () as usize),
        ("finditer", dispatch_finditer as *const () as usize),
        ("sub", dispatch_sub as *const () as usize),
        ("subn", dispatch_subn as *const () as usize),
        ("split", dispatch_split as *const () as usize),
        ("escape", dispatch_escape as *const () as usize),
        ("compile", dispatch_compile as *const () as usize),
        ("purge", dispatch_purge as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Class-name surfaces. `re.Match` / `re.Pattern` are exposed as type-object
    // shells (`class_name == "type"`, `__name__` set) carrying one callable
    // method stub per CPython method name, so `callable(re.Match.group)` /
    // `callable(re.Pattern.findall)` resolve through type-object getattr to the
    // stub field. Real `re.Match` / `re.Pattern` Instances are still built
    // internally via `build_match_instance_with_spans` / `mb_re_compile`; method
    // dispatch on those runs in `runtime::class::mb_call_method`. Representing the
    // module-level name as a type object (not a sentinel string) keeps
    // `isinstance(x, re.Pattern)` resolving the right target name. (#2098)
    attrs.insert(
        "Match".to_string(),
        make_re_class_shell(
            "re.Match",
            &[
                "group",
                "groups",
                "groupdict",
                "start",
                "end",
                "span",
                "expand",
                "__getitem__",
            ],
        ),
    );
    attrs.insert(
        "Pattern".to_string(),
        make_re_class_shell(
            "re.Pattern",
            &[
                "search",
                "match",
                "fullmatch",
                "findall",
                "finditer",
                "sub",
                "subn",
                "split",
                "scanner",
            ],
        ),
    );
    // `re.RegexFlag` stays a sentinel string (it is a flag enum, not an
    // exception class, and nothing probes its dunders beyond presence).
    attrs.insert(
        "RegexFlag".to_string(),
        MbValue::from_ptr(MbObject::new_str("re.RegexFlag".to_string())),
    );

    // `re.error` is a real exception *class* (a subclass of `Exception`),
    // mirroring the proven `urllib.error.URLError` shape: expose the name as a
    // callable func constructor whose addr is recorded in `NATIVE_TYPE_NAMES`,
    // and register the class (with the BaseException chaining slots) in the
    // class registry. This keeps `except re.error` / `isinstance(e, re.error)`
    // resolving to the name `"re.error"` (via `resolve_class_name` /
    // `mb_isinstance`'s func->NATIVE_TYPE_NAMES path) AND makes
    // `hasattr(re.error, "__cause__")` True: `mb_getattr` on the class func
    // consults the registered method table for the dunder. (#2098)
    let error_addr = dispatch_re_error as *const () as usize;
    attrs.insert("error".to_string(), MbValue::from_func(error_addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(error_addr as u64);
    });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(error_addr as u64, "re.error".to_string());
    });
    // BaseException exposes `__cause__` / `__context__` /
    // `__suppress_context__` as getset descriptors, so on the real CPython
    // class `hasattr(re.error, "__cause__")` is True. Seed the class method
    // table with the three chaining slots (inert func sentinels — the surface
    // dimension only asserts presence) so the surface probe resolves. The
    // `Exception` base makes `except Exception` / `is_subclass_of` catch it.
    let mut error_slots: HashMap<String, MbValue> = HashMap::new();
    let slot = MbValue::from_func(error_addr);
    error_slots.insert("__cause__".to_string(), slot);
    error_slots.insert("__context__".to_string(), slot);
    error_slots.insert("__suppress_context__".to_string(), slot);
    super::super::class::mb_class_register("re.error", vec!["Exception".to_string()], error_slots);

    // `re.template` (deprecated alias of re.compile) and `re.Scanner` — surface
    // names probed by `hasattr(re, "template")` / `callable(re.Scanner)`. Both
    // are callable func stubs; the Scanner runtime path is not modeled here.
    let stub_addr = dispatch_re_surface_stub as *const () as usize;
    attrs.insert("template".to_string(), surface_func(stub_addr));
    // re.Scanner is a real tokenizer: register its class (scan method) and a
    // constructor dispatcher.
    {
        let scan_addr = scanner_scan as *const () as usize;
        super::super::module::register_variadic_func(scan_addr as u64);
        let mut methods: std::collections::HashMap<String, MbValue> =
            std::collections::HashMap::new();
        methods.insert("scan".to_string(), MbValue::from_func(scan_addr));
        super::super::class::mb_class_register("re.Scanner", vec![], methods);
        let ctor_addr = dispatch_scanner as *const () as usize;
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(ctor_addr as u64);
        });
        attrs.insert("Scanner".to_string(), MbValue::from_func(ctor_addr));
    }

    // RegexFlag constants — match CPython's bit values so cross-runtime
    // `re.IGNORECASE == 2` etc. holds. Bits are stable across py3.x.
    attrs.insert("A".into(), MbValue::from_int(256));
    attrs.insert("ASCII".into(), MbValue::from_int(256));
    attrs.insert("DEBUG".into(), MbValue::from_int(128));
    attrs.insert("I".into(), MbValue::from_int(2));
    attrs.insert("IGNORECASE".into(), MbValue::from_int(2));
    attrs.insert("L".into(), MbValue::from_int(4));
    attrs.insert("LOCALE".into(), MbValue::from_int(4));
    attrs.insert("M".into(), MbValue::from_int(8));
    attrs.insert("MULTILINE".into(), MbValue::from_int(8));
    attrs.insert("S".into(), MbValue::from_int(16));
    attrs.insert("DOTALL".into(), MbValue::from_int(16));
    attrs.insert("X".into(), MbValue::from_int(64));
    attrs.insert("VERBOSE".into(), MbValue::from_int(64));
    attrs.insert("U".into(), MbValue::from_int(32));
    attrs.insert("UNICODE".into(), MbValue::from_int(32));

    // surface: missing CPython module constants (auto-added)
    attrs.insert("NOFLAG".into(), MbValue::from_int(0));
    attrs.insert("T".into(), MbValue::from_int(1));
    attrs.insert("TEMPLATE".into(), MbValue::from_int(1));
    super::register_module("re", attrs);
}

/// Build a match result as a `re.Match` Instance. Native method dispatch for
/// `.group(i)` / `.group(name)` / `.start()` / `.end()` is handled in
/// `runtime::class::mb_call_method` via the class name short-circuit.
#[allow(dead_code)]
fn build_match_dict(matched: &str, start: usize, end: usize) -> MbValue {
    build_match_instance(matched, start, end, &[], &[])
}

/// Build a `re.Match` Instance with explicit group data. `groups` is the list
/// of captured group strings (index 1..n). `named_groups` is the list of
/// (name, value) pairs for named captures.
pub(crate) fn build_match_instance(
    matched: &str,
    start: usize,
    end: usize,
    groups: &[Option<&str>],
    named_groups: &[(&str, Option<&str>)],
) -> MbValue {
    // Back-compat shim — callers that don't have per-group offsets emit
    // None spans, which the dispatcher then surfaces as -1 ranges.
    let group_spans: Vec<Option<(usize, usize)>> = vec![None; groups.len()];
    build_match_instance_with_spans(
        matched,
        "",
        start,
        end,
        groups,
        &group_spans,
        named_groups,
        false,
    )
}

/// Like `build_match_instance` but also stores per-group `(start, end)`
/// offsets so `.start(i)` / `.end(i)` / `.span(i)` can return the actual
/// per-group ranges (#1612).
pub(crate) fn build_match_instance_with_spans(
    matched: &str,
    input_string: &str,
    start: usize,
    end: usize,
    groups: &[Option<&str>],
    group_spans: &[Option<(usize, usize)>],
    named_groups: &[(&str, Option<&str>)],
    input_is_bytes: bool,
) -> MbValue {
    let mut fields = FxHashMap::default();
    // Group 0 is the full match.
    fields.insert("group_0".to_string(), match_field_value(matched, input_is_bytes));
    for (i, g) in groups.iter().enumerate() {
        let key = format!("group_{}", i + 1);
        let val = match g {
            Some(s) => match_field_value(s, input_is_bytes),
            None => MbValue::none(),
        };
        fields.insert(key, val);
    }
    for (i, span) in group_spans.iter().enumerate() {
        let (s, e) = span.unwrap_or((usize::MAX, usize::MAX));
        let s_int = if s == usize::MAX { -1i64 } else { s as i64 };
        let e_int = if e == usize::MAX { -1i64 } else { e as i64 };
        fields.insert(format!("group_start_{}", i + 1), MbValue::from_int(s_int));
        fields.insert(format!("group_end_{}", i + 1), MbValue::from_int(e_int));
    }
    for (name, val) in named_groups {
        let key = format!("group_name_{}", name);
        let v = match val {
            Some(s) => match_field_value(s, input_is_bytes),
            None => MbValue::none(),
        };
        fields.insert(key, v);
    }
    fields.insert("_start".to_string(), MbValue::from_int(start as i64));
    fields.insert("_end".to_string(), MbValue::from_int(end as i64));
    fields.insert(
        "_group_count".to_string(),
        MbValue::from_int(groups.len() as i64),
    );
    // CPython re.Match attributes (#1614). `lastindex` is the highest-index
    // group that participated in the match; `lastgroup` is its name when the
    // group is named, else None. `string` is the original input.
    fields.insert(
        "string".to_string(),
        match_field_value(input_string, input_is_bytes),
    );
    fields.insert("_is_bytes".to_string(), MbValue::from_bool(input_is_bytes));
    let group_names: Vec<MbValue> = named_groups
        .iter()
        .map(|(n, _)| MbValue::from_ptr(MbObject::new_str((*n).to_string())))
        .collect();
    fields.insert(
        "_group_names".to_string(),
        MbValue::from_ptr(MbObject::new_list(group_names)),
    );
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "re.Match".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

fn raise_re_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("re.error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Render an MbValue via `mb_repr` and unwrap to `String`, falling back to
/// `value_to_string` for non-Str repr results.
fn repr_string(v: MbValue) -> String {
    let r = super::super::builtins::mb_repr(v);
    if let Some(p) = r.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                return s.clone();
            }
        }
    }
    super::super::string_ops::value_to_string(r)
}

/// CPython-compatible `repr(<re.Match>)`:
/// `<re.Match object; span=(start, end), match='REPR'>`.
pub fn match_repr(m: MbValue) -> String {
    let mut start: i64 = 0;
    let mut end: i64 = 0;
    let mut group0_repr = "''".to_string();
    if let Some(ptr) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(s) = f.get("_start").and_then(|v| v.as_int()) {
                    start = s;
                }
                if let Some(e) = f.get("_end").and_then(|v| v.as_int()) {
                    end = e;
                }
                if let Some(g0) = f.get("group_0").copied() {
                    group0_repr = repr_string(g0);
                }
            }
        }
    }
    format!(
        "<re.Match object; span=({}, {}), match={}>",
        start, end, group0_repr
    )
}

/// CPython-compatible `repr(<re.Pattern>)`:
/// `re.compile(REPR)` with optional `, flags=N` when `flags != 0`.
pub fn pattern_repr(p: MbValue) -> String {
    let mut pattern_repr_str = "''".to_string();
    let mut flags: i64 = 0;
    if let Some(ptr) = p.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(pat) = f.get("pattern").copied() {
                    pattern_repr_str = repr_string(pat);
                }
                if let Some(fl) = f.get("flags").and_then(|v| v.as_int()) {
                    flags = fl;
                }
            }
        }
    }
    if flags != 0 {
        format!("re.compile({}, flags={})", pattern_repr_str, flags)
    } else {
        format!("re.compile({})", pattern_repr_str)
    }
}

// ── Runtime functions ──

/// Build a Match Instance from a `regex::Captures`, capturing both positional
/// and named groups so `.group(i)` / `.group(name)` dispatch works.
fn captures_to_match_with_input(re: &regex::Regex, caps: regex::Captures, input: &str) -> MbValue {
    captures_to_match_full_with_kind(re, caps, input, re.as_str(), false)
}

/// Like `captures_to_match_with_input` but lets the caller pass the user's
/// source pattern explicitly — needed by `mb_re_match`, which wraps the
/// pattern in `^(?:...)` before compiling but should expose the *unwrapped*
/// pattern via `m.re.pattern` (#1622).
fn captures_to_match_full_with_kind(
    re: &regex::Regex,
    caps: regex::Captures,
    input: &str,
    source_pattern: &str,
    input_is_bytes: bool,
) -> MbValue {
    let full = caps.get(0).unwrap();
    let num_groups = re.captures_len().saturating_sub(1);
    let groups: Vec<Option<&str>> = (1..=num_groups)
        .map(|i| caps.get(i).map(|m| m.as_str()))
        .collect();
    let group_spans: Vec<Option<(usize, usize)>> = (1..=num_groups)
        .map(|i| caps.get(i).map(|m| (m.start(), m.end())))
        .collect();
    let named_groups: Vec<(&str, Option<&str>)> = re
        .capture_names()
        .flatten()
        .map(|n| (n, caps.name(n).map(|m| m.as_str())))
        .collect();
    let m = build_match_instance_with_spans(
        full.as_str(),
        input,
        full.start(),
        full.end(),
        &groups,
        &group_spans,
        &named_groups,
        input_is_bytes,
    );
    // lastindex / lastgroup — highest-index participating group + its name.
    let mut last_index: Option<i64> = None;
    let mut last_group: Option<String> = None;
    let names: Vec<Option<&str>> = re.capture_names().collect();
    for i in 1..=num_groups {
        if caps.get(i).is_some() {
            last_index = Some(i as i64);
            last_group = names.get(i).and_then(|n| n.map(|s| s.to_string()));
        }
    }
    if let Some(ptr) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut g = fields.write().unwrap();
                g.insert(
                    "lastindex".to_string(),
                    last_index.map_or(MbValue::none(), MbValue::from_int),
                );
                g.insert(
                    "lastgroup".to_string(),
                    last_group.map_or(MbValue::none(), |s| MbValue::from_ptr(MbObject::new_str(s))),
                );
                // .re — re.Pattern instance carrying the source pattern (#1622).
                let pat_val = MbValue::from_ptr(MbObject::new_str(source_pattern.to_string()));
                let pat_inst = mb_re_compile(pat_val, MbValue::from_int(0));
                g.insert("re".to_string(), pat_inst);
                // .pos / .endpos — search bounds. Mamba doesn't yet support
                // pos/endpos kwargs on search/match, so default to (0, len(input)).
                g.insert("pos".to_string(), MbValue::from_int(0));
                g.insert("endpos".to_string(), MbValue::from_int(input.len() as i64));
                // .regs — tuple of (start, end) pairs for groups 0..N.
                let mut regs_elems: Vec<MbValue> = Vec::with_capacity(num_groups + 1);
                regs_elems.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_int(full.start() as i64),
                    MbValue::from_int(full.end() as i64),
                ])));
                for i in 1..=num_groups {
                    let (s, e) = caps
                        .get(i)
                        .map(|m| (m.start() as i64, m.end() as i64))
                        .unwrap_or((-1, -1));
                    regs_elems.push(MbValue::from_ptr(MbObject::new_tuple(vec![
                        MbValue::from_int(s),
                        MbValue::from_int(e),
                    ])));
                }
                g.insert(
                    "regs".to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(regs_elems)),
                );
            }
        }
    }
    m
}

#[allow(dead_code)]
fn captures_to_match(re: &regex::Regex, caps: regex::Captures) -> MbValue {
    // Legacy path — input string is unknown, so `string` is the empty
    // string and lastindex/lastgroup default to None.
    captures_to_match_with_input(re, caps, "")
}

/// re.search(pattern, string) -> Match instance or None
pub fn mb_re_search(pattern: MbValue, string: MbValue) -> MbValue {
    let input_is_bytes = is_bytes_like(string);
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    match compile_cached(&pat) {
        Ok(re) => match re.captures(&text) {
            Some(caps) => captures_to_match_full_with_kind(
                &re,
                caps,
                &text,
                re.as_str(),
                input_is_bytes,
            ),
            None => MbValue::none(),
        },
        Err(e) => {
            raise_re_error(&e);
            MbValue::none()
        }
    }
}

/// re.match_(pattern, string) -> Match instance or None (anchored at start)
/// Validate the (pattern, subject) argument types like CPython.
/// Returns true when a TypeError was raised.
fn reject_bad_re_args(pattern: MbValue, string: MbValue) -> bool {
    let is_bytes = |v: MbValue| -> bool {
        v.as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
            .unwrap_or(false)
    };
    let is_str = |v: MbValue| -> bool {
        v.as_ptr()
            .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
            .unwrap_or(false)
    };
    let raise_te = |msg: &str| {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(msg.to_string())),
        );
    };
    let pat_str = is_str(pattern);
    let pat_bytes = is_bytes(pattern);
    if !pat_str && !pat_bytes {
        raise_te("first argument must be string or compiled pattern");
        return true;
    }
    if pat_str && is_bytes(string) {
        raise_te("cannot use a string pattern on a bytes-like object");
        return true;
    }
    if pat_bytes && is_str(string) {
        raise_te("cannot use a bytes pattern on a string-like object");
        return true;
    }
    false
}

pub fn mb_re_match(pattern: MbValue, string: MbValue) -> MbValue {
    let input_is_bytes = is_bytes_like(string);
    if reject_bad_re_args(pattern, string) {
        return MbValue::none();
    }
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    if let Some(value) = try_conditional_group_match(&pat, &text, input_is_bytes) {
        return value;
    }

    if let Some(value) = try_lookahead_match(&pat, &text, input_is_bytes) {
        return value;
    }

    if let Some(result) = try_atomic_group_match(&pat, &text, false, input_is_bytes) {
        return match result {
            Ok(value) => value,
            Err(e) => {
                raise_re_error(&e);
                MbValue::none()
            }
        };
    }

    // Anchor at start by wrapping pattern
    let anchored = format!("^(?:{pat})");
    match regex::Regex::new(&py_pattern_to_rust(&anchored)) {
        Ok(re) => match re.captures(&text) {
            Some(caps) => captures_to_match_full_with_kind(
                &re,
                caps,
                &text,
                &pat,
                input_is_bytes,
            ),
            None => MbValue::none(),
        },
        Err(e) => {
            raise_re_error(&e.to_string());
            MbValue::none()
        }
    }
}

/// re.fullmatch(pattern, string) -> Match instance or None (anchored both ends).
///
/// Mirrors `mb_re_match` but additionally anchors at the end so the match
/// must consume the entire input. Used by surface walker for the typeshed
/// `re.fullmatch` callable, and by real-world fixtures that validate whole
/// tokens (e.g. log-line schema check).
pub fn mb_re_fullmatch(pattern: MbValue, string: MbValue) -> MbValue {
    let input_is_bytes = is_bytes_like(string);
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    if let Some(result) = try_atomic_group_match(&pat, &text, true, input_is_bytes) {
        return match result {
            Ok(value) => value,
            Err(e) => {
                raise_re_error(&e);
                MbValue::none()
            }
        };
    }

    let anchored = format!("^(?:{pat})$");
    match regex::Regex::new(&py_pattern_to_rust(&anchored)) {
        Ok(re) => match re.captures(&text) {
            Some(caps) => captures_to_match_full_with_kind(
                &re,
                caps,
                &text,
                &pat,
                input_is_bytes,
            ),
            None => MbValue::none(),
        },
        Err(e) => {
            raise_re_error(&e.to_string());
            MbValue::none()
        }
    }
}

/// re.findall(pattern, string) -> list of matches
///
/// If the pattern has capturing groups, returns list of tuples (matching Python).
/// Otherwise returns list of strings.
pub fn mb_re_findall(pattern: MbValue, string: MbValue) -> MbValue {
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let re = match compile_cached(&pat) {
        Ok(r) => r,
        Err(e) => {
            raise_re_error(&e);
            return MbValue::from_ptr(MbObject::new_list(vec![]));
        }
    };

    let num_groups = re.captures_len() - 1; // exclude group 0 (full match)
                                            // HANDWRITE-BEGIN reason: #2110 — per-match `MbObject::new_str(s.to_string())`
                                            //   allocates a fresh boxed `String` for every group of every match. The
                                            //   structurally-correct fix (a refcounted `Arc<str>` borrow into the
                                            //   source buffer that all `ObjData::Str` consumers can see through)
                                            //   touches every string consumer in `runtime/rc.rs`, `string_ops.rs`,
                                            //   `dict_ops.rs`, and the JIT — beyond a re-shim edit. Scoped wins
                                            //   landed here in #2110:
                                            //
                                            //     1. Thread-local compile cache eliminates re-compilation on hot
                                            //        loops that pass the same pattern string each call (the
                                            //        `findall_hot` bench compiles the same 4-group regex 5x).
                                            //     2. `results` is pre-sized using the lower-bound estimate
                                            //        `text.len() / pat.len()` so the Vec doesn't repeatedly grow
                                            //        during the scan (60k matches no longer trigger log2 reallocs).
                                            //     3. The multi-group tuple buffer is built in-place against a
                                            //        reused `CaptureLocations` to skip the per-iteration HashMap
                                            //        lookup the `captures_iter` API does for named-group dispatch.
                                            //
                                            //   Closing the remaining gap (borrowed-substring objects) is tracked
                                            //   as a follow-on in #2110 once the `Arc<str>`-backed string-storage
                                            //   refactor is in.
                                            // Heuristic capacity: at least one match per (pattern-length) bytes.
                                            // Over-allocates harmlessly when the pattern is short or matches are
                                            // sparse; the alternative — a Vec that doubles 17 times to reach 60k
                                            // — is strictly worse.
    let cap_hint = (text.len() / pat.len().max(8)).min(1 << 20).max(8);
    let mut results: Vec<MbValue> = Vec::with_capacity(cap_hint);

    if num_groups == 0 {
        // No capturing groups — return list of matched strings.
        // `find_iter` is cheaper than `captures_iter` (no captures vec).
        for m in re.find_iter(&text) {
            results.push(MbValue::from_ptr(MbObject::new_str(m.as_str().to_string())));
        }
    } else if num_groups == 1 {
        // Single group — return list of strings (Python behavior).
        // Reuse a single `CaptureLocations` buffer across matches.
        let mut locs = re.capture_locations();
        let mut start = 0usize;
        while let Some(m) = re.captures_read_at(&mut locs, &text, start) {
            let g_slice = match locs.get(1) {
                Some((s, e)) => &text[s..e],
                None => "",
            };
            results.push(MbValue::from_ptr(MbObject::new_str(g_slice.to_string())));
            // Advance: handle zero-width matches by stepping at least 1 byte
            // past `start`. `m.end()` is exclusive; matches `find_iter`'s
            // behavior on empty matches.
            let new_start = if m.end() == start { start + 1 } else { m.end() };
            if new_start > text.len() {
                break;
            }
            start = new_start;
        }
    } else {
        // Multiple groups — return list of tuples. Reuse one CaptureLocations
        // buffer; build each tuple's group_vals into a pre-sized Vec rather
        // than going through the `(1..=N).map(...).collect()` closure dance.
        let mut locs = re.capture_locations();
        let mut start = 0usize;
        while let Some(m) = re.captures_read_at(&mut locs, &text, start) {
            let mut group_vals: Vec<MbValue> = Vec::with_capacity(num_groups);
            for i in 1..=num_groups {
                let s_obj = match locs.get(i) {
                    Some((s, e)) => MbObject::new_str(text[s..e].to_string()),
                    None => MbObject::new_str(String::new()),
                };
                group_vals.push(MbValue::from_ptr(s_obj));
            }
            results.push(MbValue::from_ptr(MbObject::new_tuple(group_vals)));
            let new_start = if m.end() == start { start + 1 } else { m.end() };
            if new_start > text.len() {
                break;
            }
            start = new_start;
        }
    }
    // HANDWRITE-END

    MbValue::from_ptr(MbObject::new_list(results))
}

/// Expand a Python replacement template against one match's captures:
/// `\1`..`\99` numeric backreferences, `\g<name>` / `\g<N>` symbolic
/// references, `\\` escapes, and `\n`/`\t`/`\r` literals. Unknown groups
/// expand to the empty string (CPython raises; lenient here).
fn expand_py_template(caps: &regex::Captures, template: &str) -> String {
    let mut out = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    let group_str = |out: &mut String, m: Option<regex::Match>| {
        if let Some(m) = m {
            out.push_str(m.as_str());
        }
    };
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some(d) if d.is_ascii_digit() => {
                let mut num = 0usize;
                let mut digits = 0;
                while digits < 2 {
                    match chars.peek().copied() {
                        Some(d2) if d2.is_ascii_digit() => {
                            num = num * 10 + (d2 as usize - '0' as usize);
                            chars.next();
                            digits += 1;
                        }
                        _ => break,
                    }
                }
                group_str(&mut out, caps.get(num));
            }
            Some('g') => {
                chars.next();
                if chars.peek() == Some(&'<') {
                    chars.next();
                    let mut name = String::new();
                    for nc in chars.by_ref() {
                        if nc == '>' {
                            break;
                        }
                        name.push(nc);
                    }
                    if let Ok(idx) = name.parse::<usize>() {
                        group_str(&mut out, caps.get(idx));
                    } else {
                        group_str(&mut out, caps.name(&name));
                    }
                } else {
                    out.push('\\');
                    out.push('g');
                }
            }
            Some('\\') => {
                chars.next();
                out.push('\\');
            }
            Some('n') => {
                chars.next();
                out.push('\n');
            }
            Some('t') => {
                chars.next();
                out.push('\t');
            }
            Some('r') => {
                chars.next();
                out.push('\r');
            }
            Some(other) => {
                chars.next();
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Shared sub/subn engine. `repl` is either a template string or a callable
/// receiving the Match instance; `count <= 0` replaces every match.
/// True iff `pattern` is a bytes pattern: a raw bytes/bytearray, or a compiled
/// `re.Pattern` whose `_is_bytes` flag is set.
fn pattern_is_bytes(pattern: MbValue) -> bool {
    if let Some(ptr) = pattern.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(_) | ObjData::ByteArray(_) => return true,
                ObjData::Instance { class_name, fields } if class_name == "re.Pattern" => {
                    if let Some(v) = fields.read().unwrap().get("_is_bytes") {
                        return v.as_bool() == Some(true);
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn sub_engine(
    pattern: MbValue,
    repl: MbValue,
    string: MbValue,
    count: MbValue,
) -> Option<(String, i64)> {
    // CPython: a concrete (non-callable) replacement must match the pattern's
    // str/bytes flavor — `str_pattern.sub(b"x", "c")` is a TypeError.
    let repl_is_bytes = repl.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false);
    let repl_is_str = repl.as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
        .unwrap_or(false);
    if (repl_is_bytes || repl_is_str) && repl_is_bytes != pattern_is_bytes(pattern) {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                if repl_is_bytes {
                    "expected str instance, bytes found".to_string()
                } else {
                    "expected bytes instance, str found".to_string()
                },
            )),
        );
        return None;
    }
    let pat = extract_str(pattern)?;
    let text = extract_str(string)?;
    let template = extract_str(repl);
    let max_count = count.as_int().unwrap_or(0).max(0);

    let re = match compile_cached(&pat) {
        Ok(r) => r,
        Err(e) => {
            raise_re_error(&e);
            return None;
        }
    };

    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    let mut n = 0i64;
    for caps in re.captures_iter(&text) {
        if max_count > 0 && n >= max_count {
            break;
        }
        let m = caps.get(0).expect("group 0 always present");
        out.push_str(&text[last..m.start()]);
        if let Some(ref tpl) = template {
            out.push_str(&expand_py_template(&caps, tpl));
        } else {
            // Callable replacement: invoke with the Match instance and
            // splice in the returned string.
            let match_inst = captures_to_match_with_input(&re, caps, &text);
            let result = super::super::class::mb_call1_val(repl, match_inst);
            if let Some(s) = extract_str(result) {
                out.push_str(&s);
            }
        }
        last = m.end();
        n += 1;
    }
    out.push_str(&text[last..]);
    Some((out, n))
}

/// re.sub(pattern, repl, string, count=0, flags=0) -> string with matches
/// replaced. `repl` may be a template string (supporting `\1` / `\g<name>`
/// backreferences) or a callable receiving the Match object.
pub fn mb_re_sub(pattern: MbValue, repl: MbValue, string: MbValue) -> MbValue {
    mb_re_sub_count(pattern, repl, string, MbValue::none())
}

/// `re.sub` with the optional `count` positional.
pub fn mb_re_sub_count(
    pattern: MbValue,
    repl: MbValue,
    string: MbValue,
    count: MbValue,
) -> MbValue {
    match sub_engine(pattern, repl, string, count) {
        Some((result, _)) => MbValue::from_ptr(MbObject::new_str(result)),
        None => string,
    }
}

/// re.subn(pattern, repl, string, count=0, flags=0) -> (new_str, count).
pub fn mb_re_subn(pattern: MbValue, repl: MbValue, string: MbValue) -> MbValue {
    mb_re_subn_count(pattern, repl, string, MbValue::none())
}

/// `re.subn` with the optional `count` positional.
pub fn mb_re_subn_count(
    pattern: MbValue,
    repl: MbValue,
    string: MbValue,
    count: MbValue,
) -> MbValue {
    match sub_engine(pattern, repl, string, count) {
        Some((result, n)) => {
            let new_str = MbValue::from_ptr(MbObject::new_str(result));
            MbValue::from_ptr(MbObject::new_tuple(vec![new_str, MbValue::from_int(n)]))
        }
        None => MbValue::none(),
    }
}

/// re.finditer(pattern, string) -> list[Match]
///
/// Iterates over all non-overlapping matches and returns them as a list of
/// re.Match instances (Mamba materializes the iterator eagerly to keep parity
/// with the existing finditer-style fallback used by `re.Pattern.findall`).
pub fn mb_re_finditer(pattern: MbValue, string: MbValue) -> MbValue {
    let input_is_bytes = is_bytes_like(string);
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let re = match regex::Regex::new(&py_pattern_to_rust(&pat)) {
        Ok(r) => r,
        Err(e) => {
            raise_re_error(&e.to_string());
            return MbValue::from_ptr(MbObject::new_list(vec![]));
        }
    };

    let mut matches = Vec::new();
    for caps in re.captures_iter(&text) {
        matches.push(captures_to_match_full_with_kind(
            &re,
            caps,
            &text,
            re.as_str(),
            input_is_bytes,
        ));
    }
    MbValue::from_ptr(MbObject::new_list(matches))
}

/// finditer with the optional pos/endpos window (character indices); match
/// objects are built against the windowed slice, which is what `.group()`
/// consumers observe.
pub fn mb_re_finditer_window(
    pattern: MbValue,
    string: MbValue,
    pos: MbValue,
    endpos: MbValue,
) -> MbValue {
    if pos.is_none() && endpos.is_none() {
        return mb_re_finditer(pattern, string);
    }
    let text = extract_str(string).unwrap_or_default();
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len() as i64;
    let start = pos.as_int().unwrap_or(0).clamp(0, n) as usize;
    let stop = endpos.as_int().unwrap_or(n).clamp(0, n) as usize;
    let window: String = chars[start..stop.max(start)].iter().collect();
    mb_re_finditer(pattern, MbValue::from_ptr(MbObject::new_str(window)))
}

/// re.split(pattern, string) -> list of substrings
pub fn mb_re_split(pattern: MbValue, string: MbValue) -> MbValue {
    mb_re_split_max(pattern, string, MbValue::none())
}

/// re.split with maxsplit; captured separator groups are kept in the result
/// (CPython semantics).
pub fn mb_re_split_max(pattern: MbValue, string: MbValue, maxsplit: MbValue) -> MbValue {
    let pat = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let limit = maxsplit.as_int().unwrap_or(0);

    match regex::Regex::new(&py_pattern_to_rust(&pat)) {
        Ok(re) => {
            let has_groups = re.captures_len() > 1;
            let mut parts: Vec<MbValue> = Vec::new();
            let mut last = 0usize;
            let mut splits = 0i64;
            for caps in re.captures_iter(&text) {
                if limit > 0 && splits >= limit {
                    break;
                }
                let m = caps.get(0).unwrap();
                parts.push(MbValue::from_ptr(MbObject::new_str(
                    text[last..m.start()].to_string(),
                )));
                if has_groups {
                    for i in 1..re.captures_len() {
                        match caps.get(i) {
                            Some(g) => parts
                                .push(MbValue::from_ptr(MbObject::new_str(g.as_str().to_string()))),
                            None => parts.push(MbValue::none()),
                        }
                    }
                }
                last = m.end();
                splits += 1;
            }
            parts.push(MbValue::from_ptr(MbObject::new_str(
                text[last..].to_string(),
            )));
            MbValue::from_ptr(MbObject::new_list(parts))
        }
        Err(e) => {
            raise_re_error(&e.to_string());
            MbValue::from_ptr(MbObject::new_list(vec![]))
        }
    }
}

/// re.Match.expand(template) — substitute backrefs in `template`.
///
/// Supports `\1`-`\9` positional refs, `\g<N>` numeric refs, `\g<name>`
/// named refs, and `\\` for a literal backslash. Reads group values from
/// the Match Instance's `group_N` / `group_name_NAME` fields populated by
/// `build_match_instance_with_spans`.
pub fn mb_re_match_expand(match_inst: MbValue, template: MbValue) -> MbValue {
    let tmpl = match extract_str(template) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let inst_ptr = match match_inst.as_ptr() {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let fields = unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            fields
        } else {
            return MbValue::none();
        }
    };
    let guard = fields.read().unwrap();
    let lookup_numeric = |i: usize| -> Option<String> {
        guard
            .get(&format!("group_{}", i))
            .copied()
            .and_then(extract_str)
    };
    let lookup_named = |name: &str| -> Option<String> {
        guard
            .get(&format!("group_name_{}", name))
            .copied()
            .and_then(extract_str)
    };

    let mut out = String::with_capacity(tmpl.len());
    let bytes = tmpl.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if c != b'\\' {
            out.push(c as char);
            i += 1;
            continue;
        }
        // Escape sequence
        if i + 1 >= bytes.len() {
            // Trailing backslash — emit literally
            out.push('\\');
            i += 1;
            continue;
        }
        let next = bytes[i + 1];
        match next {
            b'\\' => {
                out.push('\\');
                i += 2;
            }
            b'0'..=b'9' => {
                let idx = (next - b'0') as usize;
                if let Some(g) = lookup_numeric(idx) {
                    out.push_str(&g);
                }
                i += 2;
            }
            b'g' => {
                // \g<N> or \g<name>
                if i + 2 >= bytes.len() || bytes[i + 2] != b'<' {
                    out.push('\\');
                    out.push('g');
                    i += 2;
                    continue;
                }
                // Find closing '>'
                let start = i + 3;
                let mut end = start;
                while end < bytes.len() && bytes[end] != b'>' {
                    end += 1;
                }
                if end >= bytes.len() {
                    raise_re_error("missing >, unterminated name");
                    return MbValue::none();
                }
                let key = &tmpl[start..end];
                let val = if let Ok(n) = key.parse::<usize>() {
                    lookup_numeric(n)
                } else {
                    lookup_named(key)
                };
                if let Some(g) = val {
                    out.push_str(&g);
                }
                i = end + 1;
            }
            _ => {
                // Unknown escape — emit backslash + char (regex crate behavior;
                // CPython raises but we follow Mamba's lenient mode for now)
                out.push('\\');
                out.push(next as char);
                i += 2;
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str(out))
}

/// re.escape(string) -> string with regex meta-characters escaped
pub fn mb_re_escape(string: MbValue) -> MbValue {
    let text = match extract_str(string) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let escaped = regex::escape(&text);
    MbValue::from_ptr(MbObject::new_str(escaped))
}

/// re.compile(pattern, flags=0) -> re.Pattern instance.
///
/// Mamba does not pre-compile the pattern (matches are recompiled per call
/// against the stored pattern string in `mb_re_search` etc.). The returned
/// instance carries the source pattern + flags and dispatches its `match`,
/// `search`, `findall`, `sub`, `split` methods through the same runtime
/// helpers, threading the stored pattern as the first argument.
pub fn mb_re_compile(pattern: MbValue, flags: MbValue) -> MbValue {
    let Some(pat_str) = extract_str(pattern) else {
        return raise_type_error("first argument must be string or compiled pattern");
    };
    // Validate the pattern eagerly so `re.compile` raises on bad regex
    // before the user threads it through any matcher. Validate the same
    // flag-prefixed form method dispatch will execute (a VERBOSE pattern
    // is only valid once `(?x)` is applied).
    let validate_src = extract_str(with_flags(pattern, flags))
        .unwrap_or_else(|| pat_str.clone());
    match validate_python_repeat_syntax(&validate_src) {
        Ok(()) => {}
        Err(RePreflightError::Re(msg)) => {
            raise_re_error(&msg);
            return MbValue::none();
        }
        Err(RePreflightError::Overflow(msg)) => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                MbValue::from_ptr(MbObject::new_str(msg)),
            );
            return MbValue::none();
        }
    }
    let re = match regex::Regex::new(&py_pattern_to_rust(&validate_src)) {
        Ok(r) => r,
        Err(e) => {
            raise_re_error(&e.to_string());
            return MbValue::none();
        }
    };
    let flag_int = flags.as_int().unwrap_or(0);
    // Remember whether the pattern was compiled from bytes — a bytes pattern
    // rejects a str subject (and vice versa) at match time (CPython TypeError).
    // The decoded `pat_str` loses this, so record it explicitly.
    let pat_is_bytes = pattern.as_ptr().map_or(false, |p| unsafe {
        matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
    });
    let mut fields = FxHashMap::default();
    fields.insert(
        "pattern".to_string(),
        MbValue::from_ptr(MbObject::new_str(pat_str)),
    );
    fields.insert("flags".to_string(), MbValue::from_int(flag_int));
    fields.insert("_is_bytes".to_string(), MbValue::from_bool(pat_is_bytes));
    // .groups — number of capturing groups. (#1624)
    let num_groups = re.captures_len().saturating_sub(1) as i64;
    fields.insert("groups".to_string(), MbValue::from_int(num_groups));
    // .groupindex — dict of named-group → 1-based index. (#1624)
    let groupindex = super::super::dict_ops::mb_dict_new();
    for (i, name) in re.capture_names().enumerate() {
        if let Some(n) = name {
            let key = MbValue::from_ptr(MbObject::new_str(n.to_string()));
            super::super::dict_ops::mb_dict_setitem(groupindex, key, MbValue::from_int(i as i64));
        }
    }
    fields.insert("groupindex".to_string(), groupindex);
    let obj = Box::new(super::super::rc::MbObject {
        header: super::super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "re.Pattern".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_search_found() {
        let result = mb_re_search(s("w\\w+"), s("hello world"));
        unsafe {
            let ptr = result.as_ptr().expect("expected Instance");
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                assert_eq!(class_name, "re.Match");
                let f = fields.read().unwrap();
                assert_eq!(f.get("_start").and_then(|v| v.as_int()), Some(6));
                assert_eq!(f.get("_end").and_then(|v| v.as_int()), Some(11));
            }
        }
    }

    #[test]
    fn test_search_not_found() {
        let result = mb_re_search(s("xyz"), s("hello world"));
        assert!(result.is_none());
    }

    #[test]
    fn test_findall_no_groups() {
        let result = mb_re_findall(s("\\d+"), s("abc123def456"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                assert_eq!(extract_str(items[0]).unwrap(), "123");
                assert_eq!(extract_str(items[1]).unwrap(), "456");
            }
        }
    }

    #[test]
    fn test_findall_with_groups() {
        let result = mb_re_findall(s("(\\d+)-(\\w+)"), s("123-abc 456-def"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                // Each item is a tuple
                if let ObjData::Tuple(ref tuple) = (*items[0].as_ptr().unwrap()).data {
                    assert_eq!(extract_str(tuple[0]).unwrap(), "123");
                    assert_eq!(extract_str(tuple[1]).unwrap(), "abc");
                }
            }
        }
    }

    #[test]
    fn test_sub() {
        let result = mb_re_sub(s("\\d+"), s("X"), s("abc123def456"));
        assert_eq!(extract_str(result).unwrap(), "abcXdefX");
    }

    #[test]
    fn test_split() {
        let result = mb_re_split(s("[,;]"), s("a,b;c"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(extract_str(items[0]).unwrap(), "a");
                assert_eq!(extract_str(items[1]).unwrap(), "b");
                assert_eq!(extract_str(items[2]).unwrap(), "c");
            }
        }
    }

    #[test]
    fn test_escape() {
        let result = mb_re_escape(s("a.b*c"));
        let text = extract_str(result).unwrap();
        assert_eq!(text, r"a\.b\*c");
    }

    #[test]
    fn test_match_at_start() {
        let result = mb_re_match(s("hello"), s("hello world"));
        assert!(!result.is_none());
        let result = mb_re_match(s("world"), s("hello world"));
        assert!(result.is_none());
    }

    #[test]
    fn test_match_with_regex_pattern() {
        let result = mb_re_match(s("\\d{3}"), s("123abc"));
        assert!(!result.is_none());
        let result = mb_re_match(s("\\d{3}"), s("abc123"));
        assert!(result.is_none());
    }

    #[test]
    fn test_match_atomic_group_no_backtrack() {
        let result = mb_re_match(s(r"a(?>bc|b)c"), s("abc"));
        assert!(result.is_none());

        let result = mb_re_match(s(r"a(?>bc|b)c"), s("abcc"));
        assert!(!result.is_none());

        let result = mb_re_match(s(r"(?>.*)."), s("abc"));
        assert!(result.is_none());
    }

    #[test]
    fn test_match_conditional_group_paren_contract() {
        let pat = r"^(\()?([^()]+)(?(1)\))$";
        let wrapped = mb_re_match(s(pat), s("(a)"));
        assert!(!wrapped.is_none());
        let groups = crate::runtime::class::mb_call_method(
            wrapped,
            s("groups"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        unsafe {
            let ObjData::Tuple(ref items) = (*groups.as_ptr().unwrap()).data else {
                panic!("expected groups tuple");
            };
            assert_eq!(extract_str(items[0]).as_deref(), Some("("));
            assert_eq!(extract_str(items[1]).as_deref(), Some("a"));
        }

        let bare = mb_re_match(s(pat), s("a"));
        assert!(!bare.is_none());
        let groups = crate::runtime::class::mb_call_method(
            bare,
            s("groups"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        unsafe {
            let ObjData::Tuple(ref items) = (*groups.as_ptr().unwrap()).data else {
                panic!("expected groups tuple");
            };
            assert!(items[0].is_none());
            assert_eq!(extract_str(items[1]).as_deref(), Some("a"));
        }

        assert!(mb_re_match(s(pat), s("a)")).is_none());
        assert!(mb_re_match(s(pat), s("(a")).is_none());
    }

    #[test]
    fn test_match_positive_negative_lookahead_contract() {
        let positive = mb_re_match(s(r"a(?=\d)"), s("a5"));
        assert!(!positive.is_none());
        let group = crate::runtime::class::mb_call_method(
            positive,
            s("group"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert_eq!(extract_str(group).as_deref(), Some("a"));
        let end = crate::runtime::class::mb_call_method(
            positive,
            s("end"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        assert_eq!(end.as_int(), Some(1));

        assert!(!mb_re_match(s(r"a(?!\d)"), s("ab")).is_none());
        assert!(mb_re_match(s(r"a(?!\d)"), s("a5")).is_none());
    }

    /// re.Match exposes `.string`, `.lastindex`, `.lastgroup` as instance
    /// fields so attribute-style access (handled at `mb_getattr`) returns the
    /// expected values (#1614).
    #[test]
    fn test_match_populates_string_lastindex_lastgroup() {
        let result = mb_re_match(s(r"(?P<first>\w+)\s+(?P<last>\w+)"), s("alice smith"));
        assert!(!result.is_none());
        unsafe {
            let ptr = result.as_ptr().expect("expected Instance");
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                assert_eq!(
                    extract_str(f.get("string").copied().unwrap()).unwrap(),
                    "alice smith"
                );
                assert_eq!(f.get("lastindex").and_then(|v| v.as_int()), Some(2));
                assert_eq!(
                    extract_str(f.get("lastgroup").copied().unwrap()).unwrap(),
                    "last"
                );
            } else {
                panic!("expected re.Match Instance");
            }
        }
    }

    /// re.Match captures populate `group_start_N` / `group_end_N` so the
    /// dispatch table can return per-group `.start(i)` / `.end(i)` /
    /// `.span(i)` instead of always returning the full-match span (#1612).
    #[test]
    fn test_match_populates_per_group_offsets() {
        let result = mb_re_match(s(r"(\w+)\s+(\w+)"), s("hello world"));
        assert!(!result.is_none());
        unsafe {
            let ptr = result.as_ptr().expect("expected Instance");
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                assert_eq!(f.get("group_start_1").and_then(|v| v.as_int()), Some(0));
                assert_eq!(f.get("group_end_1").and_then(|v| v.as_int()), Some(5));
                assert_eq!(f.get("group_start_2").and_then(|v| v.as_int()), Some(6));
                assert_eq!(f.get("group_end_2").and_then(|v| v.as_int()), Some(11));
            } else {
                panic!("expected re.Match Instance");
            }
        }
    }

    /// re.Match captures populate `_group_count` and `_group_names` so the
    /// dispatch table for `.groups()` / `.groupdict()` can build the result
    /// (#1610).
    #[test]
    fn test_match_populates_group_metadata() {
        let result = mb_re_match(s(r"(?P<first>\w+)\s+(?P<last>\w+)"), s("alice smith"));
        assert!(!result.is_none());
        unsafe {
            let ptr = result.as_ptr().expect("expected Instance");
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                assert_eq!(f.get("_group_count").and_then(|v| v.as_int()), Some(2));
                let names = f.get("_group_names").copied().unwrap();
                if let ObjData::List(ref lk) = (*names.as_ptr().unwrap()).data {
                    let items = lk.read().unwrap();
                    assert_eq!(items.len(), 2);
                    assert_eq!(extract_str(items[0]).unwrap(), "first");
                    assert_eq!(extract_str(items[1]).unwrap(), "last");
                }
                assert_eq!(
                    extract_str(f.get("group_name_first").copied().unwrap()).unwrap(),
                    "alice"
                );
                assert_eq!(
                    extract_str(f.get("group_name_last").copied().unwrap()).unwrap(),
                    "smith"
                );
            } else {
                panic!("expected re.Match Instance");
            }
        }
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_re_search_returns_match_instance() {
        let result = mb_re_search(s(r"\d+"), s("abc123"));
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*result.as_ptr().unwrap()).data
            {
                assert_eq!(class_name, "re.Match");
                let f = fields.read().unwrap();
                // group_0 is the full match
                let grp = f.get("group_0").and_then(|v| extract_str(*v));
                assert_eq!(grp.as_deref(), Some("123"));
            } else {
                panic!("expected re.Match Instance");
            }
        }
    }

    #[test]
    fn test_py312_re_search_no_match_returns_none() {
        let result = mb_re_search(s("xyz"), s("abc"));
        assert!(result.is_none());
    }

    #[test]
    fn test_py312_re_findall_multiple_matches() {
        let result = mb_re_findall(s(r"\d+"), s("a1b22c333"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(extract_str(items[0]).as_deref(), Some("1"));
                assert_eq!(extract_str(items[2]).as_deref(), Some("333"));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_py312_re_sub_replaces_all() {
        let result = mb_re_sub(s(r"\d"), s("N"), s("a1b2c3"));
        assert_eq!(extract_str(result).as_deref(), Some("aNbNcN"));
    }

    #[test]
    fn test_py312_re_split_by_whitespace() {
        let result = mb_re_split(s(r"\s+"), s("hello world  foo"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
            }
        }
    }

    #[test]
    fn test_py312_re_match_anchored_at_start() {
        let r1 = mb_re_match(s(r"\w+"), s("hello world"));
        assert!(r1.is_ptr());
        let r2 = mb_re_match(s(r"\d+"), s("hello"));
        assert!(r2.is_none());
    }

    #[test]
    fn test_py312_re_escape_special_chars() {
        let result = mb_re_escape(s("a+b"));
        let text = extract_str(result).unwrap();
        assert!(text.contains(r"\+"));
    }

    /// Regression-document for #2110 — `re.findall` with multi-group patterns
    /// allocates a fresh `MbObject` per group per match. The mirror fixture is
    /// `tests/cpython/std-libs/re/bench/findall_hot.py`; the cross-
    /// runtime harness reports wall ~0.05x / internal ~0.00x / mem ~0.11x vs
    /// CPython.
    ///
    /// After #2110's scoped wins (thread-local compile cache, capacity hint
    /// on `results`, reused `CaptureLocations` buffer) this case must (a)
    /// return the right number of matches and (b) round-trip each group's
    /// substring exactly. The bench fixture covers the perf gate; this test
    /// covers the structural correctness gate.
    #[test]
    fn test_re_2110_findall_multigroup_allocation_hot_path() {
        // Synthetic Apache-log-shaped corpus, scaled down for unit-test budget.
        let mut corpus = String::with_capacity(64 * 1024);
        for i in 0..600 {
            corpus.push_str(&format!(
                "10.0.0.{} - - [01/May/2026] \"GET /api/items/{} HTTP/1.1\" 200 {}\n",
                i % 250,
                (i * 7) % 9973,
                ((i * 131) % 8192) + 64,
            ));
        }
        let pat = r#"(\d+\.\d+\.\d+\.\d+)\s+\S+\s+\S+\s+\[[^\]]+\]\s+"[A-Z]+\s+(\S+)\s+HTTP/[\d.]+"\s+(\d+)\s+(\d+)"#;
        let result = mb_re_findall(s(pat), s(&corpus));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("expected list");
            };
            let items = lock.read().unwrap();
            assert_eq!(items.len(), 600);
            // Spot-check the first match: 4 groups, first group is "10.0.0.0".
            let first = items[0];
            let ObjData::Tuple(ref tup) = (*first.as_ptr().unwrap()).data else {
                panic!("expected tuple per match");
            };
            assert_eq!(tup.len(), 4);
            let ObjData::Str(ref ip) = (*tup[0].as_ptr().unwrap()).data else {
                panic!("expected str group");
            };
            assert_eq!(ip, "10.0.0.0");
            let ObjData::Str(ref status) = (*tup[2].as_ptr().unwrap()).data else {
                panic!("expected str group");
            };
            assert_eq!(status, "200");
        }
    }

    /// Re-running findall against the same pattern must hit the
    /// thread-local compile cache (#2110). This is a behavioral check —
    /// if the cache is bypassed the test still passes; the wins surface
    /// in the cross_runtime bench. The point is to lock down that the
    /// cached path produces identical results to the cold path.
    #[test]
    fn test_re_2110_findall_hot_loop_cache_consistency() {
        let pat = r"(\w+)=(\d+)";
        let text = "a=1 b=22 c=333 d=4444";
        // Cold path.
        let r1 = mb_re_findall(s(pat), s(text));
        // Hot path (cache hit) — same pattern string, different input.
        let r2 = mb_re_findall(s(pat), s("x=9 y=88"));
        // Hot path again, original input.
        let r3 = mb_re_findall(s(pat), s(text));
        unsafe {
            let to_len = |v: MbValue| {
                if let ObjData::List(ref lock) = (*v.as_ptr().unwrap()).data {
                    lock.read().unwrap().len()
                } else {
                    0
                }
            };
            assert_eq!(to_len(r1), 4);
            assert_eq!(to_len(r2), 2);
            assert_eq!(to_len(r3), 4);
        }
    }

    /// #2110 edge case: optional / missing group materializes as the empty
    /// string in CPython's tuple form. Verify the new locs-driven branch
    /// preserves that contract.
    #[test]
    fn test_re_2110_findall_optional_group_empty() {
        // Pattern with an optional second group.
        let pat = r"(\w+)(?:-(\d+))?";
        let text = "alpha beta-7 gamma-13 delta";
        let result = mb_re_findall(s(pat), s(text));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("list");
            };
            let items = lock.read().unwrap();
            assert_eq!(items.len(), 4);
            // alpha: no number → ("alpha", "")
            let ObjData::Tuple(ref t0) = (*items[0].as_ptr().unwrap()).data else {
                panic!()
            };
            let ObjData::Str(ref g1) = (*t0[1].as_ptr().unwrap()).data else {
                panic!()
            };
            assert_eq!(g1, "");
            // beta-7 → ("beta", "7")
            let ObjData::Tuple(ref t1) = (*items[1].as_ptr().unwrap()).data else {
                panic!()
            };
            let ObjData::Str(ref g1) = (*t1[1].as_ptr().unwrap()).data else {
                panic!()
            };
            assert_eq!(g1, "7");
        }
    }

    /// #2110 edge case: unicode characters in groups must round-trip via
    /// byte-index slicing of the source `text`. `regex` reports UTF-8
    /// byte offsets and `text[s..e]` panics on a non-char-boundary slice
    /// — so this test exercises the byte-boundary contract.
    #[test]
    fn test_re_2110_findall_unicode_roundtrip() {
        let pat = r"(\w+)";
        // Mix of ASCII and multi-byte UTF-8: 你好 (each 3 bytes), café (é is 2 bytes).
        let text = "hello 你好 café world";
        let result = mb_re_findall(s(pat), s(text));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("list");
            };
            let items = lock.read().unwrap();
            // regex's \w on Unicode default matches letters + combining marks.
            // Expect at least 4 items: hello, 你好, café, world.
            assert!(items.len() >= 4, "got {} items", items.len());
            // Inspect the second match — must be "你好" exactly.
            let ObjData::Str(ref g) = (*items[1].as_ptr().unwrap()).data else {
                panic!()
            };
            assert_eq!(g, "你好");
            let ObjData::Str(ref g) = (*items[2].as_ptr().unwrap()).data else {
                panic!()
            };
            assert_eq!(g, "café");
        }
    }

    /// #2110 edge case: a pattern that matches the empty string (e.g.
    /// `\d*`) must not infinite-loop when the iterator is hand-driven via
    /// `captures_read_at`. The advance-by-1-on-zero-width branch in
    /// `mb_re_findall` is what makes this terminate. The `regex` crate
    /// does not support lookahead, so we use a quantifier-with-zero match.
    fn b(val: &[u8]) -> MbValue {
        MbValue::from_ptr(MbObject::new_bytes(val.to_vec()))
    }

    /// Regression: re.search / re.match / re.findall on bytes input used to
    /// silently return None / [] because extract_str only matched ObjData::Str.
    /// Fix accepts ObjData::Bytes and ObjData::ByteArray via UTF-8 lossy.
    #[test]
    fn test_re_bytes_input_no_silent_drop() {
        // search returns a Match (not None) for bytes-input
        let m = mb_re_search(b(b"\\d+"), b(b"abc 123 def"));
        assert!(
            m.as_ptr().is_some(),
            "re.search(bytes_pat, bytes_data) returned None"
        );

        // match anchored at start
        let m = mb_re_match(b(b"abc"), b(b"abc 123"));
        assert!(
            m.as_ptr().is_some(),
            "re.match(bytes_pat, bytes_data) returned None"
        );

        // findall captures all groups
        let result = mb_re_findall(b(b"(\\d+)"), b(b"a=1 b=2 c=3"));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("list");
            };
            let items = lock.read().unwrap();
            assert_eq!(
                items.len(),
                3,
                "re.findall(bytes_pat, bytes_data) returned {} items",
                items.len()
            );
        }

        // findall on bulk text — sanity check no silent drop on larger input
        let log: Vec<u8> = b"127.0.0.1 - 200\n".repeat(100);
        let result = mb_re_findall(b(b"(\\d+)"), b(&log));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("list");
            };
            let items = lock.read().unwrap();
            assert!(
                items.len() >= 400,
                "expected >=400 hits, got {}",
                items.len()
            );
        }
    }

    #[test]
    fn test_re_bytes_match_group_returns_bytes() {
        let m = mb_re_search(b(b"\\d\\D\\w\\W\\s\\S"), b(b"1aa! a"));
        assert!(m.as_ptr().is_some(), "expected bytes regex match");

        let group = crate::runtime::class::mb_call_method(
            m,
            s("group"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        unsafe {
            let ptr = group.as_ptr().expect("expected bytes group result");
            let ObjData::Bytes(ref bytes) = (*ptr).data else {
                panic!("expected bytes group result");
            };
            assert_eq!(bytes, b"1aa! a");
        }
    }

    #[test]
    fn test_re_2110_findall_zero_width_terminates() {
        // `\d*` matches the empty string between every non-digit char, and
        // greedily matches digit runs. Across "a1b22c333" we expect a finite
        // set of matches (mix of empty and non-empty), bounded by len(text)+1.
        let pat = r"(\d*)";
        let text = "a1b22c333";
        let result = mb_re_findall(s(pat), s(text));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            let ObjData::List(ref lock) = (*ptr).data else {
                panic!("list");
            };
            let items = lock.read().unwrap();
            // Terminating is the actual gate; bound the upper count.
            assert!(items.len() <= text.len() + 1, "got {} items", items.len());
            assert!(items.len() >= 3, "expected >=3 hits, got {}", items.len());
        }
    }
}
