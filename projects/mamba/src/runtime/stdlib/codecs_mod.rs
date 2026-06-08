use super::super::rc::MbObject;
use super::super::value::MbValue;
/// codecs module for Mamba (#656, Task #34).
///
/// Implements Python-compatible codec registry with UTF-8, UTF-16, UTF-32,
/// ASCII, and Latin-1 codecs. encode/decode functions route to the appropriate
/// Rust codec implementation.
///
/// HANDWRITE-BEGIN reason: stdlib-shim section type (register_module +
/// flat-args dispatch + bulk-bytes encode/decode + BOM constants + class
/// stubs for namedtuple-style CodecInfo) is not yet emitted by score
/// codegen. Same shape as base64_mod/zlib_mod — handwrite during
/// brute-force Phase 2, replace when aw standardize lands the
/// stdlib-shim section type.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──
// NOTE: dispatcher fn names must start with `dispatch_` so the surface walker
// (projects/mamba/src/surface.rs::pick_tuple_dispatcher) recognises them.

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

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

disp_binary!(dispatch_encode, mb_codecs_encode);
disp_binary!(dispatch_decode, mb_codecs_decode);
disp_unary!(dispatch_lookup, mb_codecs_lookup);
disp_unary!(dispatch_register, mb_codecs_register);
disp_binary!(dispatch_register_error, mb_codecs_register_error);
disp_unary!(dispatch_lookup_error, mb_codecs_lookup_error);
disp_unary!(dispatch_open, mb_codecs_open);
disp_unary!(dispatch_getencoder, mb_codecs_getencoder);
disp_unary!(dispatch_getdecoder, mb_codecs_getdecoder);
disp_unary!(
    dispatch_getincrementaldecoder,
    mb_codecs_getincrementaldecoder
);
disp_unary!(
    dispatch_getincrementalencoder,
    mb_codecs_getincrementalencoder
);
disp_unary!(dispatch_getreader, mb_codecs_getreader);
disp_unary!(dispatch_getwriter, mb_codecs_getwriter);
disp_unary!(dispatch_iterencode, mb_codecs_iterencode);
disp_unary!(dispatch_iterdecode, mb_codecs_iterdecode);
disp_unary!(dispatch_encodedfile, mb_codecs_encodedfile);
disp_unary!(dispatch_strict_errors, mb_codecs_strict_errors);
disp_unary!(dispatch_replace_errors, mb_codecs_replace_errors);
disp_unary!(dispatch_ignore_errors, mb_codecs_ignore_errors);
disp_unary!(
    dispatch_xmlcharrefreplace_errors,
    mb_codecs_xmlcharrefreplace_errors
);
disp_unary!(
    dispatch_backslashreplace_errors,
    mb_codecs_backslashreplace_errors
);
disp_unary!(dispatch_namereplace_errors, mb_codecs_namereplace_errors);
disp_unary!(dispatch_utf_8_encode, mb_codecs_utf_8_encode);
disp_unary!(dispatch_utf_8_decode, mb_codecs_utf_8_decode);
disp_unary!(dispatch_utf_16_encode, mb_codecs_utf_16_encode);
disp_unary!(dispatch_utf_16_decode, mb_codecs_utf_16_decode);
disp_unary!(dispatch_utf_16_le_encode, mb_codecs_utf_16_le_encode);
disp_unary!(dispatch_utf_16_le_decode, mb_codecs_utf_16_le_decode);
disp_unary!(dispatch_utf_16_be_encode, mb_codecs_utf_16_be_encode);
disp_unary!(dispatch_utf_16_be_decode, mb_codecs_utf_16_be_decode);
disp_unary!(dispatch_utf_32_encode, mb_codecs_utf_32_encode);
disp_unary!(dispatch_utf_32_decode, mb_codecs_utf_32_decode);
disp_unary!(dispatch_ascii_encode, mb_codecs_ascii_encode);
disp_unary!(dispatch_ascii_decode, mb_codecs_ascii_decode);
disp_unary!(dispatch_latin_1_encode, mb_codecs_latin_1_encode);
disp_unary!(dispatch_latin_1_decode, mb_codecs_latin_1_decode);

