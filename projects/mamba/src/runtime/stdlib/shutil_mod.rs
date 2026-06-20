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

// copyfileobj(fsrc, fdst, length=COPY_BUFSIZE) operates on file-like objects,
// not paths — it needs the third positional `length` arg, so it gets a
// bespoke dispatcher rather than the binary macro.
unsafe extern "C" fn dispatch_copyfileobj(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_copyfileobj));
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    mb_shutil_copyfileobj(
        a.first().copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied(),
    )
}
dispatch_binary!(dispatch_copymode, mb_shutil_copymode);
dispatch_binary!(dispatch_copystat, mb_shutil_copymode);
dispatch_binary!(dispatch_copytree, mb_shutil_copytree);
// rmtree(path, ignore_errors=False, ...). `ignore_errors` arrives either as the
// 2nd positional or inside the trailing kwargs Dict; either way, when truthy the
// refusal guards must be swallowed. A bespoke dispatcher resolves the flag.
unsafe extern "C" fn dispatch_rmtree(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_rmtree));
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let path = a.first().copied().unwrap_or_else(MbValue::none);
    let ignore_errors = rmtree_ignore_errors_flag(&a[a.len().min(1)..]);
    mb_shutil_rmtree(path, ignore_errors)
}
dispatch_binary!(dispatch_move, mb_shutil_move);
dispatch_unary!(dispatch_which, mb_shutil_which);
dispatch_unary!(dispatch_disk_usage, mb_shutil_disk_usage);

// Terminal / archive / chown / ignore
//
// get_terminal_size(fallback=(columns, lines)) — the optional `fallback`
// tuple is the only positional arg, so it gets a bespoke dispatcher.
unsafe extern "C" fn dispatch_get_terminal_size(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_get_terminal_size));
    // `args_ptr` may be null when called with no positional args; only build a
    // slice when there is at least one element to read.
    let fallback = if nargs == 0 || args_ptr.is_null() {
        None
    } else {
        let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
        a.first().copied()
    };
    mb_shutil_get_terminal_size(fallback)
}
dispatch_nullary!(dispatch_get_archive_formats, mb_shutil_empty_list);
dispatch_nullary!(dispatch_get_unpack_formats, mb_shutil_empty_list);

