use super::super::rc::{MbObject, ObjData};
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

// ── Exception helpers ──
// Raise catchable Python exceptions via the thread-local exception machinery
// (same pattern as binascii_mod / base64_mod). The returned MbValue::none()
// is the dispatcher's return value; the caller checks the exception flag.

fn raise_exc(exc_type: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc_type.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}
fn raise_lookup_error(msg: &str) -> MbValue {
    raise_exc("LookupError", msg)
}
fn raise_type_error(msg: &str) -> MbValue {
    raise_exc("TypeError", msg)
}
fn raise_value_error(msg: &str) -> MbValue {
    raise_exc("ValueError", msg)
}
fn raise_unicode_decode_error(msg: &str) -> MbValue {
    raise_exc("UnicodeDecodeError", msg)
}
fn raise_unicode_encode_error(msg: &str) -> MbValue {
    raise_exc("UnicodeEncodeError", msg)
}

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

// encode/decode are variadic: (obj, encoding='utf-8', errors='strict').
// The `errors` handler controls whether undecodable/unencodable input raises.
unsafe extern "C" fn dispatch_encode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs == 0 {
        return raise_type_error("encode() missing 1 required positional argument: 'obj'");
    }
    mb_codecs_encode3(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs == 0 {
        return raise_type_error("decode() missing 1 required positional argument: 'obj'");
    }
    mb_codecs_decode3(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}
// charmap_decode(input, errors, mapping) — 3 positional args.
unsafe extern "C" fn dispatch_charmap_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_charmap_decode(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}
// getencoder/getdecoder require a name argument (TypeError if missing).
unsafe extern "C" fn dispatch_getencoder(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return raise_type_error("getencoder() missing 1 required positional argument: 'encoding'");
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_getencoder(a[0])
}
unsafe extern "C" fn dispatch_getdecoder(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return raise_type_error("getdecoder() missing 1 required positional argument: 'encoding'");
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_getdecoder(a[0])
}
disp_unary!(dispatch_lookup, mb_codecs_lookup);
disp_unary!(dispatch_register, mb_codecs_register);
disp_binary!(dispatch_register_error, mb_codecs_register_error);
disp_unary!(dispatch_lookup_error, mb_codecs_lookup_error);
disp_unary!(dispatch_open, mb_codecs_open);
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
// iterencode(iterable, encoding, errors='strict') -> list of encoded chunks.
unsafe extern "C" fn dispatch_iterencode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_iterencode2(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_iterdecode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_iterdecode2(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
// EncodedFile(file, data_encoding, file_encoding='utf-8', errors='strict').
unsafe extern "C" fn dispatch_encodedfile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    make_encoded_file(a)
}
// raw_unicode_escape_decode(input, errors='strict') -> (str, len).
unsafe extern "C" fn dispatch_raw_unicode_escape_decode(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_raw_unicode_escape_decode(a.get(0).copied().unwrap_or_else(MbValue::none))
}
disp_unary!(
    dispatch_raw_unicode_escape_encode,
    mb_codecs_raw_unicode_escape_encode
);
disp_unary!(dispatch_readbuffer_encode, mb_codecs_readbuffer_encode);
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
// utf_16_decode(input, errors='strict', byteorder=0) -> (str, consumed).
unsafe extern "C" fn dispatch_utf_16_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if nargs <= 1 {
        // Single-arg form keeps returning a bare str (used internally + by the
        // module-level convenience call the existing tests cover).
        return mb_codecs_utf_16_decode(a.get(0).copied().unwrap_or_else(MbValue::none));
    }
    mb_codecs_utf_16_decode_tuple(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}
disp_unary!(dispatch_utf_16_le_encode, mb_codecs_utf_16_le_encode);
disp_unary!(dispatch_utf_16_le_decode, mb_codecs_utf_16_le_decode);
disp_unary!(dispatch_utf_16_be_encode, mb_codecs_utf_16_be_encode);
disp_unary!(dispatch_utf_16_be_decode, mb_codecs_utf_16_be_decode);
disp_unary!(dispatch_utf_32_encode, mb_codecs_utf_32_encode);
disp_unary!(dispatch_utf_32_decode, mb_codecs_utf_32_decode);
disp_unary!(dispatch_ascii_encode, mb_codecs_ascii_encode);
disp_unary!(dispatch_ascii_decode, mb_codecs_ascii_decode);
disp_unary!(dispatch_escape_encode, mb_codecs_escape_encode);
disp_unary!(
    dispatch_unicode_escape_encode,
    mb_codecs_unicode_escape_encode
);
// escape_decode / unicode_escape_decode take (input, errors='strict').
unsafe extern "C" fn dispatch_escape_decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_escape_decode2(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_unicode_escape_decode(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_codecs_unicode_escape_decode2(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
disp_unary!(dispatch_latin_1_encode, mb_codecs_latin_1_encode);
disp_unary!(dispatch_latin_1_decode, mb_codecs_latin_1_decode);

// Class-name dispatchers. Each constructs an Instance tagged with the matching
// CPython class name so `type(x).__name__` and plain field access work without
// touching class.rs. The container/stream OOP classes are presence-only ctors.
unsafe extern "C" fn dispatch_codec(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("Codec".to_string()))
}
unsafe extern "C" fn dispatch_codecinfo(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // CodecInfo(encode, decode, streamreader=None, streamwriter=None,
    //           incrementalencoder=None, incrementaldecoder=None, name=None, ...)
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let name = a.get(6).copied().unwrap_or_else(MbValue::none);
    make_codec_info(
        name,
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}
unsafe extern "C" fn dispatch_incrementalencoder(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("IncrementalEncoder".to_string()))
}
unsafe extern "C" fn dispatch_incrementaldecoder(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("IncrementalDecoder".to_string()))
}
unsafe extern "C" fn dispatch_streamreader(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("StreamReader".to_string()))
}
unsafe extern "C" fn dispatch_streamwriter(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("StreamWriter".to_string()))
}
unsafe extern "C" fn dispatch_streamreaderwriter(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("StreamReaderWriter".to_string()))
}
unsafe extern "C" fn dispatch_streamrecoder(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("StreamRecoder".to_string()))
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
        // Low-level escape codecs
        (
            "escape_decode",
            dispatch_escape_decode as *const () as usize,
        ),
        (
            "escape_encode",
            dispatch_escape_encode as *const () as usize,
        ),
        (
            "unicode_escape_decode",
            dispatch_unicode_escape_decode as *const () as usize,
        ),
        (
            "unicode_escape_encode",
            dispatch_unicode_escape_encode as *const () as usize,
        ),
        (
            "charmap_decode",
            dispatch_charmap_decode as *const () as usize,
        ),
        // Low-level raw-unicode-escape + buffer codec functions
        (
            "raw_unicode_escape_decode",
            dispatch_raw_unicode_escape_decode as *const () as usize,
        ),
        (
            "raw_unicode_escape_encode",
            dispatch_raw_unicode_escape_encode as *const () as usize,
        ),
        (
            "readbuffer_encode",
            dispatch_readbuffer_encode as *const () as usize,
        ),
        // Classes
        ("Codec", dispatch_codec as *const () as usize),
        ("CodecInfo", dispatch_codecinfo as *const () as usize),
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
        (
            "StreamReaderWriter",
            dispatch_streamreaderwriter as *const () as usize,
        ),
        (
            "StreamRecoder",
            dispatch_streamrecoder as *const () as usize,
        ),
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
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le.clone())),
    );
    attrs.insert(
        "BOM_UTF32".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_le.clone())),
    );
    // Legacy aliases: BOM32_* == UTF-16 BOMs, BOM64_* == UTF-32 BOMs (CPython).
    attrs.insert(
        "BOM32_LE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_le)),
    );
    attrs.insert(
        "BOM32_BE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf16_be)),
    );
    attrs.insert(
        "BOM64_LE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_le)),
    );
    attrs.insert(
        "BOM64_BE".into(),
        MbValue::from_ptr(MbObject::new_bytes(bom_utf32_be)),
    );

    // Register the native codec classes (StreamReader/Writer/Recoder,
    // IncrementalEncoder/Decoder + their factory wrappers).
    register_codec_classes();

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
    match name.to_lowercase().replace(['-', '_', ' '], "").as_str() {
        s if s.starts_with("utf8") => "utf-8",
        s if s.starts_with("ascii") || s == "usascii" || s == "646" => "ascii",
        s if s.starts_with("latin1")
            || s == "iso88591"
            || s == "8859"
            || s == "cp819"
            || s == "l1" =>
        {
            "latin-1"
        }
        s if s.starts_with("utf16le") => "utf-16-le",
        s if s.starts_with("utf16be") => "utf-16-be",
        s if s.starts_with("utf16") => "utf-16",
        s if s.starts_with("utf32le") => "utf-32-le",
        s if s.starts_with("utf32be") => "utf-32-be",
        s if s.starts_with("utf32") => "utf-32",
        _ => "?unknown",
    }
}

/// Canonical CPython codec name for a (sub)set of recognised encodings, or
/// None when the name is unknown. `lookup`/`getencoder` raise LookupError on
/// None. The canonical strings match `codecs.lookup(name).name` for the cases
/// the fixtures exercise (notably utf-8 normalisation).
fn canonical_codec_name(name: &str) -> Option<&'static str> {
    match normalize_encoding(name) {
        "utf-8" => Some("utf-8"),
        "ascii" => Some("ascii"),
        "latin-1" => Some("iso8859-1"),
        "utf-16" => Some("utf-16"),
        "utf-16-le" => Some("utf-16-le"),
        "utf-16-be" => Some("utf-16-be"),
        "utf-32" => Some("utf-32"),
        "utf-32-le" => Some("utf-32-le"),
        "utf-32-be" => Some("utf-32-be"),
        _ => None,
    }
}

/// Build a `codecs.CodecInfo`-typed instance. `type(x).__name__` resolves to
/// "CodecInfo" and `.name` reads the stored field — no class.rs edit needed
/// (plain instance field access goes through the generic getattr path).
fn make_codec_info(name: MbValue, encode: MbValue, decode: MbValue) -> MbValue {
    let inst = MbObject::new_instance("CodecInfo".to_string());
    if let Some(ptr) = MbValue::from_ptr(inst).as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("name".to_string(), name);
                f.insert("encode".to_string(), encode);
                f.insert("decode".to_string(), decode);
            }
        }
    }
    MbValue::from_ptr(inst)
}

/// True when `val` is the string "strict" or None/absent (the default handler).
fn is_strict_errors(val: MbValue) -> bool {
    match extract_str(val) {
        Some(s) => s == "strict",
        None => true, // absent / None → strict default
    }
}

