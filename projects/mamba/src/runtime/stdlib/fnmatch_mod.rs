//! fnmatch module for Mamba (#670).
//!
//! Implements Python 3.12 `fnmatch` stdlib: Unix filename pattern matching
//! using shell-style wildcards. Functions match CPython 3.12 signatures.
//!
//! Pattern syntax:
//!
//! ```text
//!   *       matches any sequence of characters
//!   ?       matches exactly one character
//!   [seq]   matches any character in seq
//!   [!seq]  matches any character NOT in seq
//! ```
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

/// Extract a Rust String from an MbValue that holds a heap string object.
/// Returns None if the value is not a string.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::ObjData;
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
}

/// Classify a string-like argument as `Str` or `Bytes`, mirroring CPython's
/// `fnmatch` which works on both. Bytes are decoded as Latin-1 (ISO-8859-1)
/// so each byte maps to one `char`, exactly like CPython's
/// `str(pat, 'ISO-8859-1')` round-trip in `_compile_pattern`.
enum StrLike {
    Str(String),
    Bytes(String),
}

impl StrLike {
    fn text(&self) -> &str {
        match self {
            StrLike::Str(s) | StrLike::Bytes(s) => s,
        }
    }
    fn is_bytes(&self) -> bool {
        matches!(self, StrLike::Bytes(_))
    }
}

/// Extract a `StrLike` from a str or bytes MbValue. Returns None for any
/// other type.
fn extract_strlike(val: MbValue) -> Option<StrLike> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        match (*ptr).data {
            ObjData::Str(ref s) => Some(StrLike::Str(s.clone())),
            // Latin-1 decode: each byte → one char (0..=255).
            ObjData::Bytes(ref b) => {
                Some(StrLike::Bytes(b.iter().map(|&x| x as char).collect()))
            }
            _ => None,
        }
    })
}

