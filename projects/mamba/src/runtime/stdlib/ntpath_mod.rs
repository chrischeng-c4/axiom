/// ntpath module for Mamba (#1261 long-tail).
///
/// Real implementation of CPython 3.12 `Lib/ntpath.py`. The long_tail
/// stub returned `""` from every path-string op and `False` from every
/// FS check — completely broken for anyone porting Windows-flavored
/// path code. This brings up the pure-string Windows path surface and
/// reuses the same FS-backed dispatchers as posixpath for existence /
/// size / time queries.
///
/// Windows path quirks honored:
///   - Both `\` and `/` count as separators; canonical form is `\`.
///   - Drive letters: `C:` is a drive prefix; `C:foo` is relative-to-
///     drive-C's-cwd, `C:\foo` is absolute on drive C.
///   - UNC paths: `\\server\share` is its own drive. `\\?\` and
///     `\\.\` device prefixes preserve the entire 4-char marker as
///     drive.
///   - `normcase` lowercases AND converts `/` to `\`.

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
            // suite's FakePath, or pathlib.PurePath). CPython's ntpath
            // functions normalize their argument via os.fspath() before
            // operating on it, so a FakePath(name) must behave identically
            // to the plain str `name`.
            ObjData::Instance { .. } => fspath_via_protocol(val),
            _ => None,
        }
    }
}

/// Call `__fspath__` on an Instance and decode the result to a String.
/// Mirrors `posixpath_mod::fspath_via_protocol` so ntpath functions accept
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

fn mk_tuple3(a: String, b: String, c: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![mk_str(a), mk_str(b), mk_str(c)]))
}

fn is_sep(c: char) -> bool { c == '\\' || c == '/' }

/// CPython 3.12 `ntpath.splitroot`: split into (drive, root, tail).
/// `drive` is exactly as in `splitdrive`; `root` is a single separator or
/// empty; `tail` is everything after the root. Works on byte indices
/// because all the structural characters (`\`, `/`, `:`) are ASCII.
fn nt_splitroot(p: &str) -> (String, String, String) {
    let normp: String = p.chars().map(|c| if c == '/' { '\\' } else { c }).collect();
    let nb = normp.as_bytes();
    let pb = p.as_bytes();
    if nb.first() == Some(&b'\\') {
        if nb.get(1) == Some(&b'\\') {
            // UNC drives, e.g. \\server\share or \\?\UNC\server\share
            // Device drives, e.g. \\.\device or \\?\device
            let unc_prefix = b"\\\\?\\UNC\\";
            let start = if nb.len() >= 8
                && normp[..8].eq_ignore_ascii_case("\\\\?\\UNC\\")
            {
                8
            } else {
                2
            };
            let _ = unc_prefix;
            match nb[start..].iter().position(|&b| b == b'\\') {
                None => (p.to_string(), String::new(), String::new()),
                Some(rel) => {
                    let index = start + rel;
                    match nb[index + 1..].iter().position(|&b| b == b'\\') {
                        None => (p.to_string(), String::new(), String::new()),
                        Some(rel2) => {
                            let index2 = index + 1 + rel2;
                            (
                                p[..index2].to_string(),
                                p[index2..index2 + 1].to_string(),
                                p[index2 + 1..].to_string(),
                            )
                        }
                    }
                }
            }
        } else {
            // Relative path with root, e.g. \Windows
            (String::new(), p[..1].to_string(), p[1..].to_string())
        }
    } else if nb.get(1) == Some(&b':') {
        if nb.get(2) == Some(&b'\\') {
            // Absolute drive-letter path, e.g. X:\Windows
            (p[..2].to_string(), p[2..3].to_string(), p[3..].to_string())
        } else {
            // Relative path with drive, e.g. X:Windows
            (p[..2].to_string(), String::new(), p[2..].to_string())
        }
    } else {
        // Relative path, e.g. Windows
        let _ = pb;
        (String::new(), String::new(), p.to_string())
    }
}

// ── Pure-string Windows ops ──

/// Split off a drive prefix from `path`. Returns (drive, rest).
/// Derived from `nt_splitroot`, exactly as CPython 3.12 does:
/// `drive, root, tail = splitroot(p); return drive, root + tail`.
fn nt_splitdrive(path: &str) -> (String, String) {
    let (drive, root, tail) = nt_splitroot(path);
    (drive, format!("{}{}", root, tail))
}

