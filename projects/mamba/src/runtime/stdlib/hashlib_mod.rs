//! @codegen-skip: handwrite-pre-standardize
//!
//! hashlib module for Mamba — Python 3.12 `hashlib` stdlib.
//!
//! Provides `md5`, `sha1`, `sha224`, `sha256`, `sha384`, `sha512`,
//! `sha3_224`, `sha3_256`, `sha3_384`, `sha3_512`, `blake2b`, `blake2s`
//! (subset) + `new(name, data=b"")` constructors returning hash objects
//! with `update()`, `hexdigest()`, `digest()`, `copy()`, plus the
//! `digest_size`, `block_size`, `name` attributes.
//!
//! HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
//! shims (register_module + flat-args dispatch + bytes-borrowing extract
//! + integer-handle protocol like file_io/generator) is not yet emitted
//! by score codegen. Tracked as part of the brute-force Phase-2 sweep;
//! will be replaced when aw standardize lands the stdlib-shim section
//! type. Issue #1414 cluster anchor.
//!
//! Implementation notes:
//!
//! - Real cryptographic digests from RustCrypto crates (`md-5`, `sha1`,
//!   `sha2`, `sha3`). The previous shim shipped a custom XOR-rotate
//!   "fake" hash that returned wrong values for any caller comparing
//!   against a known digest.
//!
//! - Hash objects are **integer handles** (i64 IDs), backed by a
//!   thread-local `HashMap<u64, MbHash>` table. This matches the
//!   working OOP-receiver pattern used by `file_io` (file handles)
//!   and `generator` (generator handles) — mamba's method-dispatch
//!   path in `class.rs::mb_call_method` already routes `obj.method(...)`
//!   on int receivers through per-protocol checks
//!   (`is_file_handle`, `is_known_generator`, now `is_hashlib_handle`).
//!
//! - Hash state holds a streaming `enum HasherState` from RustCrypto,
//!   so `h.update(buf)` does one streaming write per call — no O(n²)
//!   reallocation, no full buffer retention. `h.hexdigest()` is a
//!   `.clone().finalize()` so the same object can be queried multiple
//!   times and `h.copy()` returns a true independent clone.
//!
//! - Method dispatch uses the flat-args ABI (`extern "C" fn(args_ptr,
//!   nargs) -> MbValue`) — the legacy `dispatch_x(MbValue) -> MbValue`
//!   shape that the old shim used is a no-op against the current call
//!   convention (every dispatch was receiving garbage), same bug class
//!   as the struct shim's pre-rewrite state.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN

use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Sha3_224, Sha3_256, Sha3_384, Sha3_512, Shake128, Shake256};
// SHAKE XOF traits (variable-length extendable output).
use sha3::digest::{ExtendableOutput, Update as XofUpdate, XofReader};
// Real blake2 (RFC 7693). Default-size variants: blake2b → 64-byte digest,
// blake2s → 32-byte digest. Blake2bVar/Blake2sVar carry a runtime
// digest_size; Blake2bMac512/Blake2sMac256 are the keyed (MAC) construction.
use blake2::digest::{Update as VarUpdate, VariableOutput};
use blake2::{Blake2b512, Blake2bMac512, Blake2bVar, Blake2s256, Blake2sMac256, Blake2sVar};
// PBKDF2 (RFC 8018) is hand-rolled over the RustCrypto `hmac` crate (already
// a direct dependency) so `hashlib.pbkdf2_hmac(...)` needs no extra crate.
use hmac::{Hmac, Mac};

/// Algorithms exposed via `hashlib.<name>()` and `hashlib.new(<name>)`.
/// Mirrors CPython's `algorithms_guaranteed`, including the two SHAKE XOFs
/// (`shake_128`/`shake_256`). The SHAKE entries currently fall through to the
/// Md5 hasher state at the digest layer (variable-length XOF output is not yet
/// wired through `HasherState`), so only the surface contract — presence,
/// callability, and constructibility via `new()` — is honored for them today.
const ALGOS: &[&str] = &[
    "md5",
    "sha1",
    "sha224",
    "sha256",
    "sha384",
    "sha512",
    "sha3_224",
    "sha3_256",
    "sha3_384",
    "sha3_512",
    "blake2b",
    "blake2s",
    "shake_128",
    "shake_256",
];

fn digest_size(algo: &str) -> i64 {
    match algo {
        "md5" => 16,
        "sha1" => 20,
        "sha224" | "sha3_224" => 28,
        "sha256" | "sha3_256" | "blake2s" => 32,
        "sha384" | "sha3_384" => 48,
        "sha512" | "sha3_512" | "blake2b" => 64,
        // SHAKE XOFs report digest_size == 0 in CPython (variable output).
        "shake_128" | "shake_256" => 0,
        _ => 16,
    }
}

fn block_size(algo: &str) -> i64 {
    match algo {
        "md5" | "sha1" | "sha224" | "sha256" => 64,
        "sha384" | "sha512" => 128,
        "sha3_224" => 144,
        "sha3_256" => 136,
        "sha3_384" => 104,
        "sha3_512" => 72,
        "blake2b" => 128,
        "blake2s" => 64,
        "shake_128" => 168,
        "shake_256" => 136,
        _ => 64,
    }
}

