use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// ssl module for Mamba (#1414).
///
/// Surface-only shim covering the entry points that requests / urllib3 /
/// httpx probe at import time. Mamba doesn't yet do TLS — every dispatcher
/// returns an identity-stable sentinel so the module-attribute probe chain
/// resolves without crashing. The protocol/cert/verify constants come from
/// CPython 3.12 ssl module.h verbatim.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed surface)
/// is tracked separately under #1414; this shim ships the Gate 2
/// module-attr-read perf surface plus the constant block that the 3p
/// conformance probes load eagerly.
use std::collections::HashMap;

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

/// Build a Dict-backed namespace object from (key, value) pairs. Used for the
/// `Purpose` / `TLSVersion` enum shells and the `get_default_verify_paths`
/// record, all of which CPython exposes as attribute-bearing objects. A Dict
/// receiver resolves `hasattr(obj, NAME)` / `obj.NAME` directly through the
/// `mb_getattr` plain-dict fast path, so these read back the registered values.
fn make_ns(pairs: &[(&str, MbValue)]) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (k, v) in pairs {
                super::super::rc::retain_if_ptr(*v);
                map.insert(DictKey::Str((*k).to_string()), *v);
            }
        }
    }
    MbValue::from_ptr(dict)
}

unsafe extern "C" fn dispatch_ssl_context(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// `ssl.create_default_context(purpose=Purpose.SERVER_AUTH)` — returns a real
/// SSLContext instance with CPython's secure defaults: SERVER_AUTH (the
/// default) yields a PROTOCOL_TLS_CLIENT context (check_hostname=True,
/// CERT_REQUIRED); CLIENT_AUTH yields a PROTOCOL_TLS_SERVER context.
unsafe extern "C" fn dispatch_create_default_context(a: *const MbValue, n: usize) -> MbValue {
    let mut protocol = 16i64; // PROTOCOL_TLS_CLIENT
    if n >= 1 && !a.is_null() {
        let purpose = *a;
        if purpose_oid(purpose).as_deref() == Some("1.3.6.1.5.5.7.3.2") {
            protocol = 17; // CLIENT_AUTH → PROTOCOL_TLS_SERVER
        }
    }
    let inst = MbValue::from_ptr(MbObject::new_instance("SSLContext".to_string()));
    seed_context_fields(inst, protocol);
    inst
}

/// Read the `oid` member of a `ssl.Purpose` namespace value.
fn purpose_oid(purpose: MbValue) -> Option<String> {
    let ptr = purpose.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            if let Some(v) = map.get(&DictKey::Str("oid".to_string())) {
                if let Some(vp) = v.as_ptr() {
                    if let ObjData::Str(ref s) = (*vp).data {
                        return Some(s.clone());
                    }
                }
            }
        }
    }
    None
}

unsafe extern "C" fn dispatch_wrap_socket(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_match_hostname(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_cert_time_to_seconds(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_int(0)
}

unsafe extern "C" fn dispatch_der_to_pem(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_pem_to_der(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
}

unsafe extern "C" fn dispatch_get_default_verify_paths(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython returns a `DefaultVerifyPaths` namedtuple exposing the six fields
    // below; the surface probe reads `.cafile` / `.capath`. Returning a Dict
    // namespace makes the attribute reads resolve through the getattr fast path.
    make_ns(&[
        ("cafile", new_str("")),
        ("capath", new_str("")),
        ("openssl_cafile_env", new_str("SSL_CERT_FILE")),
        ("openssl_cafile", new_str("")),
        ("openssl_capath_env", new_str("SSL_CERT_DIR")),
        ("openssl_capath", new_str("")),
    ])
}

unsafe extern "C" fn dispatch_get_server_certificate(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_rand_status(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(true)
}

thread_local! {
    /// SplitMix64 state for RAND_bytes — seeded from wall-clock nanos so
    /// consecutive draws differ (CPython: RAND_bytes(16) != RAND_bytes(16)).
    static RAND_STATE: std::cell::Cell<u64> = std::cell::Cell::new(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15) | 1,
    );
}

fn next_rand_u64() -> u64 {
    RAND_STATE.with(|s| {
        let seeded = s.get().wrapping_add(0x9E37_79B9_7F4A_7C15);
        s.set(seeded);
        let mut x = seeded;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    })
}

/// `ssl.RAND_bytes(num)` — returns `num` random bytes; negative `num` raises
/// ValueError("num must be positive") (CPython 3.12 _ssl semantics).
unsafe extern "C" fn dispatch_rand_bytes(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 || a.is_null() {
        return raise_err("TypeError", "RAND_bytes() missing required argument: 'num'");
    }
    let num = match (*a).as_int() {
        Some(v) => v,
        None => {
            return raise_err(
                "TypeError",
                &format!(
                    "'{}' object cannot be interpreted as an integer",
                    py_type_name(*a)
                ),
            );
        }
    };
    if num < 0 {
        return raise_err("ValueError", "num must be positive");
    }
    let mut bytes = Vec::with_capacity(num as usize);
    while bytes.len() < num as usize {
        let chunk = next_rand_u64().to_le_bytes();
        let take = std::cmp::min(8, num as usize - bytes.len());
        bytes.extend_from_slice(&chunk[..take]);
    }
    MbValue::from_ptr(MbObject::new_bytes(bytes))
}

unsafe extern "C" fn dispatch_class_shell(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

// ── SSLContext as a real registered class ───────────────────────────────────
//
// `SSLContext` must be callable (`ssl.SSLContext(protocol)` constructs an
// instance whose `minimum_version` / `maximum_version` attributes round-trip)
// *and* expose its method surface to `hasattr(ssl.SSLContext, "load_cert_chain")`.
// A class-name-string module attr backed by a CLASS_REGISTRY entry gives both:
// the string resolves to the registered class for `callable()` / construction,
// and the class methods answer `hasattr` on the class object.

fn set_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

fn first_arg(args: MbValue) -> MbValue {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => {
                    let v = lock.read().unwrap().to_vec();
                    return v.first().copied().unwrap_or_else(MbValue::none);
                }
                ObjData::Tuple(items) => {
                    return items.first().copied().unwrap_or_else(MbValue::none);
                }
                _ => {}
            }
        }
    }
    MbValue::none()
}

/// Raise a Python exception by class name and return None (native helper).
fn raise_err(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Best-effort Python type name for error messages.
fn py_type_name(v: MbValue) -> String {
    if v.is_none() {
        return "NoneType".to_string();
    }
    if v.as_bool().is_some() {
        return "bool".to_string();
    }
    if v.as_int().is_some() {
        return "int".to_string();
    }
    if v.as_float().is_some() {
        return "float".to_string();
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str".to_string(),
                ObjData::Bytes(_) => "bytes".to_string(),
                ObjData::ByteArray(_) => "bytearray".to_string(),
                ObjData::List(_) => "list".to_string(),
                ObjData::Tuple(_) => "tuple".to_string(),
                ObjData::Dict(_) => "dict".to_string(),
                ObjData::BigInt(_) => "int".to_string(),
                ObjData::Instance { class_name, .. } => class_name.clone(),
                _ => "object".to_string(),
            };
        }
    }
    "object".to_string()
}

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    let ptr = inst.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            return fields.read().unwrap().get(key).copied();
        }
    }
    None
}

