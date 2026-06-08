/// posixpath module for Mamba (#1261 long-tail).
///
/// Real implementation of CPython 3.12 `Lib/posixpath.py`. Previously
/// registered through `long_tail_mod` as a stub that returned empty
/// strings for `join`/`basename`/`dirname` and `False` for `exists`/
/// `isfile`/`isdir` — i.e., totally broken for any consumer.
///
/// Pure-string ops (`join`, `split`, `splitext`, `basename`, `dirname`,
/// `isabs`, `normpath`, `splitdrive`, `normcase`, `commonprefix`,
/// `commonpath`) are reimplemented to match POSIX semantics exactly.
/// FS-backed ops (`exists`, `isfile`, `isdir`, `islink`, `ismount`,
/// `getsize`, `getmtime`, `getatime`, `getctime`, `samefile`,
/// `abspath`, `realpath`, `relpath`, `expanduser`, `expandvars`)
/// hit the real filesystem / environment.
///
/// Also registered under the name `genericpath` since CPython exposes
/// the same FS-existence/size/time surface there.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

fn extract_str(val: MbValue) -> Option<String> {
    let p = val.as_ptr()?;
    unsafe {
        match (*p).data {
            ObjData::Str(ref s) => Some(s.clone()),
            // os.PathLike: an instance exposing __fspath__ (e.g. the test
            // suite's FakePath, or pathlib.PurePath). CPython's posixpath
            // functions normalize their argument via os.fspath() before
            // operating on it, so a FakePath(name) must behave identically
            // to the plain str `name`.
            ObjData::Instance { .. } => fspath_via_protocol(val),
            _ => None,
        }
    }
}

/// Call `__fspath__` on an Instance and decode the result to a String.
/// Mirrors `os_mod::fspath_via_protocol` so posixpath functions accept
/// os.PathLike arguments. Returns None for non-instances or instances
/// without `__fspath__`.
fn fspath_via_protocol(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    let class_name = unsafe {
        match (*ptr).data {
            ObjData::Instance { ref class_name, .. } => class_name.clone(),
            _ => return None,
        }
    };
    if super::super::class::lookup_method(&class_name, "__fspath__").is_none() {
        return None;
    }
    let method = MbValue::from_ptr(MbObject::new_str("__fspath__".to_string()));
    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    let result = super::super::class::mb_call_method(val, method, empty);
    // Decode the result without recursing back into __fspath__: it must
    // already be a str (or bytes, which we don't handle here).
    let rp = result.as_ptr()?;
    unsafe {
        if let ObjData::Str(ref s) = (*rp).data {
            Some(s.clone())
        } else {
            None
        }
    }
}

fn mk_str(s: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s))
}

fn mk_tuple2(a: String, b: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![mk_str(a), mk_str(b)]))
}

// ── Pure-string POSIX ops ──

fn posix_join_strs(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let mut result = parts[0].clone();
    for part in &parts[1..] {
        if part.starts_with('/') {
            result = part.clone();
        } else if result.is_empty() || result.ends_with('/') {
            result.push_str(part);
        } else {
            result.push('/');
            result.push_str(part);
        }
    }
    result
}

/// Borrow-based posix join — avoids the N+1 String clones the
/// owned-`String` variant pays per call. Used by `dispatch_join`'s
/// fast path when every arg is an `ObjData::Str` (the overwhelmingly
/// common shape for hot loops like `os.path.join("usr", "local",
/// "bin", "python3")`). Pre-sizes the output buffer to the sum of
/// arg lengths + N separators so the loop does at most one heap
/// allocation per call (vs the prior 6 in the 4-arg shape).
fn posix_join_borrowed(parts: &[&str]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    // Conservative upper bound: total bytes + one separator per arg.
    let cap: usize = parts.iter().map(|p| p.len() + 1).sum();
    let mut result = String::with_capacity(cap);
    result.push_str(parts[0]);
    for part in &parts[1..] {
        if part.starts_with('/') {
            result.clear();
            result.push_str(part);
        } else if result.is_empty() || result.ends_with('/') {
            result.push_str(part);
        } else {
            result.push('/');
            result.push_str(part);
        }
    }
    result
}