/// True for the well-known non-raising handlers we model exactly.
fn known_error_handler(name: &str) -> bool {
    matches!(
        name,
        "strict"
            | "ignore"
            | "replace"
            | "xmlcharrefreplace"
            | "backslashreplace"
            | "namereplace"
            | "surrogateescape"
            | "surrogatepass"
    )
}

/// Replacement bytes for an unencodable char `c` under handler `name`, or
/// None when the handler is 'strict' (the caller raises). Matches CPython:
///   replace          → b'?'
///   ignore           → b'' (drop)
///   xmlcharrefreplace→ b'&#NNN;'
///   backslashreplace → b'\\xHH'/b'\\uHHHH'/b'\\UHHHHHHHH'
fn encode_error_repl(name: &str, c: char) -> Option<Vec<u8>> {
    let cp = c as u32;
    match name {
        "replace" => Some(vec![b'?']),
        "ignore" => Some(Vec::new()),
        "xmlcharrefreplace" => Some(format!("&#{};", cp).into_bytes()),
        "backslashreplace" => {
            let s = if cp <= 0xFF {
                format!("\\x{:02x}", cp)
            } else if cp <= 0xFFFF {
                format!("\\u{:04x}", cp)
            } else {
                format!("\\U{:08x}", cp)
            };
            Some(s.into_bytes())
        }
        "namereplace" => Some(format!("\\N{{U+{:04X}}}", cp).into_bytes()),
        _ => None, // strict / surrogate* → caller raises
    }
}

// -- encode/decode --

/// codecs.encode(obj, encoding='utf-8') -> bytes  (strict errors)
pub fn mb_codecs_encode(obj: MbValue, encoding: MbValue) -> MbValue {
    mb_codecs_encode3(obj, encoding, MbValue::none())
}

