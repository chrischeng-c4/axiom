//! @codegen-skip: handwrite-pre-standardize
//!
//! uuid module for Mamba — Python 3.12 `uuid` stdlib.
//!
//! Provides the `UUID` class as **integer handles** (i64 IDs) backed
//! by a thread-local `HashMap<u64, UuidState>` table. Mirrors the OOP
//! pattern established by `hashlib_mod` / `hmac_mod` / `random_mod` /
//! `array_mod` / `fractions_mod` (see [[project_mamba_integer_handle_pattern]]).
//!
//! UUID has **no operator overloads** (no `__add__` etc.), so the
//! [[project_mamba_int_handle_operator_overload_gap]] does NOT apply
//! — `.hex`, `.int`, `.urn`, `.version` etc. are accessed via the
//! standard `class.rs::mb_getattr` branch.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + integer-handle protocol)
//! is not yet emitted by score codegen. Tracked as part of the
//! brute-force Phase-2 sweep; will be replaced when aw standardize
//! lands the stdlib-shim section type. Issue #1414 cluster anchor.
//!
//! Carve-outs (documented gaps from CPython parity):
//!
//! - `getnode()` returns a synthetic random 48-bit value — real MAC
//!   discovery would require platform-specific network probing.
//!   Deterministic uuid1 still produces version-1 bit pattern.
//! - `uuid6` / `uuid7` / `uuid8` (Python 3.14+) — typeshed-future,
//!   not implemented.
//! - `UUID.fields` returns a 6-tuple and hits #2128
//!   (`MbObject::new_tuple` calls `gc::gc_track`, ~150x penalty in
//!   tight loops). Pre-documented as Gate 2 carve-out; correctness
//!   unaffected.
//! - `UUID.bytes` / `UUID.bytes_le` allocate a 16-byte buffer per
//!   call (#2096 subset A); fine at small fixture sizes, bench
//!   tier:compute hot loops should avoid bytes-return paths.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN

use md5::Md5;
use rand::RngCore;
use sha1::{Digest as Sha1Digest, Sha1};

/// Handle IDs use a high range so they cannot collide with small
/// literal ints (e.g. `0`, `1` returned from `UUID.version`). 48-bit
/// MbValue int range is `[-2^47, 2^47)`; we sit at 2^41+ which is
/// well above any plausible integer attribute return.
const UUID_HANDLE_BASE: u64 = 1u64 << 41;

/// Backing state for a `UUID` instance: 16 raw bytes (big-endian
/// canonical CPython byte order) + the version nibble pre-applied.
#[derive(Clone, Copy)]
struct UuidState {
    bytes: [u8; 16],
}

impl UuidState {
    fn version(&self) -> u8 {
        (self.bytes[6] >> 4) & 0x0F
    }

    fn variant_str(&self) -> &'static str {
        // CPython's `Variant` enum collapses to four strings.
        let b8 = self.bytes[8];
        if b8 & 0x80 == 0 {
            "reserved for NCS compatibility"
        } else if b8 & 0xC0 == 0x80 {
            "specified in RFC 4122"
        } else if b8 & 0xE0 == 0xC0 {
            "reserved for Microsoft compatibility"
        } else {
            "reserved for future definition"
        }
    }

    fn to_hex(&self) -> String {
        let mut s = String::with_capacity(32);
        for b in &self.bytes {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }

    fn to_canonical(&self) -> String {
        let h = self.to_hex();
        // 8-4-4-4-12 grouping (offsets 8, 13, 18, 23).
        let mut out = String::with_capacity(36);
        out.push_str(&h[0..8]);
        out.push('-');
        out.push_str(&h[8..12]);
        out.push('-');
        out.push_str(&h[12..16]);
        out.push('-');
        out.push_str(&h[16..20]);
        out.push('-');
        out.push_str(&h[20..32]);
        out
    }

    fn to_int(&self) -> i64 {
        // CPython exposes a 128-bit int; MbValue ints are 48-bit-boxed.
        // Carve: fold the high bytes via XOR so the result is a stable
        // 47-bit fingerprint instead of truncation. Real `.int`
        // round-trip is a #2096-class gap.
        let mut acc: u64 = 0;
        for chunk in self.bytes.chunks_exact(2) {
            acc ^= ((chunk[0] as u64) << 8) | (chunk[1] as u64);
            acc = acc.wrapping_mul(0x9E37_79B9);
        }
        (acc & ((1u64 << 47) - 1)) as i64
    }

    fn from_hex(s: &str) -> Self {
        let cleaned: String = s.chars().filter(|c| c.is_ascii_hexdigit()).collect();
        let mut bytes = [0u8; 16];
        for (i, b) in bytes.iter_mut().enumerate() {
            let hi = cleaned.as_bytes().get(i * 2).copied().unwrap_or(b'0');
            let lo = cleaned.as_bytes().get(i * 2 + 1).copied().unwrap_or(b'0');
            *b = (hex_nibble(hi) << 4) | hex_nibble(lo);
        }
        UuidState { bytes }
    }
}

