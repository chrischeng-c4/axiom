use super::super::rc::{MbObject, ObjData};
use super::super::rc::{MbObjectHeader, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// io module for Mamba (#415).
///
/// Provides: StringIO and BytesIO in-memory stream objects.
/// StringIO stores text, BytesIO stores raw bytes.
///
/// The constructors return `ObjData::Instance` with class_name
/// "StringIO" / "BytesIO"; method calls (`write`, `read`, `getvalue`,
/// `seek`, `tell`, `close`) are routed through dispatch arms in
/// `class.rs::mb_call_method` — the same pattern used for
/// threading.Lock / Event / Condition.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn raise(kind: &str, msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str(kind), new_str(msg.into()));
    MbValue::none()
}

/// Short Python type name for error messages.
fn type_name_of(val: MbValue) -> &'static str {
    if val.is_bool() {
        return "bool";
    }
    if val.is_int() {
        return "int";
    }
    if val.is_float() {
        return "float";
    }
    if val.is_none() {
        return "NoneType";
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match (*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                _ => "object",
            };
        }
    }
    "object"
}

/// True iff `val` is a `str` object (used to TypeError on str→BytesIO.write).
fn is_str(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
        .unwrap_or(false)
}

/// True iff `val` is a bytes-like object (bytes / bytearray).
fn is_bytes_like(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

/// True iff this in-memory stream instance has been closed.
fn is_closed(io: MbValue) -> bool {
    if let Some(ptr) = io.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                return fields
                    .read()
                    .unwrap()
                    .get("_closed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
            }
        }
    }
    false
}

/// Raise ValueError("I/O operation on closed file") and return true when closed.
fn check_closed(io: MbValue) -> bool {
    if is_closed(io) {
        raise("ValueError", "I/O operation on closed file");
        return true;
    }
    false
}

fn field_set(io: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = io.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(key.into(), val);
            }
        }
    }
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
            ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            _ => None,
        }
    })
}

fn make_instance(class_name: &str, fields: FxHashMap<String, MbValue>) -> MbValue {
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

fn trailing_kwargs(
    a: &[MbValue],
) -> Option<indexmap::IndexMap<super::super::dict_ops::DictKey, MbValue>> {
    a.last().and_then(|v| v.as_ptr()).and_then(|p| unsafe {
        if let ObjData::Dict(ref lock) = (*p).data {
            Some(lock.read().unwrap().clone())
        } else {
            None
        }
    })
}

fn kwarg_str(
    kw: &indexmap::IndexMap<super::super::dict_ops::DictKey, MbValue>,
    key: &str,
) -> Option<String> {
    for (k, v) in kw.iter() {
        if let super::super::dict_ops::DictKey::Str(ref ks) = k {
            if ks == key {
                return extract_str(*v);
            }
        }
    }
    None
}

fn kwarg_int(
    kw: &indexmap::IndexMap<super::super::dict_ops::DictKey, MbValue>,
    key: &str,
) -> Option<i64> {
    for (k, v) in kw.iter() {
        if let super::super::dict_ops::DictKey::Str(ref ks) = k {
            if ks == key {
                return v.as_int();
            }
        }
    }
    None
}

unsafe extern "C" fn dispatch_stringio_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let initial = if nargs > 0 {
        extract_str(unsafe { *args_ptr }).unwrap_or_default()
    } else {
        String::new()
    };
    mb_stringio_new_with(initial)
}

unsafe extern "C" fn dispatch_bytesio_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let initial = if nargs > 0 {
        extract_bytes(unsafe { *args_ptr }).unwrap_or_default()
    } else {
        Vec::new()
    };
    mb_bytesio_new_with(initial)
}

unsafe extern "C" fn dispatch_textiowrapper_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    let underlying = if positional_end >= 1 {
        a[0]
    } else {
        MbValue::none()
    };
    let mut encoding = if positional_end >= 2 {
        extract_str(a[1]).unwrap_or_else(|| "utf-8".into())
    } else {
        "utf-8".into()
    };
    let mut newline: Option<String> = None;
    let mut write_through = false;
    if let Some(ref m) = kw {
        if let Some(e) = kwarg_str(m, "encoding") {
            encoding = e;
        }
        if let Some(n) = kwarg_str(m, "newline") {
            newline = Some(n);
        }
        for (k, v) in m.iter() {
            if let super::super::dict_ops::DictKey::Str(ref ks) = k {
                if ks == "write_through" {
                    write_through = !v.is_none()
                        && !matches!(v.as_int(), Some(0))
                        && !matches!(v.as_bool(), Some(false));
                }
            }
        }
    }
    mb_textiowrapper_new(underlying, encoding, newline, write_through)
}

