// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity levels for vulnerabilities.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Moderate,
    Low,
    Info,
}

/// A single vulnerability entry.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub package: String,
    pub severity: Severity,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vulnerable_versions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patched_versions: Option<String>,
    #[serde(default)]
    pub dependency_chain: Vec<String>,
}

/// Summary counts by severity.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditSummary {
    pub critical: usize,
    pub high: usize,
    pub moderate: usize,
    pub low: usize,
    pub total: usize,
}

/// Full audit report.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: AuditSummary,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl AuditReport {
    /// Whether the audit found critical or high severity issues.
    pub fn has_critical_or_high(&self) -> bool {
        self.summary.critical > 0 || self.summary.high > 0
    }
}

/// Audit client that checks packages against npm advisory API.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub struct AuditClient {
    client: reqwest::Client,
    registry_url: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl AuditClient {
    pub fn new(registry_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            registry_url: registry_url.trim_end_matches('/').to_string(),
        }
    }

    /// Run audit against the npm advisory API.
    pub async fn audit(&self, packages: &HashMap<String, String>) -> Result<AuditReport> {
        // Build the payload: { "name": { "version": "x.y.z" }, ... }
        let mut payload = serde_json::Map::new();
        for (name, version) in packages {
            let mut entry = serde_json::Map::new();
            entry.insert(
                "version".to_string(),
                serde_json::Value::String(version.clone()),
            );
            payload.insert(name.clone(), serde_json::Value::Object(entry));
        }

        let body = serde_json::json!({
            "name": "jet-audit",
            "version": "1.0.0",
            "requires": payload,
            "dependencies": payload,
        });

        let url = format!("{}/-/npm/v1/security/audits", self.registry_url);

        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            // Advisory API may not be available on all registries
            tracing::warn!(
                "Audit API returned {}, returning empty report",
                response.status()
            );
            return Ok(AuditReport {
                vulnerabilities: Vec::new(),
                summary: AuditSummary::default(),
            });
        }

        let resp_body: serde_json::Value = response.json().await?;
        Self::parse_response(&resp_body)
    }

    /// Parse the npm advisory API response into our AuditReport.
    fn parse_response(resp: &serde_json::Value) -> Result<AuditReport> {
        let mut vulnerabilities = Vec::new();

        if let Some(advisories) = resp.get("advisories").and_then(|a| a.as_object()) {
            for (id, advisory) in advisories {
                let severity = match advisory.get("severity").and_then(|s| s.as_str()) {
                    Some("critical") => Severity::Critical,
                    Some("high") => Severity::High,
                    Some("moderate") => Severity::Moderate,
                    Some("low") => Severity::Low,
                    _ => Severity::Info,
                };

                // GH #3584 — `module_name` is the actionable identifier
                // for an advisory. The prior `.unwrap_or("unknown")`
                // silently collapsed every malformed advisory onto the
                // same "unknown" package, producing an unactionable
                // report. Refuse to parse the response when an
                // advisory's `module_name` is missing or non-string,
                // naming the offending advisory id, the field, and the
                // observed JSON kind.
                let package = require_audit_string_field(advisory, "module_name", id)?;

                // `title` is informational, not actionable. Keep the
                // fall-through but warn on non-string-non-null kinds
                // so a noisy registry surfaces in the operator's logs.
                let title = optional_audit_string_field(advisory, "title", id)
                    .unwrap_or_else(|| "Unknown vulnerability".to_string());

                vulnerabilities.push(Vulnerability {
                    package,
                    severity,
                    title: advisory
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown vulnerability")
                        .to_string(),
                    url: advisory
                        .get("url")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    vulnerable_versions: advisory
                        .get("vulnerable_versions")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    patched_versions: advisory
                        .get("patched_versions")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    dependency_chain: Vec::new(),
                });
            }
        }

        let summary = AuditSummary {
            critical: vulnerabilities
                .iter()
                .filter(|v| v.severity == Severity::Critical)
                .count(),
            high: vulnerabilities
                .iter()
                .filter(|v| v.severity == Severity::High)
                .count(),
            moderate: vulnerabilities
                .iter()
                .filter(|v| v.severity == Severity::Moderate)
                .count(),
            low: vulnerabilities
                .iter()
                .filter(|v| v.severity == Severity::Low)
                .count(),
            total: vulnerabilities.len(),
        };

        Ok(AuditReport {
            vulnerabilities,
            summary,
        })
    }
}

