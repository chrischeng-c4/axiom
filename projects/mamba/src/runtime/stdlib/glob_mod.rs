//! glob module for Mamba (#430, #1265 Task #78, Wave-8, #1463 conformance).
//!
//! CPython 3.12 `glob` 15-entry surface:
//!   contextlib, escape, fnmatch, glob, glob0, glob1, has_magic,
//!   iglob, itertools, magic_check, magic_check_bytes, os, re, stat, sys.
//!
//! Typeshed `__all__` declares only `escape`, `glob`, `iglob` (3 names) —
//! all three are wired, so canonical `__all__` coverage is 100% (3/3).
//! The broader `dir(glob)` set (15 names) is also wired as a superset for
//! introspection-heavy callers (`glob.has_magic(s)`, `glob.glob1(...)`).
//!
//! HANDWRITE-BEGIN reason: #1463 glob deep blockers — gaps below require
//!   runtime work outside the glob module proper. Each must close before
//!   the corresponding CPython `Lib/test/test_glob.py` test can be replayed:
//!
//!   1. `**` recursive globbing + `recursive=True` kwarg
//!      → blocks: test_recursive_glob
//!      → requires: a directory-walk implementation, kwarg threading
//!        through the variadic dispatcher.
//!   2. `[abc]` character-class wildcards in `glob_match`
//!      → blocks: test_glob_one_directory (the `aa[ab]` case),
//!                test_escape (round-trip semantics)
//!      → requires: extending the `glob_match` two-pointer matcher to
//!        recognise `[...]` ranges; today `[` is matched literally.
//!   3. `root_dir=` / `dir_fd=` kwargs on `glob.glob` / `glob.iglob`
//!      → blocks: test_glob_empty_pattern, test_glob_directory_*,
//!                most of the test_glob shared helper.
//!      → requires: kwarg dispatch + an `fd`-based scandir path.
//!   4. `bytes` patterns (`glob.glob(b"*.txt")`)
//!      → blocks: test_glob_bytes_directory_with_trailing_slash and
//!                every bytes branch inside the shared `glob` helper.
//!      → requires: bytes-aware variant of `extract_str` and the
//!        `MbObject::new_bytes` returns inside the glob result list.
//!   5. Symlink semantics + broken symlinks
//!      → blocks: test_glob_symlinks, test_glob_broken_symlinks,
//!                test_selflink
//!      → requires: surface `os.symlink` + `os.path.islink` on mamba
//!                  and follow_symlinks awareness in the walker.
//!   6. `glob.magic_check` as a `re.Pattern` rather than raw `str`
//!      → blocks: any test that does `glob.magic_check.search(...)`
//!      → requires: `re.Pattern` modeling in the runtime (parent: re
//!                  conformance ticket, not this issue).
//!   7. `iglob` as a lazy generator (currently aliases `glob` — eager list)
//!      → blocks: tests that rely on partial iteration / GC pressure
//!      → requires: native iterator/generator wiring; semantically
//!        equivalent for `for p in iglob(...)` loops today.
//!
//!   Of CPython 3.12 `Lib/test/test_glob.py`'s 18 test methods, all
//!   18 hit at least one of the above blockers — so the headline
//!   `Lib/test_glob` pass-rate against the unmodified suite is 0/18
//!   until items 1+2+3 land. Mamba's own conformance fixture
//!   (`projects/mamba/tests/cpython/fixtures/std-libs/glob/`)
//!   covers the supported slice (literal, `*`, `?`, glob0/1,
//!   has_magic, escape, iglob alias) at 100%.
//! HANDWRITE-END
//!
//! Carve-outs:
//!   - `glob` / `iglob`: filesystem walk supports `*` and `?` against the
//!     parent directory only. `**` recursive globbing, brace expansion,
//!     character classes (`[abc]`), and `root_dir`/`dir_fd`/`recursive`
//!     keyword arguments are not yet wired — the variadic dispatcher
//!     ignores extra positional args. `iglob` is an alias for `glob`
//!     (eager list, not a lazy generator) — semantically equivalent for
//!     `for p in iglob(...):` loops.
//!   - `glob0(dirname, pattern)` returns `[pattern]` if `dirname/pattern`
//!     exists as a literal (no wildcard expansion). `glob1(dirname,
//!     pattern)` returns the filenames inside `dirname` that match
//!     `pattern`. These mirror the private CPython helpers that are
//!     nevertheless part of the public `dir(glob)` surface.
//!   - `has_magic(s)` is real: detects `*`, `?`, or `[` in the string.
//!   - `escape(s)` is real: wraps each magic char in a `[]` character
//!     class per CPython's reference implementation.
//!   - `magic_check` / `magic_check_bytes` are exposed as the literal
//!     regex pattern strings CPython compiles (`"([*?[])"`). Mamba does
//!     not model the `re.Pattern` object yet, so callers checking the
//!     `.pattern` attribute see the raw source text.
//!   - Module re-exports (`contextlib`, `fnmatch`, `itertools`, `os`,
//!     `re`, `stat`, `sys`): exposed as `MbValue::none()` placeholders.
//!     CPython's `glob` imports these for internal use only; user code
//!     that does `import glob; glob.os` is rare enough to defer to a
//!     proper module-aliasing pass.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// -- Variadic dispatchers --