fn posix_split(path: &str) -> (String, String) {
    // CPython posixpath.split: i = p.rfind('/') + 1; head, tail = p[:i], p[i:].
    // Then head is rstripped of '/' unless head is all slashes (so '/' stays
    // '/' and '//' stays '//').
    if let Some(pos) = path.rfind('/') {
        let i = pos + 1;
        let head_raw = &path[..i];
        let tail = &path[i..];
        let all_slash = head_raw.bytes().all(|b| b == b'/');
        let head = if all_slash {
            head_raw.to_string()
        } else {
            let mut end = head_raw.len();
            while end > 0 && head_raw.as_bytes()[end - 1] == b'/' {
                end -= 1;
            }
            head_raw[..end].to_string()
        };
        (head, tail.to_string())
    } else {
        (String::new(), path.to_string())
    }
}

fn posix_basename(path: &str) -> String {
    posix_split(path).1
}

fn posix_dirname(path: &str) -> String {
    posix_split(path).0
}

fn posix_splitext(path: &str) -> (String, String) {
    let base_start = path.rfind('/').map(|i| i + 1).unwrap_or(0);
    let base = &path[base_start..];
    // CPython: a basename made entirely of dots has no extension; the
    // search starts after the last leading dot.
    let dot_search_start = base.bytes().position(|b| b != b'.').unwrap_or(base.len());
    if let Some(rel_pos) = base[dot_search_start..].rfind('.') {
        let abs_pos = base_start + dot_search_start + rel_pos;
        (path[..abs_pos].to_string(), path[abs_pos..].to_string())
    } else {
        (path.to_string(), String::new())
    }
}

fn posix_isabs(path: &str) -> bool {
    path.starts_with('/')
}

fn posix_normpath(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let is_abs = path.starts_with('/');
    // POSIX: `//foo` is implementation-defined but distinct from `/foo`
    // (CPython preserves the double slash). `///foo` collapses to `/foo`.
    let leading_double_slash = path.starts_with("//") && !path.starts_with("///");
    let initial_slashes = if leading_double_slash {
        2
    } else if is_abs {
        1
    } else {
        0
    };

    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty() && *p != ".").collect();
    let mut comps: Vec<&str> = Vec::new();
    for part in parts {
        if part == ".." {
            if !comps.is_empty() && *comps.last().unwrap() != ".." {
                comps.pop();
            } else if !is_abs {
                comps.push("..");
            }
        } else {
            comps.push(part);
        }
    }
    let prefix = "/".repeat(initial_slashes);
    let joined = comps.join("/");
    let result = format!("{}{}", prefix, joined);
    if result.is_empty() {
        ".".to_string()
    } else {
        result
    }
}

fn posix_commonprefix_strs(strs: &[String]) -> String {
    if strs.is_empty() {
        return String::new();
    }
    let first = &strs[0];
    let mut end = first.len();
    for s in &strs[1..] {
        let f = first.as_bytes();
        let p = s.as_bytes();
        let mut i = 0;
        while i < end && i < p.len() && f[i] == p[i] {
            i += 1;
        }
        end = i;
        if end == 0 {
            break;
        }
    }
    // Don't cut a multi-byte UTF-8 char in the middle.
    while end > 0 && !first.is_char_boundary(end) {
        end -= 1;
    }
    first[..end].to_string()
}

fn posix_commonpath_strs(strs: &[String]) -> Option<String> {
    if strs.is_empty() {
        return Some(String::new());
    }
    let abs = strs[0].starts_with('/');
    for s in strs {
        if s.starts_with('/') != abs {
            return None; // mixing absolute and relative — CPython raises ValueError
        }
    }
    let split_parts: Vec<Vec<&str>> = strs.iter()
        .map(|p| p.split('/').filter(|s| !s.is_empty() && *s != ".").collect())
        .collect();
    let min_len = split_parts.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut common: Vec<&str> = Vec::new();
    for i in 0..min_len {
        let part = split_parts[0][i];
        if split_parts.iter().all(|v| v[i] == part) {
            common.push(part);
        } else {
            break;
        }
    }
    let prefix = if abs { "/" } else { "" };
    Some(format!("{}{}", prefix, common.join("/")))
}

fn posix_splitdrive(path: &str) -> (String, String) {
    // POSIX has no drive concept.
    (String::new(), path.to_string())
}

fn posix_normcase(path: &str) -> String {
    // POSIX: identity (case-sensitive).
    path.to_string()
}

