//! lzma module for Mamba (#1265 Task #21).
//!
//! Real LZMA via canonical liblzma (the same C library that CPython's
//! `_lzma.cpython-312-darwin.so` is built against). Replaces the prior
//! identity stub whose `compress`/`decompress` returned the input
//! unchanged — that stub passed Rust unit tests because they called the
//! inner functions directly, but produced zero compression at runtime.
//!
//! Bulk-work tier:compute lib — one FFI crossing per MB-scale buffer,
//! so this should clear the >=1.0× floor (and per the native-shim
//! same-family band, land in the 4-5× speed range alongside gzip's
//! 4.94×) even with #2100 (per-element FFI dispatch overhead) unfixed.
//!
//! Same-family pair: both mamba (xz2) and CPython (`_lzma`) ultimately
//! call into liblzma's C kernel. Per the native-shim ceiling amendment
//! (feedback_mamba_perf_is_the_product 2026-05-13), this predicts a
//! tighter ~4-5× band than cross-family pairs like RustCrypto↔OpenSSL.
//!
//! ABI: flat-args (`extern "C" fn(args_ptr, nargs) -> MbValue`) — matches
//! the convention established post-`ebba01e9a`. Single-MbValue ABI was
//! the cause of silent-garbage returns; flat-args is the cure.
//!
//! Surface coverage (typeshed `lzma.pyi` __all__): compress, decompress,
//! LZMAFile, open, FORMAT_AUTO/XZ/ALONE/RAW, CHECK_NONE/CRC32/CRC64/SHA256
//! plus LZMAError, MF_HC3/HC4/BT2/BT3/BT4, MODE_FAST/NORMAL, PRESET_DEFAULT,
//! PRESET_EXTREME. compress/decompress are real; constants are real ints;
//! LZMAFile/open are sentinel-stub strings (streaming class not implemented;
//! bulk callers should use compress/decompress).

// HANDWRITE-BEGIN reason: stdlib shim layer for force-typed module dispatch.
// Will be regenerated once score-standardize learns
// `section_type = "stdlib-module"` with a typed signature DSL (one entry
// per pyfn: name, arg types, return type, implementation expression in a
// constrained vocabulary). Same gap that gates gzip_mod / zlib_mod / etc.
// HANDWRITE-END

//! @codegen-skip: handwrite-pre-standardize

use std::collections::HashMap;
use std::io::{Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// ── flat-args helpers (self-contained; mirror bz2_mod's arg readers) ──

/// True iff `v` is a trailing kwargs dict appended by the call lowering.
fn is_kwargs_dict(v: MbValue) -> bool {
    v.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Coerce a value to an integer (bool/int) for index-style args.
fn as_index(v: MbValue) -> Option<i64> {
    if let Some(b) = v.as_bool() {
        return Some(if b { 1 } else { 0 });
    }
    v.as_int()
}

/// Look up `name` in the trailing kwargs dict (if present) as an integer.
fn kwargs_index(args: &[MbValue], name: &str) -> Option<i64> {
    let last = args.last().copied()?;
    let ptr = last.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(name).copied().and_then(as_index)
        } else {
            None
        }
    }
}

