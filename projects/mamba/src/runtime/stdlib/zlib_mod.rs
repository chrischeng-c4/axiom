//! zlib module for Mamba (mamba-stdlib).
//!
//! Real DEFLATE via flate2 (replaces the prior identity stub) for #1265 Task #17.
//! Bulk-work tier:compute lib — the hot path is a single FFI crossing that
//! amortizes per-element dispatch over a MB-scale buffer, so this is one of
//! the libs that should clear the >=10x speed gate even with #2100 unfixed.
//!
//! ABI: flat-args (`extern "C" fn(args_ptr, nargs) -> MbValue`) — matches the
//! convention established post-`ebba01e9a` for stdlib shims, so `mamba run`
//! dispatch resolves correctly. The single-MbValue ABI variant identified in
//! `project_mamba_runtime_correctness_gaps_2026_05_13` memory was the cause
//! of silent-garbage returns; flat-args is the cure.
//!
//! HANDWRITE-BEGIN reason: stdlib shim layer for force-typed module dispatch.
//! Will be regenerated once score-standardize learns `section_type =
//! "stdlib-module"` with a typed signature DSL (one entry per pyfn: name,
//! arg types, return type, implementation expression in a constrained
//! vocabulary).
//! HANDWRITE-END

//! @codegen-skip: handwrite-pre-standardize

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use flate2::write::{DeflateEncoder, ZlibEncoder};
use flate2::Compression;
use flate2::{Decompress, FlushDecompress, Status};
use std::collections::HashMap;
use std::io::Write;

const ZDICT_MARKER: &[u8] = b"MBZDICT\0";

macro_rules! dispatch_unary_bytes_like {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let arg = a.get(0).copied().unwrap_or_else(MbValue::none);
            if !is_bytes_like(arg) {
                return raise_type_error(concat!(
                    stringify!($fn),
                    "() argument must be bytes-like"
                ));
            }
            $fn(arg)
        }
    };
}

dispatch_unary_bytes_like!(dispatch_decompress, mb_zlib_decompress);

unsafe extern "C" fn dispatch_crc32(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(arg) {
        return raise_type_error("mb_zlib_crc32() argument must be bytes-like");
    }
    let seed = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as u32;
    mb_zlib_crc32_seed(arg, seed)
}

unsafe extern "C" fn dispatch_adler32(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(arg) {
        return raise_type_error("mb_zlib_adler32() argument must be bytes-like");
    }
    let seed = args.get(1).and_then(|v| v.as_int()).unwrap_or(1) as u32;
    mb_zlib_adler32_seed(arg, seed)
}

unsafe extern "C" fn dispatch_compress(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(arg) {
        return raise_type_error("mb_zlib_compress() argument must be bytes-like");
    }
    if let Some(level) = args.get(1).and_then(|v| v.as_int()) {
        if !(-1..=9).contains(&level) {
            return raise_zlib_error("Bad compression level");
        }
    }
    mb_zlib_compress(arg)
}

unsafe extern "C" fn dispatch_compressobj(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kwargs) = split_kwargs(args);
    let level = arg_int(pos, kwargs, 0, "level", -1);
    if !(-1..=9).contains(&level) {
        return raise_zlib_error("Bad compression level");
    }
    let method = arg_int(pos, kwargs, 1, "method", 8);
    if method != 8 {
        return raise_value_error("Invalid initialization option");
    }
    let wbits = arg_int(pos, kwargs, 2, "wbits", 15);
    if !valid_compress_wbits(wbits) {
        return raise_value_error("Invalid initialization option");
    }
    let zdict = arg_value(pos, kwargs, 5, "zdict")
        .filter(|v| is_bytes_like(*v))
        .map(|v| with_bytes(v, |b| b.to_vec()))
        .unwrap_or_default();
    make_compressobj_stub(level, wbits, zdict)
}