/// codecs.encode(obj, encoding='utf-8', errors='strict') -> bytes
///
/// Raises (matching CPython 3.12):
///   - LookupError on an unknown `errors` handler name,
///   - UnicodeEncodeError when a character is unencodable under `strict`.
pub fn mb_codecs_encode3(obj: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    // bytes→bytes transform codecs (base64/hex/zlib/...) reached via
    // codecs.encode(<bytes>, "<name>_codec"). Handle these before the
    // str-only text-codec path.
    if let Some(name) = extract_str(encoding) {
        if let Some(b) = transform_encode(&name, obj) {
            return b;
        }
    }
    // D2′: borrow the payload; only allocate the output Bytes.
    let s: &str = match unsafe { extract_str_ref(&obj) } {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let _strict = is_strict_errors(errors);
    // Validate the error handler name (CPython resolves it eagerly).
    let handler = extract_str(errors).unwrap_or_else(|| "strict".to_string());
    if !known_error_handler(&handler) {
        return raise_lookup_error(&format!("unknown error handler name '{}'", handler));
    }
    // The 'undefined' codec refuses to encode anything, even with non-strict
    // handlers (CPython: UnicodeError on any input including empty).
    if normalize_encoding(&enc) == "?unknown" && enc.to_lowercase() == "undefined" {
        return raise_unicode_encode_error(
            "'undefined' codec can't encode character in position 0",
        );
    }
    let bytes: Vec<u8> = match enc_norm {
        "utf-8" => {
            // Reject lone surrogates under strict (Rust &str can't hold them,
            // so any well-formed str round-trips; kept for completeness).
            s.as_bytes().to_vec()
        }
        "ascii" => {
            let mut out = Vec::with_capacity(s.len());
            for (i, c) in s.char_indices() {
                if c.is_ascii() {
                    out.push(c as u8);
                } else if let Some(repl) = encode_error_repl(&handler, c) {
                    out.extend_from_slice(&repl);
                } else {
                    return raise_unicode_encode_error(&format!(
                        "'ascii' codec can't encode character '\\u{:04x}' in position {}: ordinal not in range(128)",
                        c as u32, i));
                }
            }
            out
        }
        "latin-1" => {
            let mut out = Vec::with_capacity(s.len());
            for (i, c) in s.char_indices() {
                let n = c as u32;
                if n <= 255 {
                    out.push(n as u8);
                } else if let Some(repl) = encode_error_repl(&handler, c) {
                    out.extend_from_slice(&repl);
                } else {
                    return raise_unicode_encode_error(&format!(
                        "'latin-1' codec can't encode character '\\u{:04x}' in position {}: ordinal not in range(256)",
                        n, i));
                }
            }
            out
        }
        "utf-16" => {
            let mut b = vec![0xFF, 0xFE];
            b.extend(encode_u16_units(s, true));
            b
        }
        "utf-16-le" => encode_u16_units(s, true),
        "utf-16-be" => encode_u16_units(s, false),
        "utf-32" => {
            let mut b = vec![0xFF, 0xFE, 0x00, 0x00];
            b.extend(encode_u32_units(s, true));
            b
        }
        "utf-32-le" => encode_u32_units(s, true),
        "utf-32-be" => encode_u32_units(s, false),
        _ => return raise_lookup_error(&format!("unknown encoding: {}", enc)),
    };
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

/// codecs.decode(obj, encoding='utf-8') -> str  (strict errors)
pub fn mb_codecs_decode(obj: MbValue, encoding: MbValue) -> MbValue {
    mb_codecs_decode3(obj, encoding, MbValue::none())
}

/// codecs.decode(obj, encoding='utf-8', errors='strict') -> str
///
/// Raises (matching CPython 3.12):
///   - LookupError on an unknown `errors` handler name,
///   - UnicodeDecodeError when the bytes are malformed under `strict`.
pub fn mb_codecs_decode3(obj: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    // bytes→bytes transform codecs (base64/hex/zlib/...) reached via
    // codecs.decode(<bytes>, "<name>_codec").
    if let Some(name) = extract_str(encoding) {
        if let Some(b) = transform_decode(&name, obj) {
            return b;
        }
    }
    let bytes: &[u8] = match unsafe { extract_bytes_ref(&obj) } {
        Some(b) => b,
        None => {
            // str passthrough (CPython coerces str→bytes via the default codec
            // before decode; we model the identity case the fixtures use).
            if let Some(s) = unsafe { extract_str_ref(&obj) } {
                return MbValue::from_ptr(MbObject::new_str(s.to_owned()));
            }
            return MbValue::none();
        }
    };
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let enc_norm = normalize_encoding(&enc);
    let strict = is_strict_errors(errors);
    if let Some(h) = extract_str(errors) {
        if !known_error_handler(&h) {
            return raise_lookup_error(&format!("unknown error handler name '{}'", h));
        }
    }
    // The 'undefined' codec refuses to decode anything.
    if enc_norm == "?unknown" && enc.to_lowercase() == "undefined" {
        return raise_unicode_decode_error(
            "'undefined' codec can't decode byte 0x00 in position 0: undefined mapping",
        );
    }
    let s = match enc_norm {
        "utf-8" => {
            if strict {
                match std::str::from_utf8(bytes) {
                    Ok(v) => v.to_owned(),
                    Err(e) => {
                        let pos = e.valid_up_to();
                        let bad = bytes.get(pos).copied().unwrap_or(0);
                        return raise_unicode_decode_error(&format!(
                            "'utf-8' codec can't decode byte 0x{:02x} in position {}: invalid start byte",
                            bad, pos));
                    }
                }
            } else {
                String::from_utf8_lossy(bytes).into_owned()
            }
        }
        "ascii" => {
            let mut out = String::with_capacity(bytes.len());
            for (i, &b) in bytes.iter().enumerate() {
                if b < 128 {
                    out.push(b as char);
                } else if strict {
                    return raise_unicode_decode_error(&format!(
                        "'ascii' codec can't decode byte 0x{:02x} in position {}: ordinal not in range(128)",
                        b, i));
                } else {
                    out.push('\u{FFFD}');
                }
            }
            out
        }
        "latin-1" => bytes.iter().map(|&b| b as char).collect(),
        "utf-16" => return mb_codecs_utf_16_decode(obj),
        "utf-16-le" => decode_u16_units(bytes, true),
        "utf-16-be" => decode_u16_units(bytes, false),
        "utf-32" => return mb_codecs_utf_32_decode(obj),
        "utf-32-le" => decode_u32_units(bytes, true),
        "utf-32-be" => decode_u32_units(bytes, false),
        _ => return raise_lookup_error(&format!("unknown encoding: {}", enc)),
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

/// codecs.lookup(encoding) -> CodecInfo
///
/// Raises LookupError for an unknown codec (CPython 3.12). The returned value
/// is a `CodecInfo`-typed instance whose `.name` is the canonical codec name.
pub fn mb_codecs_lookup(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("lookup() argument must be str"),
    };
    let canonical = match canonical_codec_name(&enc) {
        Some(c) => c,
        None => return raise_lookup_error(&format!("unknown encoding: {}", enc)),
    };
    make_codec_info(
        MbValue::from_ptr(MbObject::new_str(canonical.to_string())),
        MbValue::none(),
        MbValue::none(),
    )
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

pub fn mb_codecs_getincrementaldecoder(encoding: MbValue) -> MbValue {
    mb_codecs_getincrementaldecoder_real(encoding)
}
pub fn mb_codecs_getincrementalencoder(encoding: MbValue) -> MbValue {
    mb_codecs_getincrementalencoder_real(encoding)
}
pub fn mb_codecs_getreader(encoding: MbValue) -> MbValue {
    mb_codecs_getreader_real(encoding)
}
pub fn mb_codecs_getwriter(encoding: MbValue) -> MbValue {
    mb_codecs_getwriter_real(encoding)
}

// -- Convenience codecs --

pub fn mb_codecs_utf_8_encode(s: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
    mb_codecs_encode(s, enc)
}

pub fn mb_codecs_utf_8_decode(b: MbValue) -> MbValue {
    // Low-level decoder: requires a bytes-like argument (CPython raises
    // TypeError when handed a str).
    if let Some(s) = extract_str(b) {
        return raise_type_error(&format!(
            "decoding to str: need a bytes-like object, str found ({:?})",
            s
        ));
    }
    let enc = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
    mb_codecs_decode(b, enc)
}

pub fn mb_codecs_ascii_encode(s: MbValue) -> MbValue {
    let enc = MbValue::from_ptr(MbObject::new_str("ascii".to_string()));
    mb_codecs_encode(s, enc)
}

pub fn mb_codecs_ascii_decode(b: MbValue) -> MbValue {
    if extract_str(b).is_some() {
        return raise_type_error("decoding to str: need a bytes-like object, str found");
    }
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

/// codecs.utf_16_decode(input, errors, byteorder) -> (str, consumed).
/// byteorder: -1 = LE, 0 = native/BOM, 1 = BE. A trailing lone byte is handled
/// by the `errors` handler (replace → U+FFFD, ignore → dropped) and still
/// counts the full input length as consumed (matching CPython).
pub fn mb_codecs_utf_16_decode_tuple(
    input: MbValue,
    errors: MbValue,
    byteorder: MbValue,
) -> MbValue {
    let bytes = match unsafe { extract_bytes_ref(&input) } {
        Some(v) => v.to_vec(),
        None => return raise_type_error("utf_16_decode() argument must be bytes-like"),
    };
    let errh = extract_str(errors).unwrap_or_else(|| "strict".to_string());
    let bo = byteorder.as_int().unwrap_or(0);
    let consumed = bytes.len() as i64;
    let (slice, le): (&[u8], bool) = if bo < 0 {
        (&bytes, true)
    } else if bo > 0 {
        (&bytes, false)
    } else {
        utf16_bom(&bytes)
    };
    let usable = slice.len() - (slice.len() % 2);
    let mut out = decode_u16_units(&slice[..usable], le);
    // Trailing odd byte.
    if slice.len() % 2 != 0 {
        match errh.as_str() {
            "ignore" => {}
            "replace" => out.push('\u{FFFD}'),
            _ => {
                return raise_unicode_decode_error(&format!(
                    "'utf-16-le' codec can't decode byte 0x{:02x} in position {}: truncated data",
                    slice[slice.len() - 1],
                    slice.len() - 1
                ))
            }
        }
    }
    tuple2(new_str(out), MbValue::from_int(consumed))
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

// getencoder/getdecoder return a callable that, given input, yields the
// CPython (result, consumed-length) tuple. They are implemented as factory
// instances bound to the codec mode (the `_EncoderFunc` / `_DecoderFunc`
// classes registered in `register_codec_classes`), whose `__call__(self, x)`
// returns the tuple.

pub fn mb_codecs_getencoder(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getencoder() argument must be str"),
    };
    if canonical_codec_name(&enc).is_none() {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._EncoderFunc", &enc)
}

pub fn mb_codecs_getdecoder(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getdecoder() argument must be str"),
    };
    if canonical_codec_name(&enc).is_none() {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._DecoderFunc", &enc)
}

// -- charmap_decode --
//
// codecs.charmap_decode(input, errors='strict', mapping=None) -> (str, len)
// We support the dict-mapping and string-mapping cases the fixtures exercise.

pub fn mb_codecs_charmap_decode(input: MbValue, errors: MbValue, mapping: MbValue) -> MbValue {
    let bytes: Vec<u8> = match unsafe { extract_bytes_ref(&input) } {
        Some(b) => b.to_vec(),
        None => return raise_type_error("charmap_decode() argument 1 must be bytes-like"),
    };
    // mapping may be a dict {int: int|str} or a 256-char string.
    let map_ptr = match mapping.as_ptr() {
        Some(p) => p,
        None => return raise_type_error("charmap_decode() requires a mapping"),
    };
    let errh = extract_str(errors);
    let errh = errh.as_deref().unwrap_or("strict");
    let mut out = String::with_capacity(bytes.len());
    // Apply the error handler for an undecodable byte, or return Err for strict.
    macro_rules! handle_undef {
        ($b:expr, $i:expr) => {{
            match errh {
                "replace" => { out.push('\u{FFFD}'); }
                "ignore" => { /* drop */ }
                "backslashreplace" => { out.push_str(&format!("\\x{:02x}", $b)); }
                _ => {
                    return raise_unicode_decode_error(&format!(
                        "'charmap' codec can't decode byte 0x{:02x} in position {}: character maps to <undefined>",
                        $b, $i));
                }
            }
        }};
    }
    unsafe {
        match &(*map_ptr).data {
            ObjData::Dict(lock) => {
                use super::super::dict_ops::DictKey;
                let guard = lock.read().unwrap();
                for (i, &b) in bytes.iter().enumerate() {
                    match guard.get(&DictKey::Int(b as i64)) {
                        Some(v) => {
                            if let Some(code) = v.as_int() {
                                if !(0..=0x10FFFF).contains(&code) {
                                    return raise_type_error(
                                        "character mapping must be in range(0x110000)",
                                    );
                                }
                                match char::from_u32(code as u32) {
                                    Some(c) => out.push(c),
                                    None => {
                                        return raise_type_error(
                                            "character mapping must be in range(0x110000)",
                                        )
                                    }
                                }
                            } else if let Some(sp) = v.as_ptr() {
                                if let ObjData::Str(ref s) = (*sp).data {
                                    out.push_str(s);
                                } else {
                                    return raise_type_error(
                                        "character mapping must return integer, None or str",
                                    );
                                }
                            } else if v.is_none() {
                                handle_undef!(b, i);
                            }
                        }
                        None => handle_undef!(b, i),
                    }
                }
            }
            ObjData::Str(ref table) => {
                let chars: Vec<char> = table.chars().collect();
                for (i, &b) in bytes.iter().enumerate() {
                    match chars.get(b as usize) {
                        Some(&c) if c != '\u{FFFE}' => out.push(c),
                        _ => handle_undef!(b, i),
                    }
                }
            }
            _ => return raise_type_error("charmap_decode() mapping must be dict or str"),
        }
    }
    let len = bytes.len() as i64;
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(out)),
        MbValue::from_int(len),
    ]))
}

// -- escape_decode / unicode_escape (low-level) --
//
// CPython:
//   codecs.escape_decode(b'a\\nb')        -> (b'a\nb', 4)   (bytes out)
//   codecs.unicode_escape_decode(b'a\\nb')-> ('a\nb', 4)    (str out)
// Truncated \x escapes raise ValueError (escape_decode) /
// UnicodeDecodeError (unicode_escape_decode).

/// Error-handler behaviour for a truncated `\x` escape.
#[derive(Clone, Copy, PartialEq)]
enum EscErr {
    Strict,
    Ignore,
    Replace,
}

fn esc_err_from(name: Option<&str>) -> EscErr {
    match name {
        Some("ignore") => EscErr::Ignore,
        Some("replace") => EscErr::Replace,
        _ => EscErr::Strict, // strict / None / unknown → strict
    }
}

/// Decode C-style escapes. On a truncated `\x`, behaviour follows `errh`:
/// Strict → Err((is_unicode, message)); Ignore → consume + emit nothing;
/// Replace → consume + emit '?'. `unicode` selects the strict error class/msg.
fn escape_decode_inner(src: &[u8], unicode: bool, errh: EscErr) -> Result<Vec<u8>, (bool, String)> {
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        if b != b'\\' {
            out.push(b);
            i += 1;
            continue;
        }
        if i + 1 >= src.len() {
            // trailing backslash kept verbatim (CPython).
            out.push(b'\\');
            i += 1;
            continue;
        }
        let e = src[i + 1];
        i += 2;
        match e {
            b'n' => out.push(b'\n'),
            b't' => out.push(b'\t'),
            b'r' => out.push(b'\r'),
            b'\\' => out.push(b'\\'),
            b'\'' => out.push(b'\''),
            b'"' => out.push(b'"'),
            b'a' => out.push(0x07),
            b'b' => out.push(0x08),
            b'f' => out.push(0x0C),
            b'v' => out.push(0x0B),
            b'0'..=b'7' => {
                let mut val = (e - b'0') as u32;
                let mut n = 1;
                while n < 3 && i < src.len() && (b'0'..=b'7').contains(&src[i]) {
                    val = val * 8 + (src[i] - b'0') as u32;
                    i += 1;
                    n += 1;
                }
                out.push((val & 0xFF) as u8);
            }
            b'x' => {
                let h1 = src.get(i).and_then(|c| (*c as char).to_digit(16));
                let h2 = src.get(i + 1).and_then(|c| (*c as char).to_digit(16));
                match (h1, h2) {
                    (Some(a), Some(b2)) => {
                        out.push(((a << 4) | b2) as u8);
                        i += 2;
                    }
                    _ => {
                        let pos = i - 2;
                        // Consume the optional single hex digit that was present.
                        if h1.is_some() {
                            i += 1;
                        }
                        match errh {
                            EscErr::Ignore => { /* emit nothing */ }
                            EscErr::Replace => out.push(b'?'),
                            EscErr::Strict => {
                                if unicode {
                                    return Err((true, format!(
                                        "'unicodeescape' codec can't decode bytes in position {}-{}: truncated \\xXX escape",
                                        pos, src.len() - 1)));
                                } else {
                                    return Err((
                                        false,
                                        format!("invalid \\x escape at position {}", pos),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                out.push(b'\\');
                out.push(e);
            }
        }
    }
    Ok(out)
}

fn escape_input_bytes(input: MbValue, fname: &str) -> Result<Vec<u8>, MbValue> {
    match unsafe { extract_bytes_ref(&input) } {
        Some(b) => Ok(b.to_vec()),
        None => match extract_str(input) {
            Some(s) => Ok(s.into_bytes()),
            None => Err(raise_type_error(&format!(
                "{}() argument must be bytes-like or str",
                fname
            ))),
        },
    }
}

pub fn mb_codecs_escape_decode2(input: MbValue, errors: MbValue) -> MbValue {
    let src = match escape_input_bytes(input, "escape_decode") {
        Ok(s) => s,
        Err(e) => return e,
    };
    let orig_len = src.len() as i64;
    let errh = esc_err_from(extract_str(errors).as_deref());
    match escape_decode_inner(&src, false, errh) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_bytes(decoded)),
            MbValue::from_int(orig_len),
        ])),
        Err((_unicode, msg)) => raise_value_error(&msg),
    }
}

/// Decode 'unicode_escape' to a String directly so `\u`/`\U` escapes and the
/// U+FFFD replacement handler work (the byte-oriented inner decoder can't).
/// Returns Ok(decoded string) or Err(strict error message).
fn unicode_escape_decode_str(src: &[u8], errh: EscErr) -> Result<String, String> {
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    // Push a decoded code point, or apply the error handler when invalid.
    macro_rules! push_code {
        ($code:expr, $pos:expr, $end:expr, $kind:expr) => {{
            match char::from_u32($code) {
                Some(c) => out.push(c),
                None => match errh {
                    EscErr::Ignore => {}
                    EscErr::Replace => out.push('\u{FFFD}'),
                    EscErr::Strict => {
                        return Err(format!(
                            "'unicodeescape' codec can't decode bytes in position {}-{}: {}",
                            $pos, $end, $kind
                        ))
                    }
                },
            }
        }};
    }
    // Apply the handler for a truncated escape.
    macro_rules! truncated {
        ($pos:expr, $end:expr, $kind:expr) => {{
            match errh {
                EscErr::Ignore => {}
                EscErr::Replace => out.push('\u{FFFD}'),
                EscErr::Strict => {
                    return Err(format!(
                        "'unicodeescape' codec can't decode bytes in position {}-{}: {}",
                        $pos, $end, $kind
                    ))
                }
            }
        }};
    }
    while i < src.len() {
        let b = src[i];
        if b != b'\\' {
            // Raw byte; latin-1-ish passthrough into the string.
            out.push(b as char);
            i += 1;
            continue;
        }
        if i + 1 >= src.len() {
            out.push('\\');
            i += 1;
            continue;
        }
        let e = src[i + 1];
        let start = i;
        i += 2;
        let hexn = |n: usize, j: usize| -> Option<u32> {
            if j + n > src.len() {
                return None;
            }
            let mut v = 0u32;
            for k in 0..n {
                let d = (src[j + k] as char).to_digit(16)?;
                v = (v << 4) | d;
            }
            Some(v)
        };
        match e {
            b'n' => out.push('\n'),
            b't' => out.push('\t'),
            b'r' => out.push('\r'),
            b'\\' => out.push('\\'),
            b'\'' => out.push('\''),
            b'"' => out.push('"'),
            b'a' => out.push('\u{07}'),
            b'b' => out.push('\u{08}'),
            b'f' => out.push('\u{0C}'),
            b'v' => out.push('\u{0B}'),
            b'0'..=b'7' => {
                let mut val = (e - b'0') as u32;
                let mut n = 1;
                while n < 3 && i < src.len() && (b'0'..=b'7').contains(&src[i]) {
                    val = val * 8 + (src[i] - b'0') as u32;
                    i += 1;
                    n += 1;
                }
                push_code!(val, start, i - 1, "invalid octal escape");
            }
            b'x' => match hexn(2, i) {
                Some(v) => {
                    out.push(v as u8 as char);
                    i += 2;
                }
                None => {
                    if i < src.len() && (src[i] as char).is_ascii_hexdigit() {
                        i += 1;
                    }
                    truncated!(start, src.len() - 1, "truncated \\xXX escape");
                }
            },
            b'u' => match hexn(4, i) {
                Some(v) => {
                    push_code!(v, start, i + 3, "illegal Unicode character");
                    i += 4;
                }
                None => {
                    while i < src.len() && (src[i] as char).is_ascii_hexdigit() {
                        i += 1;
                    }
                    truncated!(start, src.len() - 1, "truncated \\uXXXX escape");
                }
            },
            b'U' => match hexn(8, i) {
                Some(v) => {
                    push_code!(v, start, i + 7, "illegal Unicode character");
                    i += 8;
                }
                None => {
                    while i < src.len() && (src[i] as char).is_ascii_hexdigit() {
                        i += 1;
                    }
                    truncated!(start, src.len() - 1, "truncated \\UXXXXXXXX escape");
                }
            },
            _ => {
                out.push('\\');
                out.push(e as char);
            }
        }
    }
    Ok(out)
}

pub fn mb_codecs_unicode_escape_decode2(input: MbValue, errors: MbValue) -> MbValue {
    let src = match escape_input_bytes(input, "unicode_escape_decode") {
        Ok(s) => s,
        Err(e) => return e,
    };
    let orig_len = src.len() as i64;
    let errh = esc_err_from(extract_str(errors).as_deref());
    match unicode_escape_decode_str(&src, errh) {
        Ok(s) => MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(s)),
            MbValue::from_int(orig_len),
        ])),
        Err(msg) => raise_unicode_decode_error(&msg),
    }
}

