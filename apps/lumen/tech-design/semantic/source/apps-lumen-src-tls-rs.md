---
id: projects-lumen-src-tls-rs
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "query-planner-boolean-eval-roaring-postings"
    coverage: partial
    rationale: >
      This source unit is captured as a per-file rust-source-unit during lumen
      td_ast standardization. Thinned to an adapter by #971: PEM cert/key/CA
      loading, the rustls server/client config builders, and the Once-guarded
      crypto-provider install moved verbatim to the shared `libs/peer-tls`
      crate (parameterized by an env prefix); this file keeps only the
      `LUMEN_PEER_TLS_*`/`LUMEN_PEER_MTLS` env names and the lumen-facing
      `PeerTlsConfig` type, delegating to `peer_tls::PeerTlsConfig` via
      `From` conversions.
fill_sections: [overview, source, changes]
---

# Standardized apps/lumen/src/tls.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/lumen/src/tls.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `PeerTlsConfig` | apps/lumen/src/tls.rs | struct | pub | 34 |  |
| `from_env` | apps/lumen/src/tls.rs | function | pub | 66 | from_env() -> Result<Option<Self>> |
| `peer_transport` | apps/lumen/src/tls.rs | function | pub | 83 | peer_transport(&self) -> Result<raft_runtime::PeerTransport> |
| `reloadable` | apps/lumen/src/tls.rs | function | pub | 101 | reloadable(&self, dns_names: impl IntoIterator<Item = String>, spiffe_uris: impl IntoIterator<Item = String>) -> Result<peer_tls::ReloadableTls> |
| `rustls_client_config` | apps/lumen/src/tls.rs | function | pub | 76 | rustls_client_config(&self) -> Result<rustls::ClientConfig> |
| `rustls_server_config` | apps/lumen/src/tls.rs | function | pub | 71 | rustls_server_config(&self) -> Result<rustls::ServerConfig> |
## Source
<!-- type: rust-source-unit lang: rust -->


