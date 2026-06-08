use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
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
        if let ObjData::Str(ref s) = (*p).data {
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

fn is_sep(c: char) -> bool {
    c == '\\' || c == '/'
}

// ── Pure-string Windows ops ──

/// Split off a drive prefix from `path`. Returns (drive, rest).
/// Drive forms:
///   - Drive letter: "C:" (must be ASCII letter + ':').
///   - UNC root: "\\server\share" — drive includes everything up through the share name.
///   - Device prefix: "\\?\X" or "\\.\X" — drive is the 4-char prefix or the next component.
fn nt_splitdrive(path: &str) -> (String, String) {
    let bytes = path.as_bytes();
    if bytes.len() < 2 {
        return (String::new(), path.to_string());
    }
    // UNC / device prefixes
    if bytes.len() >= 2 && is_sep(bytes[0] as char) && is_sep(bytes[1] as char) {
        // Find the third separator. The drive is everything up to it.
        // `\\?\` and `\\.\` are special — they keep their marker as part of drive.
        let mut idx = 2usize;
        while idx < bytes.len() && !is_sep(bytes[idx] as char) {
            idx += 1;
        }
        if idx >= bytes.len() {
            return (path.to_string(), String::new());
        }
        // Find the second separator (end of share name).
        let mut idx2 = idx + 1;
        while idx2 < bytes.len() && !is_sep(bytes[idx2] as char) {
            idx2 += 1;
        }
        if idx2 == idx + 1 {
            // Empty share name between separators — treat the whole path as the rest.
            return (String::new(), path.to_string());
        }
        return (path[..idx2].to_string(), path[idx2..].to_string());
    }
    // Drive-letter form
    if bytes[1] == b':' && (bytes[0] as char).is_ascii_alphabetic() {
        return (path[..2].to_string(), path[2..].to_string());
    }
    (String::new(), path.to_string())
}

fn nt_isabs(path: &str) -> bool {
    // CPython 3.13+ semantics: a path with both drive and root is absolute.
    let (drive, rest) = nt_splitdrive(path);
    if drive.starts_with('\\') || drive.starts_with('/') {
        // UNC form: always absolute.
        return true;
    }
    // Drive-letter form: needs rest starting with sep to be absolute.
    rest.starts_with('\\') || rest.starts_with('/')
}

fn nt_normcase(path: &str) -> String {
    let lowered: String = path
        .chars()
        .map(|c| {
            if c == '/' {
                '\\'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    lowered
}

/// CPython ntpath.join: drive-aware concatenation.
fn nt_join_strs(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let (d, p) = nt_splitdrive(&parts[0]);
    let mut result_drive = d;
    let mut result_path = p;
    for tail in &parts[1..] {
        let (p_drive, p_path) = nt_splitdrive(tail);
        if p_path.starts_with('\\') || p_path.starts_with('/') {
            // Second path is rooted — discard current path.
            if !p_drive.is_empty() || result_drive.is_empty() {
                result_drive = p_drive;
            }
            result_path = p_path;
            continue;
        }
        if !p_drive.is_empty() && p_drive != result_drive {
            if !p_drive.eq_ignore_ascii_case(&result_drive) {
                // Different drive — second path wins entirely.
                result_drive = p_drive;
                result_path = p_path;
                continue;
            }
            result_drive = p_drive;
        }
        // Same drive (or no drive in tail) — concat.
        if !result_path.is_empty() && !result_path.ends_with('\\') && !result_path.ends_with('/') {
            result_path.push('\\');
        }
        result_path.push_str(&p_path);
    }
    // If the path is non-empty and has a drive but no separator after it,
    // CPython leaves it as-is (e.g. "C:foo").
    let needs_sep_after_drive = !result_drive.is_empty()
        && !result_path.is_empty()
        && !result_path.starts_with('\\')
        && !result_path.starts_with('/')
        && !result_drive.ends_with(':');
    if needs_sep_after_drive {
        result_path.insert(0, '\\');
    }
    format!("{}{}", result_drive, result_path)
}

fn nt_split(path: &str) -> (String, String) {
    let (drive, rest) = nt_splitdrive(path);
    // Find last separator in rest.
    let last_sep = rest.bytes().rposition(|b| b == b'\\' || b == b'/');
    let (head_rest, tail) = match last_sep {
        Some(i) => (&rest[..i + 1], &rest[i + 1..]),
        None => ("", rest.as_str()),
    };
    // Strip trailing separators from head, unless head is all separators.
    let all_seps = !head_rest.is_empty() && head_rest.bytes().all(|b| b == b'\\' || b == b'/');
    let head_trimmed = if all_seps {
        head_rest.to_string()
    } else {
        let mut end = head_rest.len();
        while end > 0 {
            let b = head_rest.as_bytes()[end - 1];
            if b == b'\\' || b == b'/' {
                end -= 1;
            } else {
                break;
            }
        }
        head_rest[..end].to_string()
    };
    (format!("{}{}", drive, head_trimmed), tail.to_string())
}

fn nt_basename(path: &str) -> String {
    nt_split(path).1
}
fn nt_dirname(path: &str) -> String {
    nt_split(path).0
}

fn nt_splitext(path: &str) -> (String, String) {
    // Find the basename — search after the last sep AND drive prefix.
    let (drive, rest) = nt_splitdrive(path);
    let base_start_in_rest = rest
        .bytes()
        .rposition(|b| b == b'\\' || b == b'/')
        .map(|i| i + 1)
        .unwrap_or(0);
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
    if path.is_empty() {
        return ".".to_string();
    }
    // First convert all `/` to `\`.
    let unified: String = path
        .chars()
        .map(|c| if c == '/' { '\\' } else { c })
        .collect();
    let (drive, rest) = nt_splitdrive(&unified);
    let is_absolute = rest.starts_with('\\');
    let trimmed = rest.trim_start_matches('\\');
    let parts: Vec<&str> = trimmed
        .split('\\')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let mut comps: Vec<&str> = Vec::new();
    for part in parts {
        if part == ".." {
            if !comps.is_empty() && *comps.last().unwrap() != ".." {
                comps.pop();
            } else if !is_absolute {
                comps.push("..");
            }
        } else {
            comps.push(part);
        }
    }
    let joined = comps.join("\\");
    let prefix = if is_absolute { "\\" } else { "" };
    let body = format!("{}{}", prefix, joined);
    let result = format!("{}{}", drive, body);
    if result.is_empty() {
        ".".to_string()
    } else if !drive.is_empty() && body.is_empty() {
        // Bare drive like "C:" stays as "C:.".
        format!("{}.", drive)
    } else {
        result
    }
}

fn nt_commonprefix_strs(strs: &[String]) -> String {
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
    while end > 0 && !first.is_char_boundary(end) {
        end -= 1;
    }
    first[..end].to_string()
}

fn nt_commonpath_strs(strs: &[String]) -> Option<String> {
    if strs.is_empty() {
        return Some(String::new());
    }
    // Unify path separators and split off drive.
    let unified: Vec<String> = strs.iter().map(|s| s.replace('/', "\\")).collect();
    let drives_paths: Vec<(String, String)> = unified.iter().map(|p| nt_splitdrive(p)).collect();
    let first_drive = drives_paths[0].0.to_lowercase();
    for (d, _) in &drives_paths[1..] {
        if d.to_lowercase() != first_drive {
            return None;
        }
    }
    let abs = drives_paths[0].1.starts_with('\\');
    for (_, rest) in &drives_paths {
        if rest.starts_with('\\') != abs {
            return None;
        }
    }
    let split_parts: Vec<Vec<&str>> = drives_paths
        .iter()
        .map(|(_, r)| {
            r.split('\\')
                .filter(|s| !s.is_empty() && *s != ".")
                .collect()
        })
        .collect();
    let min_len = split_parts.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut common: Vec<&str> = Vec::new();
    for i in 0..min_len {
        let part = split_parts[0][i];
        let same = split_parts.iter().all(|v| v[i].eq_ignore_ascii_case(part));
        if same {
            common.push(part);
        } else {
            break;
        }
    }
    let prefix = if abs { "\\" } else { "" };
    Some(format!(
        "{}{}{}",
        drives_paths[0].0,
        prefix,
        common.join("\\")
    ))
}

fn expand_user(path: &str) -> String {
    if !path.starts_with('~') {
        return path.to_string();
    }
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
    let Ok(meta) = std::fs::metadata(p) else {
        return 0.0;
    };
    let Ok(t) = getter(&meta) else {
        return 0.0;
    };
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
    while common < s_parts.len()
        && common < t_parts.len()
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
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(nt_basename(&s))
}

unsafe extern "C" fn dispatch_dirname(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(nt_dirname(&s))
}

unsafe extern "C" fn dispatch_splitext(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let (a, b) = nt_splitext(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_splitdrive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let (a, b) = nt_splitdrive(&s);
    mk_tuple2(a, b)
}

unsafe extern "C" fn dispatch_isabs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_bool(nt_isabs(&s))
}

unsafe extern "C" fn dispatch_normpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(nt_normpath(&s))
}

unsafe extern "C" fn dispatch_normcase(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(nt_normcase(&s))
}

unsafe extern "C" fn dispatch_commonprefix(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let strs: Vec<String> = if args.len() == 1 {
        if let Some(p) = args[0].as_ptr() {
            unsafe {
                match &(*p).data {
                    ObjData::List(lock) => lock
                        .read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect(),
                    ObjData::Tuple(items) => items.iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Str(_) => vec![extract_str(args[0]).unwrap()],
                    _ => Vec::new(),
                }
            }
        } else {
            Vec::new()
        }
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
                    ObjData::List(lock) => lock
                        .read()
                        .unwrap()
                        .iter()
                        .filter_map(|v| extract_str(*v))
                        .collect(),
                    ObjData::Tuple(items) => items.iter().filter_map(|v| extract_str(*v)).collect(),
                    ObjData::Str(_) => vec![extract_str(args[0]).unwrap()],
                    _ => Vec::new(),
                }
            }
        } else {
            Vec::new()
        }
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
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).exists())
}

