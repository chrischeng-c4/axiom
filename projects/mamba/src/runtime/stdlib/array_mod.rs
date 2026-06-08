//! array module for Mamba (#451 / #1265 Task #35).
//!
//! Typed numeric containers with bulk-bytes hot path (`frombytes` / `tobytes`).
//! Wave-1 收尾 lib — integer-handle pattern (per
//! `project_mamba_integer_handle_pattern`), backed by typed `Vec<T>` storage
//! per typecode. No external crate.
//!
//! **Supported typecodes** (subset of CPython surface):
//!   - 'b' i8, 'B' u8
//!   - 'h' i16, 'H' u16
//!   - 'i' i32, 'I' u32
//!   - 'l' i64, 'L' u64
//!   - 'q' i64, 'Q' u64
//!   - 'f' f32, 'd' f64
//!
//! **Out of scope** (queued for sweep): pickle, `__deepcopy__`, `fromfile` /
//! `tofile`, `tounicode` / `fromunicode` ('u' / 'w' typecodes), `__buffer__`.
//!
//! **ABI**: flat-args `extern "C" fn(args_ptr, nargs) -> MbValue` matching
//! the post-`ebba01e9a` convention (see
//! `project_mamba_runtime_correctness_gaps_2026_05_13`).
//!
//! HANDWRITE-BEGIN reason: stdlib shim layer for force-typed module dispatch.
//! Will be regenerated once score-standardize learns
//! `section_type = "stdlib-oop-module"` with the integer-handle DSL.
//! HANDWRITE-END
//!
//! @codegen-skip: handwrite-pre-standardize

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// Soft cap on resident array handles per thread. When the table exceeds
/// this size, `make_handle` evicts the oldest entries (FIFO) until back
/// under cap. See #2181 — without this, `ARRAYS` accumulated monotonically
/// across bench iterations because handles are tagged integers (no Drop
/// hook on the value side).
const ARRAY_SOFT_CAP: usize = 1024;

/// Typed storage. The variant tag IS the typecode; itemsize is implicit in
/// the variant's element type.
#[derive(Clone)]
enum ArrayStore {
    I8(Vec<i8>),
    U8(Vec<u8>),
    Unicode(Vec<char>),
    I16(Vec<i16>),
    U16(Vec<u16>),
    I32(Vec<i32>),
    U32(Vec<u32>),
    I64(Vec<i64>),
    U64(Vec<u64>),
    F32(Vec<f32>),
    F64(Vec<f64>),
}

impl ArrayStore {
    fn new(typecode: char) -> Self {
        match typecode {
            'b' => Self::I8(Vec::new()),
            'B' => Self::U8(Vec::new()),
            'u' => Self::Unicode(Vec::new()),
            'h' => Self::I16(Vec::new()),
            'H' => Self::U16(Vec::new()),
            'i' => Self::I32(Vec::new()),
            'I' => Self::U32(Vec::new()),
            'l' | 'q' => Self::I64(Vec::new()),
            'L' | 'Q' => Self::U64(Vec::new()),
            'f' => Self::F32(Vec::new()),
            // 'd' and any unknown → f64 (CPython would raise; we degrade to
            // double, matching the legacy stub's "default to 'd'" behaviour).
            _ => Self::F64(Vec::new()),
        }
    }

    fn typecode(&self) -> char {
        match self {
            Self::I8(_) => 'b',
            Self::U8(_) => 'B',
            Self::Unicode(_) => 'u',
            Self::I16(_) => 'h',
            Self::U16(_) => 'H',
            Self::I32(_) => 'i',
            Self::U32(_) => 'I',
            // 'l' and 'q' both map to I64; canonical surface report uses 'l'
            // (matches CPython 3.12 on macOS aarch64 where `sizeof(long) == 8`).
            Self::I64(_) => 'l',
            Self::U64(_) => 'L',
            Self::F32(_) => 'f',
            Self::F64(_) => 'd',
        }
    }

    fn itemsize(&self) -> i64 {
        match self {
            Self::I8(_) | Self::U8(_) => 1,
            Self::Unicode(_) => 4,
            Self::I16(_) | Self::U16(_) => 2,
            Self::I32(_) | Self::U32(_) | Self::F32(_) => 4,
            Self::I64(_) | Self::U64(_) | Self::F64(_) => 8,
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::I8(v) => v.len(),
            Self::U8(v) => v.len(),
            Self::Unicode(v) => v.len(),
            Self::I16(v) => v.len(),
            Self::U16(v) => v.len(),
            Self::I32(v) => v.len(),
            Self::U32(v) => v.len(),
            Self::I64(v) => v.len(),
            Self::U64(v) => v.len(),
            Self::F32(v) => v.len(),
            Self::F64(v) => v.len(),
        }
    }

    /// Push a single MbValue, enforcing CPython's scalar boundary for numeric
    /// arrays. Integer arrays reject floats/non-numbers and range-check before
    /// storing so failed appends do not silently no-op or wrap.
    fn push_value(&mut self, v: MbValue) -> bool {
        match self {
            Self::I8(buf) => push_int_checked(buf, v, i8::MIN as i64, i8::MAX as i64, |i| i as i8),
            Self::U8(buf) => push_int_checked(buf, v, 0, u8::MAX as i64, |i| i as u8),
            Self::Unicode(buf) => push_unicode_checked(buf, v),
            Self::I16(buf) => {
                push_int_checked(buf, v, i16::MIN as i64, i16::MAX as i64, |i| i as i16)
            }
            Self::U16(buf) => push_int_checked(buf, v, 0, u16::MAX as i64, |i| i as u16),
            Self::I32(buf) => {
                push_int_checked(buf, v, i32::MIN as i64, i32::MAX as i64, |i| i as i32)
            }
            Self::U32(buf) => push_int_checked(buf, v, 0, u32::MAX as i64, |i| i as u32),
            Self::I64(buf) => push_int_checked(buf, v, i64::MIN, i64::MAX, |i| i),
            Self::U64(buf) => push_int_checked(buf, v, 0, i64::MAX, |i| i as u64),
            Self::F32(buf) => push_float_checked(buf, v, |f| f as f32),
            Self::F64(buf) => push_float_checked(buf, v, |f| f),
        }
    }