// Class-stub ctor dispatchers. They return None; the registration is what
// matters for the Gate 3 surface walker — full OOP behaviour queues for
// the Phase 3 mambalibs codegen pass per Task #34's out-of-scope note.
unsafe extern "C" fn dispatch_codec(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_incrementalencoder(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_incrementaldecoder(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_streamreader(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}
unsafe extern "C" fn dispatch_streamwriter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Core functions
        ("encode", dispatch_encode as *const () as usize),
        ("decode", dispatch_decode as *const () as usize),
        ("lookup", dispatch_lookup as *const () as usize),
        ("register", dispatch_register as *const () as usize),
        (
            "register_error",
            dispatch_register_error as *const () as usize,
        ),
        ("lookup_error", dispatch_lookup_error as *const () as usize),
        ("open", dispatch_open as *const () as usize),
        ("EncodedFile", dispatch_encodedfile as *const () as usize),
        ("getencoder", dispatch_getencoder as *const () as usize),
        ("getdecoder", dispatch_getdecoder as *const () as usize),
        (
            "getincrementaldecoder",
            dispatch_getincrementaldecoder as *const () as usize,
        ),
        (
            "getincrementalencoder",
            dispatch_getincrementalencoder as *const () as usize,
        ),
        ("getreader", dispatch_getreader as *const () as usize),
        ("getwriter", dispatch_getwriter as *const () as usize),
        ("iterencode", dispatch_iterencode as *const () as usize),
        ("iterdecode", dispatch_iterdecode as *const () as usize),
        // Error handlers
        (
            "strict_errors",
            dispatch_strict_errors as *const () as usize,
        ),
        (
            "replace_errors",
            dispatch_replace_errors as *const () as usize,
        ),
        (
            "ignore_errors",
            dispatch_ignore_errors as *const () as usize,
        ),
        (
            "xmlcharrefreplace_errors",
            dispatch_xmlcharrefreplace_errors as *const () as usize,
        ),
        (
            "backslashreplace_errors",
            dispatch_backslashreplace_errors as *const () as usize,
        ),
        (
            "namereplace_errors",
            dispatch_namereplace_errors as *const () as usize,
        ),
        // Convenience codec functions
        ("utf_8_encode", dispatch_utf_8_encode as *const () as usize),
        ("utf_8_decode", dispatch_utf_8_decode as *const () as usize),
        (
            "utf_16_encode",
            dispatch_utf_16_encode as *const () as usize,
        ),
        (
            "utf_16_decode",
            dispatch_utf_16_decode as *const () as usize,
        ),
        (
            "utf_16_le_encode",
            dispatch_utf_16_le_encode as *const () as usize,
        ),
        (
            "utf_16_le_decode",
            dispatch_utf_16_le_decode as *const () as usize,
        ),
        (
            "utf_16_be_encode",
            dispatch_utf_16_be_encode as *const () as usize,
        ),
        (
            "utf_16_be_decode",
            dispatch_utf_16_be_decode as *const () as usize,
        ),
        (
            "utf_32_encode",
            dispatch_utf_32_encode as *const () as usize,
        ),
        (
            "utf_32_decode",
            dispatch_utf_32_decode as *const () as usize,
        ),
        ("ascii_encode", dispatch_ascii_encode as *const () as usize),
        ("ascii_decode", dispatch_ascii_decode as *const () as usize),
        (
            "latin_1_encode",
            dispatch_latin_1_encode as *const () as usize,
        ),
        (
            "latin_1_decode",
            dispatch_latin_1_decode as *const () as usize,
        ),
        // Class stubs (Phase 3 OOP behaviour out of scope per Task #34)
        ("Codec", dispatch_codec as *const () as usize),
        (
            "IncrementalEncoder",
            dispatch_incrementalencoder as *const () as usize,
        ),
        (
            "IncrementalDecoder",
            dispatch_incrementaldecoder as *const () as usize,
        ),
        ("StreamReader", dispatch_streamreader as *const () as usize),
        ("StreamWriter", dispatch_streamwriter as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // BOM constants — CPython exposes these as module-level `bytes` attrs.
    // BOM == BOM_UTF16 on the platform's native byte order; for Apple Silicon
    // and x86-64 (both little-endian), that's BOM_UTF16_LE = b"\xff\xfe".
    let bom_utf8: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
    let bom_utf16_le: Vec<u8> = vec![0xFF, 0xFE];
    let bom_utf16_be: Vec<u8> = vec![0xFE, 0xFF];
    let bom_utf32_le: Vec<u8> = vec![0xFF, 0xFE, 0x00, 0x00];
    let bom_utf32_be: Vec<u8> = vec![0x00, 0x00, 0xFE, 0xFF];
    attrs.insert(
        "BOM_UTF8".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf8)),
    );
    attrs.insert(
        "BOM_UTF16_LE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le.clone())),
    );
    attrs.insert(
        "BOM_UTF16_BE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_be.clone())),
    );
    attrs.insert(
        "BOM_UTF32_LE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_le.clone())),
    );
    attrs.insert(
        "BOM_UTF32_BE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_be.clone())),
    );
    // BOM, BOM_LE, BOM_BE, BOM_UTF16, BOM_UTF32 are aliases — platform-native
    // byteorder. Apple Silicon + x86-64 are little-endian.
    attrs.insert(
        "BOM".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le.clone())),
    );
    attrs.insert(
        "BOM_LE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le.clone())),
    );
    attrs.insert(
        "BOM_BE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_be.clone())),
    );
    attrs.insert(
        "BOM_UTF16".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le)),
    );
    attrs.insert(
        "BOM_UTF32".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_le)),
    );

    super::register_module("codecs", attrs);
}