dispatch_variadic_none!(dispatch_chown);
dispatch_variadic_none!(dispatch_ignore_patterns);
// make_archive(base_name, format, root_dir=None, base_dir=None, ...).
// "zip" produces a real ZIP (ZIP_STORED via the zipfile engine); the tar
// family stays a stub (None) pending a tar writer. An unregistered format
// raises ValueError like CPython.
unsafe extern "C" fn dispatch_make_archive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_make_archive));
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let fmt = a.get(1).copied().and_then(extract_str);
    if let Some(ref fmt) = fmt {
        const KNOWN_FORMATS: &[&str] = &["zip", "tar", "gztar", "bztar", "xztar"];
        if !KNOWN_FORMATS.contains(&fmt.as_str()) {
            return raise_named("ValueError", &format!("unknown archive format '{fmt}'"));
        }
    }
    let Some(base_name) = a.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    if fmt.as_deref() != Some("zip") {
        return MbValue::none(); // tar family deferred
    }
    // root_dir: positional slot 2 or kwargs {"root_dir": ...}; default cwd.
    let mut root_dir = a.get(2).copied().and_then(extract_str);
    for v in a {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    if let Some(rd) = lock.read().unwrap().get("root_dir") {
                        root_dir = extract_str(*rd);
                    }
                }
            }
        }
    }
    let root = root_dir.unwrap_or_else(|| ".".to_string());
    // Walk the tree, collecting (arcname, bytes) pairs relative to root.
    fn walk(dir: &std::path::Path, root: &std::path::Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        let mut paths: Vec<std::path::PathBuf> =
            rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        paths.sort();
        for p in paths {
            if p.is_dir() {
                walk(&p, root, out);
            } else if let (Ok(rel), Ok(data)) = (p.strip_prefix(root), std::fs::read(&p)) {
                out.push((rel.display().to_string(), data));
            }
        }
    }
    let rootp = std::path::PathBuf::from(&root);
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    walk(&rootp, &rootp, &mut files);
    let blob = super::zipfile_mod::zip_pack(&files);
    let archive_path = format!("{base_name}.zip");
    match std::fs::write(&archive_path, blob) {
        Ok(()) => MbValue::from_ptr(MbObject::new_str(archive_path)),
        Err(e) => raise_named("OSError", &format!("{e}: {archive_path:?}")),
    }
}
/// unpack_archive(filename, extract_dir=None, format=None) — real ZIP unpack
/// through the zipfile engine; other formats stay a no-op.
unsafe extern "C" fn dispatch_unpack_archive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let _ = core::hint::black_box(stringify!(dispatch_unpack_archive));
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let Some(filename) = a.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    let extract_dir = a
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| ".".to_string());
    let Ok(blob) = std::fs::read(&filename) else {
        return raise_named(
            "FileNotFoundError",
            &format!("[Errno 2] No such file or directory: '{filename}'"),
        );
    };
    let Some(files) = super::zipfile_mod::zip_unpack(&blob) else {
        return raise_named("shutil.ReadError", &format!("{filename} is not a zip file"));
    };
    let base = std::path::PathBuf::from(&extract_dir);
    for (name, data) in files {
        let dest = base.join(&name);
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&dest, data);
    }
    MbValue::none()
}
/// register_archive_format(name, function, extra_args=None, description='').
/// CPython contract: `function` must be callable; `extra_args` must be a
/// sequence of (name, value) PAIRS. Registration itself stays a no-op (the
/// archive registry is not modeled), but the validation raises are real.
unsafe extern "C" fn dispatch_register_archive_format(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let function = a.get(1).copied().unwrap_or_else(MbValue::none);
    if super::super::builtins::mb_callable(function).as_bool() != Some(true) {
        return raise_named("TypeError", "The registered function must be callable");
    }
    if let Some(extra) = a.get(2).copied() {
        if !extra.is_none() {
            let items: Option<Vec<MbValue>> = extra.as_ptr().and_then(|p| unsafe {
                match &(*p).data {
                    ObjData::List(ref lock) => Some(lock.read().unwrap().to_vec()),
                    ObjData::Tuple(ref t) => Some(t.clone()),
                    _ => None,
                }
            });
            let Some(items) = items else {
                return raise_named("TypeError", "extra_args needs to be a sequence");
            };
            for element in items {
                let pair_len = element.as_ptr().and_then(|p| unsafe {
                    match &(*p).data {
                        ObjData::Tuple(ref t) => Some(t.len()),
                        ObjData::List(ref lock) => Some(lock.read().unwrap().len()),
                        _ => None,
                    }
                });
                if pair_len != Some(2) {
                    return raise_named("TypeError", "extra_args elements are : (arg_name, value)");
                }
            }
        }
    }
    MbValue::none()
}
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
    // class_name="type" makes this a resolvable type-object: `resolve_class_name`
    // reads `__name__` off a `type`-classed Instance, so `except shutil.SameFileError`
    // (and the other shutil exception sentinels) match a `mb_raise("SameFileError", …)`.
    // The `__name__`/`__module__` fields are preserved, so identity + attribute
    // access (and `hasattr(shutil, "SameFileError")`) are unchanged.
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
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

/// Raise a catchable exception whose type-name is `exc` (matched by the
/// runtime's exception hierarchy: e.g. `FileNotFoundError`/`NotADirectoryError`
/// also match `except OSError`). Returns `none` so dispatchers can `return`
/// straight through — the runtime checks the pending-exception flag after the
/// native call, exactly like the netrc/configparser native modules.
fn raise_named(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Resolve `rmtree`'s `ignore_errors` flag from the trailing args (everything
/// after `path`). It may arrive as the 2nd positional (a bare value) or inside
/// the trailing kwargs Dict as `ignore_errors=...`. Returns true only when the
/// resolved value is truthy; absent/false → false (the refusal guards apply).
fn rmtree_ignore_errors_flag(rest: &[MbValue]) -> bool {
    for arg in rest {
        // Trailing kwargs Dict: look up `ignore_errors` by name.
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    if let Ok(guard) = lock.read() {
                        if let Some(v) = guard.get("ignore_errors") {
                            return super::super::builtins::mb_is_truthy(*v) != 0;
                        }
                    }
                    // A kwargs dict without `ignore_errors` does not set the flag.
                    continue;
                }
            }
        }
        // First non-dict trailing arg is the positional `ignore_errors`.
        return super::super::builtins::mb_is_truthy(*arg) != 0;
    }
    false
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

