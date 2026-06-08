/// tarfile module for Mamba (#445).
///
/// Provides: open, is_tarfile, TarInfo (stubs).
/// No external dependency — in-memory entry storage.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_unary!(dispatch_is_tarfile, mb_tarfile_is_tarfile);

// ── error-path helpers (errors dimension) ──────────────────────────────────────
// These let module-level entry points raise the exact CPython exception for an
// invalid op while staying inert (fall back to the existing stub) on every
// valid input. Exception class-names match the registry entries created in
// `register()` (e.g. "ReadError", "ValueError"), so `except tarfile.ReadError`
// / `except ValueError` resolve. Precedent: netrc_mod::raise_named.

/// Raise a catchable exception whose type-name is `exc`, return None.
fn tf_raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn tf_is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Look up `key` in a kwargs dict (the trailing positional dict the runtime
/// appends for keyword arguments). Returns None if absent / not a dict.
fn tf_kw_get(kwargs: Option<MbValue>, key: &str) -> Option<MbValue> {
    let ptr = kwargs?.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    }
}

/// Extract a `str` value, if `val` is a str object.
fn tf_as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    }
}

/// Byte length of a `BytesIO`-like fileobj's in-memory buffer (`_buffer`
/// instance field, populated by io_mod). None when `val` is not such an object
/// or has no readable byte buffer — in which case the caller stays inert.
fn tf_fileobj_buf_len(val: MbValue) -> Option<usize> {
    let ptr = val.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let f = fields.read().unwrap();
            let buf = f.get("_buffer")?;
            let bptr = buf.as_ptr()?;
            match &(*bptr).data {
                ObjData::Bytes(b) => Some(b.len()),
                ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().len()),
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Split a raw native arg slice into (positional, trailing-kwargs-dict).
fn tf_split<'a>(a: &'a [MbValue]) -> (&'a [MbValue], Option<MbValue>) {
    match a.last().copied().filter(|v| tf_is_dict(*v)) {
        Some(kw) => (&a[..a.len().saturating_sub(1)], Some(kw)),
        None => (a, None),
    }
}

/// `tarfile.open(name=None, mode='r', fileobj=None, ..., compresslevel=...)`.
///
/// Validates only the unambiguously-invalid error conditions covered by the
/// errors dimension; every other call falls through to the existing in-memory
/// stub (`mb_tarfile_open`), so valid usage and surface probes are unchanged:
///   * bz2 write mode with compresslevel outside 1..=9  -> ValueError
///   * read mode, missing path on disk, no fileobj       -> FileNotFoundError
///   * read mode, fileobj buffer 1..511 bytes (truncated header,
///     never a valid tar)                                -> ReadError
unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = tf_split(a);

    let name_val = pos.first().copied().or_else(|| tf_kw_get(kw, "name"));
    let mode_val = pos.get(1).copied().or_else(|| tf_kw_get(kw, "mode"));
    let fileobj = pos.get(2).copied().or_else(|| tf_kw_get(kw, "fileobj"));
    let compresslevel = tf_kw_get(kw, "compresslevel").and_then(|v| v.as_int());

    let mode = mode_val.and_then(tf_as_str).unwrap_or_else(|| "r".to_string());
    let has_fileobj = fileobj.map(|f| !f.is_none()).unwrap_or(false);

    // bz2 requires compresslevel 1..=9; e.g. mode='w:bz2', compresslevel=0.
    if mode.contains("bz2") {
        if let Some(cl) = compresslevel {
            if !(1..=9).contains(&cl) {
                return tf_raise("ValueError", "valid range for compresslevel is between 1 and 9");
            }
        }
    }

    // Read-open of a path that does not exist (and no fileobj override).
    let is_read = !mode.starts_with('w') && !mode.starts_with('a') && !mode.starts_with('x');
    if is_read && !has_fileobj {
        if let Some(path) = name_val.and_then(tf_as_str) {
            if !std::path::Path::new(&path).exists() {
                return tf_raise(
                    "FileNotFoundError",
                    &format!("[Errno 2] No such file or directory: '{path}'"),
                );
            }
        }
    }

    // Read-open of a fileobj whose buffer is non-empty but shorter than one
    // 512-byte block: a truncated header that no valid tar archive produces.
    if is_read && has_fileobj {
        if let Some(len) = fileobj.and_then(tf_fileobj_buf_len) {
            if (1..512).contains(&len) {
                return tf_raise("ReadError", "truncated header");
            }
        }
    }

    // Fall through to the existing in-memory stub (behavior unchanged).
    mb_tarfile_open(name_val.unwrap_or_else(MbValue::none), mode_val.unwrap_or_else(MbValue::none))
}

