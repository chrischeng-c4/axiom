use super::rc::MbObject;
use super::value::MbValue;
/// File I/O runtime support (#379).
///
/// Implements Python-compatible file operations: open, read, write, close.
/// Files are stored as a thread-local handle table (not heap objects) to avoid
/// complicating ObjData with non-Send types.
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};

/// File handle state.
#[allow(dead_code)]
struct MbFile {
    reader: Option<BufReader<fs::File>>,
    writer: Option<fs::File>,
    mode: String,
    path: String,
    closed: bool,
    /// Binary mode ('b' in mode): read/readline return bytes, write takes bytes.
    binary: bool,
    /// Readable (r / + modes).
    readable: bool,
    /// Writable (w / a / x / + modes).
    writable: bool,
    /// Append mode — handle is positioned at end of existing content.
    append: bool,
    /// Text-mode codec name (`f.encoding`), None in binary mode.
    encoding: Option<String>,
    /// Text-mode error handler (`f.errors`), None in binary mode.
    errors: Option<String>,
}

/// Parsed open() mode flags. Returns None on a structurally invalid mode.
struct ParsedMode {
    read: bool,
    write: bool,
    append: bool,
    create: bool, // 'x'
    plus: bool,   // '+'
    binary: bool, // 'b'
    text: bool,   // 't'
}

/// Validate and parse a CPython open() mode string. Mirrors CPython's checks:
/// exactly one of r/w/a/x, b and t are mutually exclusive, no duplicate chars,
/// only known characters allowed.
fn parse_mode(mode: &str) -> Option<ParsedMode> {
    let mut read = false;
    let mut write = false;
    let mut append = false;
    let mut create = false;
    let mut plus = false;
    let mut binary = false;
    let mut text = false;
    let mut seen = std::collections::HashSet::new();
    for ch in mode.chars() {
        if !seen.insert(ch) {
            return None; // duplicate character
        }
        match ch {
            'r' => read = true,
            'w' => write = true,
            'a' => append = true,
            'x' => create = true,
            '+' => plus = true,
            'b' => binary = true,
            't' => text = true,
            _ => return None, // unknown character
        }
    }
    // Exactly one of the primary modes.
    let primary = read as u8 + write as u8 + append as u8 + create as u8;
    if primary != 1 {
        return None;
    }
    // b and t cannot coexist.
    if binary && text {
        return None;
    }
    Some(ParsedMode {
        read,
        write,
        append,
        create,
        plus,
        binary,
        text,
    })
}

thread_local! {
    static FILES: std::cell::RefCell<HashMap<u64, MbFile>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_FILE_ID: std::cell::Cell<u64> = std::cell::Cell::new(1);
}

fn alloc_file_id() -> u64 {
    NEXT_FILE_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Check if a given ID corresponds to an open file handle.
pub fn is_file_handle(id: u64) -> bool {
    FILES.with(|files| files.borrow().contains_key(&id))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            super::rc::ObjData::Str(s) => Some(s.clone()),
            super::rc::ObjData::Bytes(bytes) => Some(String::from_utf8_lossy(bytes).into_owned()),
            super::rc::ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            super::rc::ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            super::rc::ObjData::Bytes(bytes) => Some(bytes.clone()),
            super::rc::ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

pub fn mb_file_name(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            files
                .borrow()
                .get(&(id as u64))
                .map(|file| MbValue::from_ptr(MbObject::new_str(file.path.clone())))
                .unwrap_or_else(MbValue::none)
        })
    } else {
        MbValue::none()
    }
}

