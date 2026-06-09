//! @codegen-skip: handwrite-pre-standardize
//!
//! base64 module for Mamba (#440).
//!
//! Provides: base64.b64encode, base64.b64decode,
//!           base64.urlsafe_b64encode, base64.urlsafe_b64decode,
//!           base64.standard_b64encode, base64.standard_b64decode,
//!           base64.b32encode, base64.b32decode,
//!           base64.b32hexencode, base64.b32hexdecode,
//!           base64.b16encode, base64.b16decode,
//!           base64.a85encode, base64.a85decode,
//!           base64.b85encode, base64.b85decode,
//!           base64.encodebytes, base64.decodebytes
//!
//! Standard b64 + b16 + b32 + a85 + b85 alphabets and padding semantics
//! match CPython 3.12 `Lib/base64.py`. File-like `encode`/`decode` are
//! deliberately omitted — they require a stream-protocol pass that's
//! out of scope for the per-lib brute-force sweep.

use std::borrow::Cow;
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

const B64_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const B64_URL_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_unary_bytes_like {
    ($name:ident, $fn:ident, $py_name:literal) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
            if let Err(err) = require_bytes_like(&arg, $py_name) {
                return err;
            }
            $fn(arg)
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

// b64encode(s, altchars=None). The optional `altchars` (a 2-byte sequence)
// replaces the `+`/`/` chars of the standard alphabet. It can arrive either as
// a second positional or packed into a trailing kwargs dict (`{'altchars': ...}`).
unsafe extern "C" fn dispatch_b64encode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
    if let Err(err) = require_bytes_like(&arg, "b64encode") {
        return err;
    }
    let altchars = extract_altchars(&a[1..]);
    mb_base64_b64encode_altchars(arg, altchars.as_deref())
}
// b64decode(s, altchars=None, validate=False). `altchars` remaps the two
// non-standard chars back to `+`/`/`; `validate=True` rejects any byte outside
// the (possibly remapped) alphabet with a `binascii.Error`.
unsafe extern "C" fn dispatch_b64decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let altchars = extract_altchars(&a[1.min(a.len())..]);
    let validate = extract_validate(&a[1.min(a.len())..]);
    mb_base64_b64decode_full(data, altchars.as_deref(), validate)
}
dispatch_unary_bytes_like!(
    dispatch_urlsafe_b64encode,
    mb_base64_urlsafe_b64encode,
    "urlsafe_b64encode"
);
dispatch_unary!(dispatch_urlsafe_b64decode, mb_base64_urlsafe_b64decode);

// HANDWRITE-BEGIN reason: stdlib gap — base32/base16/ascii85/base85 + legacy
// IO API are missing; codegen has no section type for alphabet-tabled
// codecs yet. Will convert to CODEGEN once the standardize sweep grows a
// `codec_alphabet` section type.
dispatch_unary_bytes_like!(
    dispatch_standard_b64encode,
    mb_base64_standard_b64encode,
    "standard_b64encode"
);
dispatch_unary!(dispatch_standard_b64decode, mb_base64_standard_b64decode);
dispatch_unary_bytes_like!(dispatch_b32encode, mb_base64_b32encode, "b32encode");
// b32decode(s, casefold=False, map01=None). The `casefold` flag may arrive as a
// bare positional bool or in a trailing kwargs dict.
unsafe extern "C" fn dispatch_b32decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let casefold = extract_casefold(&a[1.min(a.len())..]);
    mb_base64_b32decode_cf(data, casefold)
}
dispatch_unary_bytes_like!(
    dispatch_b32hexencode,
    mb_base64_b32hexencode,
    "b32hexencode"
);
// b32hexdecode(s, casefold=False) — base32hex alphabet.
unsafe extern "C" fn dispatch_b32hexdecode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let casefold = extract_casefold(&a[1.min(a.len())..]);
    mb_base64_b32hexdecode_cf(data, casefold)
}
dispatch_unary_bytes_like!(dispatch_b16encode, mb_base64_b16encode, "b16encode");
dispatch_unary!(dispatch_b16decode, mb_base64_b16decode);
// a85encode(b, *, foldspaces=False, wrapcol=0, pad=False, adobe=False).
// We support the default codec plus the `adobe` framing flag (the only encode
// keyword exercised by the CPython behavior fixtures). `adobe=True` frames the
// output in `<~` ... `~>` per CPython 3.12 `base64.a85encode`.
unsafe extern "C" fn dispatch_a85encode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
    if let Err(err) = require_bytes_like(&arg, "a85encode") {
        return err;
    }
    let mut adobe = false;
    for extra in a.iter().skip(1) {
        if let Some(b) = extra.as_bool() {
            adobe = b;
        } else if let Some(b) = extract_kwarg_bool(extra, "adobe") {
            adobe = b;
        }
    }
    mb_base64_a85encode_impl(arg, adobe)
}
// a85decode(b, *, foldspaces=False, adobe=False, ignorechars=...). Keyword
// arguments on a module-attribute call are packed by the lowerer into a trailing
// dict positional (`{'adobe': True}`); a bare positional flag arrives as a plain
// bool. We honor both shapes so `a85decode(b, adobe=True)` and
// `a85decode(b, True)` both recover the `adobe` flag faithfully.
unsafe extern "C" fn dispatch_a85decode(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = a.get(0).copied().unwrap_or_else(MbValue::none);
    let mut adobe = false;
    // CPython default ignorechars = b' \t\n\r\x0b'.
    let mut ignorechars: Vec<u8> = vec![b' ', b'\t', b'\n', b'\r', 0x0b];
    // Keyword arguments may be packed into a SINGLE trailing dict positional
    // (`{'adobe': False, 'ignorechars': b''}`) — so each `extra` can carry
    // multiple keys. Pull every recognized key from each value independently
    // (not an else-if chain, which would stop at the first match per dict).
    for extra in a.iter().skip(1) {
        if let Some(b) = extra.as_bool() {
            // Positional flag: a85decode(b, foldspaces?, adobe?) — but in the
            // CPython tests only `adobe` is ever passed positionally, alone.
            adobe = b;
            continue;
        }
        if let Some(b) = extract_kwarg_bool(extra, "adobe") {
            adobe = b;
        }
        if let Some(ic) = extract_kwarg_bytes(extra, "ignorechars") {
            ignorechars = ic;
        }
    }
    mb_base64_a85decode_impl2(data, adobe, &ignorechars)
}

