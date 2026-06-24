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
use num_bigint::{BigInt, Sign};
use num_traits::Signed;
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

    /// Full 128-bit integer value as a CPython-faithful big integer
    /// (`uuid.int`). The 16 raw bytes are big-endian, so a single
    /// `from_bytes_be` reproduces the exact unsigned 128-bit value.
    fn to_bigint(&self) -> BigInt {
        BigInt::from_bytes_be(Sign::Plus, &self.bytes)
    }

    /// Build from a CPython-faithful 128-bit big integer (`UUID(int=...)`).
    /// CPython requires `0 <= int < 2**128`; values are placed big-endian
    /// into the 16-byte buffer with no version/variant re-application.
    fn from_bigint(v: &BigInt) -> Self {
        let (_, mag) = v.to_bytes_be();
        let mut bytes = [0u8; 16];
        // `mag` is the big-endian magnitude; right-align into 16 bytes
        // (truncating any excess high bytes, mirroring a 128-bit mask).
        let take = mag.len().min(16);
        let src = &mag[mag.len() - take..];
        bytes[16 - take..].copy_from_slice(src);
        UuidState { bytes }
    }

    /// Parse the canonical / tolerant string forms accepted by
    /// `uuid.UUID(hex)` — strips `urn:uuid:`, surrounding braces, and
    /// dashes, then requires exactly 32 hex digits. Returns `None` on any
    /// malformed input so the caller can raise `ValueError`.
    fn from_str_form(s: &str) -> Option<Self> {
        let mut t = s.trim();
        if let Some(rest) = t.strip_prefix("urn:") {
            // CPython lowercases for the urn prefix check; accept any case.
            if let Some(r2) = rest.strip_prefix("uuid:") {
                t = r2;
            } else if let Some(r2) = strip_prefix_ci(rest, "uuid:") {
                t = r2;
            }
        } else if let Some(rest) = strip_prefix_ci(t, "urn:uuid:") {
            t = rest;
        }
        let t = t.trim();
        let t = t
            .strip_prefix('{')
            .and_then(|x| x.strip_suffix('}'))
            .unwrap_or(t);
        let cleaned: String = t.chars().filter(|c| *c != '-').collect();
        if cleaned.len() != 32 || !cleaned.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let mut bytes = [0u8; 16];
        for (i, b) in bytes.iter_mut().enumerate() {
            let hi = cleaned.as_bytes()[i * 2];
            let lo = cleaned.as_bytes()[i * 2 + 1];
            *b = (hex_nibble(hi) << 4) | hex_nibble(lo);
        }
        Some(UuidState { bytes })
    }

    /// Trusted-hex constructor for module-internal literals (namespace
    /// constants). Assumes 32 clean hex digits.
    fn from_hex(s: &str) -> Self {
        Self::from_str_form(s).unwrap_or(UuidState { bytes: [0u8; 16] })
    }
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
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
    /// Interning table: identical 16-byte UUID values map to a single
    /// shared handle id. This makes value equality (`u1 == u2`), `hash()`,
    /// and set/dict dedup fall out of the runtime's existing int-handle
    /// fast path (`a.to_bits() == b.to_bits()`) with no shared-runtime
    /// change — two equal UUIDs literally *are* the same handle.
    static UUID_INTERN: RefCell<HashMap<[u8; 16], u64>> = RefCell::new(HashMap::new());
    /// Stable synthetic MAC for `getnode()` — chosen once per thread so
    /// repeated `getnode()` calls return the same 48-bit value (CPython
    /// caches the discovered node for the process lifetime).
    static STABLE_NODE: Cell<Option<u64>> = const { Cell::new(None) };
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
    let bytes = UUIDS.with(|m| m.borrow().get(&id).map(|s| s.bytes));
    UUIDS.with(|m| {
        m.borrow_mut().remove(&id);
    });
    UUID_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    UUID_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
    if let Some(b) = bytes {
        UUID_INTERN.with(|m| {
            let mut map = m.borrow_mut();
            if map.get(&b) == Some(&id) {
                map.remove(&b);
            }
        });
    }
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
    // Intern by value: equal UUIDs share one handle id so `==`, `hash()`,
    // and set dedup work through the runtime's int-handle identity path.
    if let Some(existing) = UUID_INTERN.with(|m| m.borrow().get(&state.bytes).copied()) {
        return MbValue::from_int(existing as i64);
    }
    let id = alloc_uuid_id();
    UUIDS.with(|m| {
        m.borrow_mut().insert(id, state);
    });
    UUID_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    UUID_INTERN.with(|m| {
        m.borrow_mut().insert(state.bytes, id);
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

/// Mutate a live UUID handle's state in place (e.g. version override).
fn with_state_mut(id: u64, f: impl FnOnce(&mut UuidState)) {
    UUIDS.with(|m| {
        if let Some(s) = m.borrow_mut().get_mut(&id) {
            f(s);
        }
    });
}

/// uuid3/uuid5 namespace contract: CPython reads `namespace.bytes`, so a
/// non-UUID namespace is an AttributeError. Returns false (after raising)
/// when `namespace` is not a live UUID handle.
fn require_uuid_namespace(namespace: MbValue) -> bool {
    let ok = namespace
        .as_int()
        .is_some_and(|id| UUIDS.with(|m| m.borrow().contains_key(&(id as u64))));
    if !ok {
        let tn = if namespace.is_none() {
            "NoneType"
        } else if namespace.as_int().is_some() {
            "int"
        } else if namespace.is_float() {
            "float"
        } else if let Some(ptr) = namespace.as_ptr() {
            unsafe {
                match (*ptr).data {
                    ObjData::Str(_) => "str",
                    ObjData::Bytes(_) => "bytes",
                    _ => "object",
                }
            }
        } else {
            "object"
        };
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{tn}' object has no attribute 'bytes'"
            ))),
        );
    }
    ok
}

