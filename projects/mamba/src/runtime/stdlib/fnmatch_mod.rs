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
use super::super::rc::MbObject;
use super::super::value::MbValue;
use std::collections::HashMap;

/// Extract a Rust String from an MbValue that holds a heap string object.
/// Returns None if the value is not a string.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
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

/// Translate a glob pattern to a Python-compatible regex string.
///
/// CPython 3.12 behavior:
///
/// ```text
///   *       → `.*`  (wrapped in `(?s:...)\Z` overall)
///   ?       → `.`
///   [seq]   → `[seq]`
///   [!seq]  → `[^seq]`
///   other   → regex-escaped
/// ```
///
/// Output is wrapped as `(?s:{translated})\Z` matching CPython 3.12.
fn translate_pattern(pat: &str) -> String {
    let chars: Vec<char> = pat.chars().collect();
    let mut result = String::from("(?s:");
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '*' => {
                result.push_str(".*");
                i += 1;
            }
            '?' => {
                result.push('.');
                i += 1;
            }
            '[' => {
                // Find closing bracket.
                let mut j = i + 1;
                // Allow ] as first char in bracket expression.
                if j < chars.len() && chars[j] == '!' {
                    j += 1;
                }
                if j < chars.len() && chars[j] == ']' {
                    j += 1;
                }
                while j < chars.len() && chars[j] != ']' {
                    j += 1;
                }
                if j < chars.len() {
                    // Valid bracket expression [i..=j].
                    result.push('[');
                    let inner_start = i + 1;
                    // Replace leading ! with ^ for negation.
                    if inner_start < chars.len() && chars[inner_start] == '!' {
                        result.push('^');
                        for k in (inner_start + 1)..j {
                            result.push(chars[k]);
                        }
                    } else {
                        for k in inner_start..j {
                            result.push(chars[k]);
                        }
                    }
                    result.push(']');
                    i = j + 1;
                } else {
                    // Malformed bracket — escape the [.
                    result.push_str("\\[");
                    i += 1;
                }
            }
            c => {
                // Regex-escape special characters.
                if r"\.+^${}()|".contains(c) {
                    result.push('\\');
                }
                result.push(c);
                i += 1;
            }
        }
    }

    result.push_str(")\\Z");
    result
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

/// fnmatch.fnmatch(name, pat) -> bool
///
/// Test whether the filename NAME matches the pattern PAT.
/// Case sensitivity is OS-dependent: case-insensitive on macOS/Windows,
/// case-sensitive on Linux (mirrors CPython 3.12 behavior).
pub fn mb_fnmatch_fnmatch(name: MbValue, pat: MbValue) -> MbValue {
    let name_s = extract_str(name).unwrap_or_default();
    let pat_s = extract_str(pat).unwrap_or_default();

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let (n, p) = (name_s.to_lowercase(), pat_s.to_lowercase());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let (n, p) = (name_s, pat_s);

    MbValue::from_bool(glob_match(&n, &p))
}

/// fnmatch.fnmatchcase(name, pat) -> bool
///
/// Test whether the filename NAME matches the pattern PAT (always case-sensitive).
pub fn mb_fnmatch_fnmatchcase(name: MbValue, pat: MbValue) -> MbValue {
    let name_s = extract_str(name).unwrap_or_default();
    let pat_s = extract_str(pat).unwrap_or_default();
    MbValue::from_bool(glob_match(&name_s, &pat_s))
}