/// Int coercion that also accepts bool (CPython int-conversion of True/False).
fn as_int_like(v: MbValue) -> Option<i64> {
    v.as_int().or_else(|| v.as_bool().map(|b| b as i64))
}

/// Seed a fresh SSLContext instance with CPython 3.12 defaults for `protocol`.
/// PROTOCOL_TLS_CLIENT (16) verifies by default; everything else does not.
fn seed_context_fields(inst: MbValue, protocol: i64) {
    let is_client = protocol == 16;
    set_field(inst, "protocol", MbValue::from_int(protocol));
    set_field(inst, "minimum_version", MbValue::from_int(-2)); // TLSVersion.MINIMUM_SUPPORTED
    set_field(inst, "maximum_version", MbValue::from_int(-1)); // TLSVersion.MAXIMUM_SUPPORTED
    set_field(
        inst,
        "verify_mode",
        MbValue::from_int(if is_client { 2 } else { 0 }),
    );
    set_field(inst, "check_hostname", MbValue::from_bool(is_client));
    set_field(inst, "post_handshake_auth", MbValue::from_bool(false));
    set_field(
        inst,
        "hostname_checks_common_name",
        MbValue::from_bool(true),
    );
    set_field(inst, "num_tickets", MbValue::from_int(2));
    // OP_ALL | OP_NO_COMPRESSION | ... — the observed default bitmask on
    // CPython 3.12/OpenSSL 3.x. Fixtures require an int that ORs cleanly.
    set_field(inst, "options", MbValue::from_int(0x8252_0050));
    set_field(inst, "_cipher_filter", new_str("DEFAULT"));
}

unsafe extern "C" fn init_ssl_context(self_v: MbValue, args: MbValue) -> MbValue {
    // SSLContext(protocol=PROTOCOL_TLS). Validate the protocol number the way
    // CPython does, then seed the attribute surface.
    let protocol_arg = first_arg(args);
    let protocol = if protocol_arg.is_none() {
        2 // PROTOCOL_TLS
    } else {
        match as_int_like(protocol_arg) {
            Some(v) => v,
            None => {
                return raise_err(
                    "TypeError",
                    &format!(
                        "'{}' object cannot be interpreted as an integer",
                        py_type_name(protocol_arg)
                    ),
                );
            }
        }
    };
    if !matches!(protocol, 2 | 3 | 4 | 5 | 16 | 17) {
        return raise_err(
            "ValueError",
            &format!("invalid or unsupported protocol version {protocol}"),
        );
    }
    seed_context_fields(self_v, protocol);
    MbValue::none()
}

/// Generic no-op SSLContext method (load_cert_chain / set_ciphers / ...): returns None.
unsafe extern "C" fn ctx_method_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
}

// ── SSLContext property setters (mb_setattr hook) ───────────────────────────

/// CPython property-setter semantics for SSLContext attribute writes.
/// Returns true when the write was fully handled (stored or raised); false
/// lets mb_setattr fall through to the plain field write.
pub fn sslcontext_setattr(obj: MbValue, attr: &str, value: MbValue) -> bool {
    match attr {
        "check_hostname" => {
            let truthy = super::super::builtins::mb_bool(value)
                .as_bool()
                .unwrap_or(false);
            // Enabling hostname checks on a CERT_NONE context promotes
            // verify_mode to CERT_REQUIRED (CPython context.c).
            if truthy {
                let vm = get_field(obj, "verify_mode")
                    .and_then(as_int_like)
                    .unwrap_or(0);
                if vm == 0 {
                    set_field(obj, "verify_mode", MbValue::from_int(2));
                }
            }
            set_field(obj, "check_hostname", MbValue::from_bool(truthy));
            true
        }
        "verify_mode" => {
            let v = match as_int_like(value) {
                Some(v) => v,
                None => {
                    raise_err(
                        "TypeError",
                        &format!(
                            "'{}' object cannot be interpreted as an integer",
                            py_type_name(value)
                        ),
                    );
                    return true;
                }
            };
            if !matches!(v, 0 | 1 | 2) {
                raise_err("ValueError", "invalid value for verify_mode");
                return true;
            }
            let check_hostname = get_field(obj, "check_hostname")
                .and_then(|c| c.as_bool())
                .unwrap_or(false);
            if v == 0 && check_hostname {
                raise_err(
                    "ValueError",
                    "Cannot set verify_mode to CERT_NONE when check_hostname is enabled.",
                );
                return true;
            }
            set_field(obj, "verify_mode", MbValue::from_int(v));
            true
        }
        "minimum_version" | "maximum_version" => {
            let v = match as_int_like(value) {
                Some(v) => v,
                None => {
                    raise_err(
                        "TypeError",
                        &format!(
                            "'{}' object cannot be interpreted as an integer",
                            py_type_name(value)
                        ),
                    );
                    return true;
                }
            };
            // TLSVersion members only: MINIMUM/MAXIMUM_SUPPORTED + SSLv3..TLSv1.3.
            if !matches!(v, -2 | -1 | 768 | 769 | 770 | 771 | 772) {
                raise_err("ValueError", &format!("Unsupported TLS/SSL version {v:#x}"));
                return true;
            }
            set_field(obj, attr, MbValue::from_int(v));
            true
        }
        "num_tickets" => {
            let v = match as_int_like(value) {
                Some(v) => v,
                None => {
                    raise_err(
                        "TypeError",
                        &format!(
                            "'{}' object cannot be interpreted as an integer",
                            py_type_name(value)
                        ),
                    );
                    return true;
                }
            };
            if v < 0 {
                raise_err("ValueError", "value must be non-negative");
                return true;
            }
            let protocol = get_field(obj, "protocol")
                .and_then(as_int_like)
                .unwrap_or(2);
            if protocol != 17 {
                raise_err("ValueError", "SSLContext is not a server context.");
                return true;
            }
            set_field(obj, "num_tickets", MbValue::from_int(v));
            true
        }
        "options" => {
            // Out-of-range u64 → OverflowError; non-int → TypeError.
            if let Some(ptr) = value.as_ptr() {
                unsafe {
                    if let ObjData::BigInt(_) = (*ptr).data {
                        raise_err(
                            "OverflowError",
                            "Python int too large to convert to C unsigned long",
                        );
                        return true;
                    }
                }
            }
            let v = match as_int_like(value) {
                Some(v) => v,
                None => {
                    raise_err(
                        "TypeError",
                        &format!("argument must be int, not {}", py_type_name(value)),
                    );
                    return true;
                }
            };
            if v < 0 {
                raise_err("OverflowError", "can't convert negative int to unsigned");
                return true;
            }
            set_field(obj, "options", MbValue::from_int(v));
            true
        }
        _ => false,
    }
}