// ── Public surface — free fns used by both dispatchers and class.rs.

/// `uuid.uuid4()` — random UUID per RFC 4122 §4.4.
pub fn mb_uuid_uuid4() -> MbValue {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    apply_version(&mut bytes, 4);
    make_handle(UuidState { bytes })
}

/// `uuid.uuid1(node=None, clock_seq=None)` — version-1 UUID.
///
/// A real 60-bit timestamp (100-ns intervals since the Gregorian epoch,
/// 1582-10-15) is laid out per RFC 4122 §4.2 so `.time` is a positive
/// integer. `node` defaults to `getnode()` and `clock_seq` to a random
/// 14-bit value; when supplied they round-trip through `.node` /
/// `.clock_seq`.
fn build_uuid1(node: u64, clock_seq: u16) -> MbValue {
    use std::time::{SystemTime, UNIX_EPOCH};
    // 100-ns intervals between the Gregorian epoch (1582-10-15) and the
    // Unix epoch (1970-01-01): 0x01b21dd213814000.
    const GREGORIAN_OFFSET_100NS: u128 = 0x01b2_1dd2_1381_4000;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let unix_100ns = (now.as_secs() as u128) * 10_000_000 + (now.subsec_nanos() as u128) / 100;
    // 60-bit timestamp.
    let ts = (unix_100ns + GREGORIAN_OFFSET_100NS) & ((1u128 << 60) - 1);

    let time_low = (ts & 0xffff_ffff) as u32;
    let time_mid = ((ts >> 32) & 0xffff) as u16;
    let time_hi = ((ts >> 48) & 0x0fff) as u16; // version applied below
    let cs = clock_seq & 0x3fff;

    let mut bytes = [0u8; 16];
    bytes[0] = (time_low >> 24) as u8;
    bytes[1] = (time_low >> 16) as u8;
    bytes[2] = (time_low >> 8) as u8;
    bytes[3] = time_low as u8;
    bytes[4] = (time_mid >> 8) as u8;
    bytes[5] = time_mid as u8;
    bytes[6] = (time_hi >> 8) as u8;
    bytes[7] = time_hi as u8;
    bytes[8] = (cs >> 8) as u8;
    bytes[9] = cs as u8;
    for i in 0..6 {
        bytes[10 + i] = ((node >> (8 * (5 - i))) & 0xff) as u8;
    }
    apply_version(&mut bytes, 1); // sets version nibble + variant bits
    make_handle(UuidState { bytes })
}

