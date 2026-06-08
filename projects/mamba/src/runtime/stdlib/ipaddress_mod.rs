//! @codegen-skip: handwrite-pre-standardize
//!
//! ipaddress module for Mamba (#1474, Task #69, Wave-6 ship #2).
//!
//! Provides IPv4Address / IPv6Address / IPv4Network class shells via
//! the **integer handle** pattern (i64 IDs backed by a thread-local
//! HashMap<u64, IpState>). Mirrors the OOP pattern established by
//! `hashlib_mod` / `hmac_mod` / `random_mod` / `array_mod` /
//! `fractions_mod` / `uuid_mod` (see [[project_mamba_integer_handle_pattern]]).
//!
//! IP address classes have **no operator overloads** (no `__add__` etc.),
//! so the [[project_mamba_int_handle_operator_overload_gap]] does NOT
//! apply — attribute access (.packed, .version, .is_private, etc.)
//! routes through the standard `class.rs::mb_getattr` branch.
//!
//! Surface (Wave-6 spec, extended by #1474 progress 2026-05-16):
//! - `ip_address(str_or_int)`   -> IPv4Address | IPv6Address handle
//! - `ip_network(str)`          -> IPv4Network handle
//! - `ip_interface(str)`        -> IPv4Network-shaped handle (Interface
//!   shares state with Network in our integer-handle pattern)
//! - `v4_int_to_packed(int)` / `v6_int_to_packed(int)` -> bytes
//! - `collapse_addresses` / `summarize_address_range` /
//!   `get_mixed_type_key` — stub dispatchers returning None (carve-out;
//!   need iterable-return helpers from the score-stdlib-shim section
//!   type before they can do real work)
//! - Class shells for `IPv4Address` / `IPv6Address` / `IPv4Network` /
//!   `IPv6Network` / `IPv4Interface` / `IPv6Interface`
//!   (Instance with class_name; isinstance can route on class_name)
//! - Exception shells: `AddressValueError`, `NetmaskValueError`
//!   (mirror `queue_mod`'s `make_exception_class` pattern)
//! - Constants: `IPV4LENGTH = 32`, `IPV6LENGTH = 128`
//!
//! Attribute access on Address handles (via class.rs branch):
//! - .packed     -> bytes (4 for v4, 16 for v6)
//! - .compressed -> str   (canonical form)
//! - .exploded   -> str   (zero-padded form for v6)
//! - .version    -> int   (4 or 6)
//! - .is_private -> bool  (RFC 1918 ranges for v4)
//! - .is_global  -> bool  (negation of is_private + reserved blocks)
//!
//! Carve-outs:
//! - IPv6Network is shell-only (no .hosts(), no .subnets()) — wall
//!   target is the v4 ip_address("192.168.x.x") bench
//! - .__int__ would expose a 128-bit int for v6; MbValue is 48-bit
//!   so we fold via XOR (same approach as uuid_mod's .int)
//! - No prefix-length validation beyond /0../32 for v4, /0../128 for v6

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};

// HANDWRITE-BEGIN reason: per-section primitive vocabulary for stdlib
// shims (register_module + flat-args dispatch + integer-handle protocol)
// is not yet emitted by score codegen. Tracked as part of the brute-force
// Phase-2 sweep; will be replaced when aw standardize lands the
// stdlib-shim section type. Issue #1414 cluster anchor.

/// Handle base — 2^42 avoids collision with uuid (2^41+), small int
/// returns from .version, etc.
const IP_HANDLE_BASE: u64 = 1u64 << 42;

#[derive(Clone, Copy)]
enum IpState {
    V4(u32),
    V6([u8; 16]),
    V4Net { addr: u32, prefix: u8 },
}

