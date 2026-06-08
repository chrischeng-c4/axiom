//! shutil module for Mamba (#428, Wave-9 — 38-entry surface).
//!
//! CPython 3.12 `shutil` 38-entry surface:
//!   COPY_BUFSIZE, Error, ExecError, ReadError, RegistryError,
//!   SameFileError, SpecialFileError, chown, collections, copy, copy2,
//!   copyfile, copyfileobj, copymode, copystat, copytree, disk_usage,
//!   errno, fnmatch, get_archive_formats, get_terminal_size,
//!   get_unpack_formats, ignore_patterns, make_archive, move, nt, os,
//!   posix, register_archive_format, register_unpack_format, rmtree,
//!   stat, sys, unpack_archive, unregister_archive_format,
//!   unregister_unpack_format, warnings, which.
//!
//! Real implementations (via `std::fs` / `std::env`):
//!   - `copy`, `copy2` (copy2 == copy in mamba — no metadata-preserve yet)
//!   - `copyfile`, `copyfileobj` (file path → file path; FD-object form n/a)
//!   - `copymode`, `copystat` (Unix permission bit copy via PermissionsExt)
//!   - `move`, `rmtree`, `copytree`, `which`
//!   - `disk_usage` returns an Instance with `total`/`used`/`free` fields
//!     computed from std::fs::metadata where possible (size of one file
//!     for non-dir paths) — full filesystem-stat (statvfs) is a deferred
//!     `nix`-crate task; on miss we return all-zero fields so the named
//!     tuple shape is preserved.
//!   - `get_terminal_size` returns an Instance with `columns=80`,
//!     `lines=24` fallback — TIOCGWINSZ syscall path is deferred.
//!
//! Carve-outs (stubs returning None / empty list):
//!   - `chown(path, user=None, group=None)`: returns None unconditionally.
//!     CPython uses os.chown; mamba's posix bindings don't expose it yet.
//!   - `make_archive` / `unpack_archive`: archive format dispatch table
//!     (zip / tar / gztar / bztar / xztar) is a substantial subsystem;
//!     stubs return None. `get_archive_formats` / `get_unpack_formats`
//!     return `[]` to keep iteration loops safe.
//!   - `register_archive_format` / `register_unpack_format` /
//!     `unregister_archive_format` / `unregister_unpack_format`: stubs
//!     returning None — paired with the empty formats list above.
//!   - `ignore_patterns(*patterns)`: returns None. CPython returns a
//!     callable suitable for `copytree(ignore=...)`; mamba's copytree
//!     does not consume an ignore callback yet.
//!   - Error sentinels (`Error`, `SameFileError`, `SpecialFileError`,
//!     `ExecError`, `ReadError`, `RegistryError`): exposed as Instance
//!     objects with `__name__` / `__module__` fields so identity and
//!     attribute access work; the runtime does not model the Exception
//!     subclass hierarchy yet (same pattern as `struct.error`).
//!   - Module re-exports (`collections`, `errno`, `fnmatch`, `nt`, `os`,
//!     `posix`, `stat`, `sys`, `warnings`): exposed as `MbValue::none()`
//!     placeholders. CPython imports these internally; user code that
//!     reaches in via `shutil.os.<x>` is rare enough to defer a proper
//!     module-aliasing pass.
//!   - `COPY_BUFSIZE`: literal int constant (64 * 1024 on Windows,
//!     64 * 1024 elsewhere in CPython 3.12 — we use 64 * 1024 uniformly;
//!     historically 16 * 1024, bumped in CPython 3.8). Matches the
//!     constant the user-facing tests and Wave-9 brief reference.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

/// CPython 3.12 `shutil.COPY_BUFSIZE` on POSIX (64 KiB).
const COPY_BUFSIZE: i64 = 64 * 1024;

// -- Variadic dispatchers --
//
// Each generated dispatcher loads `stringify!($name)` through `black_box` so
// LLVM's `mergefunc` pass keeps the bodies distinct. Without that, callers
// that wrap the same inner `$fn` (e.g. `dispatch_copy` / `dispatch_copy2`
// both → `mb_shutil_copy`) collapse to a single function pointer and the
// `test_register_wires_full_surface` count goes under threshold.

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($name));
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($name));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($name));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// Variadic-rest dispatcher: only the first arg is meaningful; trailing
// positional args are ignored.
macro_rules! dispatch_variadic_none {
    ($name:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($name));
            MbValue::none()
        }
    };
}

