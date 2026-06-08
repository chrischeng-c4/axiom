use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// quopri module for Mamba (#1261 long-tail).
///
/// RFC 1521 quoted-printable encoding. Provides real `encodestring` /
/// `decodestring` (the value-only API) on top of bytes/str inputs;
/// `encode`/`decode` (file-like API) stay noops because Mamba doesn't
/// introspect file objects from here yet. CPython's `quote`/`unquote`
/// are exposed as the same dispatchers since they share semantics.
///
/// Encoding rules (matches CPython Modules/quopri.c):
///   - printable ASCII (33..=60, 62..=126) emitted literally
///   - bytes < 33, > 126, or `=` are escaped as `=XX` (uppercase hex)
///   - space/tab quoted only when `quotetabs=True` OR when trailing on a line
///   - lines wrap at 76 chars via soft line break `=\n`
///   - `header=True` rewrites space to `_` (RFC 2047)
use std::collections::HashMap;

const ESCAPE: u8 = b'=';
const MAXLINESIZE: usize = 76;
const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
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

fn arg_bool(val: Option<MbValue>) -> bool {
    let Some(v) = val else {
        return false;
    };
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    false
}

fn raise_type(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn needs_quoting(b: u8, quotetabs: bool) -> bool {
    if b == b'\t' || b == b' ' {
        return quotetabs;
    }
    b < 33 || b > 126 || b == ESCAPE
}

fn quote_byte(b: u8, out: &mut Vec<u8>) {
    out.push(ESCAPE);
    out.push(HEX_DIGITS[(b >> 4) as usize]);
    out.push(HEX_DIGITS[(b & 0x0f) as usize]);
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn encode_bytes(input: &[u8], quotetabs: bool, header: bool) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut line_len = 0usize;

    // Walk line-by-line so trailing whitespace can be forced-quoted
    // (RFC 1521 5.1.3). `split_inclusive` keeps the original newline
    // bytes attached so we round-trip CR/LF/CRLF endings.
    for raw_line in input.split_inclusive(|b| *b == b'\n') {
        let (content, ending) = if raw_line.ends_with(b"\r\n") {
            (
                &raw_line[..raw_line.len() - 2],
                &raw_line[raw_line.len() - 2..],
            )
        } else if raw_line.ends_with(b"\n") {
            (
                &raw_line[..raw_line.len() - 1],
                &raw_line[raw_line.len() - 1..],
            )
        } else {
            (&raw_line[..], &raw_line[raw_line.len()..])
        };

        let n = content.len();
        for (i, &b) in content.iter().enumerate() {
            let trailing_ws = (b == b' ' || b == b'\t') && i + 1 == n && !ending.is_empty();
            let header_space = header && b == b' ';
            let needs = trailing_ws || needs_quoting(b, quotetabs);
            let width = if header_space {
                1
            } else if needs {
                3
            } else {
                1
            };
            // Soft line break before we'd cross MAXLINESIZE (reserve room
            // for the `=` that introduces the break).
            if line_len + width >= MAXLINESIZE {
                out.push(ESCAPE);
                out.push(b'\n');
                line_len = 0;
            }
            if header_space {
                out.push(b'_');
            } else if needs {
                quote_byte(b, &mut out);
            } else {
                out.push(b);
            }
            line_len += width;
        }
        for &b in ending {
            out.push(b);
        }
        if !ending.is_empty() {
            line_len = 0;
        }
    }

    out
}

fn decode_bytes(input: &[u8], header: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        let b = input[i];
        if b == ESCAPE {
            // Soft line break: `=\n` or `=\r\n` → discard
            if i + 1 < input.len() && input[i + 1] == b'\n' {
                i += 2;
                continue;
            }
            if i + 2 < input.len() && input[i + 1] == b'\r' && input[i + 2] == b'\n' {
                i += 3;
                continue;
            }
            if i + 2 < input.len() {
                if let (Some(h1), Some(h2)) = (hex_val(input[i + 1]), hex_val(input[i + 2])) {
                    out.push((h1 << 4) | h2);
                    i += 3;
                    continue;
                }
            }
            // Malformed escape — pass `=` through (CPython behavior).
            out.push(b);
            i += 1;
        } else if header && b == b'_' {
            out.push(b' ');
            i += 1;
        } else {
            out.push(b);
            i += 1;
        }
    }
    out
}