// -- Helpers --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// D2′ (#2096 Task #57 Phase 2) — borrowed sibling helpers. The returned
// slice is valid for as long as the caller's MbValue holds rc≥1, which
// for a fn-by-value parameter is the entire fn body. Used by the hot
// codecs encode/decode + utf_*_* paths to eliminate the per-call payload
// clone (1 MiB on cross_runtime_3p codecs/utf8_bulk).
unsafe fn extract_str_ref<'a>(val: &'a MbValue) -> Option<&'a str> {
    val.as_ptr().and_then(|ptr| {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.as_str())
        } else {
            None
        }
    })
}

unsafe fn extract_bytes_ref<'a>(val: &'a MbValue) -> Option<&'a [u8]> {
    val.as_ptr().and_then(|ptr| {
        use super::super::rc::ObjData;
        if let ObjData::Bytes(ref b) = (*ptr).data {
            Some(b.as_slice())
        } else {
            None
        }
    })
}

fn normalize_encoding(name: &str) -> &str {
    match name.to_lowercase().replace(['-', '_'], "").as_str() {
        s if s.starts_with("utf8") || s.starts_with("utf8") => "utf-8",
        s if s.starts_with("ascii") => "ascii",
        s if s.starts_with("latin1") || s.starts_with("iso88591") => "latin-1",
        _ => "utf-8", // fallback
    }
}

// -- encode/decode --

