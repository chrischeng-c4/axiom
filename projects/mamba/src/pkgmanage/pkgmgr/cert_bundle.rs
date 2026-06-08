// TLS certificate-bundle discovery (Tick 128).
//
// pip, uv, requests, curl, and Python's `ssl` module all read one or
// more of the following env vars to locate a custom CA bundle when
// connecting to TLS index servers:
//
//   SSL_CERT_FILE       — path to a single PEM bundle (OpenSSL convention)
//   SSL_CERT_DIR        — directory of c_rehash-indexed PEM files (OpenSSL)
//   REQUESTS_CA_BUNDLE  — path to a PEM bundle (Python `requests` library)
//   CURL_CA_BUNDLE      — path to a PEM bundle (libcurl convention)
//   PIP_CERT            — pip-specific override (pep@ pep-440 not relevant
//                         here — this is the `--cert` flag's env-var twin)
//
// Precedence (matches uv / pip behaviour):
//
//   1. PIP_CERT
//   2. REQUESTS_CA_BUNDLE
//   3. CURL_CA_BUNDLE
//   4. SSL_CERT_FILE
//
// `SSL_CERT_DIR` is orthogonal — it sets a *search directory*, not a
// single bundle file — so we surface it separately. Both can be set
// at once; the HTTP client passes them to OpenSSL together.
//
// Each name is probed lowercase-first then uppercase, matching the
// `proxy_config` convention (curl's precedence rule applies here too,
// since both pip-via-requests and uv-via-reqwest inherit libcurl-style
// env handling on POSIX).
//
// This module is a pure resolver: it returns paths, it does not open
// or read the files. PEM/DER decoding belongs in the HTTP client.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Source that supplied the bundle path (used for error messages and
/// `mamba pip config debug`-style diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertBundleSource {
    PipCert,
    RequestsCaBundle,
    CurlCaBundle,
    SslCertFile,
}

impl CertBundleSource {
    pub fn env_var_name(self) -> &'static str {
        match self {
            CertBundleSource::PipCert => "PIP_CERT",
            CertBundleSource::RequestsCaBundle => "REQUESTS_CA_BUNDLE",
            CertBundleSource::CurlCaBundle => "CURL_CA_BUNDLE",
            CertBundleSource::SslCertFile => "SSL_CERT_FILE",
        }
    }
}

/// Resolved certificate-bundle configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CertBundle {
    /// Path to a single PEM bundle, if any env var supplied one. The
    /// `CertBundleSource` records which one won.
    pub bundle_path: Option<(String, CertBundleSource)>,
    /// Optional search-directory of PEM files. Orthogonal to
    /// `bundle_path`; both can be active simultaneously.
    pub bundle_dir: Option<String>,
}

impl CertBundle {
    /// Resolve via an env-var lookup closure. Empty / whitespace-only
    /// values are treated as unset, matching curl/pip behaviour.
    pub fn from_env<F>(get: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        let pick = |lower: &str, upper: &str| -> Option<String> {
            let v = get(lower).or_else(|| get(upper))?;
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        };

        // Precedence: PIP_CERT > REQUESTS_CA_BUNDLE > CURL_CA_BUNDLE >
        // SSL_CERT_FILE. First hit wins.
        let bundle_path = pick("PIP_CERT", "PIP_CERT")
            .map(|p| (p, CertBundleSource::PipCert))
            .or_else(|| {
                pick("requests_ca_bundle", "REQUESTS_CA_BUNDLE")
                    .map(|p| (p, CertBundleSource::RequestsCaBundle))
            })
            .or_else(|| {
                pick("curl_ca_bundle", "CURL_CA_BUNDLE")
                    .map(|p| (p, CertBundleSource::CurlCaBundle))
            })
            .or_else(|| {
                pick("ssl_cert_file", "SSL_CERT_FILE")
                    .map(|p| (p, CertBundleSource::SslCertFile))
            });

        let bundle_dir = pick("ssl_cert_dir", "SSL_CERT_DIR");

        CertBundle {
            bundle_path,
            bundle_dir,
        }
    }

    /// True when no env var supplied a bundle path or directory.
    pub fn is_empty(&self) -> bool {
        self.bundle_path.is_none() && self.bundle_dir.is_none()
    }

    /// Convenience: just the bundle file path, dropping the source tag.
    pub fn path(&self) -> Option<&str> {
        self.bundle_path.as_ref().map(|(p, _)| p.as_str())
    }

    /// Convenience: which env var supplied the bundle file path, if any.
    pub fn source(&self) -> Option<CertBundleSource> {
        self.bundle_path.as_ref().map(|(_, s)| *s)
    }
}

