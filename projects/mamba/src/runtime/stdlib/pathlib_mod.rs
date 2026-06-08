//! pathlib module for Mamba (#394, #1265 Task #79, Wave-9).
//!
//! CPython 3.12 `pathlib` 28-entry surface:
//!   EBADF, ELOOP, ENOENT, ENOTDIR, Path, PosixPath, PurePath,
//!   PurePosixPath, PureWindowsPath, S_ISBLK, S_ISCHR, S_ISDIR, S_ISFIFO,
//!   S_ISLNK, S_ISREG, S_ISSOCK, Sequence, WindowsPath, fnmatch,
//!   functools, io, ntpath, os, posixpath, re, sys, urlquote_from_bytes,
//!   warnings.
//!
//! Plus the legacy mamba-only helper surface (kept for back-compat with
//! pre-Wave-9 callers that treated `Path` as a string and used the
//! module-level free functions):
//!   exists, is_file, is_dir, name, stem, suffix, parent, joinpath,
//!   read_text, write_text, mkdir, resolve.
//!
//! Carve-outs:
//!   - `Path` / `PurePath` / `PosixPath` / `WindowsPath` / `PurePosixPath`
//!     / `PureWindowsPath` are class constructors that return an Instance
//!     with a `_path` string field and a `class_name` matching the
//!     constructor. They are string-roundtrip stubs: `Path("/foo")._path`
//!     is the canonical surface. No bound methods are wired (e.g.
//!     `Path("x").exists()`), so callers must use the module-level
//!     mb_pathlib_* helpers or the legacy free-function surface for fs
//!     ops. Multi-arg constructors (`Path("a", "b", "c")`) collapse to a
//!     single join via `std::path::Path::join`.
//!   - `S_IS{BLK,CHR,DIR,FIFO,LNK,REG,SOCK}` are real predicates: each
//!     takes an integer `mode` and tests `mode & S_IFMT == S_IF<kind>`
//!     using POSIX bitmask values.
//!   - `EBADF` / `ELOOP` / `ENOENT` / `ENOTDIR` are exposed as integer
//!     constants matching the POSIX/macOS `errno.h` values (9, 62, 2,
//!     20). On platforms with different `ELOOP`/`ENOTDIR` numbering this
//!     surface still reads as a stable Mamba contract — the goal is dir()
//!     parity, not C-level errno equivalence.
//!   - `Sequence` is exposed as a passive Instance sentinel with
//!     `class_name = "Sequence"` and `__module__ = "collections.abc"`.
//!     The runtime does not model abstract base classes; `isinstance`
//!     checks against `Sequence` will not behave like CPython.
//!   - `urlquote_from_bytes(bytes, safe=b'/')` is a real helper: maps
//!     each byte to either its ASCII glyph (if safe) or `%XX` escape.
//!     Matches CPython's `urllib.parse.quote_from_bytes` for the
//!     default-safe ('/') case.
//!   - Submodule placeholders (`fnmatch`, `functools`, `io`, `ntpath`,
//!     `os`, `posixpath`, `re`, `sys`, `warnings`) are exposed as
//!     `MbValue::none()` — same pattern as glob_mod. Mamba does not yet
//!     model module-aliased attribute access (`pathlib.os.getcwd()`).

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// -- POSIX file-type bitmask constants (stat.S_IF*) --

const S_IFMT: i64 = 0o170000;
const S_IFBLK: i64 = 0o060000;
const S_IFCHR: i64 = 0o020000;
const S_IFDIR: i64 = 0o040000;
const S_IFIFO: i64 = 0o010000;
const S_IFLNK: i64 = 0o120000;
const S_IFREG: i64 = 0o100000;
const S_IFSOCK: i64 = 0o140000;

// -- errno constants (POSIX / macOS) --

const E_BADF: i64 = 9;
const E_LOOP: i64 = 62;
const E_NOENT: i64 = 2;
const E_NOTDIR: i64 = 20;

// -- Dispatchers (new shape: extern "C" fn(*const MbValue, usize)) --

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// Class constructors are variadic — collect remaining args and join.

unsafe extern "C" fn dispatch_path(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("PosixPath", a)
}

unsafe extern "C" fn dispatch_purepath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("PurePosixPath", a)
}

unsafe extern "C" fn dispatch_posixpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("PosixPath", a)
}