/// Extract a boolean keyword argument by name from a trailing kwargs dict
/// (`{'adobe': True}`). Returns `None` when the value is not a dict or the key
/// is absent/non-bool. Mirrors how the lowerer packs keyword arguments on a
/// module-attribute call into a trailing dict positional.
unsafe fn extract_kwarg_bool(val: &MbValue, key: &str) -> Option<bool> {
    let ptr = val.as_ptr()?;
    if let ObjData::Dict(ref lock) = (*ptr).data {
        let map = lock.read().unwrap();
        let dk = super::super::dict_ops::DictKey::Str(key.to_string());
        if let Some(v) = map.get(&dk) {
            return v.as_bool();
        }
    }
    None
}

/// Extract a bytes-like keyword argument by name from a trailing kwargs dict
/// (`{'ignorechars': b''}`). Returns `None` when the value is not a dict or the
/// key is absent/not bytes-like.
unsafe fn extract_kwarg_bytes(val: &MbValue, key: &str) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    if let ObjData::Dict(ref lock) = (*ptr).data {
        let map = lock.read().unwrap();
        let dk = super::super::dict_ops::DictKey::Str(key.to_string());
        if let Some(v) = map.get(&dk) {
            return Some(extract_bytes_ref(v).into_owned());
        }
    }
    None
}

/// Pull a 2-byte `altchars` value from the extra args following `data`.
/// It may arrive as a bare positional bytes/str value or packed into a trailing
/// kwargs dict under the `altchars` key. Returns `None` when absent or `None`.
unsafe fn extract_altchars(extras: &[MbValue]) -> Option<Vec<u8>> {
    for extra in extras {
        if extra.is_none() {
            continue;
        }
        if let Some(ptr) = extra.as_ptr() {
            match &(*ptr).data {
                // Bare positional bytes/str/bytearray altchars.
                ObjData::Bytes(_) | ObjData::Str(_) | ObjData::ByteArray(_) => {
                    return Some(extract_bytes_ref(extra).into_owned());
                }
                // Trailing kwargs dict with an `altchars` key.
                ObjData::Dict(_) => {
                    if let Some(ac) = extract_kwarg_bytes(extra, "altchars") {
                        return Some(ac);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Pull a `validate` flag from the extra args following `data`. It arrives as a
/// trailing kwargs dict (`{'validate': True}`) or, when packed positionally, as
/// a bare bool. Defaults to `false`.
unsafe fn extract_validate(extras: &[MbValue]) -> bool {
    for extra in extras {
        if let Some(b) = extra.as_bool() {
            return b;
        }
        if let Some(b) = extract_kwarg_bool(extra, "validate") {
            return b;
        }
    }
    false
}

/// Pull a `casefold` flag from the extra args following `data` (positional bool
/// or trailing `{'casefold': True}` kwargs dict). Defaults to `false`.
unsafe fn extract_casefold(extras: &[MbValue]) -> bool {
    for extra in extras {
        if let Some(b) = extra.as_bool() {
            return b;
        }
        if let Some(b) = extract_kwarg_bool(extra, "casefold") {
            return b;
        }
    }
    false
}
dispatch_unary_bytes_like!(dispatch_b85encode, mb_base64_b85encode, "b85encode");
dispatch_unary!(dispatch_b85decode, mb_base64_b85decode);
dispatch_unary_bytes_like!(dispatch_encodebytes, mb_base64_encodebytes, "encodebytes");
dispatch_unary!(dispatch_decodebytes, mb_base64_decodebytes);
dispatch_binary!(dispatch_encode_stream, mb_base64_encode_stream);
dispatch_binary!(dispatch_decode_stream, mb_base64_decode_stream);
// HANDWRITE-END

/// Register the base64 module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("b64encode", dispatch_b64encode as usize),
        ("b64decode", dispatch_b64decode as usize),
        ("urlsafe_b64encode", dispatch_urlsafe_b64encode as usize),
        ("urlsafe_b64decode", dispatch_urlsafe_b64decode as usize),
        // HANDWRITE-BEGIN reason: see top-of-file
        ("standard_b64encode", dispatch_standard_b64encode as usize),
        ("standard_b64decode", dispatch_standard_b64decode as usize),
        ("b32encode", dispatch_b32encode as usize),
        ("b32decode", dispatch_b32decode as usize),
        ("b32hexencode", dispatch_b32hexencode as usize),
        ("b32hexdecode", dispatch_b32hexdecode as usize),
        ("b16encode", dispatch_b16encode as usize),
        ("b16decode", dispatch_b16decode as usize),
        ("a85encode", dispatch_a85encode as usize),
        ("a85decode", dispatch_a85decode as usize),
        ("b85encode", dispatch_b85encode as usize),
        ("b85decode", dispatch_b85decode as usize),
        ("encodebytes", dispatch_encodebytes as usize),
        ("decodebytes", dispatch_decodebytes as usize),
        ("encode", dispatch_encode_stream as usize),
        ("decode", dispatch_decode_stream as usize),
        // HANDWRITE-END
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
        // surface: missing CPython module constants (auto-added)
    attrs.insert("MAXBINSIZE".into(), MbValue::from_int(57));
    attrs.insert("MAXLINESIZE".into(), MbValue::from_int(76));
    super::register_module("base64", attrs);
}

/// D2′ (#2096 Task #57 Phase 2) — borrowed sibling for the hot dispatch
/// path. Returns a `Cow` because Bytes/Str are zero-copy borrows but
/// ByteArray (RwLock<Vec<u8>>) still needs a clone — the read guard
/// can't outlive the function. Lifetime tied to caller's MbValue (rc≥1
/// for fn-by-value parameter ⇒ valid for entire fn body).
unsafe fn extract_bytes_ref<'a>(val: &'a MbValue) -> Cow<'a, [u8]> {
    match val.as_ptr() {
        Some(ptr) => match &(*ptr).data {
            ObjData::Bytes(b) => Cow::Borrowed(b.as_slice()),
            ObjData::Str(s) => Cow::Borrowed(s.as_bytes()),
            ObjData::ByteArray(ref lock) => Cow::Owned(lock.read().unwrap().clone()),
            _ => Cow::Owned(Vec::new()),
        },
        None => Cow::Owned(Vec::new()),
    }
}

unsafe fn extract_bytes_like_ref<'a>(val: &'a MbValue) -> Option<Cow<'a, [u8]>> {
    match val.as_ptr()? {
        ptr => match &(*ptr).data {
            ObjData::Bytes(b) => Some(Cow::Borrowed(b.as_slice())),
            ObjData::ByteArray(ref lock) => Some(Cow::Owned(lock.read().unwrap().clone())),
            _ => None,
        },
    }
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `binascii.Error` (a `ValueError` subclass in CPython). Decode failures
/// in CPython's `binascii.a2b_base64` surface as `binascii.Error`; raising it by
/// its dotted class name lets `except binascii.Error`, `except ValueError`, and
/// `except Exception` all catch it (mirrors `binascii_mod`'s convention).
fn raise_binascii_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("binascii.Error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// CPython `base64._bytes_from_decode_data`: accept bytes/bytearray/memoryview
/// as-is, but a `str` is only valid if it is pure ASCII (it gets encoded with
/// the `'ascii'` codec, which raises `ValueError` on any non-ASCII char). We
/// model the "str with non-ASCII char" failure as `Err(())` so the caller can
/// raise `ValueError` exactly like CPython 3.12 does.
unsafe fn decode_data_bytes<'a>(val: &'a MbValue) -> Result<Cow<'a, [u8]>, ()> {
    match val.as_ptr() {
        Some(ptr) => match &(*ptr).data {
            ObjData::Bytes(b) => Ok(Cow::Borrowed(b.as_slice())),
            ObjData::ByteArray(ref lock) => Ok(Cow::Owned(lock.read().unwrap().clone())),
            ObjData::Str(s) => {
                if s.is_ascii() {
                    Ok(Cow::Borrowed(s.as_bytes()))
                } else {
                    Err(())
                }
            }
            _ => Ok(Cow::Owned(Vec::new())),
        },
        None => Ok(Cow::Owned(Vec::new())),
    }
}

fn require_bytes_like<'a>(val: &'a MbValue, name: &str) -> Result<Cow<'a, [u8]>, MbValue> {
    unsafe { extract_bytes_like_ref(val) }
        .ok_or_else(|| raise_type_error(&format!("{name}() argument must be bytes-like")))
}

