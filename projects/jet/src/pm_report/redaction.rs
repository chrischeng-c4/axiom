// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Privacy redaction policy for shared PM e2e reports (#2732).
//!
//! Evidence bundles can leak sensitive data — request URLs with query
//! tokens, `Authorization` headers, response bodies with PII, env
//! metadata revealing internal hostnames. Once a bundle is packaged
//! via `report_package` and shared on a PR or static host, redaction
//! has to happen at packaging time, not at viewer time.
//!
//! This module owns the policy + the field-level redactors. It is
//! intentionally a *policy layer*: rule sets and string transforms.
//! Pixel-level screenshot redaction is out of scope (split into a
//! later issue). The packager calls into [`RedactionPolicy::apply`]
//! before writing files; [`RedactionReport`] feeds the report-metadata
//! flag the PM shell renders.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Stable schema tag for redaction metadata embedded in report
/// manifests.
pub const PM_REDACTION_SCHEMA_VERSION: &str = "jet.pm.redaction.v1";

/// Placeholder string used everywhere a sensitive value is replaced.
/// Constant so the PM shell can detect redacted-vs-original at a
/// glance.
pub const REDACTION_PLACEHOLDER: &str = "[REDACTED]";

/// Categories of evidence a redaction rule covers. The set is closed
/// here so the policy can't silently grow new redaction surfaces
/// without a schema bump.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RedactionScope {
    /// Query string + path tokens in recorded network URLs.
    NetworkUrl,
    /// HTTP request/response headers.
    NetworkHeader,
    /// Request/response bodies.
    NetworkBody,
    /// DOM text content captured by snapshots.
    DomText,
    /// Run environment metadata (hostnames, user, CI variables).
    EnvironmentMetadata,
}