    fn get_as_value(&self, idx: usize) -> MbValue {
        match self {
            Self::I8(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::U8(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::Unicode(v) => v
                .get(idx)
                .map_or(MbValue::none(), |x| MbValue::from_ptr(MbObject::new_str(x.to_string()))),
            Self::I16(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::U16(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::I32(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::U32(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::I64(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x)),
            Self::U64(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_int(*x as i64)),
            Self::F32(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_float(*x as f64)),
            Self::F64(v) => v.get(idx).map_or(MbValue::none(), |x| MbValue::from_float(*x)),
        }
    }

    /// Append every element as an MbValue list. Materialises a Python list,
    /// allocation-bound regime per `project_mamba_re_findall_allocation_bound`
    /// — `tolist` is NOT on the bulk-bytes hot path; the bench fixture should
    /// stay on `frombytes`/`tobytes`.
    fn to_list(&self) -> Vec<MbValue> {
        let n = self.len();
        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            out.push(self.get_as_value(i));
        }
        out
    }

    /// `tobytes`: little-endian raw byte image (matches CPython on every
    /// platform mamba supports today — aarch64-darwin, x86_64-linux, both LE).
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::I8(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::U8(v) => v.clone(),
            Self::Unicode(v) => v.iter().flat_map(|x| (*x as u32).to_le_bytes()).collect(),
            Self::I16(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::U16(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::I32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::U32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::I64(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::U64(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::F32(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
            Self::F64(v) => v.iter().flat_map(|x| x.to_le_bytes()).collect(),
        }
    }

    /// `frombytes`: extend the typed buffer with elements decoded from a
    /// little-endian byte image.
    fn extend_from_bytes(&mut self, b: &[u8]) {
        let isz = self.itemsize() as usize;
        let n = b.len() / isz;
        match self {
            Self::I8(v) => v.extend(b.iter().take(n).map(|&x| x as i8)),
            Self::U8(v) => v.extend_from_slice(&b[..n]),
            Self::Unicode(v) => for i in 0..n {
                let s = i*4;
                let code = u32::from_le_bytes([b[s], b[s+1], b[s+2], b[s+3]]);
                if let Some(ch) = char::from_u32(code) {
                    v.push(ch);
                }
            },
            Self::I16(v) => for i in 0..n {
                v.push(i16::from_le_bytes([b[i*2], b[i*2+1]]));
            },
            Self::U16(v) => for i in 0..n {
                v.push(u16::from_le_bytes([b[i*2], b[i*2+1]]));
            },
            Self::I32(v) => for i in 0..n {
                v.push(i32::from_le_bytes([b[i*4], b[i*4+1], b[i*4+2], b[i*4+3]]));
            },
            Self::U32(v) => for i in 0..n {
                v.push(u32::from_le_bytes([b[i*4], b[i*4+1], b[i*4+2], b[i*4+3]]));
            },
            Self::I64(v) => for i in 0..n {
                let s = i*8;
                v.push(i64::from_le_bytes([
                    b[s], b[s+1], b[s+2], b[s+3], b[s+4], b[s+5], b[s+6], b[s+7],
                ]));
            },
            Self::U64(v) => for i in 0..n {
                let s = i*8;
                v.push(u64::from_le_bytes([
                    b[s], b[s+1], b[s+2], b[s+3], b[s+4], b[s+5], b[s+6], b[s+7],
                ]));
            },
            Self::F32(v) => for i in 0..n {
                v.push(f32::from_le_bytes([b[i*4], b[i*4+1], b[i*4+2], b[i*4+3]]));
            },
            Self::F64(v) => for i in 0..n {
                let s = i*8;
                v.push(f64::from_le_bytes([
                    b[s], b[s+1], b[s+2], b[s+3], b[s+4], b[s+5], b[s+6], b[s+7],
                ]));
            },
        }
    }

    fn byteswap(&mut self) {
        match self {
            Self::I8(_) | Self::U8(_) => {} // 1-byte items: no-op
            Self::Unicode(v) => {
                for ch in v.iter_mut() {
                    if let Some(swapped) = char::from_u32((*ch as u32).swap_bytes()) {
                        *ch = swapped;
                    }
                }
            }
            Self::I16(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::U16(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::I32(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::U32(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::I64(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::U64(v) => for x in v.iter_mut() { *x = x.swap_bytes(); },
            Self::F32(v) => for x in v.iter_mut() {
                *x = f32::from_bits(x.to_bits().swap_bytes());
            },
            Self::F64(v) => for x in v.iter_mut() {
                *x = f64::from_bits(x.to_bits().swap_bytes());
            },
        }
    }
}

fn push_int_checked<T>(
    out: &mut Vec<T>,
    value: MbValue,
    min: i64,
    max: i64,
    cast: impl FnOnce(i64) -> T,
) -> bool {
    let Some(i) = value.as_int_pyint() else {
        raise_type_error("integer argument expected");
        return false;
    };
    if i < min || i > max {
        raise_overflow_error("signed integer is greater than maximum");
        return false;
    }
    out.push(cast(i));
    true
}

fn push_float_checked<T>(
    out: &mut Vec<T>,
    value: MbValue,
    cast: impl FnOnce(f64) -> T,
) -> bool {
    let f = if let Some(f) = value.as_float() {
        f
    } else if let Some(i) = value.as_int_pyint() {
        i as f64
    } else {
        raise_type_error("must be real number, not non-number");
        return false;
    };
    out.push(cast(f));
    true
}

fn push_unicode_checked(out: &mut Vec<char>, value: MbValue) -> bool {
    if let Some(ptr) = value.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                let mut chars = s.chars();
                if let (Some(ch), None) = (chars.next(), chars.next()) {
                    out.push(ch);
                    return true;
                }
            }
        }
    }
    raise_type_error("array item must be unicode character");
    false
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn raise_overflow_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
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

fn raise_index_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

thread_local! {
    static ARRAYS: RefCell<HashMap<u64, ArrayStore>> = RefCell::new(HashMap::new());
    static ARRAY_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static ARRAY_TYPECODES: RefCell<HashMap<u64, char>> = RefCell::new(HashMap::new());
    /// Insertion-order queue for FIFO eviction once `ARRAY_SOFT_CAP` is
    /// exceeded. Kept in sync with `ARRAYS` / `ARRAY_IDS` by
    /// `make_handle` and `drop_handle`. #2181.
    static ARRAY_ORDER: RefCell<VecDeque<u64>> = RefCell::new(VecDeque::new());
    /// Per-handle refcount. Incremented by `mb_retain_value` on alias
    /// (`b = a`), decremented by `mb_release_value` on rebind / scope
    /// exit. Drops the `ARRAYS` entry when the count hits zero. #2111.
    static ARRAY_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
    /// Handle IDs start above `HANDLE_MIN_ID` (= 2^40) to avoid colliding
    /// with primitive `int` values that JIT-compiled code creates and
    /// releases constantly. See `integer_handle_registry::HANDLE_MIN_ID`.
    static NEXT_ARRAY_ID: Cell<u64> = const { Cell::new(ARRAY_HANDLE_BASE) };
}

/// Base = 1<<44. Sits above queue (1<<43) and leaves the [1<<44, 1<<45)
/// range (~17.6 trillion ids) for array. Bases for other handle-pattern
/// modules follow the same pattern. The MbValue NaN-box caps integers at
/// (1<<47) - 1, so the per-module base must stay below 1<<47.
const ARRAY_HANDLE_BASE: u64 = 1u64 << 44;

fn alloc_id() -> u64 {
    NEXT_ARRAY_ID.with(|c| {
        let id = c.get();
        c.set(id + 1);
        id
    })
}

/// `class.rs` predicate — recognises an integer as an array handle for the
/// `mb_getattr` / `mb_call_method` dispatch branches.
pub fn is_array_handle(id: u64) -> bool {
    ARRAY_IDS.with(|s| s.borrow().contains(&id))
}

/// Remove a handle's backing store. Safe to call with an unknown id
/// (no-op). Public so future class.rs Instance-drop wiring or explicit
/// stdlib finalizers can release resources eagerly. #2181.
pub fn drop_handle(id: u64) {
    let removed_from_map = ARRAYS.with(|m| m.borrow_mut().remove(&id).is_some());
    let removed_from_ids = ARRAY_IDS.with(|s| s.borrow_mut().remove(&id));
    ARRAY_TYPECODES.with(|m| { m.borrow_mut().remove(&id); });
    ARRAY_REFCOUNTS.with(|r| { r.borrow_mut().remove(&id); });
    if removed_from_map || removed_from_ids {
        ARRAY_ORDER.with(|q| {
            let mut q = q.borrow_mut();
            if let Some(pos) = q.iter().position(|&x| x == id) {
                q.remove(pos);
            }
        });
    }
}

/// `mb_retain_value` integer-handle dispatch (#2111). Bumps the per-handle
/// refcount when `id` is a live array handle; returns true on hit.
pub fn retain_handle(id: u64) -> bool {
    if !is_array_handle(id) {
        return false;
    }
    ARRAY_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111). Decrements the
/// per-handle refcount; drops the backing store when the count reaches
/// zero. Returns true on hit so the registry stops at the first owner.
pub fn release_handle(id: u64) -> bool {
    if !is_array_handle(id) {
        return false;
    }
    let should_drop = ARRAY_REFCOUNTS.with(|r| {
        let mut rc_map = r.borrow_mut();
        let rc = rc_map.entry(id).or_insert(1);
        if *rc <= 1 {
            rc_map.remove(&id);
            true
        } else {
            *rc -= 1;
            false
        }
    });
    if should_drop {
        drop_handle(id);
    }
    true
}

/// Test-only view of the resident handle count. #2181 — used by leak
/// regression tests to assert eviction.
#[cfg(test)]
pub(crate) fn handle_table_len() -> usize {
    ARRAYS.with(|m| m.borrow().len())
}

fn evict_until_under_cap() {
    // Evict from the FIFO front until `ARRAYS.len()` is strictly less than
    // `ARRAY_SOFT_CAP`. Caller is responsible for inserting the new entry
    // after this returns; we evict aggressively (`< cap`, not `<= cap`) so
    // there is always headroom for the imminent insert.
    loop {
        let too_many = ARRAYS.with(|m| m.borrow().len() >= ARRAY_SOFT_CAP);
        if !too_many {
            break;
        }
        let victim = ARRAY_ORDER.with(|q| q.borrow_mut().pop_front());
        match victim {
            Some(id) => {
                ARRAYS.with(|m| { m.borrow_mut().remove(&id); });
                ARRAY_IDS.with(|s| { s.borrow_mut().remove(&id); });
                ARRAY_TYPECODES.with(|m| { m.borrow_mut().remove(&id); });
            }
            None => break, // queue empty but map non-empty — desync; bail
        }
    }
}

fn make_handle_from_store(typecode: char, store: ArrayStore) -> MbValue {
    evict_until_under_cap();
    let id = alloc_id();
    ARRAYS.with(|m| { m.borrow_mut().insert(id, store); });
    ARRAY_IDS.with(|s| { s.borrow_mut().insert(id); });
    ARRAY_TYPECODES.with(|m| { m.borrow_mut().insert(id, typecode); });
    ARRAY_ORDER.with(|q| { q.borrow_mut().push_back(id); });
    MbValue::from_int(id as i64)
}

fn make_handle(typecode: char) -> MbValue {
    make_handle_from_store(typecode, ArrayStore::new(typecode))
}

// ── Dispatch wrappers (flat-args ABI) ──

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

dispatch_binary!(dispatch_array_new, mb_array_new);

/// Register the array module.
pub fn register() {
    let mut attrs = HashMap::new();
    let addr = dispatch_array_new as usize;
    attrs.insert("array".to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(addr as u64, "array".to_string());
    });
    // typecodes constant — eagerly evaluated as str (CPython exposes this
    // as a plain str under py<3.15, tuple under >=3.15). Mamba targets 3.12.
    attrs.insert("typecodes".to_string(),
        MbValue::from_ptr(MbObject::new_str("bBuhHiIlLqQfd".to_string())));
    // ArrayType alias (typeshed line 108).
    attrs.insert("ArrayType".to_string(), MbValue::from_func(addr));
    super::register_module("array", attrs);
    // #2111: register retain/release hooks so JIT-emitted rebind releases
    // (`a = make_array(...)` in a hot loop) drop the prior array's backing
    // storage instead of leaking it.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// ── Public surface — called by dispatch thunks AND class.rs method dispatch ──

/// `array.array(typecode, initializer=None)` — constructor returning an
/// integer-handle MbValue.
pub fn mb_array_new(typecode: MbValue, initializer: MbValue) -> MbValue {
    let mut tc = '\0';
    let mut char_count = 0usize;
    with_str(typecode, |s| {
        char_count = s.chars().count();
        tc = s.chars().next().unwrap_or('\0');
    });
    if char_count != 1 {
        return raise_type_error("array() argument 1 must be a unicode character");
    }
    if !"bBuhHiIlLqQfd".contains(tc) {
        return raise_value_error("bad typecode");
    }
    let handle = make_handle(tc);
    if !initializer.is_none() {
        mb_array_init_extend(handle, initializer);
    }
    handle
}

/// Internal init helper: extend the freshly-created handle from an
/// initializer (bytes/bytearray → frombytes path; list/iterable →
/// per-element push). Matches CPython's `array.array(tc, init)` semantics.
fn mb_array_init_extend(handle: MbValue, init: MbValue) {
    if let Some(ptr) = init.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => {
                    let b = b.clone();
                    extend_initializer_bytes_checked(handle, &b);
                }
                ObjData::ByteArray(lock) => {
                    let b = lock.read().unwrap().clone();
                    extend_initializer_bytes_checked(handle, &b);
                }
                ObjData::Str(s) => {
                    if handle_typecode(handle) == Some('u') {
                        extend_unicode_from_str(handle, s);
                    } else {
                        raise_type_error("cannot use a str to initialize an array with typecode other than 'u'");
                    }
                }
                ObjData::List(lock) => {
                    let items = lock.read().unwrap().clone();
                    mutate_store(handle, |s| {
                        for v in items {
                            if !s.push_value(v) {
                                break;
                            }
                        }
                    });
                }
                _ => {}
            }
        }
    }
}

#[inline]
fn with_str<R>(val: MbValue, f: impl FnOnce(&str) -> R) -> R {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data { return f(s.as_str()); }
        }
    }
    f("")
}

fn mutate_store(handle: MbValue, f: impl FnOnce(&mut ArrayStore)) {
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        ARRAYS.with(|m| {
            if let Some(s) = m.borrow_mut().get_mut(&id) { f(s); }
        });
    }
}

fn read_store<R>(handle: MbValue, default: R, f: impl FnOnce(&ArrayStore) -> R) -> R {
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        return ARRAYS.with(|m| {
            m.borrow().get(&id).map(f).unwrap_or(default)
        });
    }
    default
}

fn handle_typecode(handle: MbValue) -> Option<char> {
    handle.as_int().and_then(|id| {
        ARRAY_TYPECODES.with(|m| m.borrow().get(&(id as u64)).copied())
    })
}

fn cloned_store(handle: MbValue) -> Option<ArrayStore> {
    handle.as_int().and_then(|id| {
        ARRAYS.with(|m| m.borrow().get(&(id as u64)).cloned())
    })
}

fn concat_stores(lhs: &ArrayStore, rhs: &ArrayStore) -> Option<ArrayStore> {
    macro_rules! concat_variant {
        ($variant:ident, $a:ident, $b:ident) => {{
            let mut out = $a.clone();
            out.extend_from_slice($b);
            Some(ArrayStore::$variant(out))
        }};
    }
    match (lhs, rhs) {
        (ArrayStore::I8(a), ArrayStore::I8(b)) => concat_variant!(I8, a, b),
        (ArrayStore::U8(a), ArrayStore::U8(b)) => concat_variant!(U8, a, b),
        (ArrayStore::Unicode(a), ArrayStore::Unicode(b)) => concat_variant!(Unicode, a, b),
        (ArrayStore::I16(a), ArrayStore::I16(b)) => concat_variant!(I16, a, b),
        (ArrayStore::U16(a), ArrayStore::U16(b)) => concat_variant!(U16, a, b),
        (ArrayStore::I32(a), ArrayStore::I32(b)) => concat_variant!(I32, a, b),
        (ArrayStore::U32(a), ArrayStore::U32(b)) => concat_variant!(U32, a, b),
        (ArrayStore::I64(a), ArrayStore::I64(b)) => concat_variant!(I64, a, b),
        (ArrayStore::U64(a), ArrayStore::U64(b)) => concat_variant!(U64, a, b),
        (ArrayStore::F32(a), ArrayStore::F32(b)) => concat_variant!(F32, a, b),
        (ArrayStore::F64(a), ArrayStore::F64(b)) => concat_variant!(F64, a, b),
        _ => None,
    }
}

fn repeat_store(store: &ArrayStore, count: usize) -> ArrayStore {
    macro_rules! repeat_variant {
        ($variant:ident, $items:ident) => {{
            let mut out = Vec::with_capacity($items.len() * count);
            for _ in 0..count {
                out.extend_from_slice($items);
            }
            ArrayStore::$variant(out)
        }};
    }
    match store {
        ArrayStore::I8(items) => repeat_variant!(I8, items),
        ArrayStore::U8(items) => repeat_variant!(U8, items),
        ArrayStore::Unicode(items) => repeat_variant!(Unicode, items),
        ArrayStore::I16(items) => repeat_variant!(I16, items),
        ArrayStore::U16(items) => repeat_variant!(U16, items),
        ArrayStore::I32(items) => repeat_variant!(I32, items),
        ArrayStore::U32(items) => repeat_variant!(U32, items),
        ArrayStore::I64(items) => repeat_variant!(I64, items),
        ArrayStore::U64(items) => repeat_variant!(U64, items),
        ArrayStore::F32(items) => repeat_variant!(F32, items),
        ArrayStore::F64(items) => repeat_variant!(F64, items),
    }
}

pub fn mb_array_concat(lhs: MbValue, rhs: MbValue) -> MbValue {
    let Some(lhs_store) = cloned_store(lhs) else {
        return raise_type_error("can only concatenate array to array");
    };
    let Some(rhs_store) = cloned_store(rhs) else {
        return raise_type_error("can only concatenate array to array");
    };
    let lhs_typecode = handle_typecode(lhs).unwrap_or_else(|| lhs_store.typecode());
    let rhs_typecode = handle_typecode(rhs).unwrap_or_else(|| rhs_store.typecode());
    if lhs_typecode != rhs_typecode {
        return raise_type_error("can only append array of same kind");
    }
    let Some(joined) = concat_stores(&lhs_store, &rhs_store) else {
        return raise_type_error("can only append array of same kind");
    };
    make_handle_from_store(lhs_typecode, joined)
}

pub fn mb_array_repeat(handle: MbValue, count: MbValue) -> MbValue {
    let Some(store) = cloned_store(handle) else {
        return raise_type_error("can only repeat array by integer");
    };
    if count.as_int().is_some_and(|id| is_array_handle(id as u64)) {
        return raise_type_error("can't multiply sequence by non-int");
    }
    let Some(n) = count.as_int_pyint() else {
        return raise_type_error("can't multiply sequence by non-int");
    };
    let typecode = handle_typecode(handle).unwrap_or_else(|| store.typecode());
    make_handle_from_store(typecode, repeat_store(&store, n.max(0) as usize))
}

fn extend_initializer_bytes_checked(handle: MbValue, b: &[u8]) -> MbValue {
    let itemsize = read_store(handle, 0usize, |s| s.itemsize() as usize);
    if itemsize == 0 || b.len() % itemsize != 0 {
        return raise_value_error("bytes length not a multiple of item size");
    }
    mutate_store(handle, |s| s.extend_from_bytes(b));
    MbValue::none()
}

fn extend_unicode_from_str(handle: MbValue, text: &str) -> MbValue {
    let updated = RefCell::new(false);
    mutate_store(handle, |s| {
        if let ArrayStore::Unicode(items) = s {
            items.extend(text.chars());
            *updated.borrow_mut() = true;
        }
    });
    if !updated.into_inner() {
        return raise_type_error("fromunicode() may only be called on unicode arrays");
    }
    MbValue::none()
}

/// `a.append(v)`
pub fn mb_array_append(handle: MbValue, v: MbValue) -> MbValue {
    mutate_store(handle, |s| {
        s.push_value(v);
    });
    MbValue::none()
}

/// `a.extend(it)` — accepts list, bytes, bytearray, or another array handle.
pub fn mb_array_extend(handle: MbValue, iterable: MbValue) -> MbValue {
    if let Some(ptr) = iterable.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    let items = lock.read().unwrap().clone();
                    mutate_store(handle, |s| {
                        for v in items {
                            if !s.push_value(v) {
                                break;
                            }
                        }
                    });
                    return MbValue::none();
                }
                ObjData::Bytes(b) => {
                    let b = b.clone();
                    mutate_store(handle, |s| s.extend_from_bytes(&b));
                    return MbValue::none();
                }
                ObjData::ByteArray(lock) => {
                    let b = lock.read().unwrap().clone();
                    mutate_store(handle, |s| s.extend_from_bytes(&b));
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }
    // Another array handle? Copy its elements.
    if let Some(id) = iterable.as_int() {
        let other = id as u64;
        if is_array_handle(other) {
            let items: Vec<MbValue> = ARRAYS.with(|m| {
                m.borrow().get(&other).map(|s| s.to_list()).unwrap_or_default()
            });
            mutate_store(handle, |s| {
                for v in items {
                    if !s.push_value(v) {
                        break;
                    }
                }
            });
        }
    }
    MbValue::none()
}

/// `a.fromlist(lst)` — same shape as `extend` over a Python list.
pub fn mb_array_fromlist(handle: MbValue, list: MbValue) -> MbValue {
    mb_array_extend(handle, list)
}

/// `a.frombytes(buf)` — bulk-bytes hot path. Decodes little-endian bytes
/// into typed elements.
pub fn mb_array_frombytes(handle: MbValue, buf: MbValue) -> MbValue {
    if let Some(ptr) = buf.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => {
                    let b = b.clone();
                    mutate_store(handle, |s| s.extend_from_bytes(&b));
                }
                ObjData::ByteArray(lock) => {
                    let b = lock.read().unwrap().clone();
                    mutate_store(handle, |s| s.extend_from_bytes(&b));
                }
                _ => {}
            }
        }
    }
    MbValue::none()
}