/// Streaming hasher state. One variant per supported algorithm so
/// `update(buf)` is constant-overhead vs full buffer retention.
#[allow(clippy::large_enum_variant)]
enum HasherState {
    Md5(Md5),
    Sha1(Sha1),
    Sha224(Sha224),
    Sha256(Sha256),
    Sha384(Sha384),
    Sha512(Sha512),
    Sha3_224(Sha3_224),
    Sha3_256(Sha3_256),
    Sha3_384(Sha3_384),
    Sha3_512(Sha3_512),
    Blake2b(Blake2b512),
    Blake2s(Blake2s256),
    // Variable-length / keyed / XOF variants for the parameterized
    // constructors (`blake2b(digest_size=…)`, `blake2b(key=…)`,
    // `shake_128`/`shake_256`).
    Blake2bVar(Blake2bVar),
    Blake2sVar(Blake2sVar),
    Blake2bMac(Box<Blake2bMac512>),
    Blake2sMac(Box<Blake2sMac256>),
    Shake128(Shake128),
    Shake256(Shake256),
}

impl HasherState {
    fn new(algo: &str) -> Self {
        match algo {
            "md5" => HasherState::Md5(Md5::new()),
            "sha1" => HasherState::Sha1(Sha1::new()),
            "sha224" => HasherState::Sha224(Sha224::new()),
            "sha256" => HasherState::Sha256(Sha256::new()),
            "sha384" => HasherState::Sha384(Sha384::new()),
            "sha512" => HasherState::Sha512(Sha512::new()),
            "sha3_224" => HasherState::Sha3_224(Sha3_224::new()),
            "sha3_256" => HasherState::Sha3_256(Sha3_256::new()),
            "sha3_384" => HasherState::Sha3_384(Sha3_384::new()),
            "sha3_512" => HasherState::Sha3_512(Sha3_512::new()),
            "blake2b" => HasherState::Blake2b(Blake2b512::new()),
            "blake2s" => HasherState::Blake2s(Blake2s256::new()),
            "shake_128" => HasherState::Shake128(Shake128::default()),
            "shake_256" => HasherState::Shake256(Shake256::default()),
            _ => HasherState::Md5(Md5::new()),
        }
    }

    /// blake2 with optional `digest_size` / `key`. A key selects the keyed
    /// MAC construction (default output size); otherwise a digest_size
    /// selects the variable-output construction; absent both, the default.
    fn new_blake2(algo: &str, digest_size: Option<usize>, key: &[u8]) -> Self {
        let is_b = algo == "blake2b";
        if !key.is_empty() {
            return if is_b {
                HasherState::Blake2bMac(Box::new(
                    Blake2bMac512::new_from_slice(key).expect("blake2b key"),
                ))
            } else {
                HasherState::Blake2sMac(Box::new(
                    Blake2sMac256::new_from_slice(key).expect("blake2s key"),
                ))
            };
        }
        if let Some(n) = digest_size {
            return if is_b {
                HasherState::Blake2bVar(Blake2bVar::new(n).expect("blake2b size"))
            } else {
                HasherState::Blake2sVar(Blake2sVar::new(n).expect("blake2s size"))
            };
        }
        HasherState::new(algo)
    }

    fn update(&mut self, data: &[u8]) {
        match self {
            HasherState::Md5(h) => Digest::update(h, data),
            HasherState::Sha1(h) => Digest::update(h, data),
            HasherState::Sha224(h) => Digest::update(h, data),
            HasherState::Sha256(h) => Digest::update(h, data),
            HasherState::Sha384(h) => Digest::update(h, data),
            HasherState::Sha512(h) => Digest::update(h, data),
            HasherState::Sha3_224(h) => Digest::update(h, data),
            HasherState::Sha3_256(h) => Digest::update(h, data),
            HasherState::Sha3_384(h) => Digest::update(h, data),
            HasherState::Sha3_512(h) => Digest::update(h, data),
            HasherState::Blake2b(h) => Digest::update(h, data),
            HasherState::Blake2s(h) => Digest::update(h, data),
            HasherState::Blake2bVar(h) => VarUpdate::update(h, data),
            HasherState::Blake2sVar(h) => VarUpdate::update(h, data),
            HasherState::Blake2bMac(h) => Mac::update(h.as_mut(), data),
            HasherState::Blake2sMac(h) => Mac::update(h.as_mut(), data),
            HasherState::Shake128(h) => XofUpdate::update(h, data),
            HasherState::Shake256(h) => XofUpdate::update(h, data),
        }
    }