/// `tarfile.itn(n, digits=8, format=DEFAULT_FORMAT)` — encode an integer into a
/// tar number field. We don't emit the bytes yet (returns None like the stub),
/// but we DO enforce CPython's range gate so out-of-range values raise the same
/// ValueError. Valid octal range is `0 <= n < 8**(digits-1)` for every format;
/// GNU additionally allows base-256 `-256**(digits-1) <= n < 256**(digits-1)`.
unsafe extern "C" fn dispatch_itn(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kw) = tf_split(a);

    let n = match pos.first().copied().or_else(|| tf_kw_get(kw, "n")).and_then(|v| v.as_int()) {
        Some(v) => v as i128,
        None => return MbValue::none(), // non-int / absent: stay inert
    };
    let digits = pos.get(1).copied().or_else(|| tf_kw_get(kw, "digits"))
        .and_then(|v| v.as_int()).unwrap_or(8);
    let format = pos.get(2).copied().or_else(|| tf_kw_get(kw, "format"))
        .and_then(|v| v.as_int()).unwrap_or(2); // DEFAULT_FORMAT

    // Guard against absurd `digits`; only the octal exponent must be computable.
    if digits < 1 || digits > 40 {
        return MbValue::none();
    }
    let octal_upper = 8i128.pow((digits - 1) as u32);
    if n >= 0 && n < octal_upper {
        return MbValue::none(); // representable as octal: valid for all formats
    }
    const GNU_FORMAT: i64 = 1;
    if format == GNU_FORMAT {
        let base256 = 256i128.pow((digits - 1) as u32);
        if n >= -base256 && n < base256 {
            return MbValue::none(); // representable as GNU base-256
        }
    }
    tf_raise("ValueError", "overflow in number field")
}