/// `a.fromunicode(s)` — extend a unicode array from Python str characters.
pub fn mb_array_fromunicode(handle: MbValue, text: MbValue) -> MbValue {
    let Some(ptr) = text.as_ptr() else {
        return raise_type_error("fromunicode() argument must be str");
    };
    unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            return extend_unicode_from_str(handle, s);
        }
    }
    raise_type_error("fromunicode() argument must be str")
}

/// `a.tounicode()` — materialise a unicode array as Python str.
pub fn mb_array_tounicode(handle: MbValue) -> MbValue {
    let text = read_store(handle, None, |s| match s {
        ArrayStore::Unicode(items) => Some(items.iter().collect::<String>()),
        _ => None,
    });
    if let Some(text) = text {
        return MbValue::from_ptr(MbObject::new_str(text));
    }
    raise_type_error("tounicode() may only be called on unicode arrays")
}

/// `a.tobytes()` — emits the typed buffer as little-endian bytes. Bulk-bytes
/// hot path; single allocation per call (output Vec<u8> sized exactly to
/// `len * itemsize`).
pub fn mb_array_tobytes(handle: MbValue) -> MbValue {
    let out = read_store(handle, Vec::new(), |s| s.to_bytes());
    MbValue::from_ptr(MbObject::new_bytes(out))
}