```rust
// SPEC-MANAGED: apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! mTLS configuration for the peer (`:8082`) transport.
//!
//! v1 ships the configuration surface — paths to cert / key / CA bundle
//! and an `is_required` flag — so deployments can declare their TLS
//! posture today. The rustls binding is wired in alongside the raft_core-backed
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
//!
//! The PEM loading, rustls server/client config builders, and the
//! Once-guarded crypto-provider install are generic across every service
//! with a peer/replication port and live in `libs/peer-tls` (#971); this
//! module is a thin adapter over it that keeps lumen's `LUMEN_PEER_TLS_*`/
//! `LUMEN_PEER_MTLS` env names and pub API unchanged.

use std::path::PathBuf;

use anyhow::Result;

/// The prefix passed to `peer_tls::PeerTlsConfig::from_env`: derives
/// `LUMEN_PEER_TLS_CERT` / `LUMEN_PEER_TLS_KEY` / `LUMEN_PEER_TLS_CA` /
/// `LUMEN_PEER_MTLS`, preserving lumen's env contract byte-for-byte.
const ENV_PREFIX: &str = "LUMEN_PEER";

#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
pub struct PeerTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    pub required: bool,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
impl From<peer_tls::PeerTlsConfig> for PeerTlsConfig {
    fn from(cfg: peer_tls::PeerTlsConfig) -> Self {
        Self {
            cert: cfg.cert,
            key: cfg.key,
            ca: cfg.ca,
            required: cfg.required,
        }
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
impl From<PeerTlsConfig> for peer_tls::PeerTlsConfig {
    fn from(cfg: PeerTlsConfig) -> Self {
        Self {
            cert: cfg.cert,
            key: cfg.key,
            ca: cfg.ca,
            required: cfg.required,
        }
    }
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
impl PeerTlsConfig {
    /// Load from env. Returns `Ok(None)` when no TLS material is
    /// configured (plain-HTTP peer transport).
    pub fn from_env() -> Result<Option<Self>> {
        Ok(peer_tls::PeerTlsConfig::from_env(ENV_PREFIX)?.map(Self::from))
    }

    /// Build a rustls server config for the peer transport.
    pub fn rustls_server_config(&self) -> Result<rustls::ServerConfig> {
        peer_tls::PeerTlsConfig::from(self.clone()).rustls_server_config()
    }

    /// Build a rustls client config for dialing peer transports.
    pub fn rustls_client_config(&self) -> Result<rustls::ClientConfig> {
        peer_tls::PeerTlsConfig::from(self.clone()).rustls_client_config()
    }

    /// Construct the shared reloadable Raft peer transport. Lumen owns only
    /// env naming; TLS connection, identity, and reload semantics stay in
    /// `raft-runtime`.
    pub fn peer_transport(&self) -> Result<raft_runtime::PeerTransport> {
        let config = peer_tls::PeerTlsConfig::from(self.clone());
        raft_runtime::PeerTransport::from_config(&config)
    }

    /// Bind this member's projected peer material to the shared reloadable
    /// seam (#3112 R2).
    ///
    /// Lumen contributes the only two things the library cannot know — where
    /// the Secret is projected, and which identity *this* member must present —
    /// and nothing else. Validation, last-known-good retention, trust overlap
    /// during issuer rotation, and atomic activation all stay in
    /// [`peer_tls::reload`]; there is deliberately no Lumen-side reload engine
    /// for them to diverge from.
    ///
    /// Fails when no valid material exists at startup, which is the intended
    /// posture: a member that cannot prove who it is has no business joining
    /// the group.
    pub fn reloadable(
        &self,
        dns_names: impl IntoIterator<Item = String>,
        spiffe_uris: impl IntoIterator<Item = String>,
    ) -> Result<peer_tls::ReloadableTls> {
        peer_tls::ReloadableTls::required(
            peer_tls::TlsRuntimeProfile::peer(dns_names, spiffe_uris),
            std::sync::Arc::new(peer_tls::FileMaterialSource::new(
                &self.cert,
                &self.key,
                &self.ca,
            )),
        )
        .map_err(anyhow::Error::from)
    }
}

/// The switch that turns the client port into a TLS listener (#3113 R1).
const SERVING_TLS_ENV: &str = "LUMEN_TLS";

/// Where the serving leaf is projected, for the client port on `:7373`.
///
/// A separate type from [`PeerTlsConfig`] and not a mode of it. The peer port's
/// question is "is the dialer a member of this Raft group", answered by a
/// client certificate; the client port's question is "is the server the Service
/// I asked for", answered by this leaf while the *caller* proves itself with a
/// short-lived ServiceAccount token. There is no `required` flag here because
/// there is no mutual half to make optional.
///
/// ## Env contract
///
/// - `LUMEN_TLS=on` — serve TLS. Anything else leaves the port h2c.
/// - `LUMEN_TLS_CERT` / `LUMEN_TLS_KEY` / `LUMEN_TLS_CA` — the projected leaf,
///   its key, and the anchor callers are told to trust.
/// - `LUMEN_TLS_SERVER_NAMES` — comma-separated Service DNS names the leaf must
///   answer to. Optional; when absent the leaf is accepted for whatever names
///   it carries.
#[derive(Debug, Clone)]
/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
pub struct ServingTlsConfig {
    pub cert: PathBuf,
    pub key: PathBuf,
    pub ca: PathBuf,
    /// The Kubernetes Service DNS names callers dial. Checked against the leaf
    /// at load, so a certificate issued for some *other* Service fails here
    /// once instead of at every client in turn.
    pub dns_names: Vec<String>,
}

/// @spec apps/lumen/tech-design/semantic/source/apps-lumen-src-tls-rs.md#source
impl ServingTlsConfig {
    /// Load from env. `Ok(None)` means the deployment asked for h2c — the
    /// local/kind posture, and the only way to get cleartext on this port.
    ///
    /// Half a configuration is an error rather than a silent downgrade in
    /// either direction: paths without the switch would serve cleartext from a
    /// deployment that projected a certificate, and the switch without paths
    /// would come up with nothing to present. Both are the same mistake seen
    /// from opposite sides, and both are worth failing startup over.
    pub fn from_env() -> Result<Option<Self>> {
        let on = std::env::var(SERVING_TLS_ENV)
            .map(|v| v.eq_ignore_ascii_case("on"))
            .unwrap_or(false);
        let cert = std::env::var("LUMEN_TLS_CERT").ok().map(PathBuf::from);
        let key = std::env::var("LUMEN_TLS_KEY").ok().map(PathBuf::from);
        let ca = std::env::var("LUMEN_TLS_CA").ok().map(PathBuf::from);
        let dns_names = std::env::var("LUMEN_TLS_SERVER_NAMES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect();
        match (on, cert, key, ca) {
            (false, None, None, None) => Ok(None),
            (true, Some(cert), Some(key), Some(ca)) => Ok(Some(Self {
                cert,
                key,
                ca,
                dns_names,
            })),
            (true, ..) => Err(anyhow::anyhow!(
                "{SERVING_TLS_ENV}=on but LUMEN_TLS_CERT / LUMEN_TLS_KEY / LUMEN_TLS_CA are not all set"
            )),
            (false, ..) => Err(anyhow::anyhow!(
                "LUMEN_TLS_CERT / LUMEN_TLS_KEY / LUMEN_TLS_CA are set but {SERVING_TLS_ENV} is not `on`; \
                 the client port would serve cleartext against a projected certificate"
            )),
        }
    }

    /// Bind the projected material to the shared reloadable seam (#3113 R1/R9).
    ///
    /// `required`, so a pod with no valid leaf never reaches the accept loop.
    /// Rotation after that point is [`peer_tls::reload`]'s business — the
    /// listener re-reads this on every accept, which is what makes a renewal
    /// cost no restart.
    pub fn reloadable(&self) -> Result<peer_tls::ReloadableTls> {
        peer_tls::ReloadableTls::required(
            peer_tls::TlsRuntimeProfile::serving(self.dns_names.clone()),
            std::sync::Arc::new(peer_tls::FileMaterialSource::new(
                &self.cert,
                &self.key,
                &self.ca,
            )),
        )
        .map_err(anyhow::Error::from)
    }
}

pub use peer_tls::install_default_crypto_provider;

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
            std::env::remove_var("LUMEN_TLS");
            std::env::remove_var("LUMEN_TLS_CERT");
            std::env::remove_var("LUMEN_TLS_KEY");
            std::env::remove_var("LUMEN_TLS_CA");
            std::env::remove_var("LUMEN_TLS_SERVER_NAMES");
        }
    }

    // ---- #3113 R1: the serving port's env contract ------------------------
    //
    // Three states, and the two that are neither "TLS" nor "h2c" are errors.
    // The interesting one is the last: material projected with the switch left
    // off is the shape in which a deployment believes it is serving TLS and
    // is not, and nothing downstream would say otherwise.

    #[test]
    fn serving_tls_from_env_is_none_when_the_deployment_asked_for_h2c() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        assert!(ServingTlsConfig::from_env().unwrap().is_none());
    }

    #[test]
    fn serving_tls_from_env_reads_the_paths_and_the_service_names() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        unsafe {
            std::env::set_var("LUMEN_TLS", "on");
            std::env::set_var("LUMEN_TLS_CERT", "/var/run/secrets/lumen-serving/tls.crt");
            std::env::set_var("LUMEN_TLS_KEY", "/var/run/secrets/lumen-serving/tls.key");
            std::env::set_var("LUMEN_TLS_CA", "/var/run/secrets/lumen-serving/ca.crt");
            std::env::set_var(
                "LUMEN_TLS_SERVER_NAMES",
                "search.acme.svc, search.acme.svc.cluster.local",
            );
        }
        let cfg = ServingTlsConfig::from_env().unwrap().expect("Some");
        assert_eq!(
            cfg.cert.to_string_lossy(),
            "/var/run/secrets/lumen-serving/tls.crt"
        );
        assert_eq!(
            cfg.dns_names,
            vec![
                "search.acme.svc".to_string(),
                "search.acme.svc.cluster.local".to_string()
            ],
            "both Service DNS forms, whitespace-tolerant"
        );
        clear_env();
    }

    #[test]
    fn serving_tls_on_without_material_fails_startup() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        unsafe {
            std::env::set_var("LUMEN_TLS", "on");
        }
        let err = ServingTlsConfig::from_env().unwrap_err().to_string();
        assert!(err.contains("LUMEN_TLS_CERT"), "{err}");
        clear_env();
    }

    #[test]
    fn projected_material_with_the_switch_off_fails_instead_of_serving_cleartext() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        clear_env();
        unsafe {
            std::env::set_var("LUMEN_TLS_CERT", "/var/run/secrets/lumen-serving/tls.crt");
        }
        let err = ServingTlsConfig::from_env().unwrap_err().to_string();
        assert!(
            err.contains("cleartext"),
            "the error must name what would otherwise happen: {err}"
        );
        clear_env();
    }

    /// R3, read from the type: the serving configuration has no mutual half to
    /// enable. Callers prove themselves with a ServiceAccount token; a client
    /// certificate on this port would be a second, unrelated answer to a
    /// question the token already answers.
    #[test]
    fn the_serving_profile_never_requires_a_client_certificate() {
        let profile = peer_tls::TlsRuntimeProfile::serving(["search.acme.svc".to_string()]);
        assert!(!profile.mutual);
        assert_eq!(
            profile.alpn_protocols,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
            "the client port offers both HTTP/2 and HTTP/1.1"
        );
        assert!(peer_tls::TlsRuntimeProfile::peer(["lumen-0".to_string()], std::iter::empty()).mutual);
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
    fn reloadable_refuses_to_start_on_material_that_is_no_longer_valid() {
        // The fixture leaf's validity window closed in July 2026, so this is
        // exactly the shape of a projected Secret nobody renewed. Startup must
        // refuse it (#3112 R7) rather than join the group with an identity
        // peers would reject — and the refusal must come from the shared seam,
        // which is the point of the adapter being three lines long.
        let cfg = write_tls_fixture("reloadable");
        let err = cfg
            .reloadable(["lumen-peer".to_string()], std::iter::empty())
            .expect_err("expired material must not activate");
        let message = err.to_string();
        assert!(
            message.contains("expired"),
            "the refusal should name the reason: {message}"
        );
        assert!(
            !message.contains("PRIVATE KEY") && !message.contains("cert.pem"),
            "a refusal must not carry key material or projection paths: {message}"
        );
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
            use std::io::Write;
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
        assert_eq!(cfg.peer_transport().unwrap().generation(), 1);
        std::fs::remove_dir_all(cfg.cert.parent().unwrap()).ok();
    }

    // ---- #2890 R5: the peer listener, exercised rather than described ------
    //
    // The four tests below dial the real `PeerTransport::serve` /
    // `connect` / `accept` seams the serving binary hands the Raft router to.
    // The negative ones assert on a router that records whether it was ever
    // reached, because "the connection was refused eventually" and "the
    // replication endpoint never saw the frame" are different facts, and only
    // the second one is the security property.

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use rcgen::{
        BasicConstraints, Certificate, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa,
        KeyPair,
    };

    /// A throwaway certificate authority. Two of these is what "a different
    /// trust domain" means in these tests.
    struct Authority {
        cert: Certificate,
        key: KeyPair,
    }

    fn authority(name: &str) -> Authority {
        let mut params = CertificateParams::new(Vec::new()).unwrap();
        params.distinguished_name.push(DnType::CommonName, name);
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let key = KeyPair::generate().unwrap();
        let cert = params.self_signed(&key).unwrap();
        Authority { cert, key }
    }

    /// Peer material laid out exactly as `spec.peerTlsSecret` projects it —
    /// `tls.crt`, `tls.key`, `ca.crt` — so what these tests load is what a pod
    /// mounts.
    struct Material {
        _dir: tempfile::TempDir,
        config: PeerTlsConfig,
    }

    /// `identity_ca` signs the leaf; `trust_ca` is who the holder verifies its
    /// counterpart against. Separate arguments on purpose: an attacker holds a
    /// certificate nobody trusts *while* trusting the real CA, which is the
    /// only configuration in which the client-certificate direction is the one
    /// under test.
    fn material(identity_ca: &Authority, trust_ca: &Authority, dns: &[&str]) -> Material {
        let mut params =
            CertificateParams::new(dns.iter().map(|n| (*n).to_string()).collect::<Vec<_>>())
                .unwrap();
        params.distinguished_name.push(DnType::CommonName, dns[0]);
        params.extended_key_usages = vec![
            ExtendedKeyUsagePurpose::ServerAuth,
            ExtendedKeyUsagePurpose::ClientAuth,
        ];
        let key = KeyPair::generate().unwrap();
        let cert = params
            .signed_by(&key, &identity_ca.cert, &identity_ca.key)
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("tls.crt"), cert.pem()).unwrap();
        std::fs::write(dir.path().join("tls.key"), key.serialize_pem()).unwrap();
        std::fs::write(dir.path().join("ca.crt"), trust_ca.cert.pem()).unwrap();
        Material {
            config: PeerTlsConfig {
                cert: dir.path().join("tls.crt"),
                key: dir.path().join("tls.key"),
                ca: dir.path().join("ca.crt"),
                required: true,
            },
            _dir: dir,
        }
    }

    /// A running peer listener plus the one bit that matters: did anything get
    /// past the handshake and reach the router?
    struct PeerListener {
        port: u16,
        router_reached: Arc<AtomicBool>,
        shutdown: tokio::sync::oneshot::Sender<()>,
        serve: tokio::task::JoinHandle<anyhow::Result<()>>,
    }

    async fn peer_listener(config: &PeerTlsConfig) -> PeerListener {
        let router_reached = Arc::new(AtomicBool::new(false));
        let flag = router_reached.clone();
        // Stands in for the Raft router: same position in `serve`, and it can
        // testify about itself.
        let router = axum::Router::new().route(
            "/raft/append",
            axum::routing::post(move || {
                let flag = flag.clone();
                async move {
                    flag.store(true, Ordering::SeqCst);
                    "appended"
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let transport = config.peer_transport().unwrap();
        let (shutdown, rx) = tokio::sync::oneshot::channel();
        let serve = tokio::spawn(async move {
            transport
                .serve(listener, router, async move {
                    let _ = rx.await;
                })
                .await
        });
        PeerListener {
            port,
            router_reached,
            shutdown,
            serve,
        }
    }

    impl PeerListener {
        fn reached(&self) -> bool {
            self.router_reached.load(Ordering::SeqCst)
        }

        async fn stop(self) {
            let _ = self.shutdown.send(());
            self.serve.await.unwrap().unwrap();
        }
    }

    #[tokio::test]
    async fn trusted_peer_pair_exchanges_a_raft_request_over_mtls() {
        install_default_crypto_provider();
        let ca = authority("lumen peer test CA");
        let server = material(&ca, &ca, &["localhost"]);
        let peer = material(&ca, &ca, &["localhost"]);
        let listener = peer_listener(&server.config).await;

        let response = peer
            .config
            .peer_transport()
            .unwrap()
            .http_client()
            .post(format!("https://localhost:{}/raft/append", listener.port))
            .send()
            .await
            .expect("a peer holding instance-scoped material must complete the exchange");

        assert!(response.status().is_success(), "{:?}", response.status());
        assert!(
            listener.reached(),
            "the trusted exchange must actually reach the Raft router"
        );
        listener.stop().await;
    }

    #[tokio::test]
    async fn plaintext_peer_dial_never_reaches_the_raft_router() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        install_default_crypto_provider();
        let ca = authority("lumen peer test CA");
        let server = material(&ca, &ca, &["localhost"]);
        let listener = peer_listener(&server.config).await;

        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", listener.port))
            .await
            .unwrap();
        // The HTTP/2 preface with no TLS underneath — exactly what the removed
        // h2c fallback used to speak on this port (#2890 R3).
        stream
            .write_all(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n")
            .await
            .unwrap();
        let mut buf = Vec::new();
        let read = tokio::time::timeout(Duration::from_secs(5), stream.read_buf(&mut buf))
            .await
            .expect("a plaintext dial must be terminated, not left hanging");

        assert!(
            !buf.windows(8).any(|w| w == b"appended"),
            "plaintext peer got a router response back: {read:?} {buf:?}"
        );
        assert!(
            !listener.reached(),
            "a plaintext dial must die in the handshake, before the Raft router"
        );
        listener.stop().await;
    }

    #[tokio::test]
    async fn unrelated_ca_peer_certificate_never_reaches_the_raft_router() {
        install_default_crypto_provider();
        let lumen_ca = authority("lumen peer test CA");
        let unrelated_ca = authority("unrelated CA");
        let server = material(&lumen_ca, &lumen_ca, &["localhost"]);
        // The impostor trusts Lumen's real CA, so the server's identity checks
        // out from its side; the only direction left to fail is the client
        // certificate it presents.
        let impostor = material(&unrelated_ca, &lumen_ca, &["localhost"]);
        let listener = peer_listener(&server.config).await;

        let error = impostor
            .config
            .peer_transport()
            .unwrap()
            .http_client()
            .post(format!("https://localhost:{}/raft/append", listener.port))
            .send()
            .await
            .expect_err("a certificate from another trust domain must be rejected");

        assert!(
            !listener.reached(),
            "an untrusted client certificate must be rejected before the Raft \
             router sees a message: {error}"
        );
        listener.stop().await;
    }

    #[tokio::test]
    async fn peer_client_rejects_a_trusted_certificate_for_the_wrong_dns_name() {
        install_default_crypto_provider();
        let ca = authority("lumen peer test CA");
        // Chain-valid — signed by the instance's own CA. What is wrong is who
        // it claims to be: a member of a different headless Service. Verifying
        // the chain alone would accept it.
        let impostor = material(
            &ca,
            &ca,
            &["lumen-0.other-headless.lumen.svc.cluster.local"],
        );
        let dialer = material(
            &ca,
            &ca,
            &["lumen-1.lumen-headless.lumen.svc.cluster.local"],
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server_transport = impostor.config.peer_transport().unwrap();
        let client_transport = dialer.config.peer_transport().unwrap();

        let (_server, client_result) = tokio::time::timeout(Duration::from_secs(5), async {
            tokio::join!(
                async {
                    let (stream, _) = listener.accept().await.unwrap();
                    server_transport.accept(stream).await
                },
                async {
                    let stream = tokio::net::TcpStream::connect(address).await.unwrap();
                    client_transport
                        .connect(stream, "lumen-0.lumen-headless.lumen.svc.cluster.local")
                        .await
                }
            )
        })
        .await
        .expect("the wrong-identity handshake must terminate");

        let error =
            client_result.expect_err("a trusted certificate for another DNS name must not pass");
        assert!(
            error.to_string().contains("peer TLS client handshake"),
            "unexpected rejection: {error:#}"
        );
    }
}
// CODEGEN-END
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/lumen/src/tls.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      Thinned to an adapter (#971): PEM cert/key/CA loading, the rustls
      server/client config builders, and the Once-guarded crypto-provider
      install moved verbatim to the new shared libs/peer-tls crate
      (parameterized by an env prefix instead of lumen's hardcoded env var
      names). This file keeps the LUMEN_PEER_TLS_*/LUMEN_PEER_MTLS env
      contract and the lumen-facing PeerTlsConfig struct, delegating to
      peer_tls::PeerTlsConfig via From conversions so the pub API (struct
      fields, from_env/rustls_server_config/rustls_client_config,
      install_default_crypto_provider) is byte-compatible with the
      pre-extraction shape.
  - path: libs/peer-tls/src/lib.rs
    action: create
    section: rust-source-unit
    impl_mode: hand-written
    description: |
      New shared crate (#971), lifted verbatim from lumen's tls.rs: PEM
      cert/key/CA-bundle loaders, the rustls server/client config builders
      (honoring the mTLS-required client-cert-verifier branch), and the
      Once-guarded install_default_crypto_provider. PeerTlsConfig::from_env
      now takes a caller-supplied env prefix (lumen passes "LUMEN_PEER") and
      derives <prefix>_TLS_CERT/_TLS_KEY/_TLS_CA/_MTLS from it, so the
      env-key and error-text shape is unchanged for lumen while the crate
      itself is service-agnostic. keep/relay/beam adoption is out of scope.
```
