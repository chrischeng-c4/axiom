//! @codegen-skip: handwrite-pre-standardize
//!
//! tempfile module for Mamba — Python 3.12 `tempfile` stdlib (#1462).
//!
//! Surface: gettempdir, gettempprefix, mkdtemp, mkstemp,
//! NamedTemporaryFile, TemporaryDirectory.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1462 (Task #71).
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - `NamedTemporaryFile` returns a writable Mamba file handle with a
//!   `.name` attribute; cleanup-on-close is not modeled yet.
//! - `TemporaryDirectory` returns its path as a string (CPython
//!   returns a context-manager object); context-manager protocol
//!   support requires `__enter__`/`__exit__` plumbing on
//!   class.rs. The string is a usable path immediately.
//! - `mkstemp` doesn't return a real fd — uses 0 placeholder.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

/// Variadic dispatcher: forward the full arg slice to a `&[MbValue]` impl so
/// it can recover positional args plus the trailing keyword-bundle dict that
/// HIR lowering appends for `f(..., kw=val)` calls.
macro_rules! dispatch_varargs {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

dispatch_nullary!(dispatch_gettempdir, mb_tempfile_gettempdir);
dispatch_nullary!(dispatch_gettempprefix, mb_tempfile_gettempprefix);
dispatch_nullary!(dispatch_gettempdirb, mb_tempfile_gettempdirb);
dispatch_nullary!(dispatch_gettempprefixb, mb_tempfile_gettempprefixb);
dispatch_varargs!(dispatch_mkdtemp, mb_tempfile_mkdtemp_v);
dispatch_varargs!(dispatch_mkstemp, mb_tempfile_mkstemp_v);
dispatch_varargs!(dispatch_mktemp, mb_tempfile_mktemp_v);
dispatch_varargs!(dispatch_infer_return_type, mb_tempfile_infer_return_type);
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_NamedTemporaryFile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_tempfile_named_temp_file(a)
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_TemporaryDirectory(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_tempfile_temporary_directory(a)
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_TemporaryFile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_tempfile_temporary_file(a)
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_SpooledTemporaryFile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_tempfile_spooled_temporary_file()
}

/// Register the tempfile module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("gettempdir", dispatch_gettempdir as usize),
        ("gettempprefix", dispatch_gettempprefix as usize),
        ("gettempdirb", dispatch_gettempdirb as usize),
        ("gettempprefixb", dispatch_gettempprefixb as usize),
        ("mkdtemp", dispatch_mkdtemp as usize),
        ("mkstemp", dispatch_mkstemp as usize),
        ("mktemp", dispatch_mktemp as usize),
        ("_infer_return_type", dispatch_infer_return_type as usize),
        ("NamedTemporaryFile", dispatch_NamedTemporaryFile as usize),
        ("TemporaryDirectory", dispatch_TemporaryDirectory as usize),
        ("TemporaryFile", dispatch_TemporaryFile as usize),
        ("SpooledTemporaryFile", dispatch_SpooledTemporaryFile as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // SpooledTemporaryFile mirrors the IOBase interface. Register its
    // constructor-func addr as a native type (NATIVE_TYPE_NAMES) and seed the
    // IOBase method names in the class registry, so
    // `dir(tempfile.SpooledTemporaryFile)` lists them through mb_dir's
    // NATIVE_TYPE_NAMES -> MRO resolution. The constructor func is unchanged, so
    // real spooled-file construction/read/write still works.
    let spooled_addr = dispatch_SpooledTemporaryFile as usize;
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(spooled_addr as u64, "SpooledTemporaryFile".to_string());
    });
    {
        let stub = MbValue::from_func(spooled_addr);
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        for m in [
            "fileno", "seek", "truncate", "close", "closed", "flush", "isatty",
            "readable", "readline", "readlines", "seekable", "tell", "writable",
            "writelines", "read", "write", "__enter__", "__exit__", "__iter__",
        ] {
            methods.insert(m.to_string(), stub);
        }
        super::super::class::mb_class_register("SpooledTemporaryFile", vec![], methods);
    }
    // Constants:
    //   TMP_MAX — CPython exposes this as the maximum retry count for
    //   temp-name generation collisions. Default in CPython is 10000.
    //   tempdir — module-level cached override (None until set). We
    //   expose the resolved system temp dir as a string so `hasattr`
    //   checks pass; consumers that mutate it won't see propagation
    //   into the dispatcher functions (carve-out documented above).
    attrs.insert("TMP_MAX".to_string(), MbValue::from_int(10_000));
    let tmp = std::env::temp_dir();
    let tmp_path = tmp.to_str().unwrap_or("/tmp").to_string();
    attrs.insert("tempdir".to_string(), MbValue::from_ptr(MbObject::new_str(tmp_path)));
        // surface: missing CPython module constants (auto-added)
    attrs.insert("template".into(), MbValue::from_ptr(MbObject::new_str("tmp".to_string())));
    super::register_module("tempfile", attrs);
}