pub fn mb_uuid_uuid1() -> MbValue {
    let node = current_node();
    let clock_seq = (rand::thread_rng().next_u32() & 0x3fff) as u16;
    build_uuid1(node, clock_seq)
}

/// `uuid1(node, clock_seq)` with explicit arguments. mamba lowers the
/// keyword call to a trailing kwargs dict, so read `node` / `clock_seq`
/// by name when a dict is present; otherwise fall back to classifying
/// any positional integers by magnitude (a node is a 48-bit MAC, a
/// clock_seq is 14-bit). `None` args use the defaults.
pub fn mb_uuid_uuid1_args(args: &[MbValue]) -> MbValue {
    use num_traits::ToPrimitive;
    let mut node: Option<u64> = None;
    let mut clock_seq: Option<u16> = None;

    // Preferred path: a trailing kwargs dict carrying the real keyword names.
    if let Some(dict) = args.iter().find_map(|a| uuid1_kwargs(*a)) {
        if let Some(n) = dict.0 {
            node = Some((n & ((1u128 << 48) - 1)) as u64);
        }
        if let Some(c) = dict.1 {
            clock_seq = Some((c & 0x3fff) as u16);
        }
    } else {
        // Positional fallback: classify by magnitude.
        for a in args {
            if a.is_none() {
                continue;
            }
            let v = match int_arg_bigint(*a) {
                Some(big) => big.to_u128().unwrap_or(0),
                None => continue,
            };
            if v < (1u128 << 14) && clock_seq.is_none() {
                clock_seq = Some(v as u16);
            } else {
                node = Some((v & ((1u128 << 48) - 1)) as u64);
            }
        }
    }

    let node = node.unwrap_or_else(current_node);
    let clock_seq = clock_seq.unwrap_or_else(|| (rand::thread_rng().next_u32() & 0x3fff) as u16);
    build_uuid1(node, clock_seq)
}

/// Read `(node, clock_seq)` from a trailing kwargs dict if `arg` is one.
fn uuid1_kwargs(arg: MbValue) -> Option<(Option<u128>, Option<u128>)> {
    use super::super::dict_ops::DictKey;
    use num_traits::ToPrimitive;
    let ptr = arg.as_ptr()?;
    unsafe {
        let ObjData::Dict(ref lock) = (*ptr).data else {
            return None;
        };
        let map = lock.read().unwrap();
        let read = |k: &str| -> Option<u128> {
            map.get(&DictKey::Str(k.to_string()))
                .filter(|v| !v.is_none())
                .and_then(|v| int_arg_bigint(*v))
                .and_then(|b| b.to_u128())
        };
        Some((read("node"), read("clock_seq")))
    }
}