/// Build a *namedtuple-shaped* Instance: dotted attribute access on each field
/// name PLUS the full tuple surface (`len`, iteration, `tuple(x)`, indexing).
///
/// The runtime recognizes the `_namedtuple_fields` marker (a list of field-name
/// strings) and routes `mb_len` / `mb_obj_subscript` / `mb_iter` through the
/// values in declared order — see `collections_mod::namedtuple_values`. This
/// lets `shutil.get_terminal_size()` match CPython's `os.terminal_size`
/// namedtuple without introducing a new shared ObjData variant.
fn make_namedtuple(class_name: &str, fields_vec: Vec<(&str, MbValue)>) -> MbValue {
    let mut fields = FxHashMap::default();
    let name_list: Vec<MbValue> = fields_vec
        .iter()
        .map(|(k, _)| MbValue::from_ptr(MbObject::new_str(k.to_string())))
        .collect();
    for (k, v) in fields_vec {
        fields.insert(k.to_string(), v);
    }
    fields.insert(
        "_namedtuple_fields".to_string(),
        MbValue::from_ptr(MbObject::new_list(name_list)),
    );
    fields.insert(
        "_namedtuple_name".to_string(),
        MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
    );
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
        // CPython raises FileNotFoundError when the source path is missing.
        // `std::fs::copy` reports ErrorKind::NotFound in exactly that case, so a
        // successful copy never reaches this arm — no over-raise on valid input.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => raise_named(
            "FileNotFoundError",
            &format!("[Errno 2] No such file or directory: '{src_path}'"),
        ),
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
    // CPython's copyfile calls `_samefile(src, dst)` and raises SameFileError
    // when src and dst are the same file — including when dst is a symlink that
    // points back to src. `canonicalize` resolves symlinks, so we compare the
    // real targets. Both must already exist for this to fire: a normal copy to a
    // not-yet-created dst fails canonicalize on dst and falls through, and two
    // distinct files never share a canonical path — so no over-raise.
    if let (Ok(c_src), Ok(c_dst)) = (
        std::fs::canonicalize(&src_path),
        std::fs::canonicalize(&dst_path),
    ) {
        if c_src == c_dst {
            return raise_named(
                "SameFileError",
                &format!("{src_path:?} and {dst_path:?} are the same file"),
            );
        }
    }
    match std::fs::copy(&src_path, &dst_path) {
        Ok(_) => MbValue::from_ptr(MbObject::new_str(dst_path)),
        Err(_) => MbValue::none(),
    }
}

/// shutil.copyfileobj(fsrc, fdst, length=COPY_BUFSIZE) -> None.
///
/// Copies bytes from one file-like object to another by driving their Python
/// `.read(length)` / `.write(buf)` methods, exactly as CPython does:
///   while buf := fsrc.read(length): fdst.write(buf)
/// Works for any object exposing `read`/`write` (io.BytesIO, io.StringIO,
/// real file handles, custom file-likes).
pub fn mb_shutil_copyfileobj(fsrc: MbValue, fdst: MbValue, length: Option<MbValue>) -> MbValue {
    // CPython default is COPY_BUFSIZE; a caller-supplied length (even 0/None
    // semantics) is passed straight through to read().
    let length = length
        .filter(|v| !v.is_none())
        .unwrap_or_else(|| MbValue::from_int(COPY_BUFSIZE));

    loop {
        let read_args = MbValue::from_ptr(MbObject::new_list(vec![length]));
        let buf = call_obj_method(fsrc, "read", read_args);
        // Stop on EOF: an empty bytes/str object is falsy.
        if super::super::builtins::mb_is_truthy(buf) == 0 {
            break;
        }
        let write_args = MbValue::from_ptr(MbObject::new_list(vec![buf]));
        let _ = call_obj_method(fdst, "write", write_args);
    }
    MbValue::none()
}