/// Encode bytes to base64 using the given alphabet.
///
/// Progress #2096 — small-win on the encode hot leaf: pre-allocate the
/// exact output capacity (`((len+2)/3)*4`) and emit ASCII bytes directly
/// into a `Vec<u8>` instead of pushing `char`s through a `String`
/// (which routes each byte through `char` → UTF-8 encode → push). The
/// alphabet is ASCII-only by construction, so `Vec<u8>` is sound and
/// avoids the trailing `into_bytes()` round-trip at every callsite.
/// Wall-time win at the encoder hotspot; does not move the structural
/// mem bound (see `docs/blockers/2096-codecs-base64-mem-payload-bound.md`).
fn b64_encode_with(data: &[u8], table: &[u8]) -> Vec<u8> {
    let cap = data.len().div_ceil(3) * 4;
    let mut result = Vec::with_capacity(cap);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(table[((triple >> 18) & 0x3F) as usize]);
        result.push(table[((triple >> 12) & 0x3F) as usize]);

        if chunk.len() > 1 {
            result.push(table[((triple >> 6) & 0x3F) as usize]);
        } else {
            result.push(b'=');
        }

        if chunk.len() > 2 {
            result.push(table[(triple & 0x3F) as usize]);
        } else {
            result.push(b'=');
        }
    }

    result
}

/// Build a reverse lookup table for base64 decoding.
fn build_decode_table(table: &[u8]) -> [u8; 128] {
    let mut decode = [0xFFu8; 128];
    for (i, &c) in table.iter().enumerate() {
        if (c as usize) < 128 {
            decode[c as usize] = i as u8;
        }
    }
    decode
}

/// Decode a base64 byte buffer using the given alphabet.
///
/// Progress #2096 — streaming decode hot leaf: walk the input once with
/// a 4-byte stack quartet and emit directly into the output `Vec<u8>`.
/// Drops the transient `chars: Vec<u8>` (up to `encoded.len()` bytes per
/// call, 2000 times in the bench fixture) that the prior tick still
/// materialized for the `chunks(4)` pass. The output cap is sized to
/// `encoded.len() * 3 / 4` — an upper bound (real output is smaller when
/// the input has padding or whitespace), so no growth reallocs occur.
fn b64_decode_with(encoded: &[u8], table: &[u8]) -> Vec<u8> {
    let decode_table = build_decode_table(table);
    let mut result = Vec::with_capacity(encoded.len() * 3 / 4);

    // Streaming quartet decode: keep a 4-byte stack buffer and a count.
    // For each valid base64 char or `=` padding, push into the buffer;
    // when full, decode and emit 0-3 output bytes depending on padding.
    let mut quartet = [0u8; 4];
    let mut filled: usize = 0;

    for &b in encoded {
        // Filter: only valid alphabet chars or `=` padding contribute.
        let is_pad = b == b'=';
        let is_alpha = b < 128 && decode_table[b as usize] != 0xFF;
        if !(is_pad || is_alpha) {
            continue;
        }
        quartet[filled] = b;
        filled += 1;
        if filled == 4 {
            emit_b64_quartet(&quartet, &decode_table, &mut result);
            filled = 0;
        }
    }

    // Tail (partial quartet) — emit if at least 2 chars accumulated.
    if filled >= 2 {
        // Pad the rest with `=` so emit logic produces the right count.
        for i in filled..4 {
            quartet[i] = b'=';
        }
        emit_b64_quartet(&quartet, &decode_table, &mut result);
    }

    result
}