fn expand_user(path: &str) -> String {
    if !path.starts_with('~') {
        return path.to_string();
    }
    // `~` and `~/foo` use $HOME. `~user/foo` would need passwd lookup;
    // we leave that path unchanged (CPython falls back the same way
    // when the user doesn't exist).
    let split_idx = path.find('/').unwrap_or(path.len());
    let user_part = &path[..split_idx];
    let rest = &path[split_idx..];
    if user_part == "~" {
        if let Some(home) = std::env::var("HOME").ok().or_else(|| std::env::var("USERPROFILE").ok()) {
            return format!("{}{}", home, rest);
        }
    }
    path.to_string()
}

fn expand_vars(path: &str) -> String {
    // Match `$VAR`, `${VAR}`. Leave unknown vars verbatim (CPython behavior).
    let bytes = path.as_bytes();
    let mut out = String::with_capacity(path.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                if let Some(end_rel) = path[i + 2..].find('}') {
                    let name = &path[i + 2..i + 2 + end_rel];
                    if let Ok(val) = std::env::var(name) {
                        out.push_str(&val);
                    } else {
                        out.push_str(&path[i..i + 3 + end_rel]);
                    }
                    i += 3 + end_rel;
                    continue;
                }
            } else if bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_' {
                let mut j = i + 1;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                let name = &path[i + 1..j];
                if let Ok(val) = std::env::var(name) {
                    out.push_str(&val);
                } else {
                    out.push_str(&path[i..j]);
                }
                i = j;
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn metadata_time<F>(p: &str, getter: F) -> f64
where
    F: Fn(&std::fs::Metadata) -> std::io::Result<std::time::SystemTime>,
{
    let Ok(meta) = std::fs::metadata(p) else { return 0.0; };
    let Ok(t) = getter(&meta) else { return 0.0; };
    match t.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => d.as_secs_f64(),
        Err(_) => 0.0,
    }
}

fn relpath(start: &str, target: &str) -> String {
    let start_norm = posix_normpath(start);
    let target_norm = posix_normpath(target);
    let s_parts: Vec<&str> = start_norm.split('/').filter(|s| !s.is_empty()).collect();
    let t_parts: Vec<&str> = target_norm.split('/').filter(|s| !s.is_empty()).collect();
    let mut common = 0;
    while common < s_parts.len() && common < t_parts.len() && s_parts[common] == t_parts[common] {
        common += 1;
    }
    let ups = std::iter::repeat("..").take(s_parts.len() - common);
    let downs = t_parts[common..].iter().copied();
    let joined: Vec<&str> = ups.chain(downs).collect();
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined.join("/")
    }
}

// ── Dispatchers ──

unsafe extern "C" fn dispatch_join(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);

    // Borrow fast path — every arg is an `ObjData::Str` (the dominant
    // shape for `os.path.join("usr", "local", "bin", "python3")`-style
    // calls). Avoids the per-arg `String::clone` paid by `extract_str`
    // + the `parts[0].clone()` paid by `posix_join_strs`; collapses 6
    // heap allocations into 1 in the 4-arg case. Bench evidence:
    // `os_path/bench/join_hot.py` 4-arg 10k-iter loop.
    let all_str = args.iter().all(|v| {
        v.as_ptr()
            .map(|p| matches!(unsafe { &(*p).data }, ObjData::Str(_)))
            .unwrap_or(false)
    });
    if all_str {
        let borrows: Vec<&str> = args
            .iter()
            .map(|v| {
                let p = v.as_ptr().unwrap();
                match unsafe { &(*p).data } {
                    ObjData::Str(s) => s.as_str(),
                    _ => unreachable!(),
                }
            })
            .collect();
        return mk_str(posix_join_borrowed(&borrows));
    }

    // Slow path — at least one arg is not a Str (path-like object,
    // bytes, etc.). Preserve the existing extract_str behaviour.
    let strs: Vec<String> = args.iter().filter_map(|v| extract_str(*v)).collect();
    mk_str(posix_join_strs(&strs))
}