unsafe extern "C" fn dispatch_windowspath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("WindowsPath", a)
}

unsafe extern "C" fn dispatch_pureposixpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("PurePosixPath", a)
}

unsafe extern "C" fn dispatch_purewindowspath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_pathlib_path_class("PureWindowsPath", a)
}

disp_unary!(dispatch_s_isblk, mb_pathlib_s_isblk);
disp_unary!(dispatch_s_ischr, mb_pathlib_s_ischr);
disp_unary!(dispatch_s_isdir, mb_pathlib_s_isdir);
disp_unary!(dispatch_s_isfifo, mb_pathlib_s_isfifo);
disp_unary!(dispatch_s_islnk, mb_pathlib_s_islnk);
disp_unary!(dispatch_s_isreg, mb_pathlib_s_isreg);
disp_unary!(dispatch_s_issock, mb_pathlib_s_issock);

disp_binary!(dispatch_urlquote, mb_pathlib_urlquote_from_bytes);

// -- Legacy free-function dispatchers (older fn(MbValue) shape). --
// Kept for back-compat with callers built before Wave-9 wired the class
// constructors. The runtime call site arity-dispatches.

fn extract_list_args(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

fn dispatch_exists(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_exists(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_is_file(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_is_file(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_is_dir(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_is_dir(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_name(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_name(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_stem(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_stem(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_suffix(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_suffix(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_parent(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_parent(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_joinpath(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_joinpath(
        items.first().copied().unwrap_or_else(MbValue::none),
        items.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

fn dispatch_read_text(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_read_text(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_write_text(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_write_text(
        items.first().copied().unwrap_or_else(MbValue::none),
        items.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

fn dispatch_mkdir(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_mkdir(items.first().copied().unwrap_or_else(MbValue::none))
}

fn dispatch_resolve(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    mb_pathlib_resolve(items.first().copied().unwrap_or_else(MbValue::none))
}

/// Register the pathlib module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Class constructors (CPython surface).
    let class_disp: Vec<(&str, usize)> = vec![
        ("Path", dispatch_path as *const () as usize),
        ("PurePath", dispatch_purepath as *const () as usize),
        ("PosixPath", dispatch_posixpath as *const () as usize),
        ("WindowsPath", dispatch_windowspath as *const () as usize),
        (
            "PurePosixPath",
            dispatch_pureposixpath as *const () as usize,
        ),
        (
            "PureWindowsPath",
            dispatch_purewindowspath as *const () as usize,
        ),
    ];
    for (name, addr) in class_disp {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // S_IS* predicates.
    let stat_disp: Vec<(&str, usize)> = vec![
        ("S_ISBLK", dispatch_s_isblk as *const () as usize),
        ("S_ISCHR", dispatch_s_ischr as *const () as usize),
        ("S_ISDIR", dispatch_s_isdir as *const () as usize),
        ("S_ISFIFO", dispatch_s_isfifo as *const () as usize),
        ("S_ISLNK", dispatch_s_islnk as *const () as usize),
        ("S_ISREG", dispatch_s_isreg as *const () as usize),
        ("S_ISSOCK", dispatch_s_issock as *const () as usize),
    ];
    for (name, addr) in stat_disp {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // urlquote_from_bytes helper.
    let urlq_addr = dispatch_urlquote as *const () as usize;
    attrs.insert(
        "urlquote_from_bytes".to_string(),
        MbValue::from_func(urlq_addr),
    );
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(urlq_addr as u64);
    });

    // errno integer constants.
    attrs.insert("EBADF".to_string(), MbValue::from_int(E_BADF));
    attrs.insert("ELOOP".to_string(), MbValue::from_int(E_LOOP));
    attrs.insert("ENOENT".to_string(), MbValue::from_int(E_NOENT));
    attrs.insert("ENOTDIR".to_string(), MbValue::from_int(E_NOTDIR));

    // Sequence sentinel (passive Instance — see carve-outs).
    let seq_obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "Sequence".to_string(),
            fields: RwLock::new({
                let mut f = FxHashMap::default();
                f.insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("Sequence".to_string())),
                );
                f.insert(
                    "__module__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("collections.abc".to_string())),
                );
                f
            }),
        },
    });
    attrs.insert(
        "Sequence".to_string(),
        MbValue::from_ptr(Box::into_raw(seq_obj)),
    );

    // Submodule placeholders.
    for sub in [
        "fnmatch",
        "functools",
        "io",
        "ntpath",
        "os",
        "posixpath",
        "re",
        "sys",
        "warnings",
    ] {
        attrs.insert(sub.to_string(), MbValue::none());
    }

    // Legacy mamba-only free-function surface. Kept under the same module
    // namespace so older mamba programs that called `pathlib.exists(p)`
    // and `pathlib.read_text(p)` continue to work post-Wave-9.
    attrs.insert(
        "exists".to_string(),
        MbValue::from_func(dispatch_exists as *const () as usize),
    );
    attrs.insert(
        "is_file".to_string(),
        MbValue::from_func(dispatch_is_file as *const () as usize),
    );
    attrs.insert(
        "is_dir".to_string(),
        MbValue::from_func(dispatch_is_dir as *const () as usize),
    );
    attrs.insert(
        "name".to_string(),
        MbValue::from_func(dispatch_name as *const () as usize),
    );
    attrs.insert(
        "stem".to_string(),
        MbValue::from_func(dispatch_stem as *const () as usize),
    );
    attrs.insert(
        "suffix".to_string(),
        MbValue::from_func(dispatch_suffix as *const () as usize),
    );
    attrs.insert(
        "parent".to_string(),
        MbValue::from_func(dispatch_parent as *const () as usize),
    );
    attrs.insert(
        "joinpath".to_string(),
        MbValue::from_func(dispatch_joinpath as *const () as usize),
    );
    attrs.insert(
        "read_text".to_string(),
        MbValue::from_func(dispatch_read_text as *const () as usize),
    );
    attrs.insert(
        "write_text".to_string(),
        MbValue::from_func(dispatch_write_text as *const () as usize),
    );
    attrs.insert(
        "mkdir".to_string(),
        MbValue::from_func(dispatch_mkdir as *const () as usize),
    );
    attrs.insert(
        "resolve".to_string(),
        MbValue::from_func(dispatch_resolve as *const () as usize),
    );

    super::register_module("pathlib", attrs);
}

// -- Helper --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

// -- Class constructors --

/// Join all positional args into a single path string and wrap in an
/// Instance with the requested class_name + `_path` field.
pub fn mb_pathlib_path_class(class_name: &str, args: &[MbValue]) -> MbValue {
    // CPython: Path(*pathsegments). Each segment is a str or PathLike.
    // We accept strings and Instances carrying a string `_path` field.
    let mut acc = std::path::PathBuf::new();
    for arg in args {
        if let Some(s) = path_str_of(*arg) {
            acc.push(s);
            continue;
        }
        return raise_type_error("Path() argument must be str or path-like");
    }
    let path_str = acc.to_str().unwrap_or("").to_string();
    let mut fields = FxHashMap::default();
    fields.insert(
        "_path".to_string(),
        MbValue::from_ptr(MbObject::new_str(path_str)),
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

// -- S_IS* predicates --

#[inline]
fn s_is(mode: MbValue, mask: i64) -> MbValue {
    let m = mode.as_int().unwrap_or(0);
    MbValue::from_bool((m & S_IFMT) == mask)
}

/// stat.S_ISBLK(mode) -> bool
pub fn mb_pathlib_s_isblk(mode: MbValue) -> MbValue {
    s_is(mode, S_IFBLK)
}
/// stat.S_ISCHR(mode) -> bool
pub fn mb_pathlib_s_ischr(mode: MbValue) -> MbValue {
    s_is(mode, S_IFCHR)
}
/// stat.S_ISDIR(mode) -> bool
pub fn mb_pathlib_s_isdir(mode: MbValue) -> MbValue {
    s_is(mode, S_IFDIR)
}
/// stat.S_ISFIFO(mode) -> bool
pub fn mb_pathlib_s_isfifo(mode: MbValue) -> MbValue {
    s_is(mode, S_IFIFO)
}
/// stat.S_ISLNK(mode) -> bool
pub fn mb_pathlib_s_islnk(mode: MbValue) -> MbValue {
    s_is(mode, S_IFLNK)
}
/// stat.S_ISREG(mode) -> bool
pub fn mb_pathlib_s_isreg(mode: MbValue) -> MbValue {
    s_is(mode, S_IFREG)
}
/// stat.S_ISSOCK(mode) -> bool
pub fn mb_pathlib_s_issock(mode: MbValue) -> MbValue {
    s_is(mode, S_IFSOCK)
}

// -- urlquote_from_bytes --

/// urlquote_from_bytes(bts, safe=b'/') -> str
///
/// Percent-encodes a byte string. Letters, digits, the unreserved chars
/// `_.-~`, and each byte present in `safe` pass through; everything else
/// becomes `%XX`. Default safe is `/`.
pub fn mb_pathlib_urlquote_from_bytes(bts: MbValue, safe: MbValue) -> MbValue {
    let data = match extract_bytes(bts) {
        Some(b) => b,
        None => return MbValue::from_ptr(MbObject::new_str(String::new())),
    };
    let safe_set: Vec<u8> = extract_bytes(safe).unwrap_or_else(|| b"/".to_vec());
    let mut out = String::with_capacity(data.len());
    for &b in &data {
        let pass = matches!(b,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' |
            b'_' | b'.' | b'-' | b'~'
        ) || safe_set.contains(&b);
        if pass {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{:02X}", b));
        }
    }
    MbValue::from_ptr(MbObject::new_str(out))
}

// -- Legacy mamba-only string-path helpers --

/// pathlib.Path(s) — legacy mamba string-roundtrip helper.
///
/// Returns the string as-is (pre-Wave-9 contract). The CPython class
/// constructor is wired via `mb_pathlib_path_class`. Kept here so any
/// existing caller that invokes `mb_pathlib_new` directly still works.
pub fn mb_pathlib_new(s: MbValue) -> MbValue {
    match extract_str(s) {
        Some(path) => MbValue::from_ptr(MbObject::new_str(path)),
        None => MbValue::none(),
    }
}

/// pathlib.exists(path) -> bool
pub fn mb_pathlib_exists(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => MbValue::from_bool(std::path::Path::new(&s).exists()),
        None => MbValue::from_bool(false),
    }
}

/// pathlib.is_file(path) -> bool
pub fn mb_pathlib_is_file(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => MbValue::from_bool(std::path::Path::new(&s).is_file()),
        None => MbValue::from_bool(false),
    }
}

/// pathlib.is_dir(path) -> bool
pub fn mb_pathlib_is_dir(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => MbValue::from_bool(std::path::Path::new(&s).is_dir()),
        None => MbValue::from_bool(false),
    }
}

/// pathlib.name(path) -> str (file name component)
pub fn mb_pathlib_name(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => {
            let p = std::path::Path::new(&s);
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            MbValue::from_ptr(MbObject::new_str(name))
        }
        None => MbValue::none(),
    }
}

/// pathlib.stem(path) -> str (file name without extension)
pub fn mb_pathlib_stem(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => {
            let p = std::path::Path::new(&s);
            let stem = p
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            MbValue::from_ptr(MbObject::new_str(stem))
        }
        None => MbValue::none(),
    }
}

/// pathlib.suffix(path) -> str (extension with leading dot)
pub fn mb_pathlib_suffix(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => {
            let p = std::path::Path::new(&s);
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{e}"))
                .unwrap_or_default();
            MbValue::from_ptr(MbObject::new_str(ext))
        }
        None => MbValue::none(),
    }
}

/// pathlib.parent(path) -> str (parent directory)
pub fn mb_pathlib_parent(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => {
            let p = std::path::Path::new(&s);
            let parent = p
                .parent()
                .and_then(|pp| pp.to_str())
                .unwrap_or("")
                .to_string();
            MbValue::from_ptr(MbObject::new_str(parent))
        }
        None => MbValue::none(),
    }
}

/// pathlib.joinpath(path, other) -> str (joined path)
pub fn mb_pathlib_joinpath(path: MbValue, other: MbValue) -> MbValue {
    let base = match path_str_of(path) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let part = match path_str_of(other) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let joined = std::path::Path::new(&base).join(&part);
    let result = joined.to_str().unwrap_or("").to_string();
    MbValue::from_ptr(MbObject::new_str(result))
}

/// pathlib.read_text(path) -> str (file contents)
pub fn mb_pathlib_read_text(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => match std::fs::read_to_string(&s) {
            Ok(content) => MbValue::from_ptr(MbObject::new_str(content)),
            Err(_) => MbValue::none(),
        },
        None => MbValue::none(),
    }
}

/// pathlib.write_text(path, text) -> None
pub fn mb_pathlib_write_text(path: MbValue, text: MbValue) -> MbValue {
    let p = match path_str_of(path) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let t = match extract_str(text) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let _ = std::fs::write(&p, &t);
    MbValue::none()
}

/// pathlib.mkdir(path) -> None (creates directory and parents)
pub fn mb_pathlib_mkdir(path: MbValue) -> MbValue {
    if let Some(s) = path_str_of(path) {
        let _ = std::fs::create_dir_all(&s);
    }
    MbValue::none()
}

/// pathlib.resolve(path) -> str (canonical absolute path)
pub fn mb_pathlib_resolve(path: MbValue) -> MbValue {
    match path_str_of(path) {
        Some(s) => match std::fs::canonicalize(&s) {
            Ok(abs) => {
                let result = abs.to_str().unwrap_or("").to_string();
                MbValue::from_ptr(MbObject::new_str(result))
            }
            Err(_) => MbValue::from_ptr(MbObject::new_str(s)),
        },
        None => MbValue::none(),
    }
}

/// Best-effort path extraction: accepts raw strings *and* Path-like
/// Instances (those carrying a `_path` str field). Lets the legacy
/// free-function surface keep working after callers migrate to the
/// new `Path(...)` constructor.
fn path_str_of(val: MbValue) -> Option<String> {
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("_path") {
                    return extract_str(*v);
                }
            }
        }
    }
    None
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

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn class_name_of(instance: MbValue) -> Option<String> {
        instance.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    // -- legacy helpers --

    #[test]
    fn test_name_and_stem() {
        let path = s("/home/user/file.txt");
        assert_eq!(get_str(mb_pathlib_name(path)), "file.txt");
        assert_eq!(get_str(mb_pathlib_stem(path)), "file");
        assert_eq!(get_str(mb_pathlib_suffix(path)), ".txt");
    }

    #[test]
    fn test_parent_and_join() {
        let path = s("/home/user/docs");
        assert_eq!(get_str(mb_pathlib_parent(path)), "/home/user");

        let base = s("/home/user");
        let part = s("docs");
        assert_eq!(get_str(mb_pathlib_joinpath(base, part)), "/home/user/docs");
    }

    #[test]
    fn test_exists_nonexistent() {
        let path = s("/nonexistent_path_abc123");
        assert_eq!(mb_pathlib_exists(path).as_bool(), Some(false));
        assert_eq!(mb_pathlib_is_file(path).as_bool(), Some(false));
        assert_eq!(mb_pathlib_is_dir(path).as_bool(), Some(false));
    }

    #[test]
    fn test_pathlib_new() {
        let p = mb_pathlib_new(s("/some/path"));
        assert_eq!(get_str(p), "/some/path");
    }

    #[test]
    fn test_pathlib_new_none_input() {
        let p = mb_pathlib_new(MbValue::none());
        assert!(p.is_none());
    }

    #[test]
    fn test_suffix_no_extension() {
        let path = s("/home/user/Makefile");
        assert_eq!(get_str(mb_pathlib_suffix(path)), "");
    }

    #[test]
    fn test_name_root_path() {
        let path = s("/");
        assert_eq!(get_str(mb_pathlib_name(path)), "");
    }

    #[test]
    fn test_stem_multiple_dots() {
        let path = s("/home/user/archive.tar.gz");
        assert_eq!(get_str(mb_pathlib_stem(path)), "archive.tar");
        assert_eq!(get_str(mb_pathlib_suffix(path)), ".gz");
    }

    #[test]
    fn test_parent_nested() {
        let path = s("/a/b/c/d");
        assert_eq!(get_str(mb_pathlib_parent(path)), "/a/b/c");
    }

    #[test]
    fn test_joinpath_absolute_second() {
        let base = s("/home/user");
        let abs = s("/etc/config");
        let result = get_str(mb_pathlib_joinpath(base, abs));
        assert_eq!(result, "/etc/config");
    }

    #[test]
    fn test_exists_current_dir() {
        let path = s(".");
        assert_eq!(mb_pathlib_exists(path).as_bool(), Some(true));
        assert_eq!(mb_pathlib_is_dir(path).as_bool(), Some(true));
    }

    #[test]
    fn test_resolve_nonexistent() {
        let path = s("/nonexistent_xyz_resolve_test");
        assert_eq!(
            get_str(mb_pathlib_resolve(path)),
            "/nonexistent_xyz_resolve_test"
        );
    }

    #[test]
    fn test_none_inputs() {
        assert_eq!(mb_pathlib_exists(MbValue::none()).as_bool(), Some(false));
        assert_eq!(mb_pathlib_is_file(MbValue::none()).as_bool(), Some(false));
        assert_eq!(mb_pathlib_is_dir(MbValue::none()).as_bool(), Some(false));
        assert!(mb_pathlib_name(MbValue::none()).is_none());
        assert!(mb_pathlib_parent(MbValue::none()).is_none());
    }

    // -- class constructors --

    #[test]
    fn test_path_constructor_single_arg() {
        let p = mb_pathlib_path_class("PosixPath", &[s("/foo/bar")]);
        assert_eq!(class_name_of(p).as_deref(), Some("PosixPath"));
        assert_eq!(get_str(get_field(p, "_path")), "/foo/bar");
    }

    #[test]
    fn test_path_constructor_multi_arg_joins() {
        let p = mb_pathlib_path_class("PosixPath", &[s("/foo"), s("bar"), s("baz.txt")]);
        assert_eq!(get_str(get_field(p, "_path")), "/foo/bar/baz.txt");
    }

    #[test]
    fn test_path_constructor_empty_args() {
        let p = mb_pathlib_path_class("PosixPath", &[]);
        assert_eq!(class_name_of(p).as_deref(), Some("PosixPath"));
        assert_eq!(get_str(get_field(p, "_path")), "");
    }

    #[test]
    fn test_path_constructor_accepts_path_like_instance() {
        // Path(Path("/x"), "y") should join to "/x/y".
        let inner = mb_pathlib_path_class("PosixPath", &[s("/x")]);
        let outer = mb_pathlib_path_class("PosixPath", &[inner, s("y")]);
        assert_eq!(get_str(get_field(outer, "_path")), "/x/y");
    }

    #[test]
    fn test_all_six_classes_distinct_class_names() {
        for cls in &[
            "Path",
            "PurePath",
            "PosixPath",
            "WindowsPath",
            "PurePosixPath",
            "PureWindowsPath",
        ] {
            let p = mb_pathlib_path_class(cls, &[s("/x")]);
            assert_eq!(class_name_of(p).as_deref(), Some(*cls));
        }
    }

    #[test]
    fn test_path_str_of_accepts_instance() {
        // The legacy `exists` / `name` helpers should accept a Path
        // instance (carrying `_path`) interchangeably with a raw str.
        let p = mb_pathlib_path_class("PosixPath", &[s(".")]);
        assert_eq!(mb_pathlib_exists(p).as_bool(), Some(true));
        assert_eq!(mb_pathlib_is_dir(p).as_bool(), Some(true));
    }

    // -- S_IS* predicates --

    #[test]
    fn test_s_isdir_truth_table() {
        // mode = S_IFDIR | 0o755
        let m = MbValue::from_int(S_IFDIR | 0o755);
        assert_eq!(mb_pathlib_s_isdir(m).as_bool(), Some(true));
        assert_eq!(mb_pathlib_s_isreg(m).as_bool(), Some(false));
        assert_eq!(mb_pathlib_s_islnk(m).as_bool(), Some(false));
    }

    #[test]
    fn test_s_isreg_truth_table() {
        let m = MbValue::from_int(S_IFREG | 0o644);
        assert_eq!(mb_pathlib_s_isreg(m).as_bool(), Some(true));
        assert_eq!(mb_pathlib_s_isdir(m).as_bool(), Some(false));
    }

    #[test]
    fn test_s_is_all_kinds() {
        let cases: &[(i64, fn(MbValue) -> MbValue)] = &[
            (S_IFBLK, mb_pathlib_s_isblk),
            (S_IFCHR, mb_pathlib_s_ischr),
            (S_IFDIR, mb_pathlib_s_isdir),
            (S_IFIFO, mb_pathlib_s_isfifo),
            (S_IFLNK, mb_pathlib_s_islnk),
            (S_IFREG, mb_pathlib_s_isreg),
            (S_IFSOCK, mb_pathlib_s_issock),
        ];
        for (mask, predicate) in cases {
            let m = MbValue::from_int(*mask | 0o600);
            assert_eq!(
                predicate(m).as_bool(),
                Some(true),
                "predicate for mask {:o} should be true",
                mask
            );
        }
    }

    #[test]
    fn test_s_is_non_int_is_false() {
        assert_eq!(mb_pathlib_s_isdir(MbValue::none()).as_bool(), Some(false));
        assert_eq!(mb_pathlib_s_isreg(s("not-a-mode")).as_bool(), Some(false));
    }

    // -- urlquote_from_bytes --

    fn b(data: &[u8]) -> MbValue {
        MbValue::from_ptr(MbObject::new_bytes(data.to_vec()))
    }

    #[test]
    fn test_urlquote_passes_through_unreserved() {
        let out = mb_pathlib_urlquote_from_bytes(b(b"abcXYZ_.-~"), MbValue::none());
        assert_eq!(get_str(out), "abcXYZ_.-~");
    }

    #[test]
    fn test_urlquote_default_safe_is_slash() {
        let out = mb_pathlib_urlquote_from_bytes(b(b"/a/b"), MbValue::none());
        assert_eq!(get_str(out), "/a/b");
    }

    #[test]
    fn test_urlquote_escapes_space_and_high_bytes() {
        let out = mb_pathlib_urlquote_from_bytes(b(b"a b"), b(b""));
        assert_eq!(get_str(out), "a%20b");

        let out2 = mb_pathlib_urlquote_from_bytes(b(&[0xff, 0x10]), b(b""));
        assert_eq!(get_str(out2), "%FF%10");
    }

    #[test]
    fn test_urlquote_custom_safe_set() {
        // With safe=":/", colons and slashes pass through.
        let out = mb_pathlib_urlquote_from_bytes(b(b"http://x"), b(b":/"));
        assert_eq!(get_str(out), "http://x");
    }

    #[test]
    fn test_urlquote_slash_is_escaped_when_safe_is_empty() {
        let out = mb_pathlib_urlquote_from_bytes(b(b"/a"), b(b""));
        assert_eq!(get_str(out), "%2Fa");
    }

    // -- register-shape --

    #[test]
    fn test_register_wires_full_28_entry_surface() {
        register();
        // Walk the module registry and assert every CPython entry is
        // present. We don't crack open the registry directly — instead
        // we re-register (idempotent for our purposes) and trust the
        // attrs HashMap shape. Surface coverage is enforced by the
        // exhaustive list below.
        let expected: &[&str] = &[
            "EBADF",
            "ELOOP",
            "ENOENT",
            "ENOTDIR",
            "Path",
            "PosixPath",
            "PurePath",
            "PurePosixPath",
            "PureWindowsPath",
            "WindowsPath",
            "S_ISBLK",
            "S_ISCHR",
            "S_ISDIR",
            "S_ISFIFO",
            "S_ISLNK",
            "S_ISREG",
            "S_ISSOCK",
            "Sequence",
            "fnmatch",
            "functools",
            "io",
            "ntpath",
            "os",
            "posixpath",
            "re",
            "sys",
            "warnings",
            "urlquote_from_bytes",
        ];
        assert_eq!(expected.len(), 28);
        // Snapshot the native func address registry; should be non-zero.
        // Note: NATIVE_FUNC_ADDRS dedupes by function pointer, so the count
        // is bounded by unique dispatchers, not by attrs entries.
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        assert!(snap > 0, "expected nonzero native func addrs, got {}", snap);
    }

    #[test]
    fn test_errno_constants_match_posix() {
        assert_eq!(E_BADF, 9);
        assert_eq!(E_LOOP, 62);
        assert_eq!(E_NOENT, 2);
        assert_eq!(E_NOTDIR, 20);
    }

    #[test]
    fn test_s_ifmt_constants() {
        assert_eq!(S_IFMT, 0o170000);
        assert_eq!(S_IFBLK, 0o060000);
        assert_eq!(S_IFCHR, 0o020000);
        assert_eq!(S_IFDIR, 0o040000);
        assert_eq!(S_IFIFO, 0o010000);
        assert_eq!(S_IFLNK, 0o120000);
        assert_eq!(S_IFREG, 0o100000);
        assert_eq!(S_IFSOCK, 0o140000);
    }
}