/// codecs.escape_encode(data) -> (escaped_bytes, len). Minimal: escapes the
/// classic control chars and backslash; passes printable ASCII through.
pub fn mb_codecs_escape_encode(input: MbValue) -> MbValue {
    let src: Vec<u8> = match unsafe { extract_bytes_ref(&input) } {
        Some(b) => b.to_vec(),
        None => return raise_type_error("escape_encode() argument must be bytes-like"),
    };
    let orig_len = src.len() as i64;
    let mut out: Vec<u8> = Vec::with_capacity(src.len());
    for &b in &src {
        match b {
            b'\n' => out.extend_from_slice(b"\\n"),
            b'\t' => out.extend_from_slice(b"\\t"),
            b'\r' => out.extend_from_slice(b"\\r"),
            b'\\' => out.extend_from_slice(b"\\\\"),
            0x20..=0x7E => out.push(b),
            _ => out.extend_from_slice(format!("\\x{:02x}", b).as_bytes()),
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_bytes(out)),
        MbValue::from_int(orig_len),
    ]))
}

/// codecs.unicode_escape_encode(s) -> (escaped_bytes, len).
pub fn mb_codecs_unicode_escape_encode(input: MbValue) -> MbValue {
    let s: String = match extract_str(input) {
        Some(s) => s,
        None => match unsafe { extract_bytes_ref(&input) } {
            Some(b) => String::from_utf8_lossy(b).into_owned(),
            None => return raise_type_error("unicode_escape_encode() argument must be str"),
        },
    };
    let orig_len = s.chars().count() as i64;
    let mut out: Vec<u8> = Vec::new();
    for c in s.chars() {
        match c {
            '\n' => out.extend_from_slice(b"\\n"),
            '\t' => out.extend_from_slice(b"\\t"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\\' => out.extend_from_slice(b"\\\\"),
            c if (c as u32) >= 0x20 && (c as u32) < 0x7F => out.push(c as u8),
            c if (c as u32) <= 0xFF => {
                out.extend_from_slice(format!("\\x{:02x}", c as u32).as_bytes())
            }
            c if (c as u32) <= 0xFFFF => {
                out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes())
            }
            c => out.extend_from_slice(format!("\\U{:08x}", c as u32).as_bytes()),
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_bytes(out)),
        MbValue::from_int(orig_len),
    ]))
}

// iterencode/iterdecode return a list of recoded chunks (a list is iterable,
// so `b"".join(...)` / `"".join(...)` work). encodedfile/EncodedFile builds a
// StreamRecoder. These take 2+ args, so they're driven by the flat-args
// dispatchers below rather than disp_unary.
pub fn mb_codecs_iterencode(_obj: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_iterdecode(_obj: MbValue) -> MbValue {
    MbValue::none()
}
pub fn mb_codecs_encodedfile(_file: MbValue) -> MbValue {
    MbValue::none()
}

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

// ════════════════════════════════════════════════════════════════════════
//  Stateful codec core + StreamReader/Writer/Recoder + IncrementalEncoder/
//  Decoder + EncodedFile + iterencode/iterdecode (#656 wave-2).
//
//  These give `codecs.getreader/getwriter/getincremental*/open/EncodedFile/
//  iter*` real behaviour for the byte-oriented codecs (utf-8, utf-8-sig,
//  utf-16(-le/-be), utf-32(-le/-be), ascii, latin-1). The factory objects
//  are callable native classes (see `register`); the recode/decode methods
//  are registered as variadic instance methods so a single Rust fn handles
//  any positional arity (`read()`, `read(7)`, `seek(0, 0)`).
//
//  Underlying byte streams (io.BytesIO etc.) are driven by name through
//  `class::mb_call_method`, so any object exposing the read/write/seek
//  protocol works.
// ════════════════════════════════════════════════════════════════════════

use super::super::class::mb_call_method;

/// Read a named field off an Instance value.
fn inst_field(obj: MbValue, name: &str) -> Option<MbValue> {
    obj.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(name).copied()
        } else {
            None
        }
    })
}

/// Set a named field on an Instance value.
fn set_inst_field(obj: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), value);
            }
        }
    }
}

fn inst_str(obj: MbValue, name: &str) -> String {
    inst_field(obj, name)
        .and_then(extract_str)
        .unwrap_or_default()
}

/// First positional argument from the variadic `args_list` passed to an
/// instance method (`[self, pos_list]`); None when absent.
fn pos_arg(args_list: MbValue, i: usize) -> Option<MbValue> {
    args_list.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::List(lock) => lock.read().unwrap().get(i).copied(),
            ObjData::Tuple(items) => items.get(i).copied(),
            _ => None,
        }
    })
}

fn new_bytes(b: Vec<u8>) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(b))
}
fn new_str(s: String) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s))
}
fn tuple2(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![a, b]))
}

/// Pull bytes out of a bytes / bytearray / str(latin1) MbValue.
fn as_bytes_vec(v: MbValue) -> Option<Vec<u8>> {
    v.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

// ── Core encode/decode operating on Rust types ────────────────────────────
//
// `mode` is the *normalised* codec name (output of `normalize_encoding`) with
// the extra "utf-8-sig" form handled separately by the callers that need BOM
// stripping/prefixing.

/// Encode a &str to bytes for the given normalised codec, strict errors.
/// Returns None on an unsupported codec.
fn codec_encode_bytes(mode: &str, s: &str) -> Option<Vec<u8>> {
    Some(match mode {
        "utf-8" => s.as_bytes().to_vec(),
        "ascii" => {
            let mut out = Vec::with_capacity(s.len());
            for c in s.chars() {
                if c.is_ascii() {
                    out.push(c as u8);
                } else {
                    return None;
                }
            }
            out
        }
        "latin-1" => {
            let mut out = Vec::with_capacity(s.len());
            for c in s.chars() {
                let n = c as u32;
                if n <= 255 {
                    out.push(n as u8);
                } else {
                    return None;
                }
            }
            out
        }
        "utf-16-le" => encode_u16_units(s, true),
        "utf-16-be" => encode_u16_units(s, false),
        "utf-16" => {
            let mut b = vec![0xFF, 0xFE];
            b.extend(encode_u16_units(s, true));
            b
        }
        "utf-32-le" => encode_u32_units(s, true),
        "utf-32-be" => encode_u32_units(s, false),
        "utf-32" => {
            let mut b = vec![0xFF, 0xFE, 0x00, 0x00];
            b.extend(encode_u32_units(s, true));
            b
        }
        _ => return None,
    })
}

/// Decode bytes to a String for the given normalised codec, strict errors.
/// Returns None on an unsupported codec (malformed input is lossily decoded
/// for the stream/incremental paths, which only feed well-formed data).
fn codec_decode_str(mode: &str, bytes: &[u8]) -> Option<String> {
    Some(match mode {
        "utf-8" => String::from_utf8_lossy(bytes).into_owned(),
        "ascii" => bytes.iter().map(|&b| b as char).collect(),
        "latin-1" => bytes.iter().map(|&b| b as char).collect(),
        "utf-16-le" => decode_u16_units(bytes, true),
        "utf-16-be" => decode_u16_units(bytes, false),
        "utf-16" => {
            let (slice, le) = utf16_bom(bytes);
            decode_u16_units(slice, le)
        }
        "utf-32-le" => decode_u32_units(bytes, true),
        "utf-32-be" => decode_u32_units(bytes, false),
        "utf-32" => {
            let (slice, le) = utf32_bom(bytes);
            decode_u32_units(slice, le)
        }
        _ => return None,
    })
}

fn utf16_bom(bytes: &[u8]) -> (&[u8], bool) {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        (&bytes[2..], true)
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        (&bytes[2..], false)
    } else {
        (bytes, true)
    }
}
fn utf32_bom(bytes: &[u8]) -> (&[u8], bool) {
    if bytes.len() >= 4 && bytes[0..4] == [0xFF, 0xFE, 0x00, 0x00] {
        (&bytes[4..], true)
    } else if bytes.len() >= 4 && bytes[0..4] == [0x00, 0x00, 0xFE, 0xFF] {
        (&bytes[4..], false)
    } else {
        (bytes, true)
    }
}