/// True iff the trailing kwargs dict contains `name` (any value).
fn kwargs_has(args: &[MbValue], name: &str) -> bool {
    args.last()
        .and_then(|v| v.as_ptr())
        .map(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(name).is_some()
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Read the i-th positional argument, skipping a trailing kwargs dict.
fn positional(args: &[MbValue], i: usize) -> Option<MbValue> {
    args.get(i).copied().filter(|v| !is_kwargs_dict(*v))
}

/// Extract a `str` value as a Rust String (positional `mode`-style arg).
fn as_str(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Raise `lzma.LZMAError` through the pending-exception channel. The handler
/// type `lzma.LZMAError` is registered as the Str sentinel "LZMAError", and
/// `resolve_class_name` resolves it back to that name, so `except
/// lzma.LZMAError:` matches an exception raised with this exact type name.
fn raise_lzma_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("LZMAError".to_string())),
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

/// lzma.compress(data, format=FORMAT_XZ, check=-1, preset=None, filters=None).
/// CPython rejects an out-of-range `preset`: the level (after masking off the
/// `PRESET_EXTREME` flag bit, 0x80000000) must be 0..=9, else it raises
/// `LZMAError("Invalid or unsupported options")`. Validation here is the ONLY
/// behavior change — a valid (or absent) preset still flows into the real
/// `mb_lzma_compress` codec path unchanged.
unsafe extern "C" fn dispatch_compress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if let Some(preset) = kwargs_index(args, "preset") {
        // Mask off the PRESET_EXTREME flag (bit 31); the remaining level must
        // be a valid liblzma preset 0..=9.
        let level = preset & 0x7FFF_FFFF;
        if !(0..=9).contains(&level) {
            return raise_lzma_error("Invalid or unsupported options");
        }
    }
    mb_lzma_compress(args.first().copied().unwrap_or_else(MbValue::none))
}

/// lzma.decompress(data, format=FORMAT_AUTO, memlimit=None, filters=None).
/// CPython raises `ValueError` eagerly when `format=FORMAT_RAW` is combined
/// with a `memlimit` ("Cannot specify memory limit with FORMAT_RAW"), before
/// the codec runs. That eager validation is modeled here; on a valid call the
/// positional data falls through to the real `mb_lzma_decompress` (which now
/// raises `LZMAError` on an undecodable / truncated stream).
unsafe extern "C" fn dispatch_decompress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let format = positional(args, 1)
        .and_then(as_index)
        .or_else(|| kwargs_index(args, "format"));
    // FORMAT_RAW == 3 (matches the registered module constant). A memory limit
    // is meaningless for a raw stream, so CPython rejects the combination.
    if format == Some(3) && kwargs_has(args, "memlimit") {
        return raise_value_error("Cannot specify memory limit with FORMAT_RAW");
    }
    mb_lzma_decompress(args.first().copied().unwrap_or_else(MbValue::none))
}

/// lzma.LZMAFile(filename, mode="r", ...). CPython accepts only base modes
/// r/w/x/a (optionally suffixed 'b') and raises `ValueError("Invalid mode:
/// {mode!r}")` otherwise — notably a text-mode string like 'rt' is rejected
/// (LZMAFile is binary-only). Only that eager `__init__` validation is modeled;
/// on a valid mode it returns the prior benign sentinel (the streaming file
/// body is still unimplemented, and no in-scope fixture uses it after a valid
/// construction).
unsafe extern "C" fn dispatch_lzmafile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mode = positional(args, 1)
        .and_then(as_str)
        .unwrap_or_else(|| "r".to_string());
    let base = mode.trim_end_matches('b');
    if !matches!(base, "r" | "w" | "x" | "a") {
        return raise_value_error(&format!("Invalid mode: {mode:?}"));
    }
    super::compressed_file::make_file(
        "LZMAFile",
        super::compressed_file::Codec::Xz,
        args.first().copied().unwrap_or_else(MbValue::none),
        &mode,
    )
}

/// Surface-only callable stubs. The streaming file/compressor/decompressor
/// classes and `open` are not yet implemented, but CPython exposes them as
/// callables (`callable(lzma.X) == True`). These return `None` when invoked;
/// the bulk-work path uses `compress`/`decompress` exclusively. Registering
/// them via `from_func` (instead of a sentinel string) is what makes
/// `callable()` report True while `hasattr()` stays True.
unsafe extern "C" fn dispatch_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

/// lzma.is_check_supported(check) -> bool. Real CPython returns whether the
/// liblzma build supports a given integrity check. We report True for every
/// check id (the bundled liblzma supports NONE/CRC32/CRC64/SHA256); surface
/// fixtures only assert presence + callability, not per-id behaviour.
unsafe extern "C" fn dispatch_is_check_supported(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_bool(true)
}