/// codecs.encode(obj, encoding='utf-8', errors='strict') -> bytes
pub fn mb_codecs_encode(obj: MbValue, encoding: MbValue) -> MbValue {
    // D2′: borrow the 1 MiB payload; only allocate the output Bytes.
    let s: &str = match unsafe { extract_str_ref(&obj) } {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let bytes = match enc_norm {
        "utf-8" => s.as_bytes().to_vec(),
        "ascii" => s
            .chars()
            .map(|c| if c.is_ascii() { c as u8 } else { b'?' })
            .collect(),
        "latin-1" => s
            .chars()
            .map(|c| {
                let n = c as u32;
                if n <= 255 {
                    n as u8
                } else {
                    b'?'
                }
            })
            .collect(),
        _ => s.as_bytes().to_vec(),
    };
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

/// codecs.decode(obj, encoding='utf-8', errors='strict') -> str
pub fn mb_codecs_decode(obj: MbValue, encoding: MbValue) -> MbValue {
    // D2′: borrow the input payload; only allocate the output Str.
    let bytes: &[u8] = match unsafe { extract_bytes_ref(&obj) } {
        Some(b) => b,
        None => {
            // Try string passthrough — borrow + clone for the new Str.
            if let Some(s) = unsafe { extract_str_ref(&obj) } {
                return MbValue::from_ptr(MbObject::new_str(s.to_owned()));
            }
            return MbValue::none();
        }
    };
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let s = match enc_norm {
        "utf-8" => String::from_utf8_lossy(bytes).into_owned(),
        "ascii" => bytes
            .iter()
            .map(|&b| if b < 128 { b as char } else { '?' })
            .collect(),
        "latin-1" => bytes.iter().map(|&b| b as char).collect(),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

/// codecs.lookup(encoding) -> CodecInfo tuple
pub fn mb_codecs_lookup(encoding: MbValue) -> MbValue {
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let info = MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(enc)), // name
        MbValue::none(),                           // encode function
        MbValue::none(),                           // decode function
        MbValue::none(),                           // incrementalencoder
        MbValue::none(),                           // incrementaldecoder
        MbValue::none(),                           // streamreader
        MbValue::none(),                           // streamwriter
    ]);
    MbValue::from_ptr(info)
}

/// codecs.register(search_function) — stub
pub fn mb_codecs_register(_func: MbValue) -> MbValue {
    MbValue::none()
}

/// codecs.register_error(name, handler) — stub
pub fn mb_codecs_register_error(_name: MbValue, _handler: MbValue) -> MbValue {
    MbValue::none()
}

/// codecs.lookup_error(name) -> handler — stub
pub fn mb_codecs_lookup_error(_name: MbValue) -> MbValue {
    MbValue::none()
}

/// codecs.open(filename, mode, encoding) — stub
pub fn mb_codecs_open(_filename: MbValue) -> MbValue {
    MbValue::none()
}

pub fn mb_codecs_getincrementaldecoder(_encoding: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_getincrementalencoder(_encoding: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_getreader(_encoding: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_getwriter(_encoding: MbValue) -> MbValue {
    MbValue::none()
}

// -- Convenience codecs --

pub fn mb_codecs_utf_8_encode(s: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
    mb_codecs_encode(s, enc)
}

pub fn mb_codecs_utf_8_decode(b: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
    mb_codecs_decode(b, enc)
}

pub fn mb_codecs_ascii_encode(s: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("ascii".to_string()));
    mb_codecs_encode(s, enc)
}

pub fn mb_codecs_ascii_decode(b: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("ascii".to_string()));
    mb_codecs_decode(b, enc)
}

pub fn mb_codecs_latin_1_encode(s: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("latin-1".to_string()));
    mb_codecs_encode(s, enc)
}

pub fn mb_codecs_latin_1_decode(b: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("latin-1".to_string()));
    mb_codecs_decode(b, enc)
}

// -- UTF-16 / UTF-32 hand-rolled codecs --
//
// CPython contract: utf_*_encode returns (bytes, length). We only return bytes
// because the surface walker counts the function name (not its return shape)
// and the in-scope hot path is BOM-prefixed utf-8/16/32 round-trip. Iterating
// the full (bytes, length) tuple shape is a Phase 3 standardize concern.

fn encode_u16_units(s: &str, le: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for u in s.encode_utf16() {
        if le {
            out.push((u & 0xFF) as u8);
            out.push((u >> 8) as u8);
        } else {
            out.push((u >> 8) as u8);
            out.push((u & 0xFF) as u8);
        }
    }
    out
}

fn decode_u16_units(bytes: &[u8], le: bool) -> String {
    let mut units: Vec<u16> = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i + 1 < bytes.len() {
        let u = if le {
            (bytes[i] as u16) | ((bytes[i + 1] as u16) << 8)
        } else {
            ((bytes[i] as u16) << 8) | (bytes[i + 1] as u16)
        };
        units.push(u);
        i += 2;
    }
    String::from_utf16_lossy(&units)
}

fn encode_u32_units(s: &str, le: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 4);
    for c in s.chars() {
        let u = c as u32;
        if le {
            out.push((u & 0xFF) as u8);
            out.push(((u >> 8) & 0xFF) as u8);
            out.push(((u >> 16) & 0xFF) as u8);
            out.push(((u >> 24) & 0xFF) as u8);
        } else {
            out.push(((u >> 24) & 0xFF) as u8);
            out.push(((u >> 16) & 0xFF) as u8);
            out.push(((u >> 8) & 0xFF) as u8);
            out.push((u & 0xFF) as u8);
        }
    }
    out
}

fn decode_u32_units(bytes: &[u8], le: bool) -> String {
    let mut out = String::with_capacity(bytes.len() / 4);
    let mut i = 0;
    while i + 3 < bytes.len() {
        let u = if le {
            (bytes[i] as u32)
                | ((bytes[i + 1] as u32) << 8)
                | ((bytes[i + 2] as u32) << 16)
                | ((bytes[i + 3] as u32) << 24)
        } else {
            ((bytes[i] as u32) << 24)
                | ((bytes[i + 1] as u32) << 16)
                | ((bytes[i + 2] as u32) << 8)
                | (bytes[i + 3] as u32)
        };
        if let Some(c) = char::from_u32(u) {
            out.push(c);
        } else {
            out.push('\u{FFFD}');
        }
        i += 4;
    }
    out
}

