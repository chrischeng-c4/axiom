// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-cassette-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! On-disk record/replay cassettes for the HTTP mock proxy.
//!
//! A cassette is one JSON file per request key (method+host+path+query+body
//! hash), holding the recorded response. Cassettes persist across runs under a
//! stable dir so a runner records a real response once and replays it forever,
//! offline and deterministically. Bodies are base64 (always) so non-UTF8 payloads
//! round-trip.
//!
//! @spec projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#logic

use std::path::PathBuf;

use base64::Engine;
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};

/// A recorded response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-cassette-rs.md#source
pub struct Recording {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// Response body, base64-encoded.
    pub body_b64: String,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-cassette-rs.md#source
impl Recording {
    pub fn new(status: u16, headers: Vec<(String, String)>, body: &[u8]) -> Self {
        Self {
            status,
            headers,
            body_b64: base64::engine::general_purpose::STANDARD.encode(body),
        }
    }

    pub fn body(&self) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(&self.body_b64)
            .unwrap_or_default()
    }
}

/// A directory of cassettes that persists across runs.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-cassette-rs.md#source
pub struct Cassettes {
    dir: PathBuf,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-emulator-httpmock-cassette-rs.md#source
impl Cassettes {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let _ = std::fs::create_dir_all(&dir);
        Self { dir }
    }

    /// Stable key for a request: method + host + path + query + body hash.
    pub fn key(method: &str, host: &str, path_and_query: &str, body: &[u8]) -> String {
        let mut hasher = Md5::new();
        hasher.update(method.as_bytes());
        hasher.update(b"\n");
        hasher.update(host.as_bytes());
        hasher.update(b"\n");
        hasher.update(path_and_query.as_bytes());
        hasher.update(b"\n");
        hasher.update(body);
        format!("{:x}", hasher.finalize())
    }

    fn path(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.json"))
    }

    /// Load a recording for `key`, if one exists.
    pub fn get(&self, key: &str) -> Option<Recording> {
        let bytes = std::fs::read(self.path(key)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Store a recording for `key`.
    pub fn put(&self, key: &str, rec: &Recording) {
        if let Ok(bytes) = serde_json::to_vec_pretty(rec) {
            let _ = std::fs::write(self.path(key), bytes);
        }
    }

    /// All recorded keys (file stems).
    pub fn keys(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(&self.dir) {
            for entry in rd.flatten() {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
                        out.push(stem.to_string());
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_is_stable_and_distinguishes() {
        let k1 = Cassettes::key("GET", "api.test", "/v1/x", b"");
        let k2 = Cassettes::key("GET", "api.test", "/v1/x", b"");
        let k3 = Cassettes::key("POST", "api.test", "/v1/x", b"body");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn roundtrips_on_disk_including_non_utf8() {
        let dir = tempfile::tempdir().unwrap();
        let c = Cassettes::new(dir.path());
        let body = vec![0u8, 159, 146, 150]; // invalid UTF-8
        let key = Cassettes::key("GET", "h", "/p", b"");
        c.put(
            &key,
            &Recording::new(
                200,
                vec![("content-type".into(), "application/octet-stream".into())],
                &body,
            ),
        );
        let got = c.get(&key).expect("recording present");
        assert_eq!(got.status, 200);
        assert_eq!(got.body(), body);
        assert!(c.keys().contains(&key));
        assert!(c.get("missing").is_none());
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
