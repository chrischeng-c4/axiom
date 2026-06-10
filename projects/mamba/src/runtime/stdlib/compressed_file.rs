//! Shared streaming-file layer for the compression modules (#1480 family).
//!
//! `bz2.BZ2File`, `lzma.LZMAFile`, and `gzip.GzipFile` share one
//! implementation: a buffered file object over either a Python file-like
//! object (anything with `read`/`write`, e.g. `io.BytesIO`) or a filesystem
//! path.
//!
//! Model (CPython-compatible for the in-scope fixture surface):
//!
//! - Write modes (`w`/`a`/`x`) accumulate raw payload in `_wbuf`; `close()`
//!   compresses the buffer into ONE complete stream and appends it to the
//!   sink. Repeated open-write-close cycles therefore produce concatenated
//!   streams, matching CPython's append semantics.
//! - Read mode (`r`) lazily drains the source on first read, decompresses
//!   the (possibly multi-stream) payload, and serves slices from `_plain`
//!   at `_pos`. `readline` / `__next__` / `read` / `readinto` / `peek`
//!   share that single position.
//! - The classes register variadic `(self, args_list)` method tables via
//!   `mb_class_register`, so the generic Instance dispatch (including the
//!   `__enter__` / `__exit__` / `__iter__` / `__next__` dunders) flows
//!   through `class.rs` unchanged.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::AtomicU32;

use super::super::rc::{InstanceFields, MbObject, MbObjectHeader, MbRwLock, ObjData, ObjKind};
use super::super::value::MbValue;

#[derive(Clone, Copy, PartialEq)]
pub enum Codec {
    Bz2 = 0,
    Xz = 1,
    Gzip = 2,
}

impl Codec {
    fn from_field(v: Option<MbValue>) -> Codec {
        match v.and_then(|v| v.as_int()) {
            Some(1) => Codec::Xz,
            Some(2) => Codec::Gzip,
            _ => Codec::Bz2,
        }
    }

    fn compress(self, data: &[u8]) -> Vec<u8> {
        match self {
            Codec::Bz2 => {
                let mut enc = bzip2::write::BzEncoder::new(
                    Vec::with_capacity(data.len() / 2 + 64),
                    bzip2::Compression::best(),
                );
                if enc.write_all(data).is_err() {
                    return Vec::new();
                }
                enc.finish().unwrap_or_default()
            }
            Codec::Xz => {
                let mut enc = xz2::write::XzEncoder::new(
                    Vec::with_capacity(data.len() / 2 + 64),
                    6,
                );
                if enc.write_all(data).is_err() {
                    return Vec::new();
                }
                enc.finish().unwrap_or_default()
            }
            Codec::Gzip => {
                let mut enc = flate2::write::GzEncoder::new(
                    Vec::with_capacity(data.len() / 2 + 64),
                    flate2::Compression::default(),
                );
                if enc.write_all(data).is_err() {
                    return Vec::new();
                }
                enc.finish().unwrap_or_default()
            }
        }
    }

    /// Multi-stream decompress: concatenated complete streams decode to the
    /// concatenated payloads (CPython BZ2File/LZMAFile/GzipFile semantics).
    fn decompress_multi(self, data: &[u8]) -> Result<Vec<u8>, ()> {
        if data.is_empty() {
            return Ok(Vec::new());
        }
        match self {
            Codec::Bz2 => {
                let mut out = Vec::with_capacity(data.len() * 4);
                let mut dec = bzip2::read::MultiBzDecoder::new(data);
                dec.read_to_end(&mut out).map_err(|_| ())?;
                Ok(out)
            }
            Codec::Xz => {
                let mut out = Vec::with_capacity(data.len() * 4);
                let mut dec = xz2::read::XzDecoder::new_multi_decoder(data);
                dec.read_to_end(&mut out).map_err(|_| ())?;
                Ok(out)
            }
            Codec::Gzip => {
                let mut out = Vec::with_capacity(data.len() * 4);
                let mut dec = flate2::read::MultiGzDecoder::new(data);
                dec.read_to_end(&mut out).map_err(|_| ())?;
                Ok(out)
            }
        }
    }
}

