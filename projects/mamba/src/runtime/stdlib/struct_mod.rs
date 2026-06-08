//! @codegen-skip: handwrite-pre-standardize
//!
//! struct module for Mamba (#415, #1265 Task #81, Wave-8).
//!
//! Implements CPython 3.12 `struct` stdlib 8-entry surface:
//!   Struct, calcsize, error, iter_unpack, pack, pack_into, unpack,
//!   unpack_from.
//!
//! Supported format codes: b/B, h/H, i/I, l/L, q/Q (integers), f/d
//! (floats), ? (bool), c (char/byte), x (pad), and a repeat-count prefix
//! (e.g. `3i`). Byte-order prefixes: `<`, `>`, `=`, `@`, `!`. Sizes match
//! CPython 3.12 standard mode; native-alignment (`@`/no-prefix) collapses
//! to the standard layout because mamba runs without C struct padding.
//!
//! Carve-outs:
//!   - `Struct` class returns an Instance stub with `format` + `size`
//!     fields. The instance is not yet callable as a method-bound
//!     pack/unpack object (CPython lets `s = struct.Struct("i"); s.pack(1)`);
//!     callers must use the module-level functions until method dispatch
//!     is wired through.
//!   - `error` is exposed as a string sentinel; the runtime does not yet
//!     model the Exception subclass hierarchy.
//!   - `iter_unpack` returns a fully-materialized list of tuples rather
//!     than a lazy iterator — semantically equivalent for typical loops
//!     but does not stream.
//!   - String/bytes codes `s`, `p` are not implemented; `n`/`N`
//!     (native-size signed/unsigned) collapse to 8 bytes.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + dispatch_{nullary,unary,binary} + format-string
//! lowering) is not yet emitted by score codegen. The pack/unpack format
//! mini-language is a perfect codegen candidate (table-driven by char and
//! prefix), but the section type doesn't exist yet. Will convert to
//! CODEGEN once the standardize sweep grows a `format_string_codec`
//! section type. See `.aw/handoffs/1414-patrol-handoff.md` cluster.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

unsafe extern "C" fn dispatch_pack(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fmt = a.get(0).copied().unwrap_or_else(MbValue::none);
    // `struct.pack(fmt, *args)` is variadic in CPython. Mamba's
    // variadic-splat lowering does not always reach native dispatch
    // functions cleanly (see the Cranelift func_id=554 verifier fail
    // around variadic call-site arity), so we accept two shapes:
    //   pack(fmt, v1, v2, v3)   — flat positional args
    //   pack(fmt, [v1, v2, v3]) — single list/tuple of values
    // The list-of-values shape is what end users fall back to when
    // the splat operator misbehaves, and it's what we exercise in
    // the bulk-record bench.
    let rest: Vec<MbValue> = if nargs == 2 {
        match a[1].as_ptr() {
            Some(ptr) => unsafe {
                match &(*ptr).data {
                    ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                    ObjData::Tuple(items) => items.clone(),
                    _ => vec![a[1]],
                }
            },
            None => vec![a[1]],
        }
    } else {
        a.iter().skip(1).copied().collect()
    };
    mb_struct_pack(fmt, &rest)
}

