/// uu module for Mamba (#1261 long-tail).
///
/// Replaces the long_tail stub (every encode/decode/test call was a
/// no-op returning None) with a real uuencode/uudecode codec.
///
/// CPython's `uu` was deprecated in 3.11 and removed in 3.13, but
/// targets like pytest/Flask running on 3.12 still see code paths
/// that `import uu` (legacy mail/news fixtures). The mamba surface
/// covers the byte-level codec so `uu.encode_bytes` / `uu.decode_bytes`
/// (mamba-extension helpers) and the CPython-compatible `encode` /
/// `decode` filename-string forms produce real output.
///
/// The two public CPython entry points are:
///
///   - `encode(in_path, out_path, name=None, mode=None, backtick=False)`
///   - `decode(in_path, out_path=None, mode=None, quiet=False)`
///
/// Mamba doesn't yet route Python file objects (`io.BytesIO`, open
/// file handles) through native dispatchers reliably, so the encode/
/// decode entry points accept only filename strings. The bytes-level
/// helpers (`encode_bytes`, `decode_bytes`) cover the rest.

use std::collections::HashMap;
use std::fs;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => std::str::from_utf8(b).ok().map(str::to_string),
        _ => None,
    }
}

unsafe fn as_bytes(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Bytes(b) => Some(b.clone()),
        ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
        ObjData::Str(s) => Some(s.as_bytes().to_vec()),
        _ => None,
    }
}

/// Encode one line (up to 45 bytes) per CPython algorithm. With
/// `backtick=true`, encoded `0x20` (space) is rewritten as `0x60`
/// (backtick) for transports that mangle trailing spaces.
fn encode_line(data: &[u8], backtick: bool) -> Vec<u8> {
    let len = data.len().min(45);
    let mut out = Vec::with_capacity(2 + ((len + 2) / 3) * 4);
    let len_byte = (0x20 + len as u8).saturating_sub(if backtick && len == 0 { 0 } else { 0 });
    out.push(if backtick && len_byte == 0x20 { 0x60 } else { len_byte });
    let mut i = 0;
    while i < len {
        let b1 = data[i];
        let b2 = if i + 1 < len { data[i + 1] } else { 0 };
        let b3 = if i + 2 < len { data[i + 2] } else { 0 };
        let c1 = b1 >> 2;
        let c2 = ((b1 & 0x03) << 4) | (b2 >> 4);
        let c3 = ((b2 & 0x0F) << 2) | (b3 >> 6);
        let c4 = b3 & 0x3F;
        for c in [c1, c2, c3, c4] {
            let v = 0x20 + c;
            out.push(if backtick && v == 0x20 { 0x60 } else { v });
        }
        i += 3;
    }
    out.push(b'\n');
    out
}

/// Decode one uuencoded line. Returns the decoded bytes for that line.
/// Stops at the end-of-data marker (length byte == 0x20 or 0x60).
fn decode_line(line: &[u8]) -> Vec<u8> {
    if line.is_empty() { return Vec::new(); }
    // Strip carriage return/newline.
    let line: &[u8] = match line.iter().rposition(|&c| c != b'\n' && c != b'\r') {
        Some(end) => &line[..=end],
        None => return Vec::new(),
    };
    if line.is_empty() { return Vec::new(); }
    let len_byte = line[0];
    // 0x20 (space) or 0x60 (backtick) → length 0 / EOF marker.
    if len_byte == 0x60 { return Vec::new(); }
    let length = (len_byte.wrapping_sub(0x20) & 0x3F) as usize;
    if length == 0 { return Vec::new(); }
    let mut out = Vec::with_capacity(length);
    let mut i = 1;
    while i + 3 < line.len() && out.len() < length {
        let v: Vec<u8> = (0..4).map(|k| {
            let c = line[i + k];
            // Backtick → 0; otherwise (c - 0x20) & 0x3F.
            if c == b'`' { 0 } else { c.wrapping_sub(0x20) & 0x3F }
        }).collect();
        let b1 = (v[0] << 2) | (v[1] >> 4);
        let b2 = ((v[1] & 0x0F) << 4) | (v[2] >> 2);
        let b3 = ((v[2] & 0x03) << 6) | v[3];
        for &b in &[b1, b2, b3] {
            if out.len() < length { out.push(b); }
        }
        i += 4;
    }
    out
}