/// open(path, mode) → file handle (as MbValue int)
pub fn mb_open(path: MbValue, mode: MbValue) -> MbValue {
    // str/bytes/bytearray directly; otherwise os.fspath coercion (pathlib
    // instances and any `__fspath__` provider) — CPython accepts str, bytes,
    // or os.PathLike here.
    let file_path =
        match extract_str(path).or_else(|| super::stdlib::pathlib_mod::coerce_fspath(path)) {
            Some(p) => p,
            None => {
                // A failing user `__fspath__` already left its own exception
                // pending — propagate that instead of masking it.
                if super::exception::mb_has_exception().as_bool() != Some(true) {
                    raise_type_error("open() argument must be a string");
                }
                return MbValue::none();
            }
        };
    // Embedded NUL byte in path is a ValueError (CPython).
    if file_path.contains('\0') {
        raise_value_error("embedded null byte");
        return MbValue::none();
    }
    let mode_str = if mode.is_none() {
        "r".to_string()
    } else {
        match extract_str(mode) {
            Some(s) => s,
            None => "r".to_string(),
        }
    };

    let parsed = match parse_mode(&mode_str) {
        Some(p) => p,
        None => {
            raise_value_error(&format!("invalid mode: '{mode_str}'"));
            return MbValue::none();
        }
    };

    let binary = parsed.binary;
    let readable = parsed.read || parsed.plus;
    let writable = parsed.write || parsed.append || parsed.create || parsed.plus;
    let append = parsed.append;

    // Build a file handle. Exclusive create ('x') must fail if the file exists.
    let open_result: std::io::Result<fs::File> = if parsed.read && !parsed.plus {
        // Pure read.
        fs::File::open(&file_path)
    } else if parsed.create {
        // 'x' / 'x+' — create new, error if exists.
        fs::OpenOptions::new()
            .read(parsed.plus)
            .write(true)
            .create_new(true)
            .open(&file_path)
    } else if parsed.append {
        // 'a' / 'a+' — create if missing, position at end.
        fs::OpenOptions::new()
            .read(parsed.plus)
            .append(true)
            .create(true)
            .open(&file_path)
    } else if parsed.write {
        // 'w' / 'w+' — truncate / create.
        fs::OpenOptions::new()
            .read(parsed.plus)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&file_path)
    } else if parsed.read && parsed.plus {
        // 'r+' — read/write, no create, no truncate.
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&file_path)
    } else {
        fs::File::open(&file_path)
    };

    match open_result {
        Ok(mut f) => {
            use std::io::{Seek, SeekFrom};
            // Append mode positions the handle at end-of-file so tell() reports
            // the existing content length (CPython contract).
            if append {
                let _ = f.seek(SeekFrom::End(0));
            }
            let id = alloc_file_id();
            // For read+write modes we keep both a reader (buffered) and a writer
            // backed by a cloned fd so each tracks its own position only when
            // needed; pure-read or pure-write modes keep just one.
            let (reader, writer) = if readable && writable {
                match f.try_clone() {
                    Ok(clone) => (Some(BufReader::new(f)), Some(clone)),
                    Err(_) => (Some(BufReader::new(f)), None),
                }
            } else if readable {
                (Some(BufReader::new(f)), None)
            } else {
                (None, Some(f))
            };
            // CPython text streams default to the locale codec ("UTF-8" here)
            // with "strict" error handling; binary streams expose neither.
            let (encoding, errors) = if binary {
                (None, None)
            } else {
                (Some("UTF-8".to_string()), Some("strict".to_string()))
            };
            let mf = MbFile {
                reader,
                writer,
                mode: mode_str,
                path: file_path,
                closed: false,
                binary,
                readable,
                writable,
                append,
                encoding,
                errors,
            };
            FILES.with(|files| files.borrow_mut().insert(id, mf));
            MbValue::from_int(id as i64)
        }
        Err(e) => {
            match e.kind() {
                std::io::ErrorKind::NotFound => raise_file_not_found(&file_path),
                std::io::ErrorKind::AlreadyExists => raise_file_exists(&file_path),
                std::io::ErrorKind::PermissionDenied => raise_permission_error(&file_path),
                _ => raise_os_error(&format!("Cannot open '{file_path}': {e}")),
            }
            MbValue::none()
        }
    }
}

