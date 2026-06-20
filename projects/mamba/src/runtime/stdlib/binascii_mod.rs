use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// binascii module for Mamba (#1261).
///
/// Faithful reimplementation of CPython 3.12 `Modules/binascii.c` for the
/// hot codecs: hexlify/unhexlify (+ b2a_hex/a2b_hex aliases), b2a_base64 /
/// a2b_base64 (with `newline=` and `strict_mode=`), b2a_uu / a2b_uu (with
/// `backtick=`), b2a_qp / a2b_qp (with `header=`/`quotetabs=`/`istext=`),
/// crc32 and crc_hqx. The byte math is self-contained.
///
/// Decode-side helpers accept bytes/bytearray/memoryview/array('B') and ASCII
/// str (CPython binascii decodes both). All return values are `bytes`. On
/// malformed input the codecs raise `binascii.Error` — registered so that
/// `except binascii.Error`, `except ValueError`, and `except Exception` all
/// catch it (CPython's `binascii.Error` subclasses `ValueError`).
use std::collections::HashMap;

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

/// Pull owned bytes from a bytes-like value (bytes/bytearray/memoryview/array('B')).
unsafe fn as_bytes_like(val: MbValue) -> Option<Vec<u8>> {
    super::super::builtins::try_bytes_like(val)
}

/// Outcome of coercing a decode-side argument to bytes.
enum DecodeArg {
    Bytes(Vec<u8>),
    NonAsciiStr,
    NotBytesLike,
}

/// Decode-side helpers accept ASCII strings as CPython does; a str carrying a
/// non-ASCII code point is rejected with ValueError (mirrors CPython's
/// "string argument should contain only ASCII characters").
unsafe fn as_bytes_or_str(val: MbValue) -> DecodeArg {
    if let Some(ptr) = val.as_ptr() {
        if let ObjData::Str(s) = &(*ptr).data {
            if s.is_ascii() {
                return DecodeArg::Bytes(s.as_bytes().to_vec());
            }
            return DecodeArg::NonAsciiStr;
        }
    }
    match super::super::builtins::try_bytes_like(val) {
        Some(v) => DecodeArg::Bytes(v),
        None => DecodeArg::NotBytesLike,
    }
}

/// Resolve a decode-side argument, returning either the bytes or an error
/// `MbValue` (already raised). Used by the a2b_* dispatchers.
unsafe fn decode_arg_bytes(val: MbValue, what: &str) -> Result<Vec<u8>, MbValue> {
    match as_bytes_or_str(val) {
        DecodeArg::Bytes(v) => Ok(v),
        DecodeArg::NonAsciiStr => Err(raise_value_error(
            "string argument should contain only ASCII characters",
        )),
        DecodeArg::NotBytesLike => Err(raise_type_error(&format!("argument should be {}", what))),
    }
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_binascii_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("binascii.Error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn bytes_val(v: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(v))
}

/// Extract a keyword argument value by name from a trailing kwargs dict
/// (`{'header': True}`). The native call ABI flattens keyword arguments into
/// a trailing dict positional. Returns `None` when the slice carries no such
/// dict or the key is absent.
unsafe fn kwarg(extra: &[MbValue], key: &str) -> Option<MbValue> {
    for v in extra {
        if let Some(ptr) = v.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let dk = super::super::dict_ops::DictKey::Str(key.to_string());
                if let Some(found) = map.get(&dk) {
                    return Some(*found);
                }
            }
        }
    }
    None
}