fn nt_isabs(path: &str) -> bool {
    // CPython 3.12 ntpath.isabs: look at the first 3 chars (with / -> \).
    // Absolute iff it starts with a separator (UNC/device/rooted) OR has a
    // drive letter immediately followed by a separator (e.g. "C:\").
    // LEGACY BUG (matched on purpose): isabs("/x") is True here because the
    // 3.12 impl tests `startswith(sep)`.
    let prefix: String = path.chars().take(3)
        .map(|c| if c == '/' { '\\' } else { c })
        .collect();
    let pb = prefix.as_bytes();
    if pb.first() == Some(&b'\\') {
        return true;
    }
    // colon_sep == ":\\" checked at index 1
    pb.get(1) == Some(&b':') && pb.get(2) == Some(&b'\\')
}

fn nt_normcase(path: &str) -> String {
    let lowered: String = path.chars().map(|c| {
        if c == '/' { '\\' } else { c.to_ascii_lowercase() }
    }).collect();
    lowered
}

/// CPython 3.12 ntpath.join: drive/root-aware concatenation via splitroot.
fn nt_join_strs(parts: &[String]) -> String {
    if parts.is_empty() { return String::new(); }
    let (mut result_drive, mut result_root, mut result_path) = nt_splitroot(&parts[0]);
    for tail in &parts[1..] {
        let (p_drive, p_root, p_path) = nt_splitroot(tail);
        if !p_root.is_empty() {
            // Second path is absolute.
            if !p_drive.is_empty() || result_drive.is_empty() {
                result_drive = p_drive;
            }
            result_root = p_root;
            result_path = p_path;
            continue;
        } else if !p_drive.is_empty() && p_drive != result_drive {
            if p_drive.to_lowercase() != result_drive.to_lowercase() {
                // Different drives => ignore the first path entirely.
                result_drive = p_drive;
                result_root = p_root;
                result_path = p_path;
                continue;
            }
            // Same drive in different case.
            result_drive = p_drive;
        }
        // Second path is relative to the first.
        if !result_path.is_empty()
            && !result_path.ends_with('\\') && !result_path.ends_with('/')
        {
            result_path.push('\\');
        }
        result_path.push_str(&p_path);
    }
    // Add separator between UNC and non-absolute path.
    if !result_path.is_empty()
        && result_root.is_empty()
        && !result_drive.is_empty()
        && !result_drive.ends_with(':')
        && !result_drive.ends_with('\\')
        && !result_drive.ends_with('/')
    {
        return format!("{}\\{}", result_drive, result_path);
    }
    format!("{}{}{}", result_drive, result_root, result_path)
}

fn nt_split(path: &str) -> (String, String) {
    // CPython 3.12: d, r, p = splitroot(p); split p at last sep; the head
    // is d + r + (head before last sep, with trailing seps stripped).
    let (drive, root, rest) = nt_splitroot(path);
    let rb = rest.as_bytes();
    let mut i = rb.len();
    while i > 0 && rb[i - 1] != b'\\' && rb[i - 1] != b'/' {
        i -= 1;
    }
    let head = &rest[..i];
    let tail = &rest[i..];
    // Strip trailing separators from head.
    let mut end = head.len();
    while end > 0 {
        let b = head.as_bytes()[end - 1];
        if b == b'\\' || b == b'/' { end -= 1; } else { break; }
    }
    (format!("{}{}{}", drive, root, &head[..end]), tail.to_string())
}

fn nt_basename(path: &str) -> String { nt_split(path).1 }
fn nt_dirname(path: &str) -> String { nt_split(path).0 }

fn nt_splitext(path: &str) -> (String, String) {
    // Find the basename — search after the last sep AND drive prefix.
    let (drive, rest) = nt_splitdrive(path);
    let base_start_in_rest = rest.bytes().rposition(|b| b == b'\\' || b == b'/').map(|i| i + 1).unwrap_or(0);
    let base = &rest[base_start_in_rest..];
    let dot_search_start = base.bytes().position(|b| b != b'.').unwrap_or(base.len());
    if let Some(rel_pos) = base[dot_search_start..].rfind('.') {
        let abs_pos = drive.len() + base_start_in_rest + dot_search_start + rel_pos;
        (path[..abs_pos].to_string(), path[abs_pos..].to_string())
    } else {
        (path.to_string(), String::new())
    }
}

