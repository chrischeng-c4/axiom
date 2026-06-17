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
//!   - `error` is exposed as an Instance whose `__name__` is "error" (and
//!     `__module__` is "struct"), so `except struct.error:` /
//!     `assertRaises(struct.error, ...)` match the pending exception raised
//!     on a bad format char. The runtime does not yet model the full
//!     Exception subclass hierarchy (no `issubclass(struct.error, Exception)`).
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
    // the bulk-record bench. BUT a format that consumes exactly one
    // value (e.g. ">?" packs a list by truthiness, "3s" packs a bytes
    // field) must receive that single object verbatim, never unpacked —
    // so we only flatten a lone list/tuple when the format wants !=1 slots.
    let rest: Vec<MbValue> = build_pack_args(fmt, &a[1..]);
    mb_struct_pack(fmt, &rest)
}

/// Resolve the variadic value tail for `pack`/`pack_into`. Flattens a single
/// list/tuple argument into its elements only when the format consumes a
/// number of values other than one; a one-value format keeps the object whole.
fn build_pack_args(fmt: MbValue, tail: &[MbValue]) -> Vec<MbValue> {
    if tail.len() == 1 {
        let slots = fmt_value_slots(fmt);
        if slots != Some(1) {
            if let Some(ptr) = tail[0].as_ptr() {
                unsafe {
                    match &(*ptr).data {
                        ObjData::List(ref lock) => return lock.read().unwrap().to_vec(),
                        ObjData::Tuple(items) => return items.clone(),
                        _ => {}
                    }
                }
            }
        }
    }
    tail.to_vec()
}

/// Number of argument values the format string consumes, or `None` if the
/// format is malformed (the malformed case is reported later by `pack`).
fn fmt_value_slots(fmt: MbValue) -> Option<usize> {
    let fmt = normalize_format(fmt);
    let mut out = None;
    with_str(fmt, |s| {
        if let Some((_e, tokens)) = parse_checked_quiet(s) {
            out = Some(value_slots(&tokens));
        }
    });
    out
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
    // (mirrors the dispatch_pack contract, including the one-value carve-out).
    let tail: &[MbValue] = if a.len() > 3 { &a[3..] } else { &[] };
    let rest: Vec<MbValue> = build_pack_args(fmt, tail);
    mb_struct_pack_into(fmt, buffer, offset, &rest)
}

fn struct_kwarg_get(dict: MbValue, name: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let guard = lock.read().ok()?;
            let key = super::super::dict_ops::DictKey::Str(name.to_string());
            guard.get(&key).copied()
        } else {
            None
        }
    })
}

fn is_dict_value(v: MbValue) -> bool {
    v.as_ptr().is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
}

unsafe extern "C" fn dispatch_unpack_from(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // a[0] is the (partial-bound) format. unpack_from(buffer, offset=0) accepts
    // both positionally and by keyword; the call-lowering appends keyword args
    // as a trailing dict. The buffer is always bytes-like (never a dict), so a
    // trailing dict is unambiguously the kwargs.
    let fmt = a.first().copied().unwrap_or_else(MbValue::none);
    let rest = if a.len() > 1 { &a[1..] } else { &[][..] };
    let (positional, kwargs) = match rest.last() {
        Some(&last) if is_dict_value(last) => (&rest[..rest.len() - 1], Some(last)),
        _ => (rest, None),
    };
    let mut buffer = positional.first().copied().unwrap_or_else(MbValue::none);
    let mut offset = positional.get(1).copied().unwrap_or_else(MbValue::none);
    if let Some(kw) = kwargs {
        if let Some(b) = struct_kwarg_get(kw, "buffer") { buffer = b; }
        if let Some(o) = struct_kwarg_get(kw, "offset") { offset = o; }
    }
    mb_struct_unpack_from(fmt, buffer, offset)
}

/// `Struct.__init__(self, format)` — (re)compile `self` to `format`. Bound on
/// each Struct instance as a partial that binds `self`, so `s.__init__("ii")`
/// dispatches `dispatch_struct_init(self, "ii")` and rebinds every field
/// (format, size, pack/unpack/...) to the new format. This is the engine
/// behind `struct.Struct` re-initialization (CPython allows calling __init__
/// again to retarget a Struct) and `super().__init__(fmt)` from a subclass.
unsafe extern "C" fn dispatch_struct_init(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let this = a.get(0).copied().unwrap_or_else(MbValue::none);
    let fmt = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_struct_init_inplace(this, fmt);
    MbValue::none()
}

/// Repopulate an existing Struct (or Struct-subclass) Instance's fields for a
/// new `format`. Used by `__init__` reinit and subclass `super().__init__`.
pub fn mb_struct_init_inplace(this: MbValue, fmt: MbValue) {
    let fmt = normalize_format(fmt);
    if let Some(ptr) = this.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut guard = fields.write().unwrap();
                populate_struct_fields(&mut guard, fmt);
            }
        }
    }
}