/// True if the trailing kwargs dict carries any non-string key (e.g. the
/// `a2b_qp(b'', **{1: 1})` test). CPython rejects non-string keywords with
/// TypeError.
unsafe fn has_non_str_kwarg_key(extra: &[MbValue]) -> bool {
    for v in extra {
        if let Some(ptr) = v.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                for (k, _) in map.iter() {
                    if !matches!(k, super::super::dict_ops::DictKey::Str(_)) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// True if the slice carries a kwargs dict with any key not in `allowed`.
/// Used to reject unexpected keyword arguments (e.g. `b2a_qp(foo='bar')`).
unsafe fn has_unexpected_kwarg(extra: &[MbValue], allowed: &[&str]) -> Option<String> {
    for v in extra {
        if let Some(ptr) = v.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                for (k, _) in map.iter() {
                    if let super::super::dict_ops::DictKey::Str(name) = k {
                        if !allowed.contains(&name.as_str()) {
                            return Some(name.clone());
                        }
                    }
                }
            }
        }
    }
    None
}

fn truthy(v: MbValue) -> bool {
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    !v.is_none()
}

// ── hex ──

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// hexlify(data[, sep[, bytes_per_sep=1]]) — mirrors bytes.hex().
fn encode_hex(src: &[u8], sep: Option<u8>, bytes_per_sep: i64) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let n = src.len();
    let mut out = Vec::with_capacity(n * 2 + n);
    match sep {
        None => {
            for &b in src {
                out.push(HEX[(b >> 4) as usize]);
                out.push(HEX[(b & 0x0f) as usize]);
            }
        }
        Some(s) => {
            // bytes.hex groups from the right for positive bytes_per_sep and
            // from the left for negative. Replicate that exactly.
            let abs = bytes_per_sep.unsigned_abs() as usize;
            let abs = if abs == 0 { 1 } else { abs };
            for (i, &b) in src.iter().enumerate() {
                if i > 0 {
                    let boundary = if bytes_per_sep >= 0 {
                        // group from the right: insert before index where
                        // (n - i) is a multiple of abs.
                        (n - i) % abs == 0
                    } else {
                        i % abs == 0
                    };
                    if boundary {
                        out.push(s);
                    }
                }
                out.push(HEX[(b >> 4) as usize]);
                out.push(HEX[(b & 0x0f) as usize]);
            }
        }
    }
    out
}

fn decode_hex(src: &[u8]) -> Result<Vec<u8>, String> {
    if src.len() % 2 != 0 {
        return Err("Odd-length string".to_string());
    }
    let mut out = Vec::with_capacity(src.len() / 2);
    let mut i = 0;
    while i < src.len() {
        let hi = hex_digit(src[i]).ok_or_else(|| "Non-hexadecimal digit found".to_string())?;
        let lo = hex_digit(src[i + 1]).ok_or_else(|| "Non-hexadecimal digit found".to_string())?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

unsafe extern "C" fn dispatch_hexlify(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    // sep / bytes_per_sep — positional or keyword.
    let mut sep: Option<u8> = None;
    let mut bytes_per_sep: i64 = 1;
    if let Some(sv) = args.get(1).copied().filter(|v| !v.is_none()) {
        if let Some(b) = sep_byte(sv) {
            sep = Some(b);
        }
    }
    if let Some(bv) = args.get(2).and_then(|v| v.as_int()) {
        bytes_per_sep = bv;
    }
    bytes_val(encode_hex(&src, sep, bytes_per_sep))
}

/// A separator argument is a length-1 str or bytes. Extract its single byte.
unsafe fn sep_byte(v: MbValue) -> Option<u8> {
    if let Some(ptr) = v.as_ptr() {
        match &(*ptr).data {
            ObjData::Str(s) if s.len() == 1 => return Some(s.as_bytes()[0]),
            ObjData::Bytes(b) if b.len() == 1 => return Some(b[0]),
            ObjData::ByteArray(lock) => {
                let g = lock.read().unwrap();
                if g.len() == 1 {
                    return Some(g[0]);
                }
            }
            _ => {}
        }
    }
    None
}

unsafe extern "C" fn dispatch_unhexlify(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let src = match decode_arg_bytes(arg, "bytes-like or ASCII string") {
        Ok(v) => v,
        Err(e) => return e,
    };
    match decode_hex(&src) {
        Ok(v) => bytes_val(v),
        Err(e) => raise_binascii_error(&e),
    }
}

// ── base64 ──

const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Reverse table: ASCII byte -> 0..=63 or 0xFF for non-alphabet.
fn b64_value(c: u8) -> u8 {
    match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 0xFF,
    }
}

fn encode_base64(src: &[u8], newline: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity((src.len() + 2) / 3 * 4 + 1);
    let mut chunks = src.chunks_exact(3);
    for c in &mut chunks {
        let n = ((c[0] as u32) << 16) | ((c[1] as u32) << 8) | (c[2] as u32);
        out.push(B64_TABLE[(n >> 18) as usize & 0x3f]);
        out.push(B64_TABLE[(n >> 12) as usize & 0x3f]);
        out.push(B64_TABLE[(n >> 6) as usize & 0x3f]);
        out.push(B64_TABLE[n as usize & 0x3f]);
    }
    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let n = (rem[0] as u32) << 16;
            out.push(B64_TABLE[(n >> 18) as usize & 0x3f]);
            out.push(B64_TABLE[(n >> 12) as usize & 0x3f]);
            out.push(b'=');
            out.push(b'=');
        }
        2 => {
            let n = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
            out.push(B64_TABLE[(n >> 18) as usize & 0x3f]);
            out.push(B64_TABLE[(n >> 12) as usize & 0x3f]);
            out.push(B64_TABLE[(n >> 6) as usize & 0x3f]);
            out.push(b'=');
        }
        _ => {}
    }
    if newline {
        out.push(b'\n');
    }
    out
}

