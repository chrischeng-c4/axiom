// SPEC-MANAGED: .aw/tech-design/projects/httpkit/health.md#tests
// CODEGEN-BEGIN

use mambalibs_http::health::{HealthCheck, HealthStatus};
use std::str::FromStr;

#[test]
fn status_serializes_to_canonical_string() {
    let s = HealthStatus::Degraded;
    assert_eq!(s.as_str(), "degraded");
}

#[test]
fn status_round_trips_via_from_str() {
    let parsed: HealthStatus = FromStr::from_str("unhealthy").unwrap();
    assert_eq!(parsed, HealthStatus::Unhealthy);
    assert_eq!(parsed.as_str(), "unhealthy");
}

#[test]
fn status_from_str_rejects_unknown() {
    let result = HealthStatus::from_str("spicy");
    assert!(result.is_err());
}

#[test]
fn check_preserves_description_when_provided() {
    let c = HealthCheck::new(
        "db".to_string(),
        HealthStatus::Healthy,
        Some("primary replica online".to_string()),
    )
    .unwrap();
    assert_eq!(c.name, "db");
    assert_eq!(c.status, HealthStatus::Healthy);
    assert_eq!(c.description.as_deref(), Some("primary replica online"));
}

#[test]
fn check_accepts_omitted_description() {
    let c = HealthCheck::new("cache".to_string(), HealthStatus::Degraded, None).unwrap();
    assert!(c.description.is_none());
    assert_eq!(c.status, HealthStatus::Degraded);
}
// CODEGEN-END
