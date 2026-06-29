use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use crate::source::FileId;
/// compileall module for Mamba (#1261).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `compileall` entry points (`compile_dir`, `compile_file`,
/// `compile_path`, `main`). All four return identity-stable
/// sentinel callables; their job here is to short-circuit
/// CPython's module-dict probe chain for read-only compileall
/// sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1261; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the
/// stub-only conversion long-tail has closed against.
use std::collections::HashMap;

fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn extract_str_v(v: MbValue) -> Option<String> {
    // Accept str and os.PathLike (pathlib.Path) inputs.
    super::pathlib_mod::coerce_fspath(v)
}

fn compileall_emit(s: &str) {
    if !super::super::output::write_captured(s) {
        print!("{s}");
    }
}

fn quiet_level(a: &[MbValue]) -> i64 {
    match kwarg(a, "quiet") {
        Some(v) if !v.is_none() => {
            if let Some(b) = v.as_bool() {
                if b { 1 } else { 0 }
            } else {
                v.as_int().unwrap_or(0)
            }
        }
        _ => 0,
    }
}

fn ascii_backslash_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            let code = ch as u32;
            if code <= 0xffff {
                out.push_str(&format!("\\u{code:04x}"));
            } else {
                out.push_str(&format!("\\U{code:08x}"));
            }
        }
    }
    out
}

fn ident_byte(b: u8) -> bool {
    b == b'_' || b.is_ascii_alphanumeric()
}

fn bytes_literal_quote_start(bytes: &[u8], i: usize) -> Option<usize> {
    let first = *bytes.get(i)?;
    let prev_is_ident = i > 0 && ident_byte(bytes[i - 1]);
    if prev_is_ident {
        return None;
    }
    if matches!(first, b'b' | b'B') {
        let mut j = i + 1;
        if matches!(bytes.get(j), Some(b'r' | b'R')) {
            j += 1;
        }
        if matches!(bytes.get(j), Some(b'\'' | b'"')) {
            return Some(j);
        }
    }
    if matches!(first, b'r' | b'R') && matches!(bytes.get(i + 1), Some(b'b' | b'B')) {
        let j = i + 2;
        if matches!(bytes.get(j), Some(b'\'' | b'"')) {
            return Some(j);
        }
    }
    None
}

fn non_ascii_bytes_literal_line(src: &str) -> Option<usize> {
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let Some(qpos) = bytes_literal_quote_start(bytes, i) else {
            i += 1;
            continue;
        };
        let quote = bytes[qpos];
        let triple =
            qpos + 2 < bytes.len() && bytes[qpos + 1] == quote && bytes[qpos + 2] == quote;
        let mut k = qpos + if triple { 3 } else { 1 };
        while k < bytes.len() {
            if bytes[k] == b'\\' {
                k = (k + 2).min(bytes.len());
                continue;
            }
            if triple {
                if k + 2 < bytes.len()
                    && bytes[k] == quote
                    && bytes[k + 1] == quote
                    && bytes[k + 2] == quote
                {
                    i = k + 3;
                    break;
                }
            } else if bytes[k] == quote {
                i = k + 1;
                break;
            }
            if bytes[k] >= 0x80 {
                return Some(src[..k].bytes().filter(|b| *b == b'\n').count() + 1);
            }
            k += 1;
        }
        if k >= bytes.len() {
            i = k;
        }
    }
    None
}

fn emit_syntax_report(path: &str, line: usize, message: &str, a: &[MbValue]) {
    if quiet_level(a) >= 2 {
        return;
    }
    compileall_emit(&format!(
        "***   File \"{}\", line {}\nSyntaxError: {}\n\n",
        ascii_backslash_escape(path),
        line,
        message
    ));
}

/// CPython's cache tag for this oracle (Python 3.12).
const CACHE_TAG: &str = "cpython-312";

/// Compute the __pycache__ cache-file path for `src` at optimization `opt`
/// (level <= 0 → no `.opt-N` suffix), mirroring importlib.util.cache_from_source.
fn cache_path_for(src: &str, opt: i64) -> String {
    let p = std::path::Path::new(src);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let parent = p.parent().map(|x| x.to_path_buf()).unwrap_or_default();
    let suffix = if opt > 0 { format!(".opt-{opt}") } else { String::new() };
    parent
        .join("__pycache__")
        .join(format!("{stem}.{CACHE_TAG}{suffix}.pyc"))
        .to_string_lossy()
        .into_owned()
}

