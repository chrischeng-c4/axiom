//! mTLS configuration for the peer (`:8082`) transport.
//!
//! v1 ships the configuration surface — paths to cert / key / CA bundle
//! and an `is_required` flag — so deployments can declare their TLS
//! posture today. The rustls binding is wired in alongside the openraft
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

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct PeerTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    pub required: bool,
}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
}