/// Full uuencoded stream: header + 45-byte-chunk lines + end marker.
pub(crate) fn encode_bytes_impl(data: &[u8], name: &str, mode: u32, backtick: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 4 / 3 + 64);
    out.extend_from_slice(format!("begin {:o} {}\n", mode & 0o777, name).as_bytes());
    let mut i = 0;
    while i < data.len() {
        let end = (i + 45).min(data.len());
        out.extend_from_slice(&encode_line(&data[i..end], backtick));
        i = end;
    }
    // End marker: zero-length line (rendered as just '\n' after the length byte).
    out.extend_from_slice(&encode_line(&[], backtick));
    out.extend_from_slice(b"end\n");
    out
}

/// Decode a full uuencoded stream. Tolerates leading garbage before
/// the `begin` header, optional CR/LF endings, and missing trailers.
pub(crate) fn decode_bytes_impl(data: &[u8]) -> Vec<u8> {
    // Find `begin ` header.
    let mut lines = data.split(|&b| b == b'\n').peekable();
    for line in lines.by_ref() {
        let trimmed = line.strip_suffix(b"\r").unwrap_or(line);
        if trimmed.starts_with(b"begin ") { break; }
    }
    let mut out = Vec::new();
    for line in lines {
        let trimmed = line.strip_suffix(b"\r").unwrap_or(line);
        if trimmed.is_empty() { continue; }
        if trimmed == b"end" { break; }
        // EOF marker: length byte 0x20 (space) or 0x60 (backtick) alone.
        if trimmed.len() == 1 && (trimmed[0] == b' ' || trimmed[0] == b'`') {
            continue;
        }
        let decoded = decode_line(trimmed);
        if decoded.is_empty() {
            // Likely the end-marker line (length byte was 0); keep scanning
            // until `end`.
            continue;
        }
        out.extend_from_slice(&decoded);
    }
    out
}

/// `uu.encode_bytes(data: bytes, name='-', mode=0o666, backtick=False) -> bytes`
/// Mamba-extension helper for in-memory encoding.
unsafe extern "C" fn dispatch_encode_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let data = args.first().copied().and_then(|v| as_bytes(v)).unwrap_or_default();
    let name = args.get(1).copied().and_then(|v| as_str(v)).unwrap_or_else(|| "-".to_string());
    let mode = args.get(2).copied().and_then(|v| v.as_int()).unwrap_or(0o666) as u32;
    let backtick = args.get(3).copied().and_then(|v| v.as_bool()).unwrap_or(false);
    let result = encode_bytes_impl(&data, &name, mode, backtick);
    MbValue::from_ptr(MbObject::new_bytes(result))
}

/// `uu.decode_bytes(data: bytes) -> bytes`. Mamba-extension helper.
unsafe extern "C" fn dispatch_decode_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let data = args.first().copied().and_then(|v| as_bytes(v)).unwrap_or_default();
    let result = decode_bytes_impl(&data);
    MbValue::from_ptr(MbObject::new_bytes(result))
}

/// `uu.encode(in_path, out_path, name=None, mode=None, backtick=False)`.
/// Accepts only filename strings — file-object support is gated on
/// bound-method dispatch.
unsafe extern "C" fn dispatch_encode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let in_path = match args.first().copied().and_then(|v| as_str(v)) {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let out_path = match args.get(1).copied().and_then(|v| as_str(v)) {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let name = args.get(2).copied().and_then(|v| as_str(v)).unwrap_or_else(|| {
        // Default to basename of in_path, matching CPython's behavior.
        std::path::Path::new(&in_path)
            .file_name()
            .and_then(|s| s.to_str()).unwrap_or("-").to_string()
    });
    let mode = args.get(3).copied().and_then(|v| v.as_int())
        .map(|v| v as u32)
        .unwrap_or(0o666);
    let backtick = args.get(4).copied().and_then(|v| v.as_bool()).unwrap_or(false);
    let data = match fs::read(&in_path) {
        Ok(b) => b,
        Err(_) => return MbValue::none(),
    };
    let encoded = encode_bytes_impl(&data, &name, mode, backtick);
    let _ = fs::write(&out_path, encoded);
    MbValue::none()
}

/// `uu.decode(in_path, out_path=None, mode=None, quiet=False)`.
/// Returns decoded bytes when out_path is None; otherwise writes to
/// out_path and returns None.
unsafe extern "C" fn dispatch_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let in_path = match args.first().copied().and_then(|v| as_str(v)) {
        Some(p) => p,
        None => return MbValue::none(),
    };
    let data = match fs::read(&in_path) {
        Ok(b) => b,
        Err(_) => return MbValue::none(),
    };
    let decoded = decode_bytes_impl(&data);
    if let Some(out_path) = args.get(1).copied().and_then(|v| as_str(v)) {
        let _ = fs::write(&out_path, decoded);
        MbValue::none()
    } else {
        MbValue::from_ptr(MbObject::new_bytes(decoded))
    }
}

