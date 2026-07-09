// SPEC-MANAGED: apps/tape/tech-design/logic/tape-raft-host-primary-replicas.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:f0da944f" tracker="pending-tracker" reason="Thin adapter over libs/service-tls mirroring relay's src/peer_tls.rs: TAPE_PEER_TLS_CERT/KEY/CA + TAPE_PEER_MTLS=on|off env contract, PeerTlsConfig::from_env() (None when unset, error on partial config or a mis-pointed path), rustls_server_config/rustls_client_config passthroughs. Config-surface + fail-fast validation only -- raft-host's h2c peer transport has no TLS acceptor/connector seam yet (the shared gap also filed against relay/keep/lumen); termination is not applied."
//! Peer-mTLS material for the raft peer surface (#1327).
//!
//! ## Env contract (relay/lumen's names with the TAPE prefix)
//!
//! - `TAPE_PEER_TLS_CERT` -- path to this pod's PEM cert chain.
//! - `TAPE_PEER_TLS_KEY`  -- path to its private key.
//! - `TAPE_PEER_TLS_CA`   -- path to the CA bundle peers are verified against.
//! - `TAPE_PEER_MTLS=on|off` -- when `on`, non-mTLS peer connections must be
//!   rejected once termination is wired.
//!
//! The PEM loading, the rustls server/client config builders, and the
//! Once-guarded crypto-provider install are generic across every service with
//! a peer/replication port and live in `libs/service-tls`; this module is the
//! thin adapter that pins tape's `TAPE_PEER_TLS_*` / `TAPE_PEER_MTLS` env
//! names.
//!
//! ## Wiring depth (deliberate; the filed seam gap)
//!
//! The serve path loads + validates this config in replica/HA mode BEFORE the
//! raft group spawns -- partial config or a mis-pointed path is a startup
//! error, never a silent fallback -- and proves the rustls builders
//! constructible. **mTLS termination on the raft peer port is NOT yet
//! applied**: raft-host's peer transport is h2c prior-knowledge (the peer
//! router rides the cleartext serve port; peers are dialed over `http://`)
//! with no TLS acceptor/connector seam. Wiring real termination needs
//! raft-host to accept a rustls `ServerConfig`/`ClientConfig` pair -- a
//! `libs/raft-host` change benefiting keep/lumen/relay/tape alike. This
//! module deliberately does not hack a parallel TLS stack into h2c; the env
//! contract + startup validation land now so deployments can mount and verify
//! material before the seam exists.

use std::path::PathBuf;

use anyhow::Result;

/// The prefix passed to `service_tls::PeerTlsConfig::from_env`: derives
/// `TAPE_PEER_TLS_CERT` / `TAPE_PEER_TLS_KEY` / `TAPE_PEER_TLS_CA` /
/// `TAPE_PEER_MTLS`.
const ENV_PREFIX: &str = "TAPE_PEER";

/// Peer TLS material paths + the mTLS-required toggle.
#[derive(Debug, Clone)]
pub struct PeerTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    pub required: bool,
}

impl From<service_tls::PeerTlsConfig> for PeerTlsConfig {
    fn from(cfg: service_tls::PeerTlsConfig) -> Self {
        Self {
            cert: cfg.cert,
            key: cfg.key,
            ca: cfg.ca,
            required: cfg.required,
        }
    }
}

impl From<PeerTlsConfig> for service_tls::PeerTlsConfig {
    fn from(cfg: PeerTlsConfig) -> Self {
        Self {
            cert: cfg.cert,
            key: cfg.key,
            ca: cfg.ca,
            required: cfg.required,
        }
    }
}

impl PeerTlsConfig {
    /// Load from env. Returns `Ok(None)` when no TLS material is configured
    /// (plain-h2c peer transport); an error when the config is partial
    /// (all three paths must be set together) or a path does not exist
    /// (fail fast, naming the path).
    pub fn from_env() -> Result<Option<Self>> {
        Ok(service_tls::PeerTlsConfig::from_env(ENV_PREFIX)?.map(Self::from))
    }