/// open(path, mode, encoding, errors, closefd) — like `mb_open`, but records
/// the text codec / error-handler the caller passed so `f.encoding` /
/// `f.errors` reflect them. None args keep `mb_open`'s text defaults (UTF-8 /
/// strict); binary streams ignore both (they expose neither attribute).
/// `closefd=False` with a filename raises ValueError (CPython).
pub fn mb_open_ex(
    path: MbValue,
    mode: MbValue,
    encoding: MbValue,
    errors: MbValue,
    closefd: MbValue,
) -> MbValue {
    // closefd=False is only meaningful for an existing fd; with a filename
    // (a str path) CPython raises ValueError.
    if closefd.as_bool() == Some(false) && extract_str_opt(path).is_some() {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Cannot use closefd=False with file name".to_string(),
            )),
        );
        return MbValue::none();
    }
    let handle = mb_open(path, mode);
    if let Some(id) = handle.as_int() {
        let enc = extract_str_opt(encoding);
        let err = extract_str_opt(errors);
        FILES.with(|files| {
            if let Some(mf) = files.borrow_mut().get_mut(&(id as u64)) {
                if !mf.binary {
                    if let Some(e) = enc {
                        mf.encoding = Some(e);
                    }
                    if let Some(e) = err {
                        mf.errors = Some(e);
                    }
                }
            }
        });
    }
    handle
}

