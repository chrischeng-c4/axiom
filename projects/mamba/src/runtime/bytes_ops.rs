/// Bytes and ByteArray operations for the Mamba runtime (#405).
///
/// Implements Python-compatible bytes/bytearray methods.

use super::value::MbValue;
use super::rc::{MbObject, ObjData};

/// Helper: get immutable reference to bytes data.
/// Returns a cloned Vec<u8> since ByteArray is now behind RwLock.
unsafe fn as_bytes_cloned(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| {
        match &(*ptr).data {
            ObjData::Bytes(ref data) => Some(data.clone()),
            ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

// ── Creation ──

/// Convert a slice of element values (from `bytes(iterable)` /
/// `bytearray(iterable)`) into bytes, validating each is an integer in
/// `range(0, 256)`. On an out-of-range integer (or a BigInt, which is always
/// out of range) raises ValueError; on a non-integer element raises TypeError.
/// Returns `None` after raising so the caller can return early.
/// `bytes(-1)` / `bytearray(-1)` → ValueError: negative count.
fn raise_negative_count() {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str("negative count".to_string())),
    );
}

/// `bytes(2**63)` / `bytearray(<bigint>)` → OverflowError: count too large.
fn raise_count_overflow() {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "cannot fit 'int' into an index-sized integer".to_string())),
    );
}

fn raise_type_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn raise_lookup_error(msg: &str) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("LookupError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

fn str_from_value(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    })
}

fn encode_str_with_encoding(s: &str, encoding: &str) -> Option<Vec<u8>> {
    let normalized = encoding.to_ascii_lowercase().replace('_', "-");
    match normalized.as_str() {
        "utf-8" | "utf8" => Some(s.as_bytes().to_vec()),
        "ascii" if s.is_ascii() => Some(s.as_bytes().to_vec()),
        "ascii" => {
            raise_type_error("'ascii' codec can't encode character");
            None
        }
        _ => {
            raise_lookup_error(&format!("unknown encoding: {encoding}"));
            None
        }
    }
}

fn validated_bytes_from_items(items: &[MbValue]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(items.len());
    for &v in items {
        if let Some(i) = v.as_int() {
            if (0..=255).contains(&i) {
                out.push(i as u8);
                continue;
            }
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("byte must be in range(0, 256)".to_string())),
            );
            return None;
        }
        // BigInt is an integer that's always out of byte range -> ValueError;
        // any other type is not an integer -> TypeError.
        let is_bigint = unsafe { matches!(v.as_ptr().map(|p| &(*p).data), Some(ObjData::BigInt(_))) };
        if is_bigint {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str("byte must be in range(0, 256)".to_string())),
            );
        } else {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "'object' cannot be interpreted as an integer".to_string())),
            );
        }
        return None;
    }
    Some(out)
}

/// Drain an iterator handle into a Vec<u8>. Handles fast-path iter kinds
/// (Range, Reversed, List, Tuple) via `drain_iter_to_vec`; falls back to
/// the standard `mb_has_next`/`mb_next` protocol for the rest (Generator,
/// Enumerate, Zip, callable, user-defined). Released items are dropped
/// after the byte payload is extracted.
unsafe fn drain_handle_to_u8s(handle: MbValue) -> Option<Vec<u8>> {
    let items: Vec<MbValue> = if let Some(v) = super::iter::drain_iter_to_vec(handle) {
        v
    } else {
        // Slow path: standard iterator protocol. drain_iter_to_vec put the
        // entry back on miss, so the handle is still valid.
        let h = super::iter::mb_iter(handle);
        let mut out = Vec::new();
        loop {
            if super::iter::mb_has_next(h).as_bool() != Some(true) { break; }
            out.push(super::iter::mb_next(h));
        }
        out
    };
    let data = validated_bytes_from_items(&items);
    for &it in &items { super::rc::release_if_ptr(it); }
    data
}

/// Create bytes from a string.
pub fn mb_bytes_new(source: MbValue) -> MbValue {
    if source.is_none() {
        return MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
    }
    // Iterator handle: drain. Must precede the `as_int` branch — iterator IDs
    // are themselves NaN-boxed ints (base 2^32), and `as_int` would otherwise
    // treat them as multi-GB pre-allocation requests (#2103).
    if super::iter::is_iter_handle(source) {
        unsafe {
            return match drain_handle_to_u8s(source) {
                Some(d) => MbValue::from_ptr(MbObject::new_bytes(d)),
                None => MbValue::none(),
            };
        }
    }
    if let Some(n) = source.as_int() {
        if n < 0 { raise_negative_count(); return MbValue::none(); }
        return MbValue::from_ptr(MbObject::new_bytes(vec![0u8; n as usize]));
    }
    if let Some(ptr) = source.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    return MbValue::from_ptr(MbObject::new_bytes(s.as_bytes().to_vec()));
                }
                ObjData::Bytes(data) => {
                    return MbValue::from_ptr(MbObject::new_bytes(data.clone()));
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap().clone();
                    return MbValue::from_ptr(MbObject::new_bytes(data));
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap().clone();
                    return match validated_bytes_from_items(&items) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytes(d)),
                        None => MbValue::none(),
                    };
                }
                // A BigInt count is too large for a size-sized integer.
                ObjData::BigInt(_) => { raise_count_overflow(); return MbValue::none(); }
                ObjData::Instance { ref class_name, .. } => {
                    // User __bytes__ dunder; a class without one cannot
                    // convert (CPython TypeError, not a silent b'').
                    let cls = class_name.clone();
                    let m = super::class::lookup_method(&cls, "__bytes__");
                    if !m.is_none() {
                        let method = MbValue::from_ptr(MbObject::new_str("__bytes__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                        let r = super::class::mb_call_method(source, method, args);
                        if let Some(rp) = r.as_ptr() {
                            if matches!((*rp).data, ObjData::Bytes(_)) {
                                return r;
                            }
                        }
                        if super::exception::mb_has_exception().as_bool() == Some(true) {
                            return MbValue::none();
                        }
                    }
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "cannot convert '{cls}' object to bytes"
                        ))),
                    );
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

pub fn mb_bytes_new_checked(source: MbValue) -> MbValue {
    if str_from_value(source).is_some() {
        raise_type_error("string argument without an encoding");
        return MbValue::none();
    }
    mb_bytes_new(source)
}

pub fn mb_bytes_new_encoded(source: MbValue, encoding: MbValue) -> MbValue {
    if let Some(s) = str_from_value(source) {
        let Some(enc) = str_from_value(encoding) else {
            raise_type_error("encoding must be str");
            return MbValue::none();
        };
        return match encode_str_with_encoding(&s, &enc) {
            Some(data) => MbValue::from_ptr(MbObject::new_bytes(data)),
            None => MbValue::none(),
        };
    }
    mb_bytes_new(source)
}

/// Create bytearray from a string or bytes.
pub fn mb_bytearray_new(source: MbValue) -> MbValue {
    if source.is_none() {
        return MbValue::from_ptr(MbObject::new_bytearray(Vec::new()));
    }
    // Iterator handle: drain. Same #2103 rationale as `mb_bytes_new`.
    if super::iter::is_iter_handle(source) {
        unsafe {
            return match drain_handle_to_u8s(source) {
                Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                None => MbValue::none(),
            };
        }
    }
    if let Some(n) = source.as_int() {
        if n < 0 { raise_negative_count(); return MbValue::none(); }
        return MbValue::from_ptr(MbObject::new_bytearray(vec![0u8; n as usize]));
    }
    if let Some(ptr) = source.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    return MbValue::from_ptr(MbObject::new_bytearray(s.as_bytes().to_vec()));
                }
                ObjData::Bytes(data) => {
                    return MbValue::from_ptr(MbObject::new_bytearray(data.clone()));
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    return MbValue::from_ptr(MbObject::new_bytearray(data.clone()));
                }
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap().clone();
                    return match validated_bytes_from_items(&items) {
                        Some(d) => MbValue::from_ptr(MbObject::new_bytearray(d)),
                        None => MbValue::none(),
                    };
                }
                ObjData::BigInt(_) => { raise_count_overflow(); return MbValue::none(); }
                _ => {}
            }
        }
    }
    MbValue::from_ptr(MbObject::new_bytearray(Vec::new()))
}

pub fn mb_bytearray_new_checked(source: MbValue) -> MbValue {
    if str_from_value(source).is_some() {
        raise_type_error("string argument without an encoding");
        return MbValue::none();
    }
    mb_bytearray_new(source)
}

pub fn mb_bytearray_new_encoded(source: MbValue, encoding: MbValue) -> MbValue {
    if let Some(s) = str_from_value(source) {
        let Some(enc) = str_from_value(encoding) else {
            raise_type_error("encoding must be str");
            return MbValue::none();
        };
        return match encode_str_with_encoding(&s, &enc) {
            Some(data) => MbValue::from_ptr(MbObject::new_bytearray(data)),
            None => MbValue::none(),
        };
    }
    mb_bytearray_new(source)
}

// ── Access ──