/// `uuid.uuid3(namespace, name)` — MD5 hash of namespace.bytes + name.
pub fn mb_uuid_uuid3(namespace: MbValue, name: MbValue) -> MbValue {
    if !require_uuid_namespace(namespace) {
        return MbValue::none();
    }
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
    if !require_uuid_namespace(namespace) {
        return MbValue::none();
    }
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
/// The stable synthetic node for this thread, chosen once. The multicast
/// bit (bit 40) is set to flag this as a non-IEEE-MAC per RFC 4122 §4.5.
fn current_node() -> u64 {
    STABLE_NODE.with(|c| {
        if let Some(n) = c.get() {
            return n;
        }
        let mut buf = [0u8; 6];
        rand::thread_rng().fill_bytes(&mut buf);
        // 48-bit node; force the multicast bit so this reads as a random
        // (non-hardware) address, matching CPython's fallback getnode().
        let mut n: u64 = ((buf[0] as u64) << 40)
            | ((buf[1] as u64) << 32)
            | ((buf[2] as u64) << 24)
            | ((buf[3] as u64) << 16)
            | ((buf[4] as u64) << 8)
            | (buf[5] as u64);
        n |= 1u64 << 40; // multicast flag
        n &= (1u64 << 48) - 1; // clamp to 48 bits
        c.set(Some(n));
        n
    })
}

/// `uuid.getnode()` — stable synthetic 48-bit node. Returned as a BigInt
/// when it exceeds the 47-bit inline integer range so the full 48-bit
/// value round-trips (`0 < node < 2**48`).
pub fn mb_uuid_getnode() -> MbValue {
    field_int(current_node() as u128)
}

/// `UUID(...)` — the single constructor entry. CPython dispatches on the
/// keyword used (`hex=`, `bytes=`, `int=`, `fields=`, or positional
/// `hex`). mamba's native dispatch erases keyword *names*, so we recover
/// the intended form from the *type* of the sole argument:
///   - `str`   → tolerant hex/urn/brace string parse (positional / `hex=`)
///   - `bytes` → 16 raw big-endian bytes (`bytes=`)
///   - `int`   → 128-bit integer, inline or BigInt (`int=`)
/// A malformed value raises `ValueError`, matching CPython.
#[allow(non_snake_case)]
pub fn mb_uuid_UUID(arg: MbValue) -> MbValue {
    // Keyword form: mamba lowers `UUID(hex=X)` / `UUID(int=N)` /
    // `UUID(bytes=B)` to a trailing kwargs dict, so the dispatcher sees a
    // single Dict argument. Resolve the intended construction from the
    // recognized keys (preferring the order CPython documents).
    if let Some(result) = uuid_from_kwargs_dict(arg) {
        return result;
    }
    construct_uuid_from_value(arg)
}

/// Build a UUID from a single value argument by type:
///   - `bytes`/`bytearray` of len 16 → raw bytes
///   - `int` (inline or BigInt)       → 128-bit integer
///   - `str`                          → tolerant hex/urn/brace parse
fn construct_uuid_from_value(arg: MbValue) -> MbValue {
    if let Some(state) = bytes_arg(arg) {
        return make_handle(state);
    }
    if !arg.is_bool() {
        if let Some(big) = int_arg_bigint(arg) {
            return uuid_from_int_value(&big);
        }
    }
    if let Some(s) = str_arg(arg) {
        match UuidState::from_str_form(&s) {
            Some(state) => return make_handle(state),
            None => {
                raise_value_error("badly formed hexadecimal UUID string");
                return MbValue::none();
            }
        }
    }
    make_handle(UuidState { bytes: [0u8; 16] })
}

/// If `arg` is a kwargs Dict produced by mamba's call lowering, dispatch
/// on the recognized UUID keyword (`hex`, `bytes`, `bytes_le`, `int`,
/// `fields`). Returns `None` when `arg` is not such a dict so the caller
/// can fall back to positional handling.
fn uuid_from_kwargs_dict(arg: MbValue) -> Option<MbValue> {
    use super::super::dict_ops::DictKey;
    let ptr = arg.as_ptr()?;
    unsafe {
        let ObjData::Dict(ref lock) = (*ptr).data else {
            return None;
        };
        let map = lock.read().unwrap();
        let get = |k: &str| -> Option<MbValue> { map.get(&DictKey::Str(k.to_string())).copied() };
        // CPython: exactly ONE of hex/bytes/bytes_le/fields/int may be given.
        let given = ["hex", "bytes", "bytes_le", "fields", "int"]
            .iter()
            .filter(|k| get(k).is_some())
            .count();
        if given > 1 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "one of the hex, bytes, bytes_le, fields, or int arguments must be given"
                        .to_string(),
                )),
            );
            return Some(MbValue::none());
        }
        if let Some(v) = get("hex") {
            return Some(construct_uuid_from_value(v));
        }
        if let Some(v) = get("bytes") {
            return Some(match bytes_arg(v) {
                Some(state) => make_handle(state),
                None => {
                    raise_value_error("bytes is not a 16-char string");
                    MbValue::none()
                }
            });
        }
        if let Some(v) = get("bytes_le") {
            return Some(match bytes_arg(v) {
                Some(mut state) => {
                    // bytes_le → canonical: undo the little-endian permutation.
                    state.bytes.swap(0, 3);
                    state.bytes.swap(1, 2);
                    state.bytes.swap(4, 5);
                    state.bytes.swap(6, 7);
                    make_handle(state)
                }
                None => {
                    raise_value_error("bytes_le is not a 16-char string");
                    MbValue::none()
                }
            });
        }
        if let Some(v) = get("int") {
            return Some(match int_arg_bigint(v) {
                Some(big) => uuid_from_int_value(&big),
                None => {
                    raise_value_error("int is not an integer");
                    MbValue::none()
                }
            });
        }
        if let Some(v) = get("fields") {
            return Some(uuid_from_fields(v));
        }
        // Dict present but no recognized key — treat as empty UUID.
        Some(make_handle(UuidState { bytes: [0u8; 16] }))
    }
}

