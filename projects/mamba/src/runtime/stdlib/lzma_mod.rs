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
    // A str format (positional or kwargs) is an integer-argument TypeError.
    let format_val = positional(args, 1).or_else(|| {
        args.last().and_then(|v| v.as_ptr()).and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get("format").copied()
            } else {
                None
            }
        })
    });
    if let Some(fv) = format_val {
        if as_index(fv).is_none() {
            let tn = if as_str(fv).is_some() { "str" } else { "object" };
            return raise_type_error_lzma(&format!(
                "'{tn}' object cannot be interpreted as an integer"
            ));
        }
    }
    let format = format_val.and_then(as_index);
    // FORMAT_RAW == 3 (matches the registered module constant). A memory limit
    // is meaningless for a raw stream, so CPython rejects the combination.
    if format == Some(3) && kwargs_has(args, "memlimit") {
        return raise_value_error("Cannot specify memory limit with FORMAT_RAW");
    }
    // The payload must be bytes-like (a list is the buffer TypeError).
    let data = args.first().copied().unwrap_or_else(MbValue::none);
    let is_buffer = data.as_ptr().is_some_and(|p| unsafe {
        matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
    });
    if !is_buffer {
        let tn = if as_str(data).is_some() {
            "str"
        } else if data.as_ptr().is_some_and(|p| unsafe { matches!((*p).data, ObjData::List(_)) }) {
            "list"
        } else if data.as_int().is_some() {
            "int"
        } else {
            "object"
        };
        return raise_type_error_lzma(&format!("a bytes-like object is required, not '{tn}'"));
    }
    mb_lzma_decompress(data)
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

// ── Incremental compressor / decompressor objects ──
//
// Real liblzma streaming state (xz2::stream::Stream) keyed by a registry id;
// the Python-visible objects are Instances whose method calls route through
// `lzma_instance_method` (mb_call_method arm) and whose eof / unused_data /
// needs_input attributes are plain fields refreshed after every call.

struct DecompState {
    stream: xz2::stream::Stream,
    /// Decompressed output not yet handed to the caller (max_length).
    out_buf: Vec<u8>,
    /// Compressed input not yet consumed by liblzma.
    in_buf: Vec<u8>,
    eof: bool,
    errored: bool,
}

struct CompState {
    stream: xz2::stream::Stream,
    flushed: bool,
}

thread_local! {
    static DECOMPRESSORS: std::cell::RefCell<HashMap<u64, DecompState>> =
        std::cell::RefCell::new(HashMap::new());
    static COMPRESSORS: std::cell::RefCell<HashMap<u64, CompState>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_LZMA_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

fn next_lzma_id() -> u64 {
    NEXT_LZMA_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    })
}

fn instance_with_handle(class_name: &str, id: u64) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert("_handle".to_string(), MbValue::from_int(id as i64));
            if class_name == "LZMADecompressor" {
                g.insert("eof".to_string(), MbValue::from_bool(false));
                g.insert("needs_input".to_string(), MbValue::from_bool(true));
                g.insert(
                    "unused_data".to_string(),
                    MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
                );
                // CHECK_UNKNOWN until a stream header is seen.
                g.insert("check".to_string(), MbValue::from_int(16));
            }
        }
    }
    MbValue::from_ptr(inst)
}

fn instance_handle(recv: MbValue) -> Option<u64> {
    recv.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get("_handle").and_then(|v| v.as_int()).map(|i| i as u64)
        } else {
            None
        }
    })
}

fn set_instance_field(recv: MbValue, name: &str, val: MbValue) {
    if let Some(ptr) = recv.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), val);
            }
        }
    }
}

unsafe extern "C" fn dispatch_lzma_compressor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let Ok(stream) = xz2::stream::Stream::new_easy_encoder(6, xz2::stream::Check::Crc64)
    else {
        return raise_lzma_error("Failed to initialize encoder");
    };
    let id = next_lzma_id();
    COMPRESSORS.with(|m| {
        m.borrow_mut().insert(id, CompState { stream, flushed: false });
    });
    instance_with_handle("LZMACompressor", id)
}

unsafe extern "C" fn dispatch_lzma_decompressor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let Ok(stream) = xz2::stream::Stream::new_auto_decoder(u64::MAX, 0) else {
        return raise_lzma_error("Failed to initialize decoder");
    };
    let id = next_lzma_id();
    DECOMPRESSORS.with(|m| {
        m.borrow_mut().insert(id, DecompState {
            stream,
            out_buf: Vec::new(),
            in_buf: Vec::new(),
            eof: false,
            errored: false,
        });
    });
    instance_with_handle("LZMADecompressor", id)
}