/// `a.tolist()` — materialises every element as an MbValue (allocation-bound
/// regime; NOT on the bulk-bytes hot path — fixtures should prefer
/// `tobytes`/`frombytes` for perf measurement per
/// `project_mamba_re_findall_allocation_bound`).
pub fn mb_array_tolist(handle: MbValue) -> MbValue {
    let items = read_store(handle, Vec::new(), |s| s.to_list());
    MbValue::from_ptr(MbObject::new_list(items))
}

/// `a.buffer_info()` — `(addr, length)` tuple. `addr` is the integer handle
/// (CPython returns the real address; we expose the handle ID as a stable
/// proxy — sufficient for non-pointer-arithmetic real-world uses).
pub fn mb_array_buffer_info(handle: MbValue) -> MbValue {
    let len = read_store(handle, 0i64, |s| s.len() as i64);
    let addr = handle.as_int().unwrap_or(0);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(addr),
        MbValue::from_int(len),
    ]))
}

/// `a.byteswap()` — flips byte order of every element in-place.
pub fn mb_array_byteswap(handle: MbValue) -> MbValue {
    mutate_store(handle, |s| s.byteswap());
    MbValue::none()
}

/// `a.count(v)` — number of equal occurrences.
pub fn mb_array_count(handle: MbValue, target: MbValue) -> MbValue {
    let mut count: i64 = 0;
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        ARRAYS.with(|m| {
            if let Some(s) = m.borrow().get(&id) {
                let n = s.len();
                for i in 0..n {
                    if values_equal(s.get_as_value(i), target) {
                        count += 1;
                    }
                }
            }
        });
    }
    MbValue::from_int(count)
}