unsafe extern "C" fn dispatch_bufferedreader_new(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    let underlying = if positional_end >= 1 {
        a[0]
    } else {
        MbValue::none()
    };
    let mut buffer_size: i64 = 8192;
    if positional_end >= 2 {
        if let Some(n) = a[1].as_int() {
            buffer_size = n;
        }
    }
    if let Some(ref m) = kw {
        if let Some(n) = kwarg_int(m, "buffer_size") {
            buffer_size = n;
        }
    }
    mb_bufferedreader_new(underlying, buffer_size)
}

/// Register a native io abstract base class with the class registry so that
/// `isinstance(x, io.SomeBase)` resolves through the MRO. `bases` must already
/// be registered (call in inheritance order, base → derived).
fn register_io_class(name: &str, bases: &[&str], methods: Vec<(&str, *const ())>) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (mname, addr) in methods {
        map.insert(mname.to_string(), MbValue::from_func(addr as usize));
    }
    let base_vec: Vec<String> = bases.iter().map(|b| b.to_string()).collect();
    super::super::class::mb_class_register(name, base_vec, map);
}

/// Register the io module.
pub fn register() {
    // ── Class hierarchy (CPython io ABC tree) ─────────────────────────────
    // Register base → derived so transitive MRO resolves. These back
    // isinstance(BytesIO(), io.BufferedIOBase) and friends.
    register_io_class("IOBase", &["object"], vec![]);
    register_io_class("RawIOBase", &["IOBase"], vec![]);
    register_io_class("BufferedIOBase", &["IOBase"], vec![]);
    register_io_class("TextIOBase", &["IOBase"], vec![]);
    register_io_class("FileIO", &["RawIOBase"], vec![]);
    register_io_class("BufferedReader", &["BufferedIOBase"], vec![]);
    register_io_class("BufferedWriter", &["BufferedIOBase"], vec![]);
    register_io_class("BufferedRandom", &["BufferedIOBase"], vec![]);
    register_io_class("BufferedRWPair", &["BufferedIOBase"], vec![]);
    register_io_class("TextIOWrapper", &["TextIOBase"], vec![]);
    // Concrete in-memory streams carry a native __iter__ so `for line in x` and
    // `list(x)` iterate over lines.
    register_io_class(
        "BytesIO",
        &["BufferedIOBase"],
        vec![("__iter__", dispatch_bytesio_iter as *const ())],
    );
    register_io_class(
        "StringIO",
        &["TextIOBase"],
        vec![("__iter__", dispatch_stringio_iter as *const ())],
    );

    let mut attrs = HashMap::new();

    // ── Constructor dispatchers (native funcs returning real Instances) ──
    let addr_sio = dispatch_stringio_new as *const () as usize;
    attrs.insert("StringIO".into(), MbValue::from_func(addr_sio));

    let addr_bio = dispatch_bytesio_new as *const () as usize;
    attrs.insert("BytesIO".into(), MbValue::from_func(addr_bio));

    let addr_tio = dispatch_textiowrapper_new as *const () as usize;
    attrs.insert("TextIOWrapper".into(), MbValue::from_func(addr_tio));

    let addr_br = dispatch_bufferedreader_new as *const () as usize;
    attrs.insert("BufferedReader".into(), MbValue::from_func(addr_br));

    let addr_bw = dispatch_bufferedwriter_new as *const () as usize;
    attrs.insert("BufferedWriter".into(), MbValue::from_func(addr_bw));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_sio as u64);
        set.insert(addr_bio as u64);
        set.insert(addr_tio as u64);
        set.insert(addr_br as u64);
        set.insert(addr_bw as u64);
    });

    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        let mut map = m.borrow_mut();
        map.insert(addr_sio as u64, "StringIO".into());
        map.insert(addr_bio as u64, "BytesIO".into());
        map.insert(addr_tio as u64, "TextIOWrapper".into());
        map.insert(addr_br as u64, "BufferedReader".into());
        map.insert(addr_bw as u64, "BufferedWriter".into());
    });

    // ── Abstract base classes exposed as registered class-name strings.
    // A registered class name is callable() and usable as an isinstance target.
    for cls in [
        "IOBase",
        "RawIOBase",
        "BufferedIOBase",
        "TextIOBase",
        "FileIO",
        "BufferedRandom",
        "BufferedRWPair",
    ] {
        attrs.insert(cls.into(), new_str(cls));
    }

    // ── open / open_code: route to the builtin open implementation.
    let addr_open = super::super::file_io::mb_open as *const () as usize;
    let open_val = MbValue::from_func(addr_open);
    attrs.insert("open".into(), open_val);
    attrs.insert("open_code".into(), open_val);

    // ── text_encoding(encoding[, stacklevel]) → encoding or "utf-8".
    let addr_te = dispatch_text_encoding as *const () as usize;
    attrs.insert("text_encoding".into(), MbValue::from_func(addr_te));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr_te as u64);
    });

    // ── UnsupportedOperation: subclass of OSError + ValueError (CPython).
    super::super::class::mb_class_register(
        "UnsupportedOperation",
        vec!["OSError".into(), "ValueError".into()],
        HashMap::new(),
    );
    attrs.insert(
        "UnsupportedOperation".into(),
        new_str("UnsupportedOperation"),
    );

    // ── IncrementalNewlineDecoder + BlockingIOError surface.
    attrs.insert(
        "IncrementalNewlineDecoder".into(),
        new_str("IncrementalNewlineDecoder"),
    );
    // BlockingIOError is a builtin exception already registered in exception.rs.
    attrs.insert("BlockingIOError".into(), new_str("BlockingIOError"));

    attrs.insert("DEFAULT_BUFFER_SIZE".into(), MbValue::from_int(8192));
    attrs.insert("SEEK_SET".into(), MbValue::from_int(0));
    attrs.insert("SEEK_CUR".into(), MbValue::from_int(1));
    attrs.insert("SEEK_END".into(), MbValue::from_int(2));

    super::register_module("io", attrs);
}