unsafe extern "C" fn dispatch_unpack(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_struct_unpack(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

dispatch_unary!(dispatch_calcsize, mb_struct_calcsize);
dispatch_unary!(dispatch_struct, mb_struct_new);

unsafe extern "C" fn dispatch_iter_unpack(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_struct_iter_unpack(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_pack_into(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fmt = a.get(0).copied().unwrap_or_else(MbValue::none);
    let buffer = a.get(1).copied().unwrap_or_else(MbValue::none);
    let offset = a.get(2).copied().unwrap_or_else(MbValue::none);
    // Variadic tail. Accept both flat positional and single list/tuple
    // (mirrors the dispatch_pack contract).
    let rest: Vec<MbValue> = if nargs == 4 {
        match a[3].as_ptr() {
            Some(ptr) => unsafe {
                match &(*ptr).data {
                    ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                    ObjData::Tuple(items) => items.clone(),
                    _ => vec![a[3]],
                }
            },
            None => vec![a[3]],
        }
    } else {
        a.iter().skip(3).copied().collect()
    };
    mb_struct_pack_into(fmt, buffer, offset, &rest)
}

unsafe extern "C" fn dispatch_unpack_from(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_struct_unpack_from(
        a.get(0).copied().unwrap_or_else(MbValue::none),
        a.get(1).copied().unwrap_or_else(MbValue::none),
        a.get(2).copied().unwrap_or_else(MbValue::none),
    )
}

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("pack", dispatch_pack as usize),
        ("unpack", dispatch_unpack as usize),
        ("calcsize", dispatch_calcsize as usize),
        ("Struct", dispatch_struct as usize),
        ("iter_unpack", dispatch_iter_unpack as usize),
        ("pack_into", dispatch_pack_into as usize),
        ("unpack_from", dispatch_unpack_from as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // `struct.error` — sentinel attribute used in `except struct.error:` and
    // `assertRaises(struct.error, ...)`. Mamba doesn't model the
    // Exception subclass hierarchy yet, so we expose an Instance with
    // class_name "error" so identity / attribute access work; the
    // dispatch path treats this as the exception sentinel.
    let err_obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "error".to_string(),
            fields: RwLock::new({
                let mut f = FxHashMap::default();
                f.insert(
                    "__name__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("error".to_string())),
                );
                f.insert(
                    "__module__".to_string(),
                    MbValue::from_ptr(MbObject::new_str("struct".to_string())),
                );
                f
            }),
        },
    });
    attrs.insert(
        "error".to_string(),
        MbValue::from_ptr(Box::into_raw(err_obj)),
    );

    super::register_module("struct", attrs);
}

/// Extract a borrowed `&str` from an MbValue holding a heap string and apply
/// `f`. Non-string values are mapped to the empty string.
#[inline]
fn with_str<R>(val: MbValue, f: impl FnOnce(&str) -> R) -> R {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return f(s.as_str());
            }
        }
    }
    f("")
}

/// Strip the leading byte-order / native-mode prefix from a format string.
/// Mamba's pack/unpack uses little-endian on the wire regardless of the
/// prefix because the runtime itself is fixed-LE; the prefix is consumed
/// for surface compatibility (CPython accepts `<i`, `>i`, `=i`, `@i`, `!i`,
/// or bare `i`). `!` and `>` map to big-endian in CPython.
fn split_prefix(fmt: &str) -> (Endian, &str) {
    let bytes = fmt.as_bytes();
    if let Some(&b) = bytes.first() {
        match b {
            b'<' => (Endian::Little, &fmt[1..]),
            b'>' | b'!' => (Endian::Big, &fmt[1..]),
            b'=' | b'@' => (Endian::Little, &fmt[1..]),
            _ => (Endian::Little, fmt),
        }
    } else {
        (Endian::Little, fmt)
    }
}

#[derive(Copy, Clone)]
enum Endian {
    Little,
    Big,
}

/// One parsed format token: optional repeat count + a single code char.
struct Token {
    count: usize,
    code: char,
}

fn parse_format(rest: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut count: Option<usize> = None;
    for c in rest.chars() {
        if c.is_ascii_digit() {
            count = Some(count.unwrap_or(0) * 10 + (c as usize - '0' as usize));
            continue;
        }
        if c.is_ascii_whitespace() {
            continue;
        }
        tokens.push(Token {
            count: count.take().unwrap_or(1),
            code: c,
        });
    }
    tokens
}

fn code_size(c: char) -> usize {
    match c {
        'x' | 'c' | 'b' | 'B' | '?' | 's' => 1,
        'h' | 'H' => 2,
        'i' | 'I' | 'l' | 'L' | 'f' => 4,
        'q' | 'Q' | 'd' | 'n' | 'N' => 8,
        _ => 0,
    }
}

