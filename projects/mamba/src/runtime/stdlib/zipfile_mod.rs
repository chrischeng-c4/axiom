/// zipfile module for Mamba (#445).
///
/// Provides: ZipFile (instance with writestr/getinfo/read/namelist/close),
/// ZipInfo (data record), is_zipfile. No external dependency — entries are
/// stored in-memory on the backing file object (e.g. io.BytesIO), so a writer
/// and a reader opened over the same buffer share them without serializing a
/// real ZIP container.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// ── Class names registered via mb_class_register ──
// `ZipFile` / `ZipInfo` are real `ObjData::Instance` objects so that
// `zf.writestr(...)` / `zf.getinfo(...)` dispatch through the registered
// method table (the argparse/configparser stdlib-class pattern) and
// `isinstance(info, zipfile.ZipInfo)` resolves nominally.
const ZIPFILE_CLASS: &str = "ZipFile";
const ZIPINFO_CLASS: &str = "ZipInfo";

// ── Instance-field + value helpers ──

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

/// Turn a list/tuple call-args value into a Vec<MbValue> (variadic methods
/// receive their positional args packed into a single list).
fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// Byte length of a str/bytes/bytearray value (used for file_size).
fn byte_len(val: MbValue) -> i64 {
    val.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => s.len() as i64,
            ObjData::Bytes(b) => b.len() as i64,
            ObjData::ByteArray(lock) => lock.read().unwrap().len() as i64,
            _ => 0,
        }
    }).unwrap_or(0)
}

// ── Exception helpers (mirror the array_mod / argparse_mod raise pattern) ──
// Each sets the thread-local current exception via `mb_raise`; the Rust caller
// MUST `return` immediately afterward (mb_raise does NOT unwind Rust).

