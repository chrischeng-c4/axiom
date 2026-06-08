/// certifi module for Mamba (#1483).
///
/// Returns a usable CA-bundle path so SSL libraries that call
/// `certifi.where()` don't get a stub-dict back. We probe a short
/// list of well-known OS locations:
///   - `SSL_CERT_FILE` env var (overrides everything, matches OpenSSL)
///   - macOS LibreSSL bundle (`/etc/ssl/cert.pem`)
///   - Debian/Ubuntu/Alpine (`/etc/ssl/certs/ca-certificates.crt`)
///   - RHEL/CentOS/Fedora (`/etc/pki/tls/certs/ca-bundle.crt`)
/// Falls back to `/etc/ssl/cert.pem` if nothing exists.
///
/// `contents()` reads and returns the bundle as a string (matching
/// CPython's `certifi.contents()`); empty string on read failure.
///
/// CPython exposes `certifi.core` as a submodule. The stub returns
/// `where`'s function as a callable handle so attribute reads succeed.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

const CA_BUNDLE_CANDIDATES: &[&str] = &[
    "/etc/ssl/cert.pem",                            // macOS / OpenBSD / LibreSSL
    "/etc/ssl/certs/ca-certificates.crt",           // Debian / Ubuntu / Alpine
    "/etc/pki/tls/certs/ca-bundle.crt",             // RHEL / CentOS / Fedora
    "/etc/pki/ca-trust/extracted/pem/tls-ca-bundle.pem",
    "/usr/local/share/certs/ca-root-nss.crt",       // FreeBSD
    "/etc/ssl/ca-bundle.pem",                       // OpenSUSE
];

fn resolve_ca_bundle() -> String {
    if let Ok(p) = std::env::var("SSL_CERT_FILE") {
        if !p.is_empty() {
            return p;
        }
    }
    for path in CA_BUNDLE_CANDIDATES {
        if std::path::Path::new(path).exists() {
            return (*path).to_string();
        }
    }
    // No bundle found — return the most likely path so callers see a
    // string, not an empty value. They'll get an OSError when they try
    // to actually open it.
    CA_BUNDLE_CANDIDATES[0].to_string()
}

unsafe extern "C" fn dispatch_where(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(resolve_ca_bundle()))
}

unsafe extern "C" fn dispatch_contents(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let path = resolve_ca_bundle();
    let contents = std::fs::read_to_string(&path).unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(contents))
}

unsafe extern "C" fn dispatch_core(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    // `certifi.core` is the submodule; CPython returns the module object.
    // Mamba's module table doesn't yet expose submodule-as-value, so hand
    // back an empty dict — callers that do `from certifi.core import where`
    // hit the import machinery, not this dispatch.
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the certifi module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_where = dispatch_where as *const () as usize;
    attrs.insert("where".into(), MbValue::from_func(addr_where));

    let addr_contents = dispatch_contents as *const () as usize;
    attrs.insert("contents".into(), MbValue::from_func(addr_contents));

    let addr_core = dispatch_core as *const () as usize;
    attrs.insert("core".into(), MbValue::from_func(addr_core));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_where as u64);
        set.insert(addr_contents as u64);
        set.insert(addr_core as u64);
    });

    super::register_module("certifi", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::rc::ObjData;

    #[test]
    fn where_returns_string_path() {
        let v = unsafe { dispatch_where(std::ptr::null(), 0) };
        unsafe {
            let p = v.as_ptr().expect("ptr");
            if let ObjData::Str(ref s) = (*p).data {
                assert!(!s.is_empty(), "path must be non-empty");
                assert!(s.starts_with('/'), "expected absolute path, got {:?}", s);
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn env_var_overrides() {
        let prev = std::env::var("SSL_CERT_FILE").ok();
        std::env::set_var("SSL_CERT_FILE", "/tmp/mamba-test-ca.pem");
        let v = unsafe { dispatch_where(std::ptr::null(), 0) };
        unsafe {
            let p = v.as_ptr().expect("ptr");
            if let ObjData::Str(ref s) = (*p).data {
                assert_eq!(s, "/tmp/mamba-test-ca.pem");
            } else {
                panic!("expected Str");
            }
        }
        match prev {
            Some(v) => std::env::set_var("SSL_CERT_FILE", v),
            None => std::env::remove_var("SSL_CERT_FILE"),
        }
    }
}
