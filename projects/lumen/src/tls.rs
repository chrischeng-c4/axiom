// SPEC-MANAGED: projects/lumen/tech-design/semantic/source/projects-lumen-src-tls-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! mTLS configuration for the peer (`:8082`) transport.
//!
//! v1 ships the configuration surface — paths to cert / key / CA bundle
//! and an `is_required` flag — so deployments can declare their TLS
//! posture today. The rustls binding is wired in alongside the raftcore-backed
//! peer transport.
//!
//! ## Env contract
//!
//! - `LUMEN_PEER_TLS_CERT` — path to this pod's PEM cert chain.
//! - `LUMEN_PEER_TLS_KEY`  — path to its private key.
//! - `LUMEN_PEER_TLS_CA`   — path to the CA bundle peers are verified against.
//! - `LUMEN_PEER_MTLS=on|off` — when `on`, non-mTLS peer connections are rejected.
//!
//! The presence of all three paths + `LUMEN_PEER_MTLS=on` enables mTLS;
//! any other combination falls back to plain HTTP/2 (with a warning).

use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Once};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-tls-rs.md#source
pub struct PeerTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    pub required: bool,
}

/// @spec projects/lumen/tech-design/semantic/source/projects-lumen-src-tls-rs.md#source
impl PeerTlsConfig {
    /// Load from env. Returns `Ok(None)` when no TLS material is
    /// configured (plain-HTTP peer transport).
    pub fn from_env() -> Result<Option<Self>> {
        let cert = std::env::var("LUMEN_PEER_TLS_CERT").ok().map(PathBuf::from);
        let key = std::env::var("LUMEN_PEER_TLS_KEY").ok().map(PathBuf::from);
        let ca = std::env::var("LUMEN_PEER_TLS_CA").ok().map(PathBuf::from);
        let required = std::env::var("LUMEN_PEER_MTLS")
            .map(|v| v.eq_ignore_ascii_case("on"))
            .unwrap_or(false);
        match (cert, key, ca) {
            (Some(cert), Some(key), Some(ca)) => {
                let cfg = Self {
                    cert,
                    key,
                    ca,
                    required,
                };
                cfg.verify_paths()?;
                Ok(Some(cfg))
            }
            (None, None, None) if !required => Ok(None),
            (None, None, None) => Err(anyhow!(
                "LUMEN_PEER_MTLS=on but no cert/key/ca paths set"
            )),
            _ => Err(anyhow!(
                "LUMEN_PEER_TLS_CERT / LUMEN_PEER_TLS_KEY / LUMEN_PEER_TLS_CA must all be set together"
            )),
        }
    }

    fn verify_paths(&self) -> Result<()> {
        for (name, p) in [("cert", &self.cert), ("key", &self.key), ("ca", &self.ca)] {
            if !p.exists() {
                return Err(anyhow!("TLS {name} not found at {}", p.display()));
            }
            std::fs::metadata(p).with_context(|| format!("stat {name} {}", p.display()))?;
        }
        Ok(())
    }

    /// Build a rustls server config for the peer transport.
    pub fn rustls_server_config(&self) -> Result<rustls::ServerConfig> {
        install_default_crypto_provider();
        let cert_chain = load_cert_chain(&self.cert)?;
        let key = load_private_key(&self.key)?;
        let builder = rustls::ServerConfig::builder();
        let server = if self.required {
            let client_roots = load_root_store(&self.ca)?;
            let verifier =
                rustls::server::WebPkiClientVerifier::builder(Arc::new(client_roots)).build()?;
            builder.with_client_cert_verifier(verifier)
        } else {
            builder.with_no_client_auth()
        };
        server
            .with_single_cert(cert_chain, key)
            .context("build peer rustls server config")
    }

    /// Build a rustls client config for dialing peer transports.
    pub fn rustls_client_config(&self) -> Result<rustls::ClientConfig> {
        install_default_crypto_provider();
        let roots = load_root_store(&self.ca)?;
        let cert_chain = load_cert_chain(&self.cert)?;
        let key = load_private_key(&self.key)?;
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_client_auth_cert(cert_chain, key)
            .context("build peer rustls client config")
    }
}

fn install_default_crypto_provider() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });
}

fn load_cert_chain(path: &Path) -> Result<Vec<rustls::pki_types::CertificateDer<'static>>> {
    let file =
        std::fs::File::open(path).with_context(|| format!("open cert {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("parse cert {}", path.display()))?;
    if certs.is_empty() {
        return Err(anyhow!("no certificates found at {}", path.display()));
    }
    Ok(certs)
}

fn load_private_key(path: &Path) -> Result<rustls::pki_types::PrivateKeyDer<'static>> {
    let file = std::fs::File::open(path).with_context(|| format!("open key {}", path.display()))?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::private_key(&mut reader)
        .with_context(|| format!("parse key {}", path.display()))?
        .ok_or_else(|| anyhow!("no private key found at {}", path.display()))
}