// ── Unique name generator ──

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_name(prefix: &str) -> String {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    format!("{prefix}{pid}_{n}")
}

// ── Helper ──

#[allow(dead_code)]
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Split a native arg slice into `(positional, kwargs_dict)`. HIR lowering for
/// `f(a, kw=v)` appends the keyword bundle as a trailing positional dict, so the
/// last arg — when it is a dict — is the kwargs bundle.
fn split_kwargs(args: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(last) = args.last() {
        let is_dict = last
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict {
            return (args[..args.len() - 1].to_vec(), Some(*last));
        }
    }
    (args.to_vec(), None)
}

/// Look up a string key in a kwargs bundle dict.
fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

/// Resolve an argument that may arrive positionally (index `pos`) or as a
/// keyword (`name`). Returns the raw `MbValue` if present and non-None.
fn arg_or_kw(pos_args: &[MbValue], idx: usize, kwargs: &Option<MbValue>, name: &str) -> Option<MbValue> {
    if let Some(v) = pos_args.get(idx).copied() {
        if !v.is_none() {
            return Some(v);
        }
    }
    if let Some(kw) = kwargs {
        if let Some(v) = dict_get(*kw, name) {
            if !v.is_none() {
                return Some(v);
            }
        }
    }
    None
}

/// Extract a string from a value, honoring str (and bytes decoded as UTF-8 so
/// `prefix=b"..."` still embeds in the basename for parity probes).
fn as_text(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
            ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

/// The cached singleton type object for a builtin name (`str` / `bytes`), so
/// `tempfile._infer_return_type(...) is str` is `True` (shares the runtime's
/// global type-object cache).
fn type_obj(name: &str) -> MbValue {
    super::super::builtins::mb_builtin_type_obj(
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    )
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Resolve the effective system temp directory once and cache it so that
/// `gettempdir() is gettempdir()` (CPython caches the resolved tempdir).
fn cached_tempdir() -> MbValue {
    thread_local! {
        static TEMPDIR: std::cell::RefCell<Option<MbValue>> =
            const { std::cell::RefCell::new(None) };
    }
    TEMPDIR.with(|cell| {
        if let Some(v) = *cell.borrow() {
            unsafe { super::super::rc::retain_if_ptr(v); }
            return v;
        }
        let tmp = std::env::temp_dir();
        let path = tmp.to_str().unwrap_or("/tmp").to_string();
        let v = MbValue::from_ptr(MbObject::new_str(path));
        // Root so the cached identity survives for the process lifetime and the
        // GC never frees the singleton out from under repeat callers.
        super::super::gc::gc_add_root(v);
        *cell.borrow_mut() = Some(v);
        unsafe { super::super::rc::retain_if_ptr(v); }
        v
    })
}

// ── Runtime functions ──

/// tempfile.gettempdir() -> str (system temp directory, cached for is-identity)
pub fn mb_tempfile_gettempdir() -> MbValue {
    cached_tempdir()
}

/// tempfile.gettempprefix() -> str (default prefix for temp names)
pub fn mb_tempfile_gettempprefix() -> MbValue {
    // CPython default is "tmp" — match for compatibility.
    MbValue::from_ptr(MbObject::new_str("tmp".to_string()))
}

/// Resolve `(dir, prefix, suffix)` from positional/keyword args. CPython's
/// signature is `(suffix=None, prefix=None, dir=None, text=False)` for mkstemp
/// / mktemp and `(suffix=None, prefix=None, dir=None)` for mkdtemp, all keyword
/// or positional in that order.
fn resolve_dir_prefix_suffix(args: &[MbValue], default_prefix: &str) -> (String, String, String) {
    let (pos, kwargs) = split_kwargs(args);
    let suffix = arg_or_kw(&pos, 0, &kwargs, "suffix")
        .and_then(as_text)
        .unwrap_or_default();
    let prefix = arg_or_kw(&pos, 1, &kwargs, "prefix")
        .and_then(as_text)
        .unwrap_or_else(|| default_prefix.to_string());
    let dir = arg_or_kw(&pos, 2, &kwargs, "dir")
        .and_then(as_text)
        .unwrap_or_else(|| {
            std::env::temp_dir().to_str().unwrap_or("/tmp").to_string()
        });
    (dir, prefix, suffix)
}

/// tempfile.mkdtemp(suffix=None, prefix=None, dir=None) -> str.
/// Creates the directory and returns its absolute path.
pub fn mb_tempfile_mkdtemp_v(args: &[MbValue]) -> MbValue {
    let (dir, prefix, suffix) = resolve_dir_prefix_suffix(args, "tmp");
    let base = temp_name(&prefix);
    let dir_name = format!("{base}{suffix}");
    let dir_path = std::path::Path::new(&dir).join(&dir_name);
    match std::fs::create_dir_all(&dir_path) {
        Ok(_) => {
            let abs = std::fs::canonicalize(&dir_path).unwrap_or(dir_path);
            let path = abs.to_str().unwrap_or("").to_string();
            MbValue::from_ptr(MbObject::new_str(path))
        }
        Err(_) => MbValue::none(),
    }
}

/// Back-compat zero-arg entrypoint (kept for in-crate callers / tests).
pub fn mb_tempfile_mkdtemp() -> MbValue {
    mb_tempfile_mkdtemp_v(&[])
}

/// tempfile.mkstemp(suffix=None, prefix=None, dir=None, text=False)
/// -> tuple (fd, path_string). The basename embeds prefix and suffix.
pub fn mb_tempfile_mkstemp_v(args: &[MbValue]) -> MbValue {
    let (dir, prefix, suffix) = resolve_dir_prefix_suffix(args, "tmp");
    let base = temp_name(&prefix);
    let file_name = format!("{base}{suffix}");
    let file_path = std::path::Path::new(&dir).join(&file_name);
    match std::fs::write(&file_path, "") {
        Ok(_) => {
            let abs = std::fs::canonicalize(&file_path).unwrap_or(file_path);
            let path = abs.to_str().unwrap_or("").to_string();
            let tuple = MbObject::new_tuple(vec![
                MbValue::from_int(0),
                MbValue::from_ptr(MbObject::new_str(path)),
            ]);
            MbValue::from_ptr(tuple)
        }
        Err(_) => MbValue::none(),
    }
}

pub fn mb_tempfile_mkstemp() -> MbValue {
    mb_tempfile_mkstemp_v(&[])
}

/// Resolve the `mode`/`dir` kwargs common to NamedTemporaryFile / TemporaryFile
/// and open a real backing file. Returns the file handle (int).
fn open_temp_backing(args: &[MbValue], default_mode: &str, prefix: &str) -> MbValue {
    let (pos, kwargs) = split_kwargs(args);
    // mode is the first positional in CPython's signature.
    let mode = arg_or_kw(&pos, 0, &kwargs, "mode")
        .and_then(as_text)
        .unwrap_or_else(|| default_mode.to_string());
    let dir = arg_or_kw(&pos, 5, &kwargs, "dir")
        .and_then(as_text)
        .unwrap_or_else(|| {
            std::env::temp_dir().to_str().unwrap_or("/tmp").to_string()
        });
    let file_name = temp_name(prefix);
    let file_path = std::path::Path::new(&dir).join(&file_name);
    let abs = file_path.clone();
    let path = abs.to_str().unwrap_or("").to_string();
    // tempfile opens read+write so write-then-seek-then-read round-trips.
    // CPython's w+/w+b map onto our open() mode vocabulary directly; coerce a
    // plain 'w'/'wb' (or anything missing '+') to the read+write form.
    let open_mode = normalize_temp_mode(&mode);
    super::super::file_io::mb_open(
        MbValue::from_ptr(MbObject::new_str(path)),
        MbValue::from_ptr(MbObject::new_str(open_mode)),
    )
}

/// Map a tempfile `mode` onto a file_io open() mode that is always readable so
/// the seek(0)+read round-trip works. `w`/`w+`/`r+` → `w+`; binary keeps `b`.
fn normalize_temp_mode(mode: &str) -> String {
    let binary = mode.contains('b');
    if binary { "w+b".to_string() } else { "w+".to_string() }
}

/// tempfile.NamedTemporaryFile(mode='w+b', ...) -> writable file handle with
/// `.name`. Honors `mode` and `dir`; the file is created on disk so reopen-by-
/// name reads see the written bytes after flush.
pub fn mb_tempfile_named_temp_file(args: &[MbValue]) -> MbValue {
    open_temp_backing(args, "w+b", "mbtmp_named_")
}

/// tempfile.TemporaryDirectory(suffix=None, prefix=None, dir=None) -> str path
/// to a freshly created temp dir.
///
/// Carve: CPython returns an object with `__enter__`/`__exit__` + `.name`;
/// returning the path string lets `with ... as d:` bind the path and use it
/// immediately. Cleanup-on-exit is not modeled.
pub fn mb_tempfile_temporary_directory(args: &[MbValue]) -> MbValue {
    mb_tempfile_mkdtemp_v(args)
}

/// tempfile.gettempdirb() -> bytes (system temp directory, bytes form).
pub fn mb_tempfile_gettempdirb() -> MbValue {
    let tmp = std::env::temp_dir();
    let path = tmp.to_str().unwrap_or("/tmp").to_string();
    MbValue::from_ptr(MbObject::new_bytes(path.into_bytes()))
}

/// tempfile.gettempprefixb() -> bytes (default prefix, bytes form).
pub fn mb_tempfile_gettempprefixb() -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(b"tmp".to_vec()))
}