/// bytes[index] → int
pub fn mb_bytes_getitem(bytes: MbValue, index: MbValue) -> MbValue {
    unsafe {
        if let (Some(data), Some(idx)) = (as_bytes_cloned(bytes), index.as_int()) {
            let len = data.len() as i64;
            let actual = if idx < 0 { idx + len } else { idx };
            if actual >= 0 && actual < len {
                MbValue::from_int(data[actual as usize] as i64)
            } else {
                MbValue::none()
            }
        } else {
            MbValue::none()
        }
    }
}

/// len(bytes) → int
pub fn mb_bytes_len(bytes: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            MbValue::from_int(data.len() as i64)
        } else {
            MbValue::from_int(0)
        }
    }
}

/// bytes.decode(encoding) → str
pub fn mb_bytes_decode(bytes: MbValue, encoding: MbValue) -> MbValue {
    mb_bytes_decode_with(bytes, encoding, MbValue::none())
}

/// `bytes.decode(encoding="utf-8", errors="strict")` — full form.
///
/// Supported encodings: `utf-8` / `utf8`, `ascii` / `us-ascii`, and
/// `latin-1` / `iso-8859-1`. Errors handlers: `strict` / `ignore` /
/// `replace`. UTF-8 strict-mode UnicodeDecodeError isn't modeled — we
/// fall back to the replacement char rather than raising — but the
/// `ignore` and `replace` paths produce CPython-equivalent output.
pub fn mb_bytes_decode_with(bytes: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    unsafe {
        let data = match as_bytes_cloned(bytes) {
            Some(d) => d,
            None => return MbValue::from_ptr(MbObject::new_str(String::new())),
        };
        let enc = match encoding.as_ptr().and_then(|p| match &(*p).data {
            ObjData::Str(s) => Some(s.to_ascii_lowercase()),
            _ => None,
        }) {
            Some(e) => e,
            None => "utf-8".to_string(),
        };
        let err = encoding_errors_kind(errors);
        let s = match enc.as_str() {
            "utf-8" | "utf8" | "u8" => decode_utf8(&data, err),
            "ascii" | "us-ascii" => decode_ascii(&data, err),
            "latin-1" | "latin_1" | "iso-8859-1" | "8859" => decode_latin1(&data),
            "utf-16be" | "utf-16-be" | "utf_16_be" => decode_utf16(&data, true),
            "utf-16le" | "utf-16-le" | "utf_16_le" => decode_utf16(&data, false),
            // Bare "utf-16": consume a leading BOM to pick endianness (LE
            // default, matching CPython). Used by test_xml_encodings.
            "utf-16" | "utf16" => {
                if data.len() >= 2 && data[0] == 0xFE && data[1] == 0xFF {
                    decode_utf16(&data[2..], true)
                } else if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xFE {
                    decode_utf16(&data[2..], false)
                } else {
                    decode_utf16(&data, false)
                }
            }
            "utf-32be" | "utf-32-be" | "utf_32_be" => decode_utf32(&data, true),
            "utf-32le" | "utf-32-le" | "utf_32_le" => decode_utf32(&data, false),
            "utf-32" | "utf32" => {
                if data.len() >= 4 && data[..4] == [0x00, 0x00, 0xFE, 0xFF] {
                    decode_utf32(&data[4..], true)
                } else if data.len() >= 4 && data[..4] == [0xFF, 0xFE, 0x00, 0x00] {
                    decode_utf32(&data[4..], false)
                } else {
                    decode_utf32(&data, false)
                }
            }
            _ => decode_utf8(&data, err),
        };
        MbValue::from_ptr(MbObject::new_str(s))
    }
}

fn encoding_errors_kind(errors: MbValue) -> ErrorsKind {
    let raw = errors.as_ptr().and_then(|p| unsafe {
        match &(*p).data { ObjData::Str(s) => Some(s.as_str()), _ => None }
    });
    match raw {
        Some("ignore") => ErrorsKind::Ignore,
        Some("replace") => ErrorsKind::Replace,
        _ => ErrorsKind::Strict,
    }
}

#[derive(Copy, Clone)]
enum ErrorsKind { Strict, Ignore, Replace }

fn decode_utf8(data: &[u8], err: ErrorsKind) -> String {
    // Walk the buffer one UTF-8 sequence at a time. On invalid bytes:
    //   - ignore → drop them
    //   - replace / strict → emit a single U+FFFD per *failure* (CPython
    //     consumes one byte per error and emits one replacement; the
    //     stdlib `from_utf8_lossy` agrees).
    let mut out = String::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        match std::str::from_utf8(&data[i..]) {
            Ok(rest) => { out.push_str(rest); break; }
            Err(e) => {
                let valid = e.valid_up_to();
                out.push_str(std::str::from_utf8(&data[i..i + valid]).unwrap_or(""));
                let bad_len = e.error_len().unwrap_or(data.len() - i - valid);
                match err {
                    ErrorsKind::Ignore => {}
                    ErrorsKind::Replace | ErrorsKind::Strict => out.push('\u{FFFD}'),
                }
                i += valid + bad_len.max(1);
            }
        }
    }
    out
}

fn decode_ascii(data: &[u8], err: ErrorsKind) -> String {
    let mut out = String::with_capacity(data.len());
    for &b in data {
        if b < 0x80 {
            out.push(b as char);
        } else {
            match err {
                ErrorsKind::Ignore => {}
                ErrorsKind::Replace | ErrorsKind::Strict => out.push('\u{FFFD}'),
            }
        }
    }
    out
}

fn decode_latin1(data: &[u8]) -> String {
    // Every byte 0x00..=0xFF is a valid Latin-1 code point — never errors.
    data.iter().map(|&b| b as char).collect()
}

fn decode_utf16(data: &[u8], big_endian: bool) -> String {
    let units: Vec<u16> = data
        .chunks_exact(2)
        .map(|c| if big_endian { u16::from_be_bytes([c[0], c[1]]) }
                 else { u16::from_le_bytes([c[0], c[1]]) })
        .collect();
    char::decode_utf16(units.into_iter())
        .map(|r| r.unwrap_or('\u{FFFD}'))
        .collect()
}

fn decode_utf32(data: &[u8], big_endian: bool) -> String {
    data.chunks_exact(4)
        .map(|c| {
            let u = if big_endian { u32::from_be_bytes([c[0], c[1], c[2], c[3]]) }
                    else { u32::from_le_bytes([c[0], c[1], c[2], c[3]]) };
            char::from_u32(u).unwrap_or('\u{FFFD}')
        })
        .collect()
}

/// bytes.hex(sep=None, bytes_per_sep=1) → str
///
/// CPython 3.8+ accepts an optional separator and grouping size:
///   - With `sep` only (or `bytes_per_sep == 1`), insert `sep` between every byte.
///   - Positive `bytes_per_sep` groups bytes from the right end (the leftmost
///     group may be shorter); negative values group from the left.
///   - `sep` may be a single-character `str` or single-byte `bytes`.
pub fn mb_bytes_hex(bytes: MbValue) -> MbValue {
    mb_bytes_hex_with_sep(bytes, MbValue::none(), MbValue::none())
}

pub fn mb_bytes_hex_with_sep(bytes: MbValue, sep: MbValue, bytes_per_sep: MbValue) -> MbValue {
    unsafe {
        let Some(data) = as_bytes_cloned(bytes) else {
            return MbValue::from_ptr(MbObject::new_str(String::new()));
        };
        // Resolve separator: accept str (first char) or bytes (first byte).
        // None / unrecognised types fall through to the unseparated form.
        let sep_char: Option<char> = if sep.is_none() {
            None
        } else if let Some(s) = sep.as_ptr().and_then(|p| match &(*p).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }) {
            s.chars().next()
        } else if let Some(b) = sep.as_ptr().and_then(|p| match &(*p).data {
            ObjData::Bytes(b) => Some(b.clone()),
            _ => None,
        }) {
            b.first().map(|byte| *byte as char)
        } else {
            None
        };
        let bps = bytes_per_sep.as_int().unwrap_or(1);
        if sep_char.is_none() || data.is_empty() {
            let hex: String = data.iter().map(|b| format!("{b:02x}")).collect();
            return MbValue::from_ptr(MbObject::new_str(hex));
        }
        let sep = sep_char.unwrap();
        let group = bps.unsigned_abs() as usize;
        if group == 0 {
            let hex: String = data.iter().map(|b| format!("{b:02x}")).collect();
            return MbValue::from_ptr(MbObject::new_str(hex));
        }
        let n = data.len();
        let mut result = String::with_capacity(n * 2 + n / group);
        if bps >= 0 {
            // Group from the right: leftmost group may be shorter.
            let rem = n % group;
            let mut i = 0;
            if rem > 0 {
                for j in 0..rem {
                    result.push_str(&format!("{:02x}", data[j]));
                }
                i = rem;
                if i < n { result.push(sep); }
            }
            while i < n {
                for j in 0..group {
                    result.push_str(&format!("{:02x}", data[i + j]));
                }
                i += group;
                if i < n { result.push(sep); }
            }
        } else {
            // Group from the left: rightmost group may be shorter.
            let mut i = 0;
            while i < n {
                let end = (i + group).min(n);
                for j in i..end {
                    result.push_str(&format!("{:02x}", data[j]));
                }
                i = end;
                if i < n { result.push(sep); }
            }
        }
        MbValue::from_ptr(MbObject::new_str(result))
    }
}