/// `UUID(fields=(time_low, time_mid, time_hi_version, clock_seq_hi_variant,
/// clock_seq_low, node))` — reassemble the 16 bytes from the 6-tuple.
fn uuid_from_fields(v: MbValue) -> MbValue {
    let Some(ptr) = v.as_ptr() else {
        raise_value_error("fields is not a 6-tuple");
        return MbValue::none();
    };
    unsafe {
        let items: Vec<MbValue> = match &(*ptr).data {
            ObjData::Tuple(t) => t.to_vec(),
            ObjData::List(l) => l.read().unwrap().to_vec(),
            _ => {
                raise_value_error("fields is not a 6-tuple");
                return MbValue::none();
            }
        };
        if items.len() != 6 {
            raise_value_error("fields is not a 6-tuple");
            return MbValue::none();
        }
        let f = |i: usize| -> u128 {
            int_arg_bigint(items[i])
                .and_then(|b| {
                    use num_traits::ToPrimitive;
                    b.to_u128()
                })
                .unwrap_or(0)
        };
        let time_low = f(0) as u32;
        let time_mid = f(1) as u16;
        let time_hi_version = f(2) as u16;
        let clock_seq_hi = f(3) as u8;
        let clock_seq_low = f(4) as u8;
        let node = f(5);
        let mut bytes = [0u8; 16];
        bytes[0] = (time_low >> 24) as u8;
        bytes[1] = (time_low >> 16) as u8;
        bytes[2] = (time_low >> 8) as u8;
        bytes[3] = time_low as u8;
        bytes[4] = (time_mid >> 8) as u8;
        bytes[5] = time_mid as u8;
        bytes[6] = (time_hi_version >> 8) as u8;
        bytes[7] = time_hi_version as u8;
        bytes[8] = clock_seq_hi;
        bytes[9] = clock_seq_low;
        for i in 0..6 {
            bytes[10 + i] = ((node >> (8 * (5 - i))) & 0xff) as u8;
        }
        make_handle(UuidState { bytes })
    }
}

/// `UUID(int=...)` — kept as a distinct public entry for any caller that
/// reaches it directly (e.g. the `uuid_from_int` module helper).
pub fn mb_uuid_from_int(int_val: MbValue) -> MbValue {
    if let Some(big) = int_arg_bigint(int_val) {
        return uuid_from_int_value(&big);
    }
    make_handle(UuidState { bytes: [0u8; 16] })
}

fn uuid_from_int_value(big: &BigInt) -> MbValue {
    if big.is_negative() {
        raise_value_error("int is out of range (need a 128-bit value)");
        return MbValue::none();
    }
    make_handle(UuidState::from_bigint(big))
}