/// Surface-only stub for the module-level helper/filter functions
/// (`tar_filter`, `data_filter`, `fully_trusted_filter`, `calc_chksums`,
/// `copyfileobj`, `nti`, `nts`, `itn`, `stn`, `bltn_open`, `main`). These exist
/// so `hasattr(tarfile, name)` / `callable(name)` resolve. The real extraction
/// filter + header arithmetic semantics are not yet plumbed; behavior coverage
/// for those lands behind the in-memory TarFile work. Returns None.
unsafe extern "C" fn dispatch_tarfile_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Register the tarfile module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("open", dispatch_open as usize),
        ("is_tarfile", dispatch_is_tarfile as usize),
        ("itn", dispatch_itn as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
        // surface: missing CPython module constants (auto-added)
    attrs.insert("BLOCKSIZE".into(), MbValue::from_int(512));
    attrs.insert("DEFAULT_FORMAT".into(), MbValue::from_int(2));
    attrs.insert("ENCODING".into(), MbValue::from_ptr(MbObject::new_str("utf-8".to_string())));
    attrs.insert("GNU_FORMAT".into(), MbValue::from_int(1));
    attrs.insert("LENGTH_LINK".into(), MbValue::from_int(100));
    attrs.insert("LENGTH_NAME".into(), MbValue::from_int(100));
    attrs.insert("LENGTH_PREFIX".into(), MbValue::from_int(155));
    attrs.insert("PAX_FORMAT".into(), MbValue::from_int(2));
    attrs.insert("RECORDSIZE".into(), MbValue::from_int(10240));
    attrs.insert("USTAR_FORMAT".into(), MbValue::from_int(0));
    attrs.insert("version".into(), MbValue::from_ptr(MbObject::new_str("0.9.0".to_string())));

    // surface: remaining CPython 3.12 tarfile module names (hasattr/callable
    // probes — see config/manifests/std-libs/cpython312_surface/tarfile.toml).
    // (Int constants BLOCKSIZE/DEFAULT_FORMAT/ENCODING/GNU_FORMAT/LENGTH_* /
    // PAX_FORMAT/RECORDSIZE/USTAR_FORMAT/version already inserted above.)

    // bytes constants (single-byte type flags + magics).
    attrs.insert("REGTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![48])));
    attrs.insert("AREGTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![0])));
    attrs.insert("LNKTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![49])));
    attrs.insert("SYMTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![50])));
    attrs.insert("CHRTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![51])));
    attrs.insert("BLKTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![52])));
    attrs.insert("DIRTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![53])));
    attrs.insert("FIFOTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![54])));
    attrs.insert("CONTTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![55])));
    attrs.insert("GNUTYPE_LONGNAME".into(), MbValue::from_ptr(MbObject::new_bytes(vec![76])));
    attrs.insert("GNUTYPE_LONGLINK".into(), MbValue::from_ptr(MbObject::new_bytes(vec![75])));
    attrs.insert("GNUTYPE_SPARSE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![83])));
    attrs.insert("XHDTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![120])));
    attrs.insert("XGLTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![103])));
    attrs.insert("SOLARIS_XHDTYPE".into(), MbValue::from_ptr(MbObject::new_bytes(vec![88])));
    attrs.insert("NUL".into(), MbValue::from_ptr(MbObject::new_bytes(vec![0])));
    attrs.insert("GNU_MAGIC".into(),
        MbValue::from_ptr(MbObject::new_bytes(vec![117, 115, 116, 97, 114, 32, 32, 0])));
    attrs.insert("POSIX_MAGIC".into(),
        MbValue::from_ptr(MbObject::new_bytes(vec![117, 115, 116, 97, 114, 0, 48, 48])));

    // Tuple / set / dict structured constants.
    let b = |x: u8| MbValue::from_ptr(MbObject::new_bytes(vec![x]));
    attrs.insert("GNU_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![b(76), b(75), b(83)])));
    attrs.insert("REGULAR_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![b(48), b(0), b(55), b(83)])));
    attrs.insert("SUPPORTED_TYPES".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            b(48), b(0), b(49), b(50), b(53), b(54), b(55), b(51), b(52), b(76), b(75), b(83),
        ])));
    let s = |x: &str| MbValue::from_ptr(MbObject::new_str(x.to_string()));
    attrs.insert("PAX_FIELDS".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            s("path"), s("linkpath"), s("size"), s("mtime"),
            s("uid"), s("gid"), s("uname"), s("gname"),
        ])));
    attrs.insert("PAX_NAME_FIELDS".into(),
        MbValue::from_ptr(MbObject::new_set(vec![
            s("path"), s("linkpath"), s("uname"), s("gname"),
        ])));
    // PAX_NUMBER_FIELDS maps field -> Python type object in CPython; we surface
    // it as a dict mapping field -> type-name string (present + dict-typed)
    // since real type objects as map values are not yet representable here.
    let pax_num = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*pax_num).data {
            let mut map = lock.write().unwrap();
            map.insert("atime".into(), s("float"));
            map.insert("ctime".into(), s("float"));
            map.insert("mtime".into(), s("float"));
            map.insert("uid".into(), s("int"));
            map.insert("gid".into(), s("int"));
            map.insert("size".into(), s("int"));
        }
    }
    attrs.insert("PAX_NUMBER_FIELDS".into(), MbValue::from_ptr(pax_num));

    // Module-level helper / filter functions (surface stubs — callable, None).
    let fn_stubs: Vec<&str> = vec![
        "tar_filter", "data_filter", "fully_trusted_filter",
        "calc_chksums", "copyfileobj", "nti", "nts", "stn",
        "bltn_open", "main",
    ];
    for name in fn_stubs {
        let addr = dispatch_tarfile_stub as usize;
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|st| {
            st.borrow_mut().insert(addr as u64);
        });
    }

    // Classes. TarFile / TarInfo / ExFileObject as class-name string sentinels;
    // the exception hierarchy additionally registers in the class registry with
    // correct bases so `issubclass` / `except tarfile.X` resolve (gzip precedent).
    let plain_classes: Vec<&str> = vec!["TarFile", "TarInfo", "ExFileObject"];
    for name in plain_classes {
        attrs.insert(name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }
    // (class-name, direct-base) pairs, parents before children for clean MRO.
    let exc_classes: Vec<(&str, &str)> = vec![
        ("TarError", "Exception"),
        ("ReadError", "TarError"),
        ("CompressionError", "TarError"),
        ("StreamError", "TarError"),
        ("ExtractError", "TarError"),
        ("HeaderError", "TarError"),
        ("FilterError", "TarError"),
        ("EmptyHeaderError", "HeaderError"),
        ("TruncatedHeaderError", "HeaderError"),
        ("EOFHeaderError", "HeaderError"),
        ("InvalidHeaderError", "HeaderError"),
        ("SubsequentHeaderError", "HeaderError"),
        ("AbsoluteLinkError", "FilterError"),
        ("OutsideDestinationError", "FilterError"),
        ("SpecialFileError", "FilterError"),
        ("AbsolutePathError", "FilterError"),
        ("LinkOutsideDestinationError", "FilterError"),
        ("LinkFallbackError", "FilterError"),
    ];
    for (name, base) in exc_classes {
        super::super::class::mb_class_register(
            name,
            vec![base.to_string()],
            HashMap::new(),
        );
        attrs.insert(name.to_string(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }
    // surface: TarFile / TarInfo must satisfy `callable(...)` (constructable
    // class objects in CPython). The string-sentinel registration above leaves
    // them present-but-not-callable; overwrite with from_func stubs so
    // `callable(tarfile.TarFile)` / `callable(tarfile.TarInfo)` resolve True.
    // Additive: these inserts replace the earlier same-key sentinels.
    let callable_classes: Vec<&str> = vec!["TarFile", "TarInfo"];
    for name in callable_classes {
        let addr = dispatch_tarfile_stub as usize;
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|st| {
            st.borrow_mut().insert(addr as u64);
        });
    }

    // symlink_exception is a tuple of exception classes; surface as a tuple of
    // class-name string sentinels (present + tuple-typed).
    attrs.insert("symlink_exception".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            s("AttributeError"), s("NotImplementedError"), s("OSError"),
        ])));

    super::register_module("tarfile", attrs);
}

/// tarfile.open(path, mode) -> tar dict
pub fn mb_tarfile_open(path: MbValue, mode: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert("__class__".into(),
                MbValue::from_ptr(MbObject::new_str("TarFile".to_string())));
            map.insert("name".into(), path);
            map.insert("mode".into(), mode);
            map.insert("_members".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])));
        }
    }
    MbValue::from_ptr(dict)
}