/// Faithful port of CPython 3.12 `binascii_a2b_base64_impl`. Bytes are emitted
/// as data characters arrive; padding terminates the quad. In non-strict mode
/// (default) non-alphabet bytes and stray padding are skipped; strict mode
/// reports the precise CPython error categories.
fn decode_base64(src: &[u8], strict: bool) -> Result<Vec<u8>, String> {
    let ascii_len = src.len();
    let mut out: Vec<u8> = Vec::with_capacity((ascii_len + 3) / 4 * 3);
    let mut padding_started = false;

    if strict && ascii_len > 0 && src[0] == b'=' {
        return Err("Leading padding not allowed".to_string());
    }

    let mut quad_pos: i32 = 0;
    let mut leftchar: u8 = 0;
    let mut pads: i32 = 0;

    let mut i = 0usize;
    while i < ascii_len {
        let this_ch = src[i];

        if this_ch == b'=' {
            padding_started = true;
            if strict && quad_pos == 0 {
                return Err("Excess padding not allowed".to_string());
            }
            if quad_pos >= 2 {
                pads += 1;
                if quad_pos + pads >= 4 {
                    // Pad sequence complete; stop parsing. In strict mode,
                    // any remaining characters are excess data.
                    if strict && i + 1 < ascii_len {
                        return Err("Excess data after padding".to_string());
                    }
                    return Ok(out);
                }
            }
            i += 1;
            continue;
        }

        let v = b64_value(this_ch);
        if v >= 64 {
            if strict {
                return Err("Only base64 data is allowed".to_string());
            }
            i += 1;
            continue;
        }

        if strict && padding_started {
            return Err("Discontinuous padding not allowed".to_string());
        }
        pads = 0;

        match quad_pos {
            0 => {
                quad_pos = 1;
                leftchar = v;
            }
            1 => {
                quad_pos = 2;
                out.push((leftchar << 2) | (v >> 4));
                leftchar = v & 0x0f;
            }
            2 => {
                quad_pos = 3;
                out.push((leftchar << 4) | (v >> 2));
                leftchar = v & 0x03;
            }
            _ => {
                quad_pos = 0;
                out.push((leftchar << 6) | v);
                leftchar = 0;
            }
        }
        i += 1;
    }

    if quad_pos != 0 {
        if quad_pos == 1 {
            let n = (out.len() / 3) * 4 + 1;
            return Err(format!(
                "Invalid base64-encoded string: number of data characters ({}) cannot be 1 more than a multiple of 4",
                n
            ));
        }
        return Err("Incorrect padding".to_string());
    }
    Ok(out)
}

unsafe extern "C" fn dispatch_b2a_base64(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    let extra = &args[1..];
    let mut newline = true;
    if let Some(v) = kwarg(extra, "newline") {
        newline = truthy(v);
    }
    bytes_val(encode_base64(&src, newline))
}

unsafe extern "C" fn dispatch_a2b_base64(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let src = match decode_arg_bytes(arg, "bytes-like or ASCII string") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let extra = &args[1..];
    let mut strict = false;
    if let Some(v) = kwarg(extra, "strict_mode") {
        strict = truthy(v);
    }
    match decode_base64(&src, strict) {
        Ok(v) => bytes_val(v),
        Err(e) => raise_binascii_error(&e),
    }
}

// ── crc32 / crc_hqx ──

unsafe extern "C" fn dispatch_crc32(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    let seed = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as u32;
    let crc = crc32(&src, seed);
    MbValue::from_int(crc as i64)
}