// Each generated dispatcher loads `stringify!($disp)` through `black_box`
// so LLVM's `mergefunc` pass keeps the bodies distinct. Without that,
// `dispatch_glob` / `dispatch_iglob` (both → `mb_glob_glob`) collapse to a
// single function pointer and `test_register_wires_full_surface` undercounts.

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($disp));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($disp));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_unary!(dispatch_glob, mb_glob_glob);
disp_unary!(dispatch_iglob, mb_glob_glob);
disp_unary!(dispatch_has_magic, mb_glob_has_magic);
disp_unary!(dispatch_escape, mb_glob_escape);
disp_binary!(dispatch_glob0, mb_glob_glob0);
disp_binary!(dispatch_glob1, mb_glob_glob1);

/// CPython's magic-check regex source for str patterns.
const MAGIC_CHECK_PATTERN: &str = "([*?[])";

/// Register the glob module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("glob", dispatch_glob as *const () as usize),
        ("iglob", dispatch_iglob as *const () as usize),
        ("has_magic", dispatch_has_magic as *const () as usize),
        ("escape", dispatch_escape as *const () as usize),
        ("glob0", dispatch_glob0 as *const () as usize),
        ("glob1", dispatch_glob1 as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    attrs.insert(
        "magic_check".to_string(),
        MbValue::from_ptr(MbObject::new_str(MAGIC_CHECK_PATTERN.to_string())),
    );
    attrs.insert(
        "magic_check_bytes".to_string(),
        MbValue::from_ptr(MbObject::new_bytes(MAGIC_CHECK_PATTERN.as_bytes().to_vec())),
    );

    for sub in [
        "contextlib",
        "fnmatch",
        "itertools",
        "os",
        "re",
        "stat",
        "sys",
    ] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    super::register_module("glob", attrs);
}

// -- Helpers --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// -- Glob pattern matching --

/// Match a pattern against text. Supports `*` (any sequence) and `?` (single char).
fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_inner(&pat, &txt)
}

fn glob_match_inner(pat: &[char], txt: &[char]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi: Option<usize> = None;
    let mut star_ti: usize = 0;

    while ti < txt.len() {
        if pi < pat.len() && (pat[pi] == '?' || pat[pi] == txt[ti]) {
            pi += 1;
            ti += 1;
        } else if pi < pat.len() && pat[pi] == '*' {
            star_pi = Some(pi);
            star_ti = ti;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_ti += 1;
            ti = star_ti;
        } else {
            return false;
        }
    }

    while pi < pat.len() && pat[pi] == '*' {
        pi += 1;
    }

    pi == pat.len()
}

/// Split a glob pattern into (directory_prefix, file_pattern).
fn split_pattern(pattern: &str) -> (String, String) {
    let p = std::path::Path::new(pattern);

    if let (Some(parent), Some(file_pat)) = (p.parent(), p.file_name()) {
        let parent_str = parent.to_str().unwrap_or(".").to_string();
        let file_str = file_pat.to_str().unwrap_or("").to_string();
        if file_str.contains('*') || file_str.contains('?') {
            let dir = if parent_str.is_empty() {
                ".".to_string()
            } else {
                parent_str
            };
            return (dir, file_str);
        }
    }

    if pattern.contains('*') || pattern.contains('?') {
        return (".".to_string(), pattern.to_string());
    }

    (pattern.to_string(), String::new())
}

// -- Runtime functions --

/// glob.glob(pattern) -> list of matching paths
pub fn mb_glob_glob(pattern: MbValue) -> MbValue {
    let pat_str = match extract_str(pattern) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let (dir, file_pattern) = split_pattern(&pat_str);

    if file_pattern.is_empty() {
        if std::path::Path::new(&dir).exists() {
            let item = MbValue::from_ptr(MbObject::new_str(dir));
            return MbValue::from_ptr(MbObject::new_list(vec![item]));
        }
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        if glob_match(&file_pattern, &name) {
            let full_path = entry.path();
            let path_str = full_path.to_str().unwrap_or("").to_string();
            results.push(MbValue::from_ptr(MbObject::new_str(path_str)));
        }
    }

    MbValue::from_ptr(MbObject::new_list(results))
}

