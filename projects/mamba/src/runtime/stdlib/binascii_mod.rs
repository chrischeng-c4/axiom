use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use base64::Engine;
/// binascii module for Mamba (#1261).
///
/// Implements the four hot entry points (`hexlify`, `unhexlify`,
/// `b2a_base64`, `a2b_base64`) against the workspace `base64` crate and
/// hand-rolled hex parsing. Accepts both bytes/bytearray and str inputs
/// on the decode side (CPython binascii decodes both). All return values
/// are `bytes` to match CPython semantics. On malformed input the
/// dispatchers raise `ValueError` so plain `except Exception:` catches
/// the failure.
use std::collections::HashMap;

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

/// Pull owned bytes from a bytes/bytearray value.
unsafe fn as_bytes_like(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Bytes(b) => Some(b.clone()),
        ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
        _ => None,
    }
}

/// Decode-side helpers accept ASCII strings as CPython does.
unsafe fn as_bytes_or_str(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Bytes(b) => Some(b.clone()),
        ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
        ObjData::Str(s) => Some(s.as_bytes().to_vec()),
        _ => None,
    }
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
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

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(src: &[u8]) -> Vec<u8> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = Vec::with_capacity(src.len() * 2);
    for &b in src {
        out.push(HEX[(b >> 4) as usize]);
        out.push(HEX[(b & 0x0f) as usize]);
    }
    out
}

fn decode_hex(src: &[u8]) -> Result<Vec<u8>, String> {
    // CPython's binascii.unhexlify ignores ASCII whitespace? No — it
    // rejects on odd length and non-hex digits. Match that exactly.
    if src.len() % 2 != 0 {
        return Err("Odd-length string".to_string());
    }
    let mut out = Vec::with_capacity(src.len() / 2);
    let mut i = 0;
    while i < src.len() {
        let hi = hex_digit(src[i]).ok_or_else(|| format!("Non-hexadecimal digit found"))?;
        let lo = hex_digit(src[i + 1]).ok_or_else(|| format!("Non-hexadecimal digit found"))?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

/// `binascii.hexlify(data, [sep, [bytes_per_sep]])` — only the single-arg
/// form is wired here. CPython's optional `sep`/`bytes_per_sep` are rare
/// enough that the dispatcher ignores extra args.
unsafe extern "C" fn dispatch_hexlify(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("hexlify() missing required argument");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("hexlify() argument must be bytes-like");
    };
    MbValue::from_ptr(MbObject::new_bytes(encode_hex(&src)))
}

unsafe extern "C" fn dispatch_unhexlify(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("unhexlify() missing required argument");
    };
    let Some(src) = as_bytes_or_str(arg) else {
        return raise_type_error("unhexlify() argument must be bytes-like or str");
    };
    match decode_hex(&src) {
        Ok(v) => MbValue::from_ptr(MbObject::new_bytes(v)),
        Err(e) => raise_value_error(&e),
    }
}

unsafe extern "C" fn dispatch_b2a_base64(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("b2a_base64() missing required argument");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("b2a_base64() argument must be bytes-like");
    };
    // CPython appends '\n' unless newline=False. We don't introspect
    // keyword args yet; positional arg2 (truthy) is honored.
    let mut append_newline = true;
    if let Some(opt) = args.get(1) {
        if let Some(b) = opt.as_bool() {
            append_newline = b;
        } else if let Some(i) = opt.as_int() {
            append_newline = i != 0;
        }
    }
    let mut out = base64::engine::general_purpose::STANDARD
        .encode(&src)
        .into_bytes();
    if append_newline {
        out.push(b'\n');
    }
    MbValue::from_ptr(MbObject::new_bytes(out))
}

unsafe extern "C" fn dispatch_a2b_base64(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("a2b_base64() missing required argument");
    };
    let Some(src) = as_bytes_or_str(arg) else {
        return raise_type_error("a2b_base64() argument must be bytes-like or str");
    };
    // CPython tolerates trailing whitespace/newlines; the STANDARD engine
    // does not, so strip ASCII whitespace before decoding.
    let trimmed: Vec<u8> = src
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    match base64::engine::general_purpose::STANDARD.decode(&trimmed) {
        Ok(v) => MbValue::from_ptr(MbObject::new_bytes(v)),
        Err(e) => raise_value_error(&format!("Invalid base64-encoded string: {}", e)),
    }
}

unsafe extern "C" fn dispatch_crc32(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type_error("crc32() missing required argument");
    };
    let Some(src) = as_bytes_like(arg) else {
        return raise_type_error("crc32() argument must be bytes-like");
    };
    let seed = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as u32;
    let mut hasher = crc32fast::Hasher::new_with_initial(seed);
    hasher.update(&src);
    MbValue::from_int(hasher.finalize() as i64)
}

/// Register the binascii module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_hx = dispatch_hexlify as *const () as usize;
    attrs.insert("hexlify".into(), MbValue::from_func(addr_hx));
    // CPython exposes `b2a_hex` as an alias of `hexlify`.
    attrs.insert("b2a_hex".into(), MbValue::from_func(addr_hx));

    let addr_uhx = dispatch_unhexlify as *const () as usize;
    attrs.insert("unhexlify".into(), MbValue::from_func(addr_uhx));
    attrs.insert("a2b_hex".into(), MbValue::from_func(addr_uhx));

    let addr_b2a = dispatch_b2a_base64 as *const () as usize;
    attrs.insert("b2a_base64".into(), MbValue::from_func(addr_b2a));

    let addr_a2b = dispatch_a2b_base64 as *const () as usize;
    attrs.insert("a2b_base64".into(), MbValue::from_func(addr_a2b));

    let addr_crc32 = dispatch_crc32 as *const () as usize;
    attrs.insert("crc32".into(), MbValue::from_func(addr_crc32));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_hx as u64);
        set.insert(addr_uhx as u64);
        set.insert(addr_b2a as u64);
        set.insert(addr_a2b as u64);
        set.insert(addr_crc32 as u64);
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
    fn unhexlify_bytes_input() {
        let arg = mk_bytes(b"cafebabe");
        let v = unsafe { dispatch_unhexlify(&arg, 1) };
        assert_eq!(bytes_of(v), vec![0xca, 0xfe, 0xba, 0xbe]);
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
    fn b2a_base64_no_newline() {
        let arg = mk_bytes(b"hello");
        let nl = MbValue::from_bool(false);
        let args = [arg, nl];
        let v = unsafe { dispatch_b2a_base64(args.as_ptr(), 2) };
        assert_eq!(bytes_of(v), b"aGVsbG8=");
    }

    #[test]
    fn a2b_base64_decode() {
        let arg = mk_str("aGVsbG8=");
        let v = unsafe { dispatch_a2b_base64(&arg, 1) };
        assert_eq!(bytes_of(v), b"hello");
    }

    #[test]
    fn a2b_base64_with_newlines() {
        let arg = mk_bytes(b"aGVs\nbG8=\n");
        let v = unsafe { dispatch_a2b_base64(&arg, 1) };
        assert_eq!(bytes_of(v), b"hello");
    }
}