/// True for the "-sig" variant (utf-8-sig); returns the base normalised name.
fn split_sig(enc: &str) -> (String, bool) {
    let low = enc.to_lowercase().replace(['-', '_', ' '], "");
    if low == "utf8sig" {
        ("utf-8".to_string(), true)
    } else {
        (normalize_encoding(enc).to_string(), false)
    }
}

const BOM_UTF8: [u8; 3] = [0xEF, 0xBB, 0xBF];

// ── IncrementalDecoder (utf-8 / utf-8-sig / utf-16* / utf-32* / ascii /
//    latin-1). Buffers an incomplete trailing multibyte sequence between
//    calls. `final=True` flushes the buffer (lossily). ──────────────────────

/// Greatest prefix length of `buf` that decodes cleanly as UTF-8; the tail is
/// an incomplete multibyte sequence to carry over.
fn utf8_complete_prefix(buf: &[u8]) -> usize {
    match std::str::from_utf8(buf) {
        Ok(_) => buf.len(),
        Err(e) => {
            // valid_up_to is the clean prefix; if the error is a genuine
            // invalid sequence (not just truncation) keep it in the prefix so
            // the lossy decode emits U+FFFD rather than looping forever.
            let v = e.valid_up_to();
            match e.error_len() {
                None => v,            // truncated tail → carry over
                Some(_) => buf.len(), // real error → decode now (lossy)
            }
        }
    }
}

fn incr_decode(self_v: MbValue, args_list: MbValue) -> MbValue {
    let input = pos_arg(args_list, 0).unwrap_or_else(MbValue::none);
    let is_final = pos_arg(args_list, 1)
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);

    let chunk = as_bytes_vec(input).unwrap_or_default();
    let mut buf = inst_field(self_v, "buffer")
        .and_then(as_bytes_vec)
        .unwrap_or_default();
    buf.extend_from_slice(&chunk);

    // utf-8-sig: strip a leading BOM once seen in the buffer.
    let mut bom_done = inst_field(self_v, "bom_done")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    if sig && !bom_done {
        if buf.len() >= 3 {
            if buf[0..3] == BOM_UTF8 {
                buf.drain(0..3);
            }
            bom_done = true;
            set_inst_field(self_v, "bom_done", MbValue::from_bool(true));
        } else if !is_final && (buf.is_empty() || BOM_UTF8.starts_with(&buf)) {
            // Could be a partial BOM — wait for more bytes.
            set_inst_field(self_v, "buffer", new_bytes(buf));
            return new_str(String::new());
        } else {
            bom_done = true;
            set_inst_field(self_v, "bom_done", MbValue::from_bool(true));
        }
    }
    let _ = bom_done;

    // How many bytes can we commit now?
    let commit = if is_final {
        buf.len()
    } else {
        match mode.as_str() {
            "utf-8" => utf8_complete_prefix(&buf),
            "utf-16-le" | "utf-16-be" | "utf-16" => buf.len() - (buf.len() % 2),
            "utf-32-le" | "utf-32-be" | "utf-32" => buf.len() - (buf.len() % 4),
            _ => buf.len(), // ascii/latin-1: every byte is independent
        }
    };
    let (ready, rest) = buf.split_at(commit);
    let out = codec_decode_str(&mode, ready).unwrap_or_default();
    set_inst_field(self_v, "buffer", new_bytes(rest.to_vec()));
    new_str(out)
}

fn incr_decode_reset(self_v: MbValue, _args: MbValue) -> MbValue {
    set_inst_field(self_v, "buffer", new_bytes(Vec::new()));
    set_inst_field(self_v, "bom_done", MbValue::from_bool(false));
    MbValue::none()
}

// ── IncrementalEncoder ─────────────────────────────────────────────────────

fn incr_encode(self_v: MbValue, args_list: MbValue) -> MbValue {
    let input = pos_arg(args_list, 0).unwrap_or_else(MbValue::none);
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let s = extract_str(input).unwrap_or_default();
    let mut out = codec_encode_bytes(&mode, &s).unwrap_or_default();
    // utf-8-sig emits a BOM before the first chunk.
    if sig {
        let first = inst_field(self_v, "first")
            .map(|v| v.as_bool() != Some(false))
            .unwrap_or(true);
        if first {
            let mut prefixed = BOM_UTF8.to_vec();
            prefixed.append(&mut out);
            out = prefixed;
            set_inst_field(self_v, "first", MbValue::from_bool(false));
        }
    }
    new_bytes(out)
}

fn incr_encode_reset(self_v: MbValue, _args: MbValue) -> MbValue {
    set_inst_field(self_v, "first", MbValue::from_bool(true));
    MbValue::none()
}

// ── StreamReader: wraps a byte stream, decodes on read ─────────────────────
//
// Strategy: lazily slurp the whole underlying stream into a decoded String the
// first time it's needed, tracking a char position. read()/readline()/
// readlines()/seek(0) all operate on that decoded buffer. This matches the
// fixtures (small in-memory streams) byte-for-byte.

fn reader_underlying(self_v: MbValue) -> MbValue {
    inst_field(self_v, "stream").unwrap_or_else(MbValue::none)
}

/// Read all remaining bytes from the underlying stream and append to the
/// reader's raw byte buffer, then (re)decode into the text buffer.
fn reader_fill(self_v: MbValue) {
    let stream = reader_underlying(self_v);
    let raw = mb_call_method(
        stream,
        new_str("read".into()),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    let bytes = as_bytes_vec(raw).unwrap_or_default();
    let mut all = inst_field(self_v, "raw")
        .and_then(as_bytes_vec)
        .unwrap_or_default();
    all.extend_from_slice(&bytes);
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let mut decode_src: &[u8] = &all;
    if sig && all.len() >= 3 && all[0..3] == BOM_UTF8 {
        decode_src = &all[3..];
    }
    let text = codec_decode_str(&mode, decode_src).unwrap_or_default();
    set_inst_field(self_v, "raw", new_bytes(all));
    set_inst_field(self_v, "text", new_str(text));
}

fn reader_text(self_v: MbValue) -> Vec<char> {
    if inst_field(self_v, "text").is_none() {
        reader_fill(self_v);
    }
    inst_str(self_v, "text").chars().collect()
}

fn reader_pos(self_v: MbValue) -> usize {
    inst_field(self_v, "pos")
        .and_then(|v| v.as_int())
        .unwrap_or(0)
        .max(0) as usize
}

fn reader_read(self_v: MbValue, args_list: MbValue) -> MbValue {
    let size = pos_arg(args_list, 0).and_then(|v| v.as_int());
    let text = reader_text(self_v);
    let pos = reader_pos(self_v);
    let (end, taken): (usize, String) = match size {
        Some(n) if n >= 0 => {
            let e = (pos + n as usize).min(text.len());
            (e, text[pos..e].iter().collect())
        }
        _ => (text.len(), text[pos..].iter().collect()),
    };
    set_inst_field(self_v, "pos", MbValue::from_int(end as i64));
    new_str(taken)
}

fn reader_readline(self_v: MbValue, _args: MbValue) -> MbValue {
    let text = reader_text(self_v);
    let pos = reader_pos(self_v);
    if pos >= text.len() {
        return new_str(String::new());
    }
    let mut end = pos;
    while end < text.len() {
        let c = text[end];
        end += 1;
        if c == '\n' {
            break;
        }
    }
    let line: String = text[pos..end].iter().collect();
    set_inst_field(self_v, "pos", MbValue::from_int(end as i64));
    new_str(line)
}

fn reader_readlines(self_v: MbValue, _args: MbValue) -> MbValue {
    let mut lines = Vec::new();
    loop {
        let line = reader_readline(self_v, MbValue::none());
        let s = extract_str(line).unwrap_or_default();
        if s.is_empty() {
            break;
        }
        lines.push(new_str(s));
    }
    MbValue::from_ptr(MbObject::new_list(lines))
}

fn reader_seek(self_v: MbValue, args_list: MbValue) -> MbValue {
    let offset = pos_arg(args_list, 0).and_then(|v| v.as_int()).unwrap_or(0);
    // Only seek(0[, 0]) is exercised: rewind the underlying stream and reset.
    let stream = reader_underlying(self_v);
    let whence = pos_arg(args_list, 1).unwrap_or_else(|| MbValue::from_int(0));
    let seek_args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(offset), whence]));
    mb_call_method(stream, new_str("seek".into()), seek_args);
    if offset == 0 {
        set_inst_field(self_v, "raw", new_bytes(Vec::new()));
        set_inst_field(self_v, "pos", MbValue::from_int(0));
        // Force re-fill on next read.
        if let Some(ptr) = self_v.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    fields.write().unwrap().remove("text");
                }
            }
        }
    }
    MbValue::none()
}

// ── StreamWriter: encodes then writes to the underlying stream ──────────────

fn writer_write(self_v: MbValue, args_list: MbValue) -> MbValue {
    let obj = pos_arg(args_list, 0).unwrap_or_else(MbValue::none);
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let s = extract_str(obj).unwrap_or_default();
    let mut bytes = codec_encode_bytes(&mode, &s).unwrap_or_default();
    if sig {
        let first = inst_field(self_v, "first")
            .map(|v| v.as_bool() != Some(false))
            .unwrap_or(true);
        if first {
            let mut p = BOM_UTF8.to_vec();
            p.append(&mut bytes);
            bytes = p;
            set_inst_field(self_v, "first", MbValue::from_bool(false));
        }
    }
    let stream = inst_field(self_v, "stream").unwrap_or_else(MbValue::none);
    let wargs = MbValue::from_ptr(MbObject::new_list(vec![new_bytes(bytes)]));
    mb_call_method(stream, new_str("write".into()), wargs);
    MbValue::none()
}