fn crc32(data: &[u8], seed: u32) -> u32 {
    let mut crc = !seed;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// CRC-CCITT (XModem) as used by binascii.crc_hqx.
fn crc_hqx(data: &[u8], mut crc: u32) -> u32 {
    crc &= 0xffff;
    for &b in data {
        crc = ((crc << 8) & 0xff00) ^ CRC_HQX_TABLE[(((crc >> 8) ^ (b as u32)) & 0xff) as usize];
    }
    crc & 0xffff
}

unsafe extern "C" fn dispatch_crc_hqx(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    if args.len() < 2 {
        return raise_type_error("crc_hqx() missing required argument 'crc' (pos 2)");
    }
    let Some(src) = as_bytes_like(args[0]) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    let Some(seed) = args[1].as_int() else {
        return raise_type_error("'crc' must be an integer");
    };
    let crc = crc_hqx(&src, seed as u32);
    MbValue::from_int(crc as i64)
}

// CRC-CCITT table (poly 0x1021).
const CRC_HQX_TABLE: [u32; 256] = build_crc_hqx_table();

const fn build_crc_hqx_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = (i as u32) << 8;
        let mut j = 0;
        while j < 8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
            crc &= 0xffff;
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

// ── uuencode ──

unsafe extern "C" fn dispatch_b2a_uu(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    let extra = &args[1..];
    // `backtick` is keyword-only in CPython 3.12; a bare positional second
    // arg must raise TypeError (test: b2a_uu(b'', True)).
    for v in extra {
        let is_kwdict = v
            .as_ptr()
            .map(|p| matches!(&(*p).data, ObjData::Dict(_)))
            .unwrap_or(false);
        if !is_kwdict {
            return raise_type_error("b2a_uu() takes at most 1 positional argument (2 given)");
        }
    }
    if let Some(name) = has_unexpected_kwarg(extra, &["backtick"]) {
        return raise_type_error(&format!(
            "b2a_uu() got an unexpected keyword argument '{}'",
            name
        ));
    }
    let backtick = kwarg(extra, "backtick").map(truthy).unwrap_or(false);
    if src.len() > 45 {
        return raise_binascii_error("At most 45 bytes at once");
    }
    bytes_val(encode_uu(&src, backtick))
}

fn uu_char(c: u8, backtick: bool) -> u8 {
    let v = c & 0x3f;
    if v == 0 {
        if backtick {
            b'`'
        } else {
            b' '
        }
    } else {
        v + 0x20
    }
}

fn encode_uu(src: &[u8], backtick: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len() / 3 * 4 + 2);
    // Leading length byte.
    out.push(uu_char(src.len() as u8, backtick));
    let mut i = 0;
    while i < src.len() {
        let b0 = src[i];
        let b1 = if i + 1 < src.len() { src[i + 1] } else { 0 };
        let b2 = if i + 2 < src.len() { src[i + 2] } else { 0 };
        out.push(uu_char(b0 >> 2, backtick));
        out.push(uu_char(((b0 << 4) | (b1 >> 4)) & 0x3f, backtick));
        out.push(uu_char(((b1 << 2) | (b2 >> 6)) & 0x3f, backtick));
        out.push(uu_char(b2 & 0x3f, backtick));
        i += 3;
    }
    out.push(b'\n');
    out
}

unsafe extern "C" fn dispatch_a2b_uu(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let src = match decode_arg_bytes(arg, "bytes-like or ASCII string") {
        Ok(v) => v,
        Err(e) => return e,
    };
    match decode_uu(&src) {
        Ok(v) => bytes_val(v),
        Err(e) => raise_binascii_error(&e),
    }
}

/// Decode a single uuencoded line. Faithful port of CPython 3.12
/// `binascii_a2b_uu_impl`: the leading byte gives the binary length; the body
/// is consumed one char at a time (NUL-substituting when the line is short),
/// and any non-whitespace/non-backtick trailing data is "Trailing garbage".
fn decode_uu(src: &[u8]) -> Result<Vec<u8>, String> {
    if src.is_empty() {
        return Ok(Vec::new());
    }
    // First byte: binary data length.
    let mut bin_len: isize = ((src[0].wrapping_sub(b' ')) & 0o77) as isize;
    let mut idx = 1usize; // ascii_data pointer
    let ascii_len = src.len();
    let mut out = Vec::with_capacity(bin_len.max(0) as usize);

    let mut leftbits: i32 = 0;
    let mut leftchar: u32 = 0;

    while bin_len > 0 {
        let have = idx < ascii_len;
        let this_ch = if have { src[idx] } else { 0 };
        let sextet: u32;
        if this_ch == b'\n' || this_ch == b'\r' || !have {
            // Whitespace / short line: NUL substitution.
            sextet = 0;
        } else {
            // 0x20..=0x60 inclusive (' ' .. ' '+64) is the legal range.
            if this_ch < b' ' || this_ch > b' ' + 64 {
                return Err("Illegal char".to_string());
            }
            sextet = ((this_ch - b' ') & 0o77) as u32;
        }
        leftchar = (leftchar << 6) | sextet;
        leftbits += 6;
        if leftbits >= 8 {
            leftbits -= 8;
            out.push(((leftchar >> leftbits) & 0xff) as u8);
            leftchar &= (1u32 << leftbits) - 1;
            bin_len -= 1;
        }
        idx += 1;
    }

    // Trailing check: anything left on the line must be whitespace/backtick.
    while idx < ascii_len {
        let this_ch = src[idx];
        idx += 1;
        if this_ch != b' ' && this_ch != b' ' + 64 && this_ch != b'\n' && this_ch != b'\r' {
            return Err("Trailing garbage".to_string());
        }
    }
    Ok(out)
}