/// Placeholder .pyc bytes — mamba is AOT-native, so there is no real bytecode;
/// the conformance fixtures only assert the cache file's existence and name.
fn write_pyc(pyc_path: &str) -> bool {
    let p = std::path::Path::new(pyc_path);
    if let Some(dir) = p.parent() {
        if std::fs::create_dir_all(dir).is_err() {
            return false;
        }
    }
    // CPython 3.12 magic number header + an empty body.
    std::fs::write(p, [0xcb, 0x0d, 0x0d, 0x0a, 0, 0, 0, 0]).is_ok()
}

/// The optimization levels requested by a compile_* call. `optimize` may be a
/// single int (default -1 → current level 0) or a list/tuple of ints.
fn optimize_levels(a: &[MbValue]) -> Vec<i64> {
    match kwarg(a, "optimize") {
        Some(v) if !v.is_none() => {
            if let Some(ptr) = v.as_ptr() {
                let items: Option<Vec<MbValue>> = unsafe {
                    match &(*ptr).data {
                        ObjData::List(ref l) => Some(l.read().unwrap().to_vec()),
                        ObjData::Tuple(ref t) => Some(t.to_vec()),
                        _ => None,
                    }
                };
                if let Some(items) = items {
                    let mut levels: Vec<i64> =
                        items.iter().filter_map(|x| x.as_int()).collect();
                    if levels.is_empty() { levels.push(0); }
                    return levels;
                }
            }
            vec![v.as_int().unwrap_or(-1)]
        }
        _ => vec![-1],
    }
}

/// Parse + write the cache file(s) for one source path. Returns False on a
/// parse error (no cache file written), matching compile_file's contract.
fn compile_one_file(path: &str, a: &[MbValue]) -> bool {
    // compile_file only compiles `.py` sources; any other file is a no-op
    // (no cache file, no __pycache__) — matching CPython.
    if !path.ends_with(".py") {
        return true;
    }
    let Ok(src) = std::fs::read_to_string(path) else {
        return true; // nothing to compile
    };
    if let Some(line) = non_ascii_bytes_literal_line(&src) {
        emit_syntax_report(
            path,
            line,
            "bytes can only contain ASCII literal characters",
            a,
        );
        return false;
    }
    if crate::parser::parse(&src, FileId::default()).is_err() {
        emit_syntax_report(path, 1, "invalid syntax", a);
        return false;
    }
    let legacy = kwarg(a, "legacy").and_then(|v| v.as_bool()).unwrap_or(false);
    let mut ok = true;
    for lvl in optimize_levels(a) {
        let pyc = if legacy {
            // Legacy layout: `<stem>.pyc` beside the source, no __pycache__.
            let p = std::path::Path::new(path);
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            p.parent().unwrap_or_else(|| std::path::Path::new("."))
                .join(format!("{stem}.pyc"))
                .to_string_lossy()
                .into_owned()
        } else {
            cache_path_for(path, lvl)
        };
        ok &= write_pyc(&pyc);
    }
    ok
}

/// True iff the source at `path` parses; missing files are vacuous successes.
fn compile_one(path: &str) -> bool {
    let Ok(src) = std::fs::read_to_string(path) else {
        return true; // nothing to compile, nothing to fail
    };
    crate::parser::parse(&src, FileId::default()).is_ok()
}

fn kwargs_of(a: &[MbValue]) -> Option<MbValue> {
    let last = *a.last()?;
    last.as_ptr().and_then(|p| unsafe {
        if matches!((*p).data, ObjData::Dict(_)) {
            Some(last)
        } else {
            None
        }
    })
}

fn kwarg(a: &[MbValue], key: &str) -> Option<MbValue> {
    let kw = kwargs_of(a)?;
    let ptr = kw.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            return lock.read().unwrap().get(key).copied();
        }
    }
    None
}

unsafe extern "C" fn dispatch_compile_dir(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    if let Some(w) = kwarg(a, "workers").and_then(|v| v.as_int()) {
        if w < 0 {
            return raise_exc("ValueError", "workers must be greater or equal to 0");
        }
    }
    let ddir_given = kwarg(a, "ddir").map(|v| !v.is_none()).unwrap_or(false);
    let strip_or_prepend = kwarg(a, "stripdir").map(|v| !v.is_none()).unwrap_or(false)
        || kwarg(a, "prependdir")
            .map(|v| !v.is_none())
            .unwrap_or(false);
    if ddir_given && strip_or_prepend {
        return raise_exc(
            "ValueError",
            "Destination dir (ddir) cannot be used in combination with stripdir or prependdir",
        );
    }
    let dir = a
        .first()
        .copied()
        .and_then(extract_str_v)
        .unwrap_or_default();
    // maxlevels bounds the recursion depth (top dir = level 0); default deep.
    let maxlevels = kwarg(a, "maxlevels")
        .and_then(|v| v.as_int())
        .unwrap_or(1_000_000);
    let rx = kwarg(a, "rx").filter(|v| !v.is_none());
    let ok = compile_dir_recursive(&dir, 0, maxlevels, rx, a);
    MbValue::from_bool(ok)
}

