/// quopri module for Mamba (#1261 long-tail).
///
/// RFC 1521 quoted-printable encoding. Provides byte-exact ports of CPython's
/// pure-Python `quopri` (`Lib/quopri.py`): `encodestring`/`decodestring` (the
/// value APIs) and `encode`/`decode` (the file-object APIs that read/write
/// `io.BytesIO`). CPython's `quote`/`unquote` are *not* part of quopri's public
/// surface, but legacy callers expect the string dispatchers, so we keep them.
///
/// The encode/decode algorithms mirror `Lib/quopri.py` exactly, including its
/// line-based processing, soft-line-break splitting at the 76th column (which
/// may cut an `=XX` escape), trailing-whitespace quoting, the lone-`.` rule,
/// and the `==` decode shortcut.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Read a string key out of a trailing kwargs `ObjData::Dict`.
fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn truthy(val: MbValue) -> bool {
    if let Some(b) = val.as_bool() {
        return b;
    }
    if let Some(i) = val.as_int() {
        return i != 0;
    }
    false
}

/// Resolve a boolean flag for one of the keyword/positional parameters of a
/// quopri dispatcher. Mamba lowers `f(a, kw=v)` on a module attribute into a
/// trailing kwargs `dict` argument, while a bare `f(a, v)` passes `v`
/// positionally. We accept either: a trailing dict supplies named flags, and a
/// non-dict argument at `pos` supplies the positional value.
fn flag(args: &[MbValue], pos: usize, name: &str) -> bool {
    // Trailing kwargs dict (always the last arg when keywords were used).
    if let Some(&last) = args.last() {
        if is_dict(last) {
            if let Some(v) = dict_get(last, name) {
                return truthy(v);
            }
        }
    }
    // Positional value (only when it isn't the kwargs dict itself).
    if let Some(&v) = args.get(pos) {
        if !is_dict(v) {
            return truthy(v);
        }
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

/// CPython `needsquoting(c, quotetabs, header)`.
fn needs_quoting(b: u8, quotetabs: bool, header: bool) -> bool {
    if b == b' ' || b == b'\t' {
        return quotetabs;
    }
    // header escapes `_` because `_` is the escaped space.
    if b == b'_' {
        return header;
    }
    b == ESCAPE || !(b' ' <= b && b <= b'~')
}

/// CPython `quote(c)` -> `=XX` (uppercase hex), appended to `out`.
fn quote_byte(b: u8, out: &mut Vec<u8>) {
    out.push(ESCAPE);
    out.push(HEX_DIGITS[(b >> 4) as usize]);
    out.push(HEX_DIGITS[(b & 0x0f) as usize]);
}

fn ishex(b: u8) -> bool {
    b.is_ascii_digit() || (b'a'..=b'f').contains(&b) || (b'A'..=b'F').contains(&b)
}

fn unhex2(h: u8, l: u8) -> u8 {
    fn v(c: u8) -> u8 {
        if c.is_ascii_digit() {
            c - b'0'
        } else if (b'a'..=b'f').contains(&c) {
            c - b'a' + 10
        } else {
            c - b'A' + 10
        }
    }
    (v(h) << 4) | v(l)
}

/// CPython `encode`'s inner `write(s, lineEnd)`.
///
/// RFC 1521 requires a line ending in space/tab to have that trailing byte
/// encoded; a line that is a lone `.` is encoded as well (SMTP dot-stuffing).
fn write_line(s: &[u8], line_end: &[u8], out: &mut Vec<u8>) {
    if let Some(&last) = s.last() {
        if last == b' ' || last == b'\t' {
            out.extend_from_slice(&s[..s.len() - 1]);
            quote_byte(last, out);
            out.extend_from_slice(line_end);
            return;
        }
    }
    if s == b"." {
        quote_byte(b'.', out);
        out.extend_from_slice(line_end);
        return;
    }
    out.extend_from_slice(s);
    out.extend_from_slice(line_end);
}

/// Split bytes into readline-style lines: each chunk holds the bytes up to and
/// including a `\n`; a final chunk without a trailing `\n` is yielded too.
fn readlines(input: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'\n' {
            lines.push(&input[start..=i]);
            start = i + 1;
        }
        i += 1;
    }
    if start < input.len() {
        lines.push(&input[start..]);
    }
    lines
}

/// Byte-exact port of `Lib/quopri.py::encode`.
fn encode_bytes(input: &[u8], quotetabs: bool, header: bool) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut prevline: Option<Vec<u8>> = None;
    let mut stripped: Vec<u8> = Vec::new();

    for raw in readlines(input) {
        // Strip the readline-induced trailing newline.
        let (line, this_stripped): (&[u8], &[u8]) = if raw.last() == Some(&b'\n') {
            (&raw[..raw.len() - 1], b"\n")
        } else {
            (raw, b"")
        };
        stripped = this_stripped.to_vec();

        // Build the un-length-limited encoded line.
        let mut outline: Vec<u8> = Vec::with_capacity(line.len());
        for &c in line {
            if needs_quoting(c, quotetabs, header) {
                let mut q = Vec::with_capacity(3);
                quote_byte(c, &mut q);
                // header && c == b' ' is impossible here: a space, when
                // quoted, becomes b"=20" (len 3) not b" ", so the CPython
                // `if header and c == b' '` branch only fires on the
                // *un-quoted* space below.
                outline.extend_from_slice(&q);
            } else if header && c == b' ' {
                outline.push(b'_');
            } else {
                outline.push(c);
            }
        }

        // Write out the previous line (with a hard newline).
        if let Some(pl) = prevline.take() {
            write_line(&pl, b"\n", &mut out);
        }

        // Soft line breaks for RFC length limits. `thisline` is the
        // already-quoted byte string; a break may split an `=XX` escape.
        let mut thisline = outline;
        while thisline.len() > MAXLINESIZE {
            let head = thisline[..MAXLINESIZE - 1].to_vec();
            write_line(&head, b"=\n", &mut out);
            thisline = thisline[MAXLINESIZE - 1..].to_vec();
        }
        prevline = Some(thisline);
    }

    // Write the last line, without a trailing newline.
    if let Some(pl) = prevline {
        write_line(&pl, &stripped, &mut out);
    }

    out
}