unsafe extern "C" fn dispatch_isfile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).is_file())
}

unsafe extern "C" fn dispatch_isdir(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_bool(std::path::Path::new(&s).is_dir())
}

unsafe extern "C" fn dispatch_islink(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let is_link = std::fs::symlink_metadata(&s)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    MbValue::from_bool(is_link)
}

unsafe extern "C" fn dispatch_ismount(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    // Windows ismount: a drive root like "C:\" or a UNC share root.
    let normalized = nt_normpath(&s);
    let (drive, rest) = nt_splitdrive(&normalized);
    let is_drive_root = !drive.is_empty() && (rest == "\\" || rest == "/" || rest.is_empty());
    MbValue::from_bool(is_drive_root)
}

unsafe extern "C" fn dispatch_getsize(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    match std::fs::metadata(&s) {
        Ok(m) => MbValue::from_int(m.len() as i64),
        Err(_) => MbValue::from_int(-1),
    }
}

unsafe extern "C" fn dispatch_getmtime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_float(metadata_time(&s, |m| m.modified()))
}

unsafe extern "C" fn dispatch_getatime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    MbValue::from_float(metadata_time(&s, |m| m.accessed()))
}

unsafe extern "C" fn dispatch_getctime(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
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
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let resolved = if nt_isabs(&s) {
        nt_normpath(&s)
    } else {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        nt_normpath(&nt_join_strs(&[cwd, s]))
    };
    mk_str(resolved)
}