/// `v in a` — scan elements using the same equality as `count`/`index`.
pub fn mb_array_contains(handle: MbValue, target: MbValue) -> MbValue {
    let hit = RefCell::new(false);
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        ARRAYS.with(|m| {
            if let Some(s) = m.borrow().get(&id) {
                let n = s.len();
                for i in 0..n {
                    if values_equal(s.get_as_value(i), target) {
                        *hit.borrow_mut() = true;
                        break;
                    }
                }
            }
        });
    }
    MbValue::from_bool(hit.into_inner())
}

/// `a.index(v)` — first index of `v`; CPython raises ValueError when absent.
pub fn mb_array_index(handle: MbValue, target: MbValue) -> MbValue {
    let mut found: i64 = -1;
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        ARRAYS.with(|m| {
            if let Some(s) = m.borrow().get(&id) {
                let n = s.len();
                for i in 0..n {
                    if values_equal(s.get_as_value(i), target) {
                        found = i as i64;
                        break;
                    }
                }
            }
        });
    }
    if found < 0 {
        raise_value_error("array.index(x): x not in array");
    }
    MbValue::from_int(found)
}

/// `a.insert(i, v)` — insert at index.
pub fn mb_array_insert(handle: MbValue, idx: MbValue, val: MbValue) -> MbValue {
    let raw_i = idx.as_int_pyint().unwrap_or(0);
    mutate_store(handle, |s| {
        let len = s.len() as i64;
        let i = if raw_i < 0 { (len + raw_i).max(0) } else { raw_i.min(len) };
        insert_value(s, i as usize, val);
    });
    MbValue::none()
}

fn insert_value(s: &mut ArrayStore, i: usize, v: MbValue) {
    // Push then rotate — keeps each typed branch small. For large arrays the
    // cost of rotation is the same as Python's; `insert` isn't on the hot
    // path the bench is measuring.
    if !s.push_value(v) {
        return;
    }
    let n = s.len();
    if i >= n { return; }
    match s {
        ArrayStore::I8(v) => v[i..].rotate_right(1),
        ArrayStore::U8(v) => v[i..].rotate_right(1),
        ArrayStore::Unicode(v) => v[i..].rotate_right(1),
        ArrayStore::I16(v) => v[i..].rotate_right(1),
        ArrayStore::U16(v) => v[i..].rotate_right(1),
        ArrayStore::I32(v) => v[i..].rotate_right(1),
        ArrayStore::U32(v) => v[i..].rotate_right(1),
        ArrayStore::I64(v) => v[i..].rotate_right(1),
        ArrayStore::U64(v) => v[i..].rotate_right(1),
        ArrayStore::F32(v) => v[i..].rotate_right(1),
        ArrayStore::F64(v) => v[i..].rotate_right(1),
    }
}

/// `a.pop(i=-1)` — remove and return element at index (defaults to last).
pub fn mb_array_pop(handle: MbValue, idx: MbValue) -> MbValue {
    let raw_i = idx.as_int().unwrap_or(-1);
    let result_cell: RefCell<MbValue> = RefCell::new(MbValue::none());
    let invalid = RefCell::new(false);
    mutate_store(handle, |s| {
        let n = s.len();
        if n == 0 {
            *invalid.borrow_mut() = true;
            return;
        }
        let i = if raw_i < 0 { (n as i64 + raw_i) as usize } else { raw_i as usize };
        if i >= n {
            *invalid.borrow_mut() = true;
            return;
        }
        let v = s.get_as_value(i);
        remove_at(s, i);
        *result_cell.borrow_mut() = v;
    });
    if invalid.into_inner() {
        return raise_index_error("array index out of range");
    }
    result_cell.into_inner()
}

fn remove_at(s: &mut ArrayStore, i: usize) {
    match s {
        ArrayStore::I8(v) => { v.remove(i); }
        ArrayStore::U8(v) => { v.remove(i); }
        ArrayStore::Unicode(v) => { v.remove(i); }
        ArrayStore::I16(v) => { v.remove(i); }
        ArrayStore::U16(v) => { v.remove(i); }
        ArrayStore::I32(v) => { v.remove(i); }
        ArrayStore::U32(v) => { v.remove(i); }
        ArrayStore::I64(v) => { v.remove(i); }
        ArrayStore::U64(v) => { v.remove(i); }
        ArrayStore::F32(v) => { v.remove(i); }
        ArrayStore::F64(v) => { v.remove(i); }
    }
}

/// `a.remove(v)` — remove first occurrence of `v`.
pub fn mb_array_remove(handle: MbValue, target: MbValue) -> MbValue {
    let removed = RefCell::new(false);
    mutate_store(handle, |s| {
        let n = s.len();
        for i in 0..n {
            if values_equal(s.get_as_value(i), target) {
                remove_at(s, i);
                *removed.borrow_mut() = true;
                return;
            }
        }
    });
    if !removed.into_inner() {
        return raise_value_error("array.remove(x): x not in array");
    }
    MbValue::none()
}

/// `a.reverse()` — in-place reverse.
pub fn mb_array_reverse(handle: MbValue) -> MbValue {
    mutate_store(handle, |s| match s {
        ArrayStore::I8(v) => v.reverse(),
        ArrayStore::U8(v) => v.reverse(),
        ArrayStore::Unicode(v) => v.reverse(),
        ArrayStore::I16(v) => v.reverse(),
        ArrayStore::U16(v) => v.reverse(),
        ArrayStore::I32(v) => v.reverse(),
        ArrayStore::U32(v) => v.reverse(),
        ArrayStore::I64(v) => v.reverse(),
        ArrayStore::U64(v) => v.reverse(),
        ArrayStore::F32(v) => v.reverse(),
        ArrayStore::F64(v) => v.reverse(),
    });
    MbValue::none()
}

/// `a.typecode` attribute — returns single-char str (`"i"`, `"d"`, ...).
pub fn mb_array_typecode_attr(handle: MbValue) -> MbValue {
    let tc = handle_typecode(handle).unwrap_or_else(|| read_store(handle, 'd', |s| s.typecode()));
    MbValue::from_ptr(MbObject::new_str(tc.to_string()))
}

/// `a.itemsize` attribute.
pub fn mb_array_itemsize_attr(handle: MbValue) -> MbValue {
    let isz = read_store(handle, 8i64, |s| s.itemsize());
    MbValue::from_int(isz)
}

fn slice_parts(key: MbValue) -> Option<(MbValue, MbValue, MbValue)> {
    let ptr = key.as_ptr()?;
    unsafe {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            if items.len() == 3 {
                return Some((items[0], items[1], items[2]));
            }
        }
    }
    None
}

fn clamp_slice_index(i: i64, len: i64) -> i64 {
    let idx = if i < 0 { i + len } else { i };
    idx.max(0).min(len)
}