    /// Clone-then-finalize so the same hash object can be queried multiple
    /// times — CPython invariant: `h.hexdigest()` does not invalidate `h`.
    /// `shake_len` is the requested output length for the XOF variants
    /// (ignored by fixed/var/mac variants, whose size is intrinsic).
    fn finalize_clone(&self, shake_len: Option<usize>) -> Vec<u8> {
        match self {
            HasherState::Md5(h) => h.clone().finalize().to_vec(),
            HasherState::Sha1(h) => h.clone().finalize().to_vec(),
            HasherState::Sha224(h) => h.clone().finalize().to_vec(),
            HasherState::Sha256(h) => h.clone().finalize().to_vec(),
            HasherState::Sha384(h) => h.clone().finalize().to_vec(),
            HasherState::Sha512(h) => h.clone().finalize().to_vec(),
            HasherState::Sha3_224(h) => h.clone().finalize().to_vec(),
            HasherState::Sha3_256(h) => h.clone().finalize().to_vec(),
            HasherState::Sha3_384(h) => h.clone().finalize().to_vec(),
            HasherState::Sha3_512(h) => h.clone().finalize().to_vec(),
            HasherState::Blake2b(h) => h.clone().finalize().to_vec(),
            HasherState::Blake2s(h) => h.clone().finalize().to_vec(),
            HasherState::Blake2bVar(h) => {
                let n = VariableOutput::output_size(h);
                let mut out = vec![0u8; n];
                h.clone()
                    .finalize_variable(&mut out)
                    .expect("blake2b finalize");
                out
            }
            HasherState::Blake2sVar(h) => {
                let n = VariableOutput::output_size(h);
                let mut out = vec![0u8; n];
                h.clone()
                    .finalize_variable(&mut out)
                    .expect("blake2s finalize");
                out
            }
            HasherState::Blake2bMac(h) => h.as_ref().clone().finalize().into_bytes().to_vec(),
            HasherState::Blake2sMac(h) => h.as_ref().clone().finalize().into_bytes().to_vec(),
            HasherState::Shake128(h) => {
                let mut reader = h.clone().finalize_xof();
                let mut out = vec![0u8; shake_len.unwrap_or(32)];
                reader.read(&mut out);
                out
            }
            HasherState::Shake256(h) => {
                let mut reader = h.clone().finalize_xof();
                let mut out = vec![0u8; shake_len.unwrap_or(64)];
                reader.read(&mut out);
                out
            }
        }
    }

    fn clone_state(&self) -> Self {
        match self {
            HasherState::Md5(h) => HasherState::Md5(h.clone()),
            HasherState::Sha1(h) => HasherState::Sha1(h.clone()),
            HasherState::Sha224(h) => HasherState::Sha224(h.clone()),
            HasherState::Sha256(h) => HasherState::Sha256(h.clone()),
            HasherState::Sha384(h) => HasherState::Sha384(h.clone()),
            HasherState::Sha512(h) => HasherState::Sha512(h.clone()),
            HasherState::Sha3_224(h) => HasherState::Sha3_224(h.clone()),
            HasherState::Sha3_256(h) => HasherState::Sha3_256(h.clone()),
            HasherState::Sha3_384(h) => HasherState::Sha3_384(h.clone()),
            HasherState::Sha3_512(h) => HasherState::Sha3_512(h.clone()),
            HasherState::Blake2b(h) => HasherState::Blake2b(h.clone()),
            HasherState::Blake2s(h) => HasherState::Blake2s(h.clone()),
            HasherState::Blake2bVar(h) => HasherState::Blake2bVar(h.clone()),
            HasherState::Blake2sVar(h) => HasherState::Blake2sVar(h.clone()),
            HasherState::Blake2bMac(h) => HasherState::Blake2bMac(h.clone()),
            HasherState::Blake2sMac(h) => HasherState::Blake2sMac(h.clone()),
            HasherState::Shake128(h) => HasherState::Shake128(h.clone()),
            HasherState::Shake256(h) => HasherState::Shake256(h.clone()),
        }
    }
}

struct MbHash {
    algo: String,
    state: HasherState,
}

/// Base = 1<<45. Owns [1<<45, 1<<45 + 1<<44) (~17.6T ids) — sits
/// between array (1<<44) and hmac (1<<45 + 1<<44). The MbValue NaN-box
/// caps ids at (1<<47) - 1; see `integer_handle_registry::HANDLE_MIN_ID`.
const HASH_HANDLE_BASE: u64 = 1u64 << 45;