fn hex_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

fn apply_version(bytes: &mut [u8; 16], version: u8) {
    // RFC 4122 §4.1.3 — set version nibble in byte 6 (high nibble)
    // and the variant bits in byte 8 (top two bits to 0b10).
    bytes[6] = (bytes[6] & 0x0F) | ((version & 0x0F) << 4);
    bytes[8] = (bytes[8] & 0x3F) | 0x80;
}

thread_local! {
    static UUIDS: RefCell<HashMap<u64, UuidState>> = RefCell::new(HashMap::new());
    static UUID_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_UUID_ID: Cell<u64> = const { Cell::new(UUID_HANDLE_BASE) };
    /// Per-handle refcount (#2111).
    static UUID_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_uuid_id() -> u64 {
    NEXT_UUID_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// `class.rs` calls this to decide whether an int receiver routes
/// into the uuid protocol.
pub fn is_uuid_handle(id: u64) -> bool {
    UUID_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_uuid_handle(id: u64) {
    UUIDS.with(|m| {
        m.borrow_mut().remove(&id);
    });
    UUID_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    UUID_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_uuid_handle(id) {
        return false;
    }
    UUID_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_uuid_handle(id) {
        return false;
    }
    let should_drop = UUID_REFCOUNTS.with(|r| {
        let mut map = r.borrow_mut();
        let rc = map.entry(id).or_insert(1);
        if *rc <= 1 {
            map.remove(&id);
            true
        } else {
            *rc -= 1;
            false
        }
    });
    if should_drop {
        drop_uuid_handle(id);
    }
    true
}

fn make_handle(state: UuidState) -> MbValue {
    let id = alloc_uuid_id();
    UUIDS.with(|m| {
        m.borrow_mut().insert(id, state);
    });
    UUID_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

fn load(handle: MbValue) -> UuidState {
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        if let Some(s) = UUIDS.with(|m| m.borrow().get(&id).copied()) {
            return s;
        }
    }
    UuidState { bytes: [0u8; 16] }
}

// ── Public surface — free fns used by both dispatchers and class.rs.

/// `uuid.uuid4()` — random UUID per RFC 4122 §4.4.
pub fn mb_uuid_uuid4() -> MbValue {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    apply_version(&mut bytes, 4);
    make_handle(UuidState { bytes })
}

/// `uuid.uuid1(node=None, clock_seq=None)` — synthetic version-1
/// (random node + clock-seq since real MAC discovery is out of scope).
pub fn mb_uuid_uuid1() -> MbValue {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    apply_version(&mut bytes, 1);
    make_handle(UuidState { bytes })
}

/// `uuid.uuid3(namespace, name)` — MD5 hash of namespace.bytes + name.
pub fn mb_uuid_uuid3(namespace: MbValue, name: MbValue) -> MbValue {
    let ns = load(namespace);
    let mut hasher = Md5::new();
    hasher.update(ns.bytes);
    with_str(name, |s| hasher.update(s.as_bytes()));
    let out = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&out[..16]);
    apply_version(&mut bytes, 3);
    make_handle(UuidState { bytes })
}

/// `uuid.uuid5(namespace, name)` — SHA-1 hash truncated to 16 bytes.
pub fn mb_uuid_uuid5(namespace: MbValue, name: MbValue) -> MbValue {
    let ns = load(namespace);
    let mut hasher = Sha1::new();
    hasher.update(ns.bytes);
    with_str(name, |s| hasher.update(s.as_bytes()));
    let out = hasher.finalize();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&out[..16]);
    apply_version(&mut bytes, 5);
    make_handle(UuidState { bytes })
}

/// `uuid.getnode()` — synthetic node (random) with multicast bit
/// set to flag this as non-IEEE-MAC per RFC 4122. Real platform MAC
/// discovery is intentionally out of scope (see module docstring).
/// Result is clamped to 47 bits so it fits MbValue's 48-bit-signed
/// integer payload (value.rs:73).
pub fn mb_uuid_getnode() -> MbValue {
    let mut buf = [0u8; 6];
    rand::thread_rng().fill_bytes(&mut buf);
    // Build a 40-bit node from the low five bytes so the multicast
    // bit at position 40 stays inside the 47-bit positive range.
    let mut n: u64 = ((buf[1] as u64) << 32)
        | ((buf[2] as u64) << 24)
        | ((buf[3] as u64) << 16)
        | ((buf[4] as u64) << 8)
        | (buf[5] as u64);
    n |= 1u64 << 40; // multicast flag
    MbValue::from_int(n as i64)
}

/// `UUID(hex)` — primary string constructor.
#[allow(non_snake_case)]
pub fn mb_uuid_UUID(hex: MbValue) -> MbValue {
    let mut s = String::new();
    with_str(hex, |v| s.push_str(v));
    make_handle(UuidState::from_hex(&s))
}

/// `UUID(int=...)` — int constructor; carve to from-hex since MbValue
/// ints are 48-bit-boxed. Treat the int as a 47-bit fingerprint and
/// zero-extend into the low bytes; high bytes default. Useful for
/// round-trip tests, not for arbitrary 128-bit ints.
pub fn mb_uuid_from_int(int_val: MbValue) -> MbValue {
    let n = int_val.as_int().unwrap_or(0).max(0) as u64;
    let mut bytes = [0u8; 16];
    for i in 0..8 {
        bytes[15 - i] = ((n >> (i * 8)) & 0xFF) as u8;
    }
    make_handle(UuidState { bytes })
}

// ── Attribute accessors used by class.rs::mb_getattr.

pub fn mb_uuid_hex(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(load(handle).to_hex()))
}