unsafe extern "C" fn dispatch_split(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(s) = args.first().copied().and_then(extract_str) else {
        return mk_tuple2(String::new(), String::new());
    };
    let (a, b) = posix_split(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_basename(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(posix_basename(&s))
}

unsafe extern "C" fn dispatch_dirname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(posix_dirname(&s))
}

unsafe extern "C" fn dispatch_splitext(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let (a, b) = posix_splitext(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_splitdrive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let (a, b) = posix_splitdrive(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_isabs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_bool(posix_isabs(&s))
}

unsafe extern "C" fn dispatch_normpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(posix_normpath(&s))
}

unsafe extern "C" fn dispatch_normcase(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(posix_normcase(&s))
}

unsafe extern "C" fn dispatch_commonprefix(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    // CPython takes a sequence; we accept either (list,) or varargs of strings.
    let strs: Vec<String> = if args.len() == 1 {
        if let Some(p) = args[0].as_ptr() {
            unsafe {
                match &(*p).data {
                    ObjData::List(lock) => lock.read().unwrap().iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Tuple(items) => items.iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Str(_) => vec![extract_str(args[0]).unwrap()],
                    _ => Vec::new(),
                }
            }
        } else { Vec::new() }
    } else {
        args.iter().filter_map(|v| extract_str(*v)).collect()
    };
    mk_str(posix_commonprefix_strs(&strs))
}

unsafe extern "C" fn dispatch_commonpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let strs: Vec<String> = if args.len() == 1 {
        if let Some(p) = args[0].as_ptr() {
            unsafe {
                match &(*p).data {
                    ObjData::List(lock) => lock.read().unwrap().iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Tuple(items) => items.iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Str(_) => vec![extract_str(args[0]).unwrap()],
                    _ => Vec::new(),
                }
            }
        } else { Vec::new() }
    } else {
        args.iter().filter_map(|v| extract_str(*v)).collect()
    };
    match posix_commonpath_strs(&strs) {
        Some(s) => mk_str(s),
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "Can't mix absolute and relative paths".to_string(),
                )),
            );
            MbValue::none()
        }
    }
}

unsafe extern "C" fn dispatch_exists(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).exists())
}

unsafe extern "C" fn dispatch_isfile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).is_file())
}

unsafe extern "C" fn dispatch_isdir(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).is_dir())
}

unsafe extern "C" fn dispatch_islink(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let is_link = std::fs::symlink_metadata(&s)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    MbValue::from_bool(is_link)
}

unsafe extern "C" fn dispatch_ismount(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    // True mount detection requires libc::stat dev comparison. Use a
    // cheap proxy: `/` is a mount point, plus paths whose canonical
    // form equals their parent's (root of a filesystem).
    let p = std::path::Path::new(&s);
    let is_mount = if s == "/" {
        true
    } else if let Ok(meta) = std::fs::symlink_metadata(p) {
        if let Some(parent) = p.parent() {
            if let Ok(pm) = std::fs::symlink_metadata(parent) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    pm.dev() != meta.dev()
                }
                #[cfg(not(unix))]
                {
                    let _ = (pm, &meta);
                    false
                }
            } else {
                false
            }
        } else {
            true
        }
    } else {
        false
    };
    MbValue::from_bool(is_mount)
}

unsafe extern "C" fn dispatch_getsize(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    match std::fs::metadata(&s) {
        Ok(m) => MbValue::from_int(m.len() as i64),
        Err(_) => MbValue::from_int(-1),
    }
}

unsafe extern "C" fn dispatch_getmtime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_float(metadata_time(&s, |m| m.modified()))
}

unsafe extern "C" fn dispatch_getatime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_float(metadata_time(&s, |m| m.accessed()))
}

unsafe extern "C" fn dispatch_getctime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_float(metadata_time(&s, |m| m.created()))
}

unsafe extern "C" fn dispatch_samefile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let (Some(a), Some(b)) = (
        args.first().copied().and_then(extract_str),
        args.get(1).copied().and_then(extract_str),
    ) else {
        return MbValue::from_bool(false);
    };
    let (Ok(ma), Ok(mb)) = (std::fs::metadata(&a), std::fs::metadata(&b)) else {
        return MbValue::from_bool(false);
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return MbValue::from_bool(ma.ino() == mb.ino() && ma.dev() == mb.dev());
    }
    #[cfg(not(unix))]
    {
        let _ = (ma, mb);
        MbValue::from_bool(a == b)
    }
}

unsafe extern "C" fn dispatch_abspath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let resolved = if std::path::Path::new(&s).is_absolute() {
        posix_normpath(&s)
    } else {
        let cwd = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".to_string());
        posix_normpath(&format!("{}/{}", cwd, s))
    };
    mk_str(resolved)
}

unsafe extern "C" fn dispatch_realpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    match std::fs::canonicalize(&s) {
        Ok(p) => mk_str(p.display().to_string()),
        Err(_) => {
            // CPython realpath returns the abspath even if the file doesn't
            // exist (when strict=False, the default).
            let resolved = if std::path::Path::new(&s).is_absolute() {
                posix_normpath(&s)
            } else {
                let cwd = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".to_string());
                posix_normpath(&format!("{}/{}", cwd, s))
            };
            mk_str(resolved)
        }
    }
}