// ── small instance helpers ────────────────────────────────────────

fn raise(kind: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(kind.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn inst_field(v: MbValue, key: &str) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    }
}

fn inst_set(v: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(key.to_string(), val);
            }
        }
    }
}

fn as_str(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn as_bytes(v: MbValue) -> Option<Vec<u8>> {
    v.as_ptr().and_then(|p| unsafe {
        match (*p).data {
            ObjData::Bytes(ref b) => Some(b.clone()),
            ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn bytes_val(data: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(data))
}

fn list_items(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn call_fileobj(fileobj: MbValue, method: &str, args: Vec<MbValue>) -> MbValue {
    let name = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    let args_list = MbValue::from_ptr(MbObject::new_list(args));
    super::super::class::mb_call_method(fileobj, name, args_list)
}

// ── construction ──────────────────────────────────────────────────

/// Build a compressed-file Instance for `codec` over `source` (a file-like
/// object or a path string) in `mode` (r/w/a/x with optional `b`/`t`
/// suffix). Raises ValueError on a malformed mode.
pub fn make_file(class_name: &str, codec: Codec, source: MbValue, mode: &str) -> MbValue {
    make_file_opts(class_name, codec, source, mode, None, None)
}

/// `make_file` with the text-layer options the `open()` wrappers accept:
/// a `t` mode (or an explicit encoding) makes read return str and write
/// accept str, with `errors` controlling decode failures
/// (strict/ignore/replace).
pub fn make_file_opts(
    class_name: &str,
    codec: Codec,
    source: MbValue,
    mode: &str,
    encoding: Option<String>,
    errors: Option<String>,
) -> MbValue {
    let is_text = mode.contains('t') || encoding.is_some();
    let base = mode.trim_end_matches(['b', 't']);
    if !matches!(base, "r" | "w" | "x" | "a" | "") {
        return raise("ValueError", &format!("Invalid mode: {mode:?}"));
    }
    let base = if base.is_empty() { "r" } else { base };

    // CPython raises FileExistsError eagerly when an exclusive-create open
    // hits an existing path, and creates the file at open time so a second
    // `x` open fails. Mirror both for path sources.
    if base == "x" {
        if let Some(path) = as_str(source) {
            match std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&path)
            {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return raise(
                        "FileExistsError",
                        &format!("[Errno 17] File exists: {path:?}"),
                    );
                }
                Err(_) => {
                    return raise("OSError", &format!("cannot create {path:?}"));
                }
            }
        }
    }

    let mut fields = InstanceFields::default();
    fields.insert("_codec".to_string(), MbValue::from_int(codec as i64));
    unsafe { super::super::rc::retain_if_ptr(source) };
    fields.insert("_fileobj".to_string(), source);
    fields.insert(
        "_mode".to_string(),
        MbValue::from_ptr(MbObject::new_str(base.to_string())),
    );
    fields.insert("closed".to_string(), MbValue::from_bool(false));
    fields.insert("_wbuf".to_string(), bytes_val(Vec::new()));
    fields.insert("_pos".to_string(), MbValue::from_int(0));
    fields.insert("_text".to_string(), MbValue::from_bool(is_text));
    fields.insert(
        "_encoding".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            encoding.unwrap_or_else(|| "utf-8".to_string()),
        )),
    );
    fields.insert(
        "_errors".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            errors.unwrap_or_else(|| "strict".to_string()),
        )),
    );
    // GzipFile exposes the underlying filename as `.name` ('' for
    // in-memory file objects).
    fields.insert(
        "name".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            as_str(source).unwrap_or_default(),
        )),
    );

    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: class_name.to_string(),
            fields: MbRwLock::new(fields),
        },
    });
    let val = MbValue::from_ptr(Box::into_raw(obj));
    super::super::gc::gc_track(val.as_ptr().unwrap());
    val
}