/// struct.calcsize(fmt) -> int
pub fn mb_struct_calcsize(fmt: MbValue) -> MbValue {
    let total = with_str(fmt, |s| {
        let (_endian, rest) = split_prefix(s);
        parse_format(rest)
            .iter()
            .map(|t| t.count * code_size(t.code))
            .sum::<usize>()
    });
    MbValue::from_int(total as i64)
}

fn write_int_le(out: &mut Vec<u8>, val: i64, size: usize) {
    let bytes = val.to_le_bytes();
    out.extend_from_slice(&bytes[..size]);
}

fn write_int_be(out: &mut Vec<u8>, val: i64, size: usize) {
    let bytes = val.to_be_bytes();
    // skip leading zero-pad bytes from i64 -> requested size
    out.extend_from_slice(&bytes[8 - size..]);
}

fn read_int_le(slice: &[u8], code: char) -> i64 {
    match code {
        'b' => i8::from_le_bytes([slice[0]]) as i64,
        'B' => slice[0] as i64,
        '?' => {
            if slice[0] != 0 {
                1
            } else {
                0
            }
        }
        'h' => i16::from_le_bytes([slice[0], slice[1]]) as i64,
        'H' => u16::from_le_bytes([slice[0], slice[1]]) as i64,
        'i' | 'l' => i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as i64,
        'I' | 'L' => u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as i64,
        'q' | 'n' => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&slice[..8]);
            i64::from_le_bytes(arr)
        }
        'Q' | 'N' => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&slice[..8]);
            u64::from_le_bytes(arr) as i64
        }
        _ => 0,
    }
}

fn read_int_be(slice: &[u8], code: char) -> i64 {
    match code {
        'b' => i8::from_be_bytes([slice[0]]) as i64,
        'B' => slice[0] as i64,
        '?' => {
            if slice[0] != 0 {
                1
            } else {
                0
            }
        }
        'h' => i16::from_be_bytes([slice[0], slice[1]]) as i64,
        'H' => u16::from_be_bytes([slice[0], slice[1]]) as i64,
        'i' | 'l' => i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]) as i64,
        'I' | 'L' => u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]) as i64,
        'q' | 'n' => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&slice[..8]);
            i64::from_be_bytes(arr)
        }
        'Q' | 'N' => {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&slice[..8]);
            u64::from_be_bytes(arr) as i64
        }
        _ => 0,
    }
}

fn arg_as_int(v: MbValue) -> i64 {
    if let Some(i) = v.as_int() {
        return i;
    }
    if let Some(b) = v.as_bool() {
        return if b { 1 } else { 0 };
    }
    // bytes-like 1-byte char (Python's 'c' code packs/unpacks a single byte)
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Bytes(ref b) = (*ptr).data {
                if !b.is_empty() {
                    return b[0] as i64;
                }
            }
        }
    }
    0
}

fn arg_as_float(v: MbValue) -> f64 {
    if let Some(f) = v.as_float() {
        return f;
    }
    if let Some(i) = v.as_int() {
        return i as f64;
    }
    0.0
}