/// bytes.find(sub) → int (-1 if not found)
pub fn mb_bytes_find(haystack: MbValue, needle: MbValue) -> MbValue {
    mb_bytes_find_range(haystack, needle, MbValue::none(), MbValue::none())
}

/// Normalize a bytes search needle. Accepts a bytes-like object OR an integer
/// in `range(0, 256)` (CPython lets `b.find(105)` search for that byte value).
/// Returns `None` for non-bytes / out-of-range ints (caller treats as "not
/// found").
fn needle_as_bytes(needle: MbValue) -> Option<Vec<u8>> {
    if let Some(i) = needle.as_int() {
        return if (0..=255).contains(&i) { Some(vec![i as u8]) } else { None };
    }
    unsafe { as_bytes_cloned(needle) }
}

/// `bytes.find(sub, start=0, end=len)` — search a slice from the left.
/// Mirrors `str.find`'s clamp rules: defaults `0`/`len`, negatives
/// counted from the end and clamped to 0, positives capped at `len`.
/// The returned index is absolute (not relative to start).
pub fn mb_bytes_find_range(haystack: MbValue, needle: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        if let (Some(h), Some(n)) = (as_bytes_cloned(haystack), needle_as_bytes(needle)) {
            let (s, e) = clamp_range(h.len(), start, end);
            if n.is_empty() {
                // Empty needle matches at `start` if the slice is non-empty
                // (CPython returns `start` even when start == end for empty
                // sub on empty haystack — match that quirk).
                return MbValue::from_int(s as i64);
            }
            if s + n.len() > e {
                return MbValue::from_int(-1);
            }
            let slice = &h[s..e];
            let pos = slice.windows(n.len()).position(|w| w == n.as_slice());
            MbValue::from_int(pos.map(|p| (s + p) as i64).unwrap_or(-1))
        } else {
            MbValue::from_int(-1)
        }
    }
}

/// `bytes.rfind(sub, start=0, end=len)` — search a slice from the right.
pub fn mb_bytes_rfind(haystack: MbValue, needle: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        if let (Some(h), Some(n)) = (as_bytes_cloned(haystack), needle_as_bytes(needle)) {
            let (s, e) = clamp_range(h.len(), start, end);
            if n.is_empty() {
                return MbValue::from_int(e as i64);
            }
            if s + n.len() > e {
                return MbValue::from_int(-1);
            }
            let slice = &h[s..e];
            let pos = slice.windows(n.len()).rposition(|w| w == n.as_slice());
            MbValue::from_int(pos.map(|p| (s + p) as i64).unwrap_or(-1))
        } else {
            MbValue::from_int(-1)
        }
    }
}

/// CPython start/end clamping shared by `find` / `rfind` / `count` /
/// `startswith` / `endswith` on bytes. Negative indices count from the
/// end (clamped to 0); positive indices are capped at `len`.
fn clamp_range(len: usize, start: MbValue, end: MbValue) -> (usize, usize) {
    let len_i = len as i64;
    let mut s = start.as_int().unwrap_or(0);
    if s < 0 { s = (s + len_i).max(0); } else if s > len_i { s = len_i; }
    let mut e = end.as_int().unwrap_or(len_i);
    if e < 0 { e = (e + len_i).max(0); } else if e > len_i { e = len_i; }
    if e < s { e = s; }
    (s as usize, e as usize)
}

/// bytes + bytes → new bytes
pub fn mb_bytes_concat(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        if let (Some(da), Some(db)) = (as_bytes_cloned(a), as_bytes_cloned(b)) {
            let mut result = da;
            result.extend_from_slice(&db);
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            MbValue::none()
        }
    }
}

/// value in bytes → bool
pub fn mb_bytes_contains(bytes: MbValue, value: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            if let Some(i) = value.as_int() {
                // An integer membership test checks for that byte value; the
                // integer must be a valid byte (CPython raises ValueError
                // otherwise, rather than silently truncating with `as u8`).
                if !(0..=255).contains(&i) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "byte must be in range(0, 256)".to_string())),
                    );
                    return MbValue::none();
                }
                return MbValue::from_bool(data.contains(&(i as u8)));
            }
            // A BigInt search value is an integer out of byte range.
            if matches!(value.as_ptr().map(|p| &(*p).data), Some(ObjData::BigInt(_))) {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "byte must be in range(0, 256)".to_string())),
                );
                return MbValue::none();
            }
            if let Some(sub) = as_bytes_cloned(value) {
                // Empty needle is contained in any bytes (matches CPython) and
                // would also panic `windows(0)`.
                if sub.is_empty() {
                    return MbValue::from_bool(true);
                }
                return MbValue::from_bool(
                    data.windows(sub.len()).any(|w| w == sub.as_slice())
                );
            }
            // Neither an integer nor a bytes-like object → TypeError.
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "a bytes-like object is required".to_string())),
            );
            return MbValue::none();
        }
        MbValue::from_bool(false)
    }
}

// ── ByteArray mutations ──

/// bytearray.append(int)
pub fn mb_bytearray_append(ba: MbValue, value: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                if let Some(v) = value.as_int() {
                    lock.write().unwrap().push(v as u8);
                }
            }
        }
    }
}

/// bytearray.extend(iterable)
pub fn mb_bytearray_extend(ba: MbValue, other: MbValue) {
    unsafe {
        if let Some(other_data) = as_bytes_cloned(other) {
            if let Some(ptr) = ba.as_ptr() {
                if let ObjData::ByteArray(ref lock) = (*ptr).data {
                    lock.write().unwrap().extend_from_slice(&other_data);
                }
            }
        }
    }
}

/// bytearray.clear()
pub fn mb_bytearray_clear(ba: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
}

/// bytearray.pop() → int
pub fn mb_bytearray_pop(ba: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                return lock.write().unwrap().pop()
                    .map(|b| MbValue::from_int(b as i64))
                    .unwrap_or(MbValue::none());
            }
        }
        MbValue::none()
    }
}

/// bytearray.reverse()
pub fn mb_bytearray_reverse(ba: MbValue) {
    unsafe {
        if let Some(ptr) = ba.as_ptr() {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                lock.write().unwrap().reverse();
            }
        }
    }
}

/// bytes.count(sub) -> int
pub fn mb_bytes_count(haystack: MbValue, needle: MbValue) -> MbValue {
    mb_bytes_count_range(haystack, needle, MbValue::none(), MbValue::none())
}

/// `bytes.count(sub, start=0, end=len)` — count non-overlapping
/// occurrences of `sub` within the slice. Empty needle returns
/// `len(slice) + 1` to match CPython.
pub fn mb_bytes_count_range(haystack: MbValue, needle: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        if let (Some(h), Some(n)) = (as_bytes_cloned(haystack), as_bytes_cloned(needle)) {
            let (s, e) = clamp_range(h.len(), start, end);
            let slice = &h[s..e];
            if n.is_empty() {
                return MbValue::from_int((slice.len() + 1) as i64);
            }
            // Non-overlapping count: jump by n.len() after each hit, matching
            // CPython's str.count semantics.
            let mut count = 0i64;
            let mut i = 0usize;
            while i + n.len() <= slice.len() {
                if &slice[i..i + n.len()] == n.as_slice() {
                    count += 1;
                    i += n.len();
                } else {
                    i += 1;
                }
            }
            MbValue::from_int(count)
        } else {
            MbValue::from_int(0)
        }
    }
}

/// bytes.startswith(prefix) -> bool
/// Supports both bytes and tuple of bytes as prefix argument.
pub fn mb_bytes_startswith(haystack: MbValue, prefix: MbValue) -> MbValue {
    mb_bytes_startswith_range(haystack, prefix, MbValue::none(), MbValue::none())
}

/// `bytes.startswith(prefix, start=0, end=len)` — slice-aware test.
pub fn mb_bytes_startswith_range(haystack: MbValue, prefix: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        let Some(h) = as_bytes_cloned(haystack) else { return MbValue::from_bool(false); };
        let (s, e) = clamp_range(h.len(), start, end);
        let slice = &h[s..e];
        if let Some(ptr) = prefix.as_ptr() {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                for item in items {
                    if let Some(p) = as_bytes_cloned(*item) {
                        if slice.starts_with(p.as_slice()) {
                            return MbValue::from_bool(true);
                        }
                    }
                }
                return MbValue::from_bool(false);
            }
        }
        if let Some(p) = as_bytes_cloned(prefix) {
            MbValue::from_bool(slice.starts_with(p.as_slice()))
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// bytes.endswith(suffix) -> bool
/// Supports both bytes and tuple of bytes as suffix argument.
pub fn mb_bytes_endswith(haystack: MbValue, suffix: MbValue) -> MbValue {
    mb_bytes_endswith_range(haystack, suffix, MbValue::none(), MbValue::none())
}

/// `bytes.endswith(suffix, start=0, end=len)` — slice-aware test.
pub fn mb_bytes_endswith_range(haystack: MbValue, suffix: MbValue, start: MbValue, end: MbValue) -> MbValue {
    unsafe {
        let Some(h) = as_bytes_cloned(haystack) else { return MbValue::from_bool(false); };
        let (s, e) = clamp_range(h.len(), start, end);
        let slice = &h[s..e];
        if let Some(ptr) = suffix.as_ptr() {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                for item in items {
                    if let Some(p) = as_bytes_cloned(*item) {
                        if slice.ends_with(p.as_slice()) {
                            return MbValue::from_bool(true);
                        }
                    }
                }
                return MbValue::from_bool(false);
            }
        }
        if let Some(p) = as_bytes_cloned(suffix) {
            MbValue::from_bool(slice.ends_with(p.as_slice()))
        } else {
            MbValue::from_bool(false)
        }
    }
}