fn slice_indices(len: i64, start: MbValue, stop: MbValue, step: i64) -> Vec<usize> {
    let (start, stop) = if step > 0 {
        (
            start.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(0),
            stop.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(len),
        )
    } else {
        (
            start.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(len - 1),
            stop.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(-1),
        )
    };
    let mut out = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < stop {
            if i >= 0 && i < len {
                out.push(i as usize);
            }
            i += step;
        }
    } else {
        while i > stop {
            if i >= 0 && i < len {
                out.push(i as usize);
            }
            i += step;
        }
    }
    out
}

fn slice_store(store: &ArrayStore, indices: &[usize]) -> ArrayStore {
    macro_rules! slice_variant {
        ($variant:ident, $items:ident) => {{
            ArrayStore::$variant(indices.iter().map(|&i| $items[i]).collect())
        }};
    }
    match store {
        ArrayStore::I8(items) => slice_variant!(I8, items),
        ArrayStore::U8(items) => slice_variant!(U8, items),
        ArrayStore::Unicode(items) => slice_variant!(Unicode, items),
        ArrayStore::I16(items) => slice_variant!(I16, items),
        ArrayStore::U16(items) => slice_variant!(U16, items),
        ArrayStore::I32(items) => slice_variant!(I32, items),
        ArrayStore::U32(items) => slice_variant!(U32, items),
        ArrayStore::I64(items) => slice_variant!(I64, items),
        ArrayStore::U64(items) => slice_variant!(U64, items),
        ArrayStore::F32(items) => slice_variant!(F32, items),
        ArrayStore::F64(items) => slice_variant!(F64, items),
    }
}