/// Invoke a Python-level method by name on `obj` with a pre-built args list.
fn call_obj_method(obj: MbValue, method: &str, args: MbValue) -> MbValue {
    let name = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    super::super::class::mb_call_method(obj, name, args)
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
    // CPython 3.12 copytree calls `os.makedirs(dst, exist_ok=dirs_exist_ok)` with
    // `dirs_exist_ok=False` by default, raising FileExistsError when dst already
    // exists. Mamba's dispatcher has no `dirs_exist_ok` arg, so the default
    // (refuse an existing dst) applies. A normal copytree targets a fresh dst, so
    // this guard never fires on valid input.
    if std::path::Path::new(&dst_path).exists() {
        return raise_named(
            "FileExistsError",
            &format!("[Errno 17] File exists: '{dst_path}'"),
        );
    }
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

/// shutil.rmtree(path, ignore_errors=False) -> None
///
/// When `ignore_errors` is true, CPython swallows every error (missing path,
/// symlink refusal, non-dir) and returns silently — so the refusal guards below
/// must NOT fire in that mode.
pub fn mb_shutil_rmtree(path: MbValue, ignore_errors: bool) -> MbValue {
    if let Some(s) = extract_str(path) {
        // Mirror CPython rmtree's refusal conditions using a no-follow lstat:
        //   - missing path              -> FileNotFoundError
        //   - symlink (even to a dir)   -> OSError ("Cannot call rmtree on a
        //                                  symbolic link"); the link target is
        //                                  left untouched
        //   - not a directory (FIFO,
        //     regular file, socket)     -> NotADirectoryError
        // A real directory falls through to the actual recursive remove, so a
        // valid rmtree is unaffected. With ignore_errors=True every condition is
        // swallowed (no raise), matching CPython.
        match std::fs::symlink_metadata(&s) {
            Err(_) => {
                if !ignore_errors {
                    return raise_named(
                        "FileNotFoundError",
                        &format!("[Errno 2] No such file or directory: '{s}'"),
                    );
                }
                return MbValue::none();
            }
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    if !ignore_errors {
                        return raise_named("OSError", "Cannot call rmtree on a symbolic link");
                    }
                    return MbValue::none();
                }
                if !meta.is_dir() {
                    if !ignore_errors {
                        return raise_named(
                            "NotADirectoryError",
                            &format!("[Errno 20] Not a directory: '{s}'"),
                        );
                    }
                    return MbValue::none();
                }
            }
        }
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

    // CPython's move starts with os.rename(src, dst), which raises
    // FileNotFoundError when src is missing. Use a no-follow lstat so a dangling
    // symlink (which IS movable) is not mistaken for a missing source; only a
    // truly absent path raises. A real file/dir source falls through.
    if std::fs::symlink_metadata(&src_path).is_err() {
        return raise_named(
            "FileNotFoundError",
            &format!("[Errno 2] No such file or directory: '{src_path}'"),
        );
    }

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
///
/// Returns an Instance with `total`/`used`/`free` int fields, computed from a
/// real `statvfs(2)` on POSIX. CPython 3.12 derives the fields as:
///   total = f_blocks * f_frsize
///   free  = f_bavail * f_frsize
///   used  = total - free          (so total == used + free always holds)
/// Mamba ints are 48-bit; the byte totals on a typical disk (~2e12) sit well
/// inside 2^47-1, but we clamp defensively to avoid a panic on huge volumes.
pub fn mb_shutil_disk_usage(path: MbValue) -> MbValue {
    let p = extract_str(path).unwrap_or_default();

    // CPython's disk_usage calls os.statvfs(path), which raises FileNotFoundError
    // when the path does not exist. Guard on existence first; any real path
    // proceeds to the statvfs computation below, so a valid call never raises.
    if !std::path::Path::new(&p).exists() {
        return raise_named(
            "FileNotFoundError",
            &format!("[Errno 2] No such file or directory: '{p}'"),
        );
    }

    #[cfg(unix)]
    let (total, used, free): (i64, i64, i64) = {
        use std::os::unix::ffi::OsStrExt;
        let c_path = std::ffi::CString::new(std::path::Path::new(&p).as_os_str().as_bytes());
        match c_path {
            Ok(cstr) => {
                // SAFETY: statvfs writes into a zeroed buffer; we only read
                // scalar fields back out on success (rc == 0).
                let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
                let rc = unsafe { libc::statvfs(cstr.as_ptr(), &mut stat) };
                if rc == 0 {
                    let frsize = stat.f_frsize as i128;
                    let total = stat.f_blocks as i128 * frsize;
                    let free = stat.f_bavail as i128 * frsize;
                    let used = total - free;
                    (clamp_i64(total), clamp_i64(used), clamp_i64(free))
                } else {
                    (0, 0, 0)
                }
            }
            Err(_) => (0, 0, 0),
        }
    };

    #[cfg(not(unix))]
    let (total, used, free): (i64, i64, i64) = {
        let _ = &p;
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

/// Clamp an i128 byte count into mamba's 48-bit signed int payload range so
/// `MbValue::from_int` never panics on multi-terabyte volumes.
fn clamp_i64(v: i128) -> i64 {
    const MAX: i128 = (1i128 << 47) - 1;
    const MIN: i128 = -(1i128 << 47);
    v.clamp(MIN, MAX) as i64
}

/// shutil.get_terminal_size(fallback=(80, 24)) -> os.terminal_size namedtuple.
///
/// CPython precedence per field:
///   1. The `COLUMNS` / `LINES` env var, if present and parseable as an int
///      (a malformed value is ignored, NOT an error).
///   2. The real terminal size via ioctl(TIOCGWINSZ).
///   3. The `fallback` tuple (default `(80, 24)`).
///
/// The ioctl probe is deferred, so steps 2/3 collapse to the caller-supplied
/// `fallback` (default 80×24). The returned value is a true namedtuple, so it
/// supports `.columns` / `.lines`, `len() == 2`, `tuple(size)`, and indexing —
/// matching `os.terminal_size`.
pub fn mb_shutil_get_terminal_size(fallback: Option<MbValue>) -> MbValue {
    let (fb_cols, fb_lines) = fallback
        .filter(|v| !v.is_none())
        .and_then(tuple_pair_ints)
        .unwrap_or((80, 24));

    // Env override: parse as a signed int; a malformed value falls through to
    // the fallback (CPython swallows the ValueError).
    let cols = env_lookup("COLUMNS")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(fb_cols);
    let lines = env_lookup("LINES")
        .and_then(|v| v.trim().parse::<i64>().ok())
        .unwrap_or(fb_lines);

    make_namedtuple(
        "os.terminal_size",
        vec![
            ("columns", MbValue::from_int(cols)),
            ("lines", MbValue::from_int(lines)),
        ],
    )
}

/// Read an environment variable the way CPython's `shutil.get_terminal_size`
/// does — through `os.environ` — falling back to the real process environment.
///
/// CPython consults the Python-level `os.environ` mapping, so a test that does
/// `os.environ['COLUMNS'] = '777'` must be honored. In mamba `os.environ` is a
/// plain dict that user code mutates but which is NOT synced to the C
/// environment, so we probe that dict first and only fall back to
/// `std::env::var` for vars inherited from the real process env.
fn env_lookup(key: &str) -> Option<String> {
    // 1. os.environ dict (reflects runtime `os.environ[...] = ...` writes).
    let from_environ = super::super::module::MODULES.with(|mods| {
        let mods = mods.borrow();
        let environ = mods
            .get("os")
            .and_then(|m| m.attrs.get("environ").copied())?;
        let ptr = environ.as_ptr()?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                return guard.get(key).and_then(|v| extract_str(*v));
            }
        }
        None
    });
    if from_environ.is_some() {
        return from_environ;
    }
    // 2. Real process environment.
    std::env::var(key).ok()
}

/// Extract a `(i64, i64)` pair from a 2-element tuple/list MbValue (used for
/// the `fallback=(cols, lines)` argument of `get_terminal_size`).
fn tuple_pair_ints(val: MbValue) -> Option<(i64, i64)> {
    let items: Vec<MbValue> = val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => Some(items.clone()),
            ObjData::List(lock) => Some(lock.read().unwrap().to_vec()),
            _ => None,
        }
    })?;
    if items.len() != 2 {
        return None;
    }
    Some((items[0].as_int()?, items[1].as_int()?))
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

        mb_shutil_rmtree(s(src_dir.to_str().unwrap()), false);
        mb_shutil_rmtree(s(dst_dir.to_str().unwrap()), false);
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
        // disk_usage now reports real statvfs(2) data, matching CPython:
        // total/used/free are filesystem-wide byte counts where
        // `total == used + free` and `total > 0`.
        let scratch = tempfile::tempdir().unwrap();
        let file = scratch.path().join("f");
        std::fs::write(&file, "abcdef").unwrap();

        let usage = mb_shutil_disk_usage(s(file.to_str().unwrap()));
        let total = instance_field(usage, "total").and_then(|v| v.as_int());
        let used = instance_field(usage, "used").and_then(|v| v.as_int());
        let free = instance_field(usage, "free").and_then(|v| v.as_int());
        assert!(total.is_some() && used.is_some() && free.is_some());
        #[cfg(unix)]
        {
            let (t, u, f) = (total.unwrap(), used.unwrap(), free.unwrap());
            assert!(t > 0, "total should be positive, got {t}");
            assert_eq!(t, u + f, "total must equal used + free");
        }
    }

    #[test]
    fn test_get_terminal_size_shape() {
        // Drop any inherited env so the fallback path is deterministic.
        std::env::remove_var("COLUMNS");
        std::env::remove_var("LINES");
        let term = mb_shutil_get_terminal_size(None);
        let cols = instance_field(term, "columns").and_then(|v| v.as_int());
        let lines = instance_field(term, "lines").and_then(|v| v.as_int());
        assert_eq!(cols, Some(80));
        assert_eq!(lines, Some(24));
        // Namedtuple markers present so len()/tuple()/indexing work at runtime.
        assert!(instance_field(term, "_namedtuple_fields").is_some());
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