fn nt_normpath(path: &str) -> String {
    // CPython 3.12 pure-python normpath fallback.
    let unified: String = path.chars().map(|c| if c == '/' { '\\' } else { c }).collect();
    let (drive, root, rest) = nt_splitroot(&unified);
    let prefix = format!("{}{}", drive, root);
    let root_nonempty = !root.is_empty();
    let mut comps: Vec<String> = rest.split('\\').map(|s| s.to_string()).collect();
    let mut i = 0usize;
    while i < comps.len() {
        if comps[i].is_empty() || comps[i] == "." {
            comps.remove(i);
        } else if comps[i] == ".." {
            if i > 0 && comps[i - 1] != ".." {
                comps.drain(i - 1..=i);
                i -= 1;
            } else if i == 0 && root_nonempty {
                comps.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    // If the path is now empty, substitute '.'.
    if prefix.is_empty() && comps.is_empty() {
        comps.push(".".to_string());
    }
    format!("{}{}", prefix, comps.join("\\"))
}

fn nt_commonprefix_strs(strs: &[String]) -> String {
    if strs.is_empty() { return String::new(); }
    let first = &strs[0];
    let mut end = first.len();
    for s in &strs[1..] {
        let f = first.as_bytes();
        let p = s.as_bytes();
        let mut i = 0;
        while i < end && i < p.len() && f[i] == p[i] { i += 1; }
        end = i;
        if end == 0 { break; }
    }
    while end > 0 && !first.is_char_boundary(end) { end -= 1; }
    first[..end].to_string()
}

fn nt_commonpath_strs(strs: &[String]) -> Option<String> {
    if strs.is_empty() { return Some(String::new()); }
    // Unify path separators and split off drive.
    let unified: Vec<String> = strs.iter().map(|s| s.replace('/', "\\")).collect();
    let drives_paths: Vec<(String, String)> = unified.iter().map(|p| nt_splitdrive(p)).collect();
    let first_drive = drives_paths[0].0.to_lowercase();
    for (d, _) in &drives_paths[1..] {
        if d.to_lowercase() != first_drive { return None; }
    }
    let abs = drives_paths[0].1.starts_with('\\');
    for (_, rest) in &drives_paths {
        if rest.starts_with('\\') != abs { return None; }
    }
    let split_parts: Vec<Vec<&str>> = drives_paths.iter()
        .map(|(_, r)| r.split('\\').filter(|s| !s.is_empty() && *s != ".").collect())
        .collect();
    let min_len = split_parts.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut common: Vec<&str> = Vec::new();
    for i in 0..min_len {
        let part = split_parts[0][i];
        let same = split_parts.iter().all(|v| v[i].eq_ignore_ascii_case(part));
        if same { common.push(part); } else { break; }
    }
    let prefix = if abs { "\\" } else { "" };
    Some(format!("{}{}{}", drives_paths[0].0, prefix, common.join("\\")))
}

fn expand_user(path: &str) -> String {
    if !path.starts_with('~') { return path.to_string(); }
    let split_idx = path.find(|c| c == '\\' || c == '/').unwrap_or(path.len());
    let user_part = &path[..split_idx];
    let rest = &path[split_idx..];
    if user_part == "~" {
        if let Ok(p) = std::env::var("USERPROFILE") {
            return format!("{}{}", p, rest);
        }
        if let (Ok(drive), Ok(hpath)) = (std::env::var("HOMEDRIVE"), std::env::var("HOMEPATH")) {
            return format!("{}{}{}", drive, hpath, rest);
        }
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}{}", home, rest);
        }
    }
    path.to_string()
}

fn expand_vars(path: &str) -> String {
    // ntpath also supports `%VAR%` syntax in addition to `$VAR` / `${VAR}`.
    let bytes = path.as_bytes();
    let mut out = String::with_capacity(path.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' {
            if let Some(end_rel) = path[i + 1..].find('%') {
                let name = &path[i + 1..i + 1 + end_rel];
                if let Ok(val) = std::env::var(name) {
                    out.push_str(&val);
                } else {
                    out.push_str(&path[i..i + 2 + end_rel]);
                }
                i += 2 + end_rel;
                continue;
            }
        } else if b == b'$' && i + 1 < bytes.len() {
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
    let start_norm = nt_normpath(start);
    let target_norm = nt_normpath(target);
    let (s_drive, s_rest) = nt_splitdrive(&start_norm);
    let (t_drive, t_rest) = nt_splitdrive(&target_norm);
    if s_drive.to_lowercase() != t_drive.to_lowercase() {
        // Different drives — no relative path expressible.
        return target_norm;
    }
    let s_parts: Vec<&str> = s_rest.split('\\').filter(|s| !s.is_empty()).collect();
    let t_parts: Vec<&str> = t_rest.split('\\').filter(|s| !s.is_empty()).collect();
    let mut common = 0;
    while common < s_parts.len() && common < t_parts.len()
        && s_parts[common].eq_ignore_ascii_case(t_parts[common])
    {
        common += 1;
    }
    let ups = std::iter::repeat("..").take(s_parts.len() - common);
    let downs = t_parts[common..].iter().copied();
    let joined: Vec<&str> = ups.chain(downs).collect();
    if joined.is_empty() {
        ".".to_string()
    } else {
        joined.join("\\")
    }
}

// ── Dispatchers ──

unsafe extern "C" fn dispatch_join(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let strs: Vec<String> = args.iter().filter_map(|v| extract_str(*v)).collect();
    mk_str(nt_join_strs(&strs))
}

unsafe extern "C" fn dispatch_split(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(s) = args.first().copied().and_then(extract_str) else {
        return mk_tuple2(String::new(), String::new());
    };
    let (a, b) = nt_split(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_basename(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(nt_basename(&s))
}

unsafe extern "C" fn dispatch_dirname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(nt_dirname(&s))
}

unsafe extern "C" fn dispatch_splitext(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let (a, b) = nt_splitext(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_splitdrive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let (a, b) = nt_splitdrive(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_splitroot(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let (a, b, c) = nt_splitroot(&s);
    mk_tuple3(a, b, c)
}

unsafe extern "C" fn dispatch_isabs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    MbValue::from_bool(nt_isabs(&s))
}

unsafe extern "C" fn dispatch_normpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(nt_normpath(&s))
}

unsafe extern "C" fn dispatch_normcase(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    mk_str(nt_normcase(&s))
}

unsafe extern "C" fn dispatch_commonprefix(args_ptr: *const MbValue, nargs: usize) -> MbValue {
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
    mk_str(nt_commonprefix_strs(&strs))
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
    match nt_commonpath_strs(&strs) {
        Some(s) => mk_str(s),
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "Paths don't have the same drive".to_string(),
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
    // Windows ismount: a drive root like "C:\" or a UNC share root.
    let normalized = nt_normpath(&s);
    let (drive, rest) = nt_splitdrive(&normalized);
    let is_drive_root = !drive.is_empty() && (rest == "\\" || rest == "/" || rest.is_empty());
    MbValue::from_bool(is_drive_root)
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
        MbValue::from_bool(nt_normcase(&a) == nt_normcase(&b))
    }
}

unsafe extern "C" fn dispatch_abspath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    let resolved = if nt_isabs(&s) {
        nt_normpath(&s)
    } else {
        let cwd = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".to_string());
        nt_normpath(&nt_join_strs(&[cwd, s]))
    };
    mk_str(resolved)
}

unsafe extern "C" fn dispatch_realpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args.first().copied().and_then(extract_str).unwrap_or_default();
    match std::fs::canonicalize(&s) {
        Ok(p) => mk_str(p.display().to_string()),
        Err(_) => {
            let resolved = if nt_isabs(&s) {
                nt_normpath(&s)
            } else {
                let cwd = std::env::current_dir().ok().map(|p| p.display().to_string()).unwrap_or_else(|| ".".to_string());
                nt_normpath(&nt_join_strs(&[cwd, s]))
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

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("join",         dispatch_join         as *const () as usize),
        ("split",        dispatch_split        as *const () as usize),
        ("splitext",     dispatch_splitext     as *const () as usize),
        ("splitdrive",   dispatch_splitdrive   as *const () as usize),
        ("splitroot",    dispatch_splitroot    as *const () as usize),
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
    // Constants matching CPython ntpath.
    attrs.insert("sep".to_string(),    mk_str("\\".to_string()));
    attrs.insert("altsep".to_string(), mk_str("/".to_string()));
    attrs.insert("extsep".to_string(), mk_str(".".to_string()));
    attrs.insert("pathsep".to_string(), mk_str(";".to_string()));
    attrs.insert("defpath".to_string(), mk_str(".;C:\\bin".to_string()));
    attrs.insert("curdir".to_string(), mk_str(".".to_string()));
    attrs.insert("pardir".to_string(), mk_str("..".to_string()));
    attrs.insert("devnull".to_string(), mk_str("nul".to_string()));
    attrs.insert("supports_unicode_filenames".to_string(), MbValue::from_bool(true));
    // os.path.ALLOW_MISSING sentinel: ntpath re-exports it for realpath's
    // `strict=` parameter. Fixtures only `from ntpath import ALLOW_MISSING`
    // and pass it through to realpath (which ignores extra kwargs here), so
    // a distinct opaque marker string suffices.
    attrs.insert("ALLOW_MISSING".to_string(), mk_str("os.path.ALLOW_MISSING".to_string()));
    super::register_module("ntpath", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitdrive_letter() {
        assert_eq!(nt_splitdrive("C:\\foo"), ("C:".to_string(), "\\foo".to_string()));
        assert_eq!(nt_splitdrive("C:foo"),    ("C:".to_string(), "foo".to_string()));
        assert_eq!(nt_splitdrive("foo"),      ("".to_string(),   "foo".to_string()));
    }

    #[test]
    fn splitdrive_unc() {
        assert_eq!(
            nt_splitdrive("\\\\server\\share\\foo"),
            ("\\\\server\\share".to_string(), "\\foo".to_string()),
        );
        assert_eq!(
            nt_splitdrive("\\\\server\\share"),
            ("\\\\server\\share".to_string(), "".to_string()),
        );
    }

    #[test]
    fn isabs_drive_with_root() {
        assert!(nt_isabs("C:\\foo"));
        assert!(nt_isabs("\\\\server\\share\\foo"));
        assert!(!nt_isabs("C:foo"));
        assert!(!nt_isabs("foo"));
    }

    #[test]
    fn normcase_lowercases_and_normalizes_separators() {
        assert_eq!(nt_normcase("C:/Foo/Bar"), "c:\\foo\\bar");
        assert_eq!(nt_normcase("ABC"), "abc");
    }

    #[test]
    fn join_drive_switch() {
        assert_eq!(
            nt_join_strs(&["C:\\a".into(), "D:\\b".into()]),
            "D:\\b",
        );
    }

    #[test]
    fn join_root_resets() {
        assert_eq!(
            nt_join_strs(&["C:\\foo".into(), "\\bar".into()]),
            "C:\\bar",
        );
    }

    #[test]
    fn join_simple_concat() {
        assert_eq!(
            nt_join_strs(&["foo".into(), "bar".into(), "baz".into()]),
            "foo\\bar\\baz",
        );
    }

    #[test]
    fn split_basic() {
        assert_eq!(nt_split("C:\\a\\b\\c"), ("C:\\a\\b".to_string(), "c".to_string()));
        assert_eq!(nt_split("\\a"),         ("\\".to_string(),     "a".to_string()));
        assert_eq!(nt_split("noslash"),     ("".to_string(),       "noslash".to_string()));
    }

    #[test]
    fn splitext_basic() {
        assert_eq!(nt_splitext("foo.txt"),       ("foo".to_string(),      ".txt".to_string()));
        assert_eq!(nt_splitext("C:\\a\\b.txt"),  ("C:\\a\\b".to_string(), ".txt".to_string()));
        assert_eq!(nt_splitext(".hidden"),       (".hidden".to_string(),  String::new()));
    }

    #[test]
    fn normpath_collapses_dotdot() {
        assert_eq!(nt_normpath("C:\\foo\\..\\bar"), "C:\\bar");
        assert_eq!(nt_normpath("foo/bar/../baz"),   "foo\\baz");
        assert_eq!(nt_normpath(""), ".");
    }

    #[test]
    fn normpath_preserves_drive() {
        // CPython 3.12: ntpath.normpath('C:') == 'C:' (bare drive stays bare).
        assert_eq!(nt_normpath("C:"), "C:");
    }

    #[test]
    fn commonpath_matches_drive() {
        let r = nt_commonpath_strs(&["C:\\a\\b".into(), "C:\\a\\c".into()]).unwrap();
        assert_eq!(r, "C:\\a");
    }

    #[test]
    fn commonpath_rejects_different_drives() {
        assert!(nt_commonpath_strs(&["C:\\a".into(), "D:\\a".into()]).is_none());
    }

    #[test]
    fn expand_vars_percent_form() {
        std::env::set_var("NTPATH_TEST_VAR", "Z");
        assert_eq!(expand_vars("%NTPATH_TEST_VAR%\\foo"), "Z\\foo");
    }

    #[test]
    fn relpath_within_same_drive() {
        assert_eq!(relpath("C:\\a\\b", "C:\\a\\c"), "..\\c");
    }

    #[test]
    fn basename_with_drive() {
        assert_eq!(nt_basename("C:\\a\\b.txt"), "b.txt");
        assert_eq!(nt_dirname("C:\\a\\b.txt"), "C:\\a");
    }
}