/// Extract a String from an MbValue str, or None for non-str / None.
fn extract_str_opt(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let crate::runtime::rc::ObjData::Str(ref s) = (*p).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// `f.mode` — the mode string the file was opened with.
pub fn mb_file_mode(handle: MbValue) -> MbValue {
    file_str_field(handle, |mf| Some(mf.mode.clone()))
}

/// `f.encoding` — text-mode codec name, or None in binary mode.
pub fn mb_file_encoding(handle: MbValue) -> MbValue {
    file_str_field(handle, |mf| mf.encoding.clone())
}

/// `f.errors` — text-mode error handler, or None in binary mode.
pub fn mb_file_errors(handle: MbValue) -> MbValue {
    file_str_field(handle, |mf| mf.errors.clone())
}

/// Read an optional string field off a file handle as an MbValue str (None when
/// the handle is unknown or the field is None).
fn file_str_field(handle: MbValue, f: impl Fn(&MbFile) -> Option<String>) -> MbValue {
    let Some(id) = handle.as_int() else { return MbValue::none() };
    FILES.with(|files| {
        match files.borrow().get(&(id as u64)).and_then(|mf| f(mf)) {
            Some(s) => MbValue::from_ptr(MbObject::new_str(s)),
            None => MbValue::none(),
        }
    })
}

/// file.read([size]) → str (text mode) or bytes (binary mode), entire contents
/// when size is omitted/negative, else up to size bytes.
pub fn mb_file_read(handle: MbValue) -> MbValue {
    mb_file_read_n(handle, MbValue::none())
}

/// file.read(size) — size-aware read; binary mode returns bytes.
pub fn mb_file_read_n(handle: MbValue, size: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                let binary = mf.binary;
                if let Some(ref mut reader) = mf.reader {
                    let mut bytes = Vec::new();
                    match size.as_int() {
                        Some(n) if n >= 0 => {
                            let mut limited = reader.take(n as u64);
                            let _ = limited.read_to_end(&mut bytes);
                        }
                        _ => {
                            let _ = reader.read_to_end(&mut bytes);
                        }
                    }
                    if binary {
                        return new_bytes(bytes);
                    }
                    return MbValue::from_ptr(MbObject::new_str(
                        String::from_utf8_lossy(&bytes).into_owned(),
                    ));
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.readline([size]) → str/bytes (one line)
pub fn mb_file_readline(handle: MbValue) -> MbValue {
    mb_file_readline_n(handle, MbValue::none())
}

/// file.readline(size) — size-capped line read; binary mode returns bytes.
pub fn mb_file_readline_n(handle: MbValue, size: MbValue) -> MbValue {
    // A non-integer size (e.g. float) is a TypeError, like CPython.
    if !size.is_none() && size.as_int().is_none() {
        raise_type_error("argument should be integer or None, not 'float'");
        return MbValue::none();
    }
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                let binary = mf.binary;
                let cap = match size.as_int() {
                    Some(n) if n >= 0 => Some(n as usize),
                    _ => None,
                };
                if let Some(ref mut reader) = mf.reader {
                    let mut line: Vec<u8> = Vec::new();
                    loop {
                        if let Some(c) = cap {
                            if line.len() >= c {
                                break;
                            }
                        }
                        let mut byte = [0u8; 1];
                        match reader.read(&mut byte) {
                            Ok(0) => break,
                            Ok(_) => {
                                line.push(byte[0]);
                                if byte[0] == b'\n' {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                    if binary {
                        return new_bytes(line);
                    }
                    return MbValue::from_ptr(MbObject::new_str(
                        String::from_utf8_lossy(&line).into_owned(),
                    ));
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.readlines() → list of strings
pub fn mb_file_readlines(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                let binary = mf.binary;
                if let Some(ref mut reader) = mf.reader {
                    let mut lines = Vec::new();
                    loop {
                        let mut raw: Vec<u8> = Vec::new();
                        let mut got = false;
                        loop {
                            let mut byte = [0u8; 1];
                            match reader.read(&mut byte) {
                                Ok(0) => break,
                                Ok(_) => {
                                    got = true;
                                    raw.push(byte[0]);
                                    if byte[0] == b'\n' {
                                        break;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                        if !got {
                            break;
                        }
                        if binary {
                            lines.push(new_bytes(raw));
                        } else {
                            lines.push(MbValue::from_ptr(MbObject::new_str(
                                String::from_utf8_lossy(&raw).into_owned(),
                            )));
                        }
                    }
                    return MbValue::from_ptr(MbObject::new_list(lines));
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.write(text) → number of characters written
pub fn mb_file_write(handle: MbValue, text: MbValue) -> MbValue {
    let content = extract_bytes(text).unwrap_or_default();
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut writer) = mf.writer {
                    match writer.write_all(&content) {
                        Ok(()) => {
                            let _ = writer.flush();
                            return MbValue::from_int(content.len() as i64);
                        }
                        Err(_) => return MbValue::none(),
                    }
                }
            }
            MbValue::none()
        })
    } else {
        MbValue::none()
    }
}

/// file.writelines(lines) → None
/// Writes each element of the iterable to the file (no separator added).
pub fn mb_file_writelines(handle: MbValue, lines: MbValue) -> MbValue {
    let iter_handle = super::iter::mb_iter(lines);
    if iter_handle.is_none() {
        return MbValue::none();
    }
    // Fast path: drain the iterator batch (avoids per-element HashMap lookups).
    if let Some(items) = super::iter::drain_iter_to_vec(iter_handle) {
        for item in items {
            mb_file_write(handle, item);
        }
        return MbValue::none();
    }
    // Fallback: standard iterator protocol.
    loop {
        if super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        let item = super::iter::mb_next(iter_handle);
        if item.is_none() && super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        mb_file_write(handle, item);
    }
    MbValue::none()
}

/// file.close()
pub fn mb_file_close(handle: MbValue) {
    if let Some(id) = handle.as_int() {
        FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                mf.closed = true;
                mf.reader = None;
                mf.writer = None;
            }
        });
    }
}

fn raise_type_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_value_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_file_not_found(path: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("FileNotFoundError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "No such file or directory: '{path}'"
        ))),
    );
}

fn raise_os_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_file_exists(path: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("FileExistsError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!("File exists: '{path}'"))),
    );
}

fn raise_permission_error(path: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("PermissionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!("Permission denied: '{path}'"))),
    );
}

fn new_bytes(b: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(b))
}

/// True iff `id` is an open (not closed) file handle whose ops need a closed
/// guard. Returns the handle's `closed` flag.
pub fn is_file_closed(handle: MbValue) -> bool {
    if let Some(id) = handle.as_int() {
        return FILES.with(|files| {
            files
                .borrow()
                .get(&(id as u64))
                .map(|f| f.closed)
                .unwrap_or(false)
        });
    }
    false
}

/// file.tell() — current byte offset.
pub fn mb_file_tell(handle: MbValue) -> MbValue {
    use std::io::{Seek, SeekFrom};
    if let Some(id) = handle.as_int() {
        return FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut r) = mf.reader {
                    let pos = r.stream_position().unwrap_or(0);
                    return MbValue::from_int(pos as i64);
                }
                if let Some(ref mut w) = mf.writer {
                    let pos = w.seek(SeekFrom::Current(0)).unwrap_or(0);
                    return MbValue::from_int(pos as i64);
                }
                return MbValue::from_int(0);
            }
            MbValue::none()
        });
    }
    MbValue::none()
}