pub fn mb_uuid_str(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(load(handle).to_canonical()))
}

pub fn mb_uuid_urn(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(format!(
        "urn:uuid:{}",
        load(handle).to_canonical()
    )))
}

pub fn mb_uuid_int_attr(handle: MbValue) -> MbValue {
    MbValue::from_int(load(handle).to_int())
}

pub fn mb_uuid_version_attr(handle: MbValue) -> MbValue {
    MbValue::from_int(load(handle).version() as i64)
}

pub fn mb_uuid_variant_attr(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(load(handle).variant_str().to_string()))
}

/// `.bytes` — raw big-endian 16-byte buffer (#2096 subset A: per-call
/// allocation).
pub fn mb_uuid_bytes_attr(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(load(handle).bytes.to_vec()))
}

/// `.bytes_le` — Microsoft variant little-endian permutation.
pub fn mb_uuid_bytes_le_attr(handle: MbValue) -> MbValue {
    let s = load(handle);
    let mut out = s.bytes;
    out.swap(0, 3);
    out.swap(1, 2);
    out.swap(4, 5);
    out.swap(6, 7);
    MbValue::from_ptr(MbObject::new_bytes(out.to_vec()))
}

/// `.fields` — 6-tuple of (time_low, time_mid, time_hi_version,
/// clock_seq_hi_variant, clock_seq_low, node). Hits #2128.
pub fn mb_uuid_fields_attr(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    let time_low =
        ((b[0] as i64) << 24) | ((b[1] as i64) << 16) | ((b[2] as i64) << 8) | (b[3] as i64);
    let time_mid = ((b[4] as i64) << 8) | (b[5] as i64);
    let time_hi_version = ((b[6] as i64) << 8) | (b[7] as i64);
    let clock_seq_hi = b[8] as i64;
    let clock_seq_low = b[9] as i64;
    let node = ((b[10] as i64) << 40)
        | ((b[11] as i64) << 32)
        | ((b[12] as i64) << 24)
        | ((b[13] as i64) << 16)
        | ((b[14] as i64) << 8)
        | (b[15] as i64);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(time_low),
        MbValue::from_int(time_mid),
        MbValue::from_int(time_hi_version),
        MbValue::from_int(clock_seq_hi),
        MbValue::from_int(clock_seq_low),
        MbValue::from_int(node),
    ]))
}

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