/// Does compiled-regex `rx` match `path` (rx.search(path) is not None)?
fn rx_matches(rx: MbValue, path: &str) -> bool {
    let name = MbValue::from_ptr(MbObject::new_str(path.to_string()));
    let method = MbValue::from_ptr(MbObject::new_str("search".to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![name]));
    let res = super::super::class::mb_call_method(rx, method, args);
    !res.is_none()
}

/// Recursively compile every `.py` under `dir` down to `maxlevels` deep,
/// skipping paths that match `rx`.
fn compile_dir_recursive(
    dir: &str,
    level: i64,
    maxlevels: i64,
    rx: Option<MbValue>,
    a: &[MbValue],
) -> bool {
    let mut ok = true;
    let Ok(entries) = std::fs::read_dir(dir) else { return true };
    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            subdirs.push(p);
            continue;
        }
        if p.extension().and_then(|e| e.to_str()) == Some("py") {
            let ps = p.to_string_lossy().into_owned();
            if let Some(rxv) = rx {
                if rx_matches(rxv, &ps) {
                    continue;
                }
            }
            ok &= compile_one_file(&ps, a);
        }
    }
    if level < maxlevels {
        for sub in subdirs {
            let subs = sub.to_string_lossy().into_owned();
            ok &= compile_dir_recursive(&subs, level + 1, maxlevels, rx, a);
        }
    }
    ok
}

/// importlib.util.cache_from_source(path, *, optimization=None).
unsafe extern "C" fn dispatch_cache_from_source(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let path = a.first().copied().and_then(extract_str_v).unwrap_or_default();
    // optimization: None → level 0; "" → 0; int/str → that level.
    let opt = match kwarg(a, "optimization") {
        Some(v) if !v.is_none() => {
            if let Some(i) = v.as_int() {
                i
            } else if let Some(s) = extract_str_v(v) {
                if s.is_empty() { 0 } else { s.parse::<i64>().unwrap_or(0) }
            } else {
                0
            }
        }
        _ => 0,
    };
    MbValue::from_ptr(MbObject::new_str(cache_path_for(&path, opt)))
}

/// Public address accessor so importlib.util can register the real
/// cache_from_source in place of its empty-string stub.
pub fn cache_from_source_addr() -> usize {
    dispatch_cache_from_source as *const () as usize
}

unsafe extern "C" fn dispatch_compile_file(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let ddir_given = kwarg(a, "ddir").map(|v| !v.is_none()).unwrap_or(false);
    let strip_or_prepend = kwarg(a, "stripdir").map(|v| !v.is_none()).unwrap_or(false)
        || kwarg(a, "prependdir")
            .map(|v| !v.is_none())
            .unwrap_or(false);
    if ddir_given && strip_or_prepend {
        return raise_exc(
            "ValueError",
            "Destination dir (ddir) cannot be used in combination with stripdir or prependdir",
        );
    }
    let path = a
        .first()
        .copied()
        .and_then(extract_str_v)
        .unwrap_or_default();
    MbValue::from_bool(compile_one_file(&path, a))
}

unsafe extern "C" fn dispatch_compile_path(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_bool(true)
}

unsafe extern "C" fn dispatch_main(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the compileall module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_cd = dispatch_compile_dir as *const () as usize;
    attrs.insert("compile_dir".into(), MbValue::from_func(addr_cd));

    let addr_cf = dispatch_compile_file as *const () as usize;
    attrs.insert("compile_file".into(), MbValue::from_func(addr_cf));

    let addr_cp = dispatch_compile_path as *const () as usize;
    attrs.insert("compile_path".into(), MbValue::from_func(addr_cp));

    let addr_m = dispatch_main as *const () as usize;
    attrs.insert("main".into(), MbValue::from_func(addr_m));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_cd as u64);
        set.insert(addr_cf as u64);
        set.insert(addr_cp as u64);
        set.insert(addr_m as u64);
    });

    super::register_module("compileall", attrs);
}