/// GH #3584 — require an audit advisory's actionable identifier field
/// (`module_name`) to be a non-empty string. Returns the value on
/// success; on failure returns an `anyhow::Error` whose Display names
/// the offending advisory id, the field, and the observed JSON kind.
///
/// Replaces the prior `pkg["module_name"].as_str().unwrap_or("unknown")`
/// silent fallback, which collapsed every malformed advisory onto the
/// same "unknown" package name and produced an unactionable audit
/// report.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn require_audit_string_field(
    advisory: &serde_json::Value,
    field: &str,
    advisory_id: &str,
) -> Result<String> {
    let observed_kind = match advisory.get(field) {
        Some(serde_json::Value::String(s)) if s.is_empty() => "empty-string",
        Some(serde_json::Value::String(s)) => return Ok(s.clone()),
        Some(other) => describe_audit_field_kind(other),
        None => "missing",
    };
    Err(anyhow::anyhow!(
        "{}",
        format_audit_field_err(advisory_id, field, observed_kind)
    ))
}

/// GH #3584 — read an audit advisory's informational string field
/// (`title`, …). Returns `Some` on a valid non-empty string, `None`
/// on missing/null. On non-string-non-null kinds, emits a
/// `tracing::warn!` tagged `GH #3584` so a noisy registry surfaces in
/// the operator's logs, then returns `None` so the caller can fall
/// through to its default.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn optional_audit_string_field(
    advisory: &serde_json::Value,
    field: &str,
    advisory_id: &str,
) -> Option<String> {
    match advisory.get(field) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(serde_json::Value::String(_)) => None, // empty string → fall through
        None | Some(serde_json::Value::Null) => None,
        Some(other) => {
            let kind = describe_audit_field_kind(other);
            tracing::warn!(
                target: "jet::pkg_manager::audit",
                advisory_id = %advisory_id,
                field = %field,
                observed_kind = %kind,
                "{}",
                format_audit_field_err(advisory_id, field, kind)
            );
            None
        }
    }
}

/// GH #3584 — build the error/warn message for an audit advisory
/// field that is missing, non-string, or empty. Extracted so the
/// wording (advisory id + field + kind + tag) is unit-testable
/// without provoking the actual registry response case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn format_audit_field_err(
    advisory_id: &str,
    field: &str,
    observed_kind: &str,
) -> String {
    format!(
        "GH #3584 audit advisory {advisory_id} has no non-empty string `{field}` \
         field (observed: {observed_kind}); refusing to produce a report \
         whose entries collapse onto the same placeholder identifier \
         (which would render the vulnerability list unactionable)"
    )
}