unsafe extern "C" fn dispatch_realpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    match std::fs::canonicalize(&s) {
        Ok(p) => mk_str(p.display().to_string()),
        Err(_) => {
            let resolved = if nt_isabs(&s) {
                nt_normpath(&s)
            } else {
                let cwd = std::env::current_dir()
                    .ok()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".to_string());
                nt_normpath(&nt_join_strs(&[cwd, s]))
            };
            mk_str(resolved)
        }
    }
}

unsafe extern "C" fn dispatch_relpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let target = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    let start = args
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| ".".to_string())
        });
    mk_str(relpath(&start, &target))
}

unsafe extern "C" fn dispatch_expanduser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(expand_user(&s))
}

unsafe extern "C" fn dispatch_expandvars(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let s = args
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    mk_str(expand_vars(&s))
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("join", dispatch_join as *const () as usize),
        ("split", dispatch_split as *const () as usize),
        ("splitext", dispatch_splitext as *const () as usize),
        ("splitdrive", dispatch_splitdrive as *const () as usize),
        ("basename", dispatch_basename as *const () as usize),
        ("dirname", dispatch_dirname as *const () as usize),
        ("isabs", dispatch_isabs as *const () as usize),
        ("normpath", dispatch_normpath as *const () as usize),
        ("normcase", dispatch_normcase as *const () as usize),
        ("commonprefix", dispatch_commonprefix as *const () as usize),
        ("commonpath", dispatch_commonpath as *const () as usize),
        ("exists", dispatch_exists as *const () as usize),
        ("lexists", dispatch_exists as *const () as usize),
        ("isfile", dispatch_isfile as *const () as usize),
        ("isdir", dispatch_isdir as *const () as usize),
        ("islink", dispatch_islink as *const () as usize),
        ("ismount", dispatch_ismount as *const () as usize),
        ("getsize", dispatch_getsize as *const () as usize),
        ("getmtime", dispatch_getmtime as *const () as usize),
        ("getatime", dispatch_getatime as *const () as usize),
        ("getctime", dispatch_getctime as *const () as usize),
        ("samefile", dispatch_samefile as *const () as usize),
        ("abspath", dispatch_abspath as *const () as usize),
        ("realpath", dispatch_realpath as *const () as usize),
        ("relpath", dispatch_relpath as *const () as usize),
        ("expanduser", dispatch_expanduser as *const () as usize),
        ("expandvars", dispatch_expandvars as *const () as usize),
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
    attrs.insert("sep".to_string(), mk_str("\\".to_string()));
    attrs.insert("altsep".to_string(), mk_str("/".to_string()));
    attrs.insert("extsep".to_string(), mk_str(".".to_string()));
    attrs.insert("pathsep".to_string(), mk_str(";".to_string()));
    attrs.insert("defpath".to_string(), mk_str(".;C:\\bin".to_string()));
    attrs.insert("curdir".to_string(), mk_str(".".to_string()));
    attrs.insert("pardir".to_string(), mk_str("..".to_string()));
    attrs.insert("devnull".to_string(), mk_str("nul".to_string()));
    attrs.insert(
        "supports_unicode_filenames".to_string(),
        MbValue::from_bool(true),
    );
    super::register_module("ntpath", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitdrive_letter() {
        assert_eq!(
            nt_splitdrive("C:\\foo"),
            ("C:".to_string(), "\\foo".to_string())
        );
        assert_eq!(
            nt_splitdrive("C:foo"),
            ("C:".to_string(), "foo".to_string())
        );
        assert_eq!(nt_splitdrive("foo"), ("".to_string(), "foo".to_string()));
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
        assert_eq!(nt_join_strs(&["C:\\a".into(), "D:\\b".into()]), "D:\\b",);
    }

    #[test]
    fn join_root_resets() {
        assert_eq!(nt_join_strs(&["C:\\foo".into(), "\\bar".into()]), "C:\\bar",);
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
        assert_eq!(
            nt_split("C:\\a\\b\\c"),
            ("C:\\a\\b".to_string(), "c".to_string())
        );
        assert_eq!(nt_split("\\a"), ("\\".to_string(), "a".to_string()));
        assert_eq!(nt_split("noslash"), ("".to_string(), "noslash".to_string()));
    }

    #[test]
    fn splitext_basic() {
        assert_eq!(
            nt_splitext("foo.txt"),
            ("foo".to_string(), ".txt".to_string())
        );
        assert_eq!(
            nt_splitext("C:\\a\\b.txt"),
            ("C:\\a\\b".to_string(), ".txt".to_string())
        );
        assert_eq!(
            nt_splitext(".hidden"),
            (".hidden".to_string(), String::new())
        );
    }

    #[test]
    fn normpath_collapses_dotdot() {
        assert_eq!(nt_normpath("C:\\foo\\..\\bar"), "C:\\bar");
        assert_eq!(nt_normpath("foo/bar/../baz"), "foo\\baz");
        assert_eq!(nt_normpath(""), ".");
    }

    #[test]
    fn normpath_preserves_drive() {
        assert_eq!(nt_normpath("C:"), "C:.");
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
