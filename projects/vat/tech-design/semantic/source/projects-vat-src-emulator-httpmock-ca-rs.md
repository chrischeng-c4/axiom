---
id: projects-vat-src-emulator-httpmock-ca-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/emulator/httpmock/ca.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/emulator/httpmock/ca.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/emulator/httpmock/ca.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! CA + per-host leaf certificates for the HTTP mock proxy's HTTPS MITM.
//!
//! On startup the proxy mints a throwaway CA (rcgen), writes its cert PEM to a
//! path vat exports as the runner's CA-trust bundle, and mints per-host leaf
//! certs signed by that CA on demand (cached) so it can terminate TLS for any
//! `https://host` the runner connects to. Nothing here panics on bad input.
//!
//! @spec projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#logic

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rcgen::{Certificate, CertificateParams, DnType, KeyPair};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;

/// Mints and caches per-host server TLS configs signed by a single CA.
pub struct CaStore {
    ca_cert: Certificate,
    ca_key: KeyPair,
    ca_der: CertificateDer<'static>,
    ca_pem: String,
    leaves: Mutex<HashMap<String, Arc<ServerConfig>>>,
}

impl CaStore {
    /// Generate a fresh CA in memory.
    pub fn generate() -> Result<Self> {
        let mut params = CertificateParams::new(Vec::new()).context("ca params")?;
        params
            .distinguished_name
            .push(DnType::CommonName, "vat http-mock CA");
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let ca_key = KeyPair::generate().context("ca keypair")?;
        let ca_cert = params.self_signed(&ca_key).context("self-sign ca")?;
        let ca_pem = ca_cert.pem();
        let ca_der = ca_cert.der().clone();
        Ok(Self {
            ca_cert,
            ca_key,
            ca_der,
            ca_pem,
            leaves: Mutex::new(HashMap::new()),
        })
    }

    /// The CA certificate as PEM (written to disk + exported as the trust bundle).
    pub fn ca_pem(&self) -> &str {
        &self.ca_pem
    }

    /// A rustls server config presenting a leaf cert for `host`, signed by the
    /// CA and cached.
    pub fn server_config(&self, host: &str) -> Result<Arc<ServerConfig>> {
        if let Some(cfg) = self.leaves.lock().unwrap().get(host).cloned() {
            return Ok(cfg);
        }
        let mut params = CertificateParams::new(vec![host.to_string()]).context("leaf params")?;
        params.distinguished_name.push(DnType::CommonName, host);
        let leaf_key = KeyPair::generate().context("leaf keypair")?;
        let leaf = params
            .signed_by(&leaf_key, &self.ca_cert, &self.ca_key)
            .context("sign leaf")?;

        let chain = vec![leaf.der().clone(), self.ca_der.clone()];
        let key = PrivateKeyDer::try_from(leaf_key.serialize_der())
            .map_err(|e| anyhow::anyhow!("leaf key der: {e}"))?;
        // Build with an explicit crypto provider so we don't depend on a
        // process-global default being installed (rustls 0.23 requires one).
        let provider = Arc::new(rustls::crypto::aws_lc_rs::default_provider());
        let mut cfg = ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .context("rustls protocol versions")?
            .with_no_client_auth()
            .with_single_cert(chain, key)
            .context("rustls server config")?;
        // Advertise h2 (gRPC) + http/1.1 so the MITM can serve gRPC over the
        // decrypted tunnel (network sandbox v2); HTTP/1 clients still get h1.
        cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
        let cfg = Arc::new(cfg);
        self.leaves
            .lock()
            .unwrap()
            .insert(host.to_string(), cfg.clone());
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mints_ca_and_trusted_leaf() {
        let ca = CaStore::generate().unwrap();
        assert!(ca.ca_pem().contains("BEGIN CERTIFICATE"));
        // A leaf for a host builds a usable rustls server config (parses cert+key).
        let cfg = ca.server_config("api.test").unwrap();
        assert!(Arc::strong_count(&cfg) >= 1);
        // Cached: same Arc on second call.
        let cfg2 = ca.server_config("api.test").unwrap();
        assert!(Arc::ptr_eq(&cfg, &cfg2));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/emulator/httpmock/ca.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/emulator/httpmock/ca.rs` captured during #39 vat standardization.
```
