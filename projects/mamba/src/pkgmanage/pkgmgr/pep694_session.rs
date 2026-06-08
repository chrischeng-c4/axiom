// pep694_session.rs — PEP 694 upload-session JSON shapes.
//
// PEP 694 introduces a staged upload protocol that replaces the
// legacy single-shot `/legacy/?action=file_upload` endpoint. The
// client:
//
//   1. POSTs an "initiate" body to the index to create a session
//      (Pending → Initiated).
//   2. PUTs each artifact (wheel / sdist / RECORD.jws / etc.) into
//      the session.
//   3. POSTs to the publish link to atomically promote all staged
//      files (Initiated → Completed). The index becomes visible to
//      readers at that moment.
//   4. May POST to the cancel link before publish to discard the
//      staged state (Initiated → Cancelled).
//
// This module is JSON-only: it deserializes the session-status
// response document and the file-status sub-objects, and offers a
// matching builder for the "initiate" request. The HTTP transport
// (multipart PUTs, retries, conditional GET) lives in `publish.rs`.
//
// Note: PEP 694 is a draft; the shapes here track the latest pre-RFC
// text (2024 draft) and may be refined as Warehouse stabilizes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Lifecycle state of an upload session per PEP 694.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionStatus {
    /// Client has POSTed initiate; index is provisioning storage.
    Pending,
    /// Storage is ready; client may PUT files.
    Initiated,
    /// Publish succeeded; files are visible.
    Completed,
    /// Client (or index) cancelled the session.
    Cancelled,
    /// Index encountered a non-recoverable error.
    Errored,
}

/// Lifecycle state of a single file within an upload session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileStatus {
    /// File has been declared in the session but not yet uploaded.
    Pending,
    /// File has been uploaded but not yet validated.
    Staged,
    /// File has been validated and is ready for publish.
    Uploaded,
}

/// One file entry inside the session-status `files` map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFile {
    pub status: FileStatus,
    /// Bytes (server-reported), if known.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size: Option<u64>,
    /// Algorithm → hex-digest map, if known. PEP 694 uses the same
    /// shape as PEP 691 (`{"sha256": "...", "blake2b": "..."}`).
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub hashes: BTreeMap<String, String>,
}

/// Top-level session-status response per PEP 694.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadSession {
    pub name: String,
    pub version: String,
    pub status: SessionStatus,
    /// Human-readable reason; populated when status is
    /// `Cancelled` or `Errored`.
    #[serde(rename = "status-reason", skip_serializing_if = "Option::is_none", default)]
    pub status_reason: Option<String>,
    /// RFC 3339 timestamp at which the index will reclaim staged state
    /// if not published. Kept as a string for round-trip fidelity —
    /// the caller decides how to parse.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expires: Option<String>,
    /// Per-file status. Keys are filenames inside the session.
    #[serde(default)]
    pub files: BTreeMap<String, SessionFile>,
    /// Action links keyed by rel name (`upload`, `cancel`, `publish`).
    /// Servers may add custom rels; unknown rels round-trip verbatim.
    #[serde(default)]
    pub links: BTreeMap<String, String>,
}

impl UploadSession {
    pub fn upload_link(&self) -> Option<&str> {
        self.links.get("upload").map(String::as_str)
    }
    pub fn cancel_link(&self) -> Option<&str> {
        self.links.get("cancel").map(String::as_str)
    }
    pub fn publish_link(&self) -> Option<&str> {
        self.links.get("publish").map(String::as_str)
    }
}

/// PEP 694 envelope for the "initiate" POST body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiateRequest {
    pub meta: InitiateMeta,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitiateMeta {
    /// PEP 694 negotiates capabilities via api-version. We pin the
    /// builder at `1.0` (the only version defined as of the draft).
    #[serde(rename = "api-version")]
    pub api_version: String,
}

impl InitiateRequest {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            meta: InitiateMeta {
                api_version: "1.0".into(),
            },
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Parse a session-status response document.
pub fn parse_session(src: &str) -> Result<UploadSession, IndexError> {
    serde_json::from_str(src).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: format!("PEP 694 session JSON: {e}"),
    })
}

