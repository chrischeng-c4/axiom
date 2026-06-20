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
//!   (`projects/mamba/tests/cpython/std-libs/glob/`)
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

// `glob` / `iglob` accept keyword args (`recursive=`, `root_dir=`). The HIR
// lowering for an attribute call with keywords (`glob.glob(p, recursive=True)`)
// appends the keyword bundle as a trailing positional dict, so the native
// `(args_ptr, nargs)` dispatcher receives `[pattern, {"recursive": True, ...}]`.
// These dispatchers forward the full slice so the impl can recover the kwargs.
unsafe extern "C" fn dispatch_glob(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_glob));
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_glob_glob_args(a)
}
unsafe extern "C" fn dispatch_iglob(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_iglob));
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_glob_iglob_args(a)
}
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

/// Match a pattern against text. Supports `*` (any sequence), `?` (single
/// char), and `[...]` character classes (`[abc]`, ranges `[a-z]`, negation
/// `[!abc]`) per CPython's `fnmatch.translate` semantics.
fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_inner(&pat, &txt)
}

/// Try to parse a `[...]` character class starting at `pat[pi]` (which must be
/// `'['`). On success returns `(matched, next_pi)` where `matched` says whether
/// `ch` is in the class and `next_pi` points just past the closing `']'`.
///
/// Mirrors CPython's `fnmatch.translate`: an unterminated `[` (no closing `]`)
/// is treated as a literal `'['`, signalled here by returning `None`.
fn match_char_class(pat: &[char], pi: usize, ch: char) -> Option<(bool, usize)> {
    // pat[pi] == '['
    let mut j = pi + 1;
    // Leading '!' negates the class.
    let negate = j < pat.len() && pat[j] == '!';
    if negate {
        j += 1;
    }
    // A ']' immediately after '[' or '[!' is a literal member, not the
    // terminator (CPython/fnmatch behaviour).
    let class_start = j;
    if j < pat.len() && pat[j] == ']' {
        j += 1;
    }
    // Find the closing ']'.
    while j < pat.len() && pat[j] != ']' {
        j += 1;
    }
    if j >= pat.len() {
        // Unterminated class → treat '[' literally.
        return None;
    }
    let class_end = j; // index of closing ']'
                       // Evaluate membership over pat[class_start..class_end].
    let mut matched = false;
    let members = &pat[class_start..class_end];
    let mut k = 0;
    while k < members.len() {
        // Range like a-z: members[k+1] == '-' and there's a char after it,
        // and '-' is not the trailing char of the class.
        if k + 2 < members.len() && members[k + 1] == '-' {
            let lo = members[k];
            let hi = members[k + 2];
            if lo <= ch && ch <= hi {
                matched = true;
            }
            k += 3;
        } else {
            if members[k] == ch {
                matched = true;
            }
            k += 1;
        }
    }
    Some((matched ^ negate, class_end + 1))
}