unsafe extern "C" fn dispatch_relpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let target = args.first().copied().and_then(extract_str).unwrap_or_default();
    let start = args.get(1).copied().and_then(extract_str)
        .unwrap_or_else(|| std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".to_string()));
    mk_str(relpath(&start, &target))
}

unsafe extern "C" fn dispatch_expanduser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(expand_user(&s))
}

unsafe extern "C" fn dispatch_expandvars(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(expand_vars(&s))
}

unsafe extern "C" fn dispatch_clear_cache(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

fn build_attrs() -> HashMap<String, MbValue> {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("join",         dispatch_join         as *const () as usize),
        ("split",        dispatch_split        as *const () as usize),
        ("splitext",     dispatch_splitext     as *const () as usize),
        ("splitdrive",   dispatch_splitdrive   as *const () as usize),
        ("basename",     dispatch_basename     as *const () as usize),
        ("dirname",      dispatch_dirname      as *const () as usize),
        ("isabs",        dispatch_isabs        as *const () as usize),
        ("normpath",     dispatch_normpath     as *const () as usize),
        ("normcase",     dispatch_normcase     as *const () as usize),
        ("commonprefix", dispatch_commonprefix as *const () as usize),
        ("commonpath",   dispatch_commonpath   as *const () as usize),
        ("exists",       dispatch_exists       as *const () as usize),
        ("lexists",      dispatch_exists       as *const () as usize),
        ("isfile",       dispatch_isfile       as *const () as usize),
        ("isdir",        dispatch_isdir        as *const () as usize),
        ("islink",       dispatch_islink       as *const () as usize),
        ("ismount",      dispatch_ismount      as *const () as usize),
        ("getsize",      dispatch_getsize      as *const () as usize),
        ("getmtime",     dispatch_getmtime     as *const () as usize),
        ("getatime",     dispatch_getatime     as *const () as usize),
        ("getctime",     dispatch_getctime     as *const () as usize),
        ("samefile",     dispatch_samefile     as *const () as usize),
        ("abspath",      dispatch_abspath      as *const () as usize),
        ("realpath",     dispatch_realpath     as *const () as usize),
        ("relpath",      dispatch_relpath      as *const () as usize),
        ("expanduser",   dispatch_expanduser   as *const () as usize),
        ("expandvars",   dispatch_expandvars   as *const () as usize),
        // genericpath caching helper (no-op for us).
        ("_clear_cache", dispatch_clear_cache  as *const () as usize),
    ];
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    for (name, addr) in dispatchers {
        attrs.insert((*name).to_string(), MbValue::from_func(*addr));
    }
    // Constants matching CPython posixpath.
    attrs.insert("sep".to_string(), mk_str("/".to_string()));
    attrs.insert("altsep".to_string(), MbValue::none());
    attrs.insert("extsep".to_string(), mk_str(".".to_string()));
    attrs.insert("pathsep".to_string(), mk_str(":".to_string()));
    attrs.insert("defpath".to_string(), mk_str(":/bin:/usr/bin".to_string()));
    attrs.insert("curdir".to_string(), mk_str(".".to_string()));
    attrs.insert("pardir".to_string(), mk_str("..".to_string()));
    attrs.insert("devnull".to_string(), mk_str("/dev/null".to_string()));
    attrs.insert("supports_unicode_filenames".to_string(), MbValue::from_bool(true));
    // genericpath.ALLOW_MISSING sentinel singleton (re-exported through
    // posixpath). CPython exposes it as a unique sentinel object whose repr is
    // "os.path.ALLOW_MISSING"; surface only needs the name present.
    attrs.insert("ALLOW_MISSING".to_string(),
        MbValue::from_ptr(MbObject::new_str("os.path.ALLOW_MISSING".to_string())));
    attrs
}