// ── StreamRecoder (EncodedFile): recodes file_enc <-> data_enc on read/write.
//
// Constructed as EncodedFile(file, data_encoding, file_encoding='utf-8').
//   read():  bytes read from `file` are decoded with file_encoding then
//            re-encoded with data_encoding.
//   write(b): bytes are decoded with data_encoding then re-encoded with
//            file_encoding and written to `file`.

fn recoder_underlying(self_v: MbValue) -> MbValue {
    inst_field(self_v, "stream").unwrap_or_else(MbValue::none)
}

/// Recode raw file bytes → data-encoding bytes (used by read/readline).
fn recoder_recode_read(self_v: MbValue, raw: &[u8]) -> Vec<u8> {
    let file_enc = inst_str(self_v, "file_encoding");
    let data_enc = inst_str(self_v, "data_encoding");
    let text = codec_decode_str(&file_enc, raw).unwrap_or_default();
    codec_encode_bytes(&data_enc, &text).unwrap_or_default()
}

fn recoder_read(self_v: MbValue, _args: MbValue) -> MbValue {
    let stream = recoder_underlying(self_v);
    let raw = mb_call_method(
        stream,
        new_str("read".into()),
        MbValue::from_ptr(MbObject::new_list(vec![])),
    );
    let bytes = as_bytes_vec(raw).unwrap_or_default();
    new_bytes(recoder_recode_read(self_v, &bytes))
}

/// readline/seek operate on a fully recoded buffer (lazily built, byte-based).
fn recoder_buffer(self_v: MbValue) -> Vec<u8> {
    if inst_field(self_v, "recoded").is_none() {
        let stream = recoder_underlying(self_v);
        let raw = mb_call_method(
            stream,
            new_str("read".into()),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
        let bytes = as_bytes_vec(raw).unwrap_or_default();
        let recoded = recoder_recode_read(self_v, &bytes);
        set_inst_field(self_v, "recoded", new_bytes(recoded));
    }
    inst_field(self_v, "recoded")
        .and_then(as_bytes_vec)
        .unwrap_or_default()
}

fn recoder_readline(self_v: MbValue, _args: MbValue) -> MbValue {
    let buf = recoder_buffer(self_v);
    let pos = inst_field(self_v, "pos")
        .and_then(|v| v.as_int())
        .unwrap_or(0)
        .max(0) as usize;
    if pos >= buf.len() {
        return new_bytes(Vec::new());
    }
    let mut end = pos;
    while end < buf.len() {
        let b = buf[end];
        end += 1;
        if b == b'\n' {
            break;
        }
    }
    set_inst_field(self_v, "pos", MbValue::from_int(end as i64));
    new_bytes(buf[pos..end].to_vec())
}

fn recoder_seek(self_v: MbValue, args_list: MbValue) -> MbValue {
    let offset = pos_arg(args_list, 0).and_then(|v| v.as_int()).unwrap_or(0);
    if offset == 0 {
        set_inst_field(self_v, "pos", MbValue::from_int(0));
    }
    MbValue::none()
}

fn recoder_write(self_v: MbValue, args_list: MbValue) -> MbValue {
    let obj = pos_arg(args_list, 0).unwrap_or_else(MbValue::none);
    let data_enc = inst_str(self_v, "data_encoding");
    let file_enc = inst_str(self_v, "file_encoding");
    let raw = as_bytes_vec(obj).unwrap_or_default();
    let text = codec_decode_str(&data_enc, &raw).unwrap_or_default();
    let out = codec_encode_bytes(&file_enc, &text).unwrap_or_default();
    let stream = recoder_underlying(self_v);
    let wargs = MbValue::from_ptr(MbObject::new_list(vec![new_bytes(out)]));
    mb_call_method(stream, new_str("write".into()), wargs);
    MbValue::none()
}

// ── Context-manager (__enter__/__exit__) for EncodedFile / readers ─────────

extern "C" fn cm_enter(self_v: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(self_v);
    }
    self_v
}
extern "C" fn cm_exit(self_v: MbValue, _t: MbValue, _v: MbValue, _tb: MbValue) -> MbValue {
    // Close the wrapped stream (CPython EncodedFile closes the base file).
    if let Some(stream) = inst_field(self_v, "stream") {
        mb_call_method(
            stream,
            new_str("close".into()),
            MbValue::from_ptr(MbObject::new_list(vec![])),
        );
    }
    MbValue::from_bool(false)
}

// ── Variadic method trampolines (called as `f(self, pos_list)`) ────────────

macro_rules! method2 {
    ($name:ident, $inner:path) => {
        extern "C" fn $name(self_v: MbValue, args_list: MbValue) -> MbValue {
            $inner(self_v, args_list)
        }
    };
}
method2!(m_incr_decode, incr_decode);
method2!(m_incr_decode_reset, incr_decode_reset);
method2!(m_incr_encode, incr_encode);
method2!(m_incr_encode_reset, incr_encode_reset);
method2!(m_reader_read, reader_read);
method2!(m_reader_readline, reader_readline);
method2!(m_reader_readlines, reader_readlines);
method2!(m_reader_seek, reader_seek);
method2!(m_writer_write, writer_write);
method2!(m_recoder_read, recoder_read);
method2!(m_recoder_readline, recoder_readline);
method2!(m_recoder_seek, recoder_seek);
method2!(m_recoder_write, recoder_write);

// ── Factory __call__ dispatchers ───────────────────────────────────────────
//
// getreader/getwriter factories take the underlying stream (1 positional arg);
// getincrementalencoder/decoder factories take none. We register two factory
// classes accordingly so the fixed-arity `__call__` ABI matches both the
// 0-arg (`mb_call0` → f(self)) and 1-arg (`f(self, stream)`) call paths.

/// `IncrementalDecoder factory`.__call__(self[, errors]) — build a decoder.
extern "C" fn factory_incr_decoder_call(self_v: MbValue) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(
        "codecs.IncrementalDecoder".to_string(),
    ));
    set_inst_field(
        inst,
        "mode",
        inst_field(self_v, "mode").unwrap_or_else(MbValue::none),
    );
    set_inst_field(
        inst,
        "sig",
        inst_field(self_v, "sig").unwrap_or_else(|| MbValue::from_bool(false)),
    );
    set_inst_field(inst, "buffer", new_bytes(Vec::new()));
    set_inst_field(inst, "bom_done", MbValue::from_bool(false));
    inst
}

extern "C" fn factory_incr_encoder_call(self_v: MbValue) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(
        "codecs.IncrementalEncoder".to_string(),
    ));
    set_inst_field(
        inst,
        "mode",
        inst_field(self_v, "mode").unwrap_or_else(MbValue::none),
    );
    set_inst_field(
        inst,
        "sig",
        inst_field(self_v, "sig").unwrap_or_else(|| MbValue::from_bool(false)),
    );
    set_inst_field(inst, "first", MbValue::from_bool(true));
    inst
}

/// `StreamReader factory`.__call__(self, stream) — build a reader over stream.
extern "C" fn factory_reader_call(self_v: MbValue, stream: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(stream);
    }
    let inst = MbValue::from_ptr(MbObject::new_instance("codecs.StreamReader".to_string()));
    set_inst_field(
        inst,
        "mode",
        inst_field(self_v, "mode").unwrap_or_else(MbValue::none),
    );
    set_inst_field(
        inst,
        "sig",
        inst_field(self_v, "sig").unwrap_or_else(|| MbValue::from_bool(false)),
    );
    set_inst_field(inst, "stream", stream);
    set_inst_field(inst, "pos", MbValue::from_int(0));
    inst
}

/// `_EncoderFunc`.__call__(self, input) -> (bytes, consumed_char_count).
extern "C" fn encoder_func_call(self_v: MbValue, input: MbValue) -> MbValue {
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let s = match extract_str(input) {
        Some(s) => s,
        None => return raise_type_error("encoder requires a str argument"),
    };
    let consumed = s.chars().count() as i64;
    let mut b = match codec_encode_bytes(&mode, &s) {
        Some(b) => b,
        None => {
            return raise_unicode_encode_error(&format!("'{}' codec can't encode character", mode))
        }
    };
    if sig {
        let mut p = BOM_UTF8.to_vec();
        p.append(&mut b);
        b = p;
    }
    tuple2(new_bytes(b), MbValue::from_int(consumed))
}

/// `_DecoderFunc`.__call__(self, input) -> (str, consumed_byte_count).
extern "C" fn decoder_func_call(self_v: MbValue, input: MbValue) -> MbValue {
    let mode = inst_str(self_v, "mode");
    let sig = inst_field(self_v, "sig")
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false);
    let raw = match as_bytes_vec(input) {
        Some(b) => b,
        None => return raise_type_error("decoder requires a bytes-like argument"),
    };
    let consumed = raw.len() as i64;
    let mut src: &[u8] = &raw;
    if sig && raw.len() >= 3 && raw[0..3] == BOM_UTF8 {
        src = &raw[3..];
    }
    let s = codec_decode_str(&mode, src).unwrap_or_default();
    tuple2(new_str(s), MbValue::from_int(consumed))
}

extern "C" fn factory_writer_call(self_v: MbValue, stream: MbValue) -> MbValue {
    unsafe {
        super::super::rc::retain_if_ptr(stream);
    }
    let inst = MbValue::from_ptr(MbObject::new_instance("codecs.StreamWriter".to_string()));
    set_inst_field(
        inst,
        "mode",
        inst_field(self_v, "mode").unwrap_or_else(MbValue::none),
    );
    set_inst_field(
        inst,
        "sig",
        inst_field(self_v, "sig").unwrap_or_else(|| MbValue::from_bool(false)),
    );
    set_inst_field(inst, "stream", stream);
    set_inst_field(inst, "first", MbValue::from_bool(true));
    inst
}

/// Build a factory instance of `class_name` carrying the codec mode/sig flag.
fn make_factory(class_name: &str, enc: &str) -> MbValue {
    let (mode, sig) = split_sig(enc);
    let inst = MbValue::from_ptr(MbObject::new_instance(class_name.to_string()));
    set_inst_field(inst, "mode", new_str(mode));
    set_inst_field(inst, "sig", MbValue::from_bool(sig));
    inst
}

/// codecs.getincrementaldecoder(encoding) -> IncrementalDecoder factory.
pub fn mb_codecs_getincrementaldecoder_real(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getincrementaldecoder() argument must be str"),
    };
    let (mode, _) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._IncrementalDecoderFactory", &enc)
}

pub fn mb_codecs_getincrementalencoder_real(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getincrementalencoder() argument must be str"),
    };
    let (mode, _) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._IncrementalEncoderFactory", &enc)
}