thread_local! {
    static HASHES: RefCell<HashMap<u64, MbHash>> = RefCell::new(HashMap::new());
    static HASH_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_HASH_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(HASH_HANDLE_BASE) };
    /// Per-handle refcount (#2111). Drops the HASHES entry when the count
    /// hits zero; without this, every per-iter `h = hashlib.sha256(...)`
    /// rebind leaked the prior digester state.
    static HASH_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_hash_id() -> u64 {
    NEXT_HASH_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

/// Class.rs `mb_call_method` calls this to decide whether to route
/// `int.method()` into the hashlib protocol or fall through to the
/// generic primitive int methods.
pub fn is_hashlib_handle(id: u64) -> bool {
    HASH_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_hash_handle(id: u64) {
    HASHES.with(|m| {
        m.borrow_mut().remove(&id);
    });
    HASH_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    HASH_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_hashlib_handle(id) {
        return false;
    }
    HASH_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111). Drops the
/// digester state when the per-handle refcount hits zero.
pub fn release_handle(id: u64) -> bool {
    if !is_hashlib_handle(id) {
        return false;
    }
    let should_drop = HASH_REFCOUNTS.with(|r| {
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
        drop_hash_handle(id);
    }
    true
}

fn make_handle(algo: &str) -> MbValue {
    make_handle_state(algo, HasherState::new(algo))
}

fn make_handle_state(algo: &str, state: HasherState) -> MbValue {
    let id = alloc_hash_id();
    HASHES.with(|m| {
        m.borrow_mut().insert(
            id,
            MbHash {
                algo: algo.to_string(),
                state,
            },
        );
    });
    HASH_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

/// blake2b / blake2s constructor honoring `digest_size=` and `key=`. mamba's
/// call lowering flattens those keyword VALUES into positional slots (blake2*
/// is a bare-ident builtin), so the dispatcher classifies each trailing arg
/// by type — an int is `digest_size`, a bytes-like value is `key` — and also
/// reads a trailing kwargs dict when one is present.
fn blake2_make(algo: &str, a: &[MbValue]) -> MbValue {
    let data = a.first().copied().unwrap_or_else(MbValue::none);
    if !data.is_none() && !is_bytes_like(data) {
        return raise_type_error("blake2 argument must be bytes-like");
    }
    let mut digest_size: Option<usize> = None;
    let mut key: Vec<u8> = Vec::new();
    for &arg in a.iter().skip(1) {
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let g = lock.read().unwrap();
                    if let Some(v) =
                        g.get(&super::super::dict_ops::DictKey::Str("digest_size".into()))
                    {
                        if let Some(n) = v.as_int() {
                            digest_size = Some(n.max(0) as usize);
                        }
                    }
                    if let Some(v) = g.get(&super::super::dict_ops::DictKey::Str("key".into())) {
                        key = with_bytes(*v, |b| b.to_vec());
                    }
                    continue;
                }
            }
        }
        if let Some(n) = arg.as_int() {
            digest_size = Some(n.max(0) as usize);
        } else if is_bytes_like(arg) {
            key = with_bytes(arg, |b| b.to_vec());
        }
    }
    let state = HasherState::new_blake2(algo, digest_size, &key);
    let h = make_handle_state(algo, state);
    if !data.is_none() {
        mb_hashlib_update(h, data);
    }
    h
}

/// Borrow `&[u8]` from an MbValue holding bytes/bytearray/str. Returns
/// an empty slice for other shapes. The slice does not outlive `f`.
#[inline]
fn raise_exc(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn with_bytes<R>(val: MbValue, f: impl FnOnce(&[u8]) -> R) -> R {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => return f(b.as_slice()),
                ObjData::ByteArray(lock) => {
                    let g = lock.read().unwrap();
                    return f(g.as_slice());
                }
                ObjData::Str(s) => return f(s.as_bytes()),
                _ => {}
            }
        }
    }
    f(&[])
}

#[inline]
fn is_bytes_like(val: MbValue) -> bool {
    match val.as_ptr() {
        Some(ptr) => unsafe { matches!(&(*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) },
        None => false,
    }
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
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

// ── Public surface — free functions called by the dispatch thunks
//    AND by method dispatch in class.rs::mb_call_method.

/// Construct a new hash handle for `algo`, optionally pre-seeded with `initial` bytes.
pub fn mb_hashlib_new_handle(algo: &str, initial: MbValue) -> MbValue {
    let h = make_handle(algo);
    if !initial.is_none() {
        mb_hashlib_update(h, initial);
    }
    h
}

/// `h.update(data)` — stream more bytes into the hasher state.
pub fn mb_hashlib_update(handle: MbValue, data: MbValue) -> MbValue {
    // CPython contracts: str must be encoded first; non-buffer types raise.
    let is_str = data
        .as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Str(_)) })
        .unwrap_or(false);
    if is_str {
        return raise_exc("TypeError", "Strings must be encoded before hashing");
    }
    let is_buffer = data
        .as_ptr()
        .map(|p| unsafe { matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false);
    if !is_buffer {
        return raise_exc("TypeError", "object supporting the buffer API required");
    }
    if let Some(id) = handle.as_int() {
        let id = id as u64;
        HASHES.with(|m| {
            if let Some(h) = m.borrow_mut().get_mut(&id) {
                with_bytes(data, |b| h.state.update(b));
            }
        });
    }
    MbValue::none()
}

/// Resolve the requested SHAKE output length: a SHAKE handle requires the
/// caller to pass a `length`. Returns Err (after raising) when a SHAKE handle
/// has no length, Ok(Some(n)) for a SHAKE call with a length, Ok(None) for a
/// fixed/var/mac handle (length intrinsic).
fn shake_length(handle: MbValue, length: Option<i64>, method: &str) -> Result<Option<usize>, ()> {
    let is_shake = handle.as_int().is_some_and(|id| {
        HASHES.with(|m| {
            m.borrow()
                .get(&(id as u64))
                .map(|h| h.algo.starts_with("shake_"))
                .unwrap_or(false)
        })
    });
    if is_shake {
        match length {
            Some(n) if n >= 0 => Ok(Some(n as usize)),
            _ => {
                raise_exc(
                    "TypeError",
                    &format!("{method}() missing required argument 'length' (pos 1)"),
                );
                Err(())
            }
        }
    } else {
        Ok(None)
    }
}