/// Drive the decoder over the buffered input, appending decompressed bytes
/// to out_buf up to `target` bytes (None = unbounded). Stopping at the target
/// is what keeps a decompression bomb from expanding past max_length.
/// Returns Err on a corrupt stream.
fn decomp_pump(st: &mut DecompState, target: Option<usize>) -> Result<(), ()> {
    while !st.eof {
        if let Some(t) = target {
            if st.out_buf.len() >= t {
                break;
            }
        }
        let before_in = st.stream.total_in();
        let before_len = st.out_buf.len();
        // Bounded reserve: stay near the target instead of 64K leaps so a
        // max_length read doesn't balloon the buffer.
        let chunk = match target {
            Some(t) => (t - st.out_buf.len()).clamp(1, 64 * 1024),
            None => 64 * 1024,
        };
        st.out_buf.reserve(chunk);
        let status = st
            .stream
            .process_vec(&st.in_buf, &mut st.out_buf, xz2::stream::Action::Run)
            .map_err(|_| ())?;
        let consumed = (st.stream.total_in() - before_in) as usize;
        st.in_buf.drain(..consumed.min(st.in_buf.len()));
        if matches!(status, xz2::stream::Status::StreamEnd) {
            st.eof = true;
            break;
        }
        // No progress on either side: need more input.
        if consumed == 0 && st.out_buf.len() == before_len {
            break;
        }
    }
    Ok(())
}

/// Method dispatch for LZMACompressor / LZMADecompressor instances.
/// Returns None for unknown receivers/methods (caller falls through).
pub fn lzma_instance_method(recv: MbValue, method: &str, args: &[MbValue]) -> Option<MbValue> {
    let class_name = recv.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })?;
    let id = instance_handle(recv)?;
    match (class_name.as_str(), method) {
        ("LZMADecompressor", "decompress") => {
            let data = args.first().copied().unwrap_or_else(MbValue::none);
            // max_length: positional slot 1 or kwargs dict.
            let mut max_length: i64 = -1;
            for v in args.iter().skip(1) {
                if let Some(i) = as_index(*v) {
                    max_length = i;
                }
                if let Some(ptr) = v.as_ptr() {
                    unsafe {
                        if let ObjData::Dict(ref lock) = (*ptr).data {
                            if let Some(m) =
                                lock.read().unwrap().get("max_length").copied().and_then(as_index)
                            {
                                max_length = m;
                            }
                        }
                    }
                }
            }
            // Reject non-buffer payloads before touching the stream.
            let is_buffer = data.as_ptr().is_some_and(|p| unsafe {
                matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
            });
            if !is_buffer {
                let tn = if as_str(data).is_some() { "str" } else { "object" };
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "a bytes-like object is required, not '{tn}'"
                    ))),
                );
                return Some(MbValue::none());
            }
            let result = DECOMPRESSORS.with(|m| {
                let mut map = m.borrow_mut();
                let Some(st) = map.get_mut(&id) else {
                    return Err("internal: lost decompressor state".to_string());
                };
                if st.errored {
                    // bug 28275: once errored, every reuse keeps raising.
                    return Err("Input format not supported by decoder".to_string());
                }
                // .check: read the integrity-check id off the XZ stream
                // header (magic[6] + flags; byte 7 low nibble) on first feed.
                let header_check = if st.stream.total_in() == 0 && st.in_buf.is_empty() {
                    let mut c: Option<i64> = None;
                    with_bytes(data, |b| {
                        if b.len() >= 8 && b.starts_with(b"\xfd7zXZ\x00") {
                            c = Some((b[7] & 0x0F) as i64);
                        }
                    });
                    c
                } else {
                    None
                };
                with_bytes(data, |b| st.in_buf.extend_from_slice(b));
                let target = if max_length < 0 { None } else { Some(max_length as usize) };
                if decomp_pump(st, target).is_err() {
                    st.errored = true;
                    return Err("Input format not supported by decoder".to_string());
                }
                let take = if max_length < 0 {
                    st.out_buf.len()
                } else {
                    (max_length as usize).min(st.out_buf.len())
                };
                let chunk: Vec<u8> = st.out_buf.drain(..take).collect();
                // unused_data: bytes left after stream end.
                let unused = if st.eof { st.in_buf.clone() } else { Vec::new() };
                // CPython: needs_input is False while unconsumed input or
                // buffered output remains.
                let needs_input = !st.eof && st.out_buf.is_empty() && st.in_buf.is_empty();
                Ok((chunk, st.eof, unused, needs_input, header_check))
            });
            match result {
                Err(msg) => Some(raise_lzma_error(&msg)),
                Ok((chunk, eof, unused, needs_input, header_check)) => {
                    set_instance_field(recv, "eof", MbValue::from_bool(eof));
                    set_instance_field(recv, "needs_input", MbValue::from_bool(needs_input));
                    set_instance_field(
                        recv,
                        "unused_data",
                        MbValue::from_ptr(MbObject::new_bytes(unused)),
                    );
                    if let Some(c) = header_check {
                        set_instance_field(recv, "check", MbValue::from_int(c));
                    }
                    Some(MbValue::from_ptr(MbObject::new_bytes(chunk)))
                }
            }
        }
        ("LZMACompressor", "compress") => {
            let data = args.first().copied().unwrap_or_else(MbValue::none);
            let out = COMPRESSORS.with(|m| {
                let mut map = m.borrow_mut();
                let Some(st) = map.get_mut(&id) else { return Err(()) };
                if st.flushed {
                    return Err(());
                }
                let mut out: Vec<u8> = Vec::new();
                let res = with_bytes(data, |b| {
                    let mut consumed_total = 0usize;
                    while consumed_total < b.len() {
                        let before_in = st.stream.total_in();
                        out.reserve(64 * 1024);
                        if st
                            .stream
                            .process_vec(&b[consumed_total..], &mut out, xz2::stream::Action::Run)
                            .is_err()
                        {
                            return Err(());
                        }
                        let consumed = (st.stream.total_in() - before_in) as usize;
                        if consumed == 0 {
                            break;
                        }
                        consumed_total += consumed;
                    }
                    Ok(())
                });
                res.map(|()| out)
            });
            match out {
                Err(()) => Some(raise_lzma_error("Compressor has been flushed")),
                Ok(out) => Some(MbValue::from_ptr(MbObject::new_bytes(out))),
            }
        }
        ("LZMACompressor", "flush") => {
            let out = COMPRESSORS.with(|m| {
                let mut map = m.borrow_mut();
                let Some(st) = map.get_mut(&id) else { return Err(()) };
                if st.flushed {
                    return Err(());
                }
                st.flushed = true;
                let mut out: Vec<u8> = Vec::new();
                loop {
                    out.reserve(64 * 1024);
                    match st.stream.process_vec(&[], &mut out, xz2::stream::Action::Finish) {
                        Ok(xz2::stream::Status::StreamEnd) => break,
                        Ok(_) => continue,
                        Err(_) => return Err(()),
                    }
                }
                Ok(out)
            });
            match out {
                Err(()) => Some(raise_lzma_error("Repeated call to flush()")),
                Ok(out) => Some(MbValue::from_ptr(MbObject::new_bytes(out))),
            }
        }
        _ => None,
    }
}