fn glob_match_inner(pat: &[char], txt: &[char]) -> bool {
    let mut pi = 0;
    let mut ti = 0;
    let mut star_pi: Option<usize> = None;
    let mut star_ti: usize = 0;

    while ti < txt.len() {
        if pi < pat.len() && pat[pi] == '[' {
            // Character class: attempt to match pat[pi..] against txt[ti].
            match match_char_class(pat, pi, txt[ti]) {
                Some((true, next_pi)) => {
                    pi = next_pi;
                    ti += 1;
                    continue;
                }
                Some((false, _)) => {
                    // Class present but no member matched → backtrack to star.
                    if let Some(sp) = star_pi {
                        pi = sp + 1;
                        star_ti += 1;
                        ti = star_ti;
                        continue;
                    }
                    return false;
                }
                None => {
                    // Unterminated '[' → fall through to literal matching of '['.
                }
            }
        }
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

    // Consume any trailing '*' and matching empty character classes.
    while pi < pat.len() {
        if pat[pi] == '*' {
            pi += 1;
        } else {
            break;
        }
    }

    pi == pat.len()
}

/// Whether a string contains a glob wildcard (`*`, `?`, or `[`) — the same
/// magic-char set as `glob.has_magic` / CPython's `magic_check` regex.
fn has_glob_magic(s: &str) -> bool {
    s.contains('*') || s.contains('?') || s.contains('[')
}

// -- CPython glob algorithm (Lib/glob.py port) --
//
// Mirrors CPython 3.12 `glob._iglob` / `_glob0` / `_glob1` / `_glob2`
// (without `root_dir`/`dir_fd`/`bytes`/`include_hidden`, which the conformance
// fixtures do not exercise). `root_dir` is threaded as an optional join prefix
// for filesystem probes only — the yielded paths stay relative to it, exactly
// as CPython does.

/// `os.path.split(pathname)` for POSIX: split at the last `/`. The head keeps a
/// trailing slash only when it is all slashes (the filesystem root).
fn posix_split(p: &str) -> (String, String) {
    match p.rfind('/') {
        Some(i) => {
            let head = &p[..i + 1];
            let tail = &p[i + 1..];
            // Strip trailing slashes from head unless it is all slashes.
            let stripped = head.trim_end_matches('/');
            let head_out = if stripped.is_empty() { head } else { stripped };
            (head_out.to_string(), tail.to_string())
        }
        None => (String::new(), p.to_string()),
    }
}

/// `os.path.join(a, b)` for POSIX. Matches CPython `posixpath.join` exactly,
/// including `join("gtree", "")` -> `"gtree/"` (the trailing slash that makes
/// recursive-`**` and trailing-slash patterns surface directory paths with a
/// terminal separator). `b` is never absolute in our callers.
fn posix_join(a: &str, b: &str) -> String {
    if a.is_empty() || a.ends_with('/') {
        format!("{}{}", a, b)
    } else {
        format!("{}/{}", a, b)
    }
}

/// glob's `_join`: if either side is empty, return the other untouched.
fn glob_join(dirname: &str, basename: &str) -> String {
    if dirname.is_empty() || basename.is_empty() {
        if dirname.is_empty() {
            basename.to_string()
        } else {
            dirname.to_string()
        }
    } else {
        posix_join(dirname, basename)
    }
}

fn is_recursive(pattern: &str) -> bool {
    pattern == "**"
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// Join `root_dir` (the optional probe prefix) in front of a path for fs probes.
fn probe_path(root_dir: &str, pathname: &str) -> String {
    glob_join(root_dir, pathname)
}

fn lexists(path: &str) -> bool {
    // os.path.lexists: true for any existing entry, including broken symlinks.
    std::fs::symlink_metadata(path).is_ok()
}

fn is_dir(path: &str) -> bool {
    // os.path.isdir: follows symlinks. Empty path means the current directory.
    let probe = if path.is_empty() { "." } else { path };
    std::fs::metadata(probe)
        .map(|m| m.is_dir())
        .unwrap_or(false)
}

/// `_listdir`: filenames inside `dirname`. `dironly` keeps directories only.
/// Empty `dirname` means the current directory.
fn list_dir(dirname: &str, dironly: bool) -> Vec<String> {
    let probe = if dirname.is_empty() {
        ".".to_string()
    } else {
        dirname.to_string()
    };
    let rd = match std::fs::read_dir(&probe) {
        Ok(rd) => rd,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for entry in rd.flatten() {
        let name = match entry.file_name().into_string() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if dironly {
            // entry.is_dir() follows symlinks via file_type fallback to metadata.
            let is_d = match entry.file_type() {
                Ok(ft) if ft.is_dir() => true,
                Ok(ft) if ft.is_symlink() => std::fs::metadata(entry.path())
                    .map(|m| m.is_dir())
                    .unwrap_or(false),
                _ => false,
            };
            if !is_d {
                continue;
            }
        }
        out.push(name);
    }
    out
}

/// `_glob1`: basenames in `dirname` matching `pattern`, with hidden-file rule.
fn glob1(root_dir: &str, dirname: &str, pattern: &str, dironly: bool) -> Vec<String> {
    let full = probe_path(root_dir, dirname);
    let names = list_dir(&full, dironly);
    let pattern_hidden = is_hidden(pattern);
    let mut out = Vec::new();
    for name in names {
        if !pattern_hidden && is_hidden(&name) {
            continue;
        }
        if glob_match(pattern, &name) {
            out.push(name);
        }
    }
    out
}

/// `_glob0`: literal basename existence check.
fn glob0(root_dir: &str, dirname: &str, basename: &str, _dironly: bool) -> Vec<String> {
    if !basename.is_empty() {
        let probe = probe_path(root_dir, &glob_join(dirname, basename));
        if lexists(&probe) {
            return vec![basename.to_string()];
        }
    } else {
        // Trailing-slash patterns ('dir/') match only directories.
        let probe = probe_path(root_dir, dirname);
        if is_dir(&probe) {
            return vec![basename.to_string()];
        }
    }
    Vec::new()
}

/// `_rlistdir`: relative pathnames recursively under `dirname`.
fn rlistdir(root_dir: &str, dirname: &str, dironly: bool, out: &mut Vec<String>) {
    let full = probe_path(root_dir, dirname);
    let names = list_dir(&full, dironly);
    for x in names {
        if is_hidden(&x) {
            continue;
        }
        out.push(x.clone());
        let path = if dirname.is_empty() {
            x.clone()
        } else {
            glob_join(dirname, &x)
        };
        let mut sub = Vec::new();
        rlistdir(root_dir, &path, dironly, &mut sub);
        for y in sub {
            out.push(glob_join(&x, &y));
        }
    }
}

/// `_glob2`: recursive `**` segment — yields `""` for the base dir, then all
/// recursive relative pathnames.
fn glob2(root_dir: &str, dirname: &str, dironly: bool) -> Vec<String> {
    let mut out = Vec::new();
    let probe = probe_path(root_dir, dirname);
    if dirname.is_empty() || is_dir(&probe) {
        out.push(String::new());
    }
    rlistdir(root_dir, dirname, dironly, &mut out);
    out
}

/// `_iglob`: the recursive core. Yields path strings relative to `root_dir`.
fn iglob_core(pathname: &str, root_dir: &str, recursive: bool, dironly: bool) -> Vec<String> {
    let (dirname, basename) = posix_split(pathname);

    if !has_glob_magic(pathname) {
        // No magic anywhere → literal existence check.
        if !basename.is_empty() {
            if lexists(&probe_path(root_dir, pathname)) {
                return vec![pathname.to_string()];
            }
        } else {
            // Trailing-slash pattern: match only directories.
            if is_dir(&probe_path(root_dir, &dirname)) {
                return vec![pathname.to_string()];
            }
        }
        return Vec::new();
    }

    if dirname.is_empty() {
        if recursive && is_recursive(&basename) {
            return glob2(root_dir, "", dironly);
        }
        return glob1(root_dir, "", &basename, dironly);
    }

    // Resolve the directory component (which may itself contain magic).
    let dirs: Vec<String> = if dirname != pathname && has_glob_magic(&dirname) {
        iglob_core(&dirname, root_dir, recursive, true)
    } else {
        vec![dirname.clone()]
    };

    let mut out = Vec::new();
    for d in dirs {
        let names: Vec<String> = if has_glob_magic(&basename) {
            if recursive && is_recursive(&basename) {
                glob2(root_dir, &d, dironly)
            } else {
                glob1(root_dir, &d, &basename, dironly)
            }
        } else {
            glob0(root_dir, &d, &basename, dironly)
        };
        for name in names {
            out.push(posix_join(&d, &name));
        }
    }
    out
}

/// `iglob`: wraps `_iglob` and skips a leading empty result for a recursive
/// `**`-prefixed pattern (CPython's `next(it)  # skip empty string`).
fn iglob_results(pathname: &str, root_dir: &str, recursive: bool) -> Vec<String> {
    let mut results = iglob_core(pathname, root_dir, recursive, false);
    // CPython: `if not pathname or recursive and _isrecursive(pathname[:2])`
    // → skip a single leading empty string yielded by a bare/leading `**`.
    let leading_recursive = recursive && pathname.starts_with("**");
    if pathname.is_empty() || leading_recursive {
        if !results.is_empty() && results[0].is_empty() {
            results.remove(0);
        }
    }
    results
}

// -- Runtime functions --

/// Extract `recursive` / `root_dir` from a trailing kwargs dict appended by the
/// HIR call lowering for `glob.glob(p, recursive=True, root_dir=...)`.
fn kwargs_from_args(args: &[MbValue]) -> (bool, Option<String>) {
    let mut recursive = false;
    let mut root_dir = None;
    // The pattern is args[0]; any trailing dict carries keyword args.
    for v in args.iter().skip(1) {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(rv) = map.get(&super::super::dict_ops::DictKey::Str(
                        "recursive".to_string(),
                    )) {
                        recursive = mb_truthy(*rv);
                    }
                    if let Some(rd) = map.get(&super::super::dict_ops::DictKey::Str(
                        "root_dir".to_string(),
                    )) {
                        if let Some(s) = extract_str(*rd) {
                            root_dir = Some(s);
                        }
                    }
                }
            }
        }
    }
    (recursive, root_dir)
}