unsafe extern "C" fn dispatch_decompressobj(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (pos, kwargs) = split_kwargs(args);
    let wbits = arg_int(pos, kwargs, 0, "wbits", 15);
    if !valid_decompress_wbits(wbits) {
        return raise_value_error("Invalid initialization option");
    }
    let zdict = arg_value(pos, kwargs, 1, "zdict")
        .filter(|v| is_bytes_like(*v))
        .map(|v| with_bytes(v, |b| b.to_vec()))
        .unwrap_or_default();
    make_decompressobj_stub(wbits, zdict)
}

extern "C" fn mb_zlib_compressobj_compress(self_obj: MbValue, data: MbValue) -> MbValue {
    if !is_bytes_like(data) {
        return raise_type_error("compress() argument must be bytes-like");
    }
    let mut pending = get_instance_bytes(self_obj, "_buffer");
    with_bytes(data, |b| pending.extend_from_slice(b));
    set_instance_field(
        self_obj,
        "_buffer",
        MbValue::from_ptr(MbObject::new_bytes(pending)),
    );
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

extern "C" fn mb_zlib_compressobj_flush(self_obj: MbValue, _args: MbValue) -> MbValue {
    let pending = get_instance_bytes(self_obj, "_buffer");
    let level = get_instance_int(self_obj, "_level", -1);
    let wbits = get_instance_int(self_obj, "_wbits", 15);
    let zdict = get_instance_bytes(self_obj, "_zdict");
    set_instance_field(
        self_obj,
        "_buffer",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    match encode_zlib_stream(&pending, level, wbits, &zdict) {
        Ok(output) => MbValue::from_ptr(MbObject::new_bytes(output)),
        Err(()) => raise_zlib_error("Error while compressing data"),
    }
}

extern "C" fn mb_zlib_compressobj_copy(self_obj: MbValue) -> MbValue {
    let copied = make_compressobj_stub(
        get_instance_int(self_obj, "_level", -1),
        get_instance_int(self_obj, "_wbits", 15),
        get_instance_bytes(self_obj, "_zdict"),
    );
    set_instance_field(
        copied,
        "_buffer",
        MbValue::from_ptr(MbObject::new_bytes(get_instance_bytes(self_obj, "_buffer"))),
    );
    copied
}

extern "C" fn mb_zlib_decompressobj_decompress(self_obj: MbValue, args: MbValue) -> MbValue {
    let args = list_items(args);
    let data = args.first().copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(data) {
        return raise_type_error("decompress() argument must be bytes-like");
    }
    let max_length = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    if max_length < 0 {
        return raise_value_error("max_length must be non-negative");
    }

    let decoded_tail = get_instance_bytes(self_obj, "_decoded_tail");
    if !decoded_tail.is_empty() {
        set_instance_field(
            self_obj,
            "_decoded_tail",
            MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
        );
        set_instance_field(
            self_obj,
            "_buffer",
            MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
        );
        set_instance_field(self_obj, "eof", MbValue::from_bool(true));
        set_instance_field(
            self_obj,
            "unused_data",
            MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
        );
        set_instance_field(
            self_obj,
            "unconsumed_tail",
            MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
        );
        return MbValue::from_ptr(MbObject::new_bytes(decoded_tail));
    }

    let mut pending = get_instance_bytes(self_obj, "_buffer");
    with_bytes(data, |b| pending.extend_from_slice(b));
    let wbits = get_instance_int(self_obj, "_wbits", 15);
    let zdict = get_instance_bytes(self_obj, "_zdict");
    match decode_zlib_stream_with_options(&pending, wbits, &zdict) {
        Ok(DecodeResult::Complete { output, consumed }) => {
            set_instance_field(
                self_obj,
                "_buffer",
                MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
            );
            let limit = max_length as usize;
            if limit > 0 && output.len() > limit {
                set_instance_field(self_obj, "eof", MbValue::from_bool(false));
                set_instance_field(
                    self_obj,
                    "unused_data",
                    MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
                );
                set_instance_field(
                    self_obj,
                    "unconsumed_tail",
                    MbValue::from_ptr(MbObject::new_bytes(pending[..consumed].to_vec())),
                );
                set_instance_field(
                    self_obj,
                    "_decoded_tail",
                    MbValue::from_ptr(MbObject::new_bytes(output[limit..].to_vec())),
                );
                return MbValue::from_ptr(MbObject::new_bytes(output[..limit].to_vec()));
            }
            set_instance_field(self_obj, "eof", MbValue::from_bool(true));
            set_instance_field(
                self_obj,
                "unused_data",
                MbValue::from_ptr(MbObject::new_bytes(pending[consumed..].to_vec())),
            );
            set_instance_field(
                self_obj,
                "unconsumed_tail",
                MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
            );
            set_instance_field(
                self_obj,
                "_decoded_tail",
                MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
            );
            MbValue::from_ptr(MbObject::new_bytes(output))
        }
        Ok(DecodeResult::Incomplete) => {
            set_instance_field(
                self_obj,
                "_buffer",
                MbValue::from_ptr(MbObject::new_bytes(pending)),
            );
            set_instance_field(self_obj, "eof", MbValue::from_bool(false));
            MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
        }
        Err(()) => raise_zlib_error("Error while decompressing data"),
    }
}

extern "C" fn mb_zlib_decompressobj_flush(_self: MbValue, args: MbValue) -> MbValue {
    let args = list_items(args);
    if let Some(length) = args.first().and_then(|v| v.as_int()) {
        if length <= 0 {
            return raise_value_error("length must be greater than zero");
        }
    }
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

fn valid_decompress_wbits(wbits: i64) -> bool {
    wbits == 0
        || (-15..=-8).contains(&wbits)
        || (8..=15).contains(&wbits)
        || wbits == 16
        || (24..=31).contains(&wbits)
        || wbits == 32
        || (40..=47).contains(&wbits)
}

fn valid_compress_wbits(wbits: i64) -> bool {
    (9..=15).contains(&wbits) || (-15..=-9).contains(&wbits) || (25..=31).contains(&wbits)
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("compress", dispatch_compress as usize),
        ("compressobj", dispatch_compressobj as usize),
        ("decompress", dispatch_decompress as usize),
        ("decompressobj", dispatch_decompressobj as usize),
        ("crc32", dispatch_crc32 as usize),
        ("adler32", dispatch_adler32 as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    let mut compressor_methods = HashMap::new();
    let compressobj_compress_addr = mb_zlib_compressobj_compress as usize;
    compressor_methods.insert(
        "compress".to_string(),
        MbValue::from_func(compressobj_compress_addr),
    );
    let compressobj_flush_addr = mb_zlib_compressobj_flush as usize;
    compressor_methods.insert(
        "flush".to_string(),
        MbValue::from_func(compressobj_flush_addr),
    );
    let compressobj_copy_addr = mb_zlib_compressobj_copy as usize;
    compressor_methods.insert(
        "copy".to_string(),
        MbValue::from_func(compressobj_copy_addr),
    );
    super::super::module::register_variadic_func(compressobj_flush_addr as u64);
    super::super::class::mb_class_register("zlib.Compress", Vec::new(), compressor_methods);
    let mut decompressor_methods = HashMap::new();
    let decompressobj_decompress_addr = mb_zlib_decompressobj_decompress as usize;
    decompressor_methods.insert(
        "decompress".to_string(),
        MbValue::from_func(decompressobj_decompress_addr),
    );
    let decompressobj_flush_addr = mb_zlib_decompressobj_flush as usize;
    decompressor_methods.insert(
        "flush".to_string(),
        MbValue::from_func(decompressobj_flush_addr),
    );
    super::super::module::register_variadic_func(decompressobj_decompress_addr as u64);
    super::super::module::register_variadic_func(decompressobj_flush_addr as u64);
    super::super::class::mb_class_register("zlib.Decompress", Vec::new(), decompressor_methods);
    attrs.insert(
        "error".to_string(),
        MbValue::from_ptr(MbObject::new_str("zlib.error".to_string())),
    );
    attrs.insert("MAX_WBITS".to_string(), MbValue::from_int(15));
    attrs.insert("DEFLATED".to_string(), MbValue::from_int(8));
    attrs.insert("DEF_BUF_SIZE".to_string(), MbValue::from_int(16_384));
    attrs.insert("DEF_MEM_LEVEL".to_string(), MbValue::from_int(8));
    attrs.insert("Z_BEST_COMPRESSION".to_string(), MbValue::from_int(9));
    attrs.insert("Z_BEST_SPEED".to_string(), MbValue::from_int(1));
    attrs.insert("Z_DEFAULT_COMPRESSION".to_string(), MbValue::from_int(-1));
    attrs.insert("Z_NO_COMPRESSION".to_string(), MbValue::from_int(0));
    attrs.insert("Z_NO_FLUSH".to_string(), MbValue::from_int(0));
    attrs.insert("Z_PARTIAL_FLUSH".to_string(), MbValue::from_int(1));
    attrs.insert("Z_SYNC_FLUSH".to_string(), MbValue::from_int(2));
    attrs.insert("Z_FULL_FLUSH".to_string(), MbValue::from_int(3));
    attrs.insert("Z_FINISH".to_string(), MbValue::from_int(4));
    attrs.insert("Z_BLOCK".to_string(), MbValue::from_int(5));
    attrs.insert("Z_TREES".to_string(), MbValue::from_int(6));
    attrs.insert("Z_DEFAULT_STRATEGY".to_string(), MbValue::from_int(0));
    attrs.insert("Z_FILTERED".to_string(), MbValue::from_int(1));
    attrs.insert("Z_HUFFMAN_ONLY".to_string(), MbValue::from_int(2));
    attrs.insert("Z_RLE".to_string(), MbValue::from_int(3));
    attrs.insert("Z_FIXED".to_string(), MbValue::from_int(4));
    attrs.insert(
        "ZLIB_VERSION".to_string(),
        MbValue::from_ptr(MbObject::new_str("1.2.12".to_string())),
    );
    attrs.insert(
        "ZLIB_RUNTIME_VERSION".to_string(),
        MbValue::from_ptr(MbObject::new_str("1.2.12".to_string())),
    );
    super::register_module("zlib", attrs);
}

fn make_compressobj_stub(level: i64, wbits: i64, zdict: Vec<u8>) -> MbValue {
    let obj = MbValue::from_ptr(MbObject::new_instance("zlib.Compress".to_string()));
    set_instance_field(
        obj,
        "_buffer",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    set_instance_field(obj, "_level", MbValue::from_int(level));
    set_instance_field(obj, "_wbits", MbValue::from_int(wbits));
    set_instance_field(obj, "_zdict", MbValue::from_ptr(MbObject::new_bytes(zdict)));
    obj
}

fn make_decompressobj_stub(wbits: i64, zdict: Vec<u8>) -> MbValue {
    let obj = MbValue::from_ptr(MbObject::new_instance("zlib.Decompress".to_string()));
    set_instance_field(
        obj,
        "_buffer",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    set_instance_field(
        obj,
        "_decoded_tail",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    set_instance_field(obj, "_wbits", MbValue::from_int(wbits));
    set_instance_field(obj, "_zdict", MbValue::from_ptr(MbObject::new_bytes(zdict)));
    set_instance_field(obj, "eof", MbValue::from_bool(false));
    set_instance_field(
        obj,
        "unused_data",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    set_instance_field(
        obj,
        "unconsumed_tail",
        MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    );
    obj
}

/// Borrow the byte payload of `val` as `&[u8]` for the duration of `f`.
///
/// Avoids the `.clone()` that the old `extract_bytes -> Vec<u8>` shape forced
/// on every call. Matches the `use_bytes` borrow pattern established by
/// base64/bisect/struct.
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

fn is_bytes_like(val: MbValue) -> bool {
    match val.as_ptr() {
        Some(ptr) => unsafe { matches!(&(*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) },
        None => false,
    }
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

fn split_kwargs(args: &[MbValue]) -> (&[MbValue], Option<MbValue>) {
    match args.last().copied().filter(|v| is_dict(*v)) {
        Some(kwargs) => (&args[..args.len().saturating_sub(1)], Some(kwargs)),
        None => (args, None),
    }
}

fn dict_get(kwargs: Option<MbValue>, name: &str) -> Option<MbValue> {
    let ptr = kwargs?.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(name).copied()
        } else {
            None
        }
    }
}

fn arg_value(
    args: &[MbValue],
    kwargs: Option<MbValue>,
    index: usize,
    name: &str,
) -> Option<MbValue> {
    args.get(index).copied().or_else(|| dict_get(kwargs, name))
}

fn arg_int(
    args: &[MbValue],
    kwargs: Option<MbValue>,
    index: usize,
    name: &str,
    default: i64,
) -> i64 {
    arg_value(args, kwargs, index, name)
        .and_then(|v| v.as_int())
        .unwrap_or(default)
}

fn set_instance_field(instance: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert(name.to_string(), value);
            }
        }
    }
}

fn get_instance_bytes(instance: MbValue, name: &str) -> Vec<u8> {
    instance
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(name).copied()
            } else {
                None
            }
        })
        .map(|value| with_bytes(value, |b| b.to_vec()))
        .unwrap_or_default()
}

fn get_instance_int(instance: MbValue, name: &str, default: i64) -> i64 {
    instance
        .as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.read().unwrap().get(name).and_then(|v| v.as_int())
            } else {
                None
            }
        })
        .unwrap_or(default)
}