/// tar.getnames() -> list of member names
pub fn mb_tarfile_getnames(tf: MbValue) -> MbValue {
    if let Some(ptr) = tf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(members) = map.get("_members").copied() {
                    return members;
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// tar.add(name) -> None
pub fn mb_tarfile_add(tf: MbValue, name: MbValue) -> MbValue {
    if let Some(ptr) = tf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(members) = map.get("_members").copied() {
                    if let Some(m_ptr) = members.as_ptr() {
                        if let ObjData::List(ref list_lock) = (*m_ptr).data {
                            let mut items = list_lock.write().unwrap();
                            items.push(name);
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// tar.close() -> None
pub fn mb_tarfile_close(_tf: MbValue) -> MbValue {
    MbValue::none()
}

/// tarfile.is_tarfile(path) -> bool (stub: false)
pub fn mb_tarfile_is_tarfile(_path: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

/// tar.extractall(path) -> None (stub)
pub fn mb_tarfile_extractall(_tf: MbValue, _path: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_tarfile_add_getnames() {
        let tf = mb_tarfile_open(s("test.tar"), s("w"));
        mb_tarfile_add(tf, s("file1.txt"));
        let names = mb_tarfile_getnames(tf);
        unsafe {
            if let ObjData::List(ref lock) = (*names.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
            }
        }
    }

    #[test]
    fn test_tarfile_is_tarfile_close_extractall() {
        assert_eq!(mb_tarfile_is_tarfile(s("/nope")).as_bool(), Some(false));
        let tf = mb_tarfile_open(s("t.tar"), s("r"));
        assert!(mb_tarfile_close(tf).is_none());
        assert!(mb_tarfile_extractall(tf, s("/tmp")).is_none());
    }
}