// ── Flat-args dispatch thunks (module-level entries).

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_a: *const MbValue, _n: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.first().copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.first().copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_nullary!(dispatch_uuid1, mb_uuid_uuid1);
dispatch_nullary!(dispatch_uuid4, mb_uuid_uuid4);
dispatch_nullary!(dispatch_getnode, mb_uuid_getnode);
dispatch_binary!(dispatch_uuid3, mb_uuid_uuid3);
dispatch_binary!(dispatch_uuid5, mb_uuid_uuid5);
dispatch_unary!(dispatch_UUID, mb_uuid_UUID);
dispatch_unary!(dispatch_from_int, mb_uuid_from_int);
// Attribute-getter dispatchers, also exposed as module-level helpers
// for benches that want a callable shape (matches the fractions
// `fraction_numerator` pattern — see
// [[project_mamba_int_handle_operator_overload_gap]]).
dispatch_unary!(dispatch_uuid_hex, mb_uuid_hex);
dispatch_unary!(dispatch_uuid_int, mb_uuid_int_attr);
dispatch_unary!(dispatch_uuid_version, mb_uuid_version_attr);

/// Build the four well-known namespace UUIDs (CPython literals).
fn make_namespace(hex: &str) -> MbValue {
    let s = UuidState::from_hex(hex);
    make_handle(s)
}