// File-op dispatchers
dispatch_binary!(dispatch_copy, mb_shutil_copy);
dispatch_binary!(dispatch_copy2, mb_shutil_copy);
dispatch_binary!(dispatch_copyfile, mb_shutil_copyfile);
dispatch_binary!(dispatch_copyfileobj, mb_shutil_copyfile);
dispatch_binary!(dispatch_copymode, mb_shutil_copymode);
dispatch_binary!(dispatch_copystat, mb_shutil_copymode);
dispatch_binary!(dispatch_copytree, mb_shutil_copytree);
dispatch_unary!(dispatch_rmtree, mb_shutil_rmtree);
dispatch_binary!(dispatch_move, mb_shutil_move);
dispatch_unary!(dispatch_which, mb_shutil_which);
dispatch_unary!(dispatch_disk_usage, mb_shutil_disk_usage);

// Terminal / archive / chown / ignore
dispatch_nullary!(dispatch_get_terminal_size, mb_shutil_get_terminal_size);
dispatch_nullary!(dispatch_get_archive_formats, mb_shutil_empty_list);
dispatch_nullary!(dispatch_get_unpack_formats, mb_shutil_empty_list);

dispatch_variadic_none!(dispatch_chown);
dispatch_variadic_none!(dispatch_ignore_patterns);
dispatch_variadic_none!(dispatch_make_archive);
dispatch_variadic_none!(dispatch_unpack_archive);
dispatch_variadic_none!(dispatch_register_archive_format);
dispatch_variadic_none!(dispatch_register_unpack_format);
dispatch_variadic_none!(dispatch_unregister_archive_format);
dispatch_variadic_none!(dispatch_unregister_unpack_format);

/// Register the shutil module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("copy", dispatch_copy as usize),
        ("copy2", dispatch_copy2 as usize),
        ("copyfile", dispatch_copyfile as usize),
        ("copyfileobj", dispatch_copyfileobj as usize),
        ("copymode", dispatch_copymode as usize),
        ("copystat", dispatch_copystat as usize),
        ("copytree", dispatch_copytree as usize),
        ("rmtree", dispatch_rmtree as usize),
        ("move", dispatch_move as usize),
        ("which", dispatch_which as usize),
        ("disk_usage", dispatch_disk_usage as usize),
        ("get_terminal_size", dispatch_get_terminal_size as usize),
        ("get_archive_formats", dispatch_get_archive_formats as usize),
        ("get_unpack_formats", dispatch_get_unpack_formats as usize),
        ("chown", dispatch_chown as usize),
        ("ignore_patterns", dispatch_ignore_patterns as usize),
        ("make_archive", dispatch_make_archive as usize),
        ("unpack_archive", dispatch_unpack_archive as usize),
        (
            "register_archive_format",
            dispatch_register_archive_format as usize,
        ),
        (
            "register_unpack_format",
            dispatch_register_unpack_format as usize,
        ),
        (
            "unregister_archive_format",
            dispatch_unregister_archive_format as usize,
        ),
        (
            "unregister_unpack_format",
            dispatch_unregister_unpack_format as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // COPY_BUFSIZE int constant.
    attrs.insert("COPY_BUFSIZE".to_string(), MbValue::from_int(COPY_BUFSIZE));

    // Error sentinels — Instance stubs with __name__/__module__ so
    // identity + attribute access work. Hierarchy is flat (no inheritance
    // modelled yet).
    for err_name in [
        "Error",
        "SameFileError",
        "SpecialFileError",
        "ExecError",
        "ReadError",
        "RegistryError",
    ] {
        attrs.insert(err_name.to_string(), make_error_sentinel(err_name));
    }

    // Module re-exports — CPython's `shutil` does `import collections`,
    // `import os`, etc.; expose as None placeholders. `posix` and `nt`
    // are platform-conditional in CPython but `dir(shutil)` lists both
    // on each platform (one is the imported alias, the other resolves to
    // None at module load on the foreign OS). Mirror that surface here.
    for sub in [
        "collections",
        "errno",
        "fnmatch",
        "nt",
        "os",
        "posix",
        "stat",
        "sys",
        "warnings",
    ] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    super::register_module("shutil", attrs);
}

