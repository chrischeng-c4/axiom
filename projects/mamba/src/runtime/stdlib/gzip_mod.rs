//! gzip module for Mamba (#1265 Task #18).
//!
//! Real gzip (DEFLATE + 10-byte header + 8-byte trailer with CRC32 +
//! original-length) via `flate2::write::GzEncoder` / `flate2::read::GzDecoder`,
//! replacing the prior identity stub that registered names as strings
//! and never actually compressed/decompressed anything.
//!
//! Bulk-work tier:compute lib — one FFI crossing per MB-scale buffer,
//! so this is one of the few stdlib libs that should clear the >=10x
//! speed gate even with #2100 (per-element FFI dispatch overhead)
//! unfixed. Sibling of zlib_mod.rs which lands the DEFLATE side; gzip
//! adds the framing.
//!
//! ABI: flat-args (`extern "C" fn(args_ptr, nargs) -> MbValue`) — matches
//! the convention established post-`ebba01e9a` for stdlib shims. The
//! single-MbValue ABI variant identified in
//! `project_mamba_runtime_correctness_gaps_2026_05_13` memory was the
//! cause of silent-garbage returns; flat-args is the cure.
//!
//! Surface coverage (typeshed `gzip.pyi` __all__):
//!   BadGzipFile, GzipFile, open, compress, decompress
//! All five are wired here — compress/decompress as real callables,
//! and BadGzipFile/GzipFile/open as sentinel-stub attributes so that
//! `gzip.GzipFile` etc. surface-resolve (the streaming file class itself
//! is not implemented; bulk callers should use compress/decompress).

// HANDWRITE-BEGIN reason: stdlib shim layer for force-typed module dispatch.
// Will be regenerated once score-standardize learns
// `section_type = "stdlib-module"` with a typed signature DSL (one entry
// per pyfn: name, arg types, return type, implementation expression in a
// constrained vocabulary). Same gap that gates zlib_mod / base64_mod / etc.
// HANDWRITE-END

//! @codegen-skip: handwrite-pre-standardize

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::io::{Read, Write};

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

/// `gzip.compress(data, compresslevel=9)` — flat-args dispatcher.
///
/// `compresslevel` may arrive positionally (arg 1) or as a `compresslevel=`
/// kwarg (the call lowering appends a trailing `{name: value}` dict for keyword
/// arguments — same shape zlib_mod's dispatchers consume). CPython validates the
/// level against zlib's accepted range (`-1..=9`, where -1 is the default
/// sentinel) BEFORE compressing and raises `zlib.error('Bad compression level')`
/// for anything outside it; we mirror that exactly.
unsafe extern "C" fn dispatch_compress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let data = args.first().copied().unwrap_or_else(MbValue::none);
    let level = args
        .get(1)
        .copied()
        .filter(|v| !is_kwargs_dict(*v))
        .and_then(as_index)
        .or_else(|| kwargs_index(args, "compresslevel"))
        .unwrap_or(9);
    if !(-1..=9).contains(&level) {
        return raise_zlib_error("Bad compression level");
    }
    mb_gzip_compress_level(data, level)
}

/// `gzip.decompress(data)` — flat-args dispatcher. Only the first positional
/// argument is consumed; classification of truncation / bad-header errors lives
/// in `mb_gzip_decompress`.
unsafe extern "C" fn dispatch_decompress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_gzip_decompress(args.first().copied().unwrap_or_else(MbValue::none))
}

/// gzip.open(filename, mode="rb", ...) / gzip.GzipFile(filename, mode="rb",
/// *, fileobj=None) — real streaming files over the shared compressed-file
/// layer. The keyword form `GzipFile(fileobj=...)` lowers to a trailing
/// kwargs dict; resolve the source and mode from it when present.
unsafe extern "C" fn dispatch_gzip_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mut source = args.first().copied().unwrap_or_else(MbValue::none);
    let mut mode = "r".to_string();
    if let Some(m) = args.get(1).filter(|v| !is_kwargs_dict(**v)).and_then(|v| {
        v.as_ptr().and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }) {
        mode = m;
    }
    let mut encoding = None;
    let mut errors = None;
    if let Some(last) = args.last() {
        if let Some(p) = last.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*p).data {
                    use super::super::dict_ops::DictKey;
                    let map = lock.read().unwrap();
                    let get_str = |key: &str| -> Option<String> {
                        map.get(&DictKey::Str(key.to_string())).and_then(|v| {
                            v.as_ptr().and_then(|sp| {
                                if let ObjData::Str(ref s) = (*sp).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                        })
                    };
                    if let Some(fo) = map.get(&DictKey::Str("fileobj".to_string())) {
                        source = *fo;
                    }
                    if let Some(m) = get_str("mode") {
                        mode = m;
                    }
                    encoding = get_str("encoding");
                    errors = get_str("errors");
                }
            }
        }
    }
    super::compressed_file::make_file_opts(
        "GzipFile",
        super::compressed_file::Codec::Gzip,
        source,
        &mode,
        encoding,
        errors,
    )
}

