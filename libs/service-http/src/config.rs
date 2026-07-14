// SPEC-MANAGED: libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! `HttpConfig` — the env-driven runtime knobs every k8s-native service shares.
//!
//! Each service binary already parses these (under its own `SERVICE_*`
//! prefix via clap `env =`); this is the resolved, prefix-agnostic struct the
//! shared scaffolding (`logging::init_tracing`, `signal::shutdown_with_drain`,
//! the body-limit on the data-plane router) reads, so the common shape lives in
//! one place instead of being threaded through four hand-rolled `serve()` fns.

/// Log output format for the fmt layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub enum LogFormat {
    /// Human/agent-readable multi-line output (local dev default).
    Pretty,
    /// One JSON object per line (structured log shipping in-cluster).
    Json,
}

/// Stable resource identity attached to optional shared trace export.
#[derive(Clone, Debug, PartialEq, Eq)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub struct ServiceIdentity {
    name: String,
    version: String,
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
impl ServiceIdentity {
    /// Create an identity for the `service.name` and `service.version`
    /// resource attributes. Empty fields are rejected before startup.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> anyhow::Result<Self> {
        let name = name.into();
        let version = version.into();
        if name.trim().is_empty() {
            anyhow::bail!("service tracing identity name must not be empty");
        }
        if version.trim().is_empty() {
            anyhow::bail!("service tracing identity version must not be empty");
        }
        Ok(Self { name, version })
    }

    /// Stable service name supplied by the owning application.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Stable service version supplied by the owning application.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Resolved HTTP-service configuration.
///
/// Built by a service binary from its flags/env and handed to the shared
/// scaffolding. Construct with [`HttpConfig::new`].
#[derive(Clone, Debug)]
/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
pub struct HttpConfig {
    /// Bind host. k8s passes `0.0.0.0`; local dev defaults to `127.0.0.1`.
    pub host: String,
    /// Bind port.
    pub port: u16,
    /// Base log level (`trace|debug|info|warn|error`). `RUST_LOG` still wins.
    pub log_level: String,
    /// Log output format.
    pub log_format: LogFormat,
    /// Graceful-drain window (seconds) held after SIGTERM before the listener
    /// closes, so k8s stops routing while `/readyz` reports 503.
    pub grace_secs: u64,
    /// Max request body size (bytes) for the data plane. The probe routes carry
    /// no body limit regardless.
    pub body_limit_bytes: usize,
    /// OTLP gRPC endpoint for trace export, e.g. `http://otel-collector:4317`.
    /// Opt-in: when `None`, no OTLP wiring is attempted. See
    /// [`crate::logging::init_tracing`] for the current stub status.
    pub otlp_endpoint: Option<String>,
}

/// @spec libs/service-http/tech-design/semantic/source/libs-service-http-src-config-rs.md#source
impl HttpConfig {
    /// Construct a config from already-resolved values. Every field is explicit
    /// so a service binary maps its own flags/env in one place.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        host: impl Into<String>,
        port: u16,
        log_level: impl Into<String>,
        log_format: LogFormat,
        grace_secs: u64,
        body_limit_bytes: usize,
        otlp_endpoint: Option<String>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            log_level: log_level.into(),
            log_format,
            grace_secs,
            body_limit_bytes,
            otlp_endpoint,
        }
    }

    /// `host:port` bind string for `TcpListener::bind`.
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_populates_every_field() {
        let cfg = HttpConfig::new(
            "0.0.0.0",
            7373,
            "debug",
            LogFormat::Json,
            45,
            8 * 1024 * 1024,
            Some("http://otel:4317".to_string()),
        );
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 7373);
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.log_format, LogFormat::Json);
        assert_eq!(cfg.grace_secs, 45);
        assert_eq!(cfg.body_limit_bytes, 8 * 1024 * 1024);
        assert_eq!(cfg.otlp_endpoint.as_deref(), Some("http://otel:4317"));
        assert_eq!(cfg.bind_addr(), "0.0.0.0:7373");
    }

    #[test]
    fn service_identity_rejects_blank_fields() {
        assert!(ServiceIdentity::new("", "0.1.0").is_err());
        assert!(ServiceIdentity::new("service", " ").is_err());
        assert_eq!(
            ServiceIdentity::new("service", "0.1.0").unwrap(),
            ServiceIdentity {
                name: "service".to_string(),
                version: "0.1.0".to_string(),
            }
        );
    }
}
// CODEGEN-END