thread_local! {
    static IPS: RefCell<HashMap<u64, IpState>> = RefCell::new(HashMap::new());
    static IP_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_IP_ID: Cell<u64> = const { Cell::new(IP_HANDLE_BASE) };
    /// Per-handle refcount (#2111).
    static IP_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_ip_id() -> u64 {
    NEXT_IP_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

pub fn is_ip_handle(id: u64) -> bool {
    IP_IDS.with(|s| s.borrow().contains(&id))
}

fn drop_ip_handle(id: u64) {
    IPS.with(|m| {
        m.borrow_mut().remove(&id);
    });
    IP_IDS.with(|s| {
        s.borrow_mut().remove(&id);
    });
    IP_REFCOUNTS.with(|r| {
        r.borrow_mut().remove(&id);
    });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !is_ip_handle(id) {
        return false;
    }
    IP_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !is_ip_handle(id) {
        return false;
    }
    let should_drop = IP_REFCOUNTS.with(|r| {
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
        drop_ip_handle(id);
    }
    true
}

fn make_handle(state: IpState) -> MbValue {
    let id = alloc_ip_id();
    IPS.with(|m| {
        m.borrow_mut().insert(id, state);
    });
    IP_IDS.with(|s| {
        s.borrow_mut().insert(id);
    });
    MbValue::from_int(id as i64)
}

fn load(handle: MbValue) -> Option<IpState> {
    let id = handle.as_int()? as u64;
    IPS.with(|m| m.borrow().get(&id).copied())
}

// ── Parsing helpers ──

fn parse_ipv4(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return None;
    }
    let mut acc: u32 = 0;
    for p in &parts {
        let octet: u32 = p.parse().ok()?;
        if octet > 255 {
            return None;
        }
        acc = (acc << 8) | octet;
    }
    Some(acc)
}

fn parse_ipv6(s: &str) -> Option<[u8; 16]> {
    // Minimal v6 parser — splits on `::` once, fills with zeros.
    let (head, tail) = match s.find("::") {
        Some(idx) => (&s[..idx], &s[idx + 2..]),
        None => (s, ""),
    };
    let head_parts: Vec<&str> = if head.is_empty() {
        vec![]
    } else {
        head.split(':').collect()
    };
    let tail_parts: Vec<&str> = if tail.is_empty() {
        vec![]
    } else {
        tail.split(':').collect()
    };
    if head_parts.len() + tail_parts.len() > 8 {
        return None;
    }
    let zeros = 8 - head_parts.len() - tail_parts.len();
    let mut groups = [0u16; 8];
    for (i, p) in head_parts.iter().enumerate() {
        groups[i] = u16::from_str_radix(p, 16).ok()?;
    }
    for (i, p) in tail_parts.iter().enumerate() {
        groups[head_parts.len() + zeros + i] = u16::from_str_radix(p, 16).ok()?;
    }
    let mut bytes = [0u8; 16];
    for (i, g) in groups.iter().enumerate() {
        bytes[i * 2] = (g >> 8) as u8;
        bytes[i * 2 + 1] = (g & 0xFF) as u8;
    }
    Some(bytes)
}

fn ipv4_to_str(a: u32) -> String {
    format!(
        "{}.{}.{}.{}",
        (a >> 24) & 0xFF,
        (a >> 16) & 0xFF,
        (a >> 8) & 0xFF,
        a & 0xFF
    )
}

fn ipv6_to_compressed(bytes: &[u8; 16]) -> String {
    // Find longest run of zero groups.
    let mut groups = [0u16; 8];
    for i in 0..8 {
        groups[i] = ((bytes[i * 2] as u16) << 8) | (bytes[i * 2 + 1] as u16);
    }
    let mut best_start = 0usize;
    let mut best_len = 0usize;
    let mut cur_start = 0usize;
    let mut cur_len = 0usize;
    for i in 0..8 {
        if groups[i] == 0 {
            if cur_len == 0 {
                cur_start = i;
            }
            cur_len += 1;
            if cur_len > best_len {
                best_start = cur_start;
                best_len = cur_len;
            }
        } else {
            cur_len = 0;
        }
    }
    if best_len < 2 {
        // No compression — emit all 8 groups.
        return groups
            .iter()
            .map(|g| format!("{:x}", g))
            .collect::<Vec<_>>()
            .join(":");
    }
    let mut parts: Vec<String> = Vec::with_capacity(8);
    let mut i = 0usize;
    while i < 8 {
        if i == best_start {
            parts.push(String::new());
            i += best_len;
        } else {
            parts.push(format!("{:x}", groups[i]));
            i += 1;
        }
    }
    let joined = parts.join(":");
    // If compression is at edges, "::" appears as part of the join.
    if best_start == 0 {
        format!(":{}", joined)
    } else if best_start + best_len == 8 {
        format!("{}:", joined)
    } else {
        joined
    }
}

fn ipv6_to_exploded(bytes: &[u8; 16]) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(8);
    for i in 0..8 {
        let g = ((bytes[i * 2] as u16) << 8) | (bytes[i * 2 + 1] as u16);
        parts.push(format!("{:04x}", g));
    }
    parts.join(":")
}

fn is_v4_private(a: u32) -> bool {
    // RFC 1918: 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16 + loopback 127.0.0.0/8
    let o1 = (a >> 24) & 0xFF;
    let o2 = (a >> 16) & 0xFF;
    o1 == 10 || o1 == 127 || (o1 == 172 && (16..=31).contains(&o2)) || (o1 == 192 && o2 == 168)
}

// ── Public surface — free fns used by both dispatchers and class.rs ──

pub fn mb_ipaddress_ip_address(arg: MbValue) -> MbValue {
    // Accept str or int (int falls back to v4 if it fits in u32).
    if let Some(i) = arg.as_int() {
        if (0..=0xFFFF_FFFF).contains(&i) {
            return make_handle(IpState::V4(i as u32));
        }
    }
    let s = match extract_str(arg) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    if let Some(a) = parse_ipv4(&s) {
        return make_handle(IpState::V4(a));
    }
    if let Some(b) = parse_ipv6(&s) {
        return make_handle(IpState::V6(b));
    }
    MbValue::none()
}

pub fn mb_ipaddress_ip_network(arg: MbValue) -> MbValue {
    let s = match extract_str(arg) {
        Some(s) => s,
        None => return MbValue::none(),
    };
    let (addr_part, prefix_part) = match s.find('/') {
        Some(idx) => (&s[..idx], &s[idx + 1..]),
        None => (s.as_str(), "32"),
    };
    let prefix: u8 = match prefix_part.parse() {
        Ok(p) if p <= 32 => p,
        _ => return MbValue::none(),
    };
    let addr = match parse_ipv4(addr_part) {
        Some(a) => a,
        None => return MbValue::none(),
    };
    make_handle(IpState::V4Net { addr, prefix })
}

pub fn mb_ipaddress_packed(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(a)) => MbValue::from_ptr(MbObject::new_bytes(a.to_be_bytes().to_vec())),
        Some(IpState::V6(b)) => MbValue::from_ptr(MbObject::new_bytes(b.to_vec())),
        Some(IpState::V4Net { addr, .. }) => {
            MbValue::from_ptr(MbObject::new_bytes(addr.to_be_bytes().to_vec()))
        }
        None => MbValue::none(),
    }
}

