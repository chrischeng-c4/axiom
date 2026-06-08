use super::super::rc::MbObject;
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

unsafe extern "C" fn dispatch_ssl_context(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_create_default_context(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_ssl_error(_a: *const MbValue, _n: usize) -> MbValue {
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
    // Returns a DefaultVerifyPaths namedtuple shape; surface stub is a 4-tuple of empty strings.
    let empty = || MbValue::from_ptr(MbObject::new_str(String::new()));
    MbValue::from_ptr(MbObject::new_list(vec![empty(), empty(), empty(), empty()]))
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

    let dispatchers: &[(&str, usize)] = &[
        ("SSLContext", dispatch_ssl_context as *const () as usize),
        (
            "create_default_context",
            dispatch_create_default_context as *const () as usize,
        ),
        ("SSLError", dispatch_ssl_error as *const () as usize),
        (
            "SSLZeroReturnError",
            dispatch_ssl_error as *const () as usize,
        ),
        ("SSLWantReadError", dispatch_ssl_error as *const () as usize),
        (
            "SSLWantWriteError",
            dispatch_ssl_error as *const () as usize,
        ),
        ("SSLSyscallError", dispatch_ssl_error as *const () as usize),
        ("SSLEOFError", dispatch_ssl_error as *const () as usize),
        ("CertificateError", dispatch_ssl_error as *const () as usize),
        ("SSLSession", dispatch_class_shell as *const () as usize),
        ("SSLSocket", dispatch_class_shell as *const () as usize),
        ("MemoryBIO", dispatch_class_shell as *const () as usize),
        ("Purpose", dispatch_class_shell as *const () as usize),
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

    super::register_module("ssl", attrs);
}