unsafe extern "C" fn dispatch_encodestring(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type("encodestring() missing required argument");
    };
    let Some(src) = as_bytes(arg) else {
        return raise_type("encodestring() argument must be bytes-like");
    };
    let quotetabs = arg_bool(args.get(1).copied());
    MbValue::from_ptr(MbObject::new_bytes(encode_bytes(&src, quotetabs, false)))
}

unsafe extern "C" fn dispatch_decodestring(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type("decodestring() missing required argument");
    };
    let Some(src) = as_bytes(arg) else {
        return raise_type("decodestring() argument must be bytes-like");
    };
    let header = arg_bool(args.get(1).copied());
    MbValue::from_ptr(MbObject::new_bytes(decode_bytes(&src, header)))
}

unsafe extern "C" fn dispatch_noop(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_es = dispatch_encodestring as *const () as usize;
    let addr_ds = dispatch_decodestring as *const () as usize;
    let addr_noop = dispatch_noop as *const () as usize;

    attrs.insert("encodestring".into(), MbValue::from_func(addr_es));
    attrs.insert("decodestring".into(), MbValue::from_func(addr_ds));
    // `encode(input, output)` / `decode(input, output)` are file-based;
    // we don't introspect file objects from here yet — stay noops.
    attrs.insert("encode".into(), MbValue::from_func(addr_noop));
    attrs.insert("decode".into(), MbValue::from_func(addr_noop));
    // CPython exposes `quote`/`unquote` as the same semantics.
    attrs.insert("quote".into(), MbValue::from_func(addr_es));
    attrs.insert("unquote".into(), MbValue::from_func(addr_ds));

    attrs.insert("ESCAPE".into(), MbValue::from_int(ESCAPE as i64));
    attrs.insert("MAXLINESIZE".into(), MbValue::from_int(MAXLINESIZE as i64));
    attrs.insert("HEX".into(), MbValue::from_int(16));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_es as u64);
        set.insert(addr_ds as u64);
        set.insert(addr_noop as u64);
    });

    super::register_module("quopri", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn encodes_high_byte() {
        let out = encode_bytes(b"caf\xc3\xa9", false, false);
        assert_eq!(out, b"caf=C3=A9");
    }

    #[test]
    fn encodes_equals_sign() {
        let out = encode_bytes(b"a=b", false, false);
        assert_eq!(out, b"a=3Db");
    }

    #[test]
    fn keeps_plain_ascii() {
        let out = encode_bytes(b"hello world", false, false);
        assert_eq!(out, b"hello world");
    }

    #[test]
    fn quotetabs_quotes_space_and_tab() {
        let out = encode_bytes(b"a b\tc", true, false);
        assert_eq!(out, b"a=20b=09c");
    }

    #[test]
    fn trailing_whitespace_quoted_before_newline() {
        let out = encode_bytes(b"trail \n", false, false);
        assert_eq!(out, b"trail=20\n");
    }

    #[test]
    fn soft_line_break_after_75_chars() {
        let src = vec![b'a'; 100];
        let out = encode_bytes(&src, false, false);
        // Must contain a soft break `=\n` somewhere.
        assert!(out.windows(2).any(|w| w == b"=\n"));
    }

    #[test]
    fn header_space_to_underscore() {
        let out = encode_bytes(b"a b", false, true);
        assert_eq!(out, b"a_b");
    }

    #[test]
    fn decode_roundtrip() {
        let src = b"hello caf=C3=A9 world";
        let out = decode_bytes(src, false);
        assert_eq!(out, b"hello caf\xc3\xa9 world");
    }

    #[test]
    fn decode_soft_break() {
        let src = b"hello=\nworld";
        let out = decode_bytes(src, false);
        assert_eq!(out, b"helloworld");
    }

    #[test]
    fn decode_header_underscore() {
        let out = decode_bytes(b"a_b", true);
        assert_eq!(out, b"a b");
    }

    #[test]
    fn dispatch_encodestring_via_bytes_arg() {
        let arg = MbValue::from_ptr(MbObject::new_bytes(b"hi=there".to_vec()));
        let v = unsafe { dispatch_encodestring(&arg, 1) };
        assert_eq!(bytes_of(v), b"hi=3Dthere");
    }

    #[test]
    fn dispatch_decodestring_via_bytes_arg() {
        let arg = MbValue::from_ptr(MbObject::new_bytes(b"hi=3Dthere".to_vec()));
        let v = unsafe { dispatch_decodestring(&arg, 1) };
        assert_eq!(bytes_of(v), b"hi=there");
    }
}