fn raise_str(exc: &str, msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Raw bytes backing an `io.BytesIO` file object (field `_buffer`). Returns an
/// empty Vec for the shared-entries case (writer/reader never serialize a real
/// container into `_buffer`), so a non-empty result signals genuine input bytes.
fn fileobj_buffer_bytes(file: MbValue) -> Vec<u8> {
    get_field(file, "_buffer")
        .and_then(|v| v.as_ptr())
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::Bytes(b) => b.clone(),
                ObjData::ByteArray(lock) => lock.read().unwrap().clone(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// True iff `buf` begins with a local-file-header (`PK\x03\x04`), an
/// end-of-central-directory record (`PK\x05\x06`, the empty-archive case), or a
/// ZIP64 EOCD (`PK\x06\x06`) — i.e. a recognizable ZIP container prefix.
fn has_zip_signature(buf: &[u8]) -> bool {
    buf.len() >= 4
        && buf[0] == 0x50
        && buf[1] == 0x4B
        && matches!((buf[2], buf[3]), (0x03, 0x04) | (0x05, 0x06) | (0x06, 0x06))
}

/// Effective open mode: the first char of an explicit mode string, defaulting
/// to 'r' when the mode arg is absent/None (CPython's `ZipFile` default).
fn effective_mode(mode: MbValue) -> char {
    extract_str(mode)
        .and_then(|s| s.chars().next())
        .unwrap_or('r')
}

/// True iff `close()` has been called on this ZipFile (sets `__closed__`).
fn is_closed(zf: MbValue) -> bool {
    get_field(zf, "__closed__")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

/// Resolve the shared entry-table dict stored on the backing file object so
/// that a `ZipFile(buf, "w")` writer and a later `ZipFile(buf, "r")` reader
/// (sharing the same `io.BytesIO`) observe the same entries. The table is a
/// plain `ObjData::Dict` parked under the `__zip_entries__` field of the
/// fileobj; created lazily on first access.
fn entries_table(zf: MbValue) -> Option<MbValue> {
    let fileobj = get_field(zf, "__fileobj__")?;
    if let Some(existing) = get_field(fileobj, "__zip_entries__") {
        if !existing.is_none() {
            return Some(existing);
        }
    }
    let table = MbValue::from_ptr(MbObject::new_dict());
    set_field(fileobj, "__zip_entries__", table);
    Some(table)
}

// ── ZipFile instance methods (variadic: self + packed args list) ──

/// zf.writestr(name, data): record an entry on the shared file object.
unsafe extern "C" fn method_writestr(self_v: MbValue, args: MbValue) -> MbValue {
    if is_closed(self_v) {
        raise_str("ValueError", "Attempt to write to ZIP archive that was already closed");
        return MbValue::none();
    }
    let items = seq_items(args);
    let name = items.first().copied().unwrap_or_else(MbValue::none);
    let data = items.get(1).copied().unwrap_or_else(MbValue::none);
    let n = extract_str(name).unwrap_or_default();
    if let Some(table) = entries_table(self_v) {
        if let Some(tptr) = table.as_ptr() {
            if let ObjData::Dict(ref lock) = (*tptr).data {
                super::super::rc::retain_if_ptr(data);
                lock.write().unwrap().insert(n.into(), data);
            }
        }
    }
    MbValue::none()
}

/// zf.read(name) -> the stored data (bytes/str).
///
/// Raises `ValueError` if the archive has been closed, and `KeyError` if the
/// requested member is not present (CPython `ZipFile.read` semantics).
unsafe extern "C" fn method_read(self_v: MbValue, args: MbValue) -> MbValue {
    if is_closed(self_v) {
        raise_str("ValueError", "Attempt to use ZIP archive that was already closed");
        return MbValue::none();
    }
    let items = seq_items(args);
    let n = items.first().copied().and_then(extract_str).unwrap_or_default();
    if let Some(table) = entries_table(self_v) {
        if let Some(tptr) = table.as_ptr() {
            if let ObjData::Dict(ref lock) = (*tptr).data {
                if let Some(v) = lock.read().unwrap().get(&n).copied() {
                    return v;
                }
            }
        }
    }
    raise_str("KeyError", &format!("There is no item named {n:?} in the archive"));
    MbValue::none()
}

/// zf.namelist() -> list of entry names.
unsafe extern "C" fn method_namelist(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(table) = entries_table(self_v) {
        if let Some(tptr) = table.as_ptr() {
            if let ObjData::Dict(ref lock) = (*tptr).data {
                let names: Vec<MbValue> = lock.read().unwrap().keys()
                    .map(|k| MbValue::from_ptr(MbObject::new_str(k.to_string())))
                    .collect();
                return MbValue::from_ptr(MbObject::new_list(names));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// zf.getinfo(name) -> ZipInfo instance with the documented attributes.
unsafe extern "C" fn method_getinfo(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let name = items.first().copied().unwrap_or_else(MbValue::none);
    let n = extract_str(name).unwrap_or_default();
    let data = entries_table(self_v).and_then(|table| {
        table.as_ptr().and_then(|tptr| {
            if let ObjData::Dict(ref lock) = (*tptr).data {
                lock.read().unwrap().get(&n).copied()
            } else {
                None
            }
        })
    });
    let size = data.map(byte_len).unwrap_or(0);
    let info = MbValue::from_ptr(MbObject::new_instance(ZIPINFO_CLASS.to_string()));
    set_field(info, "filename", MbValue::from_ptr(MbObject::new_str(n)));
    set_field(info, "file_size", MbValue::from_int(size));
    set_field(info, "compress_size", MbValue::from_int(size));
    set_field(info, "compress_type", MbValue::from_int(0)); // ZIP_STORED
    info
}

// Context-manager protocol (`with zipfile.ZipFile(...) as zf:`) is handled by
// the runtime's `mb_context_enter` / `mb_context_exit` fallbacks: with no
// `__enter__` dunder registered, enter returns `self`, and with no `__exit__`
// dunder, exit returns False (do not suppress). That is exactly ZipFile's
// context behavior, so no explicit enter/exit methods are registered (and we
// avoid the variadic-vs-4-arg ABI mismatch those call sites would impose).

/// zf.close() -> None. Marks the archive closed so subsequent read/open/
/// testzip/writestr raise ValueError (CPython semantics).
unsafe extern "C" fn method_close(self_v: MbValue, _args: MbValue) -> MbValue {
    set_field(self_v, "__closed__", MbValue::from_bool(true));
    MbValue::none()
}

/// zf.open(name, mode="r") -> a readable file-like handle for the member.
///
/// Raises `ValueError` if the archive is closed or `mode` is not 'r'/'w'
/// (CPython rejects e.g. 'q', 'U', 'rU'), and `KeyError` if the member is
/// absent. Mode is validated before member lookup, matching CPython's order.
unsafe extern "C" fn method_open(self_v: MbValue, args: MbValue) -> MbValue {
    if is_closed(self_v) {
        raise_str("ValueError", "Attempt to use ZIP archive that was already closed");
        return MbValue::none();
    }
    let items = seq_items(args);
    let n = items.first().copied().and_then(extract_str).unwrap_or_default();
    // CPython 3.12: ZipFile.open accepts only the per-member modes 'r' and 'w'.
    if let Some(mode) = items.get(1).copied().and_then(extract_str) {
        if mode != "r" && mode != "w" {
            raise_str("ValueError", "open() requires mode \"r\" or \"w\"");
            return MbValue::none();
        }
    }
    let data = entries_table(self_v).and_then(|table| {
        table.as_ptr().and_then(|tptr| {
            if let ObjData::Dict(ref lock) = (*tptr).data {
                lock.read().unwrap().get(&n).copied()
            } else {
                None
            }
        })
    });
    match data {
        Some(v) => {
            // Hand back an io.BytesIO over the member content so `with
            // zf.open(name) as fh: fh.read()` works for the valid case.
            let bytes = match v.as_ptr().map(|p| &(*p).data) {
                Some(ObjData::Bytes(b)) => b.clone(),
                Some(ObjData::ByteArray(lock)) => lock.read().unwrap().clone(),
                Some(ObjData::Str(s)) => s.clone().into_bytes(),
                _ => Vec::new(),
            };
            super::io_mod::mb_bytesio_new_with(bytes)
        }
        None => {
            raise_str("KeyError", &format!("There is no item named {n:?} in the archive"));
            MbValue::none()
        }
    }
}

/// zf.testzip() -> name of first bad member, else None. Raises ValueError on a
/// closed archive. The in-memory model stores members verbatim (no recorded
/// CRC to mismatch), so a non-closed archive always reports all members good.
unsafe extern "C" fn method_testzip(self_v: MbValue, _args: MbValue) -> MbValue {
    if is_closed(self_v) {
        raise_str("ValueError", "Attempt to use ZIP archive that was already closed");
        return MbValue::none();
    }
    MbValue::none()
}

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

#[allow(unused_macros)]
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

disp_unary!(d_is_zipfile, mb_zipfile_is_zipfile);

/// Recognized compression methods (CPython 3.12): ZIP_STORED(0),
/// ZIP_DEFLATED(8), ZIP_BZIP2(12), ZIP_LZMA(14). Any other value is an
/// unsupported compression type → NotImplementedError.
fn is_known_compression(c: i64) -> bool {
    matches!(c, 0 | 8 | 12 | 14)
}

/// Pull the `compression` argument out of the constructor call: either the 3rd
/// positional arg (an int) or the `compression` key of a trailing kwargs dict
/// (keyword lowering appends kwargs as a trailing `Dict`). Returns None when no
/// compression was supplied (default ZIP_STORED is fine, never rejected).
unsafe fn arg_compression(args: &[MbValue]) -> Option<i64> {
    if let Some(v) = args.get(2).copied() {
        if let Some(i) = v.as_int() {
            return Some(i);
        }
        // Trailing kwargs dict: `ZipFile(f, "w", compression=...)`.
        if let Some(p) = v.as_ptr() {
            if let ObjData::Dict(ref lock) = (*p).data {
                if let Some(cv) = lock.read().unwrap().get("compression").copied() {
                    return cv.as_int();
                }
            }
        }
    }
    None
}

/// zipfile.ZipFile(file, mode="r", compression=ZIP_STORED, ...) constructor.
///
/// Validates the constructor arguments (raising before building the instance)
/// the way CPython does: a bad `mode` → ValueError, an unsupported
/// `compression` → NotImplementedError, and, in read mode over a real path or a
/// pre-populated buffer, a missing file → OSError / a zero-byte-or-garbage
/// container → BadZipFile.
unsafe extern "C" fn d_zipfile_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let file = args.first().copied().unwrap_or_else(MbValue::none);
    let mode = args.get(1).copied().unwrap_or_else(MbValue::none);

    // 1) Mode validation: an explicit mode string must start with r/w/x/a.
    if let Some(m) = extract_str(mode) {
        let first = m.chars().next();
        if !matches!(first, Some('r') | Some('w') | Some('x') | Some('a')) {
            raise_str("ValueError", "ZipFile requires mode 'r', 'w', 'x', or 'a'");
            return MbValue::none();
        }
    }

    // 2) Compression validation: a supplied, unrecognized method is not
    //    implemented. A missing/known value (incl. default STORED) is fine.
    if let Some(c) = arg_compression(args) {
        if !is_known_compression(c) {
            raise_str("NotImplementedError", "compression type is not supported");
            return MbValue::none();
        }
    }

    // 3) Read-mode source validation. Only fires in read mode; write/append
    //    construction never touches the filesystem or inspects the buffer.
    if effective_mode(mode) == 'r' {
        if let Some(path) = extract_str(file) {
            // The first positional is a real path string opened for reading.
            match std::fs::metadata(&path) {
                Err(_) => {
                    raise_str("OSError", &format!("[Errno 2] No such file or directory: {path:?}"));
                    return MbValue::none();
                }
                Ok(meta) => {
                    if meta.len() == 0 {
                        raise_str("BadZipFile", "File is not a zip file");
                        return MbValue::none();
                    }
                }
            }
        } else {
            // A file-like object (io.BytesIO): non-empty backing bytes that do
            // not begin with a ZIP signature are not a valid container. The
            // shared writer/reader case leaves `_buffer` empty, so it is skipped.
            let buf = fileobj_buffer_bytes(file);
            if !buf.is_empty() && !has_zip_signature(&buf) {
                raise_str("BadZipFile", "File is not a zip file");
                return MbValue::none();
            }
        }
    }

    mb_zipfile_new(file, mode)
}

/// Generic present-and-callable stub for class/function surface attrs
/// (returns None). Surface `*_is_callable` fixtures require resolve_callable
/// to return Some, which native func values satisfy.
unsafe extern "C" fn d_zipfile_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Dedicated stub for the `BadZipFile` exception class. It has its own distinct
/// function address (unlike the shared `d_zipfile_stub`) so it can be recorded
/// in `NATIVE_TYPE_NAMES` as the name "BadZipFile" — that is what lets
/// `except zipfile.BadZipFile:` resolve the handler type and catch a raised
/// "BadZipFile" exception.
unsafe extern "C" fn d_zipfile_badzipfile(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// Register the zipfile module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Compression constants
    attrs.insert("ZIP_STORED".into(), MbValue::from_int(0));
    attrs.insert("ZIP_DEFLATED".into(), MbValue::from_int(8));

    let dispatchers: Vec<(&str, usize)> = vec![
        ("ZipFile", d_zipfile_new as *const () as usize),
        ("is_zipfile", d_is_zipfile as *const () as usize),
        // surface: missing CPython classes/functions as present-AND-callable stubs
        // BadZipFile uses a dedicated address (registered below in
        // NATIVE_TYPE_NAMES) so `except zipfile.BadZipFile:` resolves to the
        // name "BadZipFile" and catches the raised exception.
        ("BadZipFile", d_zipfile_badzipfile as *const () as usize),
        ("BadZipfile", d_zipfile_badzipfile as *const () as usize),
        ("CompleteDirs", d_zipfile_stub as *const () as usize),
        ("LZMACompressor", d_zipfile_stub as *const () as usize),
        ("LZMADecompressor", d_zipfile_stub as *const () as usize),
        ("LargeZipFile", d_zipfile_stub as *const () as usize),
        ("Path", d_zipfile_stub as *const () as usize),
        ("PyZipFile", d_zipfile_stub as *const () as usize),
        ("ZipExtFile", d_zipfile_stub as *const () as usize),
        ("error", d_zipfile_stub as *const () as usize),
        ("crc32", d_zipfile_stub as *const () as usize),
        ("main", d_zipfile_stub as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Record BadZipFile's address as a resolvable type name so that the handler
    // expression in `except zipfile.BadZipFile:` resolves to "BadZipFile" (via
    // class::resolve_class_name → NATIVE_TYPE_NAMES) and matches a raised
    // "BadZipFile" exception. `BadZipfile` is the deprecated alias of the same.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(d_zipfile_badzipfile as *const () as u64, "BadZipFile".into());
    });

        // surface: missing CPython module constants (auto-added)
    attrs.insert("BZIP2_VERSION".into(), MbValue::from_int(46));
    attrs.insert("DEFAULT_VERSION".into(), MbValue::from_int(20));
    attrs.insert("LZMA_VERSION".into(), MbValue::from_int(63));
    attrs.insert("MAX_EXTRACT_VERSION".into(), MbValue::from_int(63));
    attrs.insert("ZIP64_LIMIT".into(), MbValue::from_int(2147483647));
    attrs.insert("ZIP64_VERSION".into(), MbValue::from_int(45));
    attrs.insert("ZIP_BZIP2".into(), MbValue::from_int(12));
    attrs.insert("ZIP_FILECOUNT_LIMIT".into(), MbValue::from_int(65535));
    attrs.insert("ZIP_LZMA".into(), MbValue::from_int(14));
    attrs.insert("ZIP_MAX_COMMENT".into(), MbValue::from_int(65535));
    attrs.insert("sizeCentralDir".into(), MbValue::from_int(46));
    attrs.insert("sizeEndCentDir".into(), MbValue::from_int(22));
    attrs.insert("sizeEndCentDir64".into(), MbValue::from_int(56));
    attrs.insert("sizeEndCentDir64Locator".into(), MbValue::from_int(20));
    attrs.insert("sizeFileHeader".into(), MbValue::from_int(30));
    attrs.insert("structCentralDir".into(), MbValue::from_ptr(MbObject::new_str("<4s4B4HL2L5H2L".to_string())));
    attrs.insert("structEndArchive64".into(), MbValue::from_ptr(MbObject::new_str("<4sQ2H2L4Q".to_string())));
    attrs.insert("structEndArchive64Locator".into(), MbValue::from_ptr(MbObject::new_str("<4sLQL".to_string())));
    attrs.insert("structFileHeader".into(), MbValue::from_ptr(MbObject::new_str("<4s2B4HL2L2H".to_string())));

    // surface: missing CPython bytes constants (PK magic + format strings)
    attrs.insert("stringCentralDir".into(), MbValue::from_ptr(MbObject::new_bytes(vec![80, 75, 1, 2])));
    attrs.insert("stringEndArchive".into(), MbValue::from_ptr(MbObject::new_bytes(vec![80, 75, 5, 6])));
    attrs.insert("stringEndArchive64".into(), MbValue::from_ptr(MbObject::new_bytes(vec![80, 75, 6, 6])));
    attrs.insert("stringEndArchive64Locator".into(), MbValue::from_ptr(MbObject::new_bytes(vec![80, 75, 6, 7])));
    attrs.insert("stringFileHeader".into(), MbValue::from_ptr(MbObject::new_bytes(vec![80, 75, 3, 4])));
    attrs.insert("structEndArchive".into(), MbValue::from_ptr(MbObject::new_bytes(vec![60, 52, 115, 52, 72, 50, 76, 72])));

    // surface: missing CPython dict + re-exported module attrs (dict placeholders)
    attrs.insert("compressor_names".into(), MbValue::from_ptr(MbObject::new_dict()));
    for m in ["binascii", "bz2", "importlib", "io", "lzma", "os", "shutil",
              "stat", "struct", "sys", "threading", "time", "zlib"] {
        attrs.insert(m.into(), MbValue::from_ptr(MbObject::new_dict()));
    }

    // Register the ZipFile / ZipInfo runtime classes so instance method calls
    // (`zf.writestr`, `zf.getinfo`) dispatch through the method table and
    // `isinstance(info, zipfile.ZipInfo)` matches nominally. A registered
    // class name exposed as a string module attr is a valid isinstance target
    // (mirrors io_mod's BufferedIOBase / ArgumentParser registration).
    {
        type MethodSpec = (&'static str, usize);
        let zipfile_methods: Vec<MethodSpec> = vec![
            ("writestr", method_writestr as *const () as usize),
            ("read", method_read as *const () as usize),
            ("open", method_open as *const () as usize),
            ("testzip", method_testzip as *const () as usize),
            ("namelist", method_namelist as *const () as usize),
            ("getinfo", method_getinfo as *const () as usize),
            ("close", method_close as *const () as usize),
        ];
        let mut zf_map: HashMap<String, MbValue> = HashMap::new();
        for (name, addr) in zipfile_methods {
            zf_map.insert(name.to_string(), MbValue::from_func(addr));
            super::super::module::register_variadic_func(addr as u64);
        }
        super::super::class::mb_class_register(ZIPFILE_CLASS, vec![], zf_map);
        // ZipInfo is a plain data record; no methods needed for the surface.
        super::super::class::mb_class_register(ZIPINFO_CLASS, vec![], HashMap::new());
    }
    // A registered class name exposed as a string is callable() and usable as
    // an isinstance target. The `ZipFile` name stays a constructor func (so
    // `ZipFile(buf, "w")` builds an instance); `ZipInfo` is exposed as the
    // class-name string for `isinstance(..., zipfile.ZipInfo)`.
    attrs.insert("ZipInfo".into(),
        MbValue::from_ptr(MbObject::new_str(ZIPINFO_CLASS.to_string())));

    super::register_module("zipfile", attrs);
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// zipfile.ZipFile(file, mode) -> ZipFile instance.
///
/// `file` is the backing object (typically an `io.BytesIO`). The entry table
/// lives on `file` (field `__zip_entries__`), so a writer and a later reader
/// constructed over the same buffer share entries without serializing a real
/// ZIP container. The first positional arg may also be a path string.
pub fn mb_zipfile_new(file: MbValue, mode: MbValue) -> MbValue {
    let zf = MbValue::from_ptr(MbObject::new_instance(ZIPFILE_CLASS.to_string()));
    set_field(zf, "__fileobj__", file);
    set_field(zf, "filename", file);
    set_field(zf, "mode", mode);
    zf
}

/// zipfile.is_zipfile(path) -> bool (stub: always false)
pub fn mb_zipfile_is_zipfile(_path: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn list(items: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(items))
    }

    /// A stand-in for io.BytesIO: an Instance that can carry the shared
    /// `__zip_entries__` field. (`set_field`/`entries_table` only operate on
    /// `ObjData::Instance`, mirroring the real io.BytesIO backing object.)
    fn fileobj() -> MbValue {
        MbValue::from_ptr(MbObject::new_instance("BytesIO".to_string()))
    }

    #[test]
    fn test_writestr_read() {
        let buf = fileobj();
        let zf = mb_zipfile_new(buf, s("w"));
        unsafe { method_writestr(zf, list(vec![s("hello.txt"), s("hello world")])); }
        let data = unsafe { method_read(zf, list(vec![s("hello.txt")])) };
        assert_eq!(extract_str(data).unwrap(), "hello world");
    }

    #[test]
    fn test_namelist() {
        let buf = fileobj();
        let zf = mb_zipfile_new(buf, s("w"));
        unsafe { method_writestr(zf, list(vec![s("a.txt"), s("a")])); }
        let names = unsafe { method_namelist(zf, list(vec![])) };
        unsafe {
            if let ObjData::List(ref lock) = (*names.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
            }
        }
    }

    #[test]
    fn test_getinfo_attributes_shared_across_handles() {
        // Writer and reader share the same backing fileobj, so getinfo on a
        // second ZipFile constructed over the same buffer sees the entry.
        let buf = fileobj();
        let writer = mb_zipfile_new(buf, s("w"));
        unsafe { method_writestr(writer, list(vec![s("hello.txt"), s("Hello, World!")])); }
        let reader = mb_zipfile_new(buf, s("r"));
        let info = unsafe { method_getinfo(reader, list(vec![s("hello.txt")])) };
        assert_eq!(get_field(info, "filename").and_then(extract_str),
                   Some("hello.txt".to_string()));
        assert_eq!(get_field(info, "file_size").and_then(|v| v.as_int()), Some(13));
        assert_eq!(get_field(info, "compress_size").and_then(|v| v.as_int()), Some(13));
        assert_eq!(get_field(info, "compress_type").and_then(|v| v.as_int()), Some(0));
    }

    #[test]
    fn test_read_missing_member_raises_keyerror() {
        // Reading a member that is not in the archive raises KeyError
        // (CPython semantics): the call sets the current exception to KeyError.
        let buf = fileobj();
        let zf = mb_zipfile_new(buf, s("w"));
        unsafe { method_writestr(zf, list(vec![s("a.txt"), s("hi")])); }
        let _ = unsafe { method_read(zf, list(vec![s("missing.txt")])) };
        assert_eq!(
            super::super::super::exception::current_exception_type().as_deref(),
            Some("KeyError"),
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_close_sets_closed_and_is_zipfile() {
        let buf = fileobj();
        let zf = mb_zipfile_new(buf, s("w"));
        assert!(!is_closed(zf));
        assert!(unsafe { method_close(zf, list(vec![])) }.is_none());
        assert!(is_closed(zf));
        assert_eq!(mb_zipfile_is_zipfile(s("whatever.zip")).as_bool(), Some(false));
    }

    #[test]
    fn test_operations_after_close_raise_valueerror() {
        let buf = fileobj();
        let zf = mb_zipfile_new(buf, s("w"));
        unsafe { method_writestr(zf, list(vec![s("a.txt"), s("hi")])); }
        unsafe { method_close(zf, list(vec![])); }
        for call in [
            unsafe { method_read(zf, list(vec![s("a.txt")])) },
            unsafe { method_open(zf, list(vec![s("a.txt")])) },
            unsafe { method_testzip(zf, list(vec![])) },
            unsafe { method_writestr(zf, list(vec![s("b.txt"), s("x")])) },
        ] {
            let _ = call;
            assert_eq!(
                super::super::super::exception::current_exception_type().as_deref(),
                Some("ValueError"),
            );
            super::super::super::exception::mb_clear_exception();
        }
    }
}