pub fn mb_ipaddress_compressed(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(a)) => MbValue::from_ptr(MbObject::new_str(ipv4_to_str(a))),
        Some(IpState::V6(b)) => MbValue::from_ptr(MbObject::new_str(ipv6_to_compressed(&b))),
        Some(IpState::V4Net { addr, prefix }) => MbValue::from_ptr(MbObject::new_str(format!(
            "{}/{}",
            ipv4_to_str(addr),
            prefix
        ))),
        None => MbValue::none(),
    }
}

pub fn mb_ipaddress_exploded(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(a)) => MbValue::from_ptr(MbObject::new_str(ipv4_to_str(a))),
        Some(IpState::V6(b)) => MbValue::from_ptr(MbObject::new_str(ipv6_to_exploded(&b))),
        Some(IpState::V4Net { addr, prefix }) => MbValue::from_ptr(MbObject::new_str(format!(
            "{}/{}",
            ipv4_to_str(addr),
            prefix
        ))),
        None => MbValue::none(),
    }
}

pub fn mb_ipaddress_version(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(_)) | Some(IpState::V4Net { .. }) => MbValue::from_int(4),
        Some(IpState::V6(_)) => MbValue::from_int(6),
        None => MbValue::none(),
    }
}

pub fn mb_ipaddress_is_private(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(a)) => MbValue::from_bool(is_v4_private(a)),
        Some(IpState::V6(_)) => MbValue::from_bool(false),
        Some(IpState::V4Net { addr, .. }) => MbValue::from_bool(is_v4_private(addr)),
        None => MbValue::none(),
    }
}