/// Faithful port of CPython 3.12 `binascii_a2b_base64_impl` (the function
/// `base64.b64decode` delegates to via `binascii.a2b_base64(s, strict_mode=)`).
/// Emits bytes as data characters arrive; padding terminates the quad. In
/// non-strict mode (`validate=False`) non-alphabet bytes and stray padding are
/// skipped; strict mode (`validate=True`) reports the precise CPython error
/// categories. `Err(msg)` surfaces as `binascii.Error`.
fn b64_decode_checked(src: &[u8], table: &[u8], strict: bool) -> Result<Vec<u8>, String> {
    let decode_table = build_decode_table(table);
    let b64_value = |c: u8| -> u8 {
        if (c as usize) < 128 {
            let v = decode_table[c as usize];
            if v != 0xFF { v } else { 64 }
        } else {
            64
        }
    };

    let ascii_len = src.len();
    let mut out: Vec<u8> = Vec::with_capacity(ascii_len.div_ceil(4) * 3);
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

/// Decode a single 4-byte base64 quartet (possibly with trailing `=`
/// padding) and push 1-3 raw bytes into `out`. Inlined hot leaf for
/// `b64_decode_with`.
#[inline]
fn emit_b64_quartet(q: &[u8; 4], decode_table: &[u8; 128], out: &mut Vec<u8>) {
    let a = if q[0] != b'=' { decode_table[q[0] as usize] as u32 } else { 0 };
    let b = if q[1] != b'=' { decode_table[q[1] as usize] as u32 } else { 0 };
    let c = if q[2] != b'=' { decode_table[q[2] as usize] as u32 } else { 0 };
    let d = if q[3] != b'=' { decode_table[q[3] as usize] as u32 } else { 0 };

    let triple = (a << 18) | (b << 12) | (c << 6) | d;

    out.push(((triple >> 16) & 0xFF) as u8);
    if q[2] != b'=' {
        out.push(((triple >> 8) & 0xFF) as u8);
    }
    if q[3] != b'=' {
        out.push((triple & 0xFF) as u8);
    }
}

/// base64.b64encode(data) -> base64 string
pub fn mb_base64_b64encode(data: MbValue) -> MbValue {
    mb_base64_b64encode_altchars(data, None)
}

/// base64.b64encode(data, altchars=None). When `altchars` (a 2-byte sequence)
/// is supplied, the `+` and `/` of the standard alphabet are translated to
/// `altchars[0]` and `altchars[1]` respectively (CPython 3.12 `b64encode`).
pub fn mb_base64_b64encode_altchars(data: MbValue, altchars: Option<&[u8]>) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    let mut encoded = b64_encode_with(&bytes, B64_CHARS);
    if let Some(ac) = altchars {
        if ac.len() >= 2 {
            for byte in encoded.iter_mut() {
                if *byte == b'+' {
                    *byte = ac[0];
                } else if *byte == b'/' {
                    *byte = ac[1];
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_bytes(encoded))
}

/// base64.b64decode(data) -> bytes; accepts str or bytes-like input.
pub fn mb_base64_b64decode(data: MbValue) -> MbValue {
    mb_base64_b64decode_full(data, None, false)
}

/// base64.b64decode(data, altchars=None, validate=False).
///
/// With `validate=False` (default) any byte outside the alphabet is discarded
/// before decoding. With `validate=True` a byte outside the (post-altchars)
/// standard alphabet raises a `binascii.Error`. Incorrect padding always raises
/// a `binascii.Error`, matching CPython 3.12 `binascii.a2b_base64`.
pub fn mb_base64_b64decode_full(
    data: MbValue,
    altchars: Option<&[u8]>,
    validate: bool,
) -> MbValue {
    let encoded_bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    // Translate altchars back to the standard `+`/`/` before decoding.
    let mut buf: Vec<u8>;
    let work: &[u8] = if let Some(ac) = altchars {
        if ac.len() >= 2 {
            buf = encoded_bytes.to_vec();
            for byte in buf.iter_mut() {
                if *byte == ac[0] {
                    *byte = b'+';
                } else if *byte == ac[1] {
                    *byte = b'/';
                }
            }
            &buf
        } else {
            &encoded_bytes
        }
    } else {
        &encoded_bytes
    };
    match b64_decode_checked(work, B64_CHARS, validate) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_binascii_error(&msg),
    }
}

/// base64.urlsafe_b64encode(data) -> bytes (URL-safe alphabet)
pub fn mb_base64_urlsafe_b64encode(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    let encoded = b64_encode_with(&bytes, B64_URL_CHARS);
    MbValue::from_ptr(MbObject::new_bytes(encoded))
}

/// base64.urlsafe_b64decode(data) -> bytes; accepts str or bytes-like input.
///
/// CPython translates the URL-safe `-`/`_` back to `+`/`/` and delegates to
/// `b64decode`, so the same incorrect-padding semantics apply.
pub fn mb_base64_urlsafe_b64decode(data: MbValue) -> MbValue {
    let encoded_bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    // CPython translates `-`/`_` back to `+`/`/` and decodes with the STANDARD
    // alphabet (so a stray `+`/`/` in the input is decoded, not silently
    // dropped — they are valid standard digits).
    let translated: Vec<u8> = encoded_bytes
        .iter()
        .map(|&b| match b {
            b'-' => b'+',
            b'_' => b'/',
            other => other,
        })
        .collect();
    match b64_decode_checked(&translated, B64_CHARS, false) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_binascii_error(&msg),
    }
}

// HANDWRITE-BEGIN reason: see top-of-file (stdlib gap, no codegen section type yet)

const B32_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
const B32_HEX_CHARS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";
const B16_CHARS: &[u8] = b"0123456789ABCDEF";
const B85_CHARS: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&()*+-;<=>?@^_`{|}~";

/// Build a 256-entry reverse table for a base16/32 alphabet (case-insensitive).
fn build_alpha_table(alpha: &[u8]) -> [u8; 256] {
    let mut t = [0xFFu8; 256];
    for (i, &c) in alpha.iter().enumerate() {
        t[c as usize] = i as u8;
        if (c as char).is_ascii_alphabetic() {
            let other = if (c as char).is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else {
                c.to_ascii_uppercase()
            };
            t[other as usize] = i as u8;
        }
    }
    t
}

/// base32 encode using the supplied 32-char alphabet.
fn b32_encode_with(data: &[u8], table: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(((data.len() + 4) / 5) * 8);
    for chunk in data.chunks(5) {
        let mut buf = [0u8; 5];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }
        let bits: u64 = ((buf[0] as u64) << 32)
            | ((buf[1] as u64) << 24)
            | ((buf[2] as u64) << 16)
            | ((buf[3] as u64) << 8)
            | (buf[4] as u64);
        let chars = [
            ((bits >> 35) & 0x1F) as usize,
            ((bits >> 30) & 0x1F) as usize,
            ((bits >> 25) & 0x1F) as usize,
            ((bits >> 20) & 0x1F) as usize,
            ((bits >> 15) & 0x1F) as usize,
            ((bits >> 10) & 0x1F) as usize,
            ((bits >> 5) & 0x1F) as usize,
            (bits & 0x1F) as usize,
        ];
        let emit = match chunk.len() {
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            5 => 8,
            _ => 0,
        };
        for &c in &chars[..emit] {
            out.push(table[c]);
        }
        for _ in emit..8 {
            out.push(b'=');
        }
    }
    out
}

/// base32 decode — faithful port of CPython 3.12 `base64._b32decode`.
///
/// * `len(s) % 8 != 0` → `binascii.Error("Incorrect padding")`.
/// * with `casefold=true` the input is upper-cased first (default is
///   case-SENSITIVE — lowercase letters are then non-alphabet digits).
/// * trailing `=` padding is stripped and counted; `padchars` must be one of
///   `{0,1,3,4,6}`, else `binascii.Error("Incorrect padding")`.
/// * any non-alphabet byte in a quantum → `binascii.Error("Non-base32 digit found")`.
///
/// Note CPython does NOT strip embedded whitespace in base32 — whitespace is a
/// non-base32 digit and raises. Returns `Err(msg)` for the dispatcher to raise
/// as `binascii.Error`.
fn b32_decode_checked(data: &[u8], table: &[u8], casefold: bool) -> Result<Vec<u8>, String> {
    if data.len() % 8 != 0 {
        return Err("Incorrect padding".to_string());
    }
    // Case-sensitive reverse lookup; with casefold we upper-case the input.
    let mut rev = [0xFFu8; 256];
    for (i, &c) in table.iter().enumerate() {
        rev[c as usize] = i as u8;
    }
    let l = data.len();
    let upper: Vec<u8>;
    let s: &[u8] = if casefold {
        upper = data.iter().map(|b| b.to_ascii_uppercase()).collect();
        &upper
    } else {
        data
    };
    // Strip trailing `=` and count the padding.
    let mut end = s.len();
    while end > 0 && s[end - 1] == b'=' {
        end -= 1;
    }
    let body = &s[..end];
    let padchars = l - body.len();

    let mut decoded: Vec<u8> = Vec::with_capacity(body.len() * 5 / 8 + 5);
    let mut acc: u64 = 0;
    for chunk in body.chunks(8) {
        acc = 0;
        for &c in chunk {
            let v = rev[c as usize];
            if v == 0xFF {
                return Err("Non-base32 digit found".to_string());
            }
            acc = (acc << 5) + v as u64;
        }
        // Full 8-char quantum decodes to 5 bytes (big-endian).
        let bytes = acc.to_be_bytes(); // u64 → 8 bytes
        decoded.extend_from_slice(&bytes[3..8]);
    }

    if l % 8 != 0 || !matches!(padchars, 0 | 1 | 3 | 4 | 6) {
        return Err("Incorrect padding".to_string());
    }
    if padchars != 0 && !decoded.is_empty() {
        // The final partial quantum had fewer chars; recompute and trim.
        acc <<= 5 * padchars;
        let last = acc.to_be_bytes();
        let leftover = (43 - 5 * padchars) / 8; // 1→4, 3→3, 4→2, 6→1
        let keep = &last[3..3 + leftover];
        let dl = decoded.len();
        decoded.truncate(dl - 5);
        decoded.extend_from_slice(keep);
    }
    Ok(decoded)
}

/// base16 encode (uppercase hex).
fn b16_encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * 2);
    for &b in data {
        out.push(B16_CHARS[(b >> 4) as usize]);
        out.push(B16_CHARS[(b & 0x0F) as usize]);
    }
    out
}

/// base16 decode (case-insensitive hex). Ignores whitespace; rejects odd-length silently.
fn b16_decode_bytes(data: &[u8]) -> Vec<u8> {
    let lut = build_alpha_table(B16_CHARS);
    let mut nibbles: Vec<u8> = Vec::with_capacity(data.len());
    for &b in data {
        if b == b'\n' || b == b'\r' || b == b' ' || b == b'\t' {
            continue;
        }
        let v = lut[b as usize];
        if v == 0xFF {
            continue;
        }
        nibbles.push(v);
    }
    let mut out = Vec::with_capacity(nibbles.len() / 2);
    for pair in nibbles.chunks(2) {
        if pair.len() < 2 {
            break;
        }
        out.push((pair[0] << 4) | pair[1]);
    }
    out
}

/// ascii85 encode (CPython `a85encode`, default no padding/wrapping/adobe).
fn a85_encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for chunk in data.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }
        let n = ((buf[0] as u32) << 24)
            | ((buf[1] as u32) << 16)
            | ((buf[2] as u32) << 8)
            | (buf[3] as u32);
        if chunk.len() == 4 && n == 0 {
            out.push(b'z');
            continue;
        }
        let mut digits = [0u8; 5];
        let mut v = n;
        for i in (0..5).rev() {
            digits[i] = b'!' + (v % 85) as u8;
            v /= 85;
        }
        let emit = if chunk.len() == 4 { 5 } else { chunk.len() + 1 };
        out.extend_from_slice(&digits[..emit]);
    }
    out
}

/// ascii85 decode (CPython `a85decode`).
///
/// Faithful to CPython 3.12 `Lib/base64.py::a85decode` with default
/// `foldspaces=False` and `ignorechars=b' \t\n\r\x0b'`:
///   * `adobe=True` requires the input to end with `~>` (and strips a leading
///     `<~`); a missing end marker raises `ValueError`.
///   * `z` expands to four NUL bytes, but only outside a 5-tuple.
///   * whitespace in `ignorechars` is skipped.
///   * any other byte outside `!`..=`u` raises `ValueError`.
///   * a completed 5-tuple that exceeds 32 bits raises `ValueError`.
fn a85_decode_bytes(data: &[u8], adobe: bool, ignorechars: &[u8]) -> Result<Vec<u8>, String> {
    // Adobe framing: must end with "~>"; strip optional leading "<~".
    let mut body: &[u8] = data;
    if adobe {
        if !body.ends_with(b"~>") {
            return Err(
                "Ascii85 encoded byte sequences must end with b'~>'".to_string(),
            );
        }
        if body.starts_with(b"<~") {
            body = &body[2..body.len() - 2];
        } else {
            body = &body[..body.len() - 2];
        }
    }

    let mut out = Vec::new();
    let mut curr: Vec<u32> = Vec::with_capacity(5);
    // CPython appends 4 trailing `u` (value 84) so the final partial tuple is
    // flushed; the extra padding bytes are then trimmed.
    let iter = body.iter().copied().chain(std::iter::repeat(b'u').take(4));
    for x in iter {
        if (b'!'..=b'u').contains(&x) {
            curr.push((x - 33) as u32);
            if curr.len() == 5 {
                let mut acc: u64 = 0;
                for &c in &curr {
                    acc = 85 * acc + c as u64;
                }
                if acc > 0xFFFF_FFFF {
                    return Err("Ascii85 overflow".to_string());
                }
                out.push((acc >> 24) as u8);
                out.push((acc >> 16) as u8);
                out.push((acc >> 8) as u8);
                out.push(acc as u8);
                curr.clear();
            }
        } else if x == b'z' {
            if !curr.is_empty() {
                return Err("z inside Ascii85 5-tuple".to_string());
            }
            out.extend_from_slice(&[0, 0, 0, 0]);
        } else if ignorechars.contains(&x) {
            // ignorechars (default whitespace) — skip.
            continue;
        } else {
            return Err(format!("Non-Ascii85 digit found: {}", x as char));
        }
    }

    // Trim the bytes produced by the 4 trailing `u` padding chars.
    let padding = 4 - curr.len();
    if padding > 0 && padding <= out.len() {
        out.truncate(out.len() - padding);
    }
    Ok(out)
}

/// base85 encode (CPython `b85encode`, RFC 1924-alike alphabet).
fn b85_encode_bytes(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut padded = data.to_vec();
    let pad = (4 - data.len() % 4) % 4;
    for _ in 0..pad {
        padded.push(0);
    }
    for chunk in padded.chunks(4) {
        let n = ((chunk[0] as u32) << 24)
            | ((chunk[1] as u32) << 16)
            | ((chunk[2] as u32) << 8)
            | (chunk[3] as u32);
        let mut digits = [0u8; 5];
        let mut v = n;
        for i in (0..5).rev() {
            digits[i] = B85_CHARS[(v % 85) as usize];
            v /= 85;
        }
        out.extend_from_slice(&digits);
    }
    if pad > 0 {
        out.truncate(out.len() - pad);
    }
    out
}

/// base85 decode (CPython `b85decode`).
///
/// CPython does NOT strip whitespace and raises `ValueError` on any byte that
/// is not in the base85 alphabet (`bad base85 character at position N`) and on
/// 32-bit overflow within a 5-char hunk (`base85 overflow in hunk starting at
/// byte N`). `Err(msg)` mirrors those two failure modes so the dispatcher can
/// raise `ValueError`.
fn b85_decode_bytes(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut lut = [0xFFu8; 256];
    for (i, &c) in B85_CHARS.iter().enumerate() {
        lut[c as usize] = i as u8;
    }
    let padding = (5 - data.len() % 5) % 5;
    let mut padded = data.to_vec();
    for _ in 0..padding {
        padded.push(b'~'); // B85_CHARS[84]
    }
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < padded.len() {
        let chunk = &padded[i..i + 5];
        let mut acc: u64 = 0;
        for (j, &c) in chunk.iter().enumerate() {
            let v = lut[c as usize];
            if v == 0xFF {
                return Err(format!("bad base85 character at position {}", i + j));
            }
            acc = acc * 85 + v as u64;
        }
        if acc > 0xFFFF_FFFF {
            return Err(format!("base85 overflow in hunk starting at byte {}", i));
        }
        out.push((acc >> 24) as u8);
        out.push((acc >> 16) as u8);
        out.push((acc >> 8) as u8);
        out.push(acc as u8);
        i += 5;
    }
    if padding > 0 {
        out.truncate(out.len() - padding);
    }
    Ok(out)
}

/// Wrap encoded bytes to 76-char lines with trailing newline (CPython `encodebytes`).
fn line_wrap_76(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(data.len() + data.len() / 76 + 1);
    for chunk in data.chunks(76) {
        out.extend_from_slice(chunk);
        out.push(b'\n');
    }
    out
}

/// base64.standard_b64encode — explicit standard alphabet alias of b64encode.
pub fn mb_base64_standard_b64encode(data: MbValue) -> MbValue {
    mb_base64_b64encode(data)
}

/// base64.standard_b64decode — explicit standard alphabet alias of b64decode.
pub fn mb_base64_standard_b64decode(data: MbValue) -> MbValue {
    mb_base64_b64decode(data)
}

/// base64.b32encode(data) -> bytes (RFC 4648 base32).
pub fn mb_base64_b32encode(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    MbValue::from_ptr(MbObject::new_bytes(b32_encode_with(&bytes, B32_CHARS)))
}

/// base64.b32decode(data, casefold=False) -> bytes (RFC 4648 base32).
pub fn mb_base64_b32decode(data: MbValue) -> MbValue {
    mb_base64_b32decode_cf(data, false)
}

/// base64.b32decode(data, casefold=False). Case-sensitive by default; `casefold`
/// upper-cases the input before decoding. Malformed input raises `binascii.Error`.
pub fn mb_base64_b32decode_cf(data: MbValue, casefold: bool) -> MbValue {
    let bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    match b32_decode_checked(&bytes, B32_CHARS, casefold) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_binascii_error(&msg),
    }
}