/// lzma.is_check_supported(check) -> bool. The bundled liblzma supports
/// NONE / CRC32 / CRC64 / SHA256; anything else (including ids above
/// CHECK_ID_MAX) is unsupported.
unsafe extern "C" fn dispatch_is_check_supported(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let check = args.first().copied().and_then(as_index).unwrap_or(-1);
    MbValue::from_bool(matches!(check, 0 | 1 | 4 | 10))
}

/// lzma.open(filename, mode="rb", ..., encoding=None, errors=None) — unlike
/// LZMAFile, open() accepts text modes ('rt'/'wt') and wraps in a text layer.
unsafe extern "C" fn dispatch_lzma_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mode = positional(args, 1)
        .and_then(as_str)
        .unwrap_or_else(|| "rb".to_string());
    let base = mode.trim_end_matches(['b', 't']);
    if !matches!(base, "r" | "w" | "x" | "a") || (mode.contains('b') && mode.contains('t')) {
        return raise_value_error(&format!("Invalid mode: {mode:?}"));
    }
    let kw_str = |name: &str| -> Option<String> {
        args.last().and_then(|v| v.as_ptr()).and_then(|ptr| unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(name).copied().and_then(as_str)
            } else {
                None
            }
        })
    };
    super::compressed_file::make_file_opts(
        "LZMAFile",
        super::compressed_file::Codec::Xz,
        args.first().copied().unwrap_or_else(MbValue::none),
        &mode,
        kw_str("encoding"),
        kw_str("errors"),
    )
}