/// `h.hexdigest([length])` — finalize a clone, return lowercase hex string.
pub fn mb_hashlib_hexdigest_len(handle: MbValue, length: Option<i64>) -> MbValue {
    let shake_len = match shake_length(handle, length, "hexdigest") {
        Ok(n) => n,
        Err(()) => return MbValue::none(),
    };
    let bytes = handle
        .as_int()
        .and_then(|id| {
            HASHES.with(|m| {
                m.borrow()
                    .get(&(id as u64))
                    .map(|h| h.state.finalize_clone(shake_len))
            })
        })
        .unwrap_or_default();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in &bytes {
        use std::fmt::Write;
        let _ = write!(hex, "{:02x}", byte);
    }
    MbValue::from_ptr(MbObject::new_str(hex))
}

/// `h.hexdigest()` — back-compat zero-arg entry (non-SHAKE handles).
pub fn mb_hashlib_hexdigest(handle: MbValue) -> MbValue {
    mb_hashlib_hexdigest_len(handle, None)
}

/// `h.digest([length])` — finalize a clone, return raw digest bytes.
pub fn mb_hashlib_digest_len(handle: MbValue, length: Option<i64>) -> MbValue {
    let shake_len = match shake_length(handle, length, "digest") {
        Ok(n) => n,
        Err(()) => return MbValue::none(),
    };
    let bytes = handle
        .as_int()
        .and_then(|id| {
            HASHES.with(|m| {
                m.borrow()
                    .get(&(id as u64))
                    .map(|h| h.state.finalize_clone(shake_len))
            })
        })
        .unwrap_or_default();
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

/// `h.digest()` — back-compat zero-arg entry (non-SHAKE handles).
pub fn mb_hashlib_digest(handle: MbValue) -> MbValue {
    mb_hashlib_digest_len(handle, None)
}

/// `h.copy()` — return an independent hash handle with the same internal state.
pub fn mb_hashlib_copy(handle: MbValue) -> MbValue {
    if let Some(id) = handle.as_int() {
        let (algo, state) = HASHES.with(|m| {
            m.borrow()
                .get(&(id as u64))
                .map(|h| (h.algo.clone(), h.state.clone_state()))
                .unwrap_or_else(|| ("md5".to_string(), HasherState::new("md5")))
        });
        let new_id = alloc_hash_id();
        HASHES.with(|m| m.borrow_mut().insert(new_id, MbHash { algo, state }));
        HASH_IDS.with(|s| {
            s.borrow_mut().insert(new_id);
        });
        return MbValue::from_int(new_id as i64);
    }
    MbValue::none()
}

/// Read the algorithm name for a handle (used by class.rs to expose `h.name`).
pub fn mb_hashlib_name(handle: MbValue) -> MbValue {
    let name = handle
        .as_int()
        .and_then(|id| HASHES.with(|m| m.borrow().get(&(id as u64)).map(|h| h.algo.clone())))
        .unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(name))
}

/// Read `digest_size` for a handle (CPython attribute).
pub fn mb_hashlib_digest_size_attr(handle: MbValue) -> MbValue {
    let algo = handle
        .as_int()
        .and_then(|id| HASHES.with(|m| m.borrow().get(&(id as u64)).map(|h| h.algo.clone())))
        .unwrap_or_default();
    MbValue::from_int(digest_size(&algo))
}

/// Read `block_size` for a handle (CPython attribute).
pub fn mb_hashlib_block_size_attr(handle: MbValue) -> MbValue {
    let algo = handle
        .as_int()
        .and_then(|id| HASHES.with(|m| m.borrow().get(&(id as u64)).map(|h| h.algo.clone())))
        .unwrap_or_default();
    MbValue::from_int(block_size(&algo))
}

// ── Flat-args dispatch thunks (free-function entry points)

macro_rules! disp_algo {
    ($disp:ident, $algo:literal) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let data = a.first().copied().unwrap_or_else(MbValue::none);
            if !data.is_none() && !is_bytes_like(data) {
                return raise_type_error(concat!($algo, "() argument must be bytes-like"));
            }
            mb_hashlib_new_handle($algo, data)
        }
    };
}

disp_algo!(dispatch_md5, "md5");
disp_algo!(dispatch_sha1, "sha1");
disp_algo!(dispatch_sha224, "sha224");
disp_algo!(dispatch_sha256, "sha256");
disp_algo!(dispatch_sha384, "sha384");
disp_algo!(dispatch_sha512, "sha512");
disp_algo!(dispatch_sha3_224, "sha3_224");
disp_algo!(dispatch_sha3_256, "sha3_256");
disp_algo!(dispatch_sha3_384, "sha3_384");
disp_algo!(dispatch_sha3_512, "sha3_512");
unsafe extern "C" fn dispatch_blake2b(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    blake2_make("blake2b", a)
}
unsafe extern "C" fn dispatch_blake2s(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    blake2_make("blake2s", a)
}
disp_algo!(dispatch_shake_128, "shake_128");
disp_algo!(dispatch_shake_256, "shake_256");