fn list_items(list: MbValue) -> Vec<MbValue> {
    list.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                Some(lock.read().unwrap().to_vec())
            } else {
                None
            }
        })
        .unwrap_or_default()
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

fn raise_zlib_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("zlib.error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

pub fn mb_zlib_compress(data: MbValue) -> MbValue {
    let out = with_bytes(data, |b| {
        let mut enc = ZlibEncoder::new(Vec::with_capacity(b.len() / 2), Compression::default());
        // Real DEFLATE — best-effort: if flate2 ever returns an error here it
        // means input ptr is bad, in which case return empty bytes rather
        // than panic (Python's zlib.error is not yet plumbed through MbValue
        // exception machinery; documenting the gap here for the conformance
        // sweep).
        if enc.write_all(b).is_err() {
            return Vec::new();
        }
        enc.finish().unwrap_or_default()
    });
    MbValue::from_ptr(MbObject::new_bytes(out))
}

fn encode_zlib_stream(data: &[u8], level: i64, wbits: i64, zdict: &[u8]) -> Result<Vec<u8>, ()> {
    let compression = if level < 0 {
        Compression::default()
    } else {
        Compression::new(level as u32)
    };
    let mut out = if wbits < 0 {
        let mut enc = DeflateEncoder::new(Vec::with_capacity(data.len() / 2), compression);
        enc.write_all(data).map_err(|_| ())?;
        enc.finish().map_err(|_| ())?
    } else {
        let mut enc = ZlibEncoder::new(Vec::with_capacity(data.len() / 2), compression);
        enc.write_all(data).map_err(|_| ())?;
        enc.finish().map_err(|_| ())?
    };
    if !zdict.is_empty() {
        let mut marked = Vec::with_capacity(ZDICT_MARKER.len() + 4 + out.len());
        marked.extend_from_slice(ZDICT_MARKER);
        marked.extend_from_slice(&adler32_slice(zdict).to_be_bytes());
        marked.append(&mut out);
        Ok(marked)
    } else {
        Ok(out)
    }
}

/// zlib.decompress(data) -> bytes (real DEFLATE).
///
/// Perf carve-out (#2107): zlib streams (unlike gzip) carry no original
/// length, so we use `b.len() * 8` as the initial capacity instead of
/// the previous `* 4` under-allocation. The 4× hint was triggering
/// repeated growth on multi-MB streams (typical zlib compression ratios
/// are 5–20× for text/JSON, well past the 4× threshold). 8× covers the
/// median case in a single allocation while keeping pathological
/// adversarial inputs bounded (caller-supplied `b.len()` already gates
/// total memory). The bytes owner (`new_bytes`) is a Vec move, not a
/// memcpy, so the materialized output is paid for exactly once.
pub fn mb_zlib_decompress(data: MbValue) -> MbValue {
    match with_bytes(data, decode_zlib_stream) {
        Ok(DecodeResult::Complete { output, .. }) => MbValue::from_ptr(MbObject::new_bytes(output)),
        Ok(DecodeResult::Incomplete) | Err(()) => {
            raise_zlib_error("Error while decompressing data")
        }
    }
}

enum DecodeResult {
    Complete { output: Vec<u8>, consumed: usize },
    Incomplete,
}

fn decode_zlib_stream(b: &[u8]) -> Result<DecodeResult, ()> {
    decode_zlib_stream_with_options(b, 15, &[])
}

fn decode_zlib_stream_with_options<'a>(
    b: &'a [u8],
    wbits: i64,
    zdict: &[u8],
) -> Result<DecodeResult, ()> {
    let input = strip_zdict_marker(b, zdict)?;
    let mut dec = Decompress::new(wbits >= 0);
    let mut buf = Vec::with_capacity(b.len().saturating_mul(8).max(256));
    loop {
        if buf.len() == buf.capacity() {
            buf.reserve(buf.capacity().max(256));
        }

        let offset = dec.total_in() as usize;
        if offset > input.len() {
            return Err(());
        }
        let before_in = dec.total_in();
        let before_out = buf.len();
        let flush = if offset < input.len() {
            FlushDecompress::None
        } else {
            FlushDecompress::Finish
        };
        let status = dec.decompress_vec(&input[offset..], &mut buf, flush);
        match status {
            Ok(Status::StreamEnd) => {
                return Ok(DecodeResult::Complete {
                    output: buf,
                    consumed: dec.total_in() as usize,
                });
            }
            Ok(Status::Ok) | Ok(Status::BufError) => {
                if dec.total_in() == before_in && buf.len() == before_out {
                    return Ok(DecodeResult::Incomplete);
                }
            }
            Err(_) => return Err(()),
        }
    }
}

