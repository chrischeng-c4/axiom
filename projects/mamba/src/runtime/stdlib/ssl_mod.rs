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
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops::DictKey;

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

unsafe extern "C" fn dispatch_create_default_context(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
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
        ("cafile",            new_str("")),
        ("capath",            new_str("")),
        ("openssl_cafile_env", new_str("SSL_CERT_FILE")),
        ("openssl_cafile",    new_str("")),
        ("openssl_capath_env", new_str("SSL_CERT_DIR")),
        ("openssl_capath",    new_str("")),
    ])
}

unsafe extern "C" fn dispatch_get_server_certificate(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

unsafe extern "C" fn dispatch_rand_status(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(true)
}

unsafe extern "C" fn dispatch_rand_bytes(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_bytes(Vec::new()))
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
                if let Some(p) = prev { super::super::rc::release_if_ptr(p); }
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

unsafe extern "C" fn init_ssl_context(self_v: MbValue, args: MbValue) -> MbValue {
    // SSLContext(protocol=PROTOCOL_TLS). Store the protocol and seed the
    // version-range attributes so the instance is well-formed before the caller
    // overwrites them.
    let protocol = first_arg(args);
    let protocol = if protocol.is_none() { MbValue::from_int(2) } else { protocol };
    set_field(self_v, "protocol", protocol);
    set_field(self_v, "minimum_version", MbValue::from_int(-2)); // TLSVersion.MINIMUM_SUPPORTED
    set_field(self_v, "maximum_version", MbValue::from_int(-1)); // TLSVersion.MAXIMUM_SUPPORTED
    set_field(self_v, "verify_mode", MbValue::from_int(0));      // CERT_NONE
    set_field(self_v, "check_hostname", MbValue::from_bool(false));
    MbValue::none()
}

/// Generic no-op SSLContext method (load_cert_chain / set_ciphers / ...): returns None.
unsafe extern "C" fn ctx_method_noop(_self_v: MbValue, _args: MbValue) -> MbValue {
    MbValue::none()
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
        ("SSLError",                 &["OSError"]),
        ("SSLZeroReturnError",       &["SSLError"]),
        ("SSLWantReadError",         &["SSLError"]),
        ("SSLWantWriteError",        &["SSLError"]),
        ("SSLSyscallError",          &["SSLError"]),
        ("SSLEOFError",              &["SSLError"]),
        ("SSLCertVerificationError", &["SSLError"]),
        // CPython: CertificateError is an alias of SSLCertVerificationError;
        // model it as a sibling SSLError subclass so issubclass(.., SSLError) holds.
        ("CertificateError",         &["SSLError"]),
        // socket_error is the OSError alias re-exported from the socket layer.
        ("socket_error",             &["OSError"]),
    ];
    for &(name, bases) in exc_specs {
        let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
        super::super::class::mb_class_register(name, base_vec, empty());
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
        for m in [
            "load_cert_chain", "load_verify_locations", "load_default_certs",
            "set_alpn_protocols", "set_npn_protocols", "set_ciphers",
            "wrap_socket", "wrap_bio", "get_ciphers", "set_servername_callback",
            "session_stats", "cert_store_stats", "get_ca_certs",
        ] {
            methods.insert(m.to_string(), MbValue::from_func(noop_addr));
        }
        super::super::class::mb_class_register("SSLContext", Vec::new(), methods);
    }
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Protocol constants (CPython 3.12 ssl.h verbatim).
    attrs.insert("PROTOCOL_TLS".into(),          MbValue::from_int(2));
    attrs.insert("PROTOCOL_SSLv23".into(),       MbValue::from_int(2));
    attrs.insert("PROTOCOL_TLS_CLIENT".into(),   MbValue::from_int(16));
    attrs.insert("PROTOCOL_TLS_SERVER".into(),   MbValue::from_int(17));
    attrs.insert("PROTOCOL_TLSv1".into(),        MbValue::from_int(3));
    attrs.insert("PROTOCOL_TLSv1_1".into(),      MbValue::from_int(4));
    attrs.insert("PROTOCOL_TLSv1_2".into(),      MbValue::from_int(5));

    // Cert verification.
    attrs.insert("CERT_NONE".into(),     MbValue::from_int(0));
    attrs.insert("CERT_OPTIONAL".into(), MbValue::from_int(1));
    attrs.insert("CERT_REQUIRED".into(), MbValue::from_int(2));

    // TLS version availability.
    attrs.insert("HAS_TLSv1".into(),   MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_1".into(), MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_2".into(), MbValue::from_bool(true));
    attrs.insert("HAS_TLSv1_3".into(), MbValue::from_bool(true));
    attrs.insert("HAS_SNI".into(),     MbValue::from_bool(true));
    attrs.insert("HAS_ALPN".into(),    MbValue::from_bool(true));
    attrs.insert("HAS_NPN".into(),     MbValue::from_bool(false));
    attrs.insert("HAS_ECDH".into(),    MbValue::from_bool(true));

    // VerifyMode aliases for namespaced lookup.
    attrs.insert("VERIFY_DEFAULT".into(),            MbValue::from_int(0));
    attrs.insert("VERIFY_CRL_CHECK_LEAF".into(),     MbValue::from_int(4));
    attrs.insert("VERIFY_CRL_CHECK_CHAIN".into(),    MbValue::from_int(12));
    attrs.insert("VERIFY_X509_STRICT".into(),        MbValue::from_int(32));
    attrs.insert("VERIFY_X509_TRUSTED_FIRST".into(), MbValue::from_int(0x8000));

    // OP_* flags (most-used subset).
    attrs.insert("OP_ALL".into(),                   MbValue::from_int(0x8000_0054));
    attrs.insert("OP_NO_SSLv2".into(),              MbValue::from_int(0x0100_0000));
    attrs.insert("OP_NO_SSLv3".into(),              MbValue::from_int(0x0200_0000));
    attrs.insert("OP_NO_TLSv1".into(),              MbValue::from_int(0x0400_0000));
    attrs.insert("OP_NO_TLSv1_1".into(),            MbValue::from_int(0x1000_0000));
    attrs.insert("OP_NO_TLSv1_2".into(),            MbValue::from_int(0x0800_0000));
    attrs.insert("OP_NO_TLSv1_3".into(),            MbValue::from_int(0x2000_0000));
    attrs.insert("OP_NO_COMPRESSION".into(),        MbValue::from_int(0x0002_0000));

    // OpenSSL version triple (sentinel — not real openssl on Mamba yet).
    attrs.insert("OPENSSL_VERSION".into(),
        MbValue::from_ptr(MbObject::new_str("OpenSSL 3.0.0 (mamba shim)".to_string())));
    attrs.insert("OPENSSL_VERSION_NUMBER".into(), MbValue::from_int(0x3000_0000));
    let version_info: Vec<MbValue> = [3, 0, 0, 0, 0].iter().map(|n| MbValue::from_int(*n)).collect();
    attrs.insert("OPENSSL_VERSION_INFO".into(), MbValue::from_ptr(MbObject::new_list(version_info)));

    // Default cipher string.
    attrs.insert("_DEFAULT_CIPHERS".into(),
        MbValue::from_ptr(MbObject::new_str("DEFAULT:!aNULL:!eNULL".to_string())));

    // Channel binding tokens supported by this build.
    let cb_tokens: Vec<MbValue> = ["tls-unique"].iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string()))).collect();
    attrs.insert("CHANNEL_BINDING_TYPES".into(), MbValue::from_ptr(MbObject::new_list(cb_tokens)));

    // ALERT_DESCRIPTION_* alert codes (CPython 3.12 AlertDescription enum values).
    attrs.insert("ALERT_DESCRIPTION_CLOSE_NOTIFY".into(),                  MbValue::from_int(0));
    attrs.insert("ALERT_DESCRIPTION_UNEXPECTED_MESSAGE".into(),            MbValue::from_int(10));
    attrs.insert("ALERT_DESCRIPTION_BAD_RECORD_MAC".into(),                MbValue::from_int(20));
    attrs.insert("ALERT_DESCRIPTION_RECORD_OVERFLOW".into(),               MbValue::from_int(22));
    attrs.insert("ALERT_DESCRIPTION_DECOMPRESSION_FAILURE".into(),         MbValue::from_int(30));
    attrs.insert("ALERT_DESCRIPTION_HANDSHAKE_FAILURE".into(),             MbValue::from_int(40));
    attrs.insert("ALERT_DESCRIPTION_BAD_CERTIFICATE".into(),               MbValue::from_int(42));
    attrs.insert("ALERT_DESCRIPTION_UNSUPPORTED_CERTIFICATE".into(),       MbValue::from_int(43));
    attrs.insert("ALERT_DESCRIPTION_CERTIFICATE_REVOKED".into(),           MbValue::from_int(44));
    attrs.insert("ALERT_DESCRIPTION_CERTIFICATE_EXPIRED".into(),           MbValue::from_int(45));
    attrs.insert("ALERT_DESCRIPTION_CERTIFICATE_UNKNOWN".into(),           MbValue::from_int(46));
    attrs.insert("ALERT_DESCRIPTION_ILLEGAL_PARAMETER".into(),             MbValue::from_int(47));
    attrs.insert("ALERT_DESCRIPTION_UNKNOWN_CA".into(),                    MbValue::from_int(48));
    attrs.insert("ALERT_DESCRIPTION_ACCESS_DENIED".into(),                 MbValue::from_int(49));
    attrs.insert("ALERT_DESCRIPTION_DECODE_ERROR".into(),                  MbValue::from_int(50));
    attrs.insert("ALERT_DESCRIPTION_DECRYPT_ERROR".into(),                 MbValue::from_int(51));
    attrs.insert("ALERT_DESCRIPTION_PROTOCOL_VERSION".into(),              MbValue::from_int(70));
    attrs.insert("ALERT_DESCRIPTION_INSUFFICIENT_SECURITY".into(),         MbValue::from_int(71));
    attrs.insert("ALERT_DESCRIPTION_INTERNAL_ERROR".into(),                MbValue::from_int(80));
    attrs.insert("ALERT_DESCRIPTION_USER_CANCELLED".into(),                MbValue::from_int(90));
    attrs.insert("ALERT_DESCRIPTION_NO_RENEGOTIATION".into(),              MbValue::from_int(100));
    attrs.insert("ALERT_DESCRIPTION_UNSUPPORTED_EXTENSION".into(),         MbValue::from_int(110));
    attrs.insert("ALERT_DESCRIPTION_CERTIFICATE_UNOBTAINABLE".into(),      MbValue::from_int(111));
    attrs.insert("ALERT_DESCRIPTION_UNRECOGNIZED_NAME".into(),             MbValue::from_int(112));
    attrs.insert("ALERT_DESCRIPTION_BAD_CERTIFICATE_STATUS_RESPONSE".into(), MbValue::from_int(113));
    attrs.insert("ALERT_DESCRIPTION_BAD_CERTIFICATE_HASH_VALUE".into(),    MbValue::from_int(114));
    attrs.insert("ALERT_DESCRIPTION_UNKNOWN_PSK_IDENTITY".into(),          MbValue::from_int(115));

    // SSL_ERROR_* SSLErrorNumber enum values (CPython 3.12).
    attrs.insert("SSL_ERROR_SSL".into(),                  MbValue::from_int(1));
    attrs.insert("SSL_ERROR_WANT_READ".into(),            MbValue::from_int(2));
    attrs.insert("SSL_ERROR_WANT_WRITE".into(),           MbValue::from_int(3));
    attrs.insert("SSL_ERROR_WANT_X509_LOOKUP".into(),     MbValue::from_int(4));
    attrs.insert("SSL_ERROR_SYSCALL".into(),              MbValue::from_int(5));
    attrs.insert("SSL_ERROR_ZERO_RETURN".into(),          MbValue::from_int(6));
    attrs.insert("SSL_ERROR_WANT_CONNECT".into(),         MbValue::from_int(7));
    attrs.insert("SSL_ERROR_EOF".into(),                  MbValue::from_int(8));
    attrs.insert("SSL_ERROR_INVALID_ERROR_CODE".into(),   MbValue::from_int(10));

    // Additional OP_* flags (CPython 3.12 Options enum values).
    attrs.insert("OP_LEGACY_SERVER_CONNECT".into(),       MbValue::from_int(4));
    attrs.insert("OP_ENABLE_KTLS".into(),                 MbValue::from_int(8));
    attrs.insert("OP_IGNORE_UNEXPECTED_EOF".into(),       MbValue::from_int(128));
    attrs.insert("OP_NO_TICKET".into(),                   MbValue::from_int(16384));
    attrs.insert("OP_ENABLE_MIDDLEBOX_COMPAT".into(),     MbValue::from_int(1_048_576));
    attrs.insert("OP_CIPHER_SERVER_PREFERENCE".into(),    MbValue::from_int(4_194_304));
    attrs.insert("OP_NO_RENEGOTIATION".into(),            MbValue::from_int(1_073_741_824));
    attrs.insert("OP_SINGLE_DH_USE".into(),               MbValue::from_int(0));
    attrs.insert("OP_SINGLE_ECDH_USE".into(),             MbValue::from_int(0));

    // Additional VERIFY_* VerifyFlags enum values (CPython 3.12).
    attrs.insert("VERIFY_ALLOW_PROXY_CERTS".into(),       MbValue::from_int(64));
    attrs.insert("VERIFY_X509_PARTIAL_CHAIN".into(),      MbValue::from_int(524_288));

    // Additional HAS_* availability flags.
    attrs.insert("HAS_SSLv2".into(),                      MbValue::from_bool(false));
    attrs.insert("HAS_SSLv3".into(),                      MbValue::from_bool(false));
    attrs.insert("HAS_NEVER_CHECK_COMMON_NAME".into(),    MbValue::from_bool(true));

    // socket-layer constants re-exported by ssl (CPython 3.12).
    attrs.insert("SOCK_STREAM".into(),                    MbValue::from_int(1));
    attrs.insert("SOL_SOCKET".into(),                     MbValue::from_int(65535));
    attrs.insert("SO_TYPE".into(),                        MbValue::from_int(4104));

    // PEM framing strings.
    attrs.insert("PEM_HEADER".into(),
        MbValue::from_ptr(MbObject::new_str("-----BEGIN CERTIFICATE-----".to_string())));
    attrs.insert("PEM_FOOTER".into(),
        MbValue::from_ptr(MbObject::new_str("-----END CERTIFICATE-----".to_string())));

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
        "SSLError", "SSLZeroReturnError", "SSLWantReadError", "SSLWantWriteError",
        "SSLSyscallError", "SSLEOFError", "SSLCertVerificationError",
        "CertificateError", "socket_error",
    ] {
        attrs.insert(cls.into(), new_str(cls));
    }

    // Enum-shaped namespaces. CPython exposes these as IntEnum / namespace
    // objects with member attributes; a Dict namespace resolves
    // hasattr(ssl.X, MEMBER) directly. Values are the real CPython members.
    attrs.insert("Purpose".into(), make_ns(&[
        // ssl.Purpose.{SERVER,CLIENT}_AUTH carry the extended-key-usage OID.
        ("SERVER_AUTH", new_str("1.3.6.1.5.5.7.3.1")),
        ("CLIENT_AUTH", new_str("1.3.6.1.5.5.7.3.2")),
    ]));
    attrs.insert("TLSVersion".into(), make_ns(&[
        ("MINIMUM_SUPPORTED", MbValue::from_int(-2)),
        ("SSLv3",             MbValue::from_int(768)),
        ("TLSv1",             MbValue::from_int(769)),
        ("TLSv1_1",           MbValue::from_int(770)),
        ("TLSv1_2",           MbValue::from_int(771)),
        ("TLSv1_3",           MbValue::from_int(772)),
        ("MAXIMUM_SUPPORTED", MbValue::from_int(-1)),
    ]));

    let dispatchers: &[(&str, usize)] = &[
        ("create_default_context",      dispatch_create_default_context       as *const () as usize),
        ("SSLSession",                  dispatch_class_shell                  as *const () as usize),
        ("SSLSocket",                   dispatch_class_shell                  as *const () as usize),
        ("MemoryBIO",                   dispatch_class_shell                  as *const () as usize),
        ("wrap_socket",                 dispatch_wrap_socket                  as *const () as usize),
        ("match_hostname",              dispatch_match_hostname               as *const () as usize),
        ("cert_time_to_seconds",        dispatch_cert_time_to_seconds         as *const () as usize),
        ("DER_cert_to_PEM_cert",        dispatch_der_to_pem                   as *const () as usize),
        ("PEM_cert_to_DER_cert",        dispatch_pem_to_der                   as *const () as usize),
        ("get_default_verify_paths",    dispatch_get_default_verify_paths     as *const () as usize),
        ("get_server_certificate",      dispatch_get_server_certificate       as *const () as usize),
        ("RAND_status",                 dispatch_rand_status                  as *const () as usize),
        ("RAND_bytes",                  dispatch_rand_bytes                   as *const () as usize),
        ("RAND_pseudo_bytes",           dispatch_rand_bytes                   as *const () as usize),
        ("RAND_add",                    dispatch_rand_status                  as *const () as usize),
        // Enum classes (CPython EnumType) — callable stubs so callable()/hasattr pass.
        ("AlertDescription",            dispatch_class_shell                  as *const () as usize),
        ("Options",                     dispatch_class_shell                  as *const () as usize),
        ("SSLErrorNumber",              dispatch_class_shell                  as *const () as usize),
        ("VerifyFlags",                 dispatch_class_shell                  as *const () as usize),
        ("VerifyMode",                  dispatch_class_shell                  as *const () as usize),
        // Additional classes/types.
        ("DefaultVerifyPaths",          dispatch_class_shell                  as *const () as usize),
        ("SSLObject",                   dispatch_class_shell                  as *const () as usize),
        ("socket",                      dispatch_class_shell                  as *const () as usize),
        // Module-level functions.
        ("create_connection",           dispatch_ssl_context                  as *const () as usize),
        ("get_protocol_name",           dispatch_der_to_pem                   as *const () as usize),
        ("namedtuple",                  dispatch_class_shell                  as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers { set.insert(*addr as u64); }
    });

    register_ssl_classes();
    super::register_module("ssl", attrs);
}