/// bytes.replace(old, new) -> bytes
pub fn mb_bytes_replace(haystack: MbValue, old: MbValue, new: MbValue) -> MbValue {
    mb_bytes_replace_count(haystack, old, new, MbValue::none())
}

/// `bytes.replace(old, new, count=-1)` — replace at most `count` occurrences.
/// `count == -1` (or negative) means "all", matching CPython.
pub fn mb_bytes_replace_count(haystack: MbValue, old: MbValue, new: MbValue, count: MbValue) -> MbValue {
    unsafe {
        if let (Some(h), Some(o), Some(n)) = (
            as_bytes_cloned(haystack),
            as_bytes_cloned(old),
            as_bytes_cloned(new),
        ) {
            let max = count.as_int().unwrap_or(-1);
            let unlimited = max < 0;
            let mut remaining = if unlimited { i64::MAX } else { max };
            if o.is_empty() {
                // Replace empty pattern: insert `new` before each byte (and at
                // the end). CPython caps the insertions at `count + 1` because
                // the empty needle matches `len + 1` positions.
                let mut result = Vec::new();
                for byte in &h {
                    if remaining > 0 { result.extend_from_slice(&n); remaining -= 1; }
                    result.push(*byte);
                }
                if remaining > 0 { result.extend_from_slice(&n); }
                return MbValue::from_ptr(MbObject::new_bytes(result));
            }
            let mut result = Vec::new();
            let mut i = 0;
            while i < h.len() {
                if remaining > 0 && i + o.len() <= h.len() && &h[i..i + o.len()] == o.as_slice() {
                    result.extend_from_slice(&n);
                    i += o.len();
                    remaining -= 1;
                } else {
                    result.push(h[i]);
                    i += 1;
                }
            }
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            // Always return a new object so the JIT can release
            // input and output VRegs independently (avoids double-free).
            if let Some(h) = as_bytes_cloned(haystack) {
                MbValue::from_ptr(MbObject::new_bytes(h))
            } else {
                MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
            }
        }
    }
}

fn bytes_clamp_index(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(0).min(len)
}

fn bytes_clamp_rev(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(-1).min(len - 1)
}

/// bytes[start:stop:step] → new bytes
pub fn mb_bytes_slice_full(
    bytes: MbValue, start: MbValue, stop: MbValue, step: MbValue,
) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            let len = data.len() as i64;
            let st = step.as_int().unwrap_or(1);
            if st == 0 {
                return MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
            }
            let (s, e) = if st > 0 {
                let s = start.as_int().map(|i| bytes_clamp_index(i, len)).unwrap_or(0);
                let e = stop.as_int().map(|i| bytes_clamp_index(i, len)).unwrap_or(len);
                (s, e)
            } else {
                let s = start.as_int().map(|i| bytes_clamp_rev(i, len)).unwrap_or(len - 1);
                let e = stop.as_int().map(|i| bytes_clamp_rev(i, len)).unwrap_or(-1);
                (s, e)
            };
            let mut result = Vec::new();
            let mut i = s;
            if st > 0 {
                while i < e {
                    if i >= 0 && i < len { result.push(data[i as usize]); }
                    i += st;
                }
            } else {
                while i > e {
                    if i >= 0 && i < len { result.push(data[i as usize]); }
                    i += st;
                }
            }
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
        }
    }
}

/// bytes.split(sep) -> list[bytes]
/// Splits bytes by sep, returning a list of bytes objects.
pub fn mb_bytes_split(haystack: MbValue, sep: MbValue) -> MbValue {
    mb_bytes_split_max(haystack, sep, MbValue::none())
}

/// `bytes.split(sep, maxsplit=-1)` — at most `maxsplit` splits; the
/// remainder of the buffer becomes the last element. `-1` (or any
/// negative) means "all", matching CPython.
pub fn mb_bytes_split_max(haystack: MbValue, sep: MbValue, maxsplit: MbValue) -> MbValue {
    unsafe {
        if let Some(h) = as_bytes_cloned(haystack) {
            let max = maxsplit.as_int().unwrap_or(-1);
            let unlimited = max < 0;
            let parts: Vec<MbValue> = if sep.is_none() {
                // Whitespace-split (no sep): match CPython by walking the
                // buffer manually so we can honour `maxsplit`. Default is
                // "split on all runs"; once `maxsplit` splits have been made
                // the remainder of the buffer (with leading whitespace
                // trimmed off) becomes the last element.
                let mut result = Vec::new();
                let mut splits_done = 0i64;
                let mut i = 0usize;
                let len = h.len();
                while i < len {
                    while i < len && h[i].is_ascii_whitespace() { i += 1; }
                    if i >= len { break; }
                    if !unlimited && splits_done >= max {
                        result.push(MbValue::from_ptr(MbObject::new_bytes(h[i..].to_vec())));
                        return MbValue::from_ptr(MbObject::new_list(result));
                    }
                    let start = i;
                    while i < len && !h[i].is_ascii_whitespace() { i += 1; }
                    result.push(MbValue::from_ptr(MbObject::new_bytes(h[start..i].to_vec())));
                    splits_done += 1;
                }
                result
            } else if let Some(s) = as_bytes_cloned(sep) {
                if s.is_empty() {
                    return MbValue::from_ptr(MbObject::new_list(Vec::new()));
                }
                let mut result = Vec::new();
                let mut start = 0usize;
                let mut splits_done = 0i64;
                loop {
                    if !unlimited && splits_done >= max {
                        result.push(MbValue::from_ptr(MbObject::new_bytes(h[start..].to_vec())));
                        break;
                    }
                    if let Some(pos) = h[start..].windows(s.len()).position(|w| w == s.as_slice()) {
                        result.push(MbValue::from_ptr(MbObject::new_bytes(h[start..start + pos].to_vec())));
                        start += pos + s.len();
                        splits_done += 1;
                    } else {
                        result.push(MbValue::from_ptr(MbObject::new_bytes(h[start..].to_vec())));
                        break;
                    }
                }
                result
            } else {
                vec![haystack]
            };
            MbValue::from_ptr(MbObject::new_list(parts))
        } else {
            MbValue::from_ptr(MbObject::new_list(Vec::new()))
        }
    }
}

/// bytes.join(iterable) -> bytes
/// Joins an iterable of bytes objects with self as separator.
pub fn mb_bytes_join(sep: MbValue, parts: MbValue) -> MbValue {
    unsafe {
        let sep_data = as_bytes_cloned(sep).unwrap_or_default();
        let mut result = Vec::new();
        let mut first = true;
        // Collect items from list or iterable
        if let Some(ptr) = parts.as_ptr() {
            match &(*ptr).data {
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    for item in items.iter() {
                        if !first { result.extend_from_slice(&sep_data); }
                        if let Some(data) = as_bytes_cloned(*item) {
                            result.extend_from_slice(&data);
                        }
                        first = false;
                    }
                }
                ObjData::Tuple(ref items) => {
                    for item in items.iter() {
                        if !first { result.extend_from_slice(&sep_data); }
                        if let Some(data) = as_bytes_cloned(*item) {
                            result.extend_from_slice(&data);
                        }
                        first = false;
                    }
                }
                _ => {}
            }
        }
        // CPython: bytearray.join(...) returns a bytearray; bytes.join(...)
        // returns bytes. The result type follows the separator's type.
        let sep_is_bytearray = sep
            .as_ptr()
            .map(|ptr| matches!((*ptr).data, ObjData::ByteArray(_)))
            .unwrap_or(false);
        if sep_is_bytearray {
            MbValue::from_ptr(MbObject::new_bytearray(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytes(result))
        }
    }
}

// ── Strip methods ──

/// bytes.strip(chars) -> bytes
/// Strips leading and trailing bytes that appear in chars.
/// If chars is None, strips ASCII whitespace.
pub fn mb_bytes_strip(bytes: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, true, true);
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
        }
    }
}

/// bytes.lstrip(chars) -> bytes
/// Strips leading bytes that appear in chars.
pub fn mb_bytes_lstrip(bytes: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, true, false);
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
        }
    }
}

/// bytes.rstrip(chars) -> bytes
/// Strips trailing bytes that appear in chars.
pub fn mb_bytes_rstrip(bytes: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(bytes) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, false, true);
            MbValue::from_ptr(MbObject::new_bytes(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
        }
    }
}

/// bytearray.strip(chars) -> bytearray
pub fn mb_bytearray_strip(ba: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(ba) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, true, true);
            MbValue::from_ptr(MbObject::new_bytearray(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytearray(Vec::new()))
        }
    }
}

/// bytearray.lstrip(chars) -> bytearray
pub fn mb_bytearray_lstrip(ba: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(ba) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, true, false);
            MbValue::from_ptr(MbObject::new_bytearray(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytearray(Vec::new()))
        }
    }
}