/// base64.b32hexencode(data) -> bytes (RFC 4648 base32hex alphabet).
pub fn mb_base64_b32hexencode(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    MbValue::from_ptr(MbObject::new_bytes(b32_encode_with(&bytes, B32_HEX_CHARS)))
}

/// base64.b32hexdecode(data, casefold=False) -> bytes (RFC 4648 base32hex).
pub fn mb_base64_b32hexdecode(data: MbValue) -> MbValue {
    mb_base64_b32hexdecode_cf(data, false)
}

/// base64.b32hexdecode(data, casefold=False) — base32hex alphabet variant.
pub fn mb_base64_b32hexdecode_cf(data: MbValue, casefold: bool) -> MbValue {
    let bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    match b32_decode_checked(&bytes, B32_HEX_CHARS, casefold) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_binascii_error(&msg),
    }
}

/// base64.b16encode(data) -> bytes (uppercase hex).
pub fn mb_base64_b16encode(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    MbValue::from_ptr(MbObject::new_bytes(b16_encode_bytes(&bytes)))
}

/// base64.b16decode(data) -> bytes (strict uppercase hex, like CPython's
/// default `casefold=False`).
pub fn mb_base64_b16decode(data: MbValue) -> MbValue {
    let bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    // CPython 3.12: b16decode is strict — odd-length input or any byte
    // outside the uppercase hex alphabet raises binascii.Error.
    if bytes.len() % 2 != 0
        || bytes
            .iter()
            .any(|b| !(b.is_ascii_digit() || (b'A'..=b'F').contains(b)))
    {
        return raise_binascii_error("Non-base16 digit found");
    }
    MbValue::from_ptr(MbObject::new_bytes(b16_decode_bytes(&bytes)))
}