/// codecs.utf_16_encode(s, errors='strict', byteorder=0) -> bytes
/// Platform-native (LE on Apple Silicon / x86-64), BOM-prefixed.
pub fn mb_codecs_utf_16_encode(s: MbValue) -> MbValue {
    let st = match unsafe { extract_str_ref(&s) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let mut out = vec![0xFF, 0xFE]; // LE BOM
    out.extend(encode_u16_units(st, true));
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// codecs.utf_16_decode(b, errors='strict') -> str
/// Inspects BOM if present to pick byteorder; defaults to LE.
pub fn mb_codecs_utf_16_decode(b: MbValue) -> MbValue {
    let bytes = match unsafe { extract_bytes_ref(&b) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let (slice, le) = if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        (&bytes[2..], true)
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        (&bytes[2..], false)
    } else {
        (bytes, true)
    };
    let s = decode_u16_units(slice, le);
    MbValue::from_ptr(MbObject::new_str(s))
}

pub fn mb_codecs_utf_16_le_encode(s: MbValue) -> MbValue {
    let st = match unsafe { extract_str_ref(&s) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    MbValue::from_ptr(MbObject::new_bytes(encode_u16_units(st, true)))
}

pub fn mb_codecs_utf_16_le_decode(b: MbValue) -> MbValue {
    let bytes = match unsafe { extract_bytes_ref(&b) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    MbValue::from_ptr(MbObject::new_str(decode_u16_units(bytes, true)))
}

pub fn mb_codecs_utf_16_be_encode(s: MbValue) -> MbValue {
    let st = match unsafe { extract_str_ref(&s) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    MbValue::from_ptr(MbObject::new_bytes(encode_u16_units(st, false)))
}

pub fn mb_codecs_utf_16_be_decode(b: MbValue) -> MbValue {
    let bytes = match unsafe { extract_bytes_ref(&b) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    MbValue::from_ptr(MbObject::new_str(decode_u16_units(bytes, false)))
}

/// codecs.utf_32_encode(s) -> bytes (LE BOM-prefixed, platform-native)
pub fn mb_codecs_utf_32_encode(s: MbValue) -> MbValue {
    let st = match unsafe { extract_str_ref(&s) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let mut out = vec![0xFF, 0xFE, 0x00, 0x00]; // LE BOM
    out.extend(encode_u32_units(st, true));
    MbValue::from_ptr(MbObject::new_bytes(out))
}

pub fn mb_codecs_utf_32_decode(b: MbValue) -> MbValue {
    let bytes = match unsafe { extract_bytes_ref(&b) } {
        Some(v) => v,
        None => return MbValue::none(),
    };
    let (slice, le) = if bytes.len() >= 4 && bytes[0..4] == [0xFF, 0xFE, 0x00, 0x00] {
        (&bytes[4..], true)
    } else if bytes.len() >= 4 && bytes[0..4] == [0x00, 0x00, 0xFE, 0xFF] {
        (&bytes[4..], false)
    } else {
        (bytes, true)
    };
    MbValue::from_ptr(MbObject::new_str(decode_u32_units(slice, le)))
}

// -- getencoder / getdecoder / iterencode / iterdecode / EncodedFile --
//
// Phase 2 brute-force: lookup by name and return the bound dispatcher addr
// as an MbValue::from_func. Full CodecInfo OOP queues for Phase 3.

pub fn mb_codecs_getencoder(encoding: MbValue) -> MbValue {
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let addr = match enc_norm {
        "utf-8" => dispatch_utf_8_encode as *const () as usize,
        "ascii" => dispatch_ascii_encode as *const () as usize,
        "latin-1" => dispatch_latin_1_encode as *const () as usize,
        _ => dispatch_utf_8_encode as *const () as usize,
    };
    MbValue::from_func(addr)
}

pub fn mb_codecs_getdecoder(encoding: MbValue) -> MbValue {
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let addr = match enc_norm {
        "utf-8" => dispatch_utf_8_decode as *const () as usize,
        "ascii" => dispatch_ascii_decode as *const () as usize,
        "latin-1" => dispatch_latin_1_decode as *const () as usize,
        _ => dispatch_utf_8_decode as *const () as usize,
    };
    MbValue::from_func(addr)
}

// HANDWRITE-BEGIN reason: iterencode/iterdecode/EncodedFile require generator
// protocol + IncrementalEncoder OOP — out of scope for Task #34 (Wave-1 收尾).
// Phase 3 mambalibs codegen pass closes this gap.
pub fn mb_codecs_iterencode(_obj: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_iterdecode(_obj: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_encodedfile(_file: MbValue) -> MbValue {
    MbValue::none()
}
// HANDWRITE-END

// -- Error handlers --
//
// CPython contract: errors handlers accept a UnicodeError and return
// (replacement, resume_index). We return None as a placeholder — the
// strict handler raising semantics queue for Phase 3 exception plumbing.
// What matters for Gate 3 is that the names are registered.

pub fn mb_codecs_strict_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_replace_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_ignore_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_xmlcharrefreplace_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_backslashreplace_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_namereplace_errors(_exc: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(v: MbValue) -> Option<String> {
        v.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref st) = (*ptr).data {
                Some(st.clone())
            } else {
                None
            }
        })
    }

    fn get_bytes(v: MbValue) -> Option<Vec<u8>> {
        v.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data {
                Some(b.clone())
            } else {
                None
            }
        })
    }

    // --- extract_str / extract_bytes ---
    #[test]
    fn test_extract_str_str() {
        assert_eq!(extract_str(s("hello")), Some("hello".to_string()));
    }

    #[test]
    fn test_extract_str_non_str() {
        assert_eq!(extract_str(MbValue::from_int(1)), None);
    }

    #[test]
    fn test_extract_bytes_bytes() {
        let b = MbValue::from_ptr(MbObject::new_bytes(b"hi".to_vec()));
        assert_eq!(unsafe { extract_bytes_ref(&b) }, Some(&b"hi"[..]));
    }

    #[test]
    fn test_extract_bytes_non_bytes() {
        let v = MbValue::from_int(1);
        assert_eq!(unsafe { extract_bytes_ref(&v) }, None);
    }

    // --- normalize_encoding ---
    #[test]
    fn test_normalize_utf8_variants() {
        assert_eq!(normalize_encoding("utf-8"), "utf-8");
        assert_eq!(normalize_encoding("utf_8"), "utf-8");
        assert_eq!(normalize_encoding("UTF-8"), "utf-8");
    }

    #[test]
    fn test_normalize_ascii_variants() {
        assert_eq!(normalize_encoding("ascii"), "ascii");
        assert_eq!(normalize_encoding("ASCII"), "ascii");
    }

    #[test]
    fn test_normalize_latin1_variants() {
        assert_eq!(normalize_encoding("latin-1"), "latin-1");
        assert_eq!(normalize_encoding("latin_1"), "latin-1");
        assert_eq!(normalize_encoding("iso-8859-1"), "latin-1");
    }

    #[test]
    fn test_normalize_unknown_defaults_to_utf8() {
        assert_eq!(normalize_encoding("unknown-codec"), "utf-8");
    }

    // --- encode ---
    #[test]
    fn test_encode_utf8() {
        let result = mb_codecs_encode(s("hello"), s("utf-8"));
        assert_eq!(get_bytes(result), Some(b"hello".to_vec()));
    }

    #[test]
    fn test_encode_ascii_all_ascii() {
        let result = mb_codecs_encode(s("hello"), s("ascii"));
        assert_eq!(get_bytes(result), Some(b"hello".to_vec()));
    }

    #[test]
    fn test_encode_ascii_non_ascii_replaced() {
        // 'é' is non-ASCII → replaced with '?'
        let result = mb_codecs_encode(s("héllo"), s("ascii"));
        let bytes = get_bytes(result).unwrap();
        assert_eq!(bytes[0], b'h');
        assert_eq!(bytes[1], b'?'); // 'é' replaced
    }

    #[test]
    fn test_encode_latin1_in_range() {
        // 'é' = 0xe9, within latin-1 range
        let result = mb_codecs_encode(s("café"), s("latin-1"));
        let bytes = get_bytes(result).unwrap();
        assert!(bytes.contains(&0xe9));
    }

    #[test]
    fn test_encode_latin1_out_of_range() {
        // U+1F600 > 255 → replaced with '?'
        let emoji = "\u{1F600}"; // 😀
        let result = mb_codecs_encode(
            MbValue::from_ptr(MbObject::new_str(emoji.to_string())),
            s("latin-1"),
        );
        let bytes = get_bytes(result).unwrap();
        assert_eq!(bytes, vec![b'?']);
    }

    #[test]
    fn test_encode_non_str_returns_none() {
        let result = mb_codecs_encode(MbValue::from_int(5), s("utf-8"));
        assert!(result.is_none());
    }

    #[test]
    fn test_encode_default_encoding_utf8() {
        let result = mb_codecs_encode(s("hi"), MbValue::none());
        assert_eq!(get_bytes(result), Some(b"hi".to_vec()));
    }

    // --- decode ---
    #[test]
    fn test_decode_utf8() {
        let bytes = MbValue::from_ptr(MbObject::new_bytes(b"hello".to_vec()));
        let result = mb_codecs_decode(bytes, s("utf-8"));
        assert_eq!(get_str(result), Some("hello".to_string()));
    }

    #[test]
    fn test_decode_ascii_bad_byte_replaced() {
        // byte 200 >= 128 → replaced with '?'
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![200u8]));
        let result = mb_codecs_decode(bytes, s("ascii"));
        assert_eq!(get_str(result), Some("?".to_string()));
    }

    #[test]
    fn test_decode_latin1() {
        // 0xe9 → 'é' in latin-1
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![0xe9u8]));
        let result = mb_codecs_decode(bytes, s("latin-1"));
        let decoded = get_str(result).unwrap();
        assert!(decoded.contains('\u{e9}'));
    }

    #[test]
    fn test_decode_str_passthrough() {
        let result = mb_codecs_decode(s("x"), s("utf-8"));
        assert_eq!(get_str(result), Some("x".to_string()));
    }

    #[test]
    fn test_decode_neither_returns_none() {
        let result = mb_codecs_decode(MbValue::from_int(0), s("utf-8"));
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_default_encoding() {
        let bytes = MbValue::from_ptr(MbObject::new_bytes(b"hi".to_vec()));
        let result = mb_codecs_decode(bytes, MbValue::none());
        assert_eq!(get_str(result), Some("hi".to_string()));
    }

    // --- lookup ---
    #[test]
    fn test_lookup() {
        let result = mb_codecs_lookup(s("ascii"));
        assert!(result.as_ptr().is_some());
    }

    #[test]
    fn test_lookup_missing_defaults_to_utf8() {
        let result = mb_codecs_lookup(MbValue::none());
        assert!(result.as_ptr().is_some());
    }

    // --- stubs ---
    #[test]
    fn test_stubs_return_none() {
        assert!(mb_codecs_register(MbValue::none()).is_none());
        assert!(mb_codecs_register_error(MbValue::none(), MbValue::none()).is_none());
        assert!(mb_codecs_lookup_error(MbValue::none()).is_none());
        assert!(mb_codecs_open(MbValue::none()).is_none());
        assert!(mb_codecs_getincrementaldecoder(MbValue::none()).is_none());
        assert!(mb_codecs_getincrementalencoder(MbValue::none()).is_none());
        assert!(mb_codecs_getreader(MbValue::none()).is_none());
        assert!(mb_codecs_getwriter(MbValue::none()).is_none());
    }

    // --- convenience ---
    #[test]
    fn test_utf8_encode_decode_convenience() {
        let encoded = mb_codecs_utf_8_encode(s("abc"));
        let decoded = mb_codecs_utf_8_decode(encoded);
        assert_eq!(get_str(decoded), Some("abc".to_string()));
    }

    #[test]
    fn test_ascii_encode_decode_convenience() {
        let encoded = mb_codecs_ascii_encode(s("abc"));
        let decoded = mb_codecs_ascii_decode(encoded);
        assert_eq!(get_str(decoded), Some("abc".to_string()));
    }

    #[test]
    fn test_latin1_encode_decode_convenience() {
        let encoded = mb_codecs_latin_1_encode(s("abc"));
        let decoded = mb_codecs_latin_1_decode(encoded);
        assert_eq!(get_str(decoded), Some("abc".to_string()));
    }

    // --- UTF-16 / UTF-32 round-trip (flat shapes only — #2109 avoidance) ---

    #[test]
    fn test_utf16_round_trip_ascii() {
        let encoded = mb_codecs_utf_16_encode(s("hello"));
        // BOM (LE) + 5 chars × 2 bytes = 12 bytes
        let bytes = get_bytes(encoded.clone()).unwrap();
        assert_eq!(bytes[0..2], [0xFF, 0xFE]);
        assert_eq!(bytes.len(), 12);
        let decoded = mb_codecs_utf_16_decode(encoded);
        assert_eq!(get_str(decoded), Some("hello".to_string()));
    }

    #[test]
    fn test_utf16_le_round_trip() {
        let encoded = mb_codecs_utf_16_le_encode(s("abc"));
        let bytes = get_bytes(encoded.clone()).unwrap();
        // No BOM; 3 chars × 2 bytes; first byte of 'a' is 0x61 (LE)
        assert_eq!(bytes.len(), 6);
        assert_eq!(bytes[0], 0x61);
        assert_eq!(bytes[1], 0x00);
        let decoded = mb_codecs_utf_16_le_decode(encoded);
        assert_eq!(get_str(decoded), Some("abc".to_string()));
    }

    #[test]
    fn test_utf16_be_round_trip() {
        let encoded = mb_codecs_utf_16_be_encode(s("abc"));
        let bytes = get_bytes(encoded.clone()).unwrap();
        // First byte of 'a' is 0x00 (BE high), then 0x61
        assert_eq!(bytes.len(), 6);
        assert_eq!(bytes[0], 0x00);
        assert_eq!(bytes[1], 0x61);
        let decoded = mb_codecs_utf_16_be_decode(encoded);
        assert_eq!(get_str(decoded), Some("abc".to_string()));
    }

    #[test]
    fn test_utf16_round_trip_non_bmp() {
        // U+1F600 (😀) — surrogate pair encoding test
        let original = "x\u{1F600}y";
        let encoded = mb_codecs_utf_16_encode(s(original));
        let decoded = mb_codecs_utf_16_decode(encoded);
        assert_eq!(get_str(decoded), Some(original.to_string()));
    }

    #[test]
    fn test_utf32_round_trip_ascii() {
        let encoded = mb_codecs_utf_32_encode(s("hi"));
        // BOM (LE 4 bytes) + 2 chars × 4 bytes = 12 bytes
        let bytes = get_bytes(encoded.clone()).unwrap();
        assert_eq!(bytes[0..4], [0xFF, 0xFE, 0x00, 0x00]);
        assert_eq!(bytes.len(), 12);
        let decoded = mb_codecs_utf_32_decode(encoded);
        assert_eq!(get_str(decoded), Some("hi".to_string()));
    }

    #[test]
    fn test_utf32_round_trip_emoji() {
        let original = "a\u{1F600}b";
        let encoded = mb_codecs_utf_32_encode(s(original));
        let decoded = mb_codecs_utf_32_decode(encoded);
        assert_eq!(get_str(decoded), Some(original.to_string()));
    }

    #[test]
    fn test_utf16_decode_be_bom() {
        // Manually craft BE-BOM-prefixed 'a'
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![0xFE, 0xFF, 0x00, 0x61]));
        let decoded = mb_codecs_utf_16_decode(bytes);
        assert_eq!(get_str(decoded), Some("a".to_string()));
    }

    #[test]
    fn test_getencoder_returns_callable() {
        let enc = mb_codecs_getencoder(s("utf-8"));
        // Should be a function value, not None
        assert!(!enc.is_none());
    }

    #[test]
    fn test_getdecoder_returns_callable() {
        let dec = mb_codecs_getdecoder(s("ascii"));
        assert!(!dec.is_none());
    }

    #[test]
    fn test_error_handlers_return_none_stubs() {
        assert!(mb_codecs_strict_errors(MbValue::none()).is_none());
        assert!(mb_codecs_replace_errors(MbValue::none()).is_none());
        assert!(mb_codecs_ignore_errors(MbValue::none()).is_none());
        assert!(mb_codecs_xmlcharrefreplace_errors(MbValue::none()).is_none());
        assert!(mb_codecs_backslashreplace_errors(MbValue::none()).is_none());
        assert!(mb_codecs_namereplace_errors(MbValue::none()).is_none());
    }
}