pub fn register() {
    super::register_module("posixpath", build_attrs());
    // genericpath shares the FS-existence/size/time surface — register a
    // trimmed view there. CPython's genericpath is the parent module of
    // posixpath/ntpath; the trimmed surface is enough for the common
    // `from genericpath import *` consumers.
    let mut g = HashMap::new();
    for k in ["exists", "isfile", "isdir", "getsize", "getmtime", "getatime",
              "getctime", "samefile", "commonprefix", "_clear_cache"] {
        if let Some(v) = build_attrs().remove(k) {
            g.insert(k.to_string(), v);
        }
    }
    super::register_module("genericpath", g);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_basic() {
        assert_eq!(posix_join_strs(&["a".to_string(), "b".to_string()]), "a/b");
        assert_eq!(posix_join_strs(&["a/".to_string(), "b".to_string()]), "a/b");
        assert_eq!(posix_join_strs(&["a".to_string(), "/b".to_string()]), "/b");
        assert_eq!(posix_join_strs(&["/a".to_string(), "b".to_string(), "c".to_string()]), "/a/b/c");
        assert_eq!(posix_join_strs(&[]), "");
    }

    #[test]
    fn split_basic() {
        assert_eq!(posix_split("a/b/c"), ("a/b".to_string(), "c".to_string()));
        assert_eq!(posix_split("/a"), ("/".to_string(), "a".to_string()));
        assert_eq!(posix_split("noslash"), ("".to_string(), "noslash".to_string()));
    }

    #[test]
    fn splitext_basic() {
        assert_eq!(posix_splitext("foo.tar.gz"), ("foo.tar".to_string(), ".gz".to_string()));
        assert_eq!(posix_splitext("foo"), ("foo".to_string(), "".to_string()));
        assert_eq!(posix_splitext(".bashrc"), (".bashrc".to_string(), "".to_string()));
        assert_eq!(posix_splitext("dir/.hidden.txt"), ("dir/.hidden".to_string(), ".txt".to_string()));
    }

    #[test]
    fn normpath_basic() {
        assert_eq!(posix_normpath("a/./b"), "a/b");
        assert_eq!(posix_normpath("a/b/../c"), "a/c");
        assert_eq!(posix_normpath("/a/b/../../c"), "/c");
        assert_eq!(posix_normpath(""), ".");
        assert_eq!(posix_normpath("//foo"), "//foo");
        assert_eq!(posix_normpath("///foo"), "/foo");
    }

    #[test]
    fn isabs_basic() {
        assert!(posix_isabs("/x"));
        assert!(!posix_isabs("x"));
        assert!(!posix_isabs(""));
    }

    #[test]
    fn basename_dirname() {
        assert_eq!(posix_basename("a/b/c"), "c");
        assert_eq!(posix_dirname("a/b/c"), "a/b");
        assert_eq!(posix_dirname("/x"), "/");
        assert_eq!(posix_basename("/x"), "x");
    }

    #[test]
    fn commonprefix_basic() {
        assert_eq!(
            posix_commonprefix_strs(&["/usr/local".to_string(), "/usr/bin".to_string()]),
            "/usr/"
        );
        assert_eq!(posix_commonprefix_strs(&[]), "");
    }

    #[test]
    fn commonpath_basic() {
        assert_eq!(
            posix_commonpath_strs(&["/usr/local".to_string(), "/usr/bin".to_string()]).unwrap(),
            "/usr"
        );
        // Mixed abs+rel
        assert!(posix_commonpath_strs(&["/a".to_string(), "b".to_string()]).is_none());
    }

    #[test]
    fn expandvars_basic() {
        std::env::set_var("MAMBA_PP_TEST", "wow");
        assert_eq!(expand_vars("$MAMBA_PP_TEST/x"), "wow/x");
        assert_eq!(expand_vars("${MAMBA_PP_TEST}.foo"), "wow.foo");
        assert_eq!(expand_vars("$UNDEFINED_XYZ_VAR"), "$UNDEFINED_XYZ_VAR");
        std::env::remove_var("MAMBA_PP_TEST");
    }

    #[test]
    fn relpath_basic() {
        assert_eq!(relpath("/a/b", "/a/b/c/d"), "c/d");
        assert_eq!(relpath("/a/b", "/a/x"), "../x");
        assert_eq!(relpath("/a/b", "/a/b"), ".");
    }

    #[test]
    fn dispatch_join_via_strs() {
        let a = MbValue::from_ptr(MbObject::new_str("foo".to_string()));
        let b = MbValue::from_ptr(MbObject::new_str("bar".to_string()));
        let args = [a, b];
        let v = unsafe { dispatch_join(args.as_ptr(), 2) };
        if let Some(p) = v.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    assert_eq!(s, "foo/bar");
                    return;
                }
            }
        }
        panic!("expected str");
    }
}