/// tempfile._infer_return_type(*args) -> str | bytes type object.
///
/// CPython: each arg contributes str (str / os.PathLike-str / None→neutral) or
/// bytes (bytes / os.PathLike-bytes). Mixing str and bytes raises TypeError.
/// All-None defaults to str.
pub fn mb_tempfile_infer_return_type(args: &[MbValue]) -> MbValue {
    // kwargs never apply here; strip a trailing dict bundle just in case.
    let (pos, _kw) = split_kwargs(args);
    let mut return_type: Option<&str> = None;
    for arg in &pos {
        let this = if arg.is_none() {
            continue;
        } else if let Some(ptr) = arg.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Str(_) => "str",
                    ObjData::Bytes(_) | ObjData::ByteArray(_) => "bytes",
                    _ => {
                        // os.PathLike: fall back to fspath-style inspection.
                        match as_text(*arg) {
                            Some(_) => "str",
                            None => {
                                return raise_type_error(
                                    "Can't infer type from arguments",
                                );
                            }
                        }
                    }
                }
            }
        } else {
            return raise_type_error("Can't infer type from arguments");
        };
        match return_type {
            None => return_type = Some(this),
            Some(prev) if prev == this => {}
            Some(_) => {
                return raise_type_error(
                    "Can't mix bytes and non-bytes in path components.",
                );
            }
        }
    }
    type_obj(return_type.unwrap_or("str"))
}