unsafe extern "C" fn dispatch_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // CPython: the name must be a str — a non-str (`new(1)`) is a TypeError.
    let name_v = a.first().copied().unwrap_or_else(MbValue::none);
    let is_str = name_v
        .as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Str(_)) });
    if !is_str {
        return raise_type_error("new() argument 1 must be str, not int");
    }
    let mut algo = String::from("md5");
    with_str(name_v, |s| {
        if !s.is_empty() {
            algo = s.to_string();
        }
    });
    // CPython normalizes the algorithm name case (new("SHA256") works).
    let normalized = algo.to_lowercase();
    if !ALGOS.contains(&normalized.as_str()) {
        return raise_exc("ValueError", &format!("unsupported hash type {algo}"));
    }
    let algo = normalized;
    // CPython: new(name, data=b'', *, usedforsecurity=True). `usedforsecurity`
    // is keyword-only, but the native flat-args ABI flattens a keyword value
    // into the positional slice when `data` is omitted (e.g. `new(name,
    // usedforsecurity=False)` delivers the bool `False` here). Only honor the
    // 2nd positional as `data` when it is genuinely bytes-like; a non-bytes
    // value is the stray `usedforsecurity` keyword, not digest input, so it is
    // ignored rather than rejected.
    let arg1 = a.get(1).copied().unwrap_or_else(MbValue::none);
    let data = if is_bytes_like(arg1) {
        arg1
    } else {
        MbValue::none()
    };
    mb_hashlib_new_handle(&algo, data)
}

/// `hashlib.file_digest(fileobj, digest)` — read the file object's bytes and
/// feed them into the digest, returning the finalized hash object. `digest` is
/// either an algorithm name (str) or a zero-arg callable returning a fresh
/// hasher (e.g. `lambda: hashlib.sha256()`).
unsafe extern "C" fn dispatch_file_digest(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let fileobj = a.first().copied().unwrap_or_else(MbValue::none);
    let digest_arg = a.get(1).copied().unwrap_or_else(MbValue::none);
    // Read the whole file object (binary mode → bytes). On failure this is
    // None and the digest is left empty (the prior stub behavior).
    let data = super::super::file_io::mb_file_read(fileobj);

    let is_str = digest_arg
        .as_ptr()
        .map(|p| matches!((*p).data, ObjData::Str(_)))
        .unwrap_or(false);
    if is_str {
        let mut algo = String::from("sha256");
        with_str(digest_arg, |s| {
            if !s.is_empty() {
                algo = s.to_string();
            }
        });
        return mb_hashlib_new_handle(&algo, data);
    }
    // Callable digest: invoke with no args to obtain a fresh hasher, then feed
    // the file contents into it.
    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    let handle = super::super::builtins::mb_call_spread(digest_arg, empty);
    if !data.is_none() {
        mb_hashlib_update(handle, data);
    }
    handle
}

// ── PBKDF2-HMAC (RFC 8018) ────────────────────────────────────────────────
//
// `hashlib.pbkdf2_hmac(hash_name, password, salt, iterations, dklen=None)`.
// Hand-rolled over `hmac::Hmac<D>` so the only new crate cost is `blake2`.
// Positional-arg path only — the trailing `dklen=` keyword is dropped by the
// current native-dispatch ABI (kwargs never reach this thunk), so callers
// that pass `dklen=N` get the default output length. That last-mile keyword
// delivery is a shared call-lowering gap (reported in core_fix_needed).

/// PBKDF2-HMAC body, monomorphized per concrete HMAC type by `pbkdf2_for!`.
/// `h_len` is that HMAC's output size in bytes.
macro_rules! pbkdf2_for {
    ($hmac_ty:ty, $password:expr, $salt:expr, $rounds:expr, $dklen:expr, $h_len:expr) => {{
        let password: &[u8] = $password;
        let salt: &[u8] = $salt;
        let rounds: u32 = $rounds;
        let dklen: usize = $dklen;
        let h_len: usize = $h_len;
        let prf = <$hmac_ty>::new_from_slice(password).expect("HMAC accepts any key length");
        let mut out: Vec<u8> = Vec::with_capacity(dklen);
        let blocks = dklen.div_ceil(h_len);
        for block_index in 1..=blocks as u32 {
            // U1 = HMAC(password, salt || INT_32_BE(block_index))
            let mut mac = prf.clone();
            Mac::update(&mut mac, salt);
            Mac::update(&mut mac, &block_index.to_be_bytes());
            let mut u = mac.finalize().into_bytes();
            let mut t = u;
            for _ in 1..rounds {
                let mut mac = prf.clone();
                Mac::update(&mut mac, &u);
                u = mac.finalize().into_bytes();
                for (ti, ui) in t.iter_mut().zip(u.iter()) {
                    *ti ^= *ui;
                }
            }
            out.extend_from_slice(&t);
        }
        out.truncate(dklen);
        out
    }};
}