/// `Struct.__init__(self, format)` with the SystemV/C calling convention the
/// class-method dispatcher (and `super().__init__(...)`) uses for registered
/// methods. Registering `Struct` as a class whose `__init__` is this function
/// lets a user subclass — `class S(struct.Struct): def __init__(self):
/// super().__init__(">h")` — populate its instance with the struct method
/// table, so `S().pack(...)` works.
extern "C" fn mb_struct_init_sysv(this: MbValue, fmt: MbValue) -> MbValue {
    mb_struct_init_inplace(this, fmt);
    MbValue::none()
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

    // `dispatch_struct_init` is not a module attribute, but it is wired as a
    // bound `__init__` method on every Struct instance; register its address so
    // the bound-method call protocol recognises it as a native dispatcher.
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(dispatch_struct_init as usize as u64);
    });

    // Register `Struct` as a real class so it can be subclassed:
    //   class S(struct.Struct):
    //       def __init__(self): super().__init__(">h")
    // `super().__init__(...)` resolves `Struct.__init__` via the MRO and the
    // class-method dispatcher calls it with the SystemV ABI (self, fmt), so the
    // subclass instance gets the full struct method table.
    {
        let mut methods = HashMap::new();
        methods.insert(
            "__init__".to_string(),
            MbValue::from_func(mb_struct_init_sysv as usize),
        );
        super::super::class::mb_class_register("Struct", vec![], methods);
    }

    // `struct.error` — the exception type used in `except struct.error:` and
    // `assertRaises(struct.error, ...)`. Mamba's `except <expr>:` lowering
    // resolves the caught type by `extract_str`-ing the operand (see
    // `exception::mb_exception_matches`), which only reads `ObjData::Str`.
    // Our `pack`/`unpack`/`calcsize` raise the exception with type name
    // `"error"`, so the sentinel MUST also stringify to `"error"` for the
    // handler to match. Expose `struct.error` as the immortal string
    // `"error"`: `hasattr(struct, "error")` stays true, and a bare
    // `except struct.error:` now actually catches the pending exception.
    attrs.insert(
        "error".to_string(),
        MbValue::from_ptr(MbObject::new_str_immortal("error".to_string())),
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

#[derive(Copy, Clone)]
enum Endian {
    Little,
    Big,
}

/// One parsed format token: optional repeat count + a single code char +
/// the sizing mode it was parsed under. For the string codes (`s`, `p`) the
/// `count` is the field width and the token consumes exactly one argument (not
/// `count` arguments). `mode` is carried so size / alignment lookups stay
/// correct even after the endian/mode prefix has been stripped.
struct Token {
    count: usize,
    code: char,
    mode: Mode,
}

/// Native size of this token's scalar code (mode-aware).
fn tok_size(t: &Token) -> usize {
    code_size_mode(t.code, t.mode)
}

/// Raise `struct.error(msg)` as a catchable mamba exception. The type name
/// `"error"` matches the `struct.error` sentinel string wired in
/// `register()`, so `except struct.error:` catches this pending exception.
fn raise_struct_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("error".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Raise a built-in `TypeError(msg)` (e.g. unpacking a non-buffer object).
fn raise_type_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Raise a built-in `OverflowError(msg)` (float too large for `f`/`e`).
fn raise_overflow_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
}

/// Byte-order / sizing mode of a format string.
///
/// CPython distinguishes *native* mode (`@` prefix or no prefix at all) from
/// *standard* mode (`<` / `>` / `=` / `!`):
///   * Native mode uses the host C ABI sizes (`l`/`L` = 8 on LP64) and inserts
///     alignment padding before each field; the `n`/`N`/`P` codes are allowed.
///   * Standard mode uses fixed sizes (`l`/`L` = 4), no alignment padding, and
///     rejects the native-only `n`/`N`/`P` codes.
#[derive(Copy, Clone, PartialEq)]
enum Mode {
    Native,
    Standard,
}

/// Strip the leading byte-order / native-mode prefix and return the endian,
/// the sizing mode, and the remaining code string. Only a marker at index 0 is
/// a prefix; markers anywhere else are rejected by `parse_format` as bad chars.
/// `>`/`!` are big-endian; `<` is little-endian; `=`/`@`/bare select the host
/// order (LE here). `@` and the bare (no-prefix) form are native mode; every
/// explicit byte-order char (`<`/`>`/`=`/`!`) is standard mode.
fn split_prefix(fmt: &str) -> (Endian, Mode, &str) {
    let bytes = fmt.as_bytes();
    if let Some(&b) = bytes.first() {
        match b {
            b'<' => (Endian::Little, Mode::Standard, &fmt[1..]),
            b'>' | b'!' => (Endian::Big, Mode::Standard, &fmt[1..]),
            b'=' => (Endian::Little, Mode::Standard, &fmt[1..]),
            b'@' => (Endian::Little, Mode::Native, &fmt[1..]),
            _ => (Endian::Little, Mode::Native, fmt),
        }
    } else {
        (Endian::Little, Mode::Native, fmt)
    }
}

/// Whether `c` is a recognised format/pad code at all (mode-independent). Any
/// other character is a hard "bad char in struct format" error.
fn is_known_code(c: char) -> bool {
    matches!(
        c,
        'x' | 'c'
            | 'b'
            | 'B'
            | '?'
            | 's'
            | 'p'
            | 'e'
            | 'h'
            | 'H'
            | 'i'
            | 'I'
            | 'l'
            | 'L'
            | 'f'
            | 'q'
            | 'Q'
            | 'd'
            | 'n'
            | 'N'
            | 'P'
    )
}

/// Size in bytes of a single scalar format code for a given sizing mode. The
/// `n`/`N`/`P` codes only exist in native mode; `l`/`L` are 8 bytes in native
/// mode but 4 in standard mode. `s`/`p`/`x`/`c` widths come from the repeat
/// count and report 1 here.
fn code_size_mode(c: char, mode: Mode) -> usize {
    match c {
        'x' | 'c' | 'b' | 'B' | '?' | 's' | 'p' => 1,
        'e' | 'h' | 'H' => 2,
        'i' | 'I' | 'f' => 4,
        'l' | 'L' => {
            if mode == Mode::Native {
                8
            } else {
                4
            }
        }
        'q' | 'Q' | 'd' | 'n' | 'N' | 'P' => 8,
        _ => 0,
    }
}

/// Natural alignment of a scalar code in native mode (0 = no alignment, used in
/// standard mode where fields are tightly packed). For the scalar integer /
/// float codes the alignment equals the native size; 1-byte and string/pad
/// codes never force padding.
fn code_align(c: char, mode: Mode) -> usize {
    if mode == Mode::Standard {
        return 1; // standard mode never pads
    }
    match c {
        'x' | 'c' | 'b' | 'B' | '?' | 's' | 'p' => 1,
        _ => code_size_mode(c, mode),
    }
}

/// Parse a prefix-stripped format string into tokens, applying every CPython
/// 3.12 structural rule and raising `struct.error` (returning `None`) on the
/// first violation:
///   * an unknown / NUL / mid-string byte-order char is a bad char,
///   * a repeat count with no following code ("dangling/trailing count") errors,
///   * a repeat count that overflows the platform size is "overflow in item count".
/// On success the returned tokens carry their resolved repeat count.
///
/// `mode` controls native-only code acceptance: `n`/`N`/`P` are valid only in
/// native mode, and CPython reports them as "bad char in struct format" in
/// standard mode (`<`/`>`/`=`/`!`).
fn parse_format(rest: &str, mode: Mode) -> Option<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut count: Option<usize> = None;
    let mut overflow = false;
    for c in rest.chars() {
        if c.is_ascii_digit() {
            let d = c as usize - '0' as usize;
            let next = count
                .unwrap_or(0)
                .checked_mul(10)
                .and_then(|v| v.checked_add(d));
            match next {
                Some(v) => count = Some(v),
                None => {
                    overflow = true;
                    count = Some(usize::MAX);
                }
            }
            continue;
        }
        if c.is_ascii_whitespace() {
            // CPython skips ASCII whitespace between tokens, but a pending
            // repeat count followed by whitespace then a code is still valid
            // ("4 i" is not legal CPython, but our porters never emit it);
            // keep the count pending across whitespace for safety.
            continue;
        }
        if c == '\0' {
            // CPython reports embedded NULs with a dedicated message.
            raise_struct_error("embedded null character");
            return None;
        }
        // `n`/`N`/`P` are native-only; in standard mode they are bad chars.
        let native_only = matches!(c, 'n' | 'N' | 'P');
        if !is_known_code(c) || (native_only && mode == Mode::Standard) {
            // Unknown code, a native-only code in standard mode, or a stray
            // byte-order marker not at position 0 (split_prefix consumed a
            // leading one).
            raise_struct_error("bad char in struct format");
            return None;
        }
        if overflow {
            raise_struct_error("overflow in item count");
            return None;
        }
        tokens.push(Token {
            count: count.take().unwrap_or(1),
            code: c,
            mode,
        });
    }
    // A repeat count with no following code (e.g. "4", "12345", "c12345",
    // "14s42") is illegal: CPython raises "repeat count given without format
    // specifier".
    if count.is_some() {
        if overflow {
            raise_struct_error("overflow in item count");
        } else {
            raise_struct_error("repeat count given without format specifier");
        }
        return None;
    }
    Some(tokens)
}