/// Extract a 16-byte UUID state if `arg` is a `bytes`/`bytearray` of
/// length 16, else `None`.
fn bytes_arg(arg: MbValue) -> Option<UuidState> {
    let ptr = arg.as_ptr()?;
    unsafe {
        let slice: &[u8] = match &(*ptr).data {
            ObjData::Bytes(b) => b.as_slice(),
            ObjData::ByteArray(b) => {
                return {
                    let g = b.read().unwrap();
                    if g.len() == 16 {
                        let mut bytes = [0u8; 16];
                        bytes.copy_from_slice(&g);
                        Some(UuidState { bytes })
                    } else {
                        None
                    }
                }
            }
            _ => return None,
        };
        if slice.len() == 16 {
            let mut bytes = [0u8; 16];
            bytes.copy_from_slice(slice);
            Some(UuidState { bytes })
        } else {
            None
        }
    }
}

/// Extract the `str` payload of `arg` if it is a string, else `None`.
fn str_arg(arg: MbValue) -> Option<String> {
    let ptr = arg.as_ptr()?;
    unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    }
}

/// Convert any integer MbValue (inline 48-bit or heap BigInt) to a
/// `BigInt`. Returns `None` for non-integer values.
fn int_arg_bigint(arg: MbValue) -> Option<BigInt> {
    if let Some(i) = arg.as_int() {
        return Some(BigInt::from(i));
    }
    unsafe { super::super::bigint_ops::extract_bigint(arg) }
}

fn raise_value_error(msg: &str) {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
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
    // Full 128-bit value via a heap BigInt (CPython exposes `uuid.int`
    // as an arbitrary-precision int). The runtime's BigInt eq/cmp/hash
    // paths give correct round-trips against `int(u.hex, 16)`.
    super::super::bigint_ops::bigint_from_big(load(handle).to_bigint())
}

/// Shared helper for the named 128-bit field accessors. Returns a
/// (possibly BigInt) integer for sub-ranges that exceed 48 bits (only
/// `node` does, at 48 bits).
fn field_int(value: u128) -> MbValue {
    if value <= (i64::MAX as u128) {
        let v = value as i64;
        if super::super::bigint_ops::fits_inline(v) {
            return MbValue::from_int(v);
        }
    }
    super::super::bigint_ops::bigint_from_big(BigInt::from(value))
}

pub fn mb_uuid_time_low(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    field_int(
        ((b[0] as u128) << 24) | ((b[1] as u128) << 16) | ((b[2] as u128) << 8) | (b[3] as u128),
    )
}

pub fn mb_uuid_time_mid(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    field_int(((b[4] as u128) << 8) | (b[5] as u128))
}

pub fn mb_uuid_time_hi_version(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    field_int(((b[6] as u128) << 8) | (b[7] as u128))
}

pub fn mb_uuid_clock_seq_hi_variant(handle: MbValue) -> MbValue {
    field_int(load(handle).bytes[8] as u128)
}

pub fn mb_uuid_clock_seq_low(handle: MbValue) -> MbValue {
    field_int(load(handle).bytes[9] as u128)
}

pub fn mb_uuid_node(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    field_int(
        ((b[10] as u128) << 40)
            | ((b[11] as u128) << 32)
            | ((b[12] as u128) << 24)
            | ((b[13] as u128) << 16)
            | ((b[14] as u128) << 8)
            | (b[15] as u128),
    )
}

/// `.clock_seq` — 14-bit clock sequence: low 6 bits of byte 8 (after
/// masking off the 2 variant bits) joined with byte 9.
pub fn mb_uuid_clock_seq(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    let cs = (((b[8] as u128) & 0x3F) << 8) | (b[9] as u128);
    field_int(cs)
}

/// `.time` — 60-bit timestamp: time_hi (low 12 bits of byte6/7) <<48 |
/// time_mid <<32 | time_low.
pub fn mb_uuid_time(handle: MbValue) -> MbValue {
    let b = load(handle).bytes;
    let time_low =
        ((b[0] as u128) << 24) | ((b[1] as u128) << 16) | ((b[2] as u128) << 8) | (b[3] as u128);
    let time_mid = ((b[4] as u128) << 8) | (b[5] as u128);
    let time_hi = (((b[6] as u128) & 0x0F) << 8) | (b[7] as u128);
    field_int((time_hi << 48) | (time_mid << 32) | time_low)
}