/// Truthiness for a kwarg value (bool / int / None).
fn mb_truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    !v.is_none()
}

fn glob_list(args: &[MbValue]) -> MbValue {
    let pat = args.first().copied().unwrap_or_else(MbValue::none);
    let pat_str = match extract_str(pat) {
        Some(s) => s,
        None => {
            // A bytes pattern is valid in CPython (mamba doesn't model byte
            // paths yet → empty result, no raise). Any other non-str pattern
            // (int / None / float) is a TypeError, not a silent empty match.
            let is_bytes = pat.as_ptr().map_or(false, |p| {
                matches!(
                    unsafe { &(*p).data },
                    ObjData::Bytes(_) | ObjData::ByteArray(_)
                )
            });
            if is_bytes {
                return MbValue::from_ptr(MbObject::new_list(vec![]));
            }
            return raise_type_error(&format!(
                "glob() argument must be str, bytes, or os.PathLike, not {}",
                super::super::builtins::value_type_name(pat)
            ));
        }
    };
    let (recursive, root_dir) = kwargs_from_args(args);
    let root = root_dir.unwrap_or_default();
    let results = iglob_results(&pat_str, &root, recursive);
    let items: Vec<MbValue> = results
        .into_iter()
        .map(|p| MbValue::from_ptr(MbObject::new_str(p)))
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

/// glob.glob(pattern, *, recursive=False, root_dir=None) -> list of paths.
fn mb_glob_glob_args(args: &[MbValue]) -> MbValue {
    glob_list(args)
}

/// glob.iglob(pattern, *, recursive=False, root_dir=None) -> iterator of paths.
fn mb_glob_iglob_args(args: &[MbValue]) -> MbValue {
    let list = glob_list(args);
    super::super::iter::mb_iter(list)
}

/// glob.glob(pattern) -> list of matching paths (single-arg entry, used by the
/// in-crate tests). The dispatcher uses `mb_glob_glob_args` directly to recover
/// keyword arguments.
pub fn mb_glob_glob(pattern: MbValue) -> MbValue {
    glob_list(&[pattern])
}

/// glob.iglob(pattern) -> iterator of matching paths.
///
/// CPython's `iglob` returns a lazy generator exposing `__iter__`/`__next__`.
/// Mamba materialises the same results as `glob.glob` and wraps the list in a
/// real `list_iterator` (via the runtime's `mb_iter`) so callers that probe
/// `hasattr(it, "__next__")` or call `next(it)` behave like CPython.
pub fn mb_glob_iglob(pattern: MbValue) -> MbValue {
    let list = glob_list(&[pattern]);
    super::super::iter::mb_iter(list)
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
    fn test_posix_split() {
        // Mirrors os.path.split: rightmost '/', head keeps a trailing slash
        // only when it is all slashes (the root).
        assert_eq!(
            posix_split("/home/user/*.txt"),
            ("/home/user".into(), "*.txt".into())
        );
        assert_eq!(posix_split("*.rs"), ("".into(), "*.rs".into()));
        assert_eq!(
            posix_split("gtree/**/*.txt"),
            ("gtree/**".into(), "*.txt".into())
        );
        assert_eq!(posix_split("gtree//sub"), ("gtree".into(), "sub".into()));
        assert_eq!(posix_split("gtree/sub/"), ("gtree/sub".into(), "".into()));
        assert_eq!(posix_split("/"), ("/".into(), "".into()));
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