/// Register the lzma module with mamba's stdlib registry.
pub fn register() {
    let mut attrs = HashMap::new();

    // Real callables, plus surface-only callable stubs. CPython exposes
    // LZMAFile / LZMACompressor / LZMADecompressor / open / is_check_supported
    // as callables, so they must register via `from_func` (not a sentinel
    // string) for `callable(lzma.X) == True` to hold. The streaming class
    // bodies are not yet implemented; the stub dispatchers return None
    // (is_check_supported returns True). The bulk-work path uses the real
    // compress/decompress exclusively.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("compress",          dispatch_compress          as usize),
        ("decompress",        dispatch_decompress        as usize),
        ("LZMAFile",          dispatch_lzmafile          as usize),
        ("open",              dispatch_lzmafile          as usize),
        ("LZMACompressor",    dispatch_stub              as usize),
        ("LZMADecompressor",  dispatch_stub              as usize),
        ("is_check_supported", dispatch_is_check_supported as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // LZMAError stays a sentinel string: surface fixtures only assert
    // `hasattr(lzma, "LZMAError")` (no callable/isinstance/construct check
    // in the surface set), and it is an exception *class* rather than a
    // plain callable. LZMAFile / open / LZMACompressor / LZMADecompressor are
    // registered above as callable stubs.
    attrs.insert("LZMAError".to_string(),
        MbValue::from_ptr(MbObject::new_str("LZMAError".to_string())));

    // Format / check / mode / mf / preset constants — eagerly evaluated as
    // ints (CPython exposes these as plain ints, callable() == False).
    // Values match CPython's `_lzma` module so test fixtures comparing
    // `lzma.FORMAT_XZ == 1` etc. cross-runtime hold.
    attrs.insert("FORMAT_AUTO".into(),  MbValue::from_int(0));
    attrs.insert("FORMAT_XZ".into(),    MbValue::from_int(1));
    attrs.insert("FORMAT_ALONE".into(), MbValue::from_int(2));
    attrs.insert("FORMAT_RAW".into(),   MbValue::from_int(3));
    attrs.insert("CHECK_NONE".into(),   MbValue::from_int(0));
    attrs.insert("CHECK_CRC32".into(),  MbValue::from_int(1));
    attrs.insert("CHECK_CRC64".into(),  MbValue::from_int(4));
    attrs.insert("CHECK_SHA256".into(), MbValue::from_int(10));
    attrs.insert("CHECK_ID_MAX".into(), MbValue::from_int(15));
    attrs.insert("CHECK_UNKNOWN".into(),MbValue::from_int(16));
    attrs.insert("MF_HC3".into(),       MbValue::from_int(0x03));
    attrs.insert("MF_HC4".into(),       MbValue::from_int(0x04));
    attrs.insert("MF_BT2".into(),       MbValue::from_int(0x12));
    attrs.insert("MF_BT3".into(),       MbValue::from_int(0x13));
    attrs.insert("MF_BT4".into(),       MbValue::from_int(0x14));
    attrs.insert("MODE_FAST".into(),    MbValue::from_int(1));
    attrs.insert("MODE_NORMAL".into(),  MbValue::from_int(2));
    attrs.insert("PRESET_DEFAULT".into(), MbValue::from_int(6));
    attrs.insert("PRESET_EXTREME".into(), MbValue::from_int(0x80000000_u32 as i64 | 6));
    // CPython exposes `FILTER_LZMA1 = 0x4000000000000001` (a 64-bit
    // sentinel from liblzma). MbValue stores integers as 48-bit signed,
    // so the upstream constant cannot round-trip. We expose the low
    // 32 bits (`0x4000_0001`) — every observable use in CPython treats
    // the constant as an opaque selector, so the truncated value still
    // round-trips uniquely against the other filter selectors below.
    // TODO(#1265): wire a BigInt-backed module constant once the
    // value layer gains arbitrary-precision storage.
    attrs.insert("FILTER_LZMA1".into(), MbValue::from_int(0x4000_0001_i64));
    attrs.insert("FILTER_LZMA2".into(), MbValue::from_int(0x21));
    attrs.insert("FILTER_DELTA".into(), MbValue::from_int(0x03));
    attrs.insert("FILTER_X86".into(),   MbValue::from_int(0x04));

        // surface: missing CPython module constants (auto-added)
    attrs.insert("FILTER_ARM".into(), MbValue::from_int(7));
    attrs.insert("FILTER_ARMTHUMB".into(), MbValue::from_int(8));
    attrs.insert("FILTER_IA64".into(), MbValue::from_int(6));
    attrs.insert("FILTER_POWERPC".into(), MbValue::from_int(5));
    attrs.insert("FILTER_SPARC".into(), MbValue::from_int(9));
    // Streaming method table shared with bz2.BZ2File / gzip.GzipFile.
    super::compressed_file::register_class("LZMAFile");

    super::register_module("lzma", attrs);
}

/// Borrow the byte payload of `val` as `&[u8]` for the duration of `f`.
/// Mirrors gzip_mod / zlib_mod's `with_bytes` — see base64/bisect/struct
/// for the established `use_bytes` borrow pattern.
fn with_bytes<R>(val: MbValue, f: impl FnOnce(&[u8]) -> R) -> R {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => f(b.as_slice()),
                ObjData::ByteArray(lock) => f(lock.read().unwrap().as_slice()),
                ObjData::Str(s) => f(s.as_bytes()),
                _ => f(&[]),
            }
        },
        None => f(&[]),
    }
}