/// Byte-exact port of `Lib/quopri.py::decode`.
fn decode_bytes(input: &[u8], header: bool) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len());
    let mut new: Vec<u8> = Vec::new();

    for line in readlines(input) {
        let mut n = line.len();
        let mut i = 0usize;
        let partial;
        if n > 0 && line[n - 1] == b'\n' {
            n -= 1;
            // Strip trailing whitespace.
            while n > 0 && matches!(line[n - 1], b' ' | b'\t' | b'\r') {
                n -= 1;
            }
            partial = false;
        } else {
            partial = true;
        }

        let mut line_partial = partial;
        while i < n {
            let c = line[i];
            if c == b'_' && header {
                new.push(b' ');
                i += 1;
            } else if c != ESCAPE {
                new.push(c);
                i += 1;
            } else if i + 1 == n && !line_partial {
                // `=` at end of a complete line: soft break, drop it.
                line_partial = true;
                break;
            } else if i + 1 < n && line[i + 1] == ESCAPE {
                // `==` -> a single literal `=`.
                new.push(ESCAPE);
                i += 2;
            } else if i + 2 < n && ishex(line[i + 1]) && ishex(line[i + 2]) {
                new.push(unhex2(line[i + 1], line[i + 2]));
                i += 3;
            } else {
                // Bad escape sequence -- leave it in.
                new.push(c);
                i += 1;
            }
        }

        if !line_partial {
            out.append(&mut new);
            out.push(b'\n');
            new.clear();
        }
    }
    if !new.is_empty() {
        out.append(&mut new);
    }
    out
}

// ── BytesIO file-object helpers (mirrors io_mod's `_buffer`/`_pos` layout) ──

fn file_read_remaining(file: MbValue) -> Vec<u8> {
    if let Some(ptr) = file.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                let buf = f
                    .get("_buffer")
                    .and_then(|v| read_bytes_field(v))
                    .unwrap_or_default();
                let pos = f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0) as usize;
                let out = if pos < buf.len() { buf[pos..].to_vec() } else { Vec::new() };
                f.insert("_pos".to_string(), MbValue::from_int(buf.len() as i64));
                return out;
            }
        }
    }
    Vec::new()
}