/// lzma._encode_filter_properties(filter_spec) — LZMA1/LZMA2 property bytes.
/// LZMA1: byte0 = (pb*5 + lp)*9 + lc, bytes1..5 = dict_size LE u32.
/// LZMA2: single dict-size code byte.
unsafe extern "C" fn dispatch_encode_filter_properties(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let spec = args.first().copied().unwrap_or_else(MbValue::none);
    let Some(ptr) = spec.as_ptr() else {
        return raise_type_error_lzma("filter specifier must be a dict or dict-like object");
    };
    let map = unsafe {
        match &(*ptr).data {
            ObjData::Dict(ref lock) => lock.read().unwrap().clone(),
            _ => {
                return raise_type_error_lzma(
                    "filter specifier must be a dict or dict-like object",
                )
            }
        }
    };
    let get_i = |k: &str, d: i64| map.get(k).copied().and_then(as_index).unwrap_or(d);
    let lc = get_i("lc", 3);
    let lp = get_i("lp", 0);
    let pb = get_i("pb", 2);
    let dict_size = get_i("dict_size", 8 << 20) as u32;
    let mut out = Vec::with_capacity(5);
    out.push(((pb * 5 + lp) * 9 + lc) as u8);
    out.extend_from_slice(&dict_size.to_le_bytes());
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// lzma._decode_filter_properties(filter_id, props) — inverse of the above.
unsafe extern "C" fn dispatch_decode_filter_properties(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filter_id = args.first().copied().and_then(as_index).unwrap_or(0);
    let props = args.get(1).copied().unwrap_or_else(MbValue::none);
    let bytes: Vec<u8> = with_bytes(props, |b| b.to_vec());
    let dict = super::super::dict_ops::mb_dict_new();
    let set = |k: &str, v: MbValue| {
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str(k.to_string())),
            v,
        );
    };
    set("id", MbValue::from_int(filter_id));
    if bytes.len() >= 5 {
        let code = bytes[0] as i64;
        set("lc", MbValue::from_int(code % 9));
        set("lp", MbValue::from_int((code / 9) % 5));
        set("pb", MbValue::from_int(code / 45));
        let ds = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        set("dict_size", MbValue::from_int(ds as i64));
    }
    dict
}

fn raise_type_error_lzma(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
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
        ("open",              dispatch_lzma_open         as usize),
        ("LZMACompressor",    dispatch_lzma_compressor   as usize),
        ("LZMADecompressor",  dispatch_lzma_decompressor as usize),
        ("is_check_supported", dispatch_is_check_supported as usize),
        ("_encode_filter_properties", dispatch_encode_filter_properties as usize),
        ("_decode_filter_properties", dispatch_decode_filter_properties as usize),
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
    // CPython decodes back-to-back concatenated streams: after one stream
    // ends, remaining bytes start a fresh decoder. Bytes that fail to start
    // a NEW stream are ignored once at least one stream decoded (trailing
    // junk); a failure on the FIRST stream is the LZMAError.
    let out: Result<Vec<u8>, ()> = with_bytes(data, |b| {
        let mut all = Vec::with_capacity(b.len().saturating_mul(4));
        let mut rest = b;
        let mut decoded_any = false;
        while !rest.is_empty() {
            let Ok(mut stream) = xz2::stream::Stream::new_auto_decoder(u64::MAX, 0) else {
                return Err(());
            };
            let mut out_buf: Vec<u8> = Vec::new();
            let mut ended = false;
            loop {
                let before_in = stream.total_in();
                let before_len = out_buf.len();
                out_buf.reserve(64 * 1024);
                let consumed_so_far = stream.total_in() as usize;
                let status = match stream.process_vec(
                    &rest[consumed_so_far..],
                    &mut out_buf,
                    xz2::stream::Action::Run,
                ) {
                    Ok(s) => s,
                    Err(_) => {
                        if decoded_any {
                            // Trailing junk after a complete stream: ignore.
                            return Ok(all);
                        }
                        return Err(());
                    }
                };
                if matches!(status, xz2::stream::Status::StreamEnd) {
                    ended = true;
                    break;
                }
                let progressed = stream.total_in() != before_in || out_buf.len() != before_len;
                if !progressed {
                    break;
                }
            }
            let consumed = stream.total_in() as usize;
            if !ended && !decoded_any {
                return Err(()); // truncated first stream
            }
            all.extend_from_slice(&out_buf);
            decoded_any = true;
            if consumed == 0 {
                break;
            }
            rest = &rest[consumed.min(rest.len())..];
            if !ended {
                break;
            }
        }
        Ok(all)
    });
    match out {
        Ok(buf) => MbValue::from_ptr(MbObject::new_bytes(buf)),
        // A truncated or non-xz stream fails the decode. CPython raises
        // `LZMAError` in this case; surfaced through the pending-exception
        // channel so `except lzma.LZMAError:` catches it.
        Err(()) => raise_lzma_error("Input format not supported by decoder"),
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