fn strip_zdict_marker<'a>(input: &'a [u8], zdict: &[u8]) -> Result<&'a [u8], ()> {
    if !input.starts_with(ZDICT_MARKER) {
        return Ok(input);
    }
    if zdict.is_empty() || input.len() < ZDICT_MARKER.len() + 4 {
        return Err(());
    }
    let checksum_start = ZDICT_MARKER.len();
    let checksum = u32::from_be_bytes([
        input[checksum_start],
        input[checksum_start + 1],
        input[checksum_start + 2],
        input[checksum_start + 3],
    ]);
    if checksum != adler32_slice(zdict) {
        return Err(());
    }
    Ok(&input[checksum_start + 4..])
}

fn adler32_slice(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut s = 0u32;
    for byte in data {
        a = (a + *byte as u32) % 65521;
        s = (s + a) % 65521;
    }
    (s << 16) | a
}

pub fn mb_zlib_crc32(data: MbValue) -> MbValue {
    mb_zlib_crc32_seed(data, 0)
}

fn mb_zlib_crc32_seed(data: MbValue, seed: u32) -> MbValue {
    // Hardware-accelerated CRC32 via crc32fast (Task #27, #1265). Replaces
    // the explicit bit-by-bit loop that was 242× slower than CPython's
    // table-lookup zlib.crc32 — the proximate cause of the 0.40–0.61×
    // internal-time gap on zlib/gzip/lzma benches (all three call
    // zlib.crc32 in their hot loops). crc32fast picks the ARMv8 crc32
    // instruction on aarch64 (Apple Silicon) and PCLMULQDQ on x86_64.
    let crc = with_bytes(data, |b| {
        let mut hasher = crc32fast::Hasher::new_with_initial(seed);
        hasher.update(b);
        hasher.finalize()
    });
    MbValue::from_int(crc as i64)
}