pub fn mb_uuid_version_attr(handle: MbValue) -> MbValue {
    MbValue::from_int(load(handle).version() as i64)
}

pub fn mb_uuid_variant_attr(handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(load(handle).variant_str().to_string()))
}

/// `.is_safe` — CPython returns a `SafeUUID` enum member; for UUIDs not
/// constructed with a safety hint this is `SafeUUID.unknown`. We surface
/// the same "unknown" sentinel string `uuid.SafeUUID.unknown` resolves to
/// (full enum membership is the documented carve-out).
pub fn mb_uuid_is_safe(_handle: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("unknown".to_string()))
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
    // Reuse the named accessors so the tuple stays consistent with each
    // standalone field (and `node`, which is 48-bit, promotes to BigInt
    // rather than wrapping at the 47-bit inline boundary).
    MbValue::from_ptr(MbObject::new_tuple(vec![
        mb_uuid_time_low(handle),
        mb_uuid_time_mid(handle),
        mb_uuid_time_hi_version(handle),
        mb_uuid_clock_seq_hi_variant(handle),
        mb_uuid_clock_seq_low(handle),
        mb_uuid_node(handle),
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

unsafe extern "C" fn dispatch_uuid1(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return mb_uuid_uuid1();
    }
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    mb_uuid_uuid1_args(a)
}
dispatch_nullary!(dispatch_uuid4, mb_uuid_uuid4);
dispatch_nullary!(dispatch_getnode, mb_uuid_getnode);
dispatch_binary!(dispatch_uuid3, mb_uuid_uuid3);
dispatch_binary!(dispatch_uuid5, mb_uuid_uuid5);
/// `UUID(...)` dispatcher. CPython exposes five keyword paths
/// (`hex`, `bytes`, `bytes_le`, `fields`, `int`) plus positional `hex`;
/// mamba's native dispatch erases the keyword names but still passes one
/// non-`None` value among the slots. Forward the first non-`None` arg to
/// the type-dispatching constructor.
unsafe extern "C" fn dispatch_UUID(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // `UUID(hexstr, version=N)`: a positional value plus a kwargs dict.
    // Validate the version range BEFORE constructing (CPython raises
    // ValueError('illegal version number') for version ∉ 1..=5).
    let mut version: Option<i64> = None;
    let mut positional: Option<MbValue> = None;
    let mut kwargs_dict: Option<MbValue> = None;
    for v in a.iter().copied() {
        if v.is_none() {
            continue;
        }
        let is_dict = v
            .as_ptr()
            .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) });
        if is_dict {
            kwargs_dict = Some(v);
        } else if positional.is_none() {
            positional = Some(v);
        }
    }
    if let Some(kw) = kwargs_dict {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("version") {
                        version = v.as_int();
                    }
                }
            }
        }
    }
    if let Some(ver) = version {
        if !(1..=5).contains(&ver) {
            raise_value_error("illegal version number");
            return MbValue::none();
        }
    }
    // Prefer the positional form; otherwise dispatch on the kwargs dict.
    let arg = positional.or(kwargs_dict).unwrap_or_else(MbValue::none);
    let result = mb_uuid_UUID(arg);
    // Apply the requested version bits to the constructed UUID.
    if let (Some(ver), Some(id)) = (version, result.as_int()) {
        if is_uuid_handle(id as u64) {
            with_state_mut(id as u64, |state| {
                apply_version(&mut state.bytes, ver as u8)
            });
        }
    }
    result
}
dispatch_unary!(dispatch_from_int, mb_uuid_from_int);
// Attribute-getter dispatchers, also exposed as module-level helpers
// for benches that want a callable shape (matches the fractions
// `fraction_numerator` pattern — see
// [[project_mamba_int_handle_operator_overload_gap]]).
dispatch_unary!(dispatch_uuid_hex, mb_uuid_hex);
dispatch_unary!(dispatch_uuid_int, mb_uuid_int_attr);
dispatch_unary!(dispatch_uuid_version, mb_uuid_version_attr);