/// GH #3584 — describe the JSON kind of a value for the audit field
/// error/warn message so the dev can tell from the message what was
/// actually observed at the field slot.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn describe_audit_field_kind(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "bool",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_report() {
        let report = AuditReport {
            vulnerabilities: Vec::new(),
            summary: AuditSummary::default(),
        };
        assert!(!report.has_critical_or_high());
    }

    #[test]
    fn test_critical_report() {
        let report = AuditReport {
            vulnerabilities: vec![Vulnerability {
                package: "evil-pkg".to_string(),
                severity: Severity::Critical,
                title: "RCE".to_string(),
                url: None,
                vulnerable_versions: Some("<1.0.0".to_string()),
                patched_versions: Some(">=1.0.0".to_string()),
                dependency_chain: vec!["root".to_string(), "evil-pkg".to_string()],
            }],
            summary: AuditSummary {
                critical: 1,
                total: 1,
                ..Default::default()
            },
        };
        assert!(report.has_critical_or_high());
    }

    #[test]
    fn test_parse_empty_response() {
        let resp = serde_json::json!({});
        let report = AuditClient::parse_response(&resp).unwrap();
        assert_eq!(report.vulnerabilities.len(), 0);
    }

    // ─── GH #3584: silent "unknown" fallback for missing module_name ───

    /// GH #3584 — `format_audit_field_err` must include the issue
    /// tag, the offending advisory id, the field name, and the
    /// observed JSON kind across all 7 kinds so the cause is
    /// greppable.
    #[test]
    fn gh3584_format_audit_field_err_names_tag_id_field_and_kind() {
        for kind in [
            "missing",
            "null",
            "number",
            "bool",
            "array",
            "object",
            "empty-string",
        ] {
            let msg = format_audit_field_err("ADV-42", "module_name", kind);
            assert!(
                msg.contains("GH #3584"),
                "must include tag (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains("ADV-42"),
                "must name the advisory id (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains("module_name"),
                "must name the field (kind={kind}), got: {msg}"
            );
            assert!(
                msg.contains(kind),
                "must name the kind (kind={kind}), got: {msg}"
            );
        }
    }

    /// GH #3584 — `describe_audit_field_kind` must distinguish all
    /// JSON shapes so the audit field error is precise about what
    /// was observed at the slot.
    #[test]
    fn gh3584_describe_audit_field_kind_distinguishes_json_shapes() {
        assert_eq!(describe_audit_field_kind(&serde_json::Value::Null), "null");
        assert_eq!(
            describe_audit_field_kind(&serde_json::Value::Bool(true)),
            "bool"
        );
        assert_eq!(describe_audit_field_kind(&serde_json::json!(42)), "number");
        assert_eq!(describe_audit_field_kind(&serde_json::json!("v")), "string");
        assert_eq!(describe_audit_field_kind(&serde_json::json!([])), "array");
        assert_eq!(describe_audit_field_kind(&serde_json::json!({})), "object");
    }

    /// GH #3584 — `parse_response` on an advisory with a missing
    /// `module_name` must surface as `Err` whose Display contains
    /// the GH #3584 tag, the advisory id, the field, and the
    /// observed kind `missing` — NOT a silent "unknown" placeholder.
    #[test]
    fn gh3584_parse_response_rejects_missing_module_name() {
        let resp = serde_json::json!({
            "advisories": {
                "1337": {
                    "severity": "critical",
                    "title": "CVE-anything"
                    // no module_name
                }
            }
        });
        let err = AuditClient::parse_response(&resp)
            .expect_err("missing module_name must surface as Err");
        let chain = format!("{err:#}");
        assert!(chain.contains("GH #3584"), "must include tag, got: {chain}");
        assert!(
            chain.contains("1337"),
            "must name the advisory id, got: {chain}"
        );
        assert!(
            chain.contains("module_name"),
            "must name the field, got: {chain}"
        );
        assert!(
            chain.contains("missing"),
            "must name the kind, got: {chain}"
        );
        assert!(
            !chain.contains("unknown"),
            "must NOT silently fall back to 'unknown', got: {chain}"
        );
    }

    /// GH #3584 — happy path: a well-formed advisory continues to
    /// parse correctly. Pins that the new error path does not
    /// regress the common case.
    #[test]
    fn gh3584_parse_response_happy_path_still_parses() {
        let resp = serde_json::json!({
            "advisories": {
                "42": {
                    "module_name": "evil-pkg",
                    "severity": "high",
                    "title": "RCE",
                    "url": "https://example.com",
                    "vulnerable_versions": "<1.0.0",
                    "patched_versions": ">=1.0.0",
                }
            }
        });
        let report = AuditClient::parse_response(&resp).unwrap();
        assert_eq!(report.vulnerabilities.len(), 1);
        assert_eq!(report.vulnerabilities[0].package, "evil-pkg");
        assert_eq!(report.vulnerabilities[0].title, "RCE");
        assert_eq!(report.vulnerabilities[0].severity, Severity::High);
        assert_eq!(report.summary.high, 1);
    }

    /// GH #3584 — informational fields (`title`) with a non-string-
    /// non-null kind fall through to the default but emit a
    /// `tracing::warn!`. We don't assert the warn here (no global
    /// subscriber in unit tests); we pin that parsing still
    /// succeeds and the package field (the required identifier) is
    /// still correctly captured.
    #[test]
    fn gh3584_parse_response_title_non_string_falls_through_to_default() {
        let resp = serde_json::json!({
            "advisories": {
                "42": {
                    "module_name": "evil-pkg",
                    "severity": "high",
                    "title": 42  // non-string title
                }
            }
        });
        let report = AuditClient::parse_response(&resp).unwrap();
        assert_eq!(report.vulnerabilities.len(), 1);
        assert_eq!(report.vulnerabilities[0].package, "evil-pkg");
        assert_eq!(report.vulnerabilities[0].title, "Unknown vulnerability");
    }
}
// CODEGEN-END