/// `uu.test()` — no-op (CPython's `test()` is a command-line entry point).
unsafe extern "C" fn dispatch_test(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        ("encode",       dispatch_encode       as *const () as usize),
        ("decode",       dispatch_decode       as *const () as usize),
        ("test",         dispatch_test         as *const () as usize),
        ("encode_bytes", dispatch_encode_bytes as *const () as usize),
        ("decode_bytes", dispatch_decode_bytes as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    let shell = dispatch_class_shell as *const () as usize;
    attrs.insert("Error".into(), MbValue::from_func(shell));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers { set.insert(*addr as u64); }
        set.insert(shell as u64);
    });
    super::register_module("uu", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_empty() {
        let encoded = encode_bytes_impl(b"", "test.bin", 0o666, false);
        // Header + end-marker line + "end\n".
        assert!(encoded.starts_with(b"begin 666 test.bin\n"));
        assert!(encoded.ends_with(b"end\n"));
        let decoded = decode_bytes_impl(&encoded);
        assert_eq!(decoded, b"");
    }

    #[test]
    fn encode_decode_roundtrip_short() {
        let encoded = encode_bytes_impl(b"Hello, World!", "msg", 0o644, false);
        assert!(encoded.starts_with(b"begin 644 msg\n"));
        let decoded = decode_bytes_impl(&encoded);
        assert_eq!(decoded, b"Hello, World!");
    }

    #[test]
    fn encode_decode_roundtrip_multiline() {
        // 100 bytes spans 3 lines (45 + 45 + 10).
        let data: Vec<u8> = (0..100u8).collect();
        let encoded = encode_bytes_impl(&data, "buf", 0o600, false);
        let line_count = encoded.iter().filter(|&&b| b == b'\n').count();
        // header + 3 data lines + end-marker + end
        assert!(line_count >= 5, "expected >=5 newlines, got {line_count}");
        let decoded = decode_bytes_impl(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn encode_decode_roundtrip_backtick() {
        // The all-zero byte triggers the backtick substitution; check the
        // body lines (between begin/end header & trailer) carry no 0x20.
        let data = vec![0u8; 12];
        let encoded = encode_bytes_impl(&data, "zeros", 0o666, true);
        let s = String::from_utf8_lossy(&encoded);
        let mut lines = s.lines();
        let header = lines.next().unwrap_or("");
        assert!(header.starts_with("begin "), "header missing: {header}");
        for line in lines {
            if line == "end" || line.is_empty() { continue; }
            assert!(!line.contains(' '),
                "backtick mode must not emit 0x20 in body line: {line:?}");
        }
        let decoded = decode_bytes_impl(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn encode_decode_roundtrip_random_sizes() {
        for &n in &[0usize, 1, 2, 3, 44, 45, 46, 89, 90, 91, 1024] {
            let data: Vec<u8> = (0..n).map(|i| (i * 7 + 13) as u8).collect();
            let encoded = encode_bytes_impl(&data, "x", 0o666, false);
            let decoded = decode_bytes_impl(&encoded);
            assert_eq!(decoded, data, "roundtrip failed for n={n}");
        }
    }

    #[test]
    fn decode_tolerates_crlf() {
        let mut encoded = encode_bytes_impl(b"abc", "t", 0o644, false);
        // Convert LF → CRLF.
        let mut crlf = Vec::new();
        for &b in &encoded {
            if b == b'\n' { crlf.push(b'\r'); }
            crlf.push(b);
        }
        encoded = crlf;
        let decoded = decode_bytes_impl(&encoded);
        assert_eq!(decoded, b"abc");
    }

    #[test]
    fn decode_ignores_leading_garbage() {
        let core = encode_bytes_impl(b"data", "f", 0o644, false);
        let mut blob = b"some garbage line\nMIME-Version: 1.0\n".to_vec();
        blob.extend_from_slice(&core);
        let decoded = decode_bytes_impl(&blob);
        assert_eq!(decoded, b"data");
    }

    #[test]
    fn dispatch_encode_decode_bytes_module() {
        unsafe {
            let payload = MbValue::from_ptr(MbObject::new_bytes(b"Mamba".to_vec()));
            let name = MbValue::from_ptr(MbObject::new_str("m".into()));
            let args = [payload, name];
            let encoded_v = dispatch_encode_bytes(args.as_ptr(), 2);
            // Now decode it back.
            let decoded_v = dispatch_decode_bytes([encoded_v].as_ptr(), 1);
            let bytes = as_bytes(decoded_v).unwrap();
            assert_eq!(bytes, b"Mamba");
        }
    }
}