/// Raise a `TypeError` and return None, matching CPython's per-type messages
/// when a str/bytes pattern is applied to the wrong kind of object.
fn raise_mixed_type_error(pat_is_bytes: bool) {
    let msg = if pat_is_bytes {
        "cannot use a bytes pattern on a string-like object"
    } else {
        "cannot use a string pattern on a bytes-like object"
    };
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Perform direct glob matching (no regex crate dependency).
///
/// Supports:
///
/// ```text
///   *       — zero or more characters
///   ?       — exactly one character
///   [seq]   — one character in the set
///   [!seq]  — one character NOT in the set
/// ```
///
/// Uses a simple iterative approach tracking all possible positions
/// in the name string for each pattern segment.
fn glob_match(name: &str, pat: &str) -> bool {
    // ASCII fast path — most fnmatch traffic (filenames, simple patterns)
    // is pure ASCII. Avoiding `chars().collect::<Vec<char>>()` per call
    // saves ~100k allocations on `fnmatch.filter(NAMES, "*.py")` over a
    // 1000-entry list × 100 iters (#1464). Byte-level matching produces
    // identical results for ASCII inputs.
    if name.is_ascii() && pat.is_ascii() {
        return glob_match_bytes(name.as_bytes(), 0, pat.as_bytes(), 0);
    }
    let name_chars: Vec<char> = name.chars().collect();
    let pat_chars: Vec<char> = pat.chars().collect();
    glob_match_inner(&name_chars, 0, &pat_chars, 0)
}

/// ASCII-only byte-level glob matcher. Identical semantics to
/// `glob_match_inner` for ASCII inputs, but operates on `&[u8]` without
/// any heap allocation.
fn glob_match_bytes(name: &[u8], ni: usize, pat: &[u8], pi: usize) -> bool {
    let mut ni = ni;
    let mut pi = pi;

    loop {
        if pi == pat.len() {
            return ni == name.len();
        }

        match pat[pi] {
            b'*' => {
                let mut pi2 = pi + 1;
                while pi2 < pat.len() && pat[pi2] == b'*' {
                    pi2 += 1;
                }
                if pi2 == pat.len() {
                    return true;
                }
                for start in ni..=name.len() {
                    if glob_match_bytes(name, start, pat, pi2) {
                        return true;
                    }
                }
                return false;
            }
            b'?' => {
                if ni >= name.len() {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
            b'[' => {
                if ni >= name.len() {
                    return false;
                }
                let c = name[ni];
                let (matched, consumed) = match_bracket_bytes(&pat[pi..], c);
                if !matched {
                    return false;
                }
                ni += 1;
                pi += consumed;
            }
            lit => {
                if ni >= name.len() || name[ni] != lit {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
        }
    }
}

/// Case-insensitive ASCII byte-level glob matcher. The pattern is
/// expected to already be lowercased by the caller; each `name` byte
/// is folded to lowercase via `to_ascii_lowercase` at compare time, so
/// no `to_lowercase()` allocation is required per name (#1464).
///
/// Retained for the Windows case-insensitive filter fast path; on POSIX
/// (Linux/macOS) `filter` is case-sensitive so this is currently unused.
#[allow(dead_code)]
fn glob_match_bytes_ci(name: &[u8], ni: usize, pat: &[u8], pi: usize) -> bool {
    let mut ni = ni;
    let mut pi = pi;

    loop {
        if pi == pat.len() {
            return ni == name.len();
        }

        match pat[pi] {
            b'*' => {
                let mut pi2 = pi + 1;
                while pi2 < pat.len() && pat[pi2] == b'*' {
                    pi2 += 1;
                }
                if pi2 == pat.len() {
                    return true;
                }
                for start in ni..=name.len() {
                    if glob_match_bytes_ci(name, start, pat, pi2) {
                        return true;
                    }
                }
                return false;
            }
            b'?' => {
                if ni >= name.len() {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
            b'[' => {
                if ni >= name.len() {
                    return false;
                }
                let c = name[ni].to_ascii_lowercase();
                let (matched, consumed) = match_bracket_bytes(&pat[pi..], c);
                if !matched {
                    return false;
                }
                ni += 1;
                pi += consumed;
            }
            lit => {
                if ni >= name.len() || name[ni].to_ascii_lowercase() != lit {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
        }
    }
}

fn match_bracket_bytes(pat: &[u8], c: u8) -> (bool, usize) {
    let mut i = 1;
    let negate = i < pat.len() && pat[i] == b'!';
    if negate {
        i += 1;
    }
    let mut in_set = false;
    let mut first = true;

    while i < pat.len() {
        if pat[i] == b']' && !first {
            let consumed = i + 1;
            let matched = if negate { !in_set } else { in_set };
            return (matched, consumed);
        }
        if i + 2 < pat.len() && pat[i + 1] == b'-' && pat[i + 2] != b']' {
            let lo = pat[i];
            let hi = pat[i + 2];
            if c >= lo && c <= hi {
                in_set = true;
            }
            i += 3;
        } else {
            if pat[i] == c {
                in_set = true;
            }
            i += 1;
        }
        first = false;
    }

    let matched = c == b'[';
    (matched, 1)
}

fn glob_match_inner(name: &[char], ni: usize, pat: &[char], pi: usize) -> bool {
    let mut ni = ni;
    let mut pi = pi;

    loop {
        if pi == pat.len() {
            return ni == name.len();
        }

        match pat[pi] {
            '*' => {
                // Skip consecutive stars.
                let mut pi2 = pi + 1;
                while pi2 < pat.len() && pat[pi2] == '*' {
                    pi2 += 1;
                }
                // Star at end matches everything remaining.
                if pi2 == pat.len() {
                    return true;
                }
                // Try matching the rest of the pattern at every position.
                for start in ni..=name.len() {
                    if glob_match_inner(name, start, pat, pi2) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if ni >= name.len() {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
            '[' => {
                if ni >= name.len() {
                    return false;
                }
                let c = name[ni];
                // Parse the bracket expression.
                let (matched, consumed) = match_bracket(&pat[pi..], c);
                if !matched {
                    return false;
                }
                ni += 1;
                pi += consumed;
            }
            lit => {
                if ni >= name.len() || name[ni] != lit {
                    return false;
                }
                ni += 1;
                pi += 1;
            }
        }
    }
}

/// Parse a bracket expression `[...]` starting at pat[0] == '['.
/// Returns (matched, chars_consumed_from_pat).
/// If the bracket is malformed (no closing `]`), treat `[` as literal.
fn match_bracket(pat: &[char], c: char) -> (bool, usize) {
    // pat[0] == '['
    let mut i = 1;
    let negate = i < pat.len() && pat[i] == '!';
    if negate {
        i += 1;
    }

    // Allow `]` as first char in set (e.g. `[]]` matches `]`).
    let mut in_set = false;
    let mut first = true;
    let start = i;

    while i < pat.len() {
        if pat[i] == ']' && !first {
            // Found closing bracket.
            let consumed = i + 1; // include the ']'
            let matched = if negate { !in_set } else { in_set };
            return (matched, consumed);
        }

        // Check for range: x-y
        if i + 2 < pat.len() && pat[i + 1] == '-' && pat[i + 2] != ']' {
            let lo = pat[i];
            let hi = pat[i + 2];
            if c >= lo && c <= hi {
                in_set = true;
            }
            i += 3;
        } else {
            if pat[i] == c {
                in_set = true;
            }
            i += 1;
        }
        first = false;
        let _ = start; // suppress unused warning
    }

    // Malformed bracket — treat '[' as literal.
    let matched = c == '[';
    (matched, 1)
}

/// CPython `re.escape` for a single character: escape with a leading
/// backslash exactly those characters in the 3.12 `_special_chars_map`
/// set. ASCII alphanumerics, `_`, and other non-special characters pass
/// through unchanged.
///
/// Special set (3.12): `\t \n \x0b \x0c \r (space) # $ & ( ) * + - . ? [ \ ] ^ { | } ~`.
fn re_escape_char(c: char, out: &mut String) {
    const SPECIAL: &[char] = &[
        '\t', '\n', '\u{0b}', '\u{0c}', '\r', ' ', '#', '$', '&', '(', ')', '*',
        '+', '-', '.', '?', '[', '\\', ']', '^', '{', '|', '}', '~',
    ];
    if SPECIAL.contains(&c) {
        out.push('\\');
    }
    out.push(c);
}

/// A token of the intermediate translation: either a literal regex fragment
/// or a `*` placeholder (which becomes `.*` / an atomic group later).
enum Tok {
    Star,
    Frag(String),
}

/// Build the bracket-expression regex fragment for `stuff` (the raw text
/// between `[` and `]`), mirroring CPython 3.12 `fnmatch.translate`.
fn translate_bracket(stuff_chars: &[char]) -> String {
    // stuff_chars is the content between [ and ] (exclusive).
    let stuff: String = stuff_chars.iter().collect();

    let processed: String = if !stuff.contains('-') {
        // No ranges: just double backslashes.
        stuff.replace('\\', "\\\\")
    } else {
        // Split into chunks at hyphens that form ranges, fix up empty
        // ranges, then escape backslashes and hyphens.
        let n = stuff_chars.len();
        let mut chunks: Vec<Vec<char>> = Vec::new();
        // i indexes into stuff_chars; mirror CPython's i/k logic where
        // i starts at the first content char (CPython uses pat[i:j] with
        // i pointing past `[`; here stuff is already that slice so i=0).
        let mut i: usize = 0;
        // k = 2 if first char is '!' else 1 (skip a leading negation marker;
        // however by this point negation is handled by caller via stuff[0]=='!'
        // — CPython computes k off the original pat, but since stuff begins at
        // the char after '[', the first char may be '!').
        let mut k: usize = if !stuff_chars.is_empty() && stuff_chars[0] == '!' {
            2
        } else {
            1
        };
        loop {
            // Find next '-' in stuff_chars[k..n].
            let mut found: Option<usize> = None;
            let mut s = k;
            while s < n {
                if stuff_chars[s] == '-' {
                    found = Some(s);
                    break;
                }
                s += 1;
            }
            match found {
                None => break,
                Some(kk) => {
                    chunks.push(stuff_chars[i..kk].to_vec());
                    i = kk + 1;
                    k = kk + 3;
                }
            }
        }
        let last_chunk = stuff_chars[i..n].to_vec();
        if !last_chunk.is_empty() {
            chunks.push(last_chunk);
        } else if let Some(lastc) = chunks.last_mut() {
            lastc.push('-');
        }
        // Remove empty ranges -- invalid in RE.
        let mut ci = chunks.len();
        while ci > 1 {
            let idx = ci - 1;
            let prev_last = *chunks[idx - 1].last().unwrap();
            let cur_first = chunks[idx][0];
            if prev_last > cur_first {
                // Merge: chunks[idx-1] = chunks[idx-1][:-1] + chunks[idx][1:]
                chunks[idx - 1].pop();
                let tail: Vec<char> = chunks[idx][1..].to_vec();
                chunks[idx - 1].extend(tail);
                chunks.remove(idx);
            }
            ci -= 1;
        }
        // Escape backslashes and hyphens, join with '-'.
        let escaped: Vec<String> = chunks
            .iter()
            .map(|chunk| {
                let s: String = chunk.iter().collect();
                s.replace('\\', "\\\\").replace('-', "\\-")
            })
            .collect();
        escaped.join("-")
    };

    // Escape set operations (&&, ~~ and ||): each of & ~ | gets a backslash.
    let mut stuff2 = String::with_capacity(processed.len());
    for c in processed.chars() {
        if c == '&' || c == '~' || c == '|' {
            stuff2.push('\\');
        }
        stuff2.push(c);
    }

    if stuff2.is_empty() {
        // Empty range: never match.
        "(?!)".to_string()
    } else if stuff2 == "!" {
        // Negated empty range: match any character.
        ".".to_string()
    } else {
        let first = stuff2.chars().next().unwrap();
        let body = if first == '!' {
            format!("^{}", &stuff2[1..])
        } else if first == '^' || first == '[' {
            format!("\\{}", stuff2)
        } else {
            stuff2
        };
        format!("[{}]", body)
    }
}

/// Translate a glob pattern to a Python-compatible regex string,
/// faithful to CPython 3.12 `fnmatch.translate`.
///
/// ```text
///   *       → `.*` (last/trailing) or atomic group `(?>.*?fixed)` (interior)
///   ?       → `.`
///   [seq]   → bracket expression (with caret/`]`/range/set-op handling)
///   [!seq]  → `[^seq]`
///   other   → re.escape(c)
/// ```
///
/// Output is wrapped as `(?s:{translated})\Z`.
fn translate_pattern(pat: &str) -> String {
    let chars: Vec<char> = pat.chars().collect();
    let n = chars.len();
    let mut toks: Vec<Tok> = Vec::new();
    let mut i = 0;

    while i < n {
        let c = chars[i];
        i += 1;
        if c == '*' {
            // Compress consecutive `*` into one Star token.
            if !matches!(toks.last(), Some(Tok::Star)) {
                toks.push(Tok::Star);
            }
        } else if c == '?' {
            toks.push(Tok::Frag(".".to_string()));
        } else if c == '[' {
            let mut j = i;
            if j < n && chars[j] == '!' {
                j += 1;
            }
            if j < n && chars[j] == ']' {
                j += 1;
            }
            while j < n && chars[j] != ']' {
                j += 1;
            }
            if j >= n {
                toks.push(Tok::Frag("\\[".to_string()));
            } else {
                let stuff = &chars[i..j];
                let frag = translate_bracket(stuff);
                toks.push(Tok::Frag(frag));
                i = j + 1;
            }
        } else {
            let mut frag = String::new();
            re_escape_char(c, &mut frag);
            toks.push(Tok::Frag(frag));
        }
    }

    // Deal with STARs (atomic groups for interior `*fixed`).
    let mut res = String::new();
    let m = toks.len();
    let mut idx = 0;
    // Fixed pieces at the start.
    while idx < m {
        match &toks[idx] {
            Tok::Star => break,
            Tok::Frag(f) => {
                res.push_str(f);
                idx += 1;
            }
        }
    }
    // STAR fixed STAR fixed ...
    while idx < m {
        // toks[idx] is Star.
        idx += 1;
        if idx == m {
            res.push_str(".*");
            break;
        }
        // Collect the run of fixed frags after this star.
        let mut fixed = String::new();
        while idx < m {
            match &toks[idx] {
                Tok::Star => break,
                Tok::Frag(f) => {
                    fixed.push_str(f);
                    idx += 1;
                }
            }
        }
        if idx == m {
            res.push_str(".*");
            res.push_str(&fixed);
        } else {
            res.push_str("(?>.*?");
            res.push_str(&fixed);
            res.push(')');
        }
    }

    format!("(?s:{})\\Z", res)
}

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

dispatch_binary!(dispatch_fnmatch, mb_fnmatch_fnmatch);
dispatch_binary!(dispatch_fnmatchcase, mb_fnmatch_fnmatchcase);
dispatch_binary!(dispatch_filter, mb_fnmatch_filter);
dispatch_unary!(dispatch_translate, mb_fnmatch_translate);

/// Register the fnmatch module in the stdlib registry.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("fnmatch", dispatch_fnmatch as usize),
        ("fnmatchcase", dispatch_fnmatchcase as usize),
        ("filter", dispatch_filter as usize),
        ("translate", dispatch_translate as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("fnmatch", attrs);
}

// ── Public runtime functions ──

/// Apply CPython's `os.path.normcase` to a glob name/pattern string.
///
/// On POSIX (Linux + macOS) `normcase` is the identity, so matching is
/// case-sensitive and path separators are not rewritten. Only on Windows
/// does `ntpath.normcase` lowercase and convert `/` to `\`. This mirrors
/// CPython 3.12, where `fnmatch.fnmatch` is case-SENSITIVE on macOS.
#[allow(clippy::needless_return)]
fn normcase(s: &str) -> std::borrow::Cow<'_, str> {
    #[cfg(target_os = "windows")]
    {
        return std::borrow::Cow::Owned(s.replace('/', "\\").to_lowercase());
    }
    #[cfg(not(target_os = "windows"))]
    {
        return std::borrow::Cow::Borrowed(s);
    }
}

/// fnmatch.fnmatch(name, pat) -> bool
///
/// Test whether the filename NAME matches the pattern PAT, applying
/// `os.path.normcase` to both first (CPython 3.12 semantics: case-sensitive
/// on POSIX/macOS, case-insensitive on Windows). Raises `TypeError` when
/// `name` and `pat` mix str and bytes.
pub fn mb_fnmatch_fnmatch(name: MbValue, pat: MbValue) -> MbValue {
    let (name_sl, pat_sl) = match (extract_strlike(name), extract_strlike(pat)) {
        (Some(n), Some(p)) => (n, p),
        _ => return MbValue::from_bool(false),
    };
    if name_sl.is_bytes() != pat_sl.is_bytes() {
        raise_mixed_type_error(pat_sl.is_bytes());
        return MbValue::none();
    }
    // normcase only affects str/Windows; bytes are not case-folded.
    let n = if pat_sl.is_bytes() {
        std::borrow::Cow::Borrowed(name_sl.text())
    } else {
        normcase(name_sl.text())
    };
    let p = if pat_sl.is_bytes() {
        std::borrow::Cow::Borrowed(pat_sl.text())
    } else {
        normcase(pat_sl.text())
    };
    MbValue::from_bool(glob_match(&n, &p))
}

/// fnmatch.fnmatchcase(name, pat) -> bool
///
/// Test whether the filename NAME matches the pattern PAT (always
/// case-sensitive; no normcase). Raises `TypeError` on mixed str/bytes.
pub fn mb_fnmatch_fnmatchcase(name: MbValue, pat: MbValue) -> MbValue {
    let (name_sl, pat_sl) = match (extract_strlike(name), extract_strlike(pat)) {
        (Some(n), Some(p)) => (n, p),
        _ => return MbValue::from_bool(false),
    };
    if name_sl.is_bytes() != pat_sl.is_bytes() {
        raise_mixed_type_error(pat_sl.is_bytes());
        return MbValue::none();
    }
    MbValue::from_bool(glob_match(name_sl.text(), pat_sl.text()))
}

/// fnmatch.filter(names, pat) -> list
///
/// Return a list of those elements of NAMES that match PAT. Mirrors
/// CPython 3.12: applies `os.path.normcase` to the pattern and (on
/// non-POSIX) each name; on POSIX (Linux/macOS) matching is
/// case-sensitive. Supports both str and bytes; mixing str and bytes
/// between names and pattern raises `TypeError`.
///
/// Perf note (#1464): for each match we retain the input MbValue
/// directly rather than allocating a fresh `MbObject::new_str` — the
/// input is already a heap string, so reusing its pointer saves one
/// alloc + one `String` clone per matched element.
pub fn mb_fnmatch_filter(names: MbValue, pat: MbValue) -> MbValue {
    let pat_sl = match extract_strlike(pat) {
        Some(p) => p,
        None => return MbValue::from_ptr(MbObject::new_list(Vec::new())),
    };
    let pat_is_bytes = pat_sl.is_bytes();
    // normcase the pattern (identity on POSIX; lower + sep-swap on Windows).
    // bytes patterns are never case-folded by normcase.
    let pat_normalized: String = if pat_is_bytes {
        pat_sl.text().to_string()
    } else {
        normcase(pat_sl.text()).into_owned()
    };

    use super::super::rc::{ObjData, retain_if_ptr};
    let ptr = match names.as_ptr() {
        Some(p) => p,
        None => return MbValue::from_ptr(MbObject::new_list(Vec::new())),
    };
    let collected: Result<Vec<MbValue>, ()> = unsafe {
        if let ObjData::List(ref rw) = (*ptr).data {
            let guard = match rw.read() {
                Ok(g) => g,
                Err(_) => return MbValue::from_ptr(MbObject::new_list(Vec::new())),
            };
            let mut results: Vec<MbValue> = Vec::with_capacity(guard.len());
            for v in guard.iter() {
                let v = *v;
                let name_sl = match extract_strlike(v) {
                    Some(n) => n,
                    None => continue,
                };
                // Mixing str/bytes between a name and the pattern is a
                // TypeError, exactly as CPython raises during matching.
                if name_sl.is_bytes() != pat_is_bytes {
                    return {
                        drop(guard);
                        raise_mixed_type_error(pat_is_bytes);
                        MbValue::none()
                    };
                }
                let nname: String = if pat_is_bytes {
                    name_sl.text().to_string()
                } else {
                    normcase(name_sl.text()).into_owned()
                };
                if glob_match(&nname, &pat_normalized) {
                    retain_if_ptr(v);
                    results.push(v);
                }
            }
            Ok(results)
        } else {
            Err(())
        }
    };

    match collected {
        Ok(results) => MbValue::from_ptr(MbObject::new_list(results)),
        Err(()) => MbValue::from_ptr(MbObject::new_list(Vec::new())),
    }
}

/// fnmatch.translate(pat) -> str
///
/// Translate a shell PATTERN to a regular expression (CPython 3.12 compatible).
pub fn mb_fnmatch_translate(pat: MbValue) -> MbValue {
    let pat_s = extract_str(pat).unwrap_or_default();
    let regex_str = translate_pattern(&pat_s);
    MbValue::from_ptr(MbObject::new_str(regex_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn get_str(v: MbValue) -> String {
        extract_str(v).unwrap_or_default()
    }

    // REQ: R1, R2
    #[test]
    fn test_fnmatch_basic() {
        // "foo.txt" should match "*.txt"
        let name = make_str("foo.txt");
        let pat = make_str("*.txt");
        let result = mb_fnmatch_fnmatch(name, pat);
        assert_eq!(result.as_bool(), Some(true), "fnmatch('foo.txt', '*.txt') must be true");
    }

    // REQ: R2
    #[test]
    fn test_fnmatch_no_match() {
        // "foo.rs" should NOT match "*.txt"
        let name = make_str("foo.rs");
        let pat = make_str("*.txt");
        let result = mb_fnmatch_fnmatch(name, pat);
        assert_eq!(result.as_bool(), Some(false), "fnmatch('foo.rs', '*.txt') must be false");
    }

    // REQ: R2
    #[test]
    fn test_fnmatchcase_case_sensitive() {
        // fnmatchcase is always case-sensitive.
        let name_lower = make_str("foo.txt");
        let pat_upper = make_str("*.TXT");
        let result = mb_fnmatch_fnmatchcase(name_lower, pat_upper);
        assert_eq!(
            result.as_bool(),
            Some(false),
            "fnmatchcase('foo.txt', '*.TXT') must be false (case-sensitive)"
        );

        // Same case should match.
        let name2 = make_str("foo.TXT");
        let pat2 = make_str("*.TXT");
        let result2 = mb_fnmatch_fnmatchcase(name2, pat2);
        assert_eq!(
            result2.as_bool(),
            Some(true),
            "fnmatchcase('foo.TXT', '*.TXT') must be true"
        );
    }

    // REQ: R2
    #[test]
    fn test_filter_list() {
        // Build a list: ["foo.txt", "bar.rs", "baz.txt"]
        let elems = vec![
            make_str("foo.txt"),
            make_str("bar.rs"),
            make_str("baz.txt"),
        ];
        let names = MbValue::from_ptr(MbObject::new_list(elems));
        let pat = make_str("*.txt");
        let result = mb_fnmatch_filter(names, pat);

        // Result must be a list with 2 items.
        let count = result
            .as_ptr()
            .map(|ptr| unsafe {
                use super::super::super::rc::ObjData;
                if let ObjData::List(ref rw) = (*ptr).data {
                    rw.read().map(|g| g.len()).unwrap_or(0)
                } else {
                    0
                }
            })
            .unwrap_or(0);
        assert_eq!(count, 2, "filter should return 2 matching items");
    }

    // REQ: R2, R4
    #[test]
    fn test_translate_star() {
        // translate("*.txt") should produce a regex containing ".*"
        let pat = make_str("*.txt");
        let result = mb_fnmatch_translate(pat);
        let s = get_str(result);
        assert!(s.contains(".*"), "translate('*.txt') must contain '.*', got: {}", s);
        assert!(s.starts_with("(?s:"), "translate result must start with '(?s:': {}", s);
        assert!(s.ends_with(r")\Z"), r"translate result must end with ')\Z': {}", s);
    }

    // REQ: R2, R4
    #[test]
    fn test_translate_question_mark() {
        // translate("f?o") should produce (?s:f.o)\Z — ? becomes . in the inner pattern.
        let pat = make_str("f?o");
        let result = mb_fnmatch_translate(pat);
        let s = get_str(result);
        // The full output is (?s:f.o)\Z — extract the inner pattern between (?s: and )\Z.
        let inner = s
            .strip_prefix("(?s:")
            .and_then(|s| s.strip_suffix(r")\Z"))
            .unwrap_or(&s);
        assert_eq!(inner, "f.o", "translate('f?o') inner pattern must be 'f.o', got inner: {}", inner);
        // The inner pattern must not contain a literal '?'.
        assert!(!inner.contains('?'), "inner pattern must not contain literal '?': {}", inner);
    }

    // REQ: R2, R4
    #[test]
    fn test_fnmatchcase_question_mark() {
        // '?' matches exactly one character.
        let name = make_str("foo");
        let pat = make_str("f?o");
        let result = mb_fnmatch_fnmatchcase(name, pat);
        assert_eq!(result.as_bool(), Some(true), "fnmatchcase('foo', 'f?o') must be true");

        let name2 = make_str("fo");
        let pat2 = make_str("f?o");
        let result2 = mb_fnmatch_fnmatchcase(name2, pat2);
        assert_eq!(result2.as_bool(), Some(false), "fnmatchcase('fo', 'f?o') must be false");
    }

    // REQ: R4
    #[test]
    fn test_translate_bracket_negation() {
        // translate("[!abc]") should become "[^abc]"
        let pat = make_str("[!abc]");
        let result = mb_fnmatch_translate(pat);
        let s = get_str(result);
        assert!(s.contains("[^abc]"), "translate('[!abc]') must produce '[^abc]', got: {}", s);
    }
}