/// Number of argument values a parsed format string consumes (pad bytes take
/// none; `s`/`p` take one regardless of width; everything else takes `count`).
fn value_slots(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .map(|t| match t.code {
            'x' => 0,
            's' | 'p' => 1,
            _ => t.count,
        })
        .sum()
}

/// Parse + validate a full format string (prefix + codes). Returns the endian
/// and tokens, or `None` with a pending `struct.error` on any structural fault.
fn parse_checked(fmt_str: &str) -> Option<(Endian, Vec<Token>)> {
    let (endian, mode, rest) = split_prefix(fmt_str);
    let tokens = parse_format(rest, mode)?;
    Some((endian, tokens))
}

/// Like `parse_checked`, but never raises — any pending exception left by the
/// validating parse is cleared. Used by argument-shape heuristics that must
/// not pre-set a struct.error before `pack`/`unpack` re-validate and report it.
fn parse_checked_quiet(fmt_str: &str) -> Option<(Endian, Vec<Token>)> {
    let had_exc = super::super::exception::current_exception_type().is_some();
    let result = parse_checked(fmt_str);
    // If the parse raised (and there was no prior exception), clear it.
    if !had_exc && result.is_none() {
        super::super::exception::mb_clear_exception();
    }
    result
}

/// Packed payload width of a token, ignoring leading alignment padding. `s`/`p`
/// occupy their repeat count as a fixed field width (`s` = N data bytes, `p` =
/// 1 length byte + N-1 data bytes, total N); every other code occupies
/// `count * native_size`.
fn token_width(t: &Token) -> usize {
    match t.code {
        's' | 'p' => t.count,
        _ => t.count * tok_size(t),
    }
}

/// Alignment padding (number of zero bytes) inserted before a token whose first
/// field starts at byte `offset`. Native mode rounds `offset` up to the field's
/// natural alignment; standard mode never pads. `s`/`p`/`c`/`x`/1-byte codes
/// have alignment 1, so they never pad.
fn token_pad(t: &Token, offset: usize) -> usize {
    let align = code_align(t.code, t.mode);
    if align <= 1 {
        return 0;
    }
    let rem = offset % align;
    if rem == 0 {
        0
    } else {
        align - rem
    }
}

/// Total packed size of a parsed token list, including native alignment
/// padding inserted before each aligned field.
fn layout_size(tokens: &[Token]) -> usize {
    let mut offset = 0usize;
    for t in tokens {
        offset += token_pad(t, offset);
        offset += token_width(t);
    }
    offset
}