/// file.seek(offset[, whence]) — reposition; returns new absolute offset.
pub fn mb_file_seek(handle: MbValue, offset: MbValue, whence: MbValue) -> MbValue {
    use std::io::{Seek, SeekFrom};
    let off = offset.as_int().unwrap_or(0);
    let w = if whence.is_none() {
        0
    } else {
        whence.as_int().unwrap_or(0)
    };
    let from = match w {
        0 => {
            if off < 0 {
                raise_value_error("negative seek value");
                return MbValue::none();
            }
            SeekFrom::Start(off as u64)
        }
        1 => SeekFrom::Current(off),
        2 => SeekFrom::End(off),
        _ => {
            raise_value_error(&format!("invalid whence ({w}, should be 0, 1 or 2)"));
            return MbValue::none();
        }
    };
    if let Some(id) = handle.as_int() {
        return FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut r) = mf.reader {
                    let pos = r.seek(from).unwrap_or(0);
                    return MbValue::from_int(pos as i64);
                }
                if let Some(ref mut wr) = mf.writer {
                    let pos = wr.seek(from).unwrap_or(0);
                    return MbValue::from_int(pos as i64);
                }
                return MbValue::from_int(0);
            }
            MbValue::none()
        });
    }
    MbValue::none()
}

/// file.flush() — raises ValueError if closed; otherwise flushes the writer.
pub fn mb_file_flush(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        return FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                if let Some(ref mut w) = mf.writer {
                    let _ = w.flush();
                }
                return MbValue::none();
            }
            MbValue::none()
        });
    }
    MbValue::none()
}

/// file.truncate([size]) — truncate to size (default current position).
pub fn mb_file_truncate(handle: MbValue, size: MbValue) -> MbValue {
    use std::io::{Seek, SeekFrom};
    if let Some(id) = handle.as_int() {
        return FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if mf.closed {
                    raise_value_error("I/O operation on closed file");
                    return MbValue::none();
                }
                let n: u64 = match size.as_int() {
                    Some(nn) if nn >= 0 => nn as u64,
                    _ => {
                        // default: current position
                        if let Some(ref mut w) = mf.writer {
                            w.seek(SeekFrom::Current(0)).unwrap_or(0)
                        } else if let Some(ref mut r) = mf.reader {
                            r.stream_position().unwrap_or(0)
                        } else {
                            0
                        }
                    }
                };
                if let Some(ref mut w) = mf.writer {
                    let _ = w.set_len(n);
                }
                return MbValue::from_int(n as i64);
            }
            MbValue::none()
        });
    }
    MbValue::none()
}

/// file.readinto(buf) — read into a bytearray, return count.
pub fn mb_file_readinto(handle: MbValue, dst: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        let closed = is_file_closed(handle);
        if closed {
            raise_value_error("I/O operation on closed file");
            return MbValue::none();
        }
        return FILES.with(|files| {
            let mut files = files.borrow_mut();
            if let Some(mf) = files.get_mut(&(id as u64)) {
                if let Some(ref mut reader) = mf.reader {
                    if let Some(ptr) = dst.as_ptr() {
                        unsafe {
                            if let super::rc::ObjData::ByteArray(ref lock) = (*ptr).data {
                                let mut ba = lock.write().unwrap();
                                let n = reader.read(&mut ba).unwrap_or(0);
                                return MbValue::from_int(n as i64);
                            }
                        }
                    }
                }
            }
            MbValue::from_int(0)
        });
    }
    MbValue::none()
}

// ── Cleanup ──