fn check_open(slf: MbValue) -> bool {
    if inst_field(slf, "closed").and_then(|v| v.as_bool()) == Some(true) {
        raise("ValueError", "I/O operation on closed file");
        return false;
    }
    true
}

fn mode_of(slf: MbValue) -> String {
    inst_field(slf, "_mode").and_then(as_str).unwrap_or_else(|| "r".to_string())
}

/// Lazily decompress the full source into `_plain`; returns the plaintext.
fn ensure_plain(slf: MbValue) -> Option<Vec<u8>> {
    if let Some(p) = inst_field(slf, "_plain").and_then(as_bytes) {
        return Some(p);
    }
    let codec = Codec::from_field(inst_field(slf, "_codec"));
    let source = inst_field(slf, "_fileobj").unwrap_or(MbValue::none());
    let compressed = if let Some(path) = as_str(source) {
        std::fs::read(&path).ok()?
    } else {
        as_bytes(call_fileobj(source, "read", vec![]))?
    };
    match codec.decompress_multi(&compressed) {
        Ok(plain) => {
            inst_set(slf, "_plain", bytes_val(plain.clone()));
            Some(plain)
        }
        Err(()) => {
            raise("OSError", "Invalid data stream");
            None
        }
    }
}

fn pos_of(slf: MbValue) -> usize {
    inst_field(slf, "_pos").and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize
}

fn is_text(slf: MbValue) -> bool {
    inst_field(slf, "_text").and_then(|v| v.as_bool()) == Some(true)
}

fn encoding_of(slf: MbValue) -> String {
    inst_field(slf, "_encoding")
        .and_then(as_str)
        .unwrap_or_else(|| "utf-8".to_string())
        .to_ascii_lowercase()
        .replace('_', "-")
}

/// Encode a text payload with the file's declared encoding (utf-8 default,
/// utf-16 with BOM like CPython's TextIOWrapper).
fn encode_text(slf: MbValue, s: &str) -> Vec<u8> {
    match encoding_of(slf).as_str() {
        "utf-16" => {
            let mut out = vec![0xFF, 0xFE]; // BOM, little-endian
            for unit in s.encode_utf16() {
                out.extend_from_slice(&unit.to_le_bytes());
            }
            out
        }
        "latin-1" | "iso-8859-1" | "ascii" => s.chars().map(|c| c as u8).collect(),
        _ => s.as_bytes().to_vec(),
    }
}

/// Decode bytes with the file's declared encoding, honoring the `errors`
/// handler (strict raises UnicodeDecodeError; ignore/replace recover).
fn decode_text(slf: MbValue, data: &[u8]) -> Option<String> {
    let errors = inst_field(slf, "_errors")
        .and_then(as_str)
        .unwrap_or_else(|| "strict".to_string());
    match encoding_of(slf).as_str() {
        "utf-16" => {
            let body = if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
                &data[2..]
            } else {
                data
            };
            let units: Vec<u16> = body
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            Some(String::from_utf16_lossy(&units))
        }
        "latin-1" | "iso-8859-1" => Some(data.iter().map(|&b| b as char).collect()),
        _ => match std::str::from_utf8(data) {
            Ok(s) => Some(s.to_string()),
            Err(_) => match errors.as_str() {
                "ignore" => Some(
                    String::from_utf8_lossy(data).replace('\u{FFFD}', ""),
                ),
                "replace" => Some(String::from_utf8_lossy(data).into_owned()),
                _ => {
                    raise(
                        "UnicodeDecodeError",
                        "'utf-8' codec can't decode byte",
                    );
                    None
                }
            },
        },
    }
}

// ── methods (variadic (self, args_list) ABI) ──────────────────────