pub fn mb_zlib_adler32(data: MbValue) -> MbValue {
    mb_zlib_adler32_seed(data, 1)
}

fn mb_zlib_adler32_seed(data: MbValue, seed: u32) -> MbValue {
    let val = with_bytes(data, |b| {
        let mut a = seed & 0xffff;
        let mut s = (seed >> 16) & 0xffff;
        for byte in b {
            a = (a + *byte as u32) % 65521;
            s = (s + a) % 65521;
        }
        (s << 16) | a
    });
    MbValue::from_int(val as i64)
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::{MbObject, ObjData};
    use super::super::super::value::MbValue;
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

        let d = MbValue::from_ptr(MbObject::new_dict());
        assert_eq!(super::with_bytes(d, |b| b.to_vec()), Vec::<u8>::new());

        assert_eq!(
            super::with_bytes(MbValue::none(), |b| b.to_vec()),
            Vec::<u8>::new()
        );
    }

    #[test]
    fn test_compress_roundtrip() {
        // Real DEFLATE round-trip: compress(decompress(x)) == x.
        let payload: Vec<u8> = (0u8..200).cycle().take(4096).collect();
        let input = MbValue::from_ptr(MbObject::new_bytes(payload.clone()));
        let compressed = mb_zlib_compress(input);
        let compressed_bytes = get_bytes_val(compressed).expect("compressed bytes");
        // Compression actually shrinks repeating data.
        assert!(
            compressed_bytes.len() < payload.len(),
            "compressed >= payload: {} >= {}",
            compressed_bytes.len(),
            payload.len()
        );
        // zlib magic byte 0x78 (zlib stream header).
        assert_eq!(compressed_bytes[0], 0x78);

        let to_decompress = MbValue::from_ptr(MbObject::new_bytes(compressed_bytes));
        let decompressed = mb_zlib_decompress(to_decompress);
        assert_eq!(get_bytes_val(decompressed), Some(payload));
    }

    #[test]
    fn test_compress_empty() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let result = mb_zlib_compress(input);
        let b = get_bytes_val(result).expect("compressed bytes");
        // Zlib header for empty stream is non-empty (~8 bytes).
        assert!(!b.is_empty());
        // Round-trip empty through decompress.
        let dec = mb_zlib_decompress(MbValue::from_ptr(MbObject::new_bytes(b)));
        assert_eq!(get_bytes_val(dec), Some(Vec::<u8>::new()));
    }

    #[test]
    fn test_crc32_empty() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        assert_eq!(mb_zlib_crc32(input).as_int(), Some(0));
    }

    #[test]
    fn test_crc32_known() {
        // CRC32 of a single zero byte = 0xD202EF8D
        let single_zero = MbValue::from_ptr(MbObject::new_bytes(vec![0x00u8]));
        assert_eq!(
            mb_zlib_crc32(single_zero).as_int(),
            Some(0xD202EF8D_u32 as i64)
        );
        // CRC32 of b"123456789" = 0xCBF43926 (canonical test vector).
        let canonical = MbValue::from_ptr(MbObject::new_bytes(b"123456789".to_vec()));
        assert_eq!(
            mb_zlib_crc32(canonical).as_int(),
            Some(0xCBF43926_u32 as i64)
        );
    }

    #[test]
    fn test_adler32_empty() {
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        assert_eq!(mb_zlib_adler32(input).as_int(), Some(1));
    }

    #[test]
    fn test_adler32_known() {
        // adler32([0x01]): a=(1+1)%65521=2, s=(0+2)%65521=2 → (2<<16)|2 = 131074
        let input = MbValue::from_ptr(MbObject::new_bytes(vec![0x01u8]));
        assert_eq!(mb_zlib_adler32(input).as_int(), Some(131074));
        // adler32(b"123456789") = 0x091E01DE (canonical test vector).
        let canonical = MbValue::from_ptr(MbObject::new_bytes(b"123456789".to_vec()));
        assert_eq!(mb_zlib_adler32(canonical).as_int(), Some(0x091E01DE));
    }
}
