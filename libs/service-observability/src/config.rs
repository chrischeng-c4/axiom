// HANDWRITE-BEGIN gap="missing-generator:logic:16181194" tracker="pending-tracker" reason="Own LogFormat, ObservabilityConfig, and ServiceIdentity."
//! Typed, protocol-neutral observability configuration.

/// Log output format for the shared formatter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogFormat {
    Pretty,
    Json,
}

/// Stable resource identity attached to exported telemetry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceIdentity {
    name: String,
    version: String,
}

impl ServiceIdentity {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Resolved observability settings independent of transport and deployment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservabilityConfig {
    pub log_level: String,
    pub log_format: LogFormat,
    pub otlp_endpoint: Option<String>,
}

impl ObservabilityConfig {
    pub fn new(
        log_level: impl Into<String>,
        log_format: LogFormat,
        otlp_endpoint: Option<String>,
    ) -> Self {
        Self {
            log_level: log_level.into(),
            log_format,
            otlp_endpoint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rejects_blank_fields() {
        assert!(ServiceIdentity::new("", "0.1.0").is_err());
        assert!(ServiceIdentity::new("service", " ").is_err());
        let identity = ServiceIdentity::new("service", "0.1.0").unwrap();
        assert_eq!(identity.name(), "service");
        assert_eq!(identity.version(), "0.1.0");
    }

    #[test]
    fn config_is_transport_neutral() {
        let config = ObservabilityConfig::new(
            "debug",
            LogFormat::Json,
            Some("http://otel:4317".to_string()),
        );
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.log_format, LogFormat::Json);
        assert_eq!(config.otlp_endpoint.as_deref(), Some("http://otel:4317"));
    }
}
// HANDWRITE-END