/// io.text_encoding(encoding[, stacklevel]) → encoding or "utf-8" when None.
unsafe extern "C" fn dispatch_text_encoding(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return new_str("utf-8");
    }
    let enc = unsafe { *args_ptr };
    if enc.is_none() {
        return new_str("utf-8");
    }
    if let Some(s) = extract_str(enc) {
        return new_str(s);
    }
    new_str("utf-8")
}

// ── StringIO ──

pub fn mb_stringio_new() -> MbValue {
    mb_stringio_new_with(String::new())
}

pub fn mb_stringio_new_with(initial: String) -> MbValue {
    let mut f = FxHashMap::default();
    f.insert(
        "_buffer".into(),
        MbValue::from_ptr(MbObject::new_str(initial)),
    );
    f.insert("_pos".into(), MbValue::from_int(0));
    make_instance("StringIO", f)
}

/// Read the StringIO buffer as a Vec<char> (positions are char-based, CPython
/// semantics) and the current char position.
fn stringio_chars_pos(sio: MbValue) -> (Vec<char>, usize) {
    if let Some(ptr) = sio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let buf: Vec<char> = f
                    .get("_buffer")
                    .and_then(|v| extract_str(*v))
                    .unwrap_or_default()
                    .chars()
                    .collect();
                let pos = f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
                return (buf, pos);
            }
        }
    }
    (Vec::new(), 0)
}

fn stringio_store(sio: MbValue, chars: &[char], pos: usize) {
    let s: String = chars.iter().collect();
    field_set(sio, "_buffer", new_str(s));
    field_set(sio, "_pos", MbValue::from_int(pos as i64));
}

pub fn mb_stringio_write(sio: MbValue, data: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    if !is_str(data) {
        // CPython: StringIO.write() argument must be str, not bytes/int/...
        let tn = type_name_of(data);
        return raise("TypeError", format!("string argument expected, got '{tn}'"));
    }
    let text: Vec<char> = extract_str(data).unwrap_or_default().chars().collect();
    let written = text.len() as i64;
    let (mut buf, pos) = stringio_chars_pos(sio);
    if pos > buf.len() {
        buf.resize(pos, '\0'); // pad with NULs when seeked past end
    }
    // Splice/overwrite at pos.
    let end = pos + text.len();
    if end > buf.len() {
        buf.resize(end, '\0');
    }
    buf[pos..end].copy_from_slice(&text);
    stringio_store(sio, &buf, end);
    MbValue::from_int(written)
}

/// read() / read(n): read up to n chars from current position (n<0/None = all).
pub fn mb_stringio_read_n(sio: MbValue, n: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let (buf, pos) = stringio_chars_pos(sio);
    let avail = buf.len().saturating_sub(pos);
    let take = match n.as_int() {
        Some(nn) if nn >= 0 => (nn as usize).min(avail),
        _ => avail,
    };
    let end = pos + take;
    let result: String = if pos < buf.len() {
        buf[pos..end].iter().collect()
    } else {
        String::new()
    };
    field_set(sio, "_pos", MbValue::from_int(end as i64));
    new_str(result)
}