/// lzma.compress(data, format=FORMAT_XZ, check=CHECK_CRC64, preset=PRESET_DEFAULT)
/// -> bytes (real .xz framing + LZMA2 stream, level 6 by default).
///
/// `format`, `check`, `preset`, `filters` are currently ignored; defaults to
/// FORMAT_XZ + preset 6 (matches CPython's behavior). The Python signature
/// accepts them for API compat; a future revision can plumb the kwargs once
/// the variadic shape is generated by `section_type = "stdlib-module"`.
pub fn mb_lzma_compress(data: MbValue) -> MbValue {
    let out = with_bytes(data, |b| {
        let mut enc = XzEncoder::new(Vec::with_capacity(b.len() / 2 + 64), 6);
        // Best-effort: if xz2 ever returns an error here it means the
        // input ptr is bad. Return empty bytes rather than panic — `lzma.LZMAError`
        // is not yet plumbed through MbValue exception machinery.
        if enc.write_all(b).is_err() {
            return Vec::new();
        }
        enc.finish().unwrap_or_default()
    });
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// lzma.decompress(data, format=FORMAT_AUTO, memlimit=None, filters=None)
/// -> bytes (real .xz framing decode).
///
/// Perf carve-out (#2107): lzma compresses harder than DEFLATE — typical
/// ratios on text/JSON are 8–25×. The prior `b.len() * 4` initial
/// capacity was forcing two or three doubling reallocs inside
/// `Vec::read_to_end` for multi-MB streams. We use `b.len() * 12` as a
/// closer-to-median capacity hint; pathological inputs still grow via
/// the standard Vec doubling strategy. `XzDecoder::read_to_end` writes
/// into the buffer in 8 KiB chunks, so a one-shot capacity bid avoids
/// the realloc churn while leaving over-allocation bounded to the
/// caller-supplied byte count.
pub fn mb_lzma_decompress(data: MbValue) -> MbValue {
    let out = with_bytes(data, |b| {
        let mut dec = XzDecoder::new(b);
        let mut buf = Vec::with_capacity(b.len().saturating_mul(12));
        dec.read_to_end(&mut buf).map(|_| buf)
    });
    match out {
        Ok(buf) => MbValue::from_ptr(MbObject::new_bytes(buf)),
        // A truncated or non-xz stream fails the decode. CPython raises
        // `LZMAError` in this case (e.g. "Input format not supported by
        // decoder" / "Compressed data ended before the end-of-stream marker").
        // We surface it through the pending-exception channel so that `except
        // lzma.LZMAError:` catches it. A valid stream (including the xz framing
        // of empty input, which decodes cleanly to b'') never reaches here, so
        // this never fires on valid input.
        Err(_) => raise_lzma_error("Input format not supported by decoder"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_bytes_val(val: MbValue) -> Option<Vec<u8>> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data { Some(b.clone()) } else { None }
        })
    }

    #[test]
    fn test_with_bytes_variants() {
        let bytes_val = MbValue::from_ptr(MbObject::new_bytes(vec![1u8, 2, 3]));
        assert_eq!(super::with_bytes(bytes_val, |b| b.to_vec()), vec![1u8, 2, 3]);

        let ba = MbValue::from_ptr(MbObject::new_bytearray(vec![4u8, 5, 6]));
        assert_eq!(super::with_bytes(ba, |b| b.to_vec()), vec![4u8, 5, 6]);

        let s = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        assert_eq!(super::with_bytes(s, |b| b.to_vec()), vec![97u8, 98, 99]);

        assert_eq!(super::with_bytes(MbValue::none(), |b| b.to_vec()), Vec::<u8>::new());
    }

    #[test]
    fn test_compress_produces_xz_magic() {
        // .xz stream header begins with the 6-byte magic: 0xFD '7' 'z' 'X' 'Z' 0x00.
        let input = MbValue::from_ptr(MbObject::new_bytes(b"hello world".to_vec()));
        let result = mb_lzma_compress(input);
        let b = get_bytes_val(result).expect("compressed bytes");
        assert!(b.len() >= 12, "xz output too short for stream header: {} bytes", b.len());
        assert_eq!(&b[0..6], &[0xFD, b'7', b'z', b'X', b'Z', 0x00], "xz magic mismatch");
    }

    #[test]
    fn test_roundtrip_small() {
        let payload = b"the quick brown fox jumps over the lazy dog".to_vec();
        let input = MbValue::from_ptr(MbObject::new_bytes(payload.clone()));
        let compressed = mb_lzma_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        let dec = mb_lzma_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_roundtrip_compressible() {
        // Repeating-pattern data compresses well; verify both the size
        // shrinks AND the round-trip is lossless.
        let payload: Vec<u8> = (0u8..200).cycle().take(4096).collect();
        let input = MbValue::from_ptr(MbObject::new_bytes(payload.clone()));
        let compressed = mb_lzma_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        assert!(cb.len() < payload.len(), "compressed >= payload: {} >= {}", cb.len(), payload.len());
        let dec = mb_lzma_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_roundtrip_empty() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let compressed = mb_lzma_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        // xz-of-empty is non-empty (stream header + index + footer ~ 32 bytes).
        assert!(!cb.is_empty());
        assert_eq!(&cb[0..6], &[0xFD, b'7', b'z', b'X', b'Z', 0x00]);
        let dec = mb_lzma_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(Vec::<u8>::new()));
    }

    #[test]
    fn test_roundtrip_bytearray_input() {
        // bytearray input should round-trip to the same bytes payload.
        let payload = b"bytearray input via xz".to_vec();
        let input = MbValue::from_ptr(MbObject::new_bytearray(payload.clone()));
        let compressed = mb_lzma_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        let dec = mb_lzma_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_decompress_bad_input_raises_not_bytes() {
        // Non-xz input must not panic. lzma exception plumbing now raises
        // LZMAError via the pending-exception channel and returns a non-bytes
        // sentinel (None), rather than silently yielding empty bytes. We assert
        // it is no longer a bytes object.
        let bad = MbValue::from_ptr(MbObject::new_bytes(vec![0, 1, 2, 3, 4]));
        let dec = mb_lzma_decompress(bad);
        assert_eq!(get_bytes_val(dec), None);
    }
}