/// struct.calcsize(fmt) -> int
pub fn mb_struct_calcsize(fmt: MbValue) -> MbValue {
    let fmt = normalize_format(fmt);
    let mut total: usize = 0;
    let mut bad = false;
    with_str(fmt, |s| match parse_checked(s) {
        Some((_endian, tokens)) => {
            total = layout_size(&tokens);
        }
        None => bad = true,
    });
    if bad {
        // A pending struct.error is set; the returned value is ignored by the
        // exception-aware caller. Return 0 as a benign placeholder.
        return MbValue::from_int(0);
    }
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

/// Decode a width-`size` integer from `slice` (which must be at least `size`
/// bytes) into an i64. `signed` selects sign-extension; `endian` selects the
/// byte order. Width is mode-aware (`l`/`L` = 8 bytes in native mode, 4 in
/// standard mode), so the read is driven by `size` rather than the code char.
fn read_int_sized(slice: &[u8], size: usize, signed: bool, endian: Endian) -> i64 {
    // Assemble the value into a u64, most-significant byte handled per endian.
    let mut acc: u64 = 0;
    match endian {
        Endian::Little => {
            for i in (0..size).rev() {
                acc = (acc << 8) | slice[i] as u64;
            }
        }
        Endian::Big => {
            for i in 0..size {
                acc = (acc << 8) | slice[i] as u64;
            }
        }
    }
    if signed && size < 8 {
        // Sign-extend from the top bit of the `size`-byte field.
        let shift = 64 - size * 8;
        ((acc << shift) as i64) >> shift
    } else {
        acc as i64
    }
}

fn arg_as_float(v: MbValue) -> f64 {
    if let Some(f) = v.as_float() {
        return f;
    }
    if let Some(i) = v.as_int() {
        return i as f64;
    }
    if let Some(b) = v.as_bool() {
        return if b { 1.0 } else { 0.0 };
    }
    0.0
}

/// True when `v` is acceptable to a float code (`f`/`d`/`e`): a real Python
/// `int`, `bool`, or `float`. A `str`/`bytes`/other object is rejected.
fn is_floatable(v: MbValue) -> bool {
    v.is_float() || v.is_int() || v.is_bool()
}

/// Extract the integer an integer-format code (`b`/`B`/`h`/.../`P`) requires,
/// or `None` when the argument is not an `int`/`bool`. Floats are rejected:
/// CPython's integer codes raise `struct.error: required argument is not an
/// integer` rather than silently truncating (see test_1530559). Bytes/str
/// values are likewise rejected here (the legacy `c`-code byte coercion is
/// handled separately in the `'c'` arm).
fn arg_as_pyint(v: MbValue) -> Option<i64> {
    if let Some(i) = v.as_int() {
        return Some(i);
    }
    if let Some(b) = v.as_bool() {
        return Some(if b { 1 } else { 0 });
    }
    None
}

/// Inclusive [min, max] range an integer code accepts at a given byte width.
/// Signed codes use two's-complement bounds; unsigned codes are `[0, 2^bits-1]`.
/// The width is passed in (not derived from the code) because `l`/`L` are 4
/// bytes in standard mode but 8 in native mode. Used to reject out-of-range
/// pack values with `struct.error` instead of truncating.
fn int_code_range_sized(code: char, bytes: usize) -> Option<(i128, i128)> {
    let signed = match code {
        'b' | 'h' | 'i' | 'l' | 'q' | 'n' => true,
        'B' | 'H' | 'I' | 'L' | 'Q' | 'N' | 'P' => false,
        _ => return None,
    };
    let bits = bytes * 8;
    if signed {
        let max = (1i128 << (bits - 1)) - 1;
        let min = -(1i128 << (bits - 1));
        Some((min, max))
    } else {
        let max = (1i128 << bits) - 1;
        Some((0, max))
    }
}

/// Decompose a finite, non-zero `x` into `(f, e)` such that `x == f * 2^e`
/// with `0.5 <= f < 1.0` (the C `frexp` contract). Computed from the IEEE bits
/// so subnormals are handled exactly.
fn frexp(x: f64) -> (f64, i32) {
    let bits = x.to_bits();
    let raw_exp = ((bits >> 52) & 0x7FF) as i32;
    if raw_exp == 0 {
        // Subnormal: scale up into the normal range, then recurse.
        let (f, e) = frexp(x * 9007199254740992.0); // x * 2^53
        return (f, e - 53);
    }
    // Force the exponent field to bias-1 (0x3FE) so the mantissa lands in
    // [0.5, 1.0); the true exponent is raw_exp - 1022.
    let new_bits = (bits & 0x800F_FFFF_FFFF_FFFF) | (0x3FE << 52);
    (f64::from_bits(new_bits), raw_exp - 1022)
}

/// Convert an f64 to a 16-bit IEEE-754 half float (`e` code), rounding to
/// nearest-even. Returns `Err(())` when the magnitude overflows the half range
/// (CPython raises OverflowError rather than rounding to infinity). Infinities
/// and NaN pass through unchanged. Mirrors CPython's `PyFloat_Pack2`.
fn f64_to_half(value: f64) -> Result<u16, ()> {
    if value.is_nan() {
        // Quiet NaN with the high mantissa (quiet) bit set.
        return Ok(0x7E00);
    }
    let sign: u16 = if value.is_sign_negative() { 0x8000 } else { 0 };
    if value.is_infinite() {
        return Ok(sign | 0x7C00);
    }
    if value == 0.0 {
        return Ok(sign);
    }

    let x = value.abs();
    let (mut f, mut e) = frexp(x); // x == f * 2^e, 0.5 <= f < 1.0
                                   // Renormalize to 1.0 <= f < 2.0.
    f *= 2.0;
    e -= 1;

    let bits: u32;
    if e >= 16 {
        return Err(()); // too large for the half range
    } else if e < -25 {
        // Smaller than half the smallest subnormal: rounds to signed zero.
        f = 0.0;
        e = 0;
        bits = round_half_even(f * 1024.0);
    } else if e < -14 {
        // Subnormal: scale the significand into the fixed exponent-0 field.
        f = ldexp(f, 14 + e);
        e = 0;
        bits = round_half_even(f * 1024.0);
    } else {
        // Normal: remove the implicit leading 1, bias the exponent.
        e += 15;
        f -= 1.0;
        bits = round_half_even(f * 1024.0);
    }

    // `bits` is in [0, 1024]; a value of 1024 means the rounding carried into
    // the next binade.
    let mut frac = bits;
    let mut exp_field = e;
    if frac == 1024 {
        frac = 0;
        exp_field += 1;
    }
    if exp_field >= 31 {
        return Err(()); // rounding pushed a near-max value to infinity
    }
    Ok(sign | ((exp_field as u16) << 10) | (frac as u16))
}

/// Round a non-negative f64 to the nearest integer, ties to even.
fn round_half_even(x: f64) -> u32 {
    let floor = x.floor();
    let diff = x - floor;
    let mut n = floor as u32;
    if diff > 0.5 {
        n += 1;
    } else if diff == 0.5 && (n & 1) == 1 {
        n += 1;
    }
    n
}

/// C `ldexp`: `f * 2^exp`.
fn ldexp(f: f64, exp: i32) -> f64 {
    f * 2f64.powi(exp)
}

/// Convert a 16-bit IEEE-754 half float to f64.
fn half_to_f64(h: u16) -> f64 {
    let sign = (h >> 15) & 1;
    let exp = (h >> 10) & 0x1F;
    let frac = h & 0x3FF;
    let sign_f = if sign == 1 { -1.0 } else { 1.0 };
    if exp == 0 {
        // Zero or subnormal.
        sign_f * (frac as f64) * 2f64.powi(-24)
    } else if exp == 0x1F {
        if frac == 0 {
            sign_f * f64::INFINITY
        } else {
            f64::NAN
        }
    } else {
        let m = 1.0 + (frac as f64) / 1024.0;
        sign_f * m * 2f64.powi(exp as i32 - 15)
    }
}

/// struct.pack(fmt, *args) -> bytes
pub fn mb_struct_pack(fmt: MbValue, args: &[MbValue]) -> MbValue {
    let fmt = normalize_format(fmt);
    let mut out = Vec::new();
    let mut bad = false;
    with_str(fmt, |fmt_str| {
        let (endian, tokens) = match parse_checked(fmt_str) {
            Some(t) => t,
            None => {
                bad = true;
                return;
            }
        };
        // Argument-count check (CPython: "pack expected N items for packing").
        let needed = value_slots(&tokens);
        if args.len() != needed {
            raise_struct_error(&format!(
                "pack expected {} items for packing (got {})",
                needed,
                args.len()
            ));
            bad = true;
            return;
        }
        let mut ai = 0usize;
        for tok in &tokens {
            // Native-mode alignment: pad with zero bytes so the next field
            // starts on its natural boundary (no-op in standard mode).
            for _ in 0..token_pad(tok, out.len()) {
                out.push(0u8);
            }
            match tok.code {
                'x' => {
                    for _ in 0..tok.count {
                        out.push(0u8);
                    }
                }
                's' => {
                    // Fixed N-byte field, one bytes argument: zero-pad if short,
                    // truncate if long.
                    let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                    ai += 1;
                    let src = bytes_of(v);
                    let n = tok.count;
                    for i in 0..n {
                        out.push(src.get(i).copied().unwrap_or(0));
                    }
                }
                'p' => {
                    // Pascal string: leading length byte (capped at 255 and at
                    // N-1), then up to N-1 data bytes, zero-padded to width N.
                    let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                    ai += 1;
                    let src = bytes_of(v);
                    let n = tok.count;
                    if n == 0 {
                        // CPython stores nothing for a '0p' field.
                    } else {
                        let data_cap = n - 1;
                        let stored = src.len().min(data_cap).min(255);
                        out.push(stored as u8);
                        for i in 0..data_cap {
                            out.push(src.get(i).copied().unwrap_or(0));
                        }
                    }
                }
                'c' => {
                    for _ in 0..tok.count {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        let byte = bytes_of(v).first().copied().unwrap_or(0);
                        out.push(byte);
                    }
                }
                'f' | 'd' | 'e' => {
                    for _ in 0..tok.count {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        if !is_floatable(v) {
                            raise_struct_error("required argument is not a float");
                            bad = true;
                            return;
                        }
                        let f = arg_as_float(v);
                        match tok.code {
                            'f' => {
                                if f.is_finite() && (f as f32).is_infinite() {
                                    raise_overflow_error("float too large to pack with f format");
                                    bad = true;
                                    return;
                                }
                                let bytes = match endian {
                                    Endian::Little => (f as f32).to_le_bytes(),
                                    Endian::Big => (f as f32).to_be_bytes(),
                                };
                                out.extend_from_slice(&bytes);
                            }
                            'd' => {
                                let bytes = match endian {
                                    Endian::Little => f.to_le_bytes(),
                                    Endian::Big => f.to_be_bytes(),
                                };
                                out.extend_from_slice(&bytes);
                            }
                            _ => {
                                // 'e' half float.
                                let h = match f64_to_half(f) {
                                    Ok(h) => h,
                                    Err(()) => {
                                        raise_overflow_error(
                                            "float too large to pack with e format",
                                        );
                                        bad = true;
                                        return;
                                    }
                                };
                                let bytes = match endian {
                                    Endian::Little => h.to_le_bytes(),
                                    Endian::Big => h.to_be_bytes(),
                                };
                                out.extend_from_slice(&bytes);
                            }
                        }
                    }
                }
                '?' => {
                    for _ in 0..tok.count {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        let truthy = super::super::builtins::mb_is_truthy(v) != 0;
                        out.push(if truthy { 1 } else { 0 });
                    }
                }
                _ => {
                    // Integer codes b/B/h/H/i/I/l/L/q/Q/n/N/P.
                    let size = tok_size(tok);
                    let (min, max) = int_code_range_sized(tok.code, size)
                        .unwrap_or((i64::MIN as i128, i64::MAX as i128));
                    for _ in 0..tok.count {
                        let v = args.get(ai).copied().unwrap_or_else(MbValue::none);
                        ai += 1;
                        let n = match arg_as_pyint(v) {
                            Some(n) => n,
                            None => {
                                raise_struct_error("required argument is not an integer");
                                bad = true;
                                return;
                            }
                        };
                        if (n as i128) < min || (n as i128) > max {
                            raise_struct_error(&format!(
                                "'{}' format requires {} <= number <= {}",
                                tok.code, min, max
                            ));
                            bad = true;
                            return;
                        }
                        match endian {
                            Endian::Little => write_int_le(&mut out, n, size),
                            Endian::Big => write_int_be(&mut out, n, size),
                        }
                    }
                }
            }
        }
    });
    if bad {
        // A pending struct.error is set; result is ignored by the
        // exception-aware caller. Return empty bytes as a placeholder.
        return MbValue::from_ptr(MbObject::new_bytes(Vec::new()));
    }
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// Extract the raw bytes backing a bytes/bytearray/str value, for the `c`,
/// `s`, and `p` codes. A `str` argument is UTF-8 encoded (CPython actually
/// rejects str for `s`, but our porters only ever pass bytes; encoding keeps
/// any stray str case lossless). Non-byte-like values yield an empty slice.
fn bytes_of(v: MbValue) -> Vec<u8> {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => return b.clone(),
                ObjData::ByteArray(lock) => return lock.read().unwrap().clone(),
                ObjData::Str(s) => return s.as_bytes().to_vec(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// Decode `bytes` into `values` according to the parsed format tokens.
/// Pulled out of `mb_struct_unpack` so the borrowed-bytes fast path
/// and the owned-bytes slow path can share the same decoder body
/// without cloning the input buffer.
fn struct_unpack_into(bytes: &[u8], endian: Endian, tokens: &[Token], values: &mut Vec<MbValue>) {
    let mut offset = 0usize;
    for tok in tokens {
        // Skip native-mode alignment padding before this field (no-op in
        // standard mode).
        offset += token_pad(tok, offset);
        match tok.code {
            'x' => {
                offset += tok.count;
            }
            's' => {
                // One bytes value of width N (the repeat count).
                let n = tok.count;
                let end = (offset + n).min(bytes.len());
                let field = bytes.get(offset..end).unwrap_or(&[]).to_vec();
                values.push(MbValue::from_ptr(MbObject::new_bytes(field)));
                offset += n;
            }
            'p' => {
                // Pascal string: leading length byte (clamped to N-1), then data.
                let n = tok.count;
                if n > 0 {
                    let len_byte = bytes.get(offset).copied().unwrap_or(0) as usize;
                    let avail = n - 1;
                    let take = len_byte.min(avail);
                    let start = offset + 1;
                    let end = (start + take).min(bytes.len());
                    let field = bytes.get(start..end).unwrap_or(&[]).to_vec();
                    values.push(MbValue::from_ptr(MbObject::new_bytes(field)));
                } else {
                    values.push(MbValue::from_ptr(MbObject::new_bytes(Vec::new())));
                }
                offset += n;
            }
            _ => {
                let size = tok_size(tok);
                for _ in 0..tok.count {
                    if offset + size > bytes.len() {
                        return;
                    }
                    let slice = &bytes[offset..offset + size];
                    match tok.code {
                        'c' => values.push(MbValue::from_ptr(MbObject::new_bytes(vec![slice[0]]))),
                        'e' => {
                            let h = match endian {
                                Endian::Little => u16::from_le_bytes([slice[0], slice[1]]),
                                Endian::Big => u16::from_be_bytes([slice[0], slice[1]]),
                            };
                            values.push(MbValue::from_float(half_to_f64(h)));
                        }
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
                            // Width-driven integer decode (mode-aware: `l`/`L`
                            // are 8 bytes in native mode, 4 in standard mode).
                            let signed = matches!(tok.code, 'b' | 'h' | 'i' | 'l' | 'q' | 'n');
                            let n = read_int_sized(slice, size, signed, endian);
                            values.push(MbValue::from_int(n));
                        }
                    }
                    offset += size;
                }
            }
        }
    }
}

/// struct.unpack(fmt, data) -> tuple
///
/// CPython requires `len(data) == calcsize(fmt)` exactly; a buffer of the
/// wrong size raises `struct.error`, and a non-buffer object raises
/// `TypeError`. Accepts any bytes-like object (bytes / bytearray /
/// memoryview / `array('b'|'B')`).
pub fn mb_struct_unpack(fmt: MbValue, data: MbValue) -> MbValue {
    let fmt = normalize_format(fmt);
    let mut values: Vec<MbValue> = Vec::new();
    let mut bad = false;
    with_str(fmt, |fmt_str| {
        let (endian, tokens) = match parse_checked(fmt_str) {
            Some(t) => t,
            None => {
                bad = true;
                return;
            }
        };
        // Buffer size must match the full native layout (including alignment
        // padding), not just the summed field widths.
        let expected: usize = layout_size(&tokens);

        // A non-buffer argument (int, None, ...) is a TypeError, not a size
        // mismatch.
        let Some(buf) = buffer_bytes(data) else {
            raise_type_error("a bytes-like object is required, not a non-buffer");
            bad = true;
            return;
        };

        if buf.len() != expected {
            raise_struct_error(&format!("unpack requires a buffer of {} bytes", expected));
            bad = true;
            return;
        }
        struct_unpack_into(&buf, endian, &tokens, &mut values);
    });

    if bad {
        // A pending struct.error / TypeError is set; the empty tuple is
        // ignored by the exception-aware caller.
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    }
    MbValue::from_ptr(MbObject::new_tuple(values))
}

/// Return the raw bytes backing any bytes-like object, or `None` for a
/// non-buffer. Handles bytes / bytearray, memoryview Instances and dict-shaped
/// arrays (via `try_bytes_like`), and `array.array(...)` int-handles of any
/// typecode (via `mb_array_tobytes`, which yields the little-endian byte image
/// CPython's buffer protocol exposes). Distinguishes "not a buffer at all"
/// (TypeError) from an empty buffer.
fn buffer_bytes(data: MbValue) -> Option<Vec<u8>> {
    // `array.array(...)` instances are stored as int-tagged handles, so they
    // have no MbObject pointer; resolve them through the array runtime's
    // buffer image before the pointer-based bytes-like paths.
    if let Some(id) = data.as_int() {
        if super::array_mod::is_array_handle(id as u64) {
            let bytes_val = super::array_mod::mb_array_tobytes(data);
            if let Some(ptr) = bytes_val.as_ptr() {
                unsafe {
                    if let ObjData::Bytes(ref b) = (*ptr).data {
                        return Some(b.clone());
                    }
                }
            }
            return Some(Vec::new());
        }
    }
    let ptr = data.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => super::super::builtins::try_bytes_like(data),
        }
    }
}

/// Extract a byte buffer from any bytes-like object, defaulting to empty for
/// non-buffers (used by the offset-based helpers that clip rather than raise).
fn extract_bytes(data: MbValue) -> Vec<u8> {
    buffer_bytes(data).unwrap_or_default()
}

/// Normalize a format operand to a str MbValue. A `bytes`/`bytearray` format
/// is decoded as ASCII/UTF-8 so `Struct(fmt.encode()).format == fmt`. A str
/// (or anything else) is returned unchanged.
fn normalize_format(fmt: MbValue) -> MbValue {
    if let Some(ptr) = fmt.as_ptr() {
        unsafe {
            let decoded = match &(*ptr).data {
                ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
                ObjData::ByteArray(lock) => {
                    Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
                }
                _ => None,
            };
            if let Some(s) = decoded {
                return MbValue::from_ptr(MbObject::new_str(s));
            }
        }
    }
    fmt
}

/// Build a `functools.partial`-shaped Instance that binds `bound` as the
/// leading positional argument(s) of the native dispatcher at `addr`. When the
/// instance-method call protocol invokes this field with the user's call args,
/// `mb_call_spread` prepends `bound` and dispatches `addr(bound..., *call_args)`.
///
/// This is how a `Struct(fmt)` instance exposes `s.pack(...)` / `s.unpack(...)`
/// / `s.pack_into(...)` / `s.unpack_from(...)` / `s.iter_unpack(...)` as bound
/// methods: each method field is a partial that binds the compiled format
/// string so the module-level dispatcher receives `(fmt, *user_args)`.
fn bound_method(addr: usize, bound: Vec<MbValue>) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert("func".to_string(), MbValue::from_func(addr));
    fields.insert(
        "args".to_string(),
        MbValue::from_ptr(MbObject::new_list(bound)),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "functools.partial".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Populate a Struct instance's `fields` for a (re)compiled `format`.
/// Sets `format` + `size` and wires the bound pack/unpack/pack_into/
/// unpack_from/iter_unpack methods. Used by `mb_struct_new` and by the
/// `__init__` reinit path so re-running `__init__` rebinds every method to
/// the new format.
fn populate_struct_fields(fields: &mut FxHashMap<String, MbValue>, fmt: MbValue) {
    let size = mb_struct_calcsize(fmt);
    let format_copy = with_str(fmt, |s| s.to_string());
    let fmt_val = MbValue::from_ptr(MbObject::new_str(format_copy));
    fields.insert("format".to_string(), fmt_val);
    fields.insert("size".to_string(), size);
    // Each bound method re-resolves the format from a fresh str value so the
    // partial owns its own copy (the instance `format` field can be mutated).
    let fmt_for = || MbValue::from_ptr(MbObject::new_str(with_str(fmt, |s| s.to_string())));
    fields.insert(
        "pack".to_string(),
        bound_method(dispatch_pack as usize, vec![fmt_for()]),
    );
    fields.insert(
        "unpack".to_string(),
        bound_method(dispatch_unpack as usize, vec![fmt_for()]),
    );
    fields.insert(
        "pack_into".to_string(),
        bound_method(dispatch_pack_into as usize, vec![fmt_for()]),
    );
    fields.insert(
        "unpack_from".to_string(),
        bound_method(dispatch_unpack_from as usize, vec![fmt_for()]),
    );
    fields.insert(
        "iter_unpack".to_string(),
        bound_method(dispatch_iter_unpack as usize, vec![fmt_for()]),
    );
}

/// struct.Struct(format) -> Struct instance
///
/// Returns an Instance with `format` + `size` fields and bound pack/unpack/
/// pack_into/unpack_from/iter_unpack methods (each a `functools.partial` that
/// binds the compiled format to the module-level dispatcher). This makes
/// `s = struct.Struct("i"); s.pack(1)` work as in CPython.
pub fn mb_struct_new(fmt: MbValue) -> MbValue {
    // A bytes format (e.g. `Struct(other.format.encode())`) normalizes back to
    // the same str: decode it to a str MbValue before recording `.format`.
    let fmt = normalize_format(fmt);
    let mut fields = FxHashMap::default();
    populate_struct_fields(&mut fields, fmt);
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
    let this = MbValue::from_ptr(Box::into_raw(obj));
    attach_struct_init(this);
    this
}

/// Wire the bound `__init__` method onto a Struct (or subclass) instance. The
/// method is a partial that binds the instance itself, so a later
/// `inst.__init__(fmt)` dispatches `dispatch_struct_init(inst, fmt)` and
/// retargets the instance in place.
///
/// Binding self forms an instance↔partial reference cycle: `this` retains the
/// partial (as the `__init__` field) and the partial retains `this` (as its
/// bound arg). Mamba's refcounting does not collect cycles, so this leaks the
/// instance. That is the safe trade-off here — the alternative (a non-owning
/// borrowed self pointer) risks a use-after-free if the partial outlives the
/// instance — and Struct instances are short-lived in practice.
fn attach_struct_init(this: MbValue) {
    if let Some(ptr) = this.as_ptr() {
        unsafe {
            // Retain for the bound self stored in the partial's arg list, so the
            // refcount stays balanced when the list is eventually dropped.
            super::super::rc::retain_if_ptr(this);
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let init = bound_method(dispatch_struct_init as usize, vec![this]);
                fields.write().unwrap().insert("__init__".to_string(), init);
            }
        }
    }
}

/// struct.iter_unpack(fmt, buffer) -> iterator of tuples
///
/// Materializes the records eagerly into a list, then wraps it in a real
/// list-iterator (via `mb_iter`) so callers can drive it with `next(it)` and
/// hit `StopIteration` at the end — matching CPython's lazy unpack-iterator
/// for the `next()`/`for` access patterns the fixtures exercise.
pub fn mb_struct_iter_unpack(fmt: MbValue, buffer: MbValue) -> MbValue {
    let chunk_size = mb_struct_calcsize(fmt).as_int().unwrap_or(0) as usize;
    let bytes = extract_bytes(buffer);
    let mut results: Vec<MbValue> = Vec::new();
    if chunk_size != 0 {
        let mut off = 0usize;
        while off + chunk_size <= bytes.len() {
            let chunk = bytes[off..off + chunk_size].to_vec();
            let chunk_val = MbValue::from_ptr(MbObject::new_bytes(chunk));
            results.push(mb_struct_unpack(fmt, chunk_val));
            off += chunk_size;
        }
    }
    let list = MbValue::from_ptr(MbObject::new_list(results));
    super::super::iter::mb_iter(list)
}

/// struct.pack_into(fmt, buffer, offset, *args)
///
/// Packs `args` per `fmt` and writes the resulting bytes into `buffer`
/// (a bytearray) starting at `offset`. Returns `None`. A field that does
/// not fit raises `struct.error` with a CPython-shaped message; the format
/// itself is validated first (a bad format raises before any write).
pub fn mb_struct_pack_into(
    fmt: MbValue,
    buffer: MbValue,
    offset: MbValue,
    args: &[MbValue],
) -> MbValue {
    // Pack first; a bad format or value leaves a pending struct.error.
    let packed = mb_struct_pack(fmt, args);
    if super::super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    let packed_bytes: Vec<u8> = packed
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => b.clone(),
                _ => Vec::new(),
            }
        })
        .unwrap_or_default();
    let size = packed_bytes.len();
    let off = offset.as_int().unwrap_or(0);

    if let Some(ptr) = buffer.as_ptr() {
        unsafe {
            if let ObjData::ByteArray(ref lock) = (*ptr).data {
                let mut buf = lock.write().unwrap();
                let buflen = buf.len() as i64;
                // Negative offset: dedicated CPython messages.
                if off < 0 {
                    // An offset that resolves before the start of the buffer
                    // is "out of range" (CPython checks this first).
                    if -off > buflen {
                        raise_struct_error(&format!(
                            "offset {} out of range for {}-byte buffer",
                            off,
                            buf.len()
                        ));
                        return MbValue::none();
                    }
                    let start = (buflen + off) as usize;
                    // The field must fit between `start` and the buffer end.
                    if start + size > buf.len() {
                        raise_struct_error(&format!(
                            "no space to pack {} bytes at offset {}",
                            size, off
                        ));
                        return MbValue::none();
                    }
                    for (i, b) in packed_bytes.iter().enumerate() {
                        buf[start + i] = *b;
                    }
                    return MbValue::none();
                }
                // Non-negative offset: the buffer must hold offset + size bytes.
                if off + size as i64 > buflen {
                    raise_struct_error(&format!(
                        "pack_into requires a buffer of at least {} bytes for \
                         packing {} bytes at offset {} (actual buffer size is {})",
                        off as usize + size,
                        size,
                        off,
                        buf.len()
                    ));
                    return MbValue::none();
                }
                let start = off as usize;
                for (i, b) in packed_bytes.iter().enumerate() {
                    buf[start + i] = *b;
                }
            }
        }
    }
    MbValue::none()
}