pub fn mb_stringio_read(sio: MbValue) -> MbValue {
    mb_stringio_read_n(sio, MbValue::none())
}

/// readline(): next line including trailing '\n'.
pub fn mb_stringio_readline(sio: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let (buf, pos) = stringio_chars_pos(sio);
    if pos >= buf.len() {
        return new_str(String::new());
    }
    let mut end = pos;
    while end < buf.len() {
        let c = buf[end];
        end += 1;
        if c == '\n' {
            break;
        }
    }
    let line: String = buf[pos..end].iter().collect();
    field_set(sio, "_pos", MbValue::from_int(end as i64));
    new_str(line)
}

/// readlines(): list of all remaining lines.
pub fn mb_stringio_readlines(sio: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let mut lines: Vec<MbValue> = Vec::new();
    loop {
        let line = mb_stringio_readline(sio);
        if super::super::exception::mb_has_exception().as_bool() == Some(true) {
            return MbValue::none();
        }
        let empty = extract_str(line).map(|s| s.is_empty()).unwrap_or(true);
        if empty {
            break;
        }
        lines.push(line);
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

/// __iter__(): list-iterator over remaining lines (CPython iterates a file
/// object line by line). Returns a known iterator handle.
pub extern "C" fn dispatch_stringio_iter(sio: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let lines = mb_stringio_readlines(sio);
    super::super::iter::mb_iter(lines)
}

pub fn mb_stringio_getvalue(sio: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    if let Some(ptr) = sio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let buf = f
                    .get("_buffer")
                    .and_then(|v| extract_str(*v))
                    .unwrap_or_default();
                return new_str(buf);
            }
        }
    }
    new_str(String::new())
}

pub fn mb_stringio_seek_whence(sio: MbValue, pos: MbValue, whence: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let p = pos.as_int().unwrap_or(0);
    let w = whence.as_int().unwrap_or(0);
    let (buf, cur) = stringio_chars_pos(sio);
    let new_pos = match w {
        0 => {
            if p < 0 {
                return raise("ValueError", format!("Negative seek position {p}"));
            }
            p
        }
        1 => {
            // StringIO only supports seek-relative with offset 0.
            if p != 0 {
                return raise("ValueError", "Can't do nonzero cur-relative seeks");
            }
            cur as i64
        }
        2 => {
            if p != 0 {
                return raise("ValueError", "Can't do nonzero end-relative seeks");
            }
            buf.len() as i64
        }
        _ => {
            return raise(
                "ValueError",
                format!("invalid whence ({w}, should be 0, 1 or 2)"),
            );
        }
    };
    field_set(sio, "_pos", MbValue::from_int(new_pos));
    MbValue::from_int(new_pos)
}

pub fn mb_stringio_seek(sio: MbValue, pos: MbValue) -> MbValue {
    mb_stringio_seek_whence(sio, pos, MbValue::from_int(0))
}

pub fn mb_stringio_tell(sio: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let (_buf, pos) = stringio_chars_pos(sio);
    MbValue::from_int(pos as i64)
}

/// truncate([size]): drop everything after `size` (default: current position).
pub fn mb_stringio_truncate(sio: MbValue, size: MbValue) -> MbValue {
    if check_closed(sio) {
        return MbValue::none();
    }
    let (mut buf, pos) = stringio_chars_pos(sio);
    let n = match size.as_int() {
        Some(nn) if nn >= 0 => nn as usize,
        Some(nn) => return raise("ValueError", format!("Negative size value {nn}")),
        None => pos,
    };
    if n < buf.len() {
        buf.truncate(n);
        let s: String = buf.iter().collect();
        field_set(sio, "_buffer", new_str(s));
    }
    MbValue::from_int(n as i64)
}

pub fn mb_stringio_close(sio: MbValue) -> MbValue {
    field_set(sio, "_closed", MbValue::from_bool(true));
    MbValue::none()
}

// ── BytesIO ──

pub fn mb_bytesio_new() -> MbValue {
    mb_bytesio_new_with(Vec::new())
}

pub fn mb_bytesio_new_with(initial: Vec<u8>) -> MbValue {
    let mut f = FxHashMap::default();
    f.insert(
        "_buffer".into(),
        MbValue::from_ptr(MbObject::new_bytes(initial)),
    );
    f.insert("_pos".into(), MbValue::from_int(0));
    make_instance("BytesIO", f)
}