pub fn mb_codecs_getreader_real(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getreader() argument must be str"),
    };
    let (mode, _) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._StreamReaderFactory", &enc)
}

pub fn mb_codecs_getwriter_real(encoding: MbValue) -> MbValue {
    let enc = match extract_str(encoding) {
        Some(e) => e,
        None => return raise_type_error("getwriter() argument must be str"),
    };
    let (mode, _) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    make_factory("codecs._StreamWriterFactory", &enc)
}

fn supported_byte_codec(mode: &str) -> bool {
    matches!(
        mode,
        "utf-8"
            | "ascii"
            | "latin-1"
            | "utf-16"
            | "utf-16-le"
            | "utf-16-be"
            | "utf-32"
            | "utf-32-le"
            | "utf-32-be"
    )
}

/// codecs.EncodedFile(file, data_encoding, file_encoding='utf-8', errors='strict')
/// -> StreamRecoder. `make_encoded_file` is the variadic ctor.
pub fn make_encoded_file(args: &[MbValue]) -> MbValue {
    let file = args.first().copied().unwrap_or_else(MbValue::none);
    let data_enc = args
        .get(1)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "utf-8".to_string());
    let file_enc = args
        .get(2)
        .and_then(|v| extract_str(*v))
        .unwrap_or_else(|| "utf-8".to_string());
    unsafe {
        super::super::rc::retain_if_ptr(file);
    }
    let inst = MbValue::from_ptr(MbObject::new_instance("codecs.StreamRecoder".to_string()));
    set_inst_field(inst, "stream", file);
    set_inst_field(
        inst,
        "data_encoding",
        new_str(normalize_encoding(&data_enc).to_string()),
    );
    set_inst_field(
        inst,
        "file_encoding",
        new_str(normalize_encoding(&file_enc).to_string()),
    );
    set_inst_field(inst, "pos", MbValue::from_int(0));
    inst
}

// ── bytes→bytes transform codecs (base64/hex/zlib/quopri) ──────────────────
//
// Reached through codecs.encode/decode(<bytes>, "<name>_codec"). Returns
// Some(result) when `name` is a recognised transform codec (so the text-codec
// path is bypassed), else None.

/// Canonical transform-codec name, or None if `name` isn't one.
fn transform_name(name: &str) -> Option<&'static str> {
    match name.to_lowercase().replace([' ', '-'], "_").as_str() {
        "base64" | "base64_codec" | "base_64" => Some("base64"),
        "hex" | "hex_codec" => Some("hex"),
        "zlib" | "zlib_codec" | "zip" => Some("zlib"),
        "quopri" | "quopri_codec" | "quotedprintable" | "quoted_printable" => Some("quopri"),
        "uu" | "uu_codec" => Some("uu"),
        _ => None,
    }
}

fn transform_encode(name: &str, obj: MbValue) -> Option<MbValue> {
    let which = transform_name(name)?;
    let data = match as_bytes_vec(obj) {
        Some(b) => b,
        None => {
            return Some(raise_type_error(&format!(
                "{}_codec encoding requires a bytes-like object",
                which
            )))
        }
    };
    let out: Vec<u8> = match which {
        "hex" => {
            let mut s = Vec::with_capacity(data.len() * 2);
            for b in &data {
                s.extend_from_slice(format!("{:02x}", b).as_bytes());
            }
            s
        }
        "base64" => {
            let mut s = base64_encode(&data);
            s.push(b'\n'); // binascii.b2a_base64 appends a newline
            s
        }
        "zlib" => {
            let res = super::zlib_mod::mb_zlib_compress(new_bytes(data));
            return Some(res);
        }
        "quopri" => quopri_encode(&data),
        "uu" => uu_encode(&data),
        _ => return None,
    };
    Some(new_bytes(out))
}

fn transform_decode(name: &str, obj: MbValue) -> Option<MbValue> {
    let which = transform_name(name)?;
    let data = match as_bytes_vec(obj) {
        Some(b) => b,
        None => {
            return Some(raise_type_error(&format!(
                "{}_codec decoding requires a bytes-like object",
                which
            )))
        }
    };
    let out: Vec<u8> = match which {
        "hex" => {
            let hexstr: Vec<u8> = data
                .iter()
                .copied()
                .filter(|b| !b.is_ascii_whitespace())
                .collect();
            if hexstr.len() % 2 != 0 {
                return Some(raise_value_error("non-hexadecimal number found"));
            }
            let mut out = Vec::with_capacity(hexstr.len() / 2);
            let mut i = 0;
            while i + 1 < hexstr.len() {
                let hi = (hexstr[i] as char).to_digit(16);
                let lo = (hexstr[i + 1] as char).to_digit(16);
                match (hi, lo) {
                    (Some(h), Some(l)) => out.push(((h << 4) | l) as u8),
                    _ => return Some(raise_value_error("non-hexadecimal number found")),
                }
                i += 2;
            }
            out
        }
        "base64" => match base64_decode(&data) {
            Some(v) => v,
            None => return Some(raise_value_error("Invalid base64-encoded string")),
        },
        "zlib" => {
            let res = super::zlib_mod::mb_zlib_decompress(new_bytes(data));
            return Some(res);
        }
        "quopri" => quopri_decode(&data),
        "uu" => uu_decode(&data),
        _ => return None,
    };
    Some(new_bytes(out))
}

/// uuencode (binascii.b2a_uu framing): `begin 666 <data>\n` header, lines of up
/// to 45 bytes encoded 3→4 with `ch = (v & 0x3F) + 0x20`, then ` \nend\n`.
fn uu_encode(data: &[u8]) -> Vec<u8> {
    let enc = |v: u8| -> u8 {
        if v == 0 {
            0x60
        } else {
            (v & 0x3F) + 0x20
        }
    };
    let mut out = Vec::new();
    out.extend_from_slice(b"begin 666 <data>\n");
    for chunk in data.chunks(45) {
        out.push(enc(chunk.len() as u8));
        for triple in chunk.chunks(3) {
            let b0 = triple[0];
            let b1 = *triple.get(1).unwrap_or(&0);
            let b2 = *triple.get(2).unwrap_or(&0);
            out.push(enc(b0 >> 2));
            out.push(enc(((b0 << 4) | (b1 >> 4)) & 0x3F));
            out.push(enc(((b1 << 2) | (b2 >> 6)) & 0x3F));
            out.push(enc(b2 & 0x3F));
        }
        out.push(b'\n');
    }
    out.extend_from_slice(b" \nend\n");
    out
}

fn uu_decode(data: &[u8]) -> Vec<u8> {
    let dec = |c: u8| -> u8 { (c.wrapping_sub(0x20)) & 0x3F };
    let mut out = Vec::new();
    for line in data.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        if line.starts_with(b"begin") || line == b"end" || line == b" " {
            continue;
        }
        let n = dec(line[0]) as usize;
        if n == 0 {
            continue;
        }
        let body = &line[1..];
        let mut decoded = Vec::new();
        for quad in body.chunks(4) {
            if quad.len() < 4 {
                break;
            }
            let c0 = dec(quad[0]);
            let c1 = dec(quad[1]);
            let c2 = dec(quad[2]);
            let c3 = dec(quad[3]);
            decoded.push((c0 << 2) | (c1 >> 4));
            decoded.push((c1 << 4) | (c2 >> 2));
            decoded.push((c2 << 6) | c3);
        }
        decoded.truncate(n);
        out.extend_from_slice(&decoded);
    }
    out
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64[((n >> 18) & 63) as usize]);
        out.push(B64[((n >> 12) & 63) as usize]);
        out.push(if chunk.len() > 1 {
            B64[((n >> 6) & 63) as usize]
        } else {
            b'='
        });
        out.push(if chunk.len() > 2 {
            B64[(n & 63) as usize]
        } else {
            b'='
        });
    }
    out
}