unsafe extern "C" fn m_write(slf: MbValue, args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let mode = mode_of(slf);
    if mode == "r" {
        return raise("UnsupportedOperation", "File not open for writing");
    }
    let items = list_items(args);
    let arg = items.first().copied().unwrap_or_else(MbValue::none);
    let data = if is_text(slf) {
        match as_str(arg) {
            Some(s) => {
                // Text mode reports characters written, not bytes.
                let n = s.chars().count() as i64;
                let encoded = encode_text(slf, &s);
                let mut buf =
                    inst_field(slf, "_wbuf").and_then(as_bytes).unwrap_or_default();
                buf.extend_from_slice(&encoded);
                inst_set(slf, "_wbuf", bytes_val(buf));
                return MbValue::from_int(n);
            }
            None => return raise("TypeError", "write() argument must be str"),
        }
    } else {
        match as_bytes(arg) {
            Some(b) => b,
            None => return raise("TypeError", "a bytes-like object is required"),
        }
    };
    let mut buf = inst_field(slf, "_wbuf").and_then(as_bytes).unwrap_or_default();
    buf.extend_from_slice(&data);
    let n = data.len() as i64;
    inst_set(slf, "_wbuf", bytes_val(buf));
    MbValue::from_int(n)
}

unsafe extern "C" fn m_writelines(slf: MbValue, args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    if mode_of(slf) == "r" {
        return raise("UnsupportedOperation", "File not open for writing");
    }
    let Some(lines) = list_items(args).first().copied() else {
        return raise("TypeError", "writelines() takes an iterable of bytes");
    };
    let mut buf = inst_field(slf, "_wbuf").and_then(as_bytes).unwrap_or_default();
    for line in list_items(lines) {
        let Some(data) = as_bytes(line) else {
            return raise("TypeError", "a bytes-like object is required");
        };
        buf.extend_from_slice(&data);
    }
    inst_set(slf, "_wbuf", bytes_val(buf));
    MbValue::none()
}

