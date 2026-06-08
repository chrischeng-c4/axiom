// SPEC-MANAGED: .score/tech_design/projects/httpkit/health.md#schema
// CODEGEN-BEGIN
/// Aggregate health of a component. Ordered most- to least-healthy.
/// @spec .score/tech_design/projects/httpkit/health.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

/// @spec .score/tech_design/projects/httpkit/health.md#schema.as_str
impl HealthStatus {
    /// Canonical string representation (matches serde rename values).
    pub fn as_str(self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
        }
    }
}

/// @spec .score/tech_design/projects/httpkit/health.md#schema.from_str
impl std::str::FromStr for HealthStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "healthy" => Ok(HealthStatus::Healthy),
            "degraded" => Ok(HealthStatus::Degraded),
            "unhealthy" => Ok(HealthStatus::Unhealthy),
            _ => Err(format!("invalid HealthStatus: {}", s)),
        }
    }
}

use serde::{Deserialize, Serialize};

/// @spec .score/tech_design/projects/httpkit/health.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Identifier for the check (e.g. "db", "cache").
    pub name: String,
    pub status: HealthStatus,
    /// Free-form detail shown alongside the status.
    #[serde(default)]
    pub description: Option<String>,
}

/// @spec .score/tech_design/projects/httpkit/health.md#x-constructor
impl HealthCheck {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(
        name: String,
        status: crate::health::HealthStatus,
        description: Option<String>,
    ) -> Result<Self, String> {
        Ok(Self {
            name,
            status,
            description,
        })
    }
}

/// @spec .score/tech_design/projects/httpkit/health.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthManager {
    /// Registered checks, aggregated by `aggregate_status()` (lowest-health wins).
    pub checks: Vec<HealthCheck>,
}

/// @spec .score/tech_design/projects/httpkit/health.md#x-constructor
impl HealthManager {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(checks: Vec<crate::health::HealthCheck>) -> Result<Self, String> {
        Ok(Self { checks })
    }
}
// CODEGEN-END