pub fn mb_bytesio_write(bio: MbValue, data: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    if !is_bytes_like(data) {
        // CPython: a bytes-like object is required, not 'str'/'int'/...
        let tn = type_name_of(data);
        return raise(
            "TypeError",
            format!("a bytes-like object is required, not '{tn}'"),
        );
    }
    let new_bytes = extract_bytes(data).unwrap_or_default();
    let written = new_bytes.len() as i64;

    if let Some(ptr) = bio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                let existing = f
                    .get("_buffer")
                    .and_then(|v| {
                        v.as_ptr().map(|p| match &(*p).data {
                            ObjData::Bytes(b) => b.clone(),
                            ObjData::ByteArray(ref lock) => lock.read().unwrap().clone(),
                            _ => Vec::new(),
                        })
                    })
                    .unwrap_or_default();
                let pos = f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0) as usize;
                let mut buf = existing;
                if pos >= buf.len() {
                    if pos > buf.len() {
                        buf.resize(pos, 0);
                    }
                    buf.extend_from_slice(&new_bytes);
                } else {
                    let end = pos + new_bytes.len();
                    if end > buf.len() {
                        buf.resize(end, 0);
                    }
                    buf[pos..pos + new_bytes.len()].copy_from_slice(&new_bytes);
                }
                let new_pos = pos + new_bytes.len();
                f.insert(
                    "_buffer".into(),
                    MbValue::from_ptr(MbObject::new_bytes(buf)),
                );
                f.insert("_pos".into(), MbValue::from_int(new_pos as i64));
            }
        }
    }
    MbValue::from_int(written)
}

pub fn mb_bytesio_read(bio: MbValue) -> MbValue {
    mb_bytesio_read_n(bio, MbValue::none())
}

/// readline([size]): next line up to and including '\n', capped at size.
pub fn mb_bytesio_readline(bio: MbValue, size: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    // A non-integer (e.g. float) size is a TypeError, like CPython.
    if !size.is_none() && size.as_int().is_none() {
        let tn = type_name_of(size);
        return raise(
            "TypeError",
            format!("'{tn}' object cannot be interpreted as an integer"),
        );
    }
    let buf = bytesio_buffer(bio);
    let pos = bytesio_pos(bio);
    if pos >= buf.len() {
        return MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
    }
    let cap = match size.as_int() {
        Some(nn) if nn >= 0 => Some(nn as usize),
        _ => None, // negative or None → whole line
    };
    let mut end = pos;
    while end < buf.len() {
        if let Some(c) = cap {
            if end - pos >= c {
                break;
            }
        }
        let b = buf[end];
        end += 1;
        if b == b'\n' {
            break;
        }
    }
    let line = buf[pos..end].to_vec();
    bytesio_set_pos(bio, end as i64);
    MbValue::from_ptr(MbObject::new_bytes(line))
}

/// readlines(): list of all remaining lines.
pub fn mb_bytesio_readlines(bio: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let mut lines: Vec<MbValue> = Vec::new();
    loop {
        let line = mb_bytesio_readline(bio, MbValue::none());
        let empty = extract_bytes(line).map(|b| b.is_empty()).unwrap_or(true);
        if empty {
            break;
        }
        lines.push(line);
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

/// __iter__(): list-iterator over remaining lines.
pub extern "C" fn dispatch_bytesio_iter(bio: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let lines = mb_bytesio_readlines(bio);
    super::super::iter::mb_iter(lines)
}

/// truncate([size]): drop bytes after `size` (default: current position).
pub fn mb_bytesio_truncate(bio: MbValue, size: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let mut buf = bytesio_buffer(bio);
    let pos = bytesio_pos(bio);
    let n = match size.as_int() {
        Some(nn) if nn >= 0 => nn as usize,
        Some(nn) => return raise("ValueError", format!("negative truncate position {nn}")),
        None => pos,
    };
    if n < buf.len() {
        buf.truncate(n);
        field_set(bio, "_buffer", MbValue::from_ptr(MbObject::new_bytes(buf)));
    }
    MbValue::from_int(n as i64)
}

pub fn mb_bytesio_close(bio: MbValue) -> MbValue {
    field_set(bio, "_closed", MbValue::from_bool(true));
    MbValue::none()
}

pub fn mb_bytesio_getvalue(bio: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    if let Some(ptr) = bio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                let buf = f
                    .get("_buffer")
                    .and_then(|v| {
                        v.as_ptr().map(|p| match &(*p).data {
                            ObjData::Bytes(b) => b.clone(),
                            ObjData::ByteArray(ref lock) => lock.read().unwrap().clone(),
                            _ => Vec::new(),
                        })
                    })
                    .unwrap_or_default();
                return MbValue::from_ptr(MbObject::new_bytes(buf));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

fn bytesio_buffer(bio: MbValue) -> Vec<u8> {
    if let Some(ptr) = bio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                return f
                    .get("_buffer")
                    .and_then(|v| {
                        v.as_ptr().map(|p| match &(*p).data {
                            ObjData::Bytes(b) => b.clone(),
                            ObjData::ByteArray(ref lock) => lock.read().unwrap().clone(),
                            _ => Vec::new(),
                        })
                    })
                    .unwrap_or_default();
            }
        }
    }
    Vec::new()
}

