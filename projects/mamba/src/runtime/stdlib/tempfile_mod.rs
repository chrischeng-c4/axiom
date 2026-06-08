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

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_nullary!(dispatch_gettempdir, mb_tempfile_gettempdir);
dispatch_nullary!(dispatch_gettempprefix, mb_tempfile_gettempprefix);
dispatch_nullary!(dispatch_gettempdirb, mb_tempfile_gettempdirb);
dispatch_nullary!(dispatch_gettempprefixb, mb_tempfile_gettempprefixb);
dispatch_nullary!(dispatch_mkdtemp, mb_tempfile_mkdtemp);
dispatch_nullary!(dispatch_mkstemp, mb_tempfile_mkstemp);
dispatch_nullary!(dispatch_mktemp, mb_tempfile_mktemp);
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_NamedTemporaryFile(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_tempfile_named_temp_file()
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_TemporaryDirectory(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    mb_tempfile_temporary_directory()
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_TemporaryFile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_tempfile_temporary_file()
}
#[allow(non_snake_case)]
unsafe extern "C" fn dispatch_SpooledTemporaryFile(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
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
        ("NamedTemporaryFile", dispatch_NamedTemporaryFile as usize),
        ("TemporaryDirectory", dispatch_TemporaryDirectory as usize),
        ("TemporaryFile", dispatch_TemporaryFile as usize),
        (
            "SpooledTemporaryFile",
            dispatch_SpooledTemporaryFile as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
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
    attrs.insert(
        "tempdir".to_string(),
        MbValue::from_ptr(MbObject::new_str(tmp_path)),
    );
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
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Runtime functions ──

/// tempfile.gettempdir() -> str (system temp directory)
pub fn mb_tempfile_gettempdir() -> MbValue {
    let tmp = std::env::temp_dir();
    let path = tmp.to_str().unwrap_or("/tmp").to_string();
    MbValue::from_ptr(MbObject::new_str(path))
}

/// tempfile.gettempprefix() -> str (default prefix for temp names)
pub fn mb_tempfile_gettempprefix() -> MbValue {
    // CPython default is "tmp" — match for compatibility.
    MbValue::from_ptr(MbObject::new_str("tmp".to_string()))
}

/// tempfile.mkdtemp() -> str (create temp directory, return path)
pub fn mb_tempfile_mkdtemp() -> MbValue {
    let tmp = std::env::temp_dir();
    let dir_name = temp_name("mbtmp_d_");
    let dir_path = tmp.join(&dir_name);

    match std::fs::create_dir_all(&dir_path) {
        Ok(_) => {
            let path = dir_path.to_str().unwrap_or("").to_string();
            MbValue::from_ptr(MbObject::new_str(path))
        }
        Err(_) => MbValue::none(),
    }
}

/// tempfile.mkstemp() -> tuple (fd_placeholder, path_string)
pub fn mb_tempfile_mkstemp() -> MbValue {
    let tmp = std::env::temp_dir();
    let file_name = temp_name("mbtmp_f_");
    let file_path = tmp.join(&file_name);

    match std::fs::write(&file_path, "") {
        Ok(_) => {
            let path = file_path.to_str().unwrap_or("").to_string();
            let tuple = MbObject::new_tuple(vec![
                MbValue::from_int(0),
                MbValue::from_ptr(MbObject::new_str(path)),
            ]);
            MbValue::from_ptr(tuple)
        }
        Err(_) => MbValue::none(),
    }
}

/// tempfile.NamedTemporaryFile() -> writable file handle with `.name`.
pub fn mb_tempfile_named_temp_file() -> MbValue {
    let tmp = std::env::temp_dir();
    let file_name = temp_name("mbtmp_named_");
    let file_path = tmp.join(&file_name);
    let path = file_path.to_str().unwrap_or("").to_string();
    super::super::file_io::mb_open(
        MbValue::from_ptr(MbObject::new_str(path)),
        MbValue::from_ptr(MbObject::new_str("w".to_string())),
    )
}

/// tempfile.TemporaryDirectory() -> str (path to created temp dir).
///
/// Carve: CPython returns an object with `__enter__`/`__exit__` +
/// `.name` attribute. Returning the path string directly lets bench
/// code use it immediately as a string; full context-manager support
/// is gated on broader class.rs work.
pub fn mb_tempfile_temporary_directory() -> MbValue {
    mb_tempfile_mkdtemp()
}

/// tempfile.gettempdirb() -> bytes-like (system temp directory).
///
/// CPython returns `bytes`. Mamba's bytes vocabulary is partial; we
/// return the path as a `str` so importers that only `hasattr`-probe
/// pass. Sentinel — full bytes parity is a separate issue.
pub fn mb_tempfile_gettempdirb() -> MbValue {
    mb_tempfile_gettempdir()
}

/// tempfile.gettempprefixb() -> bytes-like (default prefix).
///
/// Sentinel — see `gettempdirb` rationale.
pub fn mb_tempfile_gettempprefixb() -> MbValue {
    mb_tempfile_gettempprefix()
}

/// tempfile.mktemp() -> str (deprecated CPython API; returns a name
/// without creating the file).
///
/// Carve: CPython emits a DeprecationWarning; we skip the warning.
pub fn mb_tempfile_mktemp() -> MbValue {
    let tmp = std::env::temp_dir();
    let file_name = temp_name("mbtmp_m_");
    let file_path = tmp.join(&file_name);
    let path = file_path.to_str().unwrap_or("").to_string();
    MbValue::from_ptr(MbObject::new_str(path))
}

/// tempfile.TemporaryFile() — sentinel binding; returns a `(0, path)`
/// tuple like `mkstemp` for now. Full file-object semantics gated on
/// file_io integration (out of scope for #1462).
pub fn mb_tempfile_temporary_file() -> MbValue {
    mb_tempfile_mkstemp()
}

/// tempfile.SpooledTemporaryFile() — sentinel binding; returns a
/// `(0, path)` tuple for now. Spooling-to-disk semantics gated on
/// file_io integration (out of scope for #1462).
pub fn mb_tempfile_spooled_temporary_file() -> MbValue {
    mb_tempfile_mkstemp()
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
        let result = mb_tempfile_temporary_directory();
        let path = extract_str(result).unwrap();
        assert!(
            std::path::Path::new(&path).is_dir(),
            "TemporaryDirectory should create a directory"
        );
        let _ = std::fs::remove_dir(&path);
    }
}