fn file_write_bytes(file: MbValue, data: &[u8]) {
    if let Some(ptr) = file.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                let mut buf = f
                    .get("_buffer")
                    .and_then(|v| read_bytes_field(v))
                    .unwrap_or_default();
                let pos = f.get("_pos").and_then(|v| v.as_int()).unwrap_or(0) as usize;
                if pos >= buf.len() {
                    if pos > buf.len() {
                        buf.resize(pos, 0);
                    }
                    buf.extend_from_slice(data);
                } else {
                    let end = pos + data.len();
                    if end > buf.len() {
                        buf.resize(end, 0);
                    }
                    buf[pos..pos + data.len()].copy_from_slice(data);
                }
                let new_pos = pos + data.len();
                f.insert("_buffer".to_string(), MbValue::from_ptr(MbObject::new_bytes(buf)));
                f.insert("_pos".to_string(), MbValue::from_int(new_pos as i64));
            }
        }
    }
}

unsafe fn read_bytes_field(v: &MbValue) -> Option<Vec<u8>> {
    v.as_ptr().map(|p| match &(*p).data {
        ObjData::Bytes(b) => b.clone(),
        ObjData::ByteArray(ref lock) => lock.read().unwrap().clone(),
        _ => Vec::new(),
    })
}

// ── Dispatchers ──

unsafe extern "C" fn dispatch_encodestring(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type("encodestring() missing required argument");
    };
    let Some(src) = as_bytes(arg) else {
        return raise_type("encodestring() argument must be bytes-like");
    };
    let quotetabs = flag(args, 1, "quotetabs");
    let header = flag(args, 2, "header");
    MbValue::from_ptr(MbObject::new_bytes(encode_bytes(&src, quotetabs, header)))
}

unsafe extern "C" fn dispatch_decodestring(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return raise_type("decodestring() missing required argument");
    };
    let Some(src) = as_bytes(arg) else {
        return raise_type("decodestring() argument must be bytes-like");
    };
    let header = flag(args, 1, "header");
    MbValue::from_ptr(MbObject::new_bytes(decode_bytes(&src, header)))
}

/// quopri.encode(input, output, quotetabs, header=False)
unsafe extern "C" fn dispatch_encode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(infp) = args.first().copied() else {
        return raise_type("encode() missing required argument 'input'");
    };
    let Some(outfp) = args.get(1).copied() else {
        return raise_type("encode() missing required argument 'output'");
    };
    // quotetabs is the 3rd positional (no default in CPython); header defaults
    // False. Both may also arrive as keyword flags in a trailing dict.
    let quotetabs = flag(args, 2, "quotetabs");
    let header = flag(args, 3, "header");
    let data = file_read_remaining(infp);
    let encoded = encode_bytes(&data, quotetabs, header);
    file_write_bytes(outfp, &encoded);
    MbValue::none()
}