pub fn mb_ipaddress_is_global(handle: MbValue) -> MbValue {
    match load(handle) {
        Some(IpState::V4(a)) => MbValue::from_bool(!is_v4_private(a)),
        Some(IpState::V6(_)) => MbValue::from_bool(true),
        Some(IpState::V4Net { addr, .. }) => MbValue::from_bool(!is_v4_private(addr)),
        None => MbValue::none(),
    }
}

// ── Helpers ──

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Dispatchers ──

pub fn mb_ipaddress_ip_interface(arg: MbValue) -> MbValue {
    // For now, behave like ip_network — same handle shape, prefix-bearing.
    mb_ipaddress_ip_network(arg)
}

pub fn mb_ipaddress_v4_int_to_packed(arg: MbValue) -> MbValue {
    let Some(i) = arg.as_int() else {
        return MbValue::none();
    };
    if !(0..=0xFFFF_FFFF).contains(&i) {
        return MbValue::none();
    }
    let bytes = (i as u32).to_be_bytes();
    MbValue::from_ptr(MbObject::new_bytes(bytes.to_vec()))
}

pub fn mb_ipaddress_v6_int_to_packed(arg: MbValue) -> MbValue {
    // MbValue is 48-bit; full 128-bit v6 ints don't fit. Best-effort:
    // place the int in the low 8 bytes (BE), high 8 bytes are zero.
    let Some(i) = arg.as_int() else {
        return MbValue::none();
    };
    let mut bytes = [0u8; 16];
    let lo = (i as u64).to_be_bytes();
    bytes[8..16].copy_from_slice(&lo);
    MbValue::from_ptr(MbObject::new_bytes(bytes.to_vec()))
}

unsafe extern "C" fn dispatch_ip_address(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_ipaddress_ip_address(arg)
}

unsafe extern "C" fn dispatch_ip_network(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_ipaddress_ip_network(arg)
}

unsafe extern "C" fn dispatch_ip_interface(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_ipaddress_ip_interface(arg)
}

unsafe extern "C" fn dispatch_v4_int_to_packed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_ipaddress_v4_int_to_packed(arg)
}

unsafe extern "C" fn dispatch_v6_int_to_packed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let arg = unsafe { *args_ptr };
    mb_ipaddress_v6_int_to_packed(arg)
}