fn pbkdf2_dispatch_impl(
    hash_name: &str,
    password: &[u8],
    salt: &[u8],
    rounds: u32,
    dklen: Option<usize>,
) -> Option<Vec<u8>> {
    // Default dklen is the underlying digest's output size.
    let h_len = digest_size(hash_name) as usize;
    let dklen = dklen.unwrap_or(h_len);
    let out = match hash_name {
        "md5" => pbkdf2_for!(Hmac<Md5>, password, salt, rounds, dklen, h_len),
        "sha1" => pbkdf2_for!(Hmac<Sha1>, password, salt, rounds, dklen, h_len),
        "sha224" => pbkdf2_for!(Hmac<Sha224>, password, salt, rounds, dklen, h_len),
        "sha256" => pbkdf2_for!(Hmac<Sha256>, password, salt, rounds, dklen, h_len),
        "sha384" => pbkdf2_for!(Hmac<Sha384>, password, salt, rounds, dklen, h_len),
        "sha512" => pbkdf2_for!(Hmac<Sha512>, password, salt, rounds, dklen, h_len),
        _ => return None,
    };
    Some(out)
}

unsafe extern "C" fn dispatch_pbkdf2_hmac(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let mut hash_name = String::new();
    with_str(a.first().copied().unwrap_or_else(MbValue::none), |s| {
        hash_name = s.to_string()
    });
    if hash_name.is_empty() {
        return raise_type_error("pbkdf2_hmac() hash_name must be a string");
    }
    let password = a.get(1).copied().unwrap_or_else(MbValue::none);
    let salt = a.get(2).copied().unwrap_or_else(MbValue::none);
    if !is_bytes_like(password) || !is_bytes_like(salt) {
        return raise_type_error("pbkdf2_hmac() password and salt must be bytes-like");
    }
    let rounds = a.get(3).and_then(|v| v.as_int()).unwrap_or(0);
    if rounds < 1 {
        return raise_exc("ValueError", "iteration value must be greater than 0.");
    }
    // dklen: a 5th positional, OR a `dklen=` keyword arriving as a trailing
    // kwargs dict (the convention for `hashlib.pbkdf2_hmac(...)` attribute
    // calls).
    let dklen = {
        let mut d: Option<i64> = None;
        for &arg in a.iter().skip(4) {
            if let Some(ptr) = arg.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        if let Some(v) = lock
                            .read()
                            .unwrap()
                            .get(&super::super::dict_ops::DictKey::Str("dklen".into()))
                        {
                            d = v.as_int();
                        }
                        continue;
                    }
                }
            }
            if let Some(n) = arg.as_int() {
                d = Some(n);
            }
        }
        d.filter(|&n| n > 0).map(|n| n as usize)
    };
    let result = with_bytes(password, |pw| {
        with_bytes(salt, |st| {
            pbkdf2_dispatch_impl(&hash_name, pw, st, rounds as u32, dklen)
        })
    });
    match result {
        Some(bytes) => MbValue::from_ptr(MbObject::new_bytes(bytes)),
        None => raise_exc("ValueError", &format!("unsupported hash type {hash_name}")),
    }
}