/// quopri.decode(input, output, header=False)
unsafe extern "C" fn dispatch_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(infp) = args.first().copied() else {
        return raise_type("decode() missing required argument 'input'");
    };
    let Some(outfp) = args.get(1).copied() else {
        return raise_type("decode() missing required argument 'output'");
    };
    let header = flag(args, 2, "header");
    let data = file_read_remaining(infp);
    let decoded = decode_bytes(&data, header);
    file_write_bytes(outfp, &decoded);
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_es = dispatch_encodestring as *const () as usize;
    let addr_ds = dispatch_decodestring as *const () as usize;
    let addr_enc = dispatch_encode as *const () as usize;
    let addr_dec = dispatch_decode as *const () as usize;

    attrs.insert("encodestring".into(), MbValue::from_func(addr_es));
    attrs.insert("decodestring".into(), MbValue::from_func(addr_ds));
    attrs.insert("encode".into(), MbValue::from_func(addr_enc));
    attrs.insert("decode".into(), MbValue::from_func(addr_dec));

    attrs.insert("ESCAPE".into(), MbValue::from_int(ESCAPE as i64));
    attrs.insert("MAXLINESIZE".into(), MbValue::from_int(MAXLINESIZE as i64));
    attrs.insert("HEX".into(), MbValue::from_int(16));
    // CPython exposes b2a_qp/a2b_qp as the binascii fast path; the test
    // toggles them to None to force the pure-Python branch, so expose None.
    attrs.insert("b2a_qp".into(), MbValue::none());
    attrs.insert("a2b_qp".into(), MbValue::none());

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_es as u64);
        set.insert(addr_ds as u64);
        set.insert(addr_enc as u64);
        set.insert(addr_dec as u64);
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
        assert_eq!(encode_bytes(b"caf\xc3\xa9", false, false), b"caf=C3=A9");
    }

    #[test]
    fn encodes_equals_sign() {
        assert_eq!(encode_bytes(b"a=b", false, false), b"a=3Db");
    }

    #[test]
    fn keeps_plain_ascii() {
        assert_eq!(encode_bytes(b"hello world", false, false), b"hello world");
    }

    #[test]
    fn quotetabs_quotes_space_and_tab() {
        assert_eq!(encode_bytes(b"a b\tc", true, false), b"a=20b=09c");
    }

    #[test]
    fn embedded_ws_quotetabs() {
        assert_eq!(encode_bytes(b"hello world", true, false), b"hello=20world");
        assert_eq!(encode_bytes(b"hello\tworld", true, false), b"hello=09world");
    }

    #[test]
    fn trailing_space_encoded() {
        assert_eq!(encode_bytes(b"hello ", false, false), b"hello=20");
        assert_eq!(encode_bytes(b"hello\t", false, false), b"hello=09");
    }

    #[test]
    fn header_space_to_underscore() {
        assert_eq!(encode_bytes(b"hello world", false, true), b"hello_world");
        // `_` must be escaped when header=True.
        assert_eq!(encode_bytes(b"hello_world", false, true), b"hello=5Fworld");
    }

    #[test]
    fn soft_break_splits_escape() {
        // 48 'x' + 8 high bytes (each =XX) + 48 'x'. CPython breaks the
        // *already-quoted* string at column 75, which falls mid-escape, so a
        // soft `=\n` lands inside the run of trailing 'x's (expected value
        // captured from CPython 3.12 `quopri.encodestring`).
        let mut src = vec![b'x'; 48];
        src.extend_from_slice(b"\xd8\xd9\xda\xdb\xdc\xdd\xde\xdf");
        src.extend(vec![b'x'; 48]);
        let expected = b"xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx=D8=D9=DA=DB=DC=DD=DE=DFxxx=\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        assert_eq!(encode_bytes(&src, false, false), expected);
    }

    #[test]
    fn decode_double_equals() {
        assert_eq!(decode_bytes(b"123==four", false), b"123=four");
    }

    #[test]
    fn decode_roundtrip_high() {
        assert_eq!(decode_bytes(b"caf=C3=A9", false), b"caf\xc3\xa9");
    }

    #[test]
    fn decode_soft_break() {
        assert_eq!(decode_bytes(b"hello=\nworld", false), b"helloworld");
    }

    #[test]
    fn decode_header_underscore() {
        assert_eq!(decode_bytes(b"a_b", true), b"a b");
    }

    #[test]
    fn encode_decode_idempotent() {
        // decodestring(encodestring(x)) == x for ascii-with-newlines payloads.
        let x = b"hello\n        there\n        world\n";
        let enc = encode_bytes(x, false, false);
        assert_eq!(decode_bytes(&enc, false), x);
    }

    #[test]
    fn dispatch_encode_header_via_kwargs_dict() {
        // Simulate `encodestring(b'hello world', header=True)` -> trailing dict.
        let arg = MbValue::from_ptr(MbObject::new_bytes(b"hello world".to_vec()));
        let kw = super::super::super::dict_ops::mb_dict_new();
        super::super::super::dict_ops::mb_dict_setitem(
            kw,
            MbValue::from_ptr(MbObject::new_str("header".to_string())),
            MbValue::from_bool(true),
        );
        let args = [arg, kw];
        let v = unsafe { dispatch_encodestring(args.as_ptr(), 2) };
        assert_eq!(bytes_of(v), b"hello_world");
    }
}