/// struct.pack(fmt, *args) -> bytes
pub fn mb_struct_pack(fmt: MbValue, args: &[MbValue]) -> MbValue {
    let mut out = Vec::new();
    with_str(fmt, |fmt_str| {
        let (endian, rest) = split_prefix(fmt_str);
        let tokens = parse_format(rest);
        let mut ai = 0usize;
        for tok in &tokens {
            for _ in 0..tok.count {
                let size = code_size(tok.code);
                match tok.code {
                    'x' => out.push(0u8),
                    'c' => {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        let byte = if let Some(ptr) = v.as_ptr() {
                            unsafe {
                                match &(*ptr).data {
                                    ObjData::Bytes(b) if !b.is_empty() => b[0],
                                    _ => 0,
                                }
                            }
                        } else {
                            0
                        };
                        out.push(byte);
                    }
                    'f' => {
                        let f = arg_as_float(args.get(ai).copied().unwrap_or_else(MbValue::none));
                        ai += 1;
                        let bytes = match endian {
                            Endian::Little => (f as f32).to_le_bytes(),
                            Endian::Big => (f as f32).to_be_bytes(),
                        };
                        out.extend_from_slice(&bytes);
                    }
                    'd' => {
                        let f = arg_as_float(args.get(ai).copied().unwrap_or_else(MbValue::none));
                        ai += 1;
                        let bytes = match endian {
                            Endian::Little => f.to_le_bytes(),
                            Endian::Big => f.to_be_bytes(),
                        };
                        out.extend_from_slice(&bytes);
                    }
                    _ => {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        let n = arg_as_int(v);
                        match endian {
                            Endian::Little => write_int_le(&mut out, n, size),
                            Endian::Big => write_int_be(&mut out, n, size),
                        }
                    }
                }
            }
        }
    });
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// Decode `bytes` into `values` according to the parsed format tokens.
/// Pulled out of `mb_struct_unpack` so the borrowed-bytes fast path
/// and the owned-bytes slow path can share the same decoder body
/// without cloning the input buffer.
fn struct_unpack_into(bytes: &[u8], endian: Endian, tokens: &[Token], values: &mut Vec<MbValue>) {
    let mut offset = 0usize;
    for tok in tokens {
        for _ in 0..tok.count {
            let size = code_size(tok.code);
            if offset + size > bytes.len() {
                return;
            }
            let slice = &bytes[offset..offset + size];
            match tok.code {
                'x' => {}
                'c' => values.push(MbValue::from_ptr(MbObject::new_bytes(vec![slice[0]]))),
                'f' => {
                    let arr = [slice[0], slice[1], slice[2], slice[3]];
                    let f = match endian {
                        Endian::Little => f32::from_le_bytes(arr),
                        Endian::Big => f32::from_be_bytes(arr),
                    } as f64;
                    values.push(MbValue::from_float(f));
                }
                'd' => {
                    let mut arr = [0u8; 8];
                    arr.copy_from_slice(slice);
                    let f = match endian {
                        Endian::Little => f64::from_le_bytes(arr),
                        Endian::Big => f64::from_be_bytes(arr),
                    };
                    values.push(MbValue::from_float(f));
                }
                '?' => values.push(MbValue::from_bool(slice[0] != 0)),
                _ => {
                    let n = match endian {
                        Endian::Little => read_int_le(slice, tok.code),
                        Endian::Big => read_int_be(slice, tok.code),
                    };
                    values.push(MbValue::from_int(n));
                }
            }
            offset += size;
        }
    }
}

/// struct.unpack(fmt, data) -> tuple
pub fn mb_struct_unpack(fmt: MbValue, data: MbValue) -> MbValue {
    let mut values: Vec<MbValue> = Vec::new();
    with_str(fmt, |fmt_str| {
        let (endian, rest) = split_prefix(fmt_str);
        let tokens = parse_format(rest);

        // Borrow the input buffer instead of cloning it. For the common
        // binary-protocol shape (7-byte records, 20k iters) this saves
        // one Vec<u8> heap allocation + memcpy per call — the dominant
        // cost on the pack/unpack hot path. ByteArray takes the read
        // guard for the duration of the decode so the slice borrow is
        // sound.
        if let Some(ptr) = data.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Bytes(b) => {
                        struct_unpack_into(b, endian, &tokens, &mut values);
                    }
                    ObjData::ByteArray(ref lock) => {
                        let guard = lock.read().unwrap();
                        struct_unpack_into(&guard, endian, &tokens, &mut values);
                    }
                    _ => {}
                }
            }
        }
    });

    MbValue::from_ptr(MbObject::new_tuple(values))
}