fn load_root_store(path: &Path) -> Result<rustls::RootCertStore> {
    let mut roots = rustls::RootCertStore::empty();
    for cert in load_cert_chain(path)? {
        roots
            .add(cert)
            .with_context(|| format!("add CA cert from {}", path.display()))?;
    }
    Ok(roots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const TEST_CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIC5zCCAc+gAwIBAgIJAPl6HZTX5LElMA0GCSqGSIb3DQEBCwUAMBUxEzARBgNV
BAMMCmx1bWVuLXBlZXIwHhcNMjYwNjE4MTQwODA4WhcNMjYwNzE4MTQwODA4WjAV
MRMwEQYDVQQDDApsdW1lbi1wZWVyMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIB
CgKCAQEAuwEAFs8xYsY9GDDbefwoV4FMiD9f49hs7iLVijVhUM7z5W0Xj9mXFCFS
Sn/DNb/bF9UtUoGJ0cdpjlevd6BjaXbjm2gMIDod1yKBZ2BXwT/elwRzjEIcTgR5
+GTu355VsWqugBYob8cYn2kGAMvVFUZeRBbC1IO02xbp9zABNaBHOWVdRTXODxiU
jbtB4gioNJOG1A71sto61lMmLMp4IL02k+BbuwekhCkkRGGNuqMHVAehkJwTmmxF
aPHK3LMifWgUXn51JWEhU2OiWe3Ja8/XQU5LZDvbc3vmMaJSuIMheOIkM5AXHyo4
LX62YgtuUpouYYOHkOqNRWRQfLrvywIDAQABozowODAVBgNVHREEDjAMggpsdW1l
bi1wZWVyMA8GA1UdEwEB/wQFMAMBAf8wDgYDVR0PAQH/BAQDAgKkMA0GCSqGSIb3
DQEBCwUAA4IBAQAavsvsmN/zKL0TVx7FLEnDRbD6L4KNg3ndPrZDKncl0Df1W5kl
4jZTujiZ2CqH7CQakra3kV51EIUuKSbc0kQBsvsCIw0Fxb/JUmsui/z9uCqrqhrT
ODlcV6pETppce5JozMAZCKUyx9460/+flP7VTqHnLt1oMrM/mmaKeZ0ImSBnx8xF
0JpJN0HyX+vlbrT/9J3xxe53v7glRPZIgBlOT1eTaroXjIk6ZzOBS8bCBpNYVec5
wN93qI3ZQWwNUMB3TXJ7IBpgIrtD+z/ZhliDnk6NOLqKPXJrch0cVwlljT0Uu+DP
Qd9/aITxkqX7P0phj2cYmALL/aBJJaWRuAfw
-----END CERTIFICATE-----
"#;

    const TEST_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
MIIEvgIBADANBgkqhkiG9w0BAQEFAASCBKgwggSkAgEAAoIBAQC7AQAWzzFixj0Y
MNt5/ChXgUyIP1/j2GzuItWKNWFQzvPlbReP2ZcUIVJKf8M1v9sX1S1SgYnRx2mO
V693oGNpduObaAwgOh3XIoFnYFfBP96XBHOMQhxOBHn4ZO7fnlWxaq6AFihvxxif
aQYAy9UVRl5EFsLUg7TbFun3MAE1oEc5ZV1FNc4PGJSNu0HiCKg0k4bUDvWy2jrW
UyYsynggvTaT4Fu7B6SEKSREYY26owdUB6GQnBOabEVo8crcsyJ9aBRefnUlYSFT
Y6JZ7clrz9dBTktkO9tze+YxolK4gyF44iQzkBcfKjgtfrZiC25Smi5hg4eQ6o1F
ZFB8uu/LAgMBAAECggEAF23pp/HvmxOBRg2hAeiQ2V3Oy+c8yVwtUay1mmpTtf8n
2Z/Qaup1HkWKfOEDATH3bkX8NrEaJllYpUfhKRjEO8t0et0PX95ILNMa6WvNst2g
ssURAQqrZy7yZSeoMgYxcFgQYuXjzRVhxV8wLFtdaBv35YoAgQW7XBPD3n96N1CV
nRpk/tPeIaqmGnC6xhtrd9zaRy1qZ3aX5Np27ZrwsMghmJyLNI6OfsS0FRK3dKQx
dkx5L5iMD0wmqC5FsR4nc+pkFsSgdv2uxtS95JDX2jOHuJj4qm5moh0Z6eXQ8lCD
Nhr+JN1TQXHVAL696tQPnQNJtdnQYshNpsC+R2Sg+QKBgQDjsyACww2OLLIO/QBJ
rzbuAgx0n4cRR7mSCZhrgO3xX+sKU1yGPNRtwj0R/dQsVClld0KB2Oa9brR2dzcE
QWSLGcRhAmpjgmYLFn6T2Odbyb5YfTMVF6ka53w4CELKQPm7cm5QsXimTqSYPwZb
Jth2e7bkEdVemzDS8C33WYio3QKBgQDSPweWZNLC+EtHlNHg4fguww6wjkUPcoxG
C8prGovcSEMuIXUnrJmRkWdKTxHud9ofvhfauB86Daf4tkaklGPuJ5CepxnMXyos
I8fSEnIyTPD6sYC37GNUMDhMU3iyxV+CsH077TwSGpjw4cntf8pqYklP8zjctjnq
wAPG6O2cxwKBgQCTEQnW3tatgo7LAXwjG2k2FtqmpLbfYV0pRstMfCyzHwm3VJpJ
FZb7AV7idPiKXR2TrJCnP0nhBlTGwz8kn3vqIA1nvuCqPvnbpX7BzXG5Jjer/cl1
kR+nAeaIZkWFTqw99q3rroTHnbnPn71iOFfNRyCcdCxE+6VwSLLXtNuAfQKBgQDA
05QW6FOxA96vQRuY4EcqRDXV0jYeq9VhbPDyeD9sAk6zIXZ8s72JF82fBpQQnZXN
ZSAltpbVPK8g2bRCv+JDC8CE8gckPOfF4e8jiU15Or4NfvzqMwEKtsr7ndbmR0WI
7Gt/qd5dUE2TJ9J2Y6z3Ezvf+tfc/bhyyDbumLVNAwKBgHj74ZKxCKE21mv1azYk
EF1sOEisJVtdtSq2PZN7hiGgvaMTSfKegRgM+12lGDvabf93LSoYX1pEHY7qIs2f
pky/zqjmfLFtvyP+vQvAL3F+5B/1XpFj2dRnAOJaWpq62Ebe2L9k4ff7EYNTL7oq
LkjT2UdpFBDZGWHwqDRhXX8k
-----END PRIVATE KEY-----
"#;

    // env vars are process-global, so the three scenarios share a
    // mutex to keep them from racing under `cargo test`'s default
    // parallel runner.
    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        unsafe {
            std::env::remove_var("LUMEN_PEER_TLS_CERT");
            std::env::remove_var("LUMEN_PEER_TLS_KEY");
            std::env::remove_var("LUMEN_PEER_TLS_CA");
            std::env::remove_var("LUMEN_PEER_MTLS");
        }
    }

    fn write_tls_fixture(name: &str) -> PeerTlsConfig {
        let dir =
            std::env::temp_dir().join(format!("lumen-tls-rustls-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cert.pem"), TEST_CERT).unwrap();
        std::fs::write(dir.join("key.pem"), TEST_KEY).unwrap();
        std::fs::write(dir.join("ca.pem"), TEST_CERT).unwrap();
        PeerTlsConfig {
            cert: dir.join("cert.pem"),
            key: dir.join("key.pem"),
            ca: dir.join("ca.pem"),
            required: true,
        }
    }

    #[test]
    fn from_env_returns_none_when_nothing_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        let cfg = PeerTlsConfig::from_env().unwrap();
        assert!(cfg.is_none());
    }

    #[test]
    fn from_env_loads_when_all_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        let dir = std::env::temp_dir().join(format!("lumen-tls-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["cert.pem", "key.pem", "ca.pem"] {
            let mut f = std::fs::File::create(dir.join(name)).unwrap();
            f.write_all(b"DUMMY").unwrap();
        }
        unsafe {
            std::env::set_var("LUMEN_PEER_TLS_CERT", dir.join("cert.pem"));
            std::env::set_var("LUMEN_PEER_TLS_KEY", dir.join("key.pem"));
            std::env::set_var("LUMEN_PEER_TLS_CA", dir.join("ca.pem"));
            std::env::set_var("LUMEN_PEER_MTLS", "on");
        }
        let cfg = PeerTlsConfig::from_env().unwrap().expect("Some");
        assert!(cfg.required);
        std::fs::remove_dir_all(&dir).ok();
        clear_env();
    }

    #[test]
    fn from_env_errors_on_partial_config() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        unsafe {
            std::env::set_var("LUMEN_PEER_TLS_CERT", "/tmp/dummy-cert");
        }
        let err = PeerTlsConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("must all be set together"));
        clear_env();
    }

    #[test]
    fn builds_rustls_peer_configs_from_pem_material() {
        let cfg = write_tls_fixture("builder");
        cfg.rustls_server_config()
            .expect("server config should build");
        cfg.rustls_client_config()
            .expect("client config should build");
        std::fs::remove_dir_all(cfg.cert.parent().unwrap()).ok();
    }
}
// CODEGEN-END