/// Policy bundle the packager applies before writing a report. The
/// defaults are the safe-for-sharing baseline: header tokens, URL
/// query parameters, body fields named like secrets, env vars
/// containing TOKEN/SECRET/KEY are all redacted.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionPolicy {
    pub schema_version: String,
    /// Header names (case-insensitive) whose values are redacted.
    pub redacted_header_names: BTreeSet<String>,
    /// URL query-parameter names (case-insensitive) whose values are
    /// redacted in network URLs.
    pub redacted_query_params: BTreeSet<String>,
    /// Substrings (case-insensitive) that mark a body field name as
    /// sensitive when found inside `"name":"value"` pairs.
    pub body_key_substrings: BTreeSet<String>,
    /// Substrings (case-insensitive) that mark an env-var name as
    /// sensitive.
    pub env_key_substrings: BTreeSet<String>,
    /// Categories the packager is allowed to touch. Set explicitly so
    /// a report owner can opt out of one category (e.g. keep
    /// `network_body` raw inside a sealed bundle).
    pub enabled_scopes: BTreeSet<RedactionScope>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl Default for RedactionPolicy {
    fn default() -> Self {
        Self::shareable_baseline()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl RedactionPolicy {
    /// Safe defaults for "share this bundle on a PR" — every scope
    /// enabled, headers/query/body/env name lists set to the common
    /// secret families.
    pub fn shareable_baseline() -> Self {
        let mut scopes = BTreeSet::new();
        scopes.insert(RedactionScope::NetworkUrl);
        scopes.insert(RedactionScope::NetworkHeader);
        scopes.insert(RedactionScope::NetworkBody);
        scopes.insert(RedactionScope::DomText);
        scopes.insert(RedactionScope::EnvironmentMetadata);
        Self {
            schema_version: PM_REDACTION_SCHEMA_VERSION.to_string(),
            redacted_header_names: [
                "authorization",
                "proxy-authorization",
                "cookie",
                "set-cookie",
                "x-api-key",
                "x-auth-token",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            redacted_query_params: [
                "token",
                "access_token",
                "id_token",
                "refresh_token",
                "api_key",
                "signature",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            body_key_substrings: ["password", "token", "secret", "api_key", "authorization"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            env_key_substrings: ["token", "secret", "key", "password"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            enabled_scopes: scopes,
        }
    }

    /// Empty policy: no redaction, no scopes enabled. Useful for
    /// signed sealed-storage targets where the reviewer should see
    /// the raw bundle.
    pub fn passthrough() -> Self {
        Self {
            schema_version: PM_REDACTION_SCHEMA_VERSION.to_string(),
            redacted_header_names: BTreeSet::new(),
            redacted_query_params: BTreeSet::new(),
            body_key_substrings: BTreeSet::new(),
            env_key_substrings: BTreeSet::new(),
            enabled_scopes: BTreeSet::new(),
        }
    }

    /// Redact a single header value if its name matches the policy.
    pub fn redact_header(&self, name: &str, value: &str) -> String {
        if !self.enabled_scopes.contains(&RedactionScope::NetworkHeader) {
            return value.to_string();
        }
        if self
            .redacted_header_names
            .contains(&name.to_ascii_lowercase())
        {
            REDACTION_PLACEHOLDER.to_string()
        } else {
            value.to_string()
        }
    }

    /// Replace the value of any matched query parameter inside `url`
    /// with the placeholder. Other parameters and the path are left
    /// untouched.
    pub fn redact_url(&self, url: &str) -> String {
        if !self.enabled_scopes.contains(&RedactionScope::NetworkUrl) {
            return url.to_string();
        }
        let Some(qpos) = url.find('?') else {
            return url.to_string();
        };
        let (base, query) = url.split_at(qpos);
        let body = &query[1..];
        let redacted: Vec<String> = body
            .split('&')
            .map(|pair| {
                let (k, _v) = pair.split_once('=').unwrap_or((pair, ""));
                if self.redacted_query_params.contains(&k.to_ascii_lowercase()) {
                    format!("{k}={REDACTION_PLACEHOLDER}")
                } else {
                    pair.to_string()
                }
            })
            .collect();
        format!("{base}?{}", redacted.join("&"))
    }

    /// Redact an env-var pair if the key contains any sensitive
    /// substring. Case-insensitive on the key.
    pub fn redact_env(&self, key: &str, value: &str) -> String {
        if !self
            .enabled_scopes
            .contains(&RedactionScope::EnvironmentMetadata)
        {
            return value.to_string();
        }
        let lower = key.to_ascii_lowercase();
        if self
            .env_key_substrings
            .iter()
            .any(|needle| lower.contains(needle))
        {
            REDACTION_PLACEHOLDER.to_string()
        } else {
            value.to_string()
        }
    }

    /// True when a JSON body field name suggests a secret value. The
    /// packager uses this to decide whether to rewrite the value to
    /// the placeholder before serialising.
    pub fn should_redact_body_key(&self, key: &str) -> bool {
        if !self.enabled_scopes.contains(&RedactionScope::NetworkBody) {
            return false;
        }
        let lower = key.to_ascii_lowercase();
        self.body_key_substrings
            .iter()
            .any(|needle| lower.contains(needle))
    }

    /// Apply the policy in one pass and return the report metadata
    /// the PM shell renders alongside the bundle. `payload` exposes
    /// every redactable field so the policy can decide independently.
    pub fn apply(&self, payload: &RedactablePayload<'_>) -> RedactionReport {
        let mut redacted_headers = 0usize;
        let mut redacted_query_params = 0usize;
        let mut redacted_body_fields = 0usize;
        let mut redacted_env_vars = 0usize;

        for (name, value) in payload.headers.iter() {
            if self.redact_header(name, value) != *value {
                redacted_headers += 1;
            }
        }
        for url in payload.network_urls.iter() {
            let redacted = self.redact_url(url);
            if redacted != *url {
                let extra = url.matches('=').count();
                let kept = redacted.matches('=').count();
                let _ = extra;
                let _ = kept;
                redacted_query_params += url
                    .split('?')
                    .nth(1)
                    .map(|q| {
                        q.split('&')
                            .filter(|pair| {
                                let k = pair.split_once('=').map(|(k, _)| k).unwrap_or(pair);
                                self.redacted_query_params.contains(&k.to_ascii_lowercase())
                            })
                            .count()
                    })
                    .unwrap_or(0);
            }
        }
        for key in payload.body_keys.iter() {
            if self.should_redact_body_key(key) {
                redacted_body_fields += 1;
            }
        }
        for (key, value) in payload.env_vars.iter() {
            if self.redact_env(key, value) != *value {
                redacted_env_vars += 1;
            }
        }

        let any =
            redacted_headers + redacted_query_params + redacted_body_fields + redacted_env_vars > 0;
        RedactionReport {
            schema_version: PM_REDACTION_SCHEMA_VERSION.to_string(),
            redaction_applied: any || !self.enabled_scopes.is_empty(),
            redacted_headers,
            redacted_query_params,
            redacted_body_fields,
            redacted_env_vars,
            enabled_scopes: self.enabled_scopes.iter().copied().collect(),
        }
    }
}

/// Read-only view the packager passes to [`RedactionPolicy::apply`].
/// Borrows are deliberate so the policy never copies large bundle
/// data — it just counts and inspects.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone)]
pub struct RedactablePayload<'a> {
    pub headers: Vec<(&'a str, &'a str)>,
    pub network_urls: Vec<&'a str>,
    pub body_keys: Vec<&'a str>,
    pub env_vars: Vec<(&'a str, &'a str)>,
}

impl<'a> RedactablePayload<'a> {
    pub fn empty() -> Self {
        Self {
            headers: Vec::new(),
            network_urls: Vec::new(),
            body_keys: Vec::new(),
            env_vars: Vec::new(),
        }
    }
}

/// Metadata embedded in the report manifest so the PM shell can
/// surface a "redacted before share" badge and tests can assert
/// counts.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionReport {
    pub schema_version: String,
    pub redaction_applied: bool,
    pub redacted_headers: usize,
    pub redacted_query_params: usize,
    pub redacted_body_fields: usize,
    pub redacted_env_vars: usize,
    pub enabled_scopes: Vec<RedactionScope>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_redacts_known_sensitive_headers() {
        // Stop condition (#2732): known sensitive network/header
        // fields are redacted by default.
        let p = RedactionPolicy::shareable_baseline();
        assert_eq!(p.redact_header("Authorization", "Bearer abc"), "[REDACTED]");
        assert_eq!(p.redact_header("Cookie", "sid=42"), "[REDACTED]");
        assert_eq!(
            p.redact_header("Content-Type", "application/json"),
            "application/json"
        );
    }

    #[test]
    fn baseline_redacts_url_query_tokens_but_keeps_path() {
        let p = RedactionPolicy::shareable_baseline();
        let url = "https://api.example.com/v1/items?id=42&access_token=abcXYZ&page=2";
        let redacted = p.redact_url(url);
        assert!(redacted.contains("id=42"), "{redacted}");
        assert!(redacted.contains("page=2"), "{redacted}");
        assert!(redacted.contains("access_token=[REDACTED]"), "{redacted}");
        assert!(redacted.starts_with("https://api.example.com/v1/items?"));
    }

    #[test]
    fn baseline_recognises_secret_body_field_names() {
        let p = RedactionPolicy::shareable_baseline();
        assert!(p.should_redact_body_key("password"));
        assert!(p.should_redact_body_key("checkout_api_key"));
        assert!(!p.should_redact_body_key("customer_name"));
    }

    #[test]
    fn baseline_redacts_secret_env_vars() {
        let p = RedactionPolicy::shareable_baseline();
        assert_eq!(
            p.redact_env("STRIPE_SECRET_KEY", "sk_live_abc"),
            "[REDACTED]"
        );
        assert_eq!(p.redact_env("DEPLOY_TOKEN", "xyz"), "[REDACTED]");
        assert_eq!(p.redact_env("NODE_ENV", "production"), "production");
    }

    #[test]
    fn passthrough_policy_disables_all_scopes() {
        let p = RedactionPolicy::passthrough();
        assert_eq!(p.redact_header("Authorization", "Bearer x"), "Bearer x");
        assert_eq!(
            p.redact_url("https://x?access_token=y"),
            "https://x?access_token=y"
        );
        assert_eq!(p.redact_env("STRIPE_SECRET_KEY", "v"), "v");
        assert!(!p.should_redact_body_key("password"));
    }

    #[test]
    fn report_metadata_records_per_scope_counts() {
        // Stop condition (#2732): fixture metadata reports redaction.
        let p = RedactionPolicy::shareable_baseline();
        let report = p.apply(&RedactablePayload {
            headers: vec![("Authorization", "Bearer x"), ("Accept", "*/*")],
            network_urls: vec!["https://x.test/p?access_token=y&id=1"],
            body_keys: vec!["password", "username"],
            env_vars: vec![("API_TOKEN", "v"), ("PATH", "/usr/bin")],
        });
        assert!(report.redaction_applied);
        assert_eq!(report.redacted_headers, 1);
        assert_eq!(report.redacted_query_params, 1);
        assert_eq!(report.redacted_body_fields, 1);
        assert_eq!(report.redacted_env_vars, 1);
        assert_eq!(report.enabled_scopes.len(), 5);
    }

    #[test]
    fn passthrough_apply_reports_no_redaction() {
        // Stop condition (#2732): redacted vs unredacted bundle
        // rendering both supported.
        let p = RedactionPolicy::passthrough();
        let report = p.apply(&RedactablePayload {
            headers: vec![("Authorization", "Bearer x")],
            network_urls: vec!["https://x.test?access_token=y"],
            body_keys: vec!["password"],
            env_vars: vec![("API_TOKEN", "v")],
        });
        assert!(!report.redaction_applied);
        assert_eq!(report.redacted_headers, 0);
        assert_eq!(report.redacted_query_params, 0);
        assert_eq!(report.redacted_body_fields, 0);
        assert_eq!(report.redacted_env_vars, 0);
        assert!(report.enabled_scopes.is_empty());
    }

    #[test]
    fn report_round_trips_through_json() {
        let p = RedactionPolicy::shareable_baseline();
        let report = p.apply(&RedactablePayload::empty());
        let json = serde_json::to_string(&report).unwrap();
        let back: RedactionReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back, report);
        assert!(json.contains("\"network-header\""), "{json}");
    }
}
// CODEGEN-END