fn replace_store_range(target: &mut ArrayStore, start: usize, end: usize, replacement: &ArrayStore) -> bool {
    macro_rules! replace_variant {
        ($target:ident, $replacement:ident) => {{
            $target.splice(start..end, $replacement.iter().copied());
            true
        }};
    }
    match (target, replacement) {
        (ArrayStore::I8(target), ArrayStore::I8(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::U8(target), ArrayStore::U8(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::Unicode(target), ArrayStore::Unicode(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::I16(target), ArrayStore::I16(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::U16(target), ArrayStore::U16(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::I32(target), ArrayStore::I32(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::U32(target), ArrayStore::U32(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::I64(target), ArrayStore::I64(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::U64(target), ArrayStore::U64(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::F32(target), ArrayStore::F32(replacement)) => replace_variant!(target, replacement),
        (ArrayStore::F64(target), ArrayStore::F64(replacement)) => replace_variant!(target, replacement),
        _ => false,
    }
}

fn delete_store_range(store: &mut ArrayStore, start: usize, end: usize) {
    match store {
        ArrayStore::I8(items) => { items.drain(start..end); }
        ArrayStore::U8(items) => { items.drain(start..end); }
        ArrayStore::Unicode(items) => { items.drain(start..end); }
        ArrayStore::I16(items) => { items.drain(start..end); }
        ArrayStore::U16(items) => { items.drain(start..end); }
        ArrayStore::I32(items) => { items.drain(start..end); }
        ArrayStore::U32(items) => { items.drain(start..end); }
        ArrayStore::I64(items) => { items.drain(start..end); }
        ArrayStore::U64(items) => { items.drain(start..end); }
        ArrayStore::F32(items) => { items.drain(start..end); }
        ArrayStore::F64(items) => { items.drain(start..end); }
    }
}

fn mb_array_getslice(handle: MbValue, start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    let st = step.as_int_pyint().unwrap_or(1);
    if st == 0 {
        return raise_value_error("slice step cannot be zero");
    }
    let Some(store) = cloned_store(handle) else {
        return make_handle('d');
    };
    let indices = slice_indices(store.len() as i64, start, stop, st);
    let typecode = handle_typecode(handle).unwrap_or_else(|| store.typecode());
    make_handle_from_store(typecode, slice_store(&store, &indices))
}

/// `a[i]` and `a[start:stop:step]` support.
pub fn mb_array_getitem(handle: MbValue, key: MbValue) -> MbValue {
    if let Some((start, stop, step)) = slice_parts(key) {
        return mb_array_getslice(handle, start, stop, step);
    }
    let Some(raw_i) = key.as_int_pyint() else {
        return raise_type_error("array indices must be integers");
    };
    let mut result = MbValue::none();
    let mut invalid = false;
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        ARRAYS.with(|m| {
            if let Some(s) = m.borrow().get(&id) {
                let n = s.len() as i64;
                let i = if raw_i < 0 { n + raw_i } else { raw_i };
                if i < 0 || i >= n {
                    invalid = true;
                    return;
                }
                result = s.get_as_value(i as usize);
            }
        });
    }
    if invalid {
        return raise_index_error("array index out of range");
    }
    result
}

pub fn mb_array_setitem(handle: MbValue, key: MbValue, value: MbValue) -> MbValue {
    if let Some((start, stop, step)) = slice_parts(key) {
        let st = step.as_int_pyint().unwrap_or(1);
        if st != 1 {
            return raise_type_error("array slice assignment only supports step 1");
        }
        let Some(replacement) = cloned_store(value) else {
            return raise_type_error("can only assign array to array slice");
        };
        let handle_tc = handle_typecode(handle);
        let value_tc = handle_typecode(value);
        if handle_tc != value_tc {
            return raise_type_error("can only assign array of same kind");
        }
        let replaced = RefCell::new(true);
        mutate_store(handle, |s| {
            let len = s.len() as i64;
            let s_idx = start.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(0) as usize;
            let e_idx = stop.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(len) as usize;
            let s_idx = s_idx.min(s.len());
            let e_idx = e_idx.min(s.len()).max(s_idx);
            if !replace_store_range(s, s_idx, e_idx, &replacement) {
                *replaced.borrow_mut() = false;
            }
        });
        if !replaced.into_inner() {
            return raise_type_error("can only assign array of same kind");
        }
    }
    MbValue::none()
}

pub fn mb_array_delitem(handle: MbValue, key: MbValue) -> MbValue {
    if let Some((start, stop, step)) = slice_parts(key) {
        let st = step.as_int_pyint().unwrap_or(1);
        if st == 0 {
            return raise_value_error("slice step cannot be zero");
        }
        mutate_store(handle, |s| {
            if st == 1 {
                let len = s.len() as i64;
                let s_idx = start.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(0) as usize;
                let e_idx = stop.as_int_pyint().map(|i| clamp_slice_index(i, len)).unwrap_or(len) as usize;
                let s_idx = s_idx.min(s.len());
                let e_idx = e_idx.min(s.len()).max(s_idx);
                delete_store_range(s, s_idx, e_idx);
            } else {
                let indices = slice_indices(s.len() as i64, start, stop, st);
                for i in indices.into_iter().rev() {
                    remove_at(s, i);
                }
            }
        });
        return MbValue::none();
    }
    let Some(raw_i) = key.as_int_pyint() else {
        return raise_type_error("array indices must be integers");
    };
    mutate_store(handle, |s| {
        let len = s.len() as i64;
        let i = if raw_i < 0 { len + raw_i } else { raw_i };
        if i >= 0 && i < len {
            remove_at(s, i as usize);
        }
    });
    MbValue::none()
}

/// `len(a)` support — used by class.rs `__len__` dispatch.
pub fn mb_array_len(handle: MbValue) -> MbValue {
    MbValue::from_int(read_store(handle, 0i64, |s| s.len() as i64))
}

pub fn mb_array_eq_bool(lhs: MbValue, rhs: MbValue) -> Option<bool> {
    let lhs_is_array = lhs.as_int().is_some_and(|id| is_array_handle(id as u64));
    let rhs_is_array = rhs.as_int().is_some_and(|id| is_array_handle(id as u64));
    if !lhs_is_array && !rhs_is_array {
        return None;
    }
    if !lhs_is_array || !rhs_is_array {
        return Some(false);
    }
    let lhs_store = cloned_store(lhs)?;
    let rhs_store = cloned_store(rhs)?;
    if lhs_store.len() != rhs_store.len() {
        return Some(false);
    }
    Some((0..lhs_store.len()).all(|i| values_equal(lhs_store.get_as_value(i), rhs_store.get_as_value(i))))
}

pub fn mb_array_lt_bool(lhs: MbValue, rhs: MbValue) -> Option<bool> {
    let lhs_is_array = lhs.as_int().is_some_and(|id| is_array_handle(id as u64));
    let rhs_is_array = rhs.as_int().is_some_and(|id| is_array_handle(id as u64));
    if !lhs_is_array && !rhs_is_array {
        return None;
    }
    if !lhs_is_array || !rhs_is_array {
        return Some(false);
    }
    let lhs_store = cloned_store(lhs)?;
    let rhs_store = cloned_store(rhs)?;
    let shared_len = lhs_store.len().min(rhs_store.len());
    for i in 0..shared_len {
        let lhs_value = lhs_store.get_as_value(i);
        let rhs_value = rhs_store.get_as_value(i);
        if value_less(lhs_value, rhs_value) {
            return Some(true);
        }
        if value_less(rhs_value, lhs_value) {
            return Some(false);
        }
        if !values_equal(lhs_value, rhs_value) {
            return Some(false);
        }
    }
    Some(lhs_store.len() < rhs_store.len())
}

fn values_equal(a: MbValue, b: MbValue) -> bool {
    match (a.as_int(), b.as_int()) {
        (Some(x), Some(y)) => return x == y,
        _ => {}
    }
    match (a.as_float(), b.as_float()) {
        (Some(x), Some(y)) => x == y,
        _ => {
            if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
                unsafe {
                    if let (ObjData::Str(a), ObjData::Str(b)) = (&(*pa).data, &(*pb).data) {
                        return a == b;
                    }
                }
            }
            false
        }
    }
}

fn value_less(a: MbValue, b: MbValue) -> bool {
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    if let (Some(af), Some(bf)) = (af, bf) {
        return af < bf;
    }
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            if let (ObjData::Str(a), ObjData::Str(b)) = (&(*pa).data, &(*pb).data) {
                return a < b;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(text.to_string()))
    }

    fn bytes_data(val: MbValue) -> Vec<u8> {
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                if let ObjData::Bytes(ref b) = (*ptr).data { return b.clone(); }
            }
        }
        vec![]
    }

    fn list_items(val: MbValue) -> Vec<MbValue> {
        if let Some(ptr) = val.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().to_vec();
                }
            }
        }
        vec![]
    }

    #[test]
    fn test_handle_predicate() {
        let h = mb_array_new(s("i"), MbValue::none());
        let id = h.as_int().unwrap() as u64;
        assert!(is_array_handle(id));
        assert!(!is_array_handle(u64::MAX));
    }

    #[test]
    fn test_typecode_attr_and_itemsize() {
        let h = mb_array_new(s("i"), MbValue::none());
        assert_eq!(mb_array_itemsize_attr(h).as_int(), Some(4));
        let tc = mb_array_typecode_attr(h);
        assert_eq!(unsafe {
            tc.as_ptr().map(|p| if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() })
        }, Some("i".to_string()));

        let hd = mb_array_new(s("d"), MbValue::none());
        assert_eq!(mb_array_itemsize_attr(hd).as_int(), Some(8));

        let hb = mb_array_new(s("B"), MbValue::none());
        assert_eq!(mb_array_itemsize_attr(hb).as_int(), Some(1));
    }

    #[test]
    fn test_int32_roundtrip_via_frombytes_tobytes() {
        let h = mb_array_new(s("i"), MbValue::none());
        // 4 int32 elements: 1, -2, 3, -4 → 16 bytes little-endian
        let src_bytes: Vec<u8> = vec![
            1, 0, 0, 0,
            254, 255, 255, 255, // -2 as i32 LE
            3, 0, 0, 0,
            252, 255, 255, 255, // -4 as i32 LE
        ];
        let buf = MbValue::from_ptr(MbObject::new_bytes(src_bytes.clone()));
        mb_array_frombytes(h, buf);
        assert_eq!(mb_array_len(h).as_int(), Some(4));

        let out = bytes_data(mb_array_tobytes(h));
        assert_eq!(out, src_bytes);

        let items = list_items(mb_array_tolist(h));
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(-2));
        assert_eq!(items[2].as_int(), Some(3));
        assert_eq!(items[3].as_int(), Some(-4));
    }

    #[test]
    fn test_float64_roundtrip_via_frombytes_tobytes() {
        let h = mb_array_new(s("d"), MbValue::none());
        // f64 = 1.5 → 8 bytes LE = 00 00 00 00 00 00 F8 3F
        let src = 1.5_f64.to_le_bytes();
        let buf = MbValue::from_ptr(MbObject::new_bytes(src.to_vec()));
        mb_array_frombytes(h, buf);
        assert_eq!(mb_array_len(h).as_int(), Some(1));
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items[0].as_float(), Some(1.5));
        assert_eq!(bytes_data(mb_array_tobytes(h)), src.to_vec());
    }

    #[test]
    fn test_uint8_byteswap_noop() {
        let h = mb_array_new(s("B"), MbValue::none());
        let buf = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2, 3, 4]));
        mb_array_frombytes(h, buf);
        mb_array_byteswap(h);
        assert_eq!(bytes_data(mb_array_tobytes(h)), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_uint16_byteswap_flips_endian() {
        let h = mb_array_new(s("H"), MbValue::none());
        let buf = MbValue::from_ptr(MbObject::new_bytes(vec![0x01, 0x02, 0x03, 0x04]));
        mb_array_frombytes(h, buf);
        // After byteswap: 0x0201 → 0x0102, 0x0403 → 0x0304
        mb_array_byteswap(h);
        assert_eq!(bytes_data(mb_array_tobytes(h)), vec![0x02, 0x01, 0x04, 0x03]);
    }

    #[test]
    fn test_append_and_count() {
        let h = mb_array_new(s("i"), MbValue::none());
        mb_array_append(h, MbValue::from_int(7));
        mb_array_append(h, MbValue::from_int(7));
        mb_array_append(h, MbValue::from_int(3));
        assert_eq!(mb_array_count(h, MbValue::from_int(7)).as_int(), Some(2));
        assert_eq!(mb_array_count(h, MbValue::from_int(99)).as_int(), Some(0));
    }

    #[test]
    fn test_index_returns_first_match() {
        let h = mb_array_new(s("i"), MbValue::none());
        mb_array_append(h, MbValue::from_int(10));
        mb_array_append(h, MbValue::from_int(20));
        mb_array_append(h, MbValue::from_int(20));
        assert_eq!(mb_array_index(h, MbValue::from_int(20)).as_int(), Some(1));
        assert_eq!(mb_array_index(h, MbValue::from_int(99)).as_int(), Some(-1));
    }

    #[test]
    fn test_insert_pop_remove_reverse() {
        let h = mb_array_new(s("i"), MbValue::none());
        mb_array_extend(h, MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2), MbValue::from_int(3),
        ])));
        mb_array_insert(h, MbValue::from_int(1), MbValue::from_int(99));
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items.iter().map(|v| v.as_int().unwrap()).collect::<Vec<_>>(), vec![1, 99, 2, 3]);

        let popped = mb_array_pop(h, MbValue::from_int(-1));
        assert_eq!(popped.as_int(), Some(3));

        mb_array_remove(h, MbValue::from_int(99));
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items.iter().map(|v| v.as_int().unwrap()).collect::<Vec<_>>(), vec![1, 2]);

        mb_array_reverse(h);
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items.iter().map(|v| v.as_int().unwrap()).collect::<Vec<_>>(), vec![2, 1]);
    }

    #[test]
    fn test_buffer_info_returns_handle_and_len() {
        let h = mb_array_new(s("i"), MbValue::none());
        mb_array_extend(h, MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ])));
        let info = mb_array_buffer_info(h);
        let items = if let Some(ptr) = info.as_ptr() {
            unsafe {
                if let ObjData::Tuple(ref v) = (*ptr).data { v.clone() } else { vec![] }
            }
        } else { vec![] };
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].as_int(), Some(2));
    }

    #[test]
    fn test_extend_from_array_handle() {
        let src = mb_array_new(s("i"), MbValue::none());
        mb_array_extend(src, MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(10), MbValue::from_int(20),
        ])));
        let dst = mb_array_new(s("i"), MbValue::none());
        mb_array_extend(dst, src);
        let items = list_items(mb_array_tolist(dst));
        assert_eq!(items.iter().map(|v| v.as_int().unwrap()).collect::<Vec<_>>(), vec![10, 20]);
    }

    #[test]
    fn test_initializer_from_bytes() {
        let buf = MbValue::from_ptr(MbObject::new_bytes(vec![1, 0, 0, 0, 2, 0, 0, 0]));
        let h = mb_array_new(s("i"), buf);
        assert_eq!(mb_array_len(h).as_int(), Some(2));
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items[0].as_int(), Some(1));
        assert_eq!(items[1].as_int(), Some(2));
    }

    #[test]
    fn test_initializer_from_list() {
        let init = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5), MbValue::from_int(6),
        ]));
        let h = mb_array_new(s("i"), init);
        assert_eq!(mb_array_len(h).as_int(), Some(2));
    }

    #[test]
    fn test_fromlist_appends() {
        let h = mb_array_new(s("i"), MbValue::none());
        let lst = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(11), MbValue::from_int(22),
        ]));
        mb_array_fromlist(h, lst);
        assert_eq!(mb_array_len(h).as_int(), Some(2));
    }

    #[test]
    fn test_int8_negative_values() {
        let h = mb_array_new(s("b"), MbValue::none());
        // -1 i8 → 0xFF byte
        let buf = MbValue::from_ptr(MbObject::new_bytes(vec![0xFF]));
        mb_array_frombytes(h, buf);
        let items = list_items(mb_array_tolist(h));
        assert_eq!(items[0].as_int(), Some(-1));
        assert_eq!(bytes_data(mb_array_tobytes(h)), vec![0xFF]);
    }

    #[test]
    fn test_partial_byte_buffer_drops_trailing() {
        let h = mb_array_new(s("i"), MbValue::none());
        // 5 bytes — not a multiple of 4; one full element + 1 dropped byte
        let buf = MbValue::from_ptr(MbObject::new_bytes(vec![1, 0, 0, 0, 99]));
        mb_array_frombytes(h, buf);
        assert_eq!(mb_array_len(h).as_int(), Some(1));
        assert_eq!(list_items(mb_array_tolist(h))[0].as_int(), Some(1));
    }

    // ── #2181: ARRAYS thread_local handle-table leak regression ──

    #[test]
    fn test_drop_handle_removes_entry() {
        let h = mb_array_new(s("i"), MbValue::none());
        let id = h.as_int().unwrap() as u64;
        assert!(is_array_handle(id));

        drop_handle(id);
        assert!(!is_array_handle(id));

        // Operations on a dropped handle become no-ops / defaults.
        assert_eq!(mb_array_len(h).as_int(), Some(0));
        assert_eq!(mb_array_itemsize_attr(h).as_int(), Some(8));

        // Idempotent: dropping again is harmless.
        drop_handle(id);
        drop_handle(u64::MAX); // unknown id is a no-op
    }

    #[test]
    fn test_soft_cap_eviction_prevents_monotonic_growth() {
        // Snapshot starting size — other tests on this thread may have
        // populated entries already.
        let baseline = handle_table_len();

        // Allocate enough handles to overflow the soft cap several times.
        // The eviction policy keeps `ARRAYS.len() < ARRAY_SOFT_CAP`
        // immediately after each `mb_array_new`, so the table cannot
        // grow without bound regardless of how many handles we allocate.
        let alloc_count = ARRAY_SOFT_CAP * 4;
        let mut last_handle = MbValue::none();
        for _ in 0..alloc_count {
            last_handle = mb_array_new(s("i"), MbValue::none());
        }

        let final_len = handle_table_len();
        assert!(
            final_len <= ARRAY_SOFT_CAP,
            "ARRAYS size {final_len} exceeded soft cap {ARRAY_SOFT_CAP} after {alloc_count} allocations (baseline was {baseline})",
        );

        // The most recently allocated handle is still live (FIFO eviction
        // walks the oldest entries first).
        let last_id = last_handle.as_int().unwrap() as u64;
        assert!(is_array_handle(last_id));
    }

    // ── #2111: integer-handle refcount registry ──

    #[test]
    fn test_release_handle_drops_entry_at_rc_zero() {
        // Fresh handle starts at refcount 1; one release drops the entry.
        let baseline = handle_table_len();
        let h = mb_array_new(s("i"), MbValue::none());
        let id = h.as_int().unwrap() as u64;
        assert!(is_array_handle(id));
        assert_eq!(handle_table_len(), baseline + 1);

        assert!(release_handle(id));
        assert!(!is_array_handle(id));
        assert_eq!(handle_table_len(), baseline);
    }

    #[test]
    fn test_retain_release_keeps_entry_until_rc_zero() {
        // Simulate `b = a` (Copy lowering retains source) followed by
        // two rebinds. The handle must survive the first release.
        let baseline = handle_table_len();
        let h = mb_array_new(s("i"), MbValue::none());
        let id = h.as_int().unwrap() as u64;

        assert!(retain_handle(id)); // rc 1 → 2 (alias)
        assert!(release_handle(id)); // rc 2 → 1 (drop alias)
        assert!(is_array_handle(id), "entry must survive while rc > 0");

        assert!(release_handle(id)); // rc 1 → 0 (final drop)
        assert!(!is_array_handle(id));
        assert_eq!(handle_table_len(), baseline);
    }

    #[test]
    fn test_unknown_id_is_no_op() {
        assert!(!retain_handle(u64::MAX));
        assert!(!release_handle(u64::MAX));
        assert!(!retain_handle(0));
        assert!(!release_handle(0));
    }

    #[test]
    fn test_hot_loop_rebind_does_not_grow_table() {
        // #2111 regression: simulate the `a = make_array(...)` rebind
        // pattern. Each iter, the JIT's pre-write release would call
        // `mb_release_value(prev_id)`, which now routes through the
        // integer-handle registry. Table size must stay flat.
        let baseline = handle_table_len();

        let mut current = mb_array_new(s("i"), MbValue::none());
        let first_id = current.as_int().unwrap() as u64;
        assert_eq!(handle_table_len(), baseline + 1);

        for _ in 0..200 {
            let prev_id = current.as_int().unwrap() as u64;
            // Allocate the next handle BEFORE releasing the prior one to
            // mimic the JIT's evaluation order (RHS first, then store).
            let next = mb_array_new(s("i"), MbValue::none());
            release_handle(prev_id);
            current = next;
        }

        // Only the last handle should remain live (baseline + 1).
        assert_eq!(
            handle_table_len(),
            baseline + 1,
            "hot-loop rebind must not accumulate handles"
        );
        // Final handle is live; the original got dropped after the first iter.
        let final_id = current.as_int().unwrap() as u64;
        assert!(is_array_handle(final_id));
        assert!(!is_array_handle(first_id));

        // Cleanup.
        release_handle(final_id);
    }
}