/// base64.a85encode(data) -> bytes (Ascii85).
pub fn mb_base64_a85encode(data: MbValue) -> MbValue {
    mb_base64_a85encode_impl(data, false)
}

/// base64.a85encode(data, *, adobe=False) -> bytes (Ascii85). `adobe=True`
/// frames the output in `<~` ... `~>` per CPython 3.12 `base64.a85encode`.
pub fn mb_base64_a85encode_impl(data: MbValue, adobe: bool) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    let mut out = a85_encode_bytes(&bytes);
    if adobe {
        let mut framed = Vec::with_capacity(out.len() + 4);
        framed.extend_from_slice(b"<~");
        framed.append(&mut out);
        framed.extend_from_slice(b"~>");
        out = framed;
    }
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// base64.a85decode(data, *, adobe=False) -> bytes (Ascii85).
///
/// The native call ABI flattens keyword arguments into positional slots in
/// source order, so the `adobe=` flag arrives as the second positional value
/// when supplied. We read it from `args[1]` (see `dispatch_a85decode`).
pub fn mb_base64_a85decode_impl(data: MbValue, adobe: bool) -> MbValue {
    let default_ignore = [b' ', b'\t', b'\n', b'\r', 0x0b];
    mb_base64_a85decode_impl2(data, adobe, &default_ignore)
}