// ── Module registration ──────────────────────────────────────────────────

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("md5", dispatch_md5 as usize),
        ("sha1", dispatch_sha1 as usize),
        ("sha224", dispatch_sha224 as usize),
        ("sha256", dispatch_sha256 as usize),
        ("sha384", dispatch_sha384 as usize),
        ("sha512", dispatch_sha512 as usize),
        ("sha3_224", dispatch_sha3_224 as usize),
        ("sha3_256", dispatch_sha3_256 as usize),
        ("sha3_384", dispatch_sha3_384 as usize),
        ("sha3_512", dispatch_sha3_512 as usize),
        ("blake2b", dispatch_blake2b as usize),
        ("blake2s", dispatch_blake2s as usize),
        ("shake_128", dispatch_shake_128 as usize),
        ("shake_256", dispatch_shake_256 as usize),
        ("new", dispatch_new as usize),
        ("pbkdf2_hmac", dispatch_pbkdf2_hmac as usize),
        ("file_digest", dispatch_file_digest as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // CPython exposes `algorithms_guaranteed` as a `frozenset` and
    // `algorithms_available` as a `set` of algorithm-name strings; the
    // surface fixtures assert `isinstance(x, (set, frozenset))` and that
    // guaranteed is a subset of available. We expose the same ALGOS names in
    // both so guaranteed == available (a valid subset) and every advertised
    // name is constructible via `new()`.
    let guaranteed: Vec<MbValue> = ALGOS
        .iter()
        .map(|a| MbValue::from_ptr(MbObject::new_str(a.to_string())))
        .collect();
    let available: Vec<MbValue> = ALGOS
        .iter()
        .map(|a| MbValue::from_ptr(MbObject::new_str(a.to_string())))
        .collect();
    attrs.insert(
        "algorithms_available".into(),
        MbValue::from_ptr(MbObject::new_set(available)),
    );
    attrs.insert(
        "algorithms_guaranteed".into(),
        MbValue::from_ptr(MbObject::new_frozenset(guaranteed)),
    );

    super::register_module("hashlib", attrs);

    // #2111: register retain/release hooks so per-iter rebinds
    // (`h = hashlib.sha256(...)` in a hot loop) drop prior digester state.
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

    fn bytes_of(val: MbValue) -> Vec<u8> {
        let mut out = Vec::new();
        with_bytes(val, |b| out.extend_from_slice(b));
        out
    }

    #[test]
    fn test_md5_known_vector_empty() {
        let h = mb_hashlib_new_handle("md5", MbValue::none());
        let d = mb_hashlib_hexdigest(h);
        assert_eq!(str_of(d), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn test_md5_known_vector_abc() {
        let data = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let h = mb_hashlib_new_handle("md5", data);
        let d = mb_hashlib_hexdigest(h);
        assert_eq!(str_of(d), "900150983cd24fb0d6963f7d28e17f72");
    }

    #[test]
    fn test_sha1_known_vector_abc() {
        let data = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let h = mb_hashlib_new_handle("sha1", data);
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
    }

    #[test]
    fn test_sha256_known_vector_abc() {
        let data = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let h = mb_hashlib_new_handle("sha256", data);
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_sha512_known_vector_abc() {
        let data = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let h = mb_hashlib_new_handle("sha512", data);
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
    }

    #[test]
    fn test_sha3_256_known_vector_empty() {
        let h = mb_hashlib_new_handle("sha3_256", MbValue::none());
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
        );
    }

    #[test]
    fn test_incremental_update_equals_oneshot() {
        let h1 = mb_hashlib_new_handle("sha256", MbValue::none());
        mb_hashlib_update(
            h1,
            MbValue::from_ptr(MbObject::new_bytes(b"hello ".to_vec())),
        );
        mb_hashlib_update(
            h1,
            MbValue::from_ptr(MbObject::new_bytes(b"world".to_vec())),
        );
        let d1 = mb_hashlib_hexdigest(h1);

        let h2 = mb_hashlib_new_handle(
            "sha256",
            MbValue::from_ptr(MbObject::new_bytes(b"hello world".to_vec())),
        );
        let d2 = mb_hashlib_hexdigest(h2);

        assert_eq!(str_of(d1), str_of(d2));
    }

    #[test]
    fn test_copy_is_independent() {
        let h = mb_hashlib_new_handle(
            "md5",
            MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec())),
        );
        let c = mb_hashlib_copy(h);
        // c keeps abc; h gets def appended
        mb_hashlib_update(h, MbValue::from_ptr(MbObject::new_bytes(b"def".to_vec())));
        let d_h = str_of(mb_hashlib_hexdigest(h));
        let d_c = str_of(mb_hashlib_hexdigest(c));
        assert_eq!(d_c, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(d_h, "e80b5017098950fc58aad83c8c14978e");
        assert_ne!(d_h, d_c);
    }

    #[test]
    fn test_repeated_hexdigest_stable() {
        let h = mb_hashlib_new_handle(
            "sha1",
            MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec())),
        );
        let d1 = str_of(mb_hashlib_hexdigest(h));
        let d2 = str_of(mb_hashlib_hexdigest(h));
        assert_eq!(d1, d2);
        assert_eq!(d1, "a9993e364706816aba3e25717850c26c9cd0d89d");
    }

    #[test]
    fn test_digest_returns_bytes() {
        let h = mb_hashlib_new_handle(
            "md5",
            MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec())),
        );
        let raw = mb_hashlib_digest(h);
        let bytes = bytes_of(raw);
        assert_eq!(bytes.len(), 16);
        // First byte of md5("abc") is 0x90.
        assert_eq!(bytes[0], 0x90);
    }

    #[test]
    fn test_update_with_bytes_input() {
        let h = mb_hashlib_new_handle("md5", MbValue::none());
        mb_hashlib_update(h, MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec())));
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }

    #[test]
    fn test_digest_size_attr() {
        let h = mb_hashlib_new_handle("sha256", MbValue::none());
        let n = mb_hashlib_digest_size_attr(h);
        assert_eq!(n.as_int(), Some(32));
    }

    #[test]
    fn test_new_with_algo_name() {
        let h = mb_hashlib_new_handle(
            "sha1",
            MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec())),
        );
        assert_eq!(
            str_of(mb_hashlib_hexdigest(h)),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(str_of(mb_hashlib_name(h)), "sha1");
    }

    #[test]
    fn test_is_hashlib_handle() {
        let h = mb_hashlib_new_handle("md5", MbValue::none());
        let id = h.as_int().unwrap() as u64;
        assert!(is_hashlib_handle(id));
        // Unknown id is not a handle.
        assert!(!is_hashlib_handle(u64::MAX));
    }
}