fn bytesio_pos(bio: MbValue) -> usize {
    if let Some(ptr) = bio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                return f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0) as usize;
            }
        }
    }
    0
}

fn bytesio_set_pos(bio: MbValue, pos: i64) {
    if let Some(ptr) = bio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("_pos".into(), MbValue::from_int(pos.max(0)));
            }
        }
    }
}

pub fn mb_bytesio_seek_with_whence(bio: MbValue, pos: MbValue, whence: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let p = pos.as_int().unwrap_or(0);
    let w = whence
        .is_none()
        .then_some(0)
        .unwrap_or_else(|| whence.as_int().unwrap_or(0));
    let buf_len = bytesio_buffer(bio).len() as i64;
    let cur = bytesio_pos(bio) as i64;
    let new_pos = match w {
        0 => {
            if p < 0 {
                return raise("ValueError", format!("negative seek value {p}"));
            }
            p
        }
        1 => cur + p,
        2 => buf_len + p,
        _ => {
            return raise(
                "ValueError",
                format!("invalid whence ({w}, should be 0, 1 or 2)"),
            )
        }
    };
    if new_pos < 0 {
        return raise("ValueError", "negative seek value");
    }
    bytesio_set_pos(bio, new_pos);
    MbValue::from_int(new_pos)
}

pub fn mb_bytesio_seek(bio: MbValue, pos: MbValue) -> MbValue {
    mb_bytesio_seek_with_whence(bio, pos, MbValue::from_int(0))
}

pub fn mb_bytesio_tell(bio: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    MbValue::from_int(bytesio_pos(bio) as i64)
}

pub fn mb_bytesio_read_n(bio: MbValue, n: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let buf = bytesio_buffer(bio);
    let pos = bytesio_pos(bio);
    let take = if let Some(nn) = n.as_int() {
        if nn < 0 {
            buf.len().saturating_sub(pos)
        } else {
            (nn as usize).min(buf.len().saturating_sub(pos))
        }
    } else {
        buf.len().saturating_sub(pos)
    };
    let end = pos + take;
    let result: Vec<u8> = if pos < buf.len() {
        buf[pos..end].to_vec()
    } else {
        Vec::new()
    };
    bytesio_set_pos(bio, end as i64);
    MbValue::from_ptr(MbObject::new_bytes(result))
}

pub fn mb_bytesio_readinto(bio: MbValue, dst: MbValue) -> MbValue {
    if check_closed(bio) {
        return MbValue::none();
    }
    let buf = bytesio_buffer(bio);
    let pos = bytesio_pos(bio);
    if let Some(ptr) = dst.as_ptr() {
        unsafe {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let mut ba = lock.write().unwrap();
                let cap = ba.len();
                let avail = buf.len().saturating_sub(pos);
                let take = cap.min(avail);
                for i in 0..take {
                    ba[i] = buf[pos + i];
                }
                bytesio_set_pos(bio, (pos + take) as i64);
                return MbValue::from_int(take as i64);
            }
        }
    }
    MbValue::from_int(0)
}

// ── TextIOWrapper ──

fn encode_str(s: &str, encoding: &str) -> Vec<u8> {
    let e = encoding.to_ascii_lowercase();
    match e.as_str() {
        "utf-8" | "utf8" | "u8" | "ascii" => s.as_bytes().to_vec(),
        "utf-16" => {
            let mut out = vec![0xff, 0xfe];
            for u in s.encode_utf16() {
                out.push((u & 0xff) as u8);
                out.push((u >> 8) as u8);
            }
            out
        }
        "utf-16-le" | "utf-16le" => {
            let mut out = Vec::with_capacity(s.len() * 2);
            for u in s.encode_utf16() {
                out.push((u & 0xff) as u8);
                out.push((u >> 8) as u8);
            }
            out
        }
        "utf-16-be" | "utf-16be" => {
            let mut out = Vec::with_capacity(s.len() * 2);
            for u in s.encode_utf16() {
                out.push((u >> 8) as u8);
                out.push((u & 0xff) as u8);
            }
            out
        }
        _ => s.as_bytes().to_vec(),
    }
}