/// bytearray.rstrip(chars) -> bytearray
pub fn mb_bytearray_rstrip(ba: MbValue, chars: MbValue) -> MbValue {
    unsafe {
        if let Some(data) = as_bytes_cloned(ba) {
            let strip_set = as_bytes_cloned(chars);
            let result = strip_bytes(&data, &strip_set, false, true);
            MbValue::from_ptr(MbObject::new_bytearray(result))
        } else {
            MbValue::from_ptr(MbObject::new_bytearray(Vec::new()))
        }
    }
}

/// Internal helper: strip bytes from front/back based on a set.
/// If strip_set is None or empty, strips ASCII whitespace.
fn strip_bytes(data: &[u8], strip_set: &Option<Vec<u8>>, left: bool, right: bool) -> Vec<u8> {
    let should_strip = |b: &u8| -> bool {
        match strip_set {
            Some(ref set) if !set.is_empty() => set.contains(b),
            _ => b.is_ascii_whitespace(),
        }
    };

    let start = if left {
        data.iter().position(|b| !should_strip(b)).unwrap_or(data.len())
    } else {
        0
    };

    let end = if right {
        data.iter().rposition(|b| !should_strip(b)).map(|p| p + 1).unwrap_or(start)
    } else {
        data.len()
    };

    if start >= end {
        Vec::new()
    } else {
        data[start..end].to_vec()
    }
}

// ── Method Dispatch ──

/// Dispatch a method call on a bytes/bytearray object.
pub fn dispatch_bytes_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    let items = lock.read().unwrap();
                    return items.get(i).copied().unwrap_or(MbValue::none());
                }
            }
            MbValue::none()
        }
    };
    let argc = || -> usize {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
            0
        }
    };
    match name {
        "decode" => {
            // `b.decode()` / `b.decode("ascii")` / `b.decode("ascii", "ignore")` /
            // `b.decode("ascii", errors="replace")` — accept positional and a
            // trailing kwargs Dict.
            let mut encoding = MbValue::none();
            let mut errors = MbValue::none();
            let last_is_kwargs = if argc() > 0 {
                unsafe {
                    if let Some(ptr) = arg(argc() - 1).as_ptr() {
                        matches!((*ptr).data, ObjData::Dict(_))
                    } else { false }
                }
            } else { false };
            let positional_count = if last_is_kwargs { argc() - 1 } else { argc() };
            if positional_count >= 1 { encoding = arg(0); }
            if positional_count >= 2 { errors = arg(1); }
            if last_is_kwargs {
                unsafe {
                    if let Some(ptr) = arg(argc() - 1).as_ptr() {
                        if let ObjData::Dict(ref lock) = (*ptr).data {
                            let guard = lock.read().unwrap();
                            if encoding.is_none() {
                                if let Some(&v) = guard.get(&super::dict_ops::DictKey::Str("encoding".into())) {
                                    encoding = v;
                                }
                            }
                            if errors.is_none() {
                                if let Some(&v) = guard.get(&super::dict_ops::DictKey::Str("errors".into())) {
                                    errors = v;
                                }
                            }
                        }
                    }
                }
            }
            mb_bytes_decode_with(receiver, encoding, errors)
        }
        "hex" => mb_bytes_hex_with_sep(receiver, arg(0), arg(1)),
        "find" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_find_range(receiver, arg(0), s, e)
        }
        "rfind" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_rfind(receiver, arg(0), s, e)
        }
        "count" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_count_range(receiver, arg(0), s, e)
        }
        "startswith" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_startswith_range(receiver, arg(0), s, e)
        }
        "endswith" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_endswith_range(receiver, arg(0), s, e)
        }
        "replace" => {
            let count = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_replace_count(receiver, arg(0), arg(1), count)
        }
        "split" => {
            let maxsplit = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_bytes_split_max(receiver, arg(0), maxsplit)
        }
        "join" => mb_bytes_join(receiver, arg(0)),
        "strip" => {
            let is_bytearray = receiver.as_ptr().map(|ptr| unsafe {
                matches!((*ptr).data, ObjData::ByteArray(_))
            }).unwrap_or(false);
            if is_bytearray {
                mb_bytearray_strip(receiver, arg(0))
            } else {
                mb_bytes_strip(receiver, arg(0))
            }
        }
        "lstrip" => {
            let is_bytearray = receiver.as_ptr().map(|ptr| unsafe {
                matches!((*ptr).data, ObjData::ByteArray(_))
            }).unwrap_or(false);
            if is_bytearray {
                mb_bytearray_lstrip(receiver, arg(0))
            } else {
                mb_bytes_lstrip(receiver, arg(0))
            }
        }
        "rstrip" => {
            let is_bytearray = receiver.as_ptr().map(|ptr| unsafe {
                matches!((*ptr).data, ObjData::ByteArray(_))
            }).unwrap_or(false);
            if is_bytearray {
                mb_bytearray_rstrip(receiver, arg(0))
            } else {
                mb_bytes_rstrip(receiver, arg(0))
            }
        }
        // ByteArray-specific
        "append" => { mb_bytearray_append(receiver, arg(0)); MbValue::none() }
        "extend" => { mb_bytearray_extend(receiver, arg(0)); MbValue::none() }
        "clear" => { mb_bytearray_clear(receiver); MbValue::none() }
        "pop" => mb_bytearray_pop(receiver),
        "reverse" => { mb_bytearray_reverse(receiver); MbValue::none() }
        "partition" => mb_bytes_partition(receiver, arg(0)),
        "rpartition" => mb_bytes_rpartition(receiver, arg(0)),
        "index" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_index(receiver, arg(0), s, e)
        }
        "rindex" => {
            let s = if argc() > 1 { arg(1) } else { MbValue::none() };
            let e = if argc() > 2 { arg(2) } else { MbValue::none() };
            mb_bytes_rindex(receiver, arg(0), s, e)
        }
        "center" => mb_bytes_center(receiver, arg(0), arg(1)),
        "ljust" => mb_bytes_ljust(receiver, arg(0), arg(1)),
        "rjust" => mb_bytes_rjust(receiver, arg(0), arg(1)),
        "translate" => mb_bytes_translate(receiver, arg(0), arg(1)),
        "rsplit" => {
            let maxsplit = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_bytes_rsplit(receiver, arg(0), maxsplit)
        }
        "splitlines" => mb_bytes_splitlines(receiver, arg(0)),
        "capitalize" => mb_bytes_via_str(receiver, |s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase().collect::<String>(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        }),
        "title" => mb_bytes_via_str(receiver, |s| {
            let mut out = String::with_capacity(s.len());
            let mut new_word = true;
            for c in s.chars() {
                if c.is_alphabetic() {
                    if new_word { out.extend(c.to_uppercase()); } else { out.extend(c.to_lowercase()); }
                    new_word = false;
                } else {
                    out.push(c);
                    new_word = true;
                }
            }
            out
        }),
        "swapcase" => mb_bytes_via_str(receiver, |s| {
            s.chars().map(|c| {
                if c.is_uppercase() { c.to_lowercase().collect::<String>() }
                else if c.is_lowercase() { c.to_uppercase().collect::<String>() }
                else { c.to_string() }
            }).collect()
        }),
        "casefold" => mb_bytes_via_str(receiver, |s| s.to_lowercase()),
        "lower" => mb_bytes_via_str(receiver, |s| s.to_lowercase()),
        "upper" => mb_bytes_via_str(receiver, |s| s.to_uppercase()),
        "zfill" => mb_bytes_zfill(receiver, arg(0)),
        "expandtabs" => mb_bytes_expandtabs(receiver, arg(0)),
        "isalpha" => mb_bytes_predicate(receiver, |b| b.is_ascii_alphabetic()),
        "isdigit" => mb_bytes_predicate(receiver, |b| b.is_ascii_digit()),
        "isalnum" => mb_bytes_predicate(receiver, |b| b.is_ascii_alphanumeric()),
        "isspace" => mb_bytes_predicate(receiver, |b| b.is_ascii_whitespace()),
        "isupper" => mb_bytes_alpha_pred(receiver, |b| b.is_ascii_uppercase()),
        "islower" => mb_bytes_alpha_pred(receiver, |b| b.is_ascii_lowercase()),
        "isascii" => mb_bytes_predicate(receiver, |b| b.is_ascii()),
        "isdecimal" | "isnumeric" => mb_bytes_predicate(receiver, |b| b.is_ascii_digit()),
        "istitle" => MbValue::from_bool(false),
        "removeprefix" => mb_bytes_removeprefix(receiver, arg(0)),
        "removesuffix" => mb_bytes_removesuffix(receiver, arg(0)),
        "copy" => {
            let cloned: Option<Vec<u8>> = unsafe { as_bytes_cloned(receiver) };
            cloned.map(|d| MbValue::from_ptr(MbObject::new_bytes(d))).unwrap_or_else(MbValue::none)
        }
        "__alloc__" => MbValue::from_int(0),
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    format!("'bytes' object has no attribute '{name}'"),
                )),
            );
            MbValue::none()
        }
    }
}

// ── Helpers used by dispatch_bytes_method extension ──