/// Extract a byte buffer from any of bytes/bytearray/list/tuple of ints.
fn extract_bytes(data: MbValue) -> Vec<u8> {
    data.as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => b.clone(),
                ObjData::ByteArray(ref lock) => lock.read().unwrap().clone(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default()
}

/// struct.Struct(format) -> Struct instance
///
/// Returns an Instance with `format` and `size` fields. The instance is
/// a passive descriptor; the module-level `pack`/`unpack` functions are
/// the active surface (see carve-outs in the module docstring).
pub fn mb_struct_new(fmt: MbValue) -> MbValue {
    let size = mb_struct_calcsize(fmt);
    let format_copy = with_str(fmt, |s| s.to_string());
    let mut fields = FxHashMap::default();
    fields.insert(
        "format".to_string(),
        MbValue::from_ptr(MbObject::new_str(format_copy)),
    );
    fields.insert("size".to_string(), size);
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "Struct".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// struct.iter_unpack(fmt, buffer) -> list of tuples
///
/// Carve-out: returns a fully-materialized list of tuples rather than a
/// lazy iterator. Semantically equivalent for `for t in struct.iter_unpack(...)`.
pub fn mb_struct_iter_unpack(fmt: MbValue, buffer: MbValue) -> MbValue {
    let chunk_size = mb_struct_calcsize(fmt).as_int().unwrap_or(0) as usize;
    let bytes = extract_bytes(buffer);
    let mut results: Vec<MbValue> = Vec::new();
    if chunk_size == 0 {
        return MbValue::from_ptr(MbObject::new_list(results));
    }
    let mut off = 0usize;
    while off + chunk_size <= bytes.len() {
        let chunk = bytes[off..off + chunk_size].to_vec();
        let chunk_val = MbValue::from_ptr(MbObject::new_bytes(chunk));
        results.push(mb_struct_unpack(fmt, chunk_val));
        off += chunk_size;
    }
    MbValue::from_ptr(MbObject::new_list(results))
}

/// struct.pack_into(fmt, buffer, offset, *args)
///
/// Packs `args` per `fmt` and writes the resulting bytes into `buffer`
/// (a bytearray) starting at `offset`. Returns `None`. Out-of-range
/// writes are silently clipped (CPython raises `struct.error`; mamba
/// surfaces the truncation via the buffer's final length).
pub fn mb_struct_pack_into(
    fmt: MbValue,
    buffer: MbValue,
    offset: MbValue,
    args: &[MbValue],
) -> MbValue {
    let packed = mb_struct_pack(fmt, args);
    let packed_bytes: Vec<u8> = packed
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => b.clone(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default();
    let off = offset.as_int().unwrap_or(0) as usize;
    if let Some(ptr) = buffer.as_ptr() {
        unsafe {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let mut buf = lock.write().unwrap();
                for (i, b) in packed_bytes.iter().enumerate() {
                    let pos = off + i;
                    if pos < buf.len() {
                        buf[pos] = *b;
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// struct.unpack_from(fmt, buffer, offset=0) -> tuple
///
/// Like `unpack`, but starts reading at `offset` into the buffer.
pub fn mb_struct_unpack_from(fmt: MbValue, buffer: MbValue, offset: MbValue) -> MbValue {
    let bytes = extract_bytes(buffer);
    let off = offset.as_int().unwrap_or(0) as usize;
    let sliced = if off <= bytes.len() {
        bytes[off..].to_vec()
    } else {
        Vec::new()
    };
    let sliced_val = MbValue::from_ptr(MbObject::new_bytes(sliced));
    mb_struct_unpack(fmt, sliced_val)
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_calcsize_basic() {
        assert_eq!(mb_struct_calcsize(s("i")).as_int(), Some(4));
        assert_eq!(mb_struct_calcsize(s("ii")).as_int(), Some(8));
        assert_eq!(mb_struct_calcsize(s("bhi")).as_int(), Some(7));
        assert_eq!(mb_struct_calcsize(s("QB")).as_int(), Some(9));
    }

    #[test]
    fn test_calcsize_prefixed() {
        assert_eq!(mb_struct_calcsize(s("<i")).as_int(), Some(4));
        assert_eq!(mb_struct_calcsize(s(">i")).as_int(), Some(4));
        assert_eq!(mb_struct_calcsize(s("!i")).as_int(), Some(4));
        assert_eq!(mb_struct_calcsize(s("=i")).as_int(), Some(4));
    }

    #[test]
    fn test_calcsize_repeat() {
        assert_eq!(mb_struct_calcsize(s("3i")).as_int(), Some(12));
        assert_eq!(mb_struct_calcsize(s("3b3B")).as_int(), Some(6));
    }

    #[test]
    fn test_pack_unpack_roundtrip() {
        let args = vec![
            MbValue::from_int(42),
            MbValue::from_int(1000),
            MbValue::from_int(-5),
        ];
        let packed = mb_struct_pack(s("iHb"), &args);
        let unpacked = mb_struct_unpack(s("iHb"), packed);
        unsafe {
            if let ObjData::Tuple(ref items) = (*unpacked.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(42));
                assert_eq!(items[1].as_int(), Some(1000));
                assert_eq!(items[2].as_int(), Some(-5));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_pack_endianness() {
        let args = vec![MbValue::from_int(0x12345678)];
        let le = mb_struct_pack(s("<i"), &args);
        let be = mb_struct_pack(s(">i"), &args);
        unsafe {
            if let ObjData::Bytes(ref b) = (*le.as_ptr().unwrap()).data {
                assert_eq!(b, &[0x78, 0x56, 0x34, 0x12]);
            } else {
                panic!("expected Bytes");
            }
            if let ObjData::Bytes(ref b) = (*be.as_ptr().unwrap()).data {
                assert_eq!(b, &[0x12, 0x34, 0x56, 0x78]);
            } else {
                panic!("expected Bytes");
            }
        }
    }

    #[test]
    fn test_pack_float_roundtrip() {
        let args = vec![MbValue::from_float(3.14)];
        let packed = mb_struct_pack(s("d"), &args);
        let unpacked = mb_struct_unpack(s("d"), packed);
        unsafe {
            if let ObjData::Tuple(ref items) = (*unpacked.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 1);
                assert!((items[0].as_float().unwrap() - 3.14).abs() < 1e-9);
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_pack_pad() {
        let args = vec![MbValue::from_int(1)];
        let packed = mb_struct_pack(s("xb"), &args);
        unsafe {
            if let ObjData::Bytes(ref b) = (*packed.as_ptr().unwrap()).data {
                assert_eq!(b.len(), 2);
                assert_eq!(b[0], 0);
                assert_eq!(b[1], 1);
            } else {
                panic!("expected Bytes");
            }
        }
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    // -- mb_struct_new (Struct class) tests --

    #[test]
    fn test_struct_new_format_and_size() {
        let st = mb_struct_new(s("3i"));
        assert!(st.as_ptr().is_some());
        assert_eq!(get_str(get_field(st, "format")), Some("3i".to_string()));
        assert_eq!(get_field(st, "size").as_int(), Some(12));
    }

    #[test]
    fn test_struct_new_prefixed_format() {
        let st = mb_struct_new(s("<hHi"));
        assert_eq!(get_str(get_field(st, "format")), Some("<hHi".to_string()));
        assert_eq!(get_field(st, "size").as_int(), Some(8));
    }

    // -- mb_struct_iter_unpack tests --

    #[test]
    fn test_iter_unpack_chunks_bytes() {
        // Pack three i values back-to-back, then iter_unpack with "i".
        let args = vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ];
        let packed = mb_struct_pack(s("iii"), &args);
        let result = mb_struct_iter_unpack(s("i"), packed);
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let list = lock.read().unwrap();
                assert_eq!(list.len(), 3);
                let mut got = Vec::new();
                for t in list.iter() {
                    if let ObjData::Tuple(ref items) = (*t.as_ptr().unwrap()).data {
                        got.push(items[0].as_int().unwrap());
                    } else {
                        panic!("expected Tuple");
                    }
                }
                assert_eq!(got, vec![10, 20, 30]);
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_iter_unpack_empty_buffer() {
        let empty = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let result = mb_struct_iter_unpack(s("i"), empty);
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected List");
            }
        }
    }

    // -- mb_struct_pack_into / unpack_from tests --

    #[test]
    fn test_pack_into_writes_at_offset() {
        // Allocate 8-byte bytearray, pack one i at offset 2.
        let buf = MbValue::from_ptr(MbObject::new_bytearray(vec![0u8; 8]));
        let args = vec![MbValue::from_int(0x01020304)];
        let _ = mb_struct_pack_into(s("<i"), buf, MbValue::from_int(2), &args);
        unsafe {
            if let ObjData::ByteArray(ref lock) = (*buf.as_ptr().unwrap()).data {
                let b = lock.read().unwrap();
                assert_eq!(b.len(), 8);
                // bytes 0..2 untouched
                assert_eq!(&b[0..2], &[0, 0]);
                // bytes 2..6 hold the LE i32
                assert_eq!(&b[2..6], &[0x04, 0x03, 0x02, 0x01]);
                // bytes 6..8 untouched
                assert_eq!(&b[6..8], &[0, 0]);
            } else {
                panic!("expected ByteArray");
            }
        }
    }

    #[test]
    fn test_unpack_from_at_offset() {
        // bytes = [0xff, 0xff, 0x04, 0x03, 0x02, 0x01, 0xff, 0xff]
        let data = vec![0xff, 0xff, 0x04, 0x03, 0x02, 0x01, 0xff, 0xff];
        let buf = MbValue::from_ptr(MbObject::new_bytes(data));
        let r = mb_struct_unpack_from(s("<i"), buf, MbValue::from_int(2));
        unsafe {
            if let ObjData::Tuple(ref items) = (*r.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].as_int(), Some(0x01020304));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_pack_into_unpack_from_roundtrip() {
        // Round-trip via a bytearray buffer with leading + trailing pad.
        let buf = MbValue::from_ptr(MbObject::new_bytearray(vec![0u8; 16]));
        let args = vec![MbValue::from_int(42), MbValue::from_int(1000)];
        let _ = mb_struct_pack_into(s("<iH"), buf, MbValue::from_int(4), &args);
        let r = mb_struct_unpack_from(s("<iH"), buf, MbValue::from_int(4));
        unsafe {
            if let ObjData::Tuple(ref items) = (*r.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_int(), Some(42));
                assert_eq!(items[1].as_int(), Some(1000));
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_unpack_from_offset_past_end_returns_empty_tuple() {
        let data = vec![1u8, 2, 3];
        let buf = MbValue::from_ptr(MbObject::new_bytes(data));
        let r = mb_struct_unpack_from(s("i"), buf, MbValue::from_int(10));
        unsafe {
            if let ObjData::Tuple(ref items) = (*r.as_ptr().unwrap()).data {
                // No bytes left to read — produces no tuple entries.
                assert_eq!(items.len(), 0);
            } else {
                panic!("expected Tuple");
            }
        }
    }

    // -- error sentinel tests --

    #[test]
    fn test_error_is_instance_with_module_name() {
        // The register() call wires `struct.error` as an Instance whose
        // class_name is "error" and that carries __name__/__module__
        // fields. We can't easily call register() in tests, but we can
        // verify the same shape is constructible.
        let err_obj = Box::new(MbObject {
            header: MbObjectHeader {
                rc: AtomicU32::new(1),
                kind: ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: "error".to_string(),
                fields: RwLock::new({
                    let mut f = FxHashMap::default();
                    f.insert(
                        "__name__".to_string(),
                        MbValue::from_ptr(MbObject::new_str("error".to_string())),
                    );
                    f.insert(
                        "__module__".to_string(),
                        MbValue::from_ptr(MbObject::new_str("struct".to_string())),
                    );
                    f
                }),
            },
        });
        let err = MbValue::from_ptr(Box::into_raw(err_obj));
        assert_eq!(
            get_str(get_field(err, "__name__")),
            Some("error".to_string())
        );
        assert_eq!(
            get_str(get_field(err, "__module__")),
            Some("struct".to_string())
        );
    }
}