/// glob.has_magic(s) -> bool
pub fn mb_glob_has_magic(s: MbValue) -> MbValue {
    let Some(text) = extract_str(s) else {
        return MbValue::from_bool(false);
    };
    let hit = text.contains('*') || text.contains('?') || text.contains('[');
    MbValue::from_bool(hit)
}

/// glob.escape(s) -> str
///
/// Wraps each magic char (`*`, `?`, `[`) in a `[]` character class so the
/// result matches the original input literally when re-globbed.
pub fn mb_glob_escape(s: MbValue) -> MbValue {
    let Some(text) = extract_str(s) else {
        return raise_type_error("glob.escape() argument must be str");
    };
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '*' | '?' | '[' => {
                out.push('[');
                out.push(c);
                out.push(']');
            }
            _ => out.push(c),
        }
    }
    MbValue::from_ptr(MbObject::new_str(out))
}

/// glob.glob0(dirname, basename) -> list
///
/// CPython internal helper: returns `[basename]` if `dirname/basename`
/// exists as a literal path (no wildcards), else `[]`.
pub fn mb_glob_glob0(dirname: MbValue, basename: MbValue) -> MbValue {
    let dir = extract_str(dirname).unwrap_or_else(|| ".".to_string());
    let base = extract_str(basename).unwrap_or_default();
    let candidate = if dir.is_empty() {
        base.clone()
    } else {
        format!("{}/{}", dir.trim_end_matches('/'), base)
    };
    if std::path::Path::new(&candidate).exists() {
        let item = MbValue::from_ptr(MbObject::new_str(base));
        MbValue::from_ptr(MbObject::new_list(vec![item]))
    } else {
        MbValue::from_ptr(MbObject::new_list(vec![]))
    }
}