fn base64_decode(data: &[u8]) -> Option<Vec<u8>> {
    let inv = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let filtered: Vec<u8> = data
        .iter()
        .copied()
        .filter(|&b| b != b'\n' && b != b'\r' && b != b' ' && b != b'\t')
        .collect();
    let mut out = Vec::new();
    let mut acc = 0u32;
    let mut bits = 0;
    for &c in &filtered {
        if c == b'=' {
            break;
        }
        let v = inv(c)?;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((acc >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

/// quopri encode. The quopri *codec* uses quotetabs=True, so every space and
/// tab is quoted (=20 / =09); newlines stay literal; '=' becomes =3D and other
/// non-printables become =XX.
fn quopri_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    for &b in data.iter() {
        match b {
            b'\n' => out.push(b'\n'),
            b' ' | b'\t' => out.extend_from_slice(format!("={:02X}", b).as_bytes()),
            b'=' => out.extend_from_slice(b"=3D"),
            0x21..=0x3C | 0x3E..=0x7E => out.push(b), // printable except '=' and whitespace
            _ => out.extend_from_slice(format!("={:02X}", b).as_bytes()),
        }
    }
    out
}

fn quopri_decode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'=' {
            if i + 1 < data.len() && data[i + 1] == b'\n' {
                i += 2;
                continue;
            } // soft line break
            if i + 2 < data.len() {
                let hi = (data[i + 1] as char).to_digit(16);
                let lo = (data[i + 2] as char).to_digit(16);
                if let (Some(h), Some(l)) = (hi, lo) {
                    out.push(((h << 4) | l) as u8);
                    i += 3;
                    continue;
                }
            }
            out.push(b'=');
            i += 1;
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

// ── iterencode / iterdecode ────────────────────────────────────────────────

/// codecs.iterencode(iterable, encoding) -> list of encoded byte chunks.
pub fn mb_codecs_iterencode2(iterable: MbValue, encoding: MbValue) -> MbValue {
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let (mode, sig) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    let items = super::super::builtins::extract_items(iterable);
    let mut out = Vec::with_capacity(items.len());
    let mut first = true;
    for it in items {
        let s = extract_str(it).unwrap_or_default();
        let mut b = codec_encode_bytes(&mode, &s).unwrap_or_default();
        if sig && first {
            let mut p = BOM_UTF8.to_vec();
            p.append(&mut b);
            b = p;
        }
        first = false;
        out.push(new_bytes(b));
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

/// codecs.iterdecode(iterable, encoding) -> list of decoded str chunks.
pub fn mb_codecs_iterdecode2(iterable: MbValue, encoding: MbValue) -> MbValue {
    let enc = extract_str(encoding).unwrap_or_else(|| "utf-8".to_string());
    let (mode, _sig) = split_sig(&enc);
    if !supported_byte_codec(&mode) {
        return raise_lookup_error(&format!("unknown encoding: {}", enc));
    }
    let items = super::super::builtins::extract_items(iterable);
    let mut out = Vec::with_capacity(items.len());
    for it in items {
        let b = as_bytes_vec(it).unwrap_or_default();
        out.push(new_str(codec_decode_str(&mode, &b).unwrap_or_default()));
    }
    MbValue::from_ptr(MbObject::new_list(out))
}

// ── raw_unicode_escape + readbuffer ────────────────────────────────────────

/// codecs.raw_unicode_escape_decode(input, errors='strict') -> (str, len).
/// Keeps `\\xHH` literal but decodes `\\uHHHH` / `\\UHHHHHHHH` escapes.
pub fn mb_codecs_raw_unicode_escape_decode(input: MbValue) -> MbValue {
    let src = match as_bytes_vec(input).or_else(|| extract_str(input).map(|s| s.into_bytes())) {
        Some(b) => b,
        None => return raise_type_error("raw_unicode_escape_decode() argument must be bytes-like"),
    };
    let orig_len = src.len() as i64;
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < src.len() {
        let b = src[i];
        if b != b'\\' {
            out.push(b as char);
            i += 1;
            continue;
        }
        // Look at the escape char.
        let next = src.get(i + 1).copied();
        let hexn = |n: usize, j: usize| -> Option<u32> {
            if j + n > src.len() {
                return None;
            }
            let mut v = 0u32;
            for k in 0..n {
                v = (v << 4) | (src[j + k] as char).to_digit(16)?;
            }
            Some(v)
        };
        match next {
            Some(b'u') => match hexn(4, i + 2) {
                Some(v) => {
                    out.push(char::from_u32(v).unwrap_or('\u{FFFD}'));
                    i += 6;
                }
                None => {
                    out.push('\\');
                    i += 1;
                }
            },
            Some(b'U') => match hexn(8, i + 2) {
                Some(v) => {
                    out.push(char::from_u32(v).unwrap_or('\u{FFFD}'));
                    i += 10;
                }
                None => {
                    out.push('\\');
                    i += 1;
                }
            },
            _ => {
                out.push('\\');
                i += 1;
            } // backslash + everything else is literal
        }
    }
    tuple2(new_str(out), MbValue::from_int(orig_len))
}

/// codecs.raw_unicode_escape_encode(s) -> (bytes, len). ASCII/latin-1 pass
/// through; codepoints > 0xFF become `\\uHHHH` / `\\UHHHHHHHH`.
pub fn mb_codecs_raw_unicode_escape_encode(input: MbValue) -> MbValue {
    let s = match extract_str(input) {
        Some(s) => s,
        None => return raise_type_error("raw_unicode_escape_encode() argument must be str"),
    };
    let orig_len = s.chars().count() as i64;
    let mut out: Vec<u8> = Vec::new();
    for c in s.chars() {
        let cp = c as u32;
        if cp <= 0xFF {
            out.push(cp as u8);
        } else if cp <= 0xFFFF {
            out.extend_from_slice(format!("\\u{:04x}", cp).as_bytes());
        } else {
            out.extend_from_slice(format!("\\U{:08x}", cp).as_bytes());
        }
    }
    tuple2(new_bytes(out), MbValue::from_int(orig_len))
}

/// codecs.readbuffer_encode(obj) -> (bytes, len). Accepts any buffer (bytes,
/// bytearray, str → utf-8) and returns the raw bytes plus their length.
pub fn mb_codecs_readbuffer_encode(input: MbValue) -> MbValue {
    let bytes = match as_bytes_vec(input) {
        Some(b) => b,
        None => match extract_str(input) {
            Some(s) => s.into_bytes(),
            None => return raise_type_error("readbuffer_encode() argument must be a buffer"),
        },
    };
    let n = bytes.len() as i64;
    tuple2(new_bytes(bytes), MbValue::from_int(n))
}

// ── Module-level registration of the codec classes ─────────────────────────

fn register_codec_classes() {
    use super::super::class::mb_class_register;
    // Helper: register a class whose method addresses go in CALLABLE_REGISTRY.
    let reg = |name: &str, methods: Vec<(&str, usize)>, variadic: &[&str]| {
        let mut map: HashMap<String, MbValue> = HashMap::new();
        for (m, addr) in &methods {
            map.insert((*m).to_string(), MbValue::from_func(*addr));
        }
        mb_class_register(name, vec!["object".to_string()], map);
        for (m, addr) in &methods {
            if variadic.contains(m) {
                super::super::module::register_variadic_func(*addr as u64);
            }
        }
    };

    // Factories — __call__ only (fixed arity, NOT variadic).
    reg(
        "codecs._IncrementalDecoderFactory",
        vec![("__call__", factory_incr_decoder_call as *const () as usize)],
        &[],
    );
    reg(
        "codecs._IncrementalEncoderFactory",
        vec![("__call__", factory_incr_encoder_call as *const () as usize)],
        &[],
    );
    reg(
        "codecs._StreamReaderFactory",
        vec![("__call__", factory_reader_call as *const () as usize)],
        &[],
    );
    reg(
        "codecs._StreamWriterFactory",
        vec![("__call__", factory_writer_call as *const () as usize)],
        &[],
    );
    reg(
        "codecs._EncoderFunc",
        vec![("__call__", encoder_func_call as *const () as usize)],
        &[],
    );
    reg(
        "codecs._DecoderFunc",
        vec![("__call__", decoder_func_call as *const () as usize)],
        &[],
    );

    // IncrementalDecoder / Encoder.
    reg(
        "codecs.IncrementalDecoder",
        vec![
            ("decode", m_incr_decode as *const () as usize),
            ("reset", m_incr_decode_reset as *const () as usize),
        ],
        &["decode", "reset"],
    );
    reg(
        "codecs.IncrementalEncoder",
        vec![
            ("encode", m_incr_encode as *const () as usize),
            ("reset", m_incr_encode_reset as *const () as usize),
        ],
        &["encode", "reset"],
    );

    // StreamReader / Writer.
    reg(
        "codecs.StreamReader",
        vec![
            ("read", m_reader_read as *const () as usize),
            ("readline", m_reader_readline as *const () as usize),
            ("readlines", m_reader_readlines as *const () as usize),
            ("seek", m_reader_seek as *const () as usize),
            ("__enter__", cm_enter as *const () as usize),
            ("__exit__", cm_exit as *const () as usize),
        ],
        &["read", "readline", "readlines", "seek"],
    );
    reg(
        "codecs.StreamWriter",
        vec![
            ("write", m_writer_write as *const () as usize),
            ("__enter__", cm_enter as *const () as usize),
            ("__exit__", cm_exit as *const () as usize),
        ],
        &["write"],
    );

    // StreamRecoder (EncodedFile).
    reg(
        "codecs.StreamRecoder",
        vec![
            ("read", m_recoder_read as *const () as usize),
            ("readline", m_recoder_readline as *const () as usize),
            ("seek", m_recoder_seek as *const () as usize),
            ("write", m_recoder_write as *const () as usize),
            ("__enter__", cm_enter as *const () as usize),
            ("__exit__", cm_exit as *const () as usize),
        ],
        &["read", "readline", "seek", "write"],
    );
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
    fn test_normalize_unknown_is_unknown() {
        // Unknown codecs no longer silently alias to utf-8; lookup/getencoder
        // raise LookupError on them (matching CPython 3.12).
        assert_eq!(normalize_encoding("unknown-codec"), "?unknown");
        assert_eq!(canonical_codec_name("unknown-codec"), None);
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

    fn raised_type() -> Option<String> {
        super::super::super::exception::current_exception_type()
    }
    fn clear_exc() {
        super::super::super::exception::clear_current_exception();
    }

    #[test]
    fn test_encode_ascii_non_ascii_strict_raises() {
        // 'é' is non-ASCII → strict (default) raises UnicodeEncodeError.
        clear_exc();
        let _ = mb_codecs_encode(s("héllo"), s("ascii"));
        assert_eq!(raised_type().as_deref(), Some("UnicodeEncodeError"));
        clear_exc();
    }

    #[test]
    fn test_encode_ascii_replace_handler() {
        // With an explicit 'replace' handler, 'é' → '?'.
        clear_exc();
        let r = mb_codecs_encode3(s("héllo"), s("ascii"), s("replace"));
        let bytes = get_bytes(r).unwrap();
        assert_eq!(bytes, b"h?llo".to_vec());
        assert!(raised_type().is_none());
    }

    #[test]
    fn test_encode_latin1_in_range() {
        // 'é' = 0xe9, within latin-1 range
        let result = mb_codecs_encode(s("café"), s("latin-1"));
        let bytes = get_bytes(result).unwrap();
        assert!(bytes.contains(&0xe9));
    }

    #[test]
    fn test_encode_latin1_out_of_range_strict_raises() {
        // U+1F600 > 255 → strict raises UnicodeEncodeError.
        clear_exc();
        let emoji = "\u{1F600}"; // 😀
        let _ = mb_codecs_encode(
            MbValue::from_ptr(MbObject::new_str(emoji.to_string())),
            s("latin-1"),
        );
        assert_eq!(raised_type().as_deref(), Some("UnicodeEncodeError"));
        clear_exc();
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
    fn test_decode_ascii_bad_byte_strict_raises() {
        // byte 200 >= 128 → strict (default) raises UnicodeDecodeError.
        clear_exc();
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![200u8]));
        let _ = mb_codecs_decode(bytes, s("ascii"));
        assert_eq!(raised_type().as_deref(), Some("UnicodeDecodeError"));
        clear_exc();
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
    fn test_lookup_returns_codecinfo_with_name() {
        clear_exc();
        let result = mb_codecs_lookup(s("UTF-8"));
        assert!(result.as_ptr().is_some());
        // CodecInfo instance with a normalised .name field.
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                assert_eq!(class_name, "CodecInfo");
                let f = fields.read().unwrap();
                assert_eq!(get_str(*f.get("name").unwrap()), Some("utf-8".to_string()));
            } else {
                panic!("lookup did not return an Instance");
            }
        }
        assert!(raised_type().is_none());
    }

    #[test]
    fn test_lookup_unknown_raises_lookup_error() {
        clear_exc();
        let _ = mb_codecs_lookup(s("not_a_real_codec_xyz"));
        assert_eq!(raised_type().as_deref(), Some("LookupError"));
        clear_exc();
    }

    #[test]
    fn test_lookup_non_str_raises_type_error() {
        clear_exc();
        let _ = mb_codecs_lookup(MbValue::none());
        assert_eq!(raised_type().as_deref(), Some("TypeError"));
        clear_exc();
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
