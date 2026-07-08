// HANDWRITE-BEGIN gap="missing-generator:logic:32cbd2eb" tracker="pending-tracker" reason="install_default_crypto_provider(): delegate to service_tls::install_default_crypto_provider — the shared Once-guarded aws-lc-rs install. Unconditional since #1209: service-tls (peer-mTLS material loading) links rustls into every relay build, so the former private rustls-provider feature indirection is gone. Called at the very top of main before clap parsing (keep's pattern — kube, raft-host peer transport and the online CLI ops all link rustls)."
//! Process-level rustls crypto provider install.
//!
//! Several relay build paths link rustls: `libs/service-tls` (peer-mTLS
//! material loading, in every build since #1209), the k8s operator (kube-rs →
//! hyper + rustls), the raft-host peer transport (via reqwest), and the online
//! CLI ops (`upgrade` / `issue`). rustls 0.23 refuses to pick a default crypto
//! provider when more than one is linked in the process, so any binary that
//! reaches a TLS path must install one explicitly — before the first
//! `ClientConfig` / `ServerConfig` is built — or it panics at runtime. `relay`
//! calls this once at the very top of `main`, before command parsing, so every
//! path (serve, operator, upgrade, issue, backup, spec) is covered.
//!
//! The install itself is the shared `service_tls::install_default_crypto_provider`
//! (`Once`-guarded, ignores a provider a dependency installed first). The
//! former private `rustls-provider` feature gate is gone: service-tls links
//! rustls unconditionally, so a conditional install would only leave a
//! panic-shaped hole in the default build.

/// Install the aws-lc-rs rustls crypto provider as the process default, once.
///
/// Idempotent: a second call — or a provider a dependency installed first — is
/// ignored (we only need *a* default present). Delegates to the shared
/// `libs/service-tls` install (#1209).
pub fn install_default_crypto_provider() {
    service_tls::install_default_crypto_provider();
}
// HANDWRITE-END