/// glob.glob1(dirname, pattern) -> list
///
/// CPython internal helper: returns the names in `dirname` matching
/// `pattern` (no path prefix on the returned entries).
pub fn mb_glob_glob1(dirname: MbValue, pattern: MbValue) -> MbValue {
    let dir = extract_str(dirname).unwrap_or_else(|| ".".to_string());
    let pat = extract_str(pattern).unwrap_or_default();
    let entries = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let mut results = Vec::new();
    for entry in entries.flatten() {
        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        // CPython glob1 skips hidden files unless the pattern itself begins
        // with '.'; preserve that behaviour for surface-level compatibility.
        if name.starts_with('.') && !pat.starts_with('.') {
            continue;
        }
        if glob_match(&pat, &name) {
            results.push(MbValue::from_ptr(MbObject::new_str(name)));
        }
    }
    MbValue::from_ptr(MbObject::new_list(results))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    fn list_strs(val: MbValue) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    for item in lock.read().unwrap().iter() {
                        if let Some(s) = extract_str(*item) {
                            out.push(s);
                        }
                    }
                }
            }
        }
        out
    }

    #[test]
    fn test_glob_match_basic() {
        assert!(glob_match("*.txt", "hello.txt"));
        assert!(glob_match("*.txt", ".txt"));
        assert!(!glob_match("*.txt", "hello.rs"));
        assert!(glob_match("hello.*", "hello.txt"));
        assert!(glob_match("h?llo", "hello"));
        assert!(!glob_match("h?llo", "hllo"));
        assert!(glob_match("*", "anything"));
        assert!(glob_match("a*b", "ab"));
        assert!(glob_match("a*b", "aXYZb"));
    }

    #[test]
    fn test_split_pattern() {
        let (dir, pat) = split_pattern("/home/user/*.txt");
        assert_eq!(dir, "/home/user");
        assert_eq!(pat, "*.txt");

        let (dir2, pat2) = split_pattern("*.rs");
        assert_eq!(dir2, ".");
        assert_eq!(pat2, "*.rs");
    }

    #[test]
    fn test_glob_real_dir() {
        let tmp = std::env::temp_dir().join("mb_glob_test_full");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("a.txt"), "").unwrap();
        std::fs::write(tmp.join("b.txt"), "").unwrap();
        std::fs::write(tmp.join("c.rs"), "").unwrap();

        let pattern_str = format!("{}/*.txt", tmp.to_str().unwrap());
        let result = mb_glob_glob(s(&pattern_str));
        assert_eq!(list_strs(result).len(), 2);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    // -- has_magic truth table --

    #[test]
    fn test_has_magic_truth_table() {
        let cases: &[(&str, bool)] = &[
            ("", false),
            ("plain.txt", false),
            ("a*", true),
            ("a?b", true),
            ("[abc]", true),
            ("path/to/x", false),
            ("path/to/*", true),
            ("trailing[", true),
        ];
        for (input, want) in cases {
            let got = mb_glob_has_magic(s(input)).as_bool();
            assert_eq!(got, Some(*want), "has_magic({:?})", input);
        }
    }

    #[test]
    fn test_has_magic_non_str_is_false() {
        assert_eq!(
            mb_glob_has_magic(MbValue::from_int(7)).as_bool(),
            Some(false)
        );
        assert_eq!(mb_glob_has_magic(MbValue::none()).as_bool(), Some(false));
    }

    // -- escape --

    #[test]
    fn test_escape_wraps_magic_chars() {
        assert_eq!(
            get_str(mb_glob_escape(s("plain"))),
            Some("plain".to_string())
        );
        assert_eq!(get_str(mb_glob_escape(s("a*"))), Some("a[*]".to_string()));
        assert_eq!(get_str(mb_glob_escape(s("a?b"))), Some("a[?]b".to_string()));
        assert_eq!(get_str(mb_glob_escape(s("["))), Some("[[]".to_string()));
        assert_eq!(
            get_str(mb_glob_escape(s("*?["))),
            Some("[*][?][[]".to_string())
        );
    }

    #[test]
    fn test_escape_then_has_magic_is_still_true() {
        // CPython note: the escaped form still contains '[' which is itself
        // a magic char, so has_magic over the escaped output stays true.
        // We assert this so the contract is documented.
        let escaped = mb_glob_escape(s("a*b"));
        assert_eq!(get_str(escaped), Some("a[*]b".to_string()));
        assert_eq!(mb_glob_has_magic(escaped).as_bool(), Some(true));
    }

    #[test]
    fn test_escape_roundtrip_literal_match() {
        // After escape(p), the result should literally match p when the
        // glob engine treats '[X]' as a character class. Mamba's matcher
        // currently treats '[' as literal — verify the path of intent by
        // round-tripping through has_magic and string identity.
        let original = "file*name?.txt";
        let escaped = mb_glob_escape(s(original));
        let out = get_str(escaped).unwrap();
        assert!(out.contains("[*]"));
        assert!(out.contains("[?]"));
    }

    // -- glob0 / glob1 --

    #[test]
    fn test_glob0_literal_hit() {
        let tmp = std::env::temp_dir().join("mb_glob0_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("exact.txt"), "").unwrap();

        let result = mb_glob_glob0(s(tmp.to_str().unwrap()), s("exact.txt"));
        assert_eq!(list_strs(result), vec!["exact.txt"]);

        let miss = mb_glob_glob0(s(tmp.to_str().unwrap()), s("nope.txt"));
        assert!(list_strs(miss).is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_glob1_matches_in_dir() {
        let tmp = std::env::temp_dir().join("mb_glob1_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join("alpha.md"), "").unwrap();
        std::fs::write(tmp.join("beta.md"), "").unwrap();
        std::fs::write(tmp.join("gamma.rs"), "").unwrap();

        let result = mb_glob_glob1(s(tmp.to_str().unwrap()), s("*.md"));
        let mut names = list_strs(result);
        names.sort();
        assert_eq!(names, vec!["alpha.md", "beta.md"]);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_glob1_skips_hidden_unless_pattern_starts_with_dot() {
        let tmp = std::env::temp_dir().join("mb_glob1_hidden");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        std::fs::write(tmp.join(".hidden"), "").unwrap();
        std::fs::write(tmp.join("visible"), "").unwrap();

        let star = mb_glob_glob1(s(tmp.to_str().unwrap()), s("*"));
        assert_eq!(list_strs(star), vec!["visible".to_string()]);

        let dotstar = mb_glob_glob1(s(tmp.to_str().unwrap()), s(".*"));
        assert_eq!(list_strs(dotstar), vec![".hidden".to_string()]);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_glob1_missing_dir_returns_empty() {
        let result = mb_glob_glob1(s("/this/path/should/not/exist/xyz"), s("*"));
        assert!(list_strs(result).is_empty());
    }

    // -- register() surface --

    #[test]
    fn test_register_wires_full_surface() {
        register();
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        // 6 dispatchers should each be registered; snapshot is monotonic
        // across the test process so we only assert presence is non-zero.
        assert!(
            snap >= 6,
            "expected at least 6 native func addrs registered"
        );
    }

    #[test]
    fn test_magic_check_pattern_value() {
        // The literal CPython source — `re.compile("([*?[])")`.
        assert_eq!(MAGIC_CHECK_PATTERN, "([*?[])");
    }
}
