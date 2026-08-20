// CODEGEN-BEGIN
//! HTTP-specific runtime knobs and observability-config projection.
//!
//! Each service binary already parses these (under its own `SERVICE_*`
//! prefix via clap `env =`); this is the resolved, prefix-agnostic struct the
//! shared HTTP scaffolding reads. Protocol-neutral logging/tracing types live
//! in `service-observability` and are re-exported here for compatibility.

pub use service_observability::{LogFormat, ServiceIdentity};

/// Resolved HTTP-service configuration.
///
/// Built by a service binary from its flags/env and handed to the shared
/// scaffolding. Construct with [`HttpConfig::new`].
#[derive(Clone, Debug)]
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
    /// Opt-in: when `None`, no OTLP wiring is attempted. With the `otlp`
    /// feature, `logging::init_tracing_with_identity` exports traces; invalid
    /// configuration safely retains structured logging.
    pub otlp_endpoint: Option<String>,
}

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

    /// Project only protocol-neutral fields into the shared observability
    /// owner. Bind, grace, and body-limit policy remain HTTP-specific.
    pub fn observability_config(&self) -> service_observability::ObservabilityConfig {
        service_observability::ObservabilityConfig::new(
            self.log_level.clone(),
            self.log_format,
            self.otlp_endpoint.clone(),
        )
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
        assert_eq!(
            cfg.observability_config(),
            service_observability::ObservabilityConfig::new(
                "debug",
                LogFormat::Json,
                Some("http://otel:4317".to_string()),
            )
        );
    }

    #[test]
    fn service_identity_rejects_blank_fields() {
        assert!(ServiceIdentity::new("", "0.1.0").is_err());
        assert!(ServiceIdentity::new("service", " ").is_err());
        let identity = ServiceIdentity::new("service", "0.1.0").unwrap();
        assert_eq!(identity.name(), "service");
        assert_eq!(identity.version(), "0.1.0");
    }
}
// CODEGEN-END