fn extract_bytes(val: MbValue) -> Option<Vec<u8>> {
    unsafe { as_bytes_cloned(val) }
}

fn bytes_to_value(receiver: MbValue, data: Vec<u8>) -> MbValue {
    let is_bytearray = receiver.as_ptr().map(|ptr| unsafe {
        matches!((*ptr).data, ObjData::ByteArray(_))
    }).unwrap_or(false);
    if is_bytearray {
        MbValue::from_ptr(MbObject::new_bytearray(data))
    } else {
        MbValue::from_ptr(MbObject::new_bytes(data))
    }
}

fn fill_byte(arg: MbValue) -> u8 {
    if let Some(v) = extract_bytes(arg) {
        return *v.first().unwrap_or(&b' ');
    }
    if let Some(i) = arg.as_int() {
        return (i & 0xFF) as u8;
    }
    b' '
}

pub fn mb_bytes_partition(receiver: MbValue, sep: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let sep_b = extract_bytes(sep).unwrap_or_default();
    let make = |d: Vec<u8>| bytes_to_value(receiver, d);
    if sep_b.is_empty() {
        return MbValue::from_ptr(MbObject::new_tuple(vec![
            make(data),
            make(Vec::new()),
            make(Vec::new()),
        ]));
    }
    if let Some(idx) = data.windows(sep_b.len()).position(|w| w == sep_b.as_slice()) {
        let before = data[..idx].to_vec();
        let after = data[idx + sep_b.len()..].to_vec();
        MbValue::from_ptr(MbObject::new_tuple(vec![make(before), make(sep_b), make(after)]))
    } else {
        MbValue::from_ptr(MbObject::new_tuple(vec![make(data), make(Vec::new()), make(Vec::new())]))
    }
}

pub fn mb_bytes_rpartition(receiver: MbValue, sep: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let sep_b = extract_bytes(sep).unwrap_or_default();
    let make = |d: Vec<u8>| bytes_to_value(receiver, d);
    if sep_b.is_empty() {
        return MbValue::from_ptr(MbObject::new_tuple(vec![
            make(Vec::new()),
            make(Vec::new()),
            make(data),
        ]));
    }
    if let Some(idx) = data.windows(sep_b.len()).rposition(|w| w == sep_b.as_slice()) {
        let before = data[..idx].to_vec();
        let after = data[idx + sep_b.len()..].to_vec();
        MbValue::from_ptr(MbObject::new_tuple(vec![make(before), make(sep_b), make(after)]))
    } else {
        MbValue::from_ptr(MbObject::new_tuple(vec![make(Vec::new()), make(Vec::new()), make(data)]))
    }
}

pub fn mb_bytes_index(receiver: MbValue, needle: MbValue, start: MbValue, end: MbValue) -> MbValue {
    let r = mb_bytes_find_range(receiver, needle, start, end);
    if r.as_int() == Some(-1) {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str("subsection not found".to_string())),
        );
    }
    r
}

pub fn mb_bytes_rindex(receiver: MbValue, needle: MbValue, start: MbValue, end: MbValue) -> MbValue {
    let r = mb_bytes_rfind(receiver, needle, start, end);
    if r.as_int() == Some(-1) {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str("subsection not found".to_string())),
        );
    }
    r
}