    /// Build a rustls server config for the peer transport (client-cert
    /// verification against the CA bundle when `required`).
    pub fn rustls_server_config(&self) -> Result<rustls::ServerConfig> {
        service_tls::PeerTlsConfig::from(self.clone()).rustls_server_config()
    }

    /// Build a rustls client config for dialing peer transports.
    pub fn rustls_client_config(&self) -> Result<rustls::ClientConfig> {
        service_tls::PeerTlsConfig::from(self.clone()).rustls_client_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    use std::sync::Mutex;
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        unsafe {
            std::env::remove_var("TAPE_PEER_TLS_CERT");
            std::env::remove_var("TAPE_PEER_TLS_KEY");
            std::env::remove_var("TAPE_PEER_TLS_CA");
            std::env::remove_var("TAPE_PEER_MTLS");
        }
    }

    fn write_tls_fixture(name: &str) -> PeerTlsConfig {
        let dir = std::env::temp_dir().join(format!("tape-peer-tls-{}-{name}", std::process::id()));
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

    /// R6: nothing set => `Ok(None)` (plain h2c peers, today's default).
    #[test]
    fn from_env_returns_none_when_nothing_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        let cfg = PeerTlsConfig::from_env().unwrap();
        assert!(cfg.is_none());
    }

    /// R6: all three paths + `TAPE_PEER_MTLS=on` load with `required = true`.
    #[test]
    fn from_env_loads_when_all_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        let dir = std::env::temp_dir().join(format!("tape-peer-tls-env-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["cert.pem", "key.pem", "ca.pem"] {
            std::fs::write(dir.join(name), b"DUMMY").unwrap();
        }
        unsafe {
            std::env::set_var("TAPE_PEER_TLS_CERT", dir.join("cert.pem"));
            std::env::set_var("TAPE_PEER_TLS_KEY", dir.join("key.pem"));
            std::env::set_var("TAPE_PEER_TLS_CA", dir.join("ca.pem"));
            std::env::set_var("TAPE_PEER_MTLS", "on");
        }
        let cfg = PeerTlsConfig::from_env().unwrap().expect("Some");
        assert!(cfg.required);
        std::fs::remove_dir_all(&dir).ok();
        clear_env();
    }

    /// R6: partial config is a startup error naming the var family -- never a
    /// silent fallback to plaintext.
    #[test]
    fn from_env_errors_on_partial_config() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        unsafe {
            std::env::set_var("TAPE_PEER_TLS_CERT", "/tmp/dummy-cert");
        }
        let err = PeerTlsConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("must all be set together"));
        clear_env();
    }

    /// R6: a mis-pointed cert path fails fast with an error naming the
    /// missing path.
    #[test]
    fn mis_pointed_cert_path_fails_fast_naming_the_path() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        let dir = std::env::temp_dir().join(format!("tape-peer-tls-miss-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("key.pem"), b"DUMMY").unwrap();
        std::fs::write(dir.join("ca.pem"), b"DUMMY").unwrap();
        unsafe {
            std::env::set_var("TAPE_PEER_TLS_CERT", dir.join("missing-cert.pem"));
            std::env::set_var("TAPE_PEER_TLS_KEY", dir.join("key.pem"));
            std::env::set_var("TAPE_PEER_TLS_CA", dir.join("ca.pem"));
            std::env::set_var("TAPE_PEER_MTLS", "on");
        }
        let err = PeerTlsConfig::from_env().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing-cert.pem"),
            "error must name the mis-pointed path, got: {msg}"
        );
        std::fs::remove_dir_all(&dir).ok();
        clear_env();
    }

    /// R6: the PEM fixture builds both rustls configs through the adapter
    /// passthroughs -- the material is proven usable even though the
    /// raft-host/h2c seam cannot terminate mTLS yet (filed gap).
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
// HANDWRITE-END