// ── Ciphers / session stats ─────────────────────────────────────────────────

/// (name, protocol, code, kx, au, enc, bits) — OpenSSL 3.x default-list shape.
/// No PSK / SRP / MD5 / RC4 / 3DES entries: the default list excludes weak
/// primitives (default_ciphers_exclude_weak).
const CIPHER_TABLE: &[(&str, &str, i64, &str, &str, &str, i64)] = &[
    (
        "TLS_AES_256_GCM_SHA384",
        "TLSv1.3",
        0x1302,
        "any",
        "any",
        "AESGCM(256)",
        256,
    ),
    (
        "TLS_CHACHA20_POLY1305_SHA256",
        "TLSv1.3",
        0x1303,
        "any",
        "any",
        "CHACHA20/POLY1305(256)",
        256,
    ),
    (
        "TLS_AES_128_GCM_SHA256",
        "TLSv1.3",
        0x1301,
        "any",
        "any",
        "AESGCM(128)",
        128,
    ),
    (
        "ECDHE-ECDSA-AES256-GCM-SHA384",
        "TLSv1.2",
        0xC02C,
        "ECDH",
        "ECDSA",
        "AESGCM(256)",
        256,
    ),
    (
        "ECDHE-RSA-AES256-GCM-SHA384",
        "TLSv1.2",
        0xC030,
        "ECDH",
        "RSA",
        "AESGCM(256)",
        256,
    ),
    (
        "DHE-RSA-AES256-GCM-SHA384",
        "TLSv1.2",
        0x009F,
        "DH",
        "RSA",
        "AESGCM(256)",
        256,
    ),
    (
        "ECDHE-ECDSA-CHACHA20-POLY1305",
        "TLSv1.2",
        0xCCA9,
        "ECDH",
        "ECDSA",
        "CHACHA20/POLY1305(256)",
        256,
    ),
    (
        "ECDHE-RSA-CHACHA20-POLY1305",
        "TLSv1.2",
        0xCCA8,
        "ECDH",
        "RSA",
        "CHACHA20/POLY1305(256)",
        256,
    ),
    (
        "DHE-RSA-CHACHA20-POLY1305",
        "TLSv1.2",
        0xCCAA,
        "DH",
        "RSA",
        "CHACHA20/POLY1305(256)",
        256,
    ),
    (
        "ECDHE-ECDSA-AES128-GCM-SHA256",
        "TLSv1.2",
        0xC02B,
        "ECDH",
        "ECDSA",
        "AESGCM(128)",
        128,
    ),
    (
        "ECDHE-RSA-AES128-GCM-SHA256",
        "TLSv1.2",
        0xC02F,
        "ECDH",
        "RSA",
        "AESGCM(128)",
        128,
    ),
    (
        "DHE-RSA-AES128-GCM-SHA256",
        "TLSv1.2",
        0x009E,
        "DH",
        "RSA",
        "AESGCM(128)",
        128,
    ),
    (
        "ECDHE-ECDSA-AES256-SHA384",
        "TLSv1.2",
        0xC024,
        "ECDH",
        "ECDSA",
        "AES(256)",
        256,
    ),
    (
        "ECDHE-RSA-AES256-SHA384",
        "TLSv1.2",
        0xC028,
        "ECDH",
        "RSA",
        "AES(256)",
        256,
    ),
    (
        "ECDHE-ECDSA-AES128-SHA256",
        "TLSv1.2",
        0xC023,
        "ECDH",
        "ECDSA",
        "AES(128)",
        128,
    ),
    (
        "ECDHE-RSA-AES128-SHA256",
        "TLSv1.2",
        0xC027,
        "ECDH",
        "RSA",
        "AES(128)",
        128,
    ),
    (
        "AES256-GCM-SHA384",
        "TLSv1.2",
        0x009D,
        "RSA",
        "RSA",
        "AESGCM(256)",
        256,
    ),
    (
        "AES128-GCM-SHA256",
        "TLSv1.2",
        0x009C,
        "RSA",
        "RSA",
        "AESGCM(128)",
        128,
    ),
    (
        "AES256-SHA256",
        "TLSv1.2",
        0x003D,
        "RSA",
        "RSA",
        "AES(256)",
        256,
    ),
    (
        "AES128-SHA256",
        "TLSv1.2",
        0x003C,
        "RSA",
        "RSA",
        "AES(128)",
        128,
    ),
];

