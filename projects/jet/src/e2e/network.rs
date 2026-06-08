// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Network observation primitive for e2e (#2725).
//!
//! Product-flow E2E cases need to record the network traffic each
//! step issued so failures can be diagnosed without re-running the
//! browser. This module defines the minimal observation record
//! (request + response pair) and the per-step container that
//! attaches one or more records to a single product step. The
//! actual capture path (CDP interception, fetch instrumentation,
//! etc.) is the driver's responsibility — this module only models
//! the on-disk shape so run mode and open mode emit identical
//! evidence.
//!
//! Time/storage/permission controls are out of scope (see #2879,
//! #2880, #2881).

use serde::{Deserialize, Serialize};

/// Stable schema tag for [`NetworkObservation`].
pub const NETWORK_OBSERVATION_SCHEMA_VERSION: &str = "jet.e2e.network.v1";

/// One HTTP-style request issued during a step. The body is omitted
/// by default; drivers attach it only when the case opts in. Headers
/// are kept as `(name, value)` pairs to preserve duplicate keys
/// (e.g. `Set-Cookie`) without losing ordering.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub method: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
    /// `None` when the driver did not capture the body (default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// Response paired with the request. `None` for [`NetworkObservation::response`]
/// when the request failed before any response was received (DNS
/// failure, connection reset, deliberate abort).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub status: u16,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
    /// Body bytes recorded for the response, when the case opted in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

/// Failure recorded when no response arrived. Mirrors the lexicon
/// used by selector evidence so consumers can render mixed
/// failures with one switch.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkFailureKind {
    /// DNS resolution failed.
    Dns,
    /// TCP/TLS handshake or connection reset.
    Connection,
    /// Request aborted before completion (deliberate or otherwise).
    Aborted,
    /// Round-trip exceeded the configured deadline.
    Timeout,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkFailure {
    pub kind: NetworkFailureKind,
    pub message: String,
}

/// One observed request/response pair, plus the step-relative timing
/// the driver recorded. Either `response` or `failure` is populated
/// — never both.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkObservation {
    pub schema_version: String,
    pub request: NetworkRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<NetworkResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<NetworkFailure>,
    /// Wall-time the round-trip took, measured by the driver.
    pub duration_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl NetworkObservation {
    pub fn completed(request: NetworkRequest, response: NetworkResponse, duration_ms: u64) -> Self {
        Self {
            schema_version: NETWORK_OBSERVATION_SCHEMA_VERSION.to_string(),
            request,
            response: Some(response),
            failure: None,
            duration_ms,
        }
    }

    pub fn failed(request: NetworkRequest, failure: NetworkFailure, duration_ms: u64) -> Self {
        Self {
            schema_version: NETWORK_OBSERVATION_SCHEMA_VERSION.to_string(),
            request,
            response: None,
            failure: Some(failure),
            duration_ms,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.response.is_some()
    }
}

/// Per-step container. One step records 0..N observations; the
/// envelope embeds this object next to existing step evidence.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StepNetworkRecord {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub observations: Vec<NetworkObservation>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StepNetworkRecord {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, observation: NetworkObservation) {
        self.observations.push(observation);
    }

    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, url: &str) -> NetworkRequest {
        NetworkRequest {
            method: method.into(),
            url: url.into(),
            headers: vec![],
            body: None,
        }
    }

    #[test]
    fn single_observation_attaches_to_step_record() {
        // Stop condition (#2725): one network request/response record
        // attaches to the current step.
        let mut step = StepNetworkRecord::new();
        step.push(NetworkObservation::completed(
            request("GET", "https://aut.test/api/orders"),
            NetworkResponse {
                status: 200,
                headers: vec![("content-type".into(), "application/json".into())],
                body: None,
            },
            34,
        ));
        assert_eq!(step.len(), 1);
        assert!(step.observations[0].is_completed());
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"status\":200"), "{json}");
        assert!(json.contains("\"method\":\"GET\""), "{json}");
        assert!(json.contains("https://aut.test/api/orders"), "{json}");

        let back: StepNetworkRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(back, step);
    }

    #[test]
    fn failed_request_records_failure_kind_and_no_response() {
        let obs = NetworkObservation::failed(
            request("POST", "https://aut.test/api/checkout"),
            NetworkFailure {
                kind: NetworkFailureKind::Timeout,
                message: "Request exceeded 5000ms".into(),
            },
            5000,
        );
        assert!(!obs.is_completed());
        let json = serde_json::to_string(&obs).unwrap();
        assert!(json.contains("\"kind\":\"timeout\""), "{json}");
        assert!(!json.contains("\"response\""), "{json}");
    }

    #[test]
    fn empty_step_record_skips_serialising_observations() {
        let step = StepNetworkRecord::new();
        let json = serde_json::to_string(&step).unwrap();
        assert!(!json.contains("\"observations\""), "{json}");
    }

    #[test]
    fn multiple_observations_preserve_order() {
        let mut step = StepNetworkRecord::new();
        step.push(NetworkObservation::completed(
            request("GET", "https://aut.test/a"),
            NetworkResponse {
                status: 200,
                headers: vec![],
                body: None,
            },
            1,
        ));
        step.push(NetworkObservation::completed(
            request("GET", "https://aut.test/b"),
            NetworkResponse {
                status: 204,
                headers: vec![],
                body: None,
            },
            2,
        ));
        assert_eq!(step.observations[0].request.url, "https://aut.test/a");
        assert_eq!(step.observations[1].request.url, "https://aut.test/b");
    }

    #[test]
    fn body_skip_serialises_when_not_captured() {
        let obs = NetworkObservation::completed(
            request("GET", "https://aut.test/api"),
            NetworkResponse {
                status: 200,
                headers: vec![],
                body: None,
            },
            1,
        );
        let json = serde_json::to_string(&obs).unwrap();
        assert!(!json.contains("\"body\""), "{json}");
    }

    #[test]
    fn body_round_trips_when_present() {
        let mut req = request("POST", "https://aut.test/api/orders");
        req.body = Some(r#"{"id":42}"#.into());
        let obs = NetworkObservation::completed(
            req,
            NetworkResponse {
                status: 201,
                headers: vec![],
                body: Some(r#"{"ok":true}"#.into()),
            },
            18,
        );
        let json = serde_json::to_string(&obs).unwrap();
        let back: NetworkObservation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.request.body.as_deref(), Some(r#"{"id":42}"#));
        assert_eq!(
            back.response.as_ref().unwrap().body.as_deref(),
            Some(r#"{"ok":true}"#),
        );
    }
}
// CODEGEN-END