/// Render an initiate-request body as JSON.
pub fn render_initiate(req: &InitiateRequest) -> Result<String, IndexError> {
    serde_json::to_string(req).map_err(|e| IndexError::ParseError {
        url: String::new(),
        detail: format!("PEP 694 initiate JSON: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_detail(err: IndexError) -> String {
        match err {
            IndexError::ParseError { detail, .. } => detail,
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    // ---- parse_session -------------------------------------------------

    #[test]
    fn parse_minimal_pending_session() {
        let src = r#"{
            "name": "mypkg",
            "version": "1.0.0",
            "status": "pending"
        }"#;
        let s = parse_session(src).unwrap();
        assert_eq!(s.name, "mypkg");
        assert_eq!(s.version, "1.0.0");
        assert_eq!(s.status, SessionStatus::Pending);
        assert!(s.status_reason.is_none());
        assert!(s.files.is_empty());
        assert!(s.links.is_empty());
    }

    #[test]
    fn parse_status_variants_all_round_trip() {
        for (text, expected) in [
            ("pending", SessionStatus::Pending),
            ("initiated", SessionStatus::Initiated),
            ("completed", SessionStatus::Completed),
            ("cancelled", SessionStatus::Cancelled),
            ("errored", SessionStatus::Errored),
        ] {
            let src = format!(
                r#"{{"name":"x","version":"1.0","status":"{text}"}}"#
            );
            assert_eq!(parse_session(&src).unwrap().status, expected);
        }
    }

    #[test]
    fn parse_errored_session_with_reason() {
        let src = r#"{
            "name": "mypkg",
            "version": "1.0.0",
            "status": "errored",
            "status-reason": "file digest mismatch"
        }"#;
        let s = parse_session(src).unwrap();
        assert_eq!(s.status, SessionStatus::Errored);
        assert_eq!(s.status_reason.as_deref(), Some("file digest mismatch"));
    }

    #[test]
    fn parse_session_with_files_and_links() {
        let src = r#"{
            "name": "mypkg",
            "version": "1.0.0",
            "status": "initiated",
            "expires": "2026-06-01T12:00:00Z",
            "files": {
                "mypkg-1.0-py3-none-any.whl": {
                    "status": "uploaded",
                    "size": 123456,
                    "hashes": {"sha256": "abc123"}
                },
                "mypkg-1.0.tar.gz": {
                    "status": "pending"
                }
            },
            "links": {
                "upload": "https://example.com/upload/abc",
                "cancel": "https://example.com/cancel/abc",
                "publish": "https://example.com/publish/abc"
            }
        }"#;
        let s = parse_session(src).unwrap();
        assert_eq!(s.expires.as_deref(), Some("2026-06-01T12:00:00Z"));
        assert_eq!(s.files.len(), 2);
        let wheel = &s.files["mypkg-1.0-py3-none-any.whl"];
        assert_eq!(wheel.status, FileStatus::Uploaded);
        assert_eq!(wheel.size, Some(123456));
        assert_eq!(wheel.hashes.get("sha256"), Some(&"abc123".to_string()));
        let sdist = &s.files["mypkg-1.0.tar.gz"];
        assert_eq!(sdist.status, FileStatus::Pending);
        assert_eq!(sdist.size, None);
        assert!(sdist.hashes.is_empty());

        assert_eq!(
            s.upload_link(),
            Some("https://example.com/upload/abc")
        );
        assert_eq!(
            s.cancel_link(),
            Some("https://example.com/cancel/abc")
        );
        assert_eq!(
            s.publish_link(),
            Some("https://example.com/publish/abc")
        );
    }

    #[test]
    fn parse_session_preserves_unknown_link_rels() {
        let src = r#"{
            "name": "x",
            "version": "1.0",
            "status": "initiated",
            "links": {
                "upload": "https://up/",
                "custom-rel": "https://custom/"
            }
        }"#;
        let s = parse_session(src).unwrap();
        assert_eq!(s.links.get("custom-rel"), Some(&"https://custom/".to_string()));
    }

    #[test]
    fn parse_rejects_unknown_status_value() {
        let src = r#"{"name":"x","version":"1.0","status":"frobnicated"}"#;
        let err = parse_session(src).unwrap_err();
        assert!(err_detail(err).contains("PEP 694 session JSON"));
    }

    #[test]
    fn parse_rejects_unknown_file_status() {
        let src = r#"{
            "name":"x","version":"1.0","status":"initiated",
            "files":{"a.whl":{"status":"frob"}}
        }"#;
        let err = parse_session(src).unwrap_err();
        assert!(err_detail(err).contains("PEP 694 session JSON"));
    }

    #[test]
    fn parse_rejects_missing_required_field() {
        let src = r#"{"name":"x","status":"pending"}"#;
        let err = parse_session(src).unwrap_err();
        assert!(err_detail(err).contains("PEP 694 session JSON"));
    }

    #[test]
    fn parse_rejects_invalid_json() {
        let err = parse_session("not json").unwrap_err();
        assert!(err_detail(err).contains("PEP 694 session JSON"));
    }

    // ---- file_status variants -----------------------------------------

    #[test]
    fn file_status_variants_round_trip() {
        for (text, expected) in [
            ("pending", FileStatus::Pending),
            ("staged", FileStatus::Staged),
            ("uploaded", FileStatus::Uploaded),
        ] {
            let src = format!(
                r#"{{"name":"x","version":"1.0","status":"initiated",
                     "files":{{"a.whl":{{"status":"{text}"}}}}}}"#
            );
            assert_eq!(parse_session(&src).unwrap().files["a.whl"].status, expected);
        }
    }

    // ---- InitiateRequest builder --------------------------------------

    #[test]
    fn initiate_builder_pins_api_version_1_0() {
        let req = InitiateRequest::new("mypkg", "1.2.3");
        assert_eq!(req.meta.api_version, "1.0");
        assert_eq!(req.name, "mypkg");
        assert_eq!(req.version, "1.2.3");
    }

    #[test]
    fn initiate_render_emits_pep694_shape() {
        let req = InitiateRequest::new("mypkg", "1.2.3");
        let json = render_initiate(&req).unwrap();
        // Field order is deterministic from derive macro.
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["meta"]["api-version"].as_str(), Some("1.0"));
        assert_eq!(parsed["name"].as_str(), Some("mypkg"));
        assert_eq!(parsed["version"].as_str(), Some("1.2.3"));
    }

    #[test]
    fn initiate_round_trip_via_serde() {
        let req = InitiateRequest::new("mypkg", "1.2.3");
        let json = render_initiate(&req).unwrap();
        let back: InitiateRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    // ---- Link accessors -----------------------------------------------

    #[test]
    fn link_accessors_return_none_when_absent() {
        let src = r#"{"name":"x","version":"1.0","status":"pending"}"#;
        let s = parse_session(src).unwrap();
        assert!(s.upload_link().is_none());
        assert!(s.cancel_link().is_none());
        assert!(s.publish_link().is_none());
    }

    // ---- realistic workflow -------------------------------------------

    #[test]
    fn realistic_workflow_pending_then_initiated_then_completed() {
        // 1. Pending
        let s1 = parse_session(r#"{"name":"mypkg","version":"1.0","status":"pending"}"#)
            .unwrap();
        assert_eq!(s1.status, SessionStatus::Pending);

        // 2. Initiated, files pending
        let s2 = parse_session(
            r#"{
                "name":"mypkg","version":"1.0","status":"initiated",
                "files":{"mypkg-1.0-py3-none-any.whl":{"status":"pending"}},
                "links":{"upload":"https://u/","publish":"https://p/","cancel":"https://c/"}
            }"#,
        )
        .unwrap();
        assert_eq!(s2.status, SessionStatus::Initiated);
        assert_eq!(s2.files.len(), 1);

        // 3. Completed
        let s3 = parse_session(
            r#"{
                "name":"mypkg","version":"1.0","status":"completed",
                "files":{"mypkg-1.0-py3-none-any.whl":{"status":"uploaded","size":1024}}
            }"#,
        )
        .unwrap();
        assert_eq!(s3.status, SessionStatus::Completed);
        assert_eq!(s3.files["mypkg-1.0-py3-none-any.whl"].size, Some(1024));
    }
}
