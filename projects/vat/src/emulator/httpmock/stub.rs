// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Stub registry + matcher for the HTTP mock proxy.
//!
//! A stub pairs a request matcher (method / host / path / path-prefix) with a
//! canned response. Stubs are registered via the admin API and always take
//! precedence over cassette replay. Matching is first-registered-wins.
//!
//! @spec projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#logic

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
pub struct Matcher {
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    /// Match when the request path starts with this prefix.
    #[serde(default)]
    pub path_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
pub struct StubResponse {
    #[serde(default = "default_status")]
    pub status: u16,
    #[serde(default)]
    pub headers: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub body: String,
}

fn default_status() -> u16 {
    200
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
pub struct Stub {
    #[serde(default)]
    pub r#match: Matcher,
    pub response: StubResponse,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
impl Stub {
    /// Whether this stub matches the request. `path` excludes the query string.
    pub fn matches(&self, method: &str, host: &str, path: &str) -> bool {
        let m = &self.r#match;
        m.method
            .as_deref()
            .map(|x| x.eq_ignore_ascii_case(method))
            .unwrap_or(true)
            && m.host.as_deref().map(|x| x == host).unwrap_or(true)
            && m.path.as_deref().map(|x| x == path).unwrap_or(true)
            && m.path_prefix
                .as_deref()
                .map(|x| path.starts_with(x))
                .unwrap_or(true)
    }
}

/// Thread-safe registry of stubs (first match wins).
#[derive(Default)]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
pub struct Registry {
    stubs: Mutex<Vec<Stub>>,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-stub-rs.md#source
impl Registry {
    pub fn add(&self, stub: Stub) {
        self.stubs.lock().unwrap().push(stub);
    }

    pub fn clear(&self) {
        self.stubs.lock().unwrap().clear();
    }

    pub fn find(&self, method: &str, host: &str, path: &str) -> Option<StubResponse> {
        self.stubs
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.matches(method, host, path))
            .map(|s| s.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stub(json: &str) -> Stub {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn matches_by_method_host_path() {
        let reg = Registry::default();
        reg.add(stub(
            r#"{"match":{"method":"GET","host":"api.test","path":"/v1/x"},"response":{"status":201,"body":"ok"}}"#,
        ));
        let hit = reg.find("GET", "api.test", "/v1/x").unwrap();
        assert_eq!(hit.status, 201);
        assert_eq!(hit.body, "ok");
        // method mismatch
        assert!(reg.find("POST", "api.test", "/v1/x").is_none());
        // host mismatch
        assert!(reg.find("GET", "other", "/v1/x").is_none());
    }

    #[test]
    fn path_prefix_and_defaults() {
        let reg = Registry::default();
        reg.add(stub(
            r#"{"match":{"path_prefix":"/v1/"},"response":{"body":"any"}}"#,
        ));
        let hit = reg.find("DELETE", "anyhost", "/v1/deep/thing").unwrap();
        assert_eq!(hit.status, 200); // default
        assert_eq!(hit.body, "any");
        assert!(reg.find("GET", "h", "/v2/x").is_none());
        reg.clear();
        assert!(reg.find("GET", "h", "/v1/x").is_none());
    }
}
// CODEGEN-END