pub fn mb_bytes_center(receiver: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let w = width.as_int().unwrap_or(0).max(0) as usize;
    if data.len() >= w { return bytes_to_value(receiver, data); }
    let pad = w - data.len();
    let left = pad / 2;
    let right = pad - left;
    let f = fill_byte(fill);
    let mut out = Vec::with_capacity(w);
    out.extend(std::iter::repeat_n(f, left));
    out.extend_from_slice(&data);
    out.extend(std::iter::repeat_n(f, right));
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_ljust(receiver: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let w = width.as_int().unwrap_or(0).max(0) as usize;
    if data.len() >= w { return bytes_to_value(receiver, data); }
    let f = fill_byte(fill);
    let mut out = data.clone();
    out.extend(std::iter::repeat_n(f, w - data.len()));
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_rjust(receiver: MbValue, width: MbValue, fill: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let w = width.as_int().unwrap_or(0).max(0) as usize;
    if data.len() >= w { return bytes_to_value(receiver, data); }
    let f = fill_byte(fill);
    let pad = w - data.len();
    let mut out = Vec::with_capacity(w);
    out.extend(std::iter::repeat_n(f, pad));
    out.extend_from_slice(&data);
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_translate(receiver: MbValue, table: MbValue, delete: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let table_b = extract_bytes(table);
    let delete_b = extract_bytes(delete).unwrap_or_default();
    let mut out = Vec::with_capacity(data.len());
    for &b in &data {
        if delete_b.contains(&b) { continue; }
        let mapped = match &table_b {
            Some(t) if t.len() == 256 => t[b as usize],
            _ => b,
        };
        out.push(mapped);
    }
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_rsplit(receiver: MbValue, sep: MbValue, _maxsplit: MbValue) -> MbValue {
    // Best-effort: full split, reverse order semantics not honored for maxsplit.
    let result = mb_bytes_split(receiver, sep);
    result
}

pub fn mb_bytes_splitlines(receiver: MbValue, _keepends: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let mut lines: Vec<MbValue> = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\n' {
            lines.push(bytes_to_value(receiver, data[start..i].to_vec()));
            start = i + 1;
        } else if data[i] == b'\r' {
            lines.push(bytes_to_value(receiver, data[start..i].to_vec()));
            if i + 1 < data.len() && data[i+1] == b'\n' { i += 1; }
            start = i + 1;
        }
        i += 1;
    }
    if start < data.len() {
        lines.push(bytes_to_value(receiver, data[start..].to_vec()));
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

pub fn mb_bytes_via_str<F: Fn(&str) -> String>(receiver: MbValue, f: F) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let s = String::from_utf8_lossy(&data);
    let out = f(&s);
    bytes_to_value(receiver, out.into_bytes())
}

pub fn mb_bytes_zfill(receiver: MbValue, width: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let w = width.as_int().unwrap_or(0).max(0) as usize;
    if data.len() >= w { return bytes_to_value(receiver, data); }
    let pad = w - data.len();
    let mut out = Vec::with_capacity(w);
    let (sign, body) = match data.first() {
        Some(b) if *b == b'+' || *b == b'-' => (Some(*b), &data[1..]),
        _ => (None, data.as_slice()),
    };
    if let Some(s) = sign { out.push(s); }
    out.extend(std::iter::repeat_n(b'0', pad));
    out.extend_from_slice(body);
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_expandtabs(receiver: MbValue, tabsize: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let ts = tabsize.as_int().unwrap_or(8).max(0) as usize;
    let mut out = Vec::with_capacity(data.len());
    let mut col = 0usize;
    for &b in &data {
        if b == b'\t' {
            let pad = if ts == 0 { 0 } else { ts - (col % ts) };
            out.extend(std::iter::repeat_n(b' ', pad));
            col += pad;
        } else {
            out.push(b);
            if b == b'\n' || b == b'\r' { col = 0; } else { col += 1; }
        }
    }
    bytes_to_value(receiver, out)
}

pub fn mb_bytes_predicate<F: Fn(u8) -> bool>(receiver: MbValue, f: F) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    if data.is_empty() { return MbValue::from_bool(false); }
    MbValue::from_bool(data.iter().all(|&b| f(b)))
}

pub fn mb_bytes_alpha_pred<F: Fn(u8) -> bool>(receiver: MbValue, f: F) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let any_alpha = data.iter().any(|b| b.is_ascii_alphabetic());
    if !any_alpha { return MbValue::from_bool(false); }
    MbValue::from_bool(data.iter().filter(|b| b.is_ascii_alphabetic()).all(|&b| f(b)))
}

pub fn mb_bytes_removeprefix(receiver: MbValue, prefix: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let p = extract_bytes(prefix).unwrap_or_default();
    if data.starts_with(&p) {
        bytes_to_value(receiver, data[p.len()..].to_vec())
    } else {
        bytes_to_value(receiver, data)
    }
}

pub fn mb_bytes_removesuffix(receiver: MbValue, suffix: MbValue) -> MbValue {
    let data = extract_bytes(receiver).unwrap_or_default();
    let s = extract_bytes(suffix).unwrap_or_default();
    if !s.is_empty() && data.ends_with(&s) {
        bytes_to_value(receiver, data[..data.len() - s.len()].to_vec())
    } else {
        bytes_to_value(receiver, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bytes(data: &[u8]) -> MbValue {
        MbValue::from_ptr(MbObject::new_bytes(data.to_vec()))
    }

    fn make_bytearray(data: &[u8]) -> MbValue {
        MbValue::from_ptr(MbObject::new_bytearray(data.to_vec()))
    }

    // ── bytes creation ──

    #[test]
    fn test_bytes_new_from_none() {
        let b = mb_bytes_new(MbValue::none());
        unsafe {
            assert_eq!(as_bytes_cloned(b).unwrap(), Vec::<u8>::new());
        }
    }

    #[test]
    fn test_bytes_new_from_int() {
        let b = mb_bytes_new(MbValue::from_int(5));
        unsafe {
            assert_eq!(as_bytes_cloned(b).unwrap(), vec![0u8; 5]);
        }
    }

    #[test]
    fn test_bytes_new_from_string() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".into()));
        let b = mb_bytes_new(s);
        unsafe {
            assert_eq!(as_bytes_cloned(b).unwrap(), b"hello".to_vec());
        }
    }

    #[test]
    fn test_bytes_new_from_bytes() {
        let src = make_bytes(b"data");
        let b = mb_bytes_new(src);
        unsafe {
            assert_eq!(as_bytes_cloned(b).unwrap(), b"data".to_vec());
        }
    }

    #[test]
    fn test_bytes_new_from_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(72), MbValue::from_int(105),
        ]));
        let b = mb_bytes_new(list);
        assert_eq!(mb_bytes_len(b).as_int(), Some(2));
        assert_eq!(mb_bytes_getitem(b, MbValue::from_int(0)).as_int(), Some(72));
    }

    /// Regression for #2103: `bytes(reversed(range(N)))` used to produce a
    /// 2^32-byte object because the iterator handle ID (NaN-boxed int,
    /// base 2^32) was accepted by `as_int()` and treated as a zero-fill
    /// count. The fix probes ITERATORS first and drains the handle.
    ///
    /// Sizes are limited to the byte-valid range (`0..=255`): per CPython
    /// 3.12, `bytes(reversed(range(N)))` raises `ValueError: bytes must be
    /// in range(0, 256)` once any element exceeds 255 (confirmed against
    /// python3.12). The original `1024`/`65536` cases predate constructor
    /// element validation and asserted non-CPython lengths; the #2103
    /// regression (handle drained item-by-item, not misread as a count) is
    /// still fully exercised by a non-empty iterator that yields its items.
    #[test]
    fn test_bytes_new_from_reversed_range_handle_2103() {
        for n in [0i64, 1, 64, 256] {
            let range_iter = super::super::iter::mb_range_iter(
                MbValue::from_int(0),
                MbValue::from_int(n),
                MbValue::from_int(1),
            );
            let rev = super::super::iter::mb_reversed(range_iter);
            let b = mb_bytes_new(rev);
            assert_eq!(
                mb_bytes_len(b).as_int(),
                Some(n),
                "bytes(reversed(range({n}))) length must equal {n}"
            );
        }
        // n>255 yields values outside 0..=255: CPython 3.12 raises
        // ValueError("byte must be in range(0, 256)") (python3.12-confirmed) —
        // it does NOT produce a long bytes. Assert the rejection so the
        // out-of-range path stays covered (was previously a stale len==1024 assert).
        super::super::exception::mb_clear_exception();
        let big = super::super::iter::mb_reversed(super::super::iter::mb_range_iter(
            MbValue::from_int(0), MbValue::from_int(1024), MbValue::from_int(1),
        ));
        let r = mb_bytes_new(big);
        assert!(r.is_none(), "bytes() of out-of-range values must not return a value");
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("ValueError"),
        );
        super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_bytearray_new_from_reversed_range_handle_2103() {
        let range_iter = super::super::iter::mb_range_iter(
            MbValue::from_int(0),
            MbValue::from_int(8),
            MbValue::from_int(1),
        );
        let rev = super::super::iter::mb_reversed(range_iter);
        let b = mb_bytearray_new(rev);
        assert_eq!(mb_bytes_len(b).as_int(), Some(8));
        // Contents are reversed: index 0 should be 7.
        assert_eq!(mb_bytes_getitem(b, MbValue::from_int(0)).as_int(), Some(7));
        assert_eq!(mb_bytes_getitem(b, MbValue::from_int(7)).as_int(), Some(0));
    }

    // ── bytearray creation ──

    #[test]
    fn test_bytearray_new_from_none() {
        let ba = mb_bytearray_new(MbValue::none());
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), Vec::<u8>::new());
        }
    }

    #[test]
    fn test_bytearray_new_from_int() {
        let ba = mb_bytearray_new(MbValue::from_int(3));
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), vec![0u8; 3]);
        }
    }

    #[test]
    fn test_bytearray_new_from_string() {
        let s = MbValue::from_ptr(MbObject::new_str("hi".to_string()));
        let ba = mb_bytearray_new(s);
        assert_eq!(mb_bytes_len(ba).as_int(), Some(2));
    }

    #[test]
    fn test_bytearray_new_from_bytes() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![10, 20]));
        let ba = mb_bytearray_new(b);
        assert_eq!(mb_bytes_len(ba).as_int(), Some(2));
    }

    #[test]
    fn test_bytearray_new_from_bytearray() {
        let src = make_bytearray(b"abc");
        let ba = mb_bytearray_new(src);
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), b"abc".to_vec());
        }
    }

    #[test]
    fn test_bytearray_new_from_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(65), MbValue::from_int(66),
        ]));
        let ba = mb_bytearray_new(list);
        assert_eq!(mb_bytes_len(ba).as_int(), Some(2));
    }

    // ── getitem ──

    #[test]
    fn test_bytes_getitem_positive() {
        let b = make_bytes(b"hello");
        let v = mb_bytes_getitem(b, MbValue::from_int(1));
        assert_eq!(v.as_int(), Some(b'e' as i64));
    }

    #[test]
    fn test_bytes_getitem_negative() {
        let b = make_bytes(b"hello");
        let v = mb_bytes_getitem(b, MbValue::from_int(-1));
        assert_eq!(v.as_int(), Some(b'o' as i64));
    }

    #[test]
    fn test_bytes_getitem_out_of_range() {
        let b = make_bytes(b"hi");
        let v = mb_bytes_getitem(b, MbValue::from_int(10));
        assert!(v.is_none());
    }

    // ── len ──

    #[test]
    fn test_bytes_len() {
        let b = make_bytes(b"hello");
        assert_eq!(mb_bytes_len(b).as_int(), Some(5));
    }

    #[test]
    fn test_bytes_len_empty() {
        let b = make_bytes(b"");
        assert_eq!(mb_bytes_len(b).as_int(), Some(0));
    }

    #[test]
    fn test_bytes_len_non_bytes() {
        assert_eq!(mb_bytes_len(MbValue::from_int(0)).as_int(), Some(0));
    }

    // ── decode ──

    #[test]
    fn test_bytes_decode() {
        let b = make_bytes(b"hello");
        let s = mb_bytes_decode(b, MbValue::none());
        unsafe {
            if let Some(ptr) = s.as_ptr() {
                if let ObjData::Str(ref st) = (*ptr).data {
                    assert_eq!(st, "hello");
                    return;
                }
            }
            panic!("expected string");
        }
    }

    // ── hex ──

    #[test]
    fn test_bytes_hex() {
        let b = make_bytes(&[0xDE, 0xAD]);
        let h = mb_bytes_hex(b);
        unsafe {
            if let Some(ptr) = h.as_ptr() {
                if let ObjData::Str(ref st) = (*ptr).data {
                    assert_eq!(st, "dead");
                    return;
                }
            }
            panic!("expected hex string");
        }
    }

    // ── find ──

    #[test]
    fn test_bytes_find_found() {
        let h = make_bytes(b"hello world");
        let n = make_bytes(b"world");
        assert_eq!(mb_bytes_find(h, n).as_int(), Some(6));
    }

    #[test]
    fn test_bytes_find_not_found() {
        let h = make_bytes(b"hello");
        let n = make_bytes(b"xyz");
        assert_eq!(mb_bytes_find(h, n).as_int(), Some(-1));
    }

    // ── concat ──

    #[test]
    fn test_bytes_concat() {
        let a = make_bytes(b"hel");
        let b = make_bytes(b"lo");
        let c = mb_bytes_concat(a, b);
        unsafe {
            assert_eq!(as_bytes_cloned(c).unwrap(), b"hello".to_vec());
        }
    }

    #[test]
    fn test_bytes_concat_non_bytes() {
        assert!(mb_bytes_concat(MbValue::from_int(0), MbValue::from_int(0)).is_none());
    }

    // ── contains ──

    #[test]
    fn test_bytes_contains_int() {
        let b = make_bytes(b"abc");
        assert_eq!(mb_bytes_contains(b, MbValue::from_int(b'a' as i64)).as_bool(), Some(true));
    }

    #[test]
    fn test_bytes_contains_sub() {
        let b = make_bytes(b"hello world");
        let sub = make_bytes(b"llo");
        assert_eq!(mb_bytes_contains(b, sub).as_bool(), Some(true));
    }

    #[test]
    fn test_bytes_contains_not_found() {
        let b = make_bytes(b"abc");
        assert_eq!(mb_bytes_contains(b, MbValue::from_int(b'z' as i64)).as_bool(), Some(false));
    }

    // ── bytearray mutations ──

    #[test]
    fn test_bytearray_append() {
        let ba = make_bytearray(b"ab");
        mb_bytearray_append(ba, MbValue::from_int(b'c' as i64));
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), b"abc".to_vec());
        }
    }

    #[test]
    fn test_bytearray_extend() {
        let ba = make_bytearray(b"ab");
        let ext = make_bytes(b"cd");
        mb_bytearray_extend(ba, ext);
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), b"abcd".to_vec());
        }
    }

    #[test]
    fn test_bytearray_clear() {
        let ba = make_bytearray(b"data");
        mb_bytearray_clear(ba);
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), Vec::<u8>::new());
        }
    }

    #[test]
    fn test_bytearray_pop() {
        let ba = make_bytearray(b"abc");
        let v = mb_bytearray_pop(ba);
        assert_eq!(v.as_int(), Some(b'c' as i64));
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), b"ab".to_vec());
        }
    }

    #[test]
    fn test_bytearray_pop_empty() {
        let ba = make_bytearray(b"");
        let v = mb_bytearray_pop(ba);
        assert!(v.is_none());
    }

    #[test]
    fn test_bytearray_reverse() {
        let ba = make_bytearray(b"abc");
        mb_bytearray_reverse(ba);
        unsafe {
            assert_eq!(as_bytes_cloned(ba).unwrap(), b"cba".to_vec());
        }
    }

    // ── dispatch ──

    #[test]
    fn test_dispatch_decode() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![65]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::none()]));
        let r = dispatch_bytes_method("decode", b, args);
        assert!(r.is_ptr());
    }

    #[test]
    fn test_dispatch_hex() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![0xab]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let r = dispatch_bytes_method("hex", b, args);
        assert!(r.is_ptr());
    }

    #[test]
    fn test_dispatch_find() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2, 3]));
        let needle = MbValue::from_ptr(MbObject::new_bytes(vec![2, 3]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![needle]));
        assert_eq!(dispatch_bytes_method("find", b, args).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_bytearray_append() {
        let ba = MbValue::from_ptr(MbObject::new_bytearray(vec![]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(10)]));
        dispatch_bytes_method("append", ba, args);
        assert_eq!(mb_bytes_len(ba).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_bytearray_pop() {
        let ba = MbValue::from_ptr(MbObject::new_bytearray(vec![5]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(dispatch_bytes_method("pop", ba, args).as_int(), Some(5));
    }

    #[test]
    fn test_dispatch_unknown() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert!(dispatch_bytes_method("nonexistent", b, args).is_none());
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_bytes_indexing_returns_int() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![104, 101, 108, 108, 111]));
        assert_eq!(mb_bytes_getitem(b, MbValue::from_int(0)).as_int(), Some(104));
        assert_eq!(mb_bytes_getitem(b, MbValue::from_int(-1)).as_int(), Some(111));
    }

    #[test]
    fn test_py312_bytes_fromhex_roundtrip() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![0xff, 0xab]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let hex_str = dispatch_bytes_method("hex", b, args);
        assert!(hex_str.is_ptr());
    }

    #[test]
    fn test_py312_bytes_decode_utf8() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![104, 101, 108, 108, 111]));
        let enc = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![enc]));
        let result = dispatch_bytes_method("decode", b, args);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_py312_bytes_count() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![108, 108, 101]));
        let needle = MbValue::from_ptr(MbObject::new_bytes(vec![108]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![needle]));
        assert_eq!(dispatch_bytes_method("count", b, args).as_int(), Some(2));
    }

    #[test]
    fn test_py312_bytes_startswith() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![104, 101, 108]));
        let prefix = MbValue::from_ptr(MbObject::new_bytes(vec![104, 101]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![prefix]));
        assert_eq!(dispatch_bytes_method("startswith", b, args).as_bool(), Some(true));
    }

    #[test]
    fn test_py312_bytes_endswith() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![104, 101, 108]));
        let suffix = MbValue::from_ptr(MbObject::new_bytes(vec![101, 108]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![suffix]));
        assert_eq!(dispatch_bytes_method("endswith", b, args).as_bool(), Some(true));
    }

    #[test]
    fn test_py312_bytearray_extend() {
        let ba = MbValue::from_ptr(MbObject::new_bytearray(vec![1, 2]));
        let ext = MbValue::from_ptr(MbObject::new_bytes(vec![3, 4]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![ext]));
        dispatch_bytes_method("extend", ba, args);
        assert_eq!(mb_bytes_len(ba).as_int(), Some(4));
    }

    #[test]
    fn test_py312_bytes_replace() {
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2, 1, 3]));
        let old = MbValue::from_ptr(MbObject::new_bytes(vec![1]));
        let new_b = MbValue::from_ptr(MbObject::new_bytes(vec![9]));
        let args = MbValue::from_ptr(MbObject::new_list(vec![old, new_b]));
        let result = dispatch_bytes_method("replace", b, args);
        assert_eq!(mb_bytes_len(result).as_int(), Some(4));
        assert_eq!(mb_bytes_getitem(result, MbValue::from_int(0)).as_int(), Some(9));
    }

    // ── R3: Scenario-matching tests from spec ──

    // R3.1 — b"hello world".replace(b"world", b"mamba") == b"hello mamba"
    #[test]
    fn test_bytes_replace_scenario_hello_world() {
        let haystack = make_bytes(b"hello world");
        let old = make_bytes(b"world");
        let new_b = make_bytes(b"mamba");
        let result = mb_bytes_replace(haystack, old, new_b);
        unsafe {
            assert_eq!(as_bytes_cloned(result).unwrap(), b"hello mamba".to_vec());
        }
    }

    // R3.3 — b"hello".startswith(b"he") == True
    #[test]
    fn test_bytes_startswith_scenario() {
        let b = make_bytes(b"hello");
        let prefix = make_bytes(b"he");
        assert_eq!(mb_bytes_startswith(b, prefix).as_bool(), Some(true));
    }

    // R3.3 — b"hello".endswith(b"lo") == True
    #[test]
    fn test_bytes_endswith_scenario() {
        let b = make_bytes(b"hello");
        let suffix = make_bytes(b"lo");
        assert_eq!(mb_bytes_endswith(b, suffix).as_bool(), Some(true));
    }

    // R3.3 — negative cases
    #[test]
    fn test_bytes_startswith_false() {
        let b = make_bytes(b"hello");
        let prefix = make_bytes(b"world");
        assert_eq!(mb_bytes_startswith(b, prefix).as_bool(), Some(false));
    }

    #[test]
    fn test_bytes_endswith_false() {
        let b = make_bytes(b"hello");
        let suffix = make_bytes(b"he");
        assert_eq!(mb_bytes_endswith(b, suffix).as_bool(), Some(false));
    }

    // R3.4 — bytearray versions (as_bytes_cloned handles both Bytes and ByteArray)
    #[test]
    fn test_bytearray_startswith() {
        let ba = make_bytearray(b"hello");
        let prefix = make_bytes(b"he");
        assert_eq!(mb_bytes_startswith(ba, prefix).as_bool(), Some(true));
    }

    #[test]
    fn test_bytearray_endswith() {
        let ba = make_bytearray(b"hello");
        let suffix = make_bytes(b"lo");
        assert_eq!(mb_bytes_endswith(ba, suffix).as_bool(), Some(true));
    }

    #[test]
    fn test_bytearray_replace_scenario() {
        let ba = make_bytearray(b"hello world");
        let old = make_bytes(b"world");
        let new_b = make_bytes(b"mamba");
        let result = mb_bytes_replace(ba, old, new_b);
        unsafe {
            assert_eq!(as_bytes_cloned(result).unwrap(), b"hello mamba".to_vec());
        }
    }

    // R3.1 — replace with empty old (insert new before each byte and at end)
    #[test]
    fn test_bytes_replace_empty_old() {
        let b = make_bytes(b"ab");
        let old = make_bytes(b"");
        let new_b = make_bytes(b"X");
        let result = mb_bytes_replace(b, old, new_b);
        // Expected: XaXbX  (insert X before each byte, and at end)
        unsafe {
            assert_eq!(as_bytes_cloned(result).unwrap(), b"XaXbX".to_vec());
        }
    }

    // R3.1 — replace with no matches returns original
    #[test]
    fn test_bytes_replace_no_match() {
        let b = make_bytes(b"hello");
        let old = make_bytes(b"xyz");
        let new_b = make_bytes(b"abc");
        let result = mb_bytes_replace(b, old, new_b);
        unsafe {
            assert_eq!(as_bytes_cloned(result).unwrap(), b"hello".to_vec());
        }
    }

    // Compile-time anchor that pins a representative subset of the 27 exported
    // mb_bytes_* / mb_bytearray_* symbols identified in the tick-167 spec-align
    // audit (runtime/bytes-ops.md). Takes each as a typed function pointer so
    // any silent deletion OR signature change fails the build — complements the
    // per-fn behavioral tests above by anchoring the R2/R3/R4 public contract.
    #[test]
    fn critical_api_surface_present() {
        let _: fn(MbValue, MbValue) -> MbValue = mb_bytes_decode;
        let _: fn(MbValue) -> MbValue = mb_bytes_hex;
        let _: fn(MbValue, MbValue) -> MbValue = mb_bytes_find;
        let _: fn(MbValue, MbValue) -> MbValue = mb_bytes_split;
        let _: fn(MbValue, MbValue) -> MbValue = mb_bytes_join;
        let _: fn(MbValue, MbValue) -> MbValue = mb_bytes_strip;
        let _: fn(MbValue, MbValue) = mb_bytearray_append;
        let _: fn(MbValue, MbValue) = mb_bytearray_extend;
        let _: fn(MbValue) -> MbValue = mb_bytearray_pop;
        let _: fn(MbValue) = mb_bytearray_reverse;
    }
}