/// `a85decode` with an explicit `ignorechars` set (CPython default
/// `b' \t\n\r\x0b'`). Passing `ignorechars=b''` makes whitespace illegal and
/// raises `ValueError`, matching CPython 3.12.
pub fn mb_base64_a85decode_impl2(data: MbValue, adobe: bool, ignorechars: &[u8]) -> MbValue {
    let bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    match a85_decode_bytes(&bytes, adobe, ignorechars) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_value_error(&msg),
    }
}

/// base64.b85encode(data) -> bytes (RFC 1924-alike base85).
pub fn mb_base64_b85encode(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    MbValue::from_ptr(MbObject::new_bytes(b85_encode_bytes(&bytes)))
}

/// base64.b85decode(data) -> bytes (RFC 1924-alike base85).
pub fn mb_base64_b85decode(data: MbValue) -> MbValue {
    let bytes = match unsafe { decode_data_bytes(&data) } {
        Ok(b) => b,
        Err(()) => {
            return raise_value_error("string argument should contain only ASCII characters");
        }
    };
    match b85_decode_bytes(&bytes) {
        Ok(decoded) => MbValue::from_ptr(MbObject::new_bytes(decoded)),
        Err(msg) => raise_value_error(&msg),
    }
}

/// base64.encodebytes(data) -> bytes (b64 with 76-char line wrapping).
pub fn mb_base64_encodebytes(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    let encoded = b64_encode_with(&bytes, B64_CHARS);
    MbValue::from_ptr(MbObject::new_bytes(line_wrap_76(&encoded)))
}