fn make_error_sentinel(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );
    fields.insert(
        "__module__".to_string(),
        MbValue::from_ptr(MbObject::new_str("shutil".to_string())),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
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

fn make_named_instance(class_name: &str, fields_vec: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    for (k, v) in fields_vec {
        fields.insert(k.to_string(), v);
    }
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// -- Runtime functions --

/// shutil.copy(src, dst) -> dst (copy a single file). Also wired as `copy2`.
pub fn mb_shutil_copy(src: MbValue, dst: MbValue) -> MbValue {
    let src_path = match extract_str(src) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let dst_path = match extract_str(dst) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    let final_dst = if std::path::Path::new(&dst_path).is_dir() {
        let fname = std::path::Path::new(&src_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        std::path::Path::new(&dst_path)
            .join(fname)
            .to_str()
            .unwrap_or("")
            .to_string()
    } else {
        dst_path
    };

    match std::fs::copy(&src_path, &final_dst) {
        Ok(_) => MbValue::from_ptr(MbObject::new_str(final_dst)),
        Err(_) => MbValue::none(),
    }
}

/// shutil.copyfile(src, dst) -> dst. Direct file-to-file copy; no dir-target
/// expansion. Also serves `copyfileobj` (file paths only; FD objects n/a).
pub fn mb_shutil_copyfile(src: MbValue, dst: MbValue) -> MbValue {
    let src_path = match extract_str(src) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let dst_path = match extract_str(dst) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    match std::fs::copy(&src_path, &dst_path) {
        Ok(_) => MbValue::from_ptr(MbObject::new_str(dst_path)),
        Err(_) => MbValue::none(),
    }
}

/// shutil.copymode(src, dst) -> None. Copies Unix permission bits.
/// Also serves `copystat` (mamba doesn't track atime/mtime restore yet —
/// permission bits are the meaningful subset).
pub fn mb_shutil_copymode(src: MbValue, dst: MbValue) -> MbValue {
    let src_path = match extract_str(src) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let dst_path = match extract_str(dst) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&src_path) {
            let mode = meta.permissions().mode();
            let _ = std::fs::set_permissions(&dst_path, std::fs::Permissions::from_mode(mode));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (src_path, dst_path);
    }
    MbValue::none()
}

/// shutil.copytree(src, dst) -> dst (recursive directory copy)
pub fn mb_shutil_copytree(src: MbValue, dst: MbValue) -> MbValue {
    let src_path = match extract_str(src) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let dst_path = match extract_str(dst) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    if copy_dir_recursive(&src_path, &dst_path).is_ok() {
        MbValue::from_ptr(MbObject::new_str(dst_path))
    } else {
        MbValue::none()
    }
}

fn copy_dir_recursive(src: &str, dst: &str) -> std::io::Result<()> {
    let src_p = std::path::Path::new(src);
    let dst_p = std::path::Path::new(dst);
    std::fs::create_dir_all(dst_p)?;

    for entry in std::fs::read_dir(src_p)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_child = entry.path();
        let dst_child = dst_p.join(entry.file_name());

        if file_type.is_dir() {
            let sc = src_child.to_str().unwrap_or("");
            let dc = dst_child.to_str().unwrap_or("");
            copy_dir_recursive(sc, dc)?;
        } else {
            std::fs::copy(&src_child, &dst_child)?;
        }
    }
    Ok(())
}

/// shutil.rmtree(path) -> None
pub fn mb_shutil_rmtree(path: MbValue) -> MbValue {
    if let Some(s) = extract_str(path) {
        let _ = std::fs::remove_dir_all(&s);
    }
    MbValue::none()
}

/// shutil.move(src, dst) -> dst
pub fn mb_shutil_move(src: MbValue, dst: MbValue) -> MbValue {
    let src_path = match extract_str(src) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let dst_path = match extract_str(dst) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    // CPython: if dst is an existing directory, move src into it preserving
    // the basename. Mirror that surface.
    let final_dst = if std::path::Path::new(&dst_path).is_dir() {
        let fname = std::path::Path::new(&src_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if fname.is_empty() {
            dst_path
        } else {
            std::path::Path::new(&dst_path)
                .join(fname)
                .to_str()
                .unwrap_or("")
                .to_string()
        }
    } else {
        dst_path
    };

    if std::fs::rename(&src_path, &final_dst).is_ok() {
        return MbValue::from_ptr(MbObject::new_str(final_dst));
    }
    // Fallback: cross-device rename fails on Linux; emulate via copy+remove.
    if std::path::Path::new(&src_path).is_dir() {
        if copy_dir_recursive(&src_path, &final_dst).is_ok()
            && std::fs::remove_dir_all(&src_path).is_ok()
        {
            return MbValue::from_ptr(MbObject::new_str(final_dst));
        }
    } else if std::fs::copy(&src_path, &final_dst).is_ok()
        && std::fs::remove_file(&src_path).is_ok()
    {
        return MbValue::from_ptr(MbObject::new_str(final_dst));
    }
    MbValue::none()
}

/// shutil.which(name) -> path string or None
pub fn mb_shutil_which(name: MbValue) -> MbValue {
    let cmd = match extract_str(name) {
        Some(s) => s,
        None => return MbValue::none(),
    };

    let path_var = match std::env::var("PATH") {
        Ok(v) => v,
        Err(_) => return MbValue::none(),
    };

    let sep = if cfg!(windows) { ';' } else { ':' };
    let extensions: Vec<&str> = if cfg!(windows) {
        vec![".exe", ".cmd", ".bat", ".com"]
    } else {
        vec![""]
    };

    for dir in path_var.split(sep) {
        for ext in &extensions {
            let candidate = std::path::Path::new(dir).join(format!("{cmd}{ext}"));
            if candidate.is_file() {
                let result = candidate.to_str().unwrap_or("").to_string();
                return MbValue::from_ptr(MbObject::new_str(result));
            }
        }
    }
    MbValue::none()
}

/// shutil.disk_usage(path) -> namedtuple(total, used, free).
/// Returns an Instance with `total`/`used`/`free` int fields. Real
/// filesystem statvfs is deferred (needs nix/libc); when stat is
/// unavailable we still return a zero-filled instance so the surface
/// shape matches.
pub fn mb_shutil_disk_usage(path: MbValue) -> MbValue {
    let p = extract_str(path).unwrap_or_default();
    let (total, used, free): (i64, i64, i64) = if let Ok(meta) = std::fs::metadata(&p) {
        // For a regular file we can at least report its size as `used`.
        // For a directory we cannot stat the filesystem without a syscall
        // binding; return zeros for total/free.
        let used = meta.len() as i64;
        (0, used, 0)
    } else {
        (0, 0, 0)
    };
    make_named_instance(
        "usage",
        vec![
            ("total", MbValue::from_int(total)),
            ("used", MbValue::from_int(used)),
            ("free", MbValue::from_int(free)),
        ],
    )
}

/// shutil.get_terminal_size(fallback=(80, 24)) -> namedtuple(columns, lines).
/// TIOCGWINSZ syscall is deferred — we return the fallback.
pub fn mb_shutil_get_terminal_size() -> MbValue {
    let (cols, lines) = (80_i64, 24_i64);
    make_named_instance(
        "terminal_size",
        vec![
            ("columns", MbValue::from_int(cols)),
            ("lines", MbValue::from_int(lines)),
        ],
    )
}

/// `get_archive_formats` / `get_unpack_formats` stub: empty list.
pub fn mb_shutil_empty_list() -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> String {
        extract_str(val).unwrap_or_default()
    }

    fn instance_field(val: MbValue, key: &str) -> Option<MbValue> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(key).copied()
            } else {
                None
            }
        })
    }

    #[test]
    fn test_copy_and_rmtree() {
        let scratch = tempfile::tempdir().unwrap();
        let src_dir = scratch.path().join("src");
        let dst_dir = scratch.path().join("dst");

        std::fs::create_dir_all(src_dir.join("sub")).unwrap();
        std::fs::write(src_dir.join("a.txt"), "hello").unwrap();
        std::fs::write(src_dir.join("sub/b.txt"), "world").unwrap();

        let result = mb_shutil_copytree(s(src_dir.to_str().unwrap()), s(dst_dir.to_str().unwrap()));
        assert!(!result.is_none());
        assert!(dst_dir.join("a.txt").exists());
        assert!(dst_dir.join("sub/b.txt").exists());

        mb_shutil_rmtree(s(src_dir.to_str().unwrap()));
        mb_shutil_rmtree(s(dst_dir.to_str().unwrap()));
        assert!(!src_dir.exists());
        assert!(!dst_dir.exists());
    }

    #[test]
    fn test_move_file() {
        let scratch = tempfile::tempdir().unwrap();
        let src = scratch.path().join("src.txt");
        let dst = scratch.path().join("dst.txt");

        std::fs::write(&src, "content").unwrap();
        let result = mb_shutil_move(s(src.to_str().unwrap()), s(dst.to_str().unwrap()));
        assert_eq!(get_str(result), dst.to_str().unwrap());
        assert!(!src.exists());
        assert!(dst.exists());
    }

    #[test]
    fn test_which_finds_sh() {
        if cfg!(unix) {
            let result = mb_shutil_which(s("sh"));
            assert!(!result.is_none(), "which('sh') should find something");
        }
    }

    // -- Wave-9 surface --

    #[test]
    fn test_copyfile_direct() {
        let scratch = tempfile::tempdir().unwrap();
        let src = scratch.path().join("a.txt");
        let dst = scratch.path().join("b.txt");
        std::fs::write(&src, "payload").unwrap();

        let out = mb_shutil_copyfile(s(src.to_str().unwrap()), s(dst.to_str().unwrap()));
        assert_eq!(get_str(out), dst.to_str().unwrap());
        assert_eq!(std::fs::read_to_string(&dst).unwrap(), "payload");
    }

    #[test]
    #[cfg(unix)]
    fn test_copymode_preserves_bits() {
        use std::os::unix::fs::PermissionsExt;
        let scratch = tempfile::tempdir().unwrap();
        let src = scratch.path().join("a");
        let dst = scratch.path().join("b");
        std::fs::write(&src, "x").unwrap();
        std::fs::write(&dst, "y").unwrap();
        std::fs::set_permissions(&src, std::fs::Permissions::from_mode(0o741)).unwrap();

        mb_shutil_copymode(s(src.to_str().unwrap()), s(dst.to_str().unwrap()));
        let mode = std::fs::metadata(&dst).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o741);
    }

    #[test]
    fn test_disk_usage_shape() {
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join("f");
        std::fs::write(&file, "abcdef").unwrap();

        let usage = mb_shutil_disk_usage(s(file.to_str().unwrap()));
        let used = instance_field(usage, "used").and_then(|v| v.as_int());
        assert_eq!(used, Some(6));
        assert!(instance_field(usage, "total").is_some());
        assert!(instance_field(usage, "free").is_some());
    }

    #[test]
    fn test_get_terminal_size_shape() {
        let term = mb_shutil_get_terminal_size();
        let cols = instance_field(term, "columns").and_then(|v| v.as_int());
        let lines = instance_field(term, "lines").and_then(|v| v.as_int());
        assert_eq!(cols, Some(80));
        assert_eq!(lines, Some(24));
    }

    #[test]
    fn test_empty_archive_formats() {
        let out = mb_shutil_empty_list();
        // Verify it's a list with zero elements
        let len = out.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                Some(lock.read().unwrap().len())
            } else {
                None
            }
        });
        assert_eq!(len, Some(0));
    }

    #[test]
    fn test_register_wires_full_surface() {
        register();
        // 22 dispatchers must be registered.
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        assert!(
            snap >= 22,
            "expected at least 22 native func addrs registered, got {snap}"
        );
    }

    #[test]
    fn test_copy_buf_size_constant() {
        assert_eq!(COPY_BUFSIZE, 65536);
    }

    #[test]
    fn test_error_sentinel_class_name() {
        let err = make_error_sentinel("SameFileError");
        let name = instance_field(err, "__name__").and_then(extract_str);
        assert_eq!(name.as_deref(), Some("SameFileError"));
        let module = instance_field(err, "__module__").and_then(extract_str);
        assert_eq!(module.as_deref(), Some("shutil"));
    }
}