/// Validate that an explicit `--cert <path>` CLI argument is non-empty.
/// File existence checks are left to the HTTP-client layer (which is
/// the only layer that should touch the filesystem here).
pub fn validate_cert_arg(path: &str) -> Result<&str, IndexError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(IndexError::ParseError {
            url: "<--cert>".into(),
            detail: "--cert path is empty".into(),
        });
    }
    Ok(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn lookup(map: HashMap<String, String>) -> impl Fn(&str) -> Option<String> {
        move |k| map.get(k).cloned()
    }

    #[test]
    fn empty_env_yields_empty_bundle() {
        let cb = CertBundle::from_env(lookup(env(&[])));
        assert!(cb.is_empty());
        assert!(cb.path().is_none());
        assert!(cb.source().is_none());
    }

    #[test]
    fn pip_cert_takes_highest_precedence() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("PIP_CERT", "/etc/pip/ca.pem"),
            ("REQUESTS_CA_BUNDLE", "/etc/requests/ca.pem"),
            ("CURL_CA_BUNDLE", "/etc/curl/ca.pem"),
            ("SSL_CERT_FILE", "/etc/ssl/cert.pem"),
        ])));
        assert_eq!(cb.path(), Some("/etc/pip/ca.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::PipCert));
    }

    #[test]
    fn requests_ca_bundle_beats_curl_and_ssl_cert_file() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("REQUESTS_CA_BUNDLE", "/etc/requests/ca.pem"),
            ("CURL_CA_BUNDLE", "/etc/curl/ca.pem"),
            ("SSL_CERT_FILE", "/etc/ssl/cert.pem"),
        ])));
        assert_eq!(cb.path(), Some("/etc/requests/ca.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::RequestsCaBundle));
    }

    #[test]
    fn curl_ca_bundle_beats_ssl_cert_file() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("CURL_CA_BUNDLE", "/etc/curl/ca.pem"),
            ("SSL_CERT_FILE", "/etc/ssl/cert.pem"),
        ])));
        assert_eq!(cb.path(), Some("/etc/curl/ca.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::CurlCaBundle));
    }

    #[test]
    fn ssl_cert_file_used_when_no_higher_precedence_present() {
        let cb = CertBundle::from_env(lookup(env(&[("SSL_CERT_FILE", "/etc/ssl/cert.pem")])));
        assert_eq!(cb.path(), Some("/etc/ssl/cert.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::SslCertFile));
    }

    #[test]
    fn ssl_cert_dir_is_orthogonal_to_bundle_path() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("SSL_CERT_FILE", "/etc/ssl/cert.pem"),
            ("SSL_CERT_DIR", "/etc/ssl/certs"),
        ])));
        assert_eq!(cb.path(), Some("/etc/ssl/cert.pem"));
        assert_eq!(cb.bundle_dir.as_deref(), Some("/etc/ssl/certs"));
    }

    #[test]
    fn ssl_cert_dir_alone_leaves_bundle_path_unset() {
        let cb = CertBundle::from_env(lookup(env(&[("SSL_CERT_DIR", "/etc/ssl/certs")])));
        assert!(cb.path().is_none());
        assert_eq!(cb.bundle_dir.as_deref(), Some("/etc/ssl/certs"));
        assert!(!cb.is_empty());
    }

    #[test]
    fn lowercase_takes_precedence_over_uppercase() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("requests_ca_bundle", "/lower/ca.pem"),
            ("REQUESTS_CA_BUNDLE", "/upper/ca.pem"),
        ])));
        assert_eq!(cb.path(), Some("/lower/ca.pem"));
    }

    #[test]
    fn empty_string_is_treated_as_unset() {
        let cb = CertBundle::from_env(lookup(env(&[
            ("PIP_CERT", "   "),
            ("REQUESTS_CA_BUNDLE", ""),
            ("SSL_CERT_FILE", "/etc/ssl/cert.pem"),
        ])));
        assert_eq!(cb.path(), Some("/etc/ssl/cert.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::SslCertFile));
    }

    #[test]
    fn paths_are_trimmed() {
        let cb = CertBundle::from_env(lookup(env(&[(
            "SSL_CERT_FILE",
            "  /etc/ssl/cert.pem  ",
        )])));
        assert_eq!(cb.path(), Some("/etc/ssl/cert.pem"));
    }

    #[test]
    fn env_var_names_round_trip() {
        assert_eq!(CertBundleSource::PipCert.env_var_name(), "PIP_CERT");
        assert_eq!(
            CertBundleSource::RequestsCaBundle.env_var_name(),
            "REQUESTS_CA_BUNDLE"
        );
        assert_eq!(CertBundleSource::CurlCaBundle.env_var_name(), "CURL_CA_BUNDLE");
        assert_eq!(CertBundleSource::SslCertFile.env_var_name(), "SSL_CERT_FILE");
    }

    #[test]
    fn validate_cert_arg_rejects_empty_and_whitespace() {
        assert!(validate_cert_arg("").is_err());
        assert!(validate_cert_arg("   ").is_err());
        assert_eq!(validate_cert_arg("  /etc/ca.pem  ").unwrap(), "/etc/ca.pem");
    }

    #[test]
    fn realistic_uv_dev_env_resolves_to_requests_ca_bundle() {
        // Common dev setup: corporate CA injected via REQUESTS_CA_BUNDLE
        // (the Python requests convention) without overriding system
        // SSL_CERT_FILE.
        let cb = CertBundle::from_env(lookup(env(&[
            ("REQUESTS_CA_BUNDLE", "/opt/corp/ca/bundle.pem"),
            ("SSL_CERT_DIR", "/etc/ssl/certs"),
        ])));
        assert_eq!(cb.path(), Some("/opt/corp/ca/bundle.pem"));
        assert_eq!(cb.source(), Some(CertBundleSource::RequestsCaBundle));
        assert_eq!(cb.bundle_dir.as_deref(), Some("/etc/ssl/certs"));
    }
}