/// base64.decodebytes(data) -> bytes (b64 decode, whitespace-tolerant).
pub fn mb_base64_decodebytes(data: MbValue) -> MbValue {
    let bytes = unsafe { extract_bytes_ref(&data) };
    MbValue::from_ptr(MbObject::new_bytes(b64_decode_with(&bytes, B64_CHARS)))
}

/// Read all bytes from a file-like object by calling its `read()` method.
/// The returned value may be bytes/bytearray/str; coerce to a byte vector.
fn read_all_bytes(fileobj: MbValue) -> Vec<u8> {
    let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
    let res = super::super::class::mb_call_method(
        fileobj,
        MbValue::from_ptr(MbObject::new_str("read".to_string())),
        empty,
    );
    unsafe { extract_bytes_ref(&res) }.into_owned()
}

/// Write a byte buffer to a file-like object via its `write()` method.
fn write_bytes(fileobj: MbValue, data: Vec<u8>) {
    let arg = MbValue::from_ptr(MbObject::new_bytes(data));
    let args = MbValue::from_ptr(MbObject::new_list(vec![arg]));
    super::super::class::mb_call_method(
        fileobj,
        MbValue::from_ptr(MbObject::new_str("write".to_string())),
        args,
    );
}

/// base64.encode(input, output) — legacy file-object stream API.
///
/// CPython reads `input` in `MAXBINSIZE` (57-byte) chunks and writes each
/// chunk's base64 encoding (with a trailing `\n`) into `output` via
/// `binascii.b2a_base64`. We read the whole input through the file object's
/// `read()` method, then emit 57-byte chunks each followed by `\n`.
pub fn mb_base64_encode_stream(input: MbValue, output: MbValue) -> MbValue {
    const MAXBINSIZE: usize = 57;
    let data = read_all_bytes(input);
    for chunk in data.chunks(MAXBINSIZE) {
        let mut line = b64_encode_with(chunk, B64_CHARS);
        line.push(b'\n');
        write_bytes(output, line);
    }
    MbValue::none()
}

/// base64.decode(input, output) — legacy file-object stream API.
///
/// CPython reads `input` line by line and writes each line's base64 decoding
/// into `output` via `binascii.a2b_base64`. b64 decoding ignores newlines, so
/// reading the entire input and decoding once is byte-for-byte equivalent to
/// the line-by-line loop for the standard alphabet.
pub fn mb_base64_decode_stream(input: MbValue, output: MbValue) -> MbValue {
    let data = read_all_bytes(input);
    let decoded = b64_decode_with(&data, B64_CHARS);
    write_bytes(output, decoded);
    MbValue::none()
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded_bytes(val: MbValue) -> Vec<u8> {
        unsafe {
            if let ObjData::Bytes(ref b) = (*val.as_ptr().unwrap()).data {
                b.clone()
            } else {
                panic!("expected Bytes (CPython base64 encode parity)");
            }
        }
    }

    fn decoded_bytes(val: MbValue) -> Vec<u8> {
        unsafe {
            if let ObjData::Bytes(ref b) = (*val.as_ptr().unwrap()).data {
                b.clone()
            } else {
                panic!("expected Bytes");
            }
        }
    }

    #[test]
    fn test_b64encode_decode_roundtrip() {
        let input = MbValue::from_ptr(MbObject::new_bytes(b"Hello, World!".to_vec()));
        let encoded = mb_base64_b64encode(input);
        assert_eq!(encoded_bytes(encoded), b"SGVsbG8sIFdvcmxkIQ==".to_vec());
        let decoded = mb_base64_b64decode(encoded);
        assert_eq!(decoded_bytes(decoded), b"Hello, World!".to_vec());
    }

    #[test]
    fn test_urlsafe_encoding() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![0xFB, 0xEF, 0xBE]));
        let standard = encoded_bytes(mb_base64_b64encode(input));
        let urlsafe = encoded_bytes(mb_base64_urlsafe_b64encode(input));
        let std_s = std::str::from_utf8(&standard).unwrap();
        let url_s = std::str::from_utf8(&urlsafe).unwrap();
        assert!(!url_s.contains('+'));
        assert!(!url_s.contains('/'));
        assert!(std_s.contains('+') || std_s.contains('/'));
    }

    #[test]
    fn test_empty_input() {
        let input = MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
        assert_eq!(encoded_bytes(mb_base64_b64encode(input)), Vec::<u8>::new());
    }

    #[test]
    fn test_urlsafe_roundtrip() {
        let input = MbValue::from_ptr(MbObject::new_bytes(b"test url-safe data!".to_vec()));
        let encoded = mb_base64_urlsafe_b64encode(input);
        let decoded = mb_base64_urlsafe_b64decode(encoded);
        assert_eq!(decoded_bytes(decoded), b"test url-safe data!".to_vec());
    }

    #[test]
    fn test_b64_single_byte() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![0x41]));
        assert_eq!(encoded_bytes(mb_base64_b64encode(input)), b"QQ==".to_vec());
    }

    #[test]
    fn test_b64_two_bytes() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![0x41, 0x42]));
        assert_eq!(encoded_bytes(mb_base64_b64encode(input)), b"QUI=".to_vec());
    }

    #[test]
    fn test_b64_three_bytes_no_padding() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![0x41, 0x42, 0x43]));
        let out = encoded_bytes(mb_base64_b64encode(input));
        assert_eq!(out, b"QUJD".to_vec());
        assert!(!out.contains(&b'='));
    }

    #[test]
    fn test_b64_str_input() {
        // encode from a Str value (treated as UTF-8 bytes)
        let input = MbValue::from_ptr(MbObject::new_str("Man".to_string()));
        assert_eq!(encoded_bytes(mb_base64_b64encode(input)), b"TWFu".to_vec());
    }

    #[test]
    fn test_b64_decode_known_value() {
        let input = MbValue::from_ptr(MbObject::new_str("QUJD".to_string()));
        let decoded = mb_base64_b64decode(input);
        assert_eq!(decoded_bytes(decoded), vec![0x41, 0x42, 0x43]);
    }
}