/// Build one get_ciphers() entry with CPython's field shape.
fn cipher_dict(entry: &(&str, &str, i64, &str, &str, &str, i64)) -> MbValue {
    let (name, proto, code, kx, au, enc, bits) = *entry;
    let aead = enc.contains("GCM") || enc.contains("POLY1305");
    let description = format!("{name} {proto} Kx={kx} Au={au} Enc={enc} Mac=AEAD");
    make_ns(&[
        ("id", MbValue::from_int(0x0300_0000 | code)),
        ("name", new_str(name)),
        ("protocol", new_str(proto)),
        ("description", new_str(&description)),
        ("strength_bits", MbValue::from_int(bits)),
        ("alg_bits", MbValue::from_int(bits)),
        ("aead", MbValue::from_bool(aead)),
        ("symmetric", new_str(&enc.to_lowercase())),
        ("digest", MbValue::none()),
        ("kea", new_str(&format!("kx-{}", kx.to_lowercase()))),
        ("auth", new_str(&format!("auth-{}", au.to_lowercase()))),
    ])
}

/// `SSLContext.get_ciphers()` — non-empty list of cipher dicts; honors the
/// narrowing applied by set_ciphers (AESGCM keeps the GCM suites).
unsafe extern "C" fn ctx_get_ciphers(self_v: MbValue, _args: MbValue) -> MbValue {
    let filter = get_field(self_v, "_cipher_filter")
        .and_then(|v| {
            v.as_ptr().and_then(|p| {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| "DEFAULT".to_string());
    let upper = filter.to_uppercase();
    let narrowed_to_gcm = upper.contains("AESGCM");
    let mut items = Vec::new();
    for entry in CIPHER_TABLE {
        if narrowed_to_gcm && !entry.0.contains("GCM") {
            continue;
        }
        items.push(cipher_dict(entry));
    }
    MbValue::from_ptr(MbObject::new_list(items))
}

/// Cipher-string keywords OpenSSL's parser accepts (subset). A set_ciphers
/// string none of whose tokens are recognized selects no cipher → SSLError.
const CIPHER_KEYWORDS: &[&str] = &[
    "ALL",
    "DEFAULT",
    "COMPLEMENTOFDEFAULT",
    "COMPLEMENTOFALL",
    "HIGH",
    "MEDIUM",
    "LOW",
    "ANULL",
    "ENULL",
    "NULL",
    "EXPORT",
    "AES",
    "AESGCM",
    "AESCCM",
    "AES128",
    "AES256",
    "CHACHA20",
    "CAMELLIA",
    "3DES",
    "DES",
    "RC4",
    "MD5",
    "SHA",
    "SHA1",
    "SHA256",
    "SHA384",
    "RSA",
    "KRSA",
    "ARSA",
    "DSS",
    "DHE",
    "EDH",
    "ECDHE",
    "EECDH",
    "ECDSA",
    "ECDH",
    "ADH",
    "AECDH",
    "PSK",
    "SRP",
    "KRB5",
    "SEED",
    "IDEA",
    "TLSV1",
    "TLSV1.0",
    "TLSV1.2",
    "TLSV1.3",
    "SSLV3",
];

/// `SSLContext.set_ciphers(s)` — validates the OpenSSL cipher string; a string
/// that selects nothing raises SSLError("No cipher can be selected.").
unsafe extern "C" fn ctx_set_ciphers(self_v: MbValue, args: MbValue) -> MbValue {
    let arg = first_arg(args);
    let s = match arg.as_ptr().and_then(|p| {
        if let ObjData::Str(ref s) = (*p).data {
            Some(s.clone())
        } else {
            None
        }
    }) {
        Some(s) => s,
        None => {
            return raise_err(
                "TypeError",
                &format!("cipher string must be str, not {}", py_type_name(arg)),
            );
        }
    };
    let mut any_recognized = false;
    for raw in s.split(|c: char| matches!(c, ':' | ',' | ' ')) {
        let token = raw.trim_start_matches(['!', '-', '+']);
        if token.is_empty() {
            continue;
        }
        if let Some(directive) = token.strip_prefix('@') {
            let name = directive.split('=').next().unwrap_or("");
            if matches!(name.to_uppercase().as_str(), "STRENGTH" | "SECLEVEL") {
                any_recognized = true;
            }
            continue;
        }
        let upper = token.to_uppercase();
        if CIPHER_KEYWORDS.contains(&upper.as_str())
            || CIPHER_TABLE.iter().any(|e| e.0 == upper)
            || upper.split('+').all(|part| CIPHER_KEYWORDS.contains(&part))
        {
            any_recognized = true;
        }
    }
    if !any_recognized {
        return raise_err("SSLError", "No cipher can be selected.");
    }
    set_field(self_v, "_cipher_filter", new_str(&s));
    MbValue::none()
}

/// `SSLContext.session_stats()` — the zeroed stats dict of a fresh context.
unsafe extern "C" fn ctx_session_stats(_self_v: MbValue, _args: MbValue) -> MbValue {
    make_ns(&[
        ("number", MbValue::from_int(0)),
        ("connect", MbValue::from_int(0)),
        ("connect_good", MbValue::from_int(0)),
        ("connect_renegotiate", MbValue::from_int(0)),
        ("accept", MbValue::from_int(0)),
        ("accept_good", MbValue::from_int(0)),
        ("accept_renegotiate", MbValue::from_int(0)),
        ("hits", MbValue::from_int(0)),
        ("misses", MbValue::from_int(0)),
        ("timeouts", MbValue::from_int(0)),
        ("cache_full", MbValue::from_int(0)),
    ])
}

/// load_cert_chain / load_verify_locations: a missing file path raises
/// FileNotFoundError (CPython surfaces the OSError from OpenSSL's BIO open).
unsafe extern "C" fn ctx_load_path_checked(_self_v: MbValue, args: MbValue) -> MbValue {
    let arg = first_arg(args);
    if let Some(p) = arg.as_ptr() {
        if let ObjData::Str(ref path) = (*p).data {
            if !std::path::Path::new(path.as_str()).exists() {
                return raise_err(
                    "FileNotFoundError",
                    &format!("No such file or directory: {path:?}"),
                );
            }
        }
    }
    MbValue::none()
}

/// `SSLContext.wrap_socket(sock, ...)` — TLS itself is not wired yet, but the
/// CPython argument contract holds: a non-socket first argument fails with
/// AttributeError when wrap_socket reaches for the socket surface.
unsafe extern "C" fn ctx_wrap_socket(_self_v: MbValue, args: MbValue) -> MbValue {
    let sock = first_arg(args);
    let is_instance = sock
        .as_ptr()
        .map(|p| matches!((*p).data, ObjData::Instance { .. }))
        .unwrap_or(false);
    if !is_instance {
        return raise_err(
            "AttributeError",
            &format!("'{}' object has no attribute 'fileno'", py_type_name(sock)),
        );
    }
    MbValue::none()
}

/// Named curves OpenSSL 3.x accepts for set_ecdh_curve.
const KNOWN_CURVES: &[&str] = &[
    "prime192v1",
    "prime256v1",
    "secp224r1",
    "secp256k1",
    "secp384r1",
    "secp521r1",
    "brainpoolP256r1",
    "brainpoolP384r1",
    "brainpoolP512r1",
    "sect233k1",
    "sect233r1",
    "sect283k1",
    "sect283r1",
    "sect409k1",
    "sect409r1",
    "sect571k1",
    "sect571r1",
    "x25519",
    "x448",
];

/// `SSLContext.set_ecdh_curve(name)` — str/bytes of a known curve; None is a
/// TypeError, an unknown curve name a ValueError.
unsafe extern "C" fn ctx_set_ecdh_curve(_self_v: MbValue, args: MbValue) -> MbValue {
    let arg = first_arg(args);
    let name = match arg.as_ptr().and_then(|p| match &(*p).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
        _ => None,
    }) {
        Some(s) => s,
        None => {
            return raise_err(
                "TypeError",
                &format!("curve name must be str or bytes, not {}", py_type_name(arg)),
            );
        }
    };
    if !KNOWN_CURVES.iter().any(|c| c.eq_ignore_ascii_case(&name)) {
        return raise_err("ValueError", &format!("unknown curve name: {name}"));
    }
    MbValue::none()
}

/// All positional args from the packed args list (variadic-init convention).
fn args_vec(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

/// `SSLError.__init__` — OSError-style: 2+ args store (errno, strerror);
/// `args` always carries the full constructor tuple.
unsafe extern "C" fn ssl_error_init(self_v: MbValue, args: MbValue) -> MbValue {
    let items = args_vec(args);
    let args_tuple = MbValue::from_ptr(MbObject::new_tuple(items.clone()));
    set_field(self_v, "args", args_tuple);
    if items.len() >= 2 {
        set_field(self_v, "errno", items[0]);
        set_field(self_v, "strerror", items[1]);
        // message powers the generic traceback/str fallbacks.
        if let Some(p) = items[1].as_ptr() {
            if let ObjData::Str(ref s) = (*p).data {
                set_field(self_v, "message", new_str(s));
            }
        }
    } else if let Some(first) = items.first() {
        set_field(self_v, "message", super::super::builtins::mb_str(*first));
    }
    MbValue::none()
}

/// `SSLError.__str__` — CPython Modules/_ssl.c SSLError_str: the strerror
/// when it is a str, otherwise str(self.args).
unsafe extern "C" fn ssl_error_str(self_v: MbValue) -> MbValue {
    if let Some(strerror) = get_field(self_v, "strerror") {
        if let Some(p) = strerror.as_ptr() {
            if let ObjData::Str(ref s) = (*p).data {
                return MbValue::from_ptr(MbObject::new_str(s.clone()));
            }
        }
    }
    let args = get_field(self_v, "args")
        .unwrap_or_else(|| MbValue::from_ptr(MbObject::new_tuple(Vec::new())));
    super::super::builtins::mb_str(args)
}

/// SSLObject / SSLSocket have no public constructor (CPython 3.12).
unsafe extern "C" fn ssl_object_init(_self_v: MbValue, _args: MbValue) -> MbValue {
    raise_err("TypeError",
        "SSLObject does not have a public constructor. Instances are returned by SSLContext.wrap_bio().")
}

unsafe extern "C" fn ssl_socket_init(_self_v: MbValue, _args: MbValue) -> MbValue {
    raise_err("TypeError",
        "SSLSocket does not have a public constructor. Instances are returned by SSLContext.wrap_socket().")
}

/// Register the ssl exception taxonomy + `SSLContext` in CLASS_REGISTRY.
///
/// The exception classes mirror CPython's hierarchy so `issubclass` walks the
/// real MRO: `SSLError` ⊂ `OSError`, and the want-read / want-write / EOF /
/// zero-return / cert-verification / syscall errors all ⊂ `SSLError`. The
/// builtin `OSError` is registered before stdlib init, so the computed MRO
/// chains up through `OSError → Exception → object`.
fn register_ssl_classes() {
    let empty = || HashMap::<String, MbValue>::new();

    // Error taxonomy. Module attrs for these are plain class-name strings
    // (registered in `register()`), which resolve to these CLASS_REGISTRY
    // entries for issubclass / callable.
    let exc_specs: &[(&str, &[&str])] = &[
        ("SSLError", &["OSError"]),
        ("SSLZeroReturnError", &["SSLError"]),
        ("SSLWantReadError", &["SSLError"]),
        ("SSLWantWriteError", &["SSLError"]),
        ("SSLSyscallError", &["SSLError"]),
        ("SSLEOFError", &["SSLError"]),
        ("SSLCertVerificationError", &["SSLError"]),
        // CPython: CertificateError is an alias of SSLCertVerificationError;
        // model it as a sibling SSLError subclass so issubclass(.., SSLError) holds.
        ("CertificateError", &["SSLError"]),
        // socket_error is the OSError alias re-exported from the socket layer.
        ("socket_error", &["OSError"]),
    ];
    for &(name, bases) in exc_specs {
        let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
        if name == "SSLError" {
            // OSError-style (errno, strerror) constructor + strerror-first str.
            // Subclasses inherit both through the MRO.
            let init_addr = ssl_error_init as usize;
            super::super::module::register_variadic_func(init_addr as u64);
            let mut methods: HashMap<String, MbValue> = HashMap::new();
            methods.insert("__init__".to_string(), MbValue::from_func(init_addr));
            // __str__ takes self only — registered non-variadic so the
            // single-arg mb_call_method1 dispatch matches its ABI.
            methods.insert(
                "__str__".to_string(),
                MbValue::from_func(ssl_error_str as usize),
            );
            super::super::class::mb_class_register(name, base_vec, methods);
        } else {
            super::super::class::mb_class_register(name, base_vec, empty());
        }
    }

    // SSLObject / SSLSocket: real classes (subclassable) whose __init__ raises
    // CPython's "no public constructor" TypeError.
    {
        let obj_init = ssl_object_init as usize;
        let sock_init = ssl_socket_init as usize;
        super::super::module::register_variadic_func(obj_init as u64);
        super::super::module::register_variadic_func(sock_init as u64);
        let mut obj_methods: HashMap<String, MbValue> = HashMap::new();
        obj_methods.insert("__init__".to_string(), MbValue::from_func(obj_init));
        super::super::class::mb_class_register("SSLObject", Vec::new(), obj_methods);
        let mut sock_methods: HashMap<String, MbValue> = HashMap::new();
        sock_methods.insert("__init__".to_string(), MbValue::from_func(sock_init));
        super::super::class::mb_class_register("SSLSocket", Vec::new(), sock_methods);
    }

    // SSLContext: real class so `ssl.SSLContext(protocol)` constructs an
    // instance (attrs round-trip) and `hasattr(ssl.SSLContext, METHOD)` passes.
    {
        let init_addr = init_ssl_context as usize;
        let noop_addr = ctx_method_noop as usize;
        super::super::module::register_variadic_func(init_addr as u64);
        super::super::module::register_variadic_func(noop_addr as u64);
        let mut methods: HashMap<String, MbValue> = HashMap::new();
        methods.insert("__init__".to_string(), MbValue::from_func(init_addr));
        let typed: &[(&str, usize)] = &[
            ("get_ciphers", ctx_get_ciphers as usize),
            ("set_ciphers", ctx_set_ciphers as usize),
            ("session_stats", ctx_session_stats as usize),
            ("load_cert_chain", ctx_load_path_checked as usize),
            ("load_verify_locations", ctx_load_path_checked as usize),
            ("set_ecdh_curve", ctx_set_ecdh_curve as usize),
            ("wrap_socket", ctx_wrap_socket as usize),
        ];
        for (m, addr) in typed {
            super::super::module::register_variadic_func(*addr as u64);
            methods.insert((*m).to_string(), MbValue::from_func(*addr));
        }
        for m in [
            "load_default_certs",
            "set_alpn_protocols",
            "set_npn_protocols",
            "wrap_bio",
            "set_servername_callback",
            "cert_store_stats",
            "get_ca_certs",
        ] {
            methods.insert(m.to_string(), MbValue::from_func(noop_addr));
        }
        super::super::class::mb_class_register("SSLContext", Vec::new(), methods);
    }
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Protocol constants (CPython 3.12 ssl.h verbatim).
    attrs.insert("PROTOCOL_TLS".into(), MbValue::from_int(2));
    attrs.insert("PROTOCOL_SSLv23".into(), MbValue::from_int(2));
    attrs.insert("PROTOCOL_TLS_CLIENT".into(), MbValue::from_int(16));
    attrs.insert("PROTOCOL_TLS_SERVER".into(), MbValue::from_int(17));
    attrs.insert("PROTOCOL_TLSv1".into(), MbValue::from_int(3));
    attrs.insert("PROTOCOL_TLSv1_1".into(), MbValue::from_int(4));
    attrs.insert("PROTOCOL_TLSv1_2".into(), MbValue::from_int(5));

    // Cert verification.
    attrs.insert("CERT_NONE".into(), MbValue::from_int(0));
    attrs.insert("CERT_OPTIONAL".into(), MbValue::from_int(1));
    attrs.insert("CERT_REQUIRED".into(), MbValue::from_int(2));

    // TLS version availability.
    attrs.insert("HAS_TLSv1".into(), MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_1".into(), MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_2".into(), MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_3".into(), MbValue::from_bool(true));
    attrs.insert("HAS_SNI".into(), MbValue::from_bool(true));
    attrs.insert("HAS_ALPN".into(), MbValue::from_bool(true));
    attrs.insert("HAS_NPN".into(), MbValue::from_bool(false));
    attrs.insert("HAS_ECDH".into(), MbValue::from_bool(true));

    // VerifyMode aliases for namespaced lookup.
    attrs.insert("VERIFY_DEFAULT".into(), MbValue::from_int(0));
    attrs.insert("VERIFY_CRL_CHECK_LEAF".into(), MbValue::from_int(4));
    attrs.insert("VERIFY_CRL_CHECK_CHAIN".into(), MbValue::from_int(12));
    attrs.insert("VERIFY_X509_STRICT".into(), MbValue::from_int(32));
    attrs.insert(
        "VERIFY_X509_TRUSTED_FIRST".into(),
        MbValue::from_int(0x8000),
    );

    // OP_* flags (most-used subset).
    attrs.insert("OP_ALL".into(), MbValue::from_int(0x8000_0054));
    attrs.insert("OP_NO_SSLv2".into(), MbValue::from_int(0x0100_0000));
    attrs.insert("OP_NO_SSLv3".into(), MbValue::from_int(0x0200_0000));
    attrs.insert("OP_NO_TLSv1".into(), MbValue::from_int(0x0400_0000));
    attrs.insert("OP_NO_TLSv1_1".into(), MbValue::from_int(0x1000_0000));
    attrs.insert("OP_NO_TLSv1_2".into(), MbValue::from_int(0x0800_0000));
    attrs.insert("OP_NO_TLSv1_3".into(), MbValue::from_int(0x2000_0000));
    attrs.insert("OP_NO_COMPRESSION".into(), MbValue::from_int(0x0002_0000));

    // OpenSSL version triple (sentinel — not real openssl on Mamba yet).
    attrs.insert(
        "OPENSSL_VERSION".into(),
        MbValue::from_ptr(MbObject::new_str("OpenSSL 3.0.0 (mamba shim)".to_string())),
    );
    attrs.insert(
        "OPENSSL_VERSION_NUMBER".into(),
        MbValue::from_int(0x3000_0000),
    );
    let version_info: Vec<MbValue> = [3, 0, 0, 0, 0]
        .iter()
        .map(|n| MbValue::from_int(*n))
        .collect();
    attrs.insert(
        "OPENSSL_VERSION_INFO".into(),
        MbValue::from_ptr(MbObject::new_list(version_info)),
    );

    // Default cipher string.
    attrs.insert(
        "_DEFAULT_CIPHERS".into(),
        MbValue::from_ptr(MbObject::new_str("DEFAULT:!aNULL:!eNULL".to_string())),
    );

    // Channel binding tokens supported by this build.
    let cb_tokens: Vec<MbValue> = ["tls-unique"]
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
        .collect();
    attrs.insert(
        "CHANNEL_BINDING_TYPES".into(),
        MbValue::from_ptr(MbObject::new_list(cb_tokens)),
    );

    // ALERT_DESCRIPTION_* alert codes (CPython 3.12 AlertDescription enum values).
    attrs.insert(
        "ALERT_DESCRIPTION_CLOSE_NOTIFY".into(),
        MbValue::from_int(0),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_UNEXPECTED_MESSAGE".into(),
        MbValue::from_int(10),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_BAD_RECORD_MAC".into(),
        MbValue::from_int(20),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_RECORD_OVERFLOW".into(),
        MbValue::from_int(22),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_DECOMPRESSION_FAILURE".into(),
        MbValue::from_int(30),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_HANDSHAKE_FAILURE".into(),
        MbValue::from_int(40),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_BAD_CERTIFICATE".into(),
        MbValue::from_int(42),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_UNSUPPORTED_CERTIFICATE".into(),
        MbValue::from_int(43),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_CERTIFICATE_REVOKED".into(),
        MbValue::from_int(44),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_CERTIFICATE_EXPIRED".into(),
        MbValue::from_int(45),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_CERTIFICATE_UNKNOWN".into(),
        MbValue::from_int(46),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_ILLEGAL_PARAMETER".into(),
        MbValue::from_int(47),
    );
    attrs.insert("ALERT_DESCRIPTION_UNKNOWN_CA".into(), MbValue::from_int(48));
    attrs.insert(
        "ALERT_DESCRIPTION_ACCESS_DENIED".into(),
        MbValue::from_int(49),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_DECODE_ERROR".into(),
        MbValue::from_int(50),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_DECRYPT_ERROR".into(),
        MbValue::from_int(51),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_PROTOCOL_VERSION".into(),
        MbValue::from_int(70),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_INSUFFICIENT_SECURITY".into(),
        MbValue::from_int(71),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_INTERNAL_ERROR".into(),
        MbValue::from_int(80),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_USER_CANCELLED".into(),
        MbValue::from_int(90),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_NO_RENEGOTIATION".into(),
        MbValue::from_int(100),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_UNSUPPORTED_EXTENSION".into(),
        MbValue::from_int(110),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_CERTIFICATE_UNOBTAINABLE".into(),
        MbValue::from_int(111),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_UNRECOGNIZED_NAME".into(),
        MbValue::from_int(112),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_BAD_CERTIFICATE_STATUS_RESPONSE".into(),
        MbValue::from_int(113),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_BAD_CERTIFICATE_HASH_VALUE".into(),
        MbValue::from_int(114),
    );
    attrs.insert(
        "ALERT_DESCRIPTION_UNKNOWN_PSK_IDENTITY".into(),
        MbValue::from_int(115),
    );

    // SSL_ERROR_* SSLErrorNumber enum values (CPython 3.12).
    attrs.insert("SSL_ERROR_SSL".into(), MbValue::from_int(1));
    attrs.insert("SSL_ERROR_WANT_READ".into(), MbValue::from_int(2));
    attrs.insert("SSL_ERROR_WANT_WRITE".into(), MbValue::from_int(3));
    attrs.insert("SSL_ERROR_WANT_X509_LOOKUP".into(), MbValue::from_int(4));
    attrs.insert("SSL_ERROR_SYSCALL".into(), MbValue::from_int(5));
    attrs.insert("SSL_ERROR_ZERO_RETURN".into(), MbValue::from_int(6));
    attrs.insert("SSL_ERROR_WANT_CONNECT".into(), MbValue::from_int(7));
    attrs.insert("SSL_ERROR_EOF".into(), MbValue::from_int(8));
    attrs.insert("SSL_ERROR_INVALID_ERROR_CODE".into(), MbValue::from_int(10));

    // Additional OP_* flags (CPython 3.12 Options enum values).
    attrs.insert("OP_LEGACY_SERVER_CONNECT".into(), MbValue::from_int(4));
    attrs.insert("OP_ENABLE_KTLS".into(), MbValue::from_int(8));
    attrs.insert("OP_IGNORE_UNEXPECTED_EOF".into(), MbValue::from_int(128));
    attrs.insert("OP_NO_TICKET".into(), MbValue::from_int(16384));
    attrs.insert(
        "OP_ENABLE_MIDDLEBOX_COMPAT".into(),
        MbValue::from_int(1_048_576),
    );
    attrs.insert(
        "OP_CIPHER_SERVER_PREFERENCE".into(),
        MbValue::from_int(4_194_304),
    );
    attrs.insert(
        "OP_NO_RENEGOTIATION".into(),
        MbValue::from_int(1_073_741_824),
    );
    attrs.insert("OP_SINGLE_DH_USE".into(), MbValue::from_int(0));
    attrs.insert("OP_SINGLE_ECDH_USE".into(), MbValue::from_int(0));

    // Additional VERIFY_* VerifyFlags enum values (CPython 3.12).
    attrs.insert("VERIFY_ALLOW_PROXY_CERTS".into(), MbValue::from_int(64));
    attrs.insert(
        "VERIFY_X509_PARTIAL_CHAIN".into(),
        MbValue::from_int(524_288),
    );

    // Additional HAS_* availability flags.
    attrs.insert("HAS_SSLv2".into(), MbValue::from_bool(false));
    attrs.insert("HAS_SSLv3".into(), MbValue::from_bool(false));
    attrs.insert(
        "HAS_NEVER_CHECK_COMMON_NAME".into(),
        MbValue::from_bool(true),
    );

    // socket-layer constants re-exported by ssl (CPython 3.12).
    attrs.insert("SOCK_STREAM".into(), MbValue::from_int(1));
    attrs.insert("SOL_SOCKET".into(), MbValue::from_int(65535));
    attrs.insert("SO_TYPE".into(), MbValue::from_int(4104));

    // PEM framing strings.
    attrs.insert(
        "PEM_HEADER".into(),
        MbValue::from_ptr(MbObject::new_str("-----BEGIN CERTIFICATE-----".to_string())),
    );
    attrs.insert(
        "PEM_FOOTER".into(),
        MbValue::from_ptr(MbObject::new_str("-----END CERTIFICATE-----".to_string())),
    );

    // Re-exported modules / OSError alias — present as opaque values so
    // hasattr(ssl, NAME) resolves (these are not callable in CPython, so they
    // are registered as plain sentinel values rather than func dispatchers).
    for name in ["base64", "errno", "os", "sys", "warnings"] {
        attrs.insert(name.into(), MbValue::from_ptr(MbObject::new_dict()));
    }

    // `SSLContext` and the SSL error taxonomy are real registered classes
    // (see register_ssl_classes); their module attrs are class-name strings so
    // callable() / construction / issubclass resolve through CLASS_REGISTRY.
    for cls in [
        "SSLContext",
        "SSLObject",
        "SSLSocket",
        "SSLError",
        "SSLZeroReturnError",
        "SSLWantReadError",
        "SSLWantWriteError",
        "SSLSyscallError",
        "SSLEOFError",
        "SSLCertVerificationError",
        "CertificateError",
        "socket_error",
    ] {
        attrs.insert(cls.into(), new_str(cls));
    }

    // Enum-shaped namespaces. CPython exposes these as IntEnum / namespace
    // objects with member attributes; a Dict namespace resolves
    // hasattr(ssl.X, MEMBER) directly. Values are the real CPython members.
    attrs.insert(
        "Purpose".into(),
        make_ns(&[
            // ssl.Purpose.{SERVER,CLIENT}_AUTH are _ASN1Object-shaped members
            // carrying (nid, shortname, longname, oid) for the extended-key-usage
            // OIDs (CPython Lib/ssl.py Purpose enum).
            (
                "SERVER_AUTH",
                make_ns(&[
                    ("nid", MbValue::from_int(129)),
                    ("shortname", new_str("serverAuth")),
                    ("longname", new_str("TLS Web Server Authentication")),
                    ("oid", new_str("1.3.6.1.5.5.7.3.1")),
                ]),
            ),
            (
                "CLIENT_AUTH",
                make_ns(&[
                    ("nid", MbValue::from_int(130)),
                    ("shortname", new_str("clientAuth")),
                    ("longname", new_str("TLS Web Client Authentication")),
                    ("oid", new_str("1.3.6.1.5.5.7.3.2")),
                ]),
            ),
        ]),
    );
    attrs.insert(
        "TLSVersion".into(),
        make_ns(&[
            ("MINIMUM_SUPPORTED", MbValue::from_int(-2)),
            ("SSLv3", MbValue::from_int(768)),
            ("TLSv1", MbValue::from_int(769)),
            ("TLSv1_1", MbValue::from_int(770)),
            ("TLSv1_2", MbValue::from_int(771)),
            ("TLSv1_3", MbValue::from_int(772)),
            ("MAXIMUM_SUPPORTED", MbValue::from_int(-1)),
        ]),
    );

    let dispatchers: &[(&str, usize)] = &[
        (
            "create_default_context",
            dispatch_create_default_context as *const () as usize,
        ),
        ("SSLSession", dispatch_class_shell as *const () as usize),
        ("MemoryBIO", dispatch_class_shell as *const () as usize),
        ("wrap_socket", dispatch_wrap_socket as *const () as usize),
        (
            "match_hostname",
            dispatch_match_hostname as *const () as usize,
        ),
        (
            "cert_time_to_seconds",
            dispatch_cert_time_to_seconds as *const () as usize,
        ),
        (
            "DER_cert_to_PEM_cert",
            dispatch_der_to_pem as *const () as usize,
        ),
        (
            "PEM_cert_to_DER_cert",
            dispatch_pem_to_der as *const () as usize,
        ),
        (
            "get_default_verify_paths",
            dispatch_get_default_verify_paths as *const () as usize,
        ),
        (
            "get_server_certificate",
            dispatch_get_server_certificate as *const () as usize,
        ),
        ("RAND_status", dispatch_rand_status as *const () as usize),
        ("RAND_bytes", dispatch_rand_bytes as *const () as usize),
        (
            "RAND_pseudo_bytes",
            dispatch_rand_bytes as *const () as usize,
        ),
        ("RAND_add", dispatch_rand_status as *const () as usize),
        // Enum classes (CPython EnumType) — callable stubs so callable()/hasattr pass.
        (
            "AlertDescription",
            dispatch_class_shell as *const () as usize,
        ),
        ("Options", dispatch_class_shell as *const () as usize),
        ("SSLErrorNumber", dispatch_class_shell as *const () as usize),
        ("VerifyFlags", dispatch_class_shell as *const () as usize),
        ("VerifyMode", dispatch_class_shell as *const () as usize),
        // Additional classes/types.
        (
            "DefaultVerifyPaths",
            dispatch_class_shell as *const () as usize,
        ),
        ("socket", dispatch_class_shell as *const () as usize),
        // Module-level functions.
        (
            "create_connection",
            dispatch_ssl_context as *const () as usize,
        ),
        (
            "get_protocol_name",
            dispatch_der_to_pem as *const () as usize,
        ),
        ("namedtuple", dispatch_class_shell as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });

    register_ssl_classes();
    super::register_module("ssl", attrs);
}