// ── quoted-printable ──

fn qp_hex_upper(b: u8) -> [u8; 3] {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    [b'=', HEX[(b >> 4) as usize], HEX[(b & 0x0f) as usize]]
}

const QP_MAXLINESIZE: usize = 76;

/// Faithful port of CPython 3.12 `binascii_b2a_qp_impl`.
///
/// `quotetabs`: also escape spaces/tabs (default false → only trailing ws).
/// `istext`: text mode (default true) — normalises line endings to match the
/// input's detected convention. `header`: encode spaces as '_', escape '_'.
fn encode_qp(src: &[u8], quotetabs: bool, istext: bool, header: bool) -> Vec<u8> {
    let datalen = src.len();
    let mut out: Vec<u8> = Vec::with_capacity(datalen + datalen / 3 + 8);

    // Detect CRLF convention: if the first '\n' is preceded by '\r'.
    let mut crlf = false;
    if let Some(p) = src.iter().position(|&b| b == b'\n') {
        if p > 0 && src[p - 1] == b'\r' {
            crlf = true;
        }
    }

    // `needs_escape(in)` mirrors the big condition in the C source.
    let needs_escape = |i: usize, linelen: usize| -> bool {
        let c = src[i];
        if c > 126 {
            return true;
        }
        if c == b'=' {
            return true;
        }
        if header && c == b'_' {
            return true;
        }
        if c == b'.'
            && linelen == 0
            && (i + 1 == datalen || src[i + 1] == b'\n' || src[i + 1] == b'\r' || src[i + 1] == 0)
        {
            return true;
        }
        if !istext && (c == b'\r' || c == b'\n') {
            return true;
        }
        if (c == b'\t' || c == b' ') && i + 1 == datalen {
            return true;
        }
        if c < 33 && c != b'\r' && c != b'\n' && (quotetabs || (c != b'\t' && c != b' ')) {
            return true;
        }
        false
    };

    let mut i = 0usize;
    let mut linelen = 0usize;
    while i < datalen {
        let c = src[i];
        if needs_escape(i, linelen) {
            if linelen + 3 >= QP_MAXLINESIZE {
                out.push(b'=');
                if crlf {
                    out.push(b'\r');
                }
                out.push(b'\n');
                linelen = 0;
            }
            let e = qp_hex_upper(c);
            out.extend_from_slice(&e);
            i += 1;
            linelen += 3;
        } else if istext && (c == b'\n' || (i + 1 < datalen && c == b'\r' && src[i + 1] == b'\n')) {
            linelen = 0;
            // Protect against whitespace on end of line: retro-escape the last
            // emitted space/tab.
            if let Some(&last) = out.last() {
                if last == b' ' || last == b'\t' {
                    out.pop();
                    let e = qp_hex_upper(last);
                    out.extend_from_slice(&e);
                }
            }
            if crlf {
                out.push(b'\r');
            }
            out.push(b'\n');
            if c == b'\r' {
                i += 2;
            } else {
                i += 1;
            }
        } else {
            if i + 1 != datalen && src[i + 1] != b'\n' && linelen + 1 >= QP_MAXLINESIZE {
                out.push(b'=');
                if crlf {
                    out.push(b'\r');
                }
                out.push(b'\n');
                linelen = 0;
            }
            linelen += 1;
            if header && c == b' ' {
                out.push(b'_');
            } else {
                out.push(c);
            }
            i += 1;
        }
    }
    out
}