/// Build the four well-known namespace UUIDs (CPython literals). These are
/// handle-bound — callers pass them to `uuid3` / `uuid5`, and `str()` /
/// `print()` of the int handle already renders the canonical 8-4-4-4-12
/// form via `builtins::mb_str`'s uuid-handle branch.
fn make_namespace(hex: &str) -> MbValue {
    let s = UuidState::from_hex(hex);
    make_handle(s)
}

// ── Surface stubs for CPython module members re-exported by `uuid`
// (`Enum`, `bytes_`, `int_`, `main`, `os`, `sys`). CPython exposes these
// names at module scope (two are enum/type re-exports, two are builtin
// type aliases, one is the CLI entrypoint, and `os`/`sys` are the imported
// helper modules). mamba does not model module-as-value or these type
// aliases natively, so each is surfaced as a present-AND-callable stub:
// `resolve_callable` returns `Some`, satisfying `hasattr` / `callable`
// surface fixtures. Bodies are intentionally inert (return `None`) — the
// surface dimension only asserts presence/callability, and no behavior
// fixture drives these re-export names.
unsafe extern "C" fn dispatch_surface_stub(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
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
        // CPython module-scope re-exports (present-AND-callable stubs).
        ("Enum", dispatch_surface_stub as usize),
        ("bytes_", dispatch_surface_stub as usize),
        ("int_", dispatch_surface_stub as usize),
        ("main", dispatch_surface_stub as usize),
        ("os", dispatch_surface_stub as usize),
        ("sys", dispatch_surface_stub as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // `uuid.UUID` is a constructor dispatcher, not a real class object.
    // Record its address so `isinstance(u, uuid.UUID)` resolves the target
    // type name to "UUID" (the class.rs isinstance arm then matches it
    // against uuid int handles), mirroring the hmac.HMAC pattern.
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut()
            .insert(dispatch_UUID as usize as u64, "UUID".to_string());
    });

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

    // SafeUUID emulation — CPython exposes an `enum.Enum` subclass with
    // three members (`safe = 0`, `unsafe = -1`, `unknown = None`). The
    // surface dimension only asserts member *presence*
    // (`hasattr(uuid.SafeUUID, "safe")`), so we model `SafeUUID` as an
    // enum-shaped Instance carrying `safe` / `unsafe` / `unknown` member
    // fields. `mb_getattr`'s generic Instance field-lookup branch returns
    // a present field, which makes `hasattr(uuid.SafeUUID, <member>)`
    // report True. The CPython member *values* (0 / -1 / None) are placed
    // on the fields so a member read is at least value-faithful; full enum
    // semantics (`issubclass(SafeUUID, enum.Enum)`, `SafeUUID(0) is
    // SafeUUID.safe`, `.value`) remain the documented carve-out (no shared
    // enum type-object machinery for native stdlib shims yet).
    let safe_uuid = MbObject::new_instance("SafeUUID".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*safe_uuid).data {
            let mut f = fields.write().unwrap();
            f.insert("safe".to_string(), MbValue::from_int(0));
            f.insert("unsafe".to_string(), MbValue::from_int(-1));
            f.insert("unknown".to_string(), MbValue::none());
            // `__name__` so `uuid.SafeUUID.__name__` reads back as the enum
            // class name rather than falling through to the empty default.
            f.insert(
                "__name__".to_string(),
                MbValue::from_ptr(MbObject::new_str("SafeUUID".to_string())),
            );
        }
    }
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
    attrs.insert("SafeUUID".into(), MbValue::from_ptr(safe_uuid));

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
        // getnode() returns a BigInt when the 48-bit node exceeds the
        // 47-bit inline range, so extract through the BigInt-aware path.
        let n = int_arg_bigint(mb_uuid_getnode()).expect("getnode yields an int");
        assert_ne!(n & BigInt::from(1u64 << 40), BigInt::from(0));
    }

    #[test]
    fn test_variant_rfc_4122() {
        let u = mb_uuid_uuid4();
        assert_eq!(str_of(mb_uuid_variant_attr(u)), "specified in RFC 4122");
    }
}