fn decode_bytes(bytes: &[u8], encoding: &str) -> String {
    let e = encoding.to_ascii_lowercase();
    match e.as_str() {
        "utf-8" | "utf8" | "u8" | "ascii" => String::from_utf8_lossy(bytes).into_owned(),
        "utf-16" => {
            let (units, be) = if bytes.starts_with(&[0xff, 0xfe]) {
                (&bytes[2..], false)
            } else if bytes.starts_with(&[0xfe, 0xff]) {
                (&bytes[2..], true)
            } else {
                (bytes, false)
            };
            decode_utf16_units(units, be)
        }
        "utf-16-le" | "utf-16le" => decode_utf16_units(bytes, false),
        "utf-16-be" | "utf-16be" => decode_utf16_units(bytes, true),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

fn decode_utf16_units(units: &[u8], be: bool) -> String {
    let mut u16s = Vec::with_capacity(units.len() / 2);
    for ch in units.chunks_exact(2) {
        let v = if be {
            ((ch[0] as u16) << 8) | (ch[1] as u16)
        } else {
            (ch[0] as u16) | ((ch[1] as u16) << 8)
        };
        u16s.push(v);
    }
    String::from_utf16_lossy(&u16s)
}

pub fn mb_textiowrapper_new(
    underlying: MbValue,
    encoding: String,
    _newline: Option<String>,
    _write_through: bool,
) -> MbValue {
    let mut f = FxHashMap::default();
    unsafe {
        super::super::rc::retain_if_ptr(underlying);
    }
    f.insert("_buffer".into(), underlying);
    f.insert(
        "encoding".into(),
        MbValue::from_ptr(MbObject::new_str(encoding)),
    );
    make_instance("TextIOWrapper", f)
}

fn textiowrapper_underlying(tio: MbValue) -> MbValue {
    if let Some(ptr) = tio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("_buffer").copied() {
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

fn textiowrapper_encoding(tio: MbValue) -> String {
    if let Some(ptr) = tio.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("encoding").copied() {
                    if let Some(s) = extract_str(v) {
                        return s;
                    }
                }
            }
        }
    }
    "utf-8".into()
}

pub fn mb_textiowrapper_write(tio: MbValue, data: MbValue) -> MbValue {
    let text = extract_str(data).unwrap_or_default();
    let enc = textiowrapper_encoding(tio);
    let bytes = encode_str(&text, &enc);
    let under = textiowrapper_underlying(tio);
    let mb_bytes = MbValue::from_ptr(MbObject::new_bytes(bytes));
    mb_bytesio_write(under, mb_bytes);
    MbValue::from_int(text.chars().count() as i64)
}

pub fn mb_textiowrapper_read(tio: MbValue) -> MbValue {
    let enc = textiowrapper_encoding(tio);
    let under = textiowrapper_underlying(tio);
    let buf = bytesio_buffer(under);
    let pos = bytesio_pos(under);
    let remaining: Vec<u8> = if pos < buf.len() {
        buf[pos..].to_vec()
    } else {
        Vec::new()
    };
    bytesio_set_pos(under, buf.len() as i64);
    let decoded = decode_bytes(&remaining, &enc);
    MbValue::from_ptr(MbObject::new_str(decoded))
}

pub fn mb_textiowrapper_flush(_tio: MbValue) -> MbValue {
    MbValue::none()
}

// ── BufferedReader ──

pub fn mb_bufferedreader_new(underlying: MbValue, buffer_size: i64) -> MbValue {
    let mut f = FxHashMap::default();
    unsafe {
        super::super::rc::retain_if_ptr(underlying);
    }
    f.insert("_buffer".into(), underlying);
    f.insert("_buffer_size".into(), MbValue::from_int(buffer_size.max(1)));
    make_instance("BufferedReader", f)
}

fn bufferedreader_underlying(br: MbValue) -> MbValue {
    if let Some(ptr) = br.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("_buffer").copied() {
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_bufferedreader_read(br: MbValue, n: MbValue) -> MbValue {
    let under = bufferedreader_underlying(br);
    mb_bytesio_read_n(under, n)
}

pub fn mb_bufferedreader_read1(br: MbValue, n: MbValue) -> MbValue {
    let under = bufferedreader_underlying(br);
    mb_bytesio_read_n(under, n)
}

pub fn mb_bufferedreader_peek(br: MbValue, _n: MbValue) -> MbValue {
    let under = bufferedreader_underlying(br);
    let buf = bytesio_buffer(under);
    let pos = bytesio_pos(under);
    let rest: Vec<u8> = if pos < buf.len() {
        buf[pos..].to_vec()
    } else {
        Vec::new()
    };
    MbValue::from_ptr(MbObject::new_bytes(rest))
}

pub fn mb_bufferedreader_readline(br: MbValue) -> MbValue {
    let under = bufferedreader_underlying(br);
    let buf = bytesio_buffer(under);
    let pos = bytesio_pos(under);
    if pos >= buf.len() {
        return MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
    }
    let mut end = pos;
    while end < buf.len() {
        let b = buf[end];
        end += 1;
        if b == b'\n' {
            break;
        }
    }
    let line = buf[pos..end].to_vec();
    bytesio_set_pos(under, end as i64);
    MbValue::from_ptr(MbObject::new_bytes(line))
}

// ── BufferedWriter ──

unsafe extern "C" fn dispatch_bufferedwriter_new(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let kw = trailing_kwargs(a);
    let positional_end = if kw.is_some() {
        a.len().saturating_sub(1)
    } else {
        a.len()
    };
    let underlying = if positional_end >= 1 {
        a[0]
    } else {
        MbValue::none()
    };
    mb_bufferedwriter_new(underlying)
}

pub fn mb_bufferedwriter_new(underlying: MbValue) -> MbValue {
    let mut f = FxHashMap::default();
    unsafe {
        super::super::rc::retain_if_ptr(underlying);
    }
    f.insert("_buffer".into(), underlying);
    make_instance("BufferedWriter", f)
}

fn bufferedwriter_underlying(bw: MbValue) -> MbValue {
    if let Some(ptr) = bw.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("_buffer").copied() {
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

pub fn mb_bufferedwriter_write(bw: MbValue, data: MbValue) -> MbValue {
    let under = bufferedwriter_underlying(bw);
    mb_bytesio_write(under, data)
}

/// read()/read1()/peek() on a write-only buffer raise io.UnsupportedOperation.
pub fn mb_bufferedwriter_read(_bw: MbValue) -> MbValue {
    raise("UnsupportedOperation", "read")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stringio_write_and_getvalue() {
        let sio = mb_stringio_new();
        let data = MbValue::from_ptr(MbObject::new_str("hello world".to_string()));
        let written = mb_stringio_write(sio, data);
        assert_eq!(written.as_int(), Some(11));

        let val = mb_stringio_getvalue(sio);
        unsafe {
            if let ObjData::Str(ref s) = (*val.as_ptr().unwrap()).data {
                assert_eq!(s, "hello world");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_stringio_read_with_position() {
        let sio = mb_stringio_new();
        let data = MbValue::from_ptr(MbObject::new_str("abcdef".to_string()));
        mb_stringio_write(sio, data);
        // After writing, pos is at end. Seek 0 to read all.
        mb_stringio_seek(sio, MbValue::from_int(0));
        let result = mb_stringio_read(sio);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "abcdef");
            }
        }
        let result2 = mb_stringio_read(sio);
        unsafe {
            if let ObjData::Str(ref s) = (*result2.as_ptr().unwrap()).data {
                assert_eq!(s, "");
            }
        }
    }

    #[test]
    fn test_bytesio_write_and_getvalue() {
        let bio = mb_bytesio_new();
        let data = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2, 3]));
        mb_bytesio_write(bio, data);
        let data2 = MbValue::from_ptr(MbObject::new_bytes(vec![4, 5]));
        mb_bytesio_write(bio, data2);
        let val = mb_bytesio_getvalue(bio);
        unsafe {
            if let ObjData::Bytes(ref b) = (*val.as_ptr().unwrap()).data {
                assert_eq!(b, &[1, 2, 3, 4, 5]);
            } else {
                panic!("expected Bytes");
            }
        }
    }

    #[test]
    fn test_stringio_with_initial() {
        let sio = mb_stringio_new_with("preloaded".to_string());
        let val = mb_stringio_getvalue(sio);
        unsafe {
            if let ObjData::Str(ref s) = (*val.as_ptr().unwrap()).data {
                assert_eq!(s, "preloaded");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_stringio_seek_and_read() {
        let sio = mb_stringio_new_with("0123456789".to_string());
        mb_stringio_seek(sio, MbValue::from_int(5));
        let result = mb_stringio_read(sio);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "56789");
            }
        }
    }

    #[test]
    fn test_bytesio_read_full() {
        let bio = mb_bytesio_new_with(vec![0, 1, 2]);
        let result = mb_bytesio_read(bio);
        unsafe {
            if let ObjData::Bytes(ref b) = (*result.as_ptr().unwrap()).data {
                assert_eq!(b, &[0, 1, 2]);
            } else {
                panic!("expected Bytes");
            }
        }
    }
}