/// Register the gzip module with mamba's stdlib registry.
pub fn register() {
    let mut attrs = HashMap::new();

    // Real callables.
    let dispatchers: Vec<(&str, usize)> = vec![
        ("compress", dispatch_compress as usize),
        ("decompress", dispatch_decompress as usize),
        // `open` and `GzipFile` construct real streaming files through the
        // shared compressed-file layer (Codec::Gzip).
        ("open", dispatch_gzip_ctor as usize),
        ("GzipFile", dispatch_gzip_ctor as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // `gzip.GzipFile` and `gzip.open` are registered above as `from_func`
    // stubs (callable surface). The bulk-work bench path uses
    // compress/decompress exclusively; the streaming file-object plumbing
    // lands behind these stubs later.

    // `gzip.BadGzipFile` is a real exception class — a subclass of OSError on
    // CPython 3.12 (Lib/gzip.py: `class BadGzipFile(OSError)`). Registering it
    // in the class registry with base `OSError` makes
    // `issubclass(gzip.BadGzipFile, OSError)` and `except gzip.BadGzipFile`
    // (matched against the OSError hierarchy) resolve correctly. The module
    // attribute stays a `Str("BadGzipFile")`: `resolve_class_name` maps that
    // string straight to the registered class, so the surface value and the
    // registry entry agree without needing a distinct class-object value.
    //
    // mro for the single-base case is `["BadGzipFile", "OSError", ... , "object"]`
    // regardless of whether `OSError` itself is registered yet at module-init
    // time, so `issubclass(.., OSError)` holds even if the builtin-exception
    // hierarchy registers after this stdlib module.
    super::super::class::mb_class_register(
        "BadGzipFile",
        vec!["OSError".to_string()],
        HashMap::new(),
    );
    let bad_gzip_file = MbValue::from_ptr(MbObject::new_str("BadGzipFile".to_string()));
    attrs.insert("BadGzipFile".to_string(), bad_gzip_file);

    // surface: missing CPython module constants (auto-added)
    attrs.insert("FCOMMENT".into(), MbValue::from_int(16));
    attrs.insert("FEXTRA".into(), MbValue::from_int(4));
    attrs.insert("FHCRC".into(), MbValue::from_int(2));
    attrs.insert("FNAME".into(), MbValue::from_int(8));
    attrs.insert("FTEXT".into(), MbValue::from_int(1));
    attrs.insert("READ".into(), MbValue::from_int(1));
    attrs.insert("READ_BUFFER_SIZE".into(), MbValue::from_int(131072));
    attrs.insert("WRITE".into(), MbValue::from_int(2));
    // Streaming method table shared with bz2.BZ2File / lzma.LZMAFile.
    super::compressed_file::register_class("GzipFile");

    super::register_module("gzip", attrs);
}

/// Borrow the byte payload of `val` as `&[u8]` for the duration of `f`.
/// Mirrors zlib_mod's `with_bytes` — see base64/bisect/struct for the
/// established `use_bytes` borrow pattern.
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

/// True iff `v` is a trailing kwargs dict appended by the call lowering.
/// Mirrors zlib_mod's `is_kwargs_dict`.
fn is_kwargs_dict(v: MbValue) -> bool {
    v.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Resolve an `MbValue` to an `i64` (plain ints and bools pass through;
/// everything else yields `None`). gzip's `compresslevel` is always a plain
/// int in practice, so this is the int/bool subset of zlib_mod's `as_index`.
fn as_index(v: MbValue) -> Option<i64> {
    if let Some(b) = v.as_bool() {
        return Some(if b { 1 } else { 0 });
    }
    v.as_int()
}

/// Pull a named integer keyword out of a trailing kwargs dict in a variadic
/// args list. The call lowering appends a `{name: value}` dict for keyword
/// arguments (see zlib_mod / bisect_mod `dict_as_kwargs`); a trailing dict here
/// is therefore always kwargs.
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

/// Raise `zlib.error` (the exception `gzip.compress` surfaces for a bad
/// compression level, matching CPython where the zlib layer validates it).
/// `import zlib` registers the `zlib.error` class; the bare type-string matches
/// `except zlib.error` by name even before that, mirroring zlib_mod.
fn raise_zlib_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("zlib.error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `EOFError` — gzip's signal that the compressed input ended before the
/// end-of-stream marker / trailer was reached (truncated stream).
fn raise_eof_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("EOFError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Raise `gzip.BadGzipFile` — input that is not a gzip stream at all (bad magic)
/// or that uses an unknown compression method. The class is registered in
/// `register()` with base `OSError`, so `except gzip.BadGzipFile` resolves.
fn raise_bad_gzip_file(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("BadGzipFile".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// gzip.compress(data, compresslevel=9) -> bytes (real gzip framing + DEFLATE).
///
/// `compresslevel` is validated by the dispatcher before we get here; we plumb
/// the (in-range) level through to flate2. `-1` is zlib's default sentinel →
/// `Compression::default()`; `0..=9` map straight to `Compression::new`.
pub fn mb_gzip_compress_level(data: MbValue, level: i64) -> MbValue {
    let compression = if level < 0 {
        Compression::default()
    } else {
        Compression::new(level as u32)
    };
    let out = with_bytes(data, |b| {
        let mut enc = GzEncoder::new(Vec::with_capacity(b.len() / 2 + 32), compression);
        // Best-effort: if flate2 ever returns an error here it means the
        // input ptr is bad. Return empty bytes rather than panic.
        if enc.write_all(b).is_err() {
            return Vec::new();
        }
        enc.finish().unwrap_or_default()
    });
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// gzip.compress(data) at the default level — retained for the in-crate tests
/// and any internal callers that don't thread a level through.
pub fn mb_gzip_compress(data: MbValue) -> MbValue {
    mb_gzip_compress_level(data, -1)
}

/// Classify a gzip-decode failure into the exception CPython raises.
///
/// Decision tree (validated against CPython 3.12 `gzip.decompress`):
///   * empty input → handled by the caller (returns `b''`, no error here).
///   * `len < 2` or magic `b[0..2] != [0x1f, 0x8b]` → `BadGzipFile`
///     ("Not a gzipped file").
///   * magic OK and `b.len() >= 3` and `b[2] != 0x08` (compression method not
///     DEFLATE) → `BadGzipFile` ("Unknown compression method").
///   * otherwise (valid magic + CM, but the stream/header/trailer is truncated
///     so flate2 returned an error) → `EOFError`.
///
/// This never fires on a valid full stream: a valid stream always has
/// `[0x1f, 0x8b, 0x08, ...]` and decodes without error, so the caller never
/// reaches this function for it.
fn raise_gzip_decode_error(b: &[u8]) -> MbValue {
    if b.len() < 2 || b[0] != 0x1f || b[1] != 0x8b {
        return raise_bad_gzip_file("Not a gzipped file");
    }
    if b.len() >= 3 && b[2] != 0x08 {
        return raise_bad_gzip_file("Unknown compression method");
    }
    raise_eof_error("Compressed file ended before the end-of-stream marker was reached")
}

/// gzip.decompress(data) -> bytes (real gzip framing + DEFLATE).
///
/// Perf carve-out (#2107): the gzip trailer's last 4 bytes are ISIZE,
/// the uncompressed size modulo 2^32. We read it to size the output
/// buffer exactly when it fits in `usize` and is below a sanity ceiling.
/// Falling back to `b.len() * 8` for the unknown / oversize cases is
/// closer to the empirical 5–20× compression ratio than the prior
/// `b.len() * 4` under-allocation that was causing repeat `realloc`+memcpy
/// inside `Vec::read_to_end` for multi-MB streams (the dominant cost on
/// the internal-time-ratio gap reported on the dual-time harness).
pub fn mb_gzip_decompress(data: MbValue) -> MbValue {
    // `Ok(buf)` → decoded bytes; `Err(())` → flate2 reported a decode error and
    // the classifier must run (it needs the raw header bytes). Empty input is a
    // valid empty stream in CPython, so short-circuit to `b''` without decoding.
    let result: Result<Vec<u8>, ()> = with_bytes(data, |b| {
        if b.is_empty() {
            return Ok(Vec::new());
        }
        let mut dec = GzDecoder::new(b);
        let cap = decompress_capacity_hint(b);
        let mut buf = Vec::with_capacity(cap);
        if dec.read_to_end(&mut buf).is_err() {
            return Err(());
        }
        Ok(buf)
    });
    match result {
        Ok(buf) => MbValue::from_ptr(MbObject::new_bytes(buf)),
        // Re-borrow to classify (BadGzipFile vs EOFError) from the raw header.
        Err(()) => with_bytes(data, raise_gzip_decode_error),
    }
}

/// Estimate the uncompressed size of a gzip stream.
///
/// The gzip trailer encodes ISIZE (uncompressed size mod 2^32) in the
/// last 4 bytes, little-endian. We use it directly when the value is
/// plausible (non-zero, under a 256 MiB cap, fits in `usize`). For
/// anything that fails the sanity check we fall back to `b.len() * 8`
/// — a better default than the previous `* 4` for typical text/JSON
/// payloads, while remaining bounded for adversarial inputs.
fn decompress_capacity_hint(b: &[u8]) -> usize {
    const SANE_MAX: u64 = 256 * 1024 * 1024;
    if b.len() >= 4 {
        let tail = &b[b.len() - 4..];
        let isize_mod = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]) as u64;
        if isize_mod > 0 && isize_mod <= SANE_MAX {
            if let Ok(n) = usize::try_from(isize_mod) {
                return n;
            }
        }
    }
    b.len().saturating_mul(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_bytes_val(val: MbValue) -> Option<Vec<u8>> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data {
                Some(b.clone())
            } else {
                None
            }
        })
    }

    #[test]
    fn test_with_bytes_variants() {
        let bytes_val = MbValue::from_ptr(MbObject::new_bytes(vec![1u8, 2, 3]));
        assert_eq!(
            super::with_bytes(bytes_val, |b| b.to_vec()),
            vec![1u8, 2, 3]
        );

        let ba = MbValue::from_ptr(MbObject::new_bytearray(vec![4u8, 5, 6]));
        assert_eq!(super::with_bytes(ba, |b| b.to_vec()), vec![4u8, 5, 6]);

        let s = MbValue::from_ptr(MbObject::new_str("abc".to_string()));
        assert_eq!(super::with_bytes(s, |b| b.to_vec()), vec![97u8, 98, 99]);

        assert_eq!(
            super::with_bytes(MbValue::none(), |b| b.to_vec()),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn test_compress_produces_gzip_magic() {
        // gzip stream header begins with 0x1F 0x8B (gzip magic).
        let input = MbValue::from_ptr(MbObject::new_bytes(b"hello world".to_vec()));
        let result = mb_gzip_compress(input);
        let b = get_bytes_val(result).expect("compressed bytes");
        assert!(
            b.len() >= 10,
            "gzip output too short to contain header: {} bytes",
            b.len()
        );
        assert_eq!(
            b[0], 0x1F,
            "gzip magic byte 0 should be 0x1F, got {:#x}",
            b[0]
        );
        assert_eq!(
            b[1], 0x8B,
            "gzip magic byte 1 should be 0x8B, got {:#x}",
            b[1]
        );
        assert_eq!(
            b[2], 0x08,
            "gzip CM byte (compression method DEFLATE=8), got {:#x}",
            b[2]
        );
    }

    #[test]
    fn test_roundtrip_small() {
        let payload = b"the quick brown fox jumps over the lazy dog".to_vec();
        let input = MbValue::from_ptr(MbObject::new_bytes(payload.clone()));
        let compressed = mb_gzip_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        let dec = mb_gzip_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_roundtrip_compressible() {
        // Repeating-pattern data compresses well; verify both the size
        // shrinks AND the round-trip is lossless.
        let payload: Vec<u8> = (0u8..200).cycle().take(4096).collect();
        let input = MbValue::from_ptr(MbObject::new_bytes(payload.clone()));
        let compressed = mb_gzip_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        assert!(
            cb.len() < payload.len(),
            "compressed >= payload: {} >= {}",
            cb.len(),
            payload.len()
        );
        let dec = mb_gzip_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_roundtrip_empty() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let compressed = mb_gzip_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        // gzip-of-empty is non-empty (header + empty deflate block + trailer ~ 20 bytes).
        assert!(!cb.is_empty());
        assert_eq!(cb[0], 0x1F);
        assert_eq!(cb[1], 0x8B);
        let dec = mb_gzip_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(Vec::<u8>::new()));
    }

    #[test]
    fn test_roundtrip_bytearray_input() {
        // bytearray input should round-trip to the same bytes payload.
        let payload = b"bytearray input goes here".to_vec();
        let input = MbValue::from_ptr(MbObject::new_bytearray(payload.clone()));
        let compressed = mb_gzip_compress(input);
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        let dec = mb_gzip_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(dec), Some(payload));
    }

    #[test]
    fn test_decompress_capacity_hint_uses_isize() {
        // Build a real gzip stream whose ISIZE trailer is non-zero and
        // well below the 256 MiB sanity cap. The capacity hint must
        // round-trip that exact value.
        let payload: Vec<u8> = (0u8..200).cycle().take(8192).collect();
        let compressed_mb =
            mb_gzip_compress(MbValue::from_ptr(MbObject::new_bytes(payload.clone())));
        let compressed_bytes = get_bytes_val(compressed_mb).expect("compressed bytes");
        let hint = super::decompress_capacity_hint(&compressed_bytes);
        assert_eq!(
            hint,
            payload.len(),
            "ISIZE-derived hint should equal uncompressed length, got {} expected {}",
            hint,
            payload.len()
        );
    }

    #[test]
    fn test_decompress_capacity_hint_short_input_falls_back() {
        // Inputs shorter than the 4-byte trailer must fall back to the
        // b.len() * 8 default rather than panic or under-read.
        assert_eq!(super::decompress_capacity_hint(&[]), 0);
        assert_eq!(super::decompress_capacity_hint(&[1, 2, 3]), 24);
    }

    #[test]
    fn test_decompress_hot_path_regression_2107() {
        // Regression guard for #2107: a multi-KB compressible payload
        // must decompress correctly with the new capacity hint and not
        // overshoot Vec capacity in a way that breaks the round-trip.
        // We do not assert on timing here (CI is noisy); the correctness
        // gate alone catches any over-/under-allocation that would
        // truncate the output or trigger a Vec growth bug.
        let payload: Vec<u8> = (0u8..255).cycle().take(64 * 1024).collect();
        let compressed = mb_gzip_compress(MbValue::from_ptr(MbObject::new_bytes(payload.clone())));
        let cb = get_bytes_val(compressed).expect("compressed bytes");
        // ISIZE-derived hint should match payload size exactly for inputs
        // below 2^32 bytes.
        assert_eq!(super::decompress_capacity_hint(&cb), payload.len());
        let decompressed = mb_gzip_decompress(MbValue::from_ptr(MbObject::new_bytes(cb)));
        assert_eq!(get_bytes_val(decompressed), Some(payload));
    }

    #[test]
    fn test_decompress_bad_input_raises_not_bytes() {
        // Non-gzip input now raises `gzip.BadGzipFile` (the exception plumbing
        // anticipated by the prior version of this test). `mb_raise` records the
        // pending exception in thread-local state and the shim returns `none`,
        // so the call must not panic and must not yield decoded bytes.
        let bad = MbValue::from_ptr(MbObject::new_bytes(vec![0, 1, 2, 3, 4]));
        let dec = mb_gzip_decompress(bad);
        assert_eq!(get_bytes_val(dec), None);
    }

    #[test]
    fn test_decompress_empty_input_is_valid_empty() {
        // Empty input is a valid empty gzip stream in CPython (returns b''), so
        // the error classifier must NOT fire on it.
        let empty = MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
        let dec = mb_gzip_decompress(empty);
        assert_eq!(get_bytes_val(dec), Some(Vec::<u8>::new()));
    }
}