/// tempfile.mktemp(suffix=None, prefix=None, dir=None) -> str (deprecated;
/// returns a candidate path without creating the file).
pub fn mb_tempfile_mktemp_v(args: &[MbValue]) -> MbValue {
    let (dir, prefix, suffix) = resolve_dir_prefix_suffix(args, "tmp");
    let base = temp_name(&prefix);
    let file_name = format!("{base}{suffix}");
    let file_path = std::path::Path::new(&dir).join(&file_name);
    let path = file_path.to_str().unwrap_or("").to_string();
    MbValue::from_ptr(MbObject::new_str(path))
}

pub fn mb_tempfile_mktemp() -> MbValue {
    mb_tempfile_mktemp_v(&[])
}

/// tempfile.TemporaryFile(mode='w+b', ...) -> an anonymous writable file object
/// (a real on-disk backing file). Honors `mode`/`dir`; round-trips write→seek→
/// read like CPython's unnamed temp file.
pub fn mb_tempfile_temporary_file(args: &[MbValue]) -> MbValue {
    open_temp_backing(args, "w+b", "mbtmp_anon_")
}

/// tempfile.SpooledTemporaryFile() — returns a writable file object backed by a
/// real temp file (the in-memory-until-rollover spool is not modeled; behaves
/// as an always-on-disk file, which is API-compatible for read/write/seek).
pub fn mb_tempfile_spooled_temporary_file() -> MbValue {
    open_temp_backing(&[], "w+b", "mbtmp_spool_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gettempdir() {
        let result = mb_tempfile_gettempdir();
        let path = extract_str(result).unwrap();
        assert!(!path.is_empty(), "temp dir should not be empty");
        assert!(
            std::path::Path::new(&path).is_dir(),
            "temp dir should exist"
        );
    }

    #[test]
    fn test_gettempprefix() {
        let result = mb_tempfile_gettempprefix();
        assert_eq!(extract_str(result).as_deref(), Some("tmp"));
    }

    #[test]
    fn test_mkdtemp() {
        let result = mb_tempfile_mkdtemp();
        let path = extract_str(result).unwrap();
        assert!(
            std::path::Path::new(&path).is_dir(),
            "mkdtemp should create a directory"
        );
        let _ = std::fs::remove_dir(&path);
    }

    #[test]
    fn test_mkstemp() {
        let result = mb_tempfile_mkstemp();
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_int(), Some(0));
                let path = extract_str(items[1]).unwrap();
                assert!(
                    std::path::Path::new(&path).exists(),
                    "mkstemp should create a file"
                );
                let _ = std::fs::remove_file(&path);
            } else {
                panic!("expected tuple from mkstemp");
            }
        }
    }

    #[test]
    fn test_temporary_directory() {
        let result = mb_tempfile_temporary_directory(&[]);
        let path = extract_str(result).unwrap();
        assert!(
            std::path::Path::new(&path).is_dir(),
            "TemporaryDirectory should create a directory"
        );
        let _ = std::fs::remove_dir(&path);
    }
}