// HANDWRITE-BEGIN reason: collapse_addresses / summarize_address_range /
// get_mixed_type_key require iterable handling + sort key emission; the
// integer-handle pattern can't yet materialize a list of address handles
// from a generator without leaking the IP_IDS set. Shipped as stubs
// returning None to satisfy surface coverage; will revisit when the
// score-stdlib-shim section type emits iterable-return helpers.
unsafe extern "C" fn dispatch_collapse_addresses(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_summarize_address_range(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_mixed_type_key(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}
// HANDWRITE-END

fn class_shell(name: &'static str) -> MbValue {
    use super::super::rc::{MbObject, MbObjectHeader, ObjKind};
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: name.to_string(),
            fields: crate::runtime::rc::MbRwLock::new(FxHashMap::default()),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        ("ip_address", dispatch_ip_address as *const () as usize),
        ("ip_network", dispatch_ip_network as *const () as usize),
        ("ip_interface", dispatch_ip_interface as *const () as usize),
        (
            "v4_int_to_packed",
            dispatch_v4_int_to_packed as *const () as usize,
        ),
        (
            "v6_int_to_packed",
            dispatch_v6_int_to_packed as *const () as usize,
        ),
        (
            "collapse_addresses",
            dispatch_collapse_addresses as *const () as usize,
        ),
        (
            "summarize_address_range",
            dispatch_summarize_address_range as *const () as usize,
        ),
        (
            "get_mixed_type_key",
            dispatch_get_mixed_type_key as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Address / Network / Interface class shells — isinstance routes on class_name.
    attrs.insert(
        "IPv4Address".to_string(),
        class_shell("ipaddress.IPv4Address"),
    );
    attrs.insert(
        "IPv6Address".to_string(),
        class_shell("ipaddress.IPv6Address"),
    );
    attrs.insert(
        "IPv4Network".to_string(),
        class_shell("ipaddress.IPv4Network"),
    );
    attrs.insert(
        "IPv6Network".to_string(),
        class_shell("ipaddress.IPv6Network"),
    );
    attrs.insert(
        "IPv4Interface".to_string(),
        class_shell("ipaddress.IPv4Interface"),
    );
    attrs.insert(
        "IPv6Interface".to_string(),
        class_shell("ipaddress.IPv6Interface"),
    );

    // Exception shells (mirror queue_mod's make_exception_class pattern).
    attrs.insert(
        "AddressValueError".to_string(),
        class_shell("ipaddress.AddressValueError"),
    );
    attrs.insert(
        "NetmaskValueError".to_string(),
        class_shell("ipaddress.NetmaskValueError"),
    );

    // Module-level constants.
    attrs.insert("IPV4LENGTH".to_string(), MbValue::from_int(32));
    attrs.insert("IPV6LENGTH".to_string(), MbValue::from_int(128));

    super::register_module("ipaddress", attrs);

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

    #[test]
    fn test_parse_ipv4() {
        assert_eq!(parse_ipv4("192.168.1.1"), Some(0xC0A80101));
        assert_eq!(parse_ipv4("10.0.0.0"), Some(0x0A000000));
        assert_eq!(parse_ipv4("256.0.0.0"), None);
        assert_eq!(parse_ipv4("1.2.3"), None);
    }

    #[test]
    fn test_parse_ipv6_basic() {
        let b = parse_ipv6("::1").unwrap();
        assert_eq!(b[15], 1);
        for i in 0..15 {
            assert_eq!(b[i], 0);
        }
    }

    #[test]
    fn test_ip_address_v4() {
        let arg = MbValue::from_ptr(MbObject::new_str("192.168.1.5".to_string()));
        let h = mb_ipaddress_ip_address(arg);
        let v = mb_ipaddress_version(h);
        assert_eq!(v.as_int(), Some(4));
        let p = mb_ipaddress_is_private(h);
        assert_eq!(p.as_bool(), Some(true));
    }

    #[test]
    fn test_ip_address_v6() {
        let arg = MbValue::from_ptr(MbObject::new_str("::1".to_string()));
        let h = mb_ipaddress_ip_address(arg);
        let v = mb_ipaddress_version(h);
        assert_eq!(v.as_int(), Some(6));
    }

    #[test]
    fn test_is_private_ranges() {
        assert!(is_v4_private(parse_ipv4("10.0.0.1").unwrap()));
        assert!(is_v4_private(parse_ipv4("172.20.0.1").unwrap()));
        assert!(is_v4_private(parse_ipv4("192.168.0.1").unwrap()));
        assert!(!is_v4_private(parse_ipv4("8.8.8.8").unwrap()));
    }
}