unsafe extern "C" fn dispatch_b2a_qp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("a bytes-like object is required, not 'str'");
    };
    let extra = &args[1..];
    if has_non_str_kwarg_key(extra) {
        return raise_type_error("keywords must be strings");
    }
    if let Some(name) = has_unexpected_kwarg(extra, &["quotetabs", "istext", "header"]) {
        return raise_type_error(&format!(
            "b2a_qp() got an unexpected keyword argument '{}'",
            name
        ));
    }
    let quotetabs = kwarg(extra, "quotetabs").map(truthy).unwrap_or(false);
    let istext = kwarg(extra, "istext").map(truthy).unwrap_or(true);
    let header = kwarg(extra, "header").map(truthy).unwrap_or(false);
    bytes_val(encode_qp(&src, quotetabs, istext, header))
}

/// CPython `binascii_a2b_qp_impl`: decode quoted-printable.
fn decode_qp(src: &[u8], header: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    let len = src.len();
    let mut i = 0;
    while i < len {
        let c = src[i];
        if c == b'=' {
            if i + 1 >= len {
                // lone '=' at end → soft break, drop it.
                i += 1;
                continue;
            }
            let n1 = src[i + 1];
            if n1 == b'\n' {
                // soft line break "=\n"
                i += 2;
                continue;
            }
            if n1 == b'\r' {
                // "=\r" — if followed by \n, soft break; else a bare "=\r"
                // is dropped (CPython drops to end of nothing → produces '').
                if i + 2 < len && src[i + 2] == b'\n' {
                    i += 3;
                } else {
                    i += 2;
                }
                continue;
            }
            if n1 == b'=' {
                // "==" → literal '='
                out.push(b'=');
                i += 2;
                continue;
            }
            // "=XX" hex escape.
            if i + 2 < len {
                let h = hex_digit(n1);
                let l = hex_digit(src[i + 2]);
                if let (Some(h), Some(l)) = (h, l) {
                    out.push((h << 4) | l);
                    i += 3;
                    continue;
                }
            } else if i + 2 == len {
                // "=X" at end (only one hex digit available) → passthrough.
                out.push(b'=');
                i += 1;
                continue;
            }
            // Malformed escape → pass the '=' through verbatim.
            out.push(b'=');
            i += 1;
            continue;
        }
        if c == b'_' && header {
            out.push(b' ');
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

unsafe extern "C" fn dispatch_a2b_qp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let extra = if args.is_empty() {
        &args[..]
    } else {
        &args[1..]
    };
    if has_non_str_kwarg_key(extra) {
        return raise_type_error("keywords must be strings");
    }
    // The first positional or the `data=` keyword carries the input.
    let arg = if let Some(a) = args.first().copied() {
        // Could be the kwargs dict if called as a2b_qp(data=..., header=...).
        let is_kwdict = a
            .as_ptr()
            .map(|p| matches!(&(*p).data, ObjData::Dict(_)))
            .unwrap_or(false);
        if is_kwdict {
            // No positional data; look for data= kwarg.
            match kwarg(extra, "data").or_else(|| kwarg(args, "data")) {
                Some(d) => d,
                None => {
                    return raise_type_error("function missing required argument 'data' (pos 1)")
                }
            }
        } else {
            a
        }
    } else {
        return raise_type_error("function missing required argument 'data' (pos 1)");
    };
    if let Some(name) = has_unexpected_kwarg(extra, &["data", "header"]) {
        return raise_type_error(&format!(
            "a2b_qp() got an unexpected keyword argument '{}'",
            name
        ));
    }
    let src = match decode_arg_bytes(arg, "bytes-like or ASCII string") {
        Ok(v) => v,
        Err(e) => return e,
    };
    let header = kwarg(extra, "header").map(truthy).unwrap_or(false);
    bytes_val(decode_qp(&src, header))
}

/// Register the binascii module.
pub fn register() {
    let mut attrs = HashMap::new();

    let entries: Vec<(&str, usize)> = vec![
        ("hexlify", dispatch_hexlify as usize),
        ("b2a_hex", dispatch_hexlify as usize),
        ("unhexlify", dispatch_unhexlify as usize),
        ("a2b_hex", dispatch_unhexlify as usize),
        ("b2a_base64", dispatch_b2a_base64 as usize),
        ("a2b_base64", dispatch_a2b_base64 as usize),
        ("crc32", dispatch_crc32 as usize),
        ("crc_hqx", dispatch_crc_hqx as usize),
        ("b2a_uu", dispatch_b2a_uu as usize),
        ("a2b_uu", dispatch_a2b_uu as usize),
        ("b2a_qp", dispatch_b2a_qp as usize),
        ("a2b_qp", dispatch_a2b_qp as usize),
    ];

    for (name, addr) in &entries {
        attrs.insert((*name).to_string(), MbValue::from_func(*addr));
    }

    // Exception classes. `binascii.Error` is a ValueError subclass; both are
    // matched by the exception machinery via name (see exception.rs).
    attrs.insert(
        "Error".into(),
        MbValue::from_ptr(MbObject::new_str("binascii.Error".to_string())),
    );
    attrs.insert(
        "Incomplete".into(),
        MbValue::from_ptr(MbObject::new_str("binascii.Incomplete".to_string())),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in &entries {
            set.insert(*addr as u64);
        }
    });

    super::register_module("binascii", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_bytes(b: &[u8]) -> MbValue {
        MbValue::from_ptr(MbObject::new_bytes(b.to_vec()))
    }

    fn mk_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn bytes_of(v: MbValue) -> Vec<u8> {
        unsafe {
            let p = v.as_ptr().expect("ptr");
            if let ObjData::Bytes(ref b) = (*p).data {
                b.clone()
            } else {
                panic!("not bytes");
            }
        }
    }

    #[test]
    fn hexlify_roundtrip() {
        let arg = mk_bytes(&[0xde, 0xad, 0xbe, 0xef]);
        let v = unsafe { dispatch_hexlify(&arg, 1) };
        assert_eq!(bytes_of(v), b"deadbeef");
    }

    #[test]
    fn unhexlify_str_input() {
        let arg = mk_str("0a0B0c");
        let v = unsafe { dispatch_unhexlify(&arg, 1) };
        assert_eq!(bytes_of(v), vec![0x0a, 0x0b, 0x0c]);
    }

    #[test]
    fn b2a_base64_with_newline() {
        let arg = mk_bytes(b"hello");
        let v = unsafe { dispatch_b2a_base64(&arg, 1) };
        assert_eq!(bytes_of(v), b"aGVsbG8=\n");
    }

    #[test]
    fn a2b_base64_decode() {
        let arg = mk_str("aGVsbG8=");
        let v = unsafe { dispatch_a2b_base64(&arg, 1) };
        assert_eq!(bytes_of(v), b"hello");
    }

    #[test]
    fn crc_hqx_known() {
        let c = crc_hqx(b"Test the CRC-32 of", 0);
        assert_eq!(c, 30730);
        assert_eq!(crc_hqx(b" this string.", c), 14290);
        // Empty data returns the seed masked to 16 bits.
        assert_eq!(crc_hqx(b"", 0xffff_ffff), 0xffff);
        assert_eq!(crc_hqx(b"", 0x1234_5678), 0x5678);
    }

    #[test]
    fn crc32_known() {
        assert_eq!(crc32(b"hello", 0), 907060870);
    }

    #[test]
    fn uu_known() {
        assert_eq!(encode_uu(b"x", false), b"!>   \n");
        assert_eq!(encode_uu(b"", false), b" \n");
        assert_eq!(encode_uu(b"\x00Cat", false), b"$ $-A=   \n");
        assert_eq!(encode_uu(b"\x00Cat", true), b"$`$-A=```\n");
    }

    #[test]
    fn uu_decode() {
        assert_eq!(decode_uu(b"!>   \n").unwrap(), b"x");
        assert_eq!(decode_uu(b"\x7f").unwrap(), vec![0u8; 31]);
        assert_eq!(decode_uu(b" \n").unwrap(), b"");
    }

    #[test]
    fn qp_basics() {
        assert_eq!(encode_qp(b"\x7f", false, true, false), b"=7F");
        assert_eq!(encode_qp(b"=", false, true, false), b"=3D");
        assert_eq!(encode_qp(b" ", false, true, false), b"=20");
        assert_eq!(encode_qp(b".", false, true, false), b"=2E");
        assert_eq!(decode_qp(b"=AB", false), vec![0xab]);
        assert_eq!(decode_qp(b"==", false), b"=");
        assert_eq!(decode_qp(b"=", false), b"");
    }
}