/// struct.unpack_from(fmt, buffer, offset=0) -> tuple
///
/// Like `unpack`, but reads exactly `calcsize(fmt)` bytes starting at
/// `offset`. A buffer too small for `offset + size` raises `struct.error`
/// with a CPython-shaped message that quotes the required size, the count,
/// and the offset.
pub fn mb_struct_unpack_from(fmt: MbValue, buffer: MbValue, offset: MbValue) -> MbValue {
    // Resolve the format size first; a bad format leaves a pending
    // struct.error and short-circuits.
    let size_val = mb_struct_calcsize(fmt);
    if super::super::exception::current_exception_type().is_some() {
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    }
    let size = size_val.as_int().unwrap_or(0) as usize;

    let Some(bytes) = buffer_bytes(buffer) else {
        raise_type_error("a bytes-like object is required, not a non-buffer");
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    };
    let off = offset.as_int().unwrap_or(0);
    let buflen = bytes.len() as i64;
    if off < 0 {
        // Negative offset that resolves before the buffer start is "out of
        // range"; one that resolves in-buffer but overruns the end is "not
        // enough data" (distinct CPython messages from the positive case).
        if -off > buflen {
            raise_struct_error(&format!(
                "offset {} out of range for {}-byte buffer",
                off,
                bytes.len()
            ));
            return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
        }
        let abs_off = buflen + off;
        if abs_off + size as i64 > buflen {
            raise_struct_error(&format!(
                "not enough data to unpack {} bytes at offset {}",
                size, off
            ));
            return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
        }
        let start = abs_off as usize;
        let sliced = bytes[start..start + size].to_vec();
        let sliced_val = MbValue::from_ptr(MbObject::new_bytes(sliced));
        return mb_struct_unpack(fmt, sliced_val);
    }
    let abs_off = off;
    if abs_off + size as i64 > buflen {
        raise_struct_error(&format!(
            "unpack_from requires a buffer of at least {} bytes for unpacking \
             {} bytes at offset {} (actual buffer size is {})",
            off as usize + size,
            size,
            off,
            bytes.len()
        ));
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    }
    let start = abs_off as usize;
    let sliced = bytes[start..start + size].to_vec();
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

    use super::super::super::exception::{current_exception_type, mb_clear_exception};

    #[test]
    fn test_calcsize_bad_char_raises_struct_error() {
        // CPython 3.12: struct.calcsize('Z') raises
        // struct.error("bad char in struct format"). The pending exception
        // type name must be "error" (== struct.error.__name__) so
        // `assertRaises(struct.error, ...)` matches.
        mb_clear_exception();
        let _ = mb_struct_calcsize(s("Z"));
        assert_eq!(current_exception_type(), Some("error".to_string()));
        mb_clear_exception();
    }

    #[test]
    fn test_calcsize_valid_format_no_exception() {
        // A fully-valid format must NOT leave a pending exception.
        mb_clear_exception();
        assert_eq!(mb_struct_calcsize(s("8B")).as_int(), Some(8));
        assert_eq!(current_exception_type(), None);
        mb_clear_exception();
    }

    #[test]
    fn test_pack_unpack_bad_char_raises_struct_error() {
        mb_clear_exception();
        let _ = mb_struct_pack(s("Z"), &[MbValue::from_int(1)]);
        assert_eq!(current_exception_type(), Some("error".to_string()));
        mb_clear_exception();
        let data = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let _ = mb_struct_unpack(s("Z"), data);
        assert_eq!(current_exception_type(), Some("error".to_string()));
        mb_clear_exception();
    }

    #[test]
    fn test_calcsize_basic() {
        assert_eq!(mb_struct_calcsize(s("i")).as_int(), Some(4));
        assert_eq!(mb_struct_calcsize(s("ii")).as_int(), Some(8));
        // Native mode (no prefix) aligns each field, so "bhi" is
        // b@0(1) + pad@1 + h@2(2) + i@4(4) = 8 (matches CPython 3.12).
        assert_eq!(mb_struct_calcsize(s("bhi")).as_int(), Some(8));
        // Standard mode is tightly packed: 1 + 2 + 4 = 7.
        assert_eq!(mb_struct_calcsize(s("=bhi")).as_int(), Some(7));
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
        use super::super::super::iter::mb_next;
        // Pack three i values back-to-back, then iter_unpack with "i". The
        // result is a real iterator (CPython returns an unpack-iterator, not a
        // list); drive it with mb_next and collect the per-record tuples.
        let args = vec![
            MbValue::from_int(10),
            MbValue::from_int(20),
            MbValue::from_int(30),
        ];
        let packed = mb_struct_pack(s("iii"), &args);
        let it = mb_struct_iter_unpack(s("i"), packed);
        let mut got = Vec::new();
        for _ in 0..3 {
            let t = mb_next(it);
            unsafe {
                if let ObjData::Tuple(ref items) = (*t.as_ptr().unwrap()).data {
                    got.push(items[0].as_int().unwrap());
                } else {
                    panic!("expected Tuple");
                }
            }
        }
        assert_eq!(got, vec![10, 20, 30]);
    }

    #[test]
    fn test_iter_unpack_empty_buffer() {
        use super::super::super::iter::mb_next;
        // An empty buffer yields no records: the very first mb_next is the
        // exhausted (None) sentinel.
        let empty = MbValue::from_ptr(MbObject::new_bytes(vec![]));
        let it = mb_struct_iter_unpack(s("i"), empty);
        assert!(mb_next(it).is_none());
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