// ── Module registration

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("uuid1", dispatch_uuid1 as usize),
        ("uuid3", dispatch_uuid3 as usize),
        ("uuid4", dispatch_uuid4 as usize),
        ("uuid5", dispatch_uuid5 as usize),
        ("getnode", dispatch_getnode as usize),
        ("UUID", dispatch_UUID as usize),
        ("uuid_from_int", dispatch_from_int as usize),
        ("uuid_hex", dispatch_uuid_hex as usize),
        ("uuid_int", dispatch_uuid_int as usize),
        ("uuid_version", dispatch_uuid_version as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Well-known namespace UUIDs (RFC 4122 Appendix C). These are
    // handle-bound — callers pass them to `uuid3` / `uuid5`.
    attrs.insert(
        "NAMESPACE_DNS".into(),
        make_namespace("6ba7b8109dad11d180b400c04fd430c8"),
    );
    attrs.insert(
        "NAMESPACE_URL".into(),
        make_namespace("6ba7b8119dad11d180b400c04fd430c8"),
    );
    attrs.insert(
        "NAMESPACE_OID".into(),
        make_namespace("6ba7b8129dad11d180b400c04fd430c8"),
    );
    attrs.insert(
        "NAMESPACE_X500".into(),
        make_namespace("6ba7b8149dad11d180b400c04fd430c8"),
    );

    // Variant constants (CPython exposes as strings).
    attrs.insert(
        "RESERVED_NCS".into(),
        MbValue::from_ptr(MbObject::new_str("reserved for NCS compatibility".into())),
    );
    attrs.insert(
        "RFC_4122".into(),
        MbValue::from_ptr(MbObject::new_str("specified in RFC 4122".into())),
    );
    attrs.insert(
        "RESERVED_MICROSOFT".into(),
        MbValue::from_ptr(MbObject::new_str(
            "reserved for Microsoft compatibility".into(),
        )),
    );
    attrs.insert(
        "RESERVED_FUTURE".into(),
        MbValue::from_ptr(MbObject::new_str("reserved for future definition".into())),
    );

    // SafeUUID emulation — CPython exposes a Flag enum with three
    // members (`safe`, `unsafe`, `unknown`). We surface those three
    // member strings, and bind `SafeUUID` itself to the canonical
    // "unknown" sentinel so `uuid.SafeUUID` resolves (sentinel-only;
    // membership query is the carve-out, see module docstring).
    attrs.insert(
        "SAFE_SAFE".into(),
        MbValue::from_ptr(MbObject::new_str("safe".into())),
    );
    attrs.insert(
        "SAFE_UNSAFE".into(),
        MbValue::from_ptr(MbObject::new_str("unsafe".into())),
    );
    attrs.insert(
        "SAFE_UNKNOWN".into(),
        MbValue::from_ptr(MbObject::new_str("unknown".into())),
    );
    attrs.insert(
        "SafeUUID".into(),
        MbValue::from_ptr(MbObject::new_str("unknown".into())),
    );

    super::register_module("uuid", attrs);

    // #2111: integer-handle refcount hooks.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// HANDWRITE-END

#[cfg(test)]
mod tests {
    use super::*;

    fn str_of(val: MbValue) -> String {
        let mut out = String::new();
        with_str(val, |s| out.push_str(s));
        out
    }

    #[test]
    fn test_uuid4_shape() {
        let u = mb_uuid_uuid4();
        let s = str_of(mb_uuid_str(u));
        assert_eq!(s.len(), 36);
        assert_eq!(s.as_bytes()[8], b'-');
        assert_eq!(s.as_bytes()[13], b'-');
        assert_eq!(s.as_bytes()[18], b'-');
        assert_eq!(s.as_bytes()[23], b'-');
        // Version nibble at position 14 must be '4'.
        assert_eq!(s.chars().nth(14).unwrap(), '4');
        assert_eq!(mb_uuid_version_attr(u).as_int(), Some(4));
    }

    #[test]
    fn test_uuid1_version_byte() {
        let u = mb_uuid_uuid1();
        let s = str_of(mb_uuid_str(u));
        assert_eq!(s.chars().nth(14).unwrap(), '1');
        assert_eq!(mb_uuid_version_attr(u).as_int(), Some(1));
    }

    #[test]
    fn test_uuid_from_hex_roundtrip() {
        let input = "550e8400-e29b-41d4-a716-446655440000";
        let u = mb_uuid_UUID(MbValue::from_ptr(MbObject::new_str(input.to_string())));
        assert_eq!(str_of(mb_uuid_str(u)), input);
        assert_eq!(str_of(mb_uuid_hex(u)), "550e8400e29b41d4a716446655440000");
        assert_eq!(mb_uuid_version_attr(u).as_int(), Some(4));
    }

    #[test]
    fn test_uuid3_deterministic() {
        let ns = mb_uuid_UUID(MbValue::from_ptr(MbObject::new_str(
            "6ba7b8109dad11d180b400c04fd430c8".to_string(),
        )));
        let name = MbValue::from_ptr(MbObject::new_str("python.org".to_string()));
        let a = mb_uuid_uuid3(ns, name);
        let b = mb_uuid_uuid3(ns, name);
        assert_eq!(str_of(mb_uuid_str(a)), str_of(mb_uuid_str(b)));
        assert_eq!(mb_uuid_version_attr(a).as_int(), Some(3));
    }

    #[test]
    fn test_uuid5_deterministic_and_known_vector() {
        let ns = mb_uuid_UUID(MbValue::from_ptr(MbObject::new_str(
            "6ba7b8109dad11d180b400c04fd430c8".to_string(),
        )));
        let name = MbValue::from_ptr(MbObject::new_str("python.org".to_string()));
        let u = mb_uuid_uuid5(ns, name);
        // Known CPython vector verified against
        // `python3 -c "import uuid; print(uuid.uuid5(uuid.NAMESPACE_DNS, 'python.org'))"`.
        assert_eq!(
            str_of(mb_uuid_str(u)),
            "886313e1-3b8a-5372-9b90-0c9aee199e5d"
        );
        assert_eq!(mb_uuid_version_attr(u).as_int(), Some(5));
    }

    #[test]
    fn test_uuid_urn_format() {
        let u = mb_uuid_UUID(MbValue::from_ptr(MbObject::new_str(
            "550e8400e29b41d4a716446655440000".to_string(),
        )));
        assert_eq!(
            str_of(mb_uuid_urn(u)),
            "urn:uuid:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_uuid_bytes_attr() {
        let u = mb_uuid_UUID(MbValue::from_ptr(MbObject::new_str(
            "00112233445566778899aabbccddeeff".to_string(),
        )));
        let b = mb_uuid_bytes_attr(u);
        unsafe {
            if let ObjData::Bytes(ref bv) = (*b.as_ptr().unwrap()).data {
                assert_eq!(bv.len(), 16);
                assert_eq!(bv[0], 0x00);
                assert_eq!(bv[15], 0xff);
            } else {
                panic!("expected Bytes");
            }
        }
    }

    #[test]
    fn test_is_uuid_handle_distinguishes() {
        let u = mb_uuid_uuid4();
        let id = u.as_int().unwrap() as u64;
        assert!(is_uuid_handle(id));
        assert!(!is_uuid_handle(7));
    }

    #[test]
    fn test_getnode_multicast_bit() {
        // CPython convention: synthetic node has multicast bit set.
        let n = mb_uuid_getnode().as_int().unwrap();
        assert_ne!(n & (1i64 << 40), 0);
    }

    #[test]
    fn test_variant_rfc_4122() {
        let u = mb_uuid_uuid4();
        assert_eq!(str_of(mb_uuid_variant_attr(u)), "specified in RFC 4122");
    }
}