unsafe extern "C" fn m_read(slf: MbValue, args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    if mode_of(slf) != "r" {
        return raise("UnsupportedOperation", "File not open for reading");
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let pos = pos_of(slf).min(plain.len());
    let n = list_items(args)
        .first()
        .and_then(|v| v.as_int())
        .filter(|n| *n >= 0)
        .map(|n| n as usize)
        .unwrap_or(plain.len() - pos);
    let end = (pos + n).min(plain.len());
    inst_set(slf, "_pos", MbValue::from_int(end as i64));
    if is_text(slf) {
        return match decode_text(slf, &plain[pos..end]) {
            Some(s) => MbValue::from_ptr(MbObject::new_str(s)),
            None => MbValue::none(),
        };
    }
    bytes_val(plain[pos..end].to_vec())
}

fn readline_slice(plain: &[u8], pos: usize) -> (Vec<u8>, usize) {
    let rest = &plain[pos.min(plain.len())..];
    match rest.iter().position(|&b| b == b'\n') {
        Some(i) => (rest[..=i].to_vec(), pos + i + 1),
        None => (rest.to_vec(), plain.len()),
    }
}

unsafe extern "C" fn m_readline(slf: MbValue, _args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let (line, new_pos) = readline_slice(&plain, pos_of(slf));
    inst_set(slf, "_pos", MbValue::from_int(new_pos as i64));
    if is_text(slf) {
        return match decode_text(slf, &line) {
            Some(s) => MbValue::from_ptr(MbObject::new_str(s)),
            None => MbValue::none(),
        };
    }
    bytes_val(line)
}

unsafe extern "C" fn m_readlines(slf: MbValue, _args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let mut pos = pos_of(slf);
    let mut lines = Vec::new();
    while pos < plain.len() {
        let (line, new_pos) = readline_slice(&plain, pos);
        lines.push(bytes_val(line));
        pos = new_pos;
    }
    inst_set(slf, "_pos", MbValue::from_int(pos as i64));
    MbValue::from_ptr(MbObject::new_list(lines))
}

unsafe extern "C" fn m_peek(slf: MbValue, _args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let pos = pos_of(slf).min(plain.len());
    let end = (pos + 512).min(plain.len());
    bytes_val(plain[pos..end].to_vec())
}

unsafe extern "C" fn m_readinto(slf: MbValue, args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let pos = pos_of(slf).min(plain.len());
    let Some(target) = list_items(args).first().copied() else {
        return raise("TypeError", "readinto() takes a writable buffer");
    };
    let Some(ptr) = target.as_ptr() else {
        return raise("TypeError", "readinto() takes a writable buffer");
    };
    unsafe {
        if let ObjData::ByteArray(ref lock) = (*ptr).data {
            let mut ba = lock.write().unwrap();
            let n = ba.len().min(plain.len() - pos);
            ba[..n].copy_from_slice(&plain[pos..pos + n]);
            inst_set(slf, "_pos", MbValue::from_int((pos + n) as i64));
            return MbValue::from_int(n as i64);
        }
    }
    raise("TypeError", "readinto() argument must be a bytearray")
}

unsafe extern "C" fn m_tell(slf: MbValue, _args: MbValue) -> MbValue {
    // Write modes report the uncompressed bytes written so far; read mode
    // reports the read position.
    if mode_of(slf) != "r" {
        let written = inst_field(slf, "_wbuf")
            .and_then(as_bytes)
            .map(|b| b.len())
            .unwrap_or(0);
        return MbValue::from_int(written as i64);
    }
    MbValue::from_int(pos_of(slf) as i64)
}

unsafe extern "C" fn m_seek(slf: MbValue, args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let items = list_items(args);
    let offset = items.first().and_then(|v| v.as_int()).unwrap_or(0);
    let whence = items.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    // CPython clamps a seek past the end to the decompressed length.
    let new_pos = (match whence {
        1 => pos_of(slf) as i64 + offset,
        2 => plain.len() as i64 + offset,
        _ => offset,
    })
    .clamp(0, plain.len() as i64) as usize;
    inst_set(slf, "_pos", MbValue::from_int(new_pos as i64));
    MbValue::from_int(new_pos as i64)
}

unsafe extern "C" fn m_flush(_slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn m_readable(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(mode_of(slf) == "r")
}

unsafe extern "C" fn m_writable(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(mode_of(slf) != "r")
}

unsafe extern "C" fn m_seekable(slf: MbValue, _args: MbValue) -> MbValue {
    MbValue::from_bool(mode_of(slf) == "r")
}

unsafe extern "C" fn m_close(slf: MbValue, _args: MbValue) -> MbValue {
    if inst_field(slf, "closed").and_then(|v| v.as_bool()) == Some(true) {
        return MbValue::none();
    }
    let mode = mode_of(slf);
    if mode != "r" {
        let codec = Codec::from_field(inst_field(slf, "_codec"));
        let raw = inst_field(slf, "_wbuf").and_then(as_bytes).unwrap_or_default();
        let compressed = codec.compress(&raw);
        let source = inst_field(slf, "_fileobj").unwrap_or(MbValue::none());
        if let Some(path) = as_str(source) {
            use std::io::Write as _;
            // `x` already created the file eagerly at open; close just
            // fills it (truncate) like `w`.
            let result = std::fs::OpenOptions::new()
                .create(true)
                .append(mode == "a")
                .write(true)
                .truncate(mode != "a")
                .open(&path)
                .and_then(|mut f| f.write_all(&compressed));
            if result.is_err() {
                return raise("OSError", &format!("cannot write {path:?}"));
            }
        } else {
            call_fileobj(source, "write", vec![bytes_val(compressed)]);
        }
    }
    inst_set(slf, "closed", MbValue::from_bool(true));
    MbValue::none()
}

unsafe extern "C" fn m_enter(slf: MbValue, _args: MbValue) -> MbValue {
    slf
}

unsafe extern "C" fn m_exit(slf: MbValue, _args: MbValue) -> MbValue {
    unsafe { m_close(slf, MbValue::none()) };
    MbValue::from_bool(false)
}

unsafe extern "C" fn m_iter(slf: MbValue, _args: MbValue) -> MbValue {
    slf
}

unsafe extern "C" fn m_next(slf: MbValue, _args: MbValue) -> MbValue {
    if !check_open(slf) {
        return MbValue::none();
    }
    let Some(plain) = ensure_plain(slf) else { return MbValue::none() };
    let pos = pos_of(slf);
    if pos >= plain.len() {
        return raise("StopIteration", "");
    }
    let (line, new_pos) = readline_slice(&plain, pos);
    inst_set(slf, "_pos", MbValue::from_int(new_pos as i64));
    if is_text(slf) {
        return match decode_text(slf, &line) {
            Some(s) => MbValue::from_ptr(MbObject::new_str(s)),
            None => MbValue::none(),
        };
    }
    bytes_val(line)
}

/// Register one compressed-file class with the shared variadic method table.
pub fn register_class(class_name: &str) {
    let methods: Vec<(&str, usize)> = vec![
        ("write", m_write as *const () as usize),
        ("writelines", m_writelines as *const () as usize),
        ("read", m_read as *const () as usize),
        ("read1", m_read as *const () as usize),
        ("readline", m_readline as *const () as usize),
        ("readlines", m_readlines as *const () as usize),
        ("peek", m_peek as *const () as usize),
        ("readinto", m_readinto as *const () as usize),
        ("tell", m_tell as *const () as usize),
        ("seek", m_seek as *const () as usize),
        ("flush", m_flush as *const () as usize),
        ("readable", m_readable as *const () as usize),
        ("writable", m_writable as *const () as usize),
        ("seekable", m_seekable as *const () as usize),
        ("close", m_close as *const () as usize),
        ("__enter__", m_enter as *const () as usize),
        ("__exit__", m_exit as *const () as usize),
        ("__iter__", m_iter as *const () as usize),
        ("__next__", m_next as *const () as usize),
    ];
    let mut table: HashMap<String, MbValue> = HashMap::new();
    for (name, addr) in methods {
        table.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        super::super::module::register_variadic_func(addr as u64);
    }
    super::super::class::mb_class_register(class_name, vec![], table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_via_path() {
        let dir = std::env::temp_dir().join(format!("mamba-cfile-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("t.bz2");
        let path_str = path.to_string_lossy().to_string();

        let f = make_file("BZ2File", Codec::Bz2, MbValue::from_ptr(MbObject::new_str(path_str.clone())), "w");
        let args = MbValue::from_ptr(MbObject::new_list(vec![bytes_val(b"hello world".to_vec())]));
        unsafe { m_write(f, args) };
        unsafe { m_close(f, MbValue::none()) };

        let g = make_file("BZ2File", Codec::Bz2, MbValue::from_ptr(MbObject::new_str(path_str)), "r");
        let out = unsafe { m_read(g, MbValue::from_ptr(MbObject::new_list(vec![]))) };
        assert_eq!(as_bytes(out).as_deref(), Some(b"hello world".as_ref()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_multistream_append() {
        let codec = Codec::Bz2;
        let mut joined = codec.compress(b"foo");
        joined.extend_from_slice(&codec.compress(b"bar"));
        assert_eq!(codec.decompress_multi(&joined).unwrap(), b"foobar");
    }

    #[test]
    fn test_readline_positions() {
        let plain = b"alpha\nbeta\ngamma\n";
        let (l1, p1) = readline_slice(plain, 0);
        assert_eq!(l1, b"alpha\n");
        let (l2, p2) = readline_slice(plain, p1);
        assert_eq!(l2, b"beta\n");
        let (l3, p3) = readline_slice(plain, p2);
        assert_eq!(l3, b"gamma\n");
        assert_eq!(p3, plain.len());
    }
}
