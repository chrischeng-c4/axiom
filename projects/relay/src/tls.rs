// HANDWRITE-BEGIN gap="missing-generator:logic:32cbd2eb" tracker="pending-tracker" reason="install_default_crypto_provider(): behind the private rustls-provider feature install the aws-lc-rs rustls default provider once (std::sync::Once, ignore a pre-installed provider); a no-op otherwise. Called at the very top of main before clap parsing (keep's pattern — kube, raft-host peer transport and the online CLI ops all link rustls)."
//! Process-level rustls crypto provider install.
//!
//! Several relay build paths link rustls: the k8s operator (kube-rs → hyper +
//! rustls), the raft-host peer transport (via reqwest), and the online CLI ops
//! (`upgrade` / `issue`). rustls 0.23 refuses to pick a default crypto provider
//! when more than one is linked in the process, so any binary that reaches a
//! TLS path must install one explicitly — before the first `ClientConfig` /
//! `ServerConfig` is built — or it panics at runtime. `relay` calls this once
//! at the very top of `main`, before command parsing, so every path (serve,
//! operator, upgrade, issue) is covered.
//!
//! Builds that enable no rustls-linking feature compile this as a no-op, so
//! the fast dev/test cycle pays nothing (keep's pattern).

/// Install the aws-lc-rs rustls crypto provider as the process default, once.
///
/// Idempotent: a second call — or a provider a dependency installed first — is
/// ignored (we only need *a* default present). A no-op in builds without a
/// rustls path (the private `rustls-provider` feature is off).
pub fn install_default_crypto_provider() {
    #[cfg(feature = "rustls-provider")]
    {
        use std::sync::Once;
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        });
    }
}
// HANDWRITE-END