/// fnmatch.filter(names, pat) -> list
///
/// Return a list of those elements of NAMES that match PAT.
/// Uses case-insensitive matching on macOS/Windows, case-sensitive on Linux.
///
/// Perf note (#1464): for each match we retain the input MbValue
/// directly rather than allocating a fresh `MbObject::new_str` — the
/// input is already a heap string, so reusing its pointer saves one
/// alloc + one `String` clone per matched element.
pub fn mb_fnmatch_filter(names: MbValue, pat: MbValue) -> MbValue {
    let pat_s = extract_str(pat).unwrap_or_default();
    // Case-fold the pattern once (macOS/Windows). The per-name path uses
    // a byte-level ASCII case-insensitive matcher so each match call no
    // longer allocates a lowercase copy of the name (#1464).
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    let pat_normalized = pat_s.to_lowercase();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let pat_normalized = pat_s;

    let pat_is_ascii = pat_normalized.is_ascii();
    let pat_bytes = pat_normalized.as_bytes();

    let matched: Vec<MbValue> = names
        .as_ptr()
        .and_then(|ptr| unsafe {
            use super::super::rc::{retain_if_ptr, ObjData};
            if let ObjData::List(ref rw) = (*ptr).data {
                let guard = rw.read().ok()?;
                let mut results: Vec<MbValue> = Vec::with_capacity(guard.len());
                for v in guard.iter() {
                    let v = *v;
                    let s_ptr = match v.as_ptr() {
                        Some(p) => p,
                        None => continue,
                    };
                    let s_ref: &str = match (*s_ptr).data {
                        ObjData::Str(ref s) => s.as_str(),
                        _ => continue,
                    };

                    let matched_one;
                    #[cfg(any(target_os = "macos", target_os = "windows"))]
                    {
                        // ASCII fast path — no allocation. Folds names
                        // case-insensitively against an already-lowercased
                        // pattern via per-byte `to_ascii_lowercase`.
                        if pat_is_ascii && s_ref.is_ascii() {
                            matched_one = glob_match_bytes_ci(s_ref.as_bytes(), 0, pat_bytes, 0);
                        } else {
                            let sn = s_ref.to_lowercase();
                            matched_one = glob_match(&sn, &pat_normalized);
                        }
                    }
                    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
                    {
                        matched_one = glob_match(s_ref, &pat_normalized);
                        let _ = pat_is_ascii;
                        let _ = pat_bytes;
                    }

                    if matched_one {
                        retain_if_ptr(v);
                        results.push(v);
                    }
                }
                Some(results)
            } else {
                None
            }
        })
        .unwrap_or_default();

    MbValue::from_ptr(MbObject::new_list(matched))
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
        assert_eq!(
            result.as_bool(),
            Some(true),
            "fnmatch('foo.txt', '*.txt') must be true"
        );
    }

    // REQ: R2
    #[test]
    fn test_fnmatch_no_match() {
        // "foo.rs" should NOT match "*.txt"
        let name = make_str("foo.rs");
        let pat = make_str("*.txt");
        let result = mb_fnmatch_fnmatch(name, pat);
        assert_eq!(
            result.as_bool(),
            Some(false),
            "fnmatch('foo.rs', '*.txt') must be false"
        );
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
        let elems = vec![make_str("foo.txt"), make_str("bar.rs"), make_str("baz.txt")];
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
        assert!(
            s.contains(".*"),
            "translate('*.txt') must contain '.*', got: {}",
            s
        );
        assert!(
            s.starts_with("(?s:"),
            "translate result must start with '(?s:': {}",
            s
        );
        assert!(
            s.ends_with(r")\Z"),
            r"translate result must end with ')\Z': {}",
            s
        );
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
        assert_eq!(
            inner, "f.o",
            "translate('f?o') inner pattern must be 'f.o', got inner: {}",
            inner
        );
        // The inner pattern must not contain a literal '?'.
        assert!(
            !inner.contains('?'),
            "inner pattern must not contain literal '?': {}",
            inner
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_fnmatchcase_question_mark() {
        // '?' matches exactly one character.
        let name = make_str("foo");
        let pat = make_str("f?o");
        let result = mb_fnmatch_fnmatchcase(name, pat);
        assert_eq!(
            result.as_bool(),
            Some(true),
            "fnmatchcase('foo', 'f?o') must be true"
        );

        let name2 = make_str("fo");
        let pat2 = make_str("f?o");
        let result2 = mb_fnmatch_fnmatchcase(name2, pat2);
        assert_eq!(
            result2.as_bool(),
            Some(false),
            "fnmatchcase('fo', 'f?o') must be false"
        );
    }

    // REQ: R4
    #[test]
    fn test_translate_bracket_negation() {
        // translate("[!abc]") should become "[^abc]"
        let pat = make_str("[!abc]");
        let result = mb_fnmatch_translate(pat);
        let s = get_str(result);
        assert!(
            s.contains("[^abc]"),
            "translate('[!abc]') must produce '[^abc]', got: {}",
            s
        );
    }
}