/// Reset all file I/O thread_local state to defaults.
/// Drains the FILES HashMap, dropping MbFile handles to close fds.
/// Called as part of centralized runtime cleanup between test executions.
pub(crate) fn cleanup_all_files() {
    let _ = FILES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = NEXT_FILE_ID.with(|c| c.set(1));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let tmp = std::env::temp_dir().join("mamba_test_file_io.txt");
        let path_str = tmp.to_string_lossy().to_string();
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_w = MbValue::from_ptr(MbObject::new_str("w".to_string()));

        // Write
        let fh = mb_open(path, mode_w);
        assert!(fh.as_int().is_some());
        let text = MbValue::from_ptr(MbObject::new_str("hello\nworld\n".to_string()));
        let written = mb_file_write(fh, text);
        assert_eq!(written.as_int(), Some(12));
        mb_file_close(fh);

        // Read
        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_r = MbValue::from_ptr(MbObject::new_str("r".to_string()));
        let fh2 = mb_open(path2, mode_r);
        let content = mb_file_read(fh2);
        assert!(content.is_ptr());
        mb_file_close(fh2);

        // Cleanup
        let _ = std::fs::remove_file(&path_str);
    }

    // ── Cleanup tests (R1, S3: per-module cleanup for files) ──

    #[test]
    fn test_cleanup_all_files_closes_handles() {
        let tmp = std::env::temp_dir().join("mamba_cleanup_file_test.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Open a file for writing
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh = mb_open(path, mode);
        assert!(fh.as_int().is_some(), "should get a valid file handle");

        let fh_id = fh.as_int().unwrap() as u64;
        assert!(
            is_file_handle(fh_id),
            "file should be in FILES before cleanup"
        );

        cleanup_all_files();

        assert!(
            !is_file_handle(fh_id),
            "FILES should be empty after cleanup — file handles dropped"
        );

        // Cleanup temp file
        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_cleanup_all_files_resets_id_counter() {
        let tmp = std::env::temp_dir().join("mamba_cleanup_id_test.txt");
        let path_str = tmp.to_string_lossy().to_string();

        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh1 = mb_open(path, mode);
        mb_file_close(fh1);

        cleanup_all_files();

        // After cleanup, next file should get ID 1 again
        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode2 = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh2 = mb_open(path2, mode2);
        assert_eq!(
            fh1.as_int(),
            fh2.as_int(),
            "file ID counter should reset after cleanup"
        );
        mb_file_close(fh2);

        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_cleanup_all_files_on_empty() {
        cleanup_all_files();
        // No panic = success
    }

    #[test]
    fn test_read_after_close_raises_value_error() {
        super::super::exception::mb_clear_exception();
        let tmp = std::env::temp_dir().join("mamba_test_read_after_close.txt");
        let path_str = tmp.to_string_lossy().to_string();
        let path = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        let fh = mb_open(path, mode);
        let text = MbValue::from_ptr(MbObject::new_str("data".to_string()));
        mb_file_write(fh, text);
        mb_file_close(fh);

        let path2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let mode_r = MbValue::from_ptr(MbObject::new_str("r".to_string()));
        let fh2 = mb_open(path2, mode_r);
        mb_file_close(fh2);
        let _result = mb_file_read(fh2);
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "ValueError must be pending after read on closed file",
        );
        let exc = super::super::exception::mb_get_exception();
        let exc_type = super::super::exception::get_exception_type_pub(exc);
        assert_eq!(exc_type.as_deref(), Some("ValueError"));
        super::super::exception::mb_clear_exception();
        let _ = std::fs::remove_file(&path_str);
    }

    #[test]
    fn test_readline_and_readlines() {
        let tmp = std::env::temp_dir().join("mamba_test_readline.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Write two lines
        let p = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh = mb_open(p, MbValue::from_ptr(MbObject::new_str("w".into())));
        mb_file_write(
            fh,
            MbValue::from_ptr(MbObject::new_str("aaa\nbbb\n".into())),
        );
        mb_file_close(fh);

        // readline returns first line
        let p2 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh2 = mb_open(p2, MbValue::from_ptr(MbObject::new_str("r".into())));
        let line = mb_file_readline(fh2);
        assert!(line.is_ptr(), "readline should return a string");
        mb_file_close(fh2);

        // readlines returns a list
        let p3 = MbValue::from_ptr(MbObject::new_str(path_str.clone()));
        let fh3 = mb_open(p3, MbValue::from_ptr(MbObject::new_str("r".into())));
        let lines = mb_file_readlines(fh3);
        assert!(lines.is_ptr(), "readlines should return a list");
        mb_file_close(fh3);

        let _ = std::fs::remove_file(&path_str);
    }

    // ── Method dispatch tests (REQ: file handle method dispatch on integer IDs) ──

    /// Verify that mb_call_method dispatches write/read/close via the file
    /// handle path (the bug fix: integers that are file handles must not fall
    /// through to the generic "int has no attribute" error).
    #[test]
    fn test_method_dispatch_write_and_read() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_write_read.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // --- write via mb_call_method ---
        let fh_w = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );
        assert!(
            fh_w.as_int().is_some(),
            "open('w') should return an int handle"
        );

        let method_write = MbValue::from_ptr(MbObject::new_str("write".into()));
        let text_val = MbValue::from_ptr(MbObject::new_str("dispatch test\n".into()));
        let args_list = MbValue::from_ptr(MbObject::new_list(vec![text_val]));
        let written = super::super::class::mb_call_method(fh_w, method_write, args_list);
        assert!(
            written.as_int().is_some(),
            "write via dispatch should return byte count"
        );

        let method_close = MbValue::from_ptr(MbObject::new_str("close".into()));
        super::super::class::mb_call_method(fh_w, method_close, MbValue::none());

        // --- read via mb_call_method ---
        let fh_r = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_read = MbValue::from_ptr(MbObject::new_str("read".into()));
        let content = super::super::class::mb_call_method(fh_r, method_read, MbValue::none());
        assert!(content.is_ptr(), "read via dispatch should return a string");

        let method_close2 = MbValue::from_ptr(MbObject::new_str("close".into()));
        super::super::class::mb_call_method(fh_r, method_close2, MbValue::none());

        let _ = std::fs::remove_file(&path_str);
    }

    /// Verify that writelines dispatches correctly through mb_call_method.
    #[test]
    fn test_method_dispatch_writelines() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_writelines.txt");
        let path_str = tmp.to_string_lossy().to_string();

        let fh = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );

        let line1 = MbValue::from_ptr(MbObject::new_str("line one\n".into()));
        let line2 = MbValue::from_ptr(MbObject::new_str("line two\n".into()));
        let lines_list = MbValue::from_ptr(MbObject::new_list(vec![line1, line2]));
        let method_wl = MbValue::from_ptr(MbObject::new_str("writelines".into()));
        let args_list = MbValue::from_ptr(MbObject::new_list(vec![lines_list]));
        super::super::class::mb_call_method(fh, method_wl, args_list);
        mb_file_close(fh);

        // Verify the file contents
        let contents = std::fs::read_to_string(&path_str).unwrap_or_default();
        assert!(
            contents.contains("line one"),
            "writelines should write first line"
        );
        assert!(
            contents.contains("line two"),
            "writelines should write second line"
        );

        let _ = std::fs::remove_file(&path_str);
    }

    /// Verify that readline dispatches correctly through mb_call_method.
    #[test]
    fn test_method_dispatch_readline_readlines() {
        let tmp = std::env::temp_dir().join("mamba_dispatch_readline.txt");
        let path_str = tmp.to_string_lossy().to_string();

        // Prepare file
        let fh_w = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("w".into())),
        );
        mb_file_write(fh_w, MbValue::from_ptr(MbObject::new_str("a\nb\n".into())));
        mb_file_close(fh_w);

        // readline via dispatch
        let fh_r = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_rl = MbValue::from_ptr(MbObject::new_str("readline".into()));
        let line = super::super::class::mb_call_method(fh_r, method_rl, MbValue::none());
        assert!(
            line.is_ptr(),
            "readline via dispatch should return a string"
        );
        mb_file_close(fh_r);

        // readlines via dispatch
        let fh_r2 = mb_open(
            MbValue::from_ptr(MbObject::new_str(path_str.clone())),
            MbValue::from_ptr(MbObject::new_str("r".into())),
        );
        let method_rls = MbValue::from_ptr(MbObject::new_str("readlines".into()));
        let lines = super::super::class::mb_call_method(fh_r2, method_rls, MbValue::none());
        assert!(
            lines.is_ptr(),
            "readlines via dispatch should return a list"
        );
        mb_file_close(fh_r2);

        let _ = std::fs::remove_file(&path_str);
    }
}
