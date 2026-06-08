// PEP 610 `.dist-info/direct_url.json` reader + writer (Tick 66).
//
// Tick 32's `direct_url.rs` parses requirement-line forms like
//   foo @ git+https://example.test/foo.git@v1
// PEP 610 is the *other* end of that story: when an installer
// installs from such a URL, it MUST drop a JSON file at
// `.dist-info/direct_url.json` recording where the distribution came
// from. Tools like `pip freeze` consult this file to reproduce the
// original install line and `pip uninstall` consults it to decide
// whether the distribution came from an index or a direct URL.
//
// JSON shape (PEP 610 §1):
//   { "url": "<absolute URL>", "<info-key>": { ... } }
//
// Exactly one of three info keys is present:
//   * `vcs_info`     — version-control checkout
//       { "vcs": "git" | "hg" | "svn" | "bzr",
//         "requested_revision": "branch / tag",        (optional)
//         "commit_id": "abc123...",                    (often required)
//         "resolved_revision": "...",                  (optional, hg/svn)
//         "subdirectory": "pkgs/foo"                   (top-level optional)
//       }
//   * `archive_info` — direct archive download
//       { "hashes": { "sha256": "..." },               (optional)
//         "hash":   "sha256=..."                       (optional, deprecated)
//       }
//   * `dir_info`     — local directory install
//       { "editable": true | false }                   (optional, default false)
//
// `subdirectory` lives at the top level alongside `url`, not inside
// the info subtable (PEP 610 §1.1.5).
//
// This module composes with Tick 32's `DirectUrl` enum: a parsed
// `DirectUrl` can be lifted to a `DirectUrlJson` for serialization,
// and a parsed JSON file can be lowered back to a `DirectUrl`.

use serde_json::{json, Map, Value};

use crate::pkgmanage::pkgmgr::direct_url::DirectUrl;
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Decoded contents of a `direct_url.json` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectUrlJson {
    /// Absolute URL (`https://…`, `git+ssh://…`, `file:///…`).
    pub url: String,
    /// Optional subdirectory within the artifact (e.g. monorepo
    /// member). Top-level per PEP 610 §1.1.5.
    pub subdirectory: Option<String>,
    pub info: DirectUrlInfo,
}

/// The "what kind of source" half of a `direct_url.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectUrlInfo {
    Vcs {
        vcs: String,
        requested_revision: Option<String>,
        commit_id: Option<String>,
        resolved_revision: Option<String>,
    },
    Archive {
        /// Hash algorithm -> hex digest. PEP 610 stores them in
        /// `hashes`; the older single-string `hash` field is
        /// normalized into the map at parse time.
        hashes: Vec<(String, String)>,
    },
    Dir {
        editable: bool,
    },
}

impl DirectUrlJson {
    /// Lift a `DirectUrl` (the requirement-line form) to the PEP 610
    /// JSON form. Archive variants emit `archive_info` with an empty
    /// hashes map — callers that recompute the hash should fill it in
    /// before writing.
    pub fn from_direct_url(d: &DirectUrl) -> Self {
        match d {
            DirectUrl::Archive { url, subdirectory } => DirectUrlJson {
                url: url.clone(),
                subdirectory: subdirectory.clone(),
                info: DirectUrlInfo::Archive { hashes: Vec::new() },
            },
            DirectUrl::Git {
                url,
                rev,
                subdirectory,
            } => DirectUrlJson {
                // PEP 610 wants the bare transport URL — the `git+`
                // prefix has already been stripped by `direct_url.rs`.
                url: url.clone(),
                subdirectory: subdirectory.clone(),
                info: DirectUrlInfo::Vcs {
                    vcs: "git".into(),
                    requested_revision: rev.clone(),
                    commit_id: None,
                    resolved_revision: None,
                },
            },
            DirectUrl::LocalPath { path, subdirectory } => DirectUrlJson {
                url: file_url_for(path),
                subdirectory: subdirectory.clone(),
                info: DirectUrlInfo::Dir { editable: false },
            },
        }
    }

    /// Lower this JSON form back to a `DirectUrl`. Hashes and
    /// commit_id are dropped — they don't fit the requirement-line
    /// representation.
    pub fn to_direct_url(&self) -> DirectUrl {
        match &self.info {
            DirectUrlInfo::Archive { .. } => DirectUrl::Archive {
                url: self.url.clone(),
                subdirectory: self.subdirectory.clone(),
            },
            DirectUrlInfo::Vcs {
                requested_revision, ..
            } => DirectUrl::Git {
                url: self.url.clone(),
                rev: requested_revision.clone(),
                subdirectory: self.subdirectory.clone(),
            },
            DirectUrlInfo::Dir { .. } => {
                let path = if let Some(p) = self.url.strip_prefix("file://") {
                    p.to_string()
                } else {
                    self.url.clone()
                };
                DirectUrl::LocalPath {
                    path,
                    subdirectory: self.subdirectory.clone(),
                }
            }
        }
    }
}

/// Wrap a local path in the canonical `file://` URL form.
fn file_url_for(path: &str) -> String {
    if path.starts_with("file://") {
        path.to_string()
    } else if path.starts_with('/') {
        format!("file://{path}")
    } else {
        // Relative paths stay relative in the JSON — PEP 610 allows
        // both absolute file URLs and relative paths in `dir_info`
        // setups, with relative paths interpreted by the consumer.
        path.to_string()
    }
}

/// Parse a `direct_url.json` body. Rejects payloads that have zero
/// or more than one info subtable.
pub fn parse_direct_url_json(src: &str) -> Result<DirectUrlJson, IndexError> {
    let raw: Value = serde_json::from_str(src).map_err(|e| IndexError::ParseError {
        url: "<direct_url.json>".into(),
        detail: format!("invalid json: {e}"),
    })?;
    let obj = raw.as_object().ok_or_else(|| IndexError::ParseError {
        url: "<direct_url.json>".into(),
        detail: "direct_url.json: root must be a JSON object".into(),
    })?;

    let url = obj
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: "<direct_url.json>".into(),
            detail: "direct_url.json: missing required string field `url`".into(),
        })?
        .to_string();

    let subdirectory = obj
        .get("subdirectory")
        .and_then(|v| v.as_str())
        .map(String::from);

    let has_vcs = obj.contains_key("vcs_info");
    let has_archive = obj.contains_key("archive_info");
    let has_dir = obj.contains_key("dir_info");
    let info_count = [has_vcs, has_archive, has_dir].iter().filter(|b| **b).count();
    if info_count == 0 {
        return Err(IndexError::ParseError {
            url: "<direct_url.json>".into(),
            detail: "direct_url.json: must contain exactly one of \
                     vcs_info / archive_info / dir_info"
                .into(),
        });
    }
    if info_count > 1 {
        return Err(IndexError::ParseError {
            url: "<direct_url.json>".into(),
            detail: "direct_url.json: must contain exactly one of \
                     vcs_info / archive_info / dir_info (found multiple)"
                .into(),
        });
    }

    let info = if has_vcs {
        decode_vcs(obj.get("vcs_info").unwrap())?
    } else if has_archive {
        decode_archive(obj.get("archive_info").unwrap())?
    } else {
        decode_dir(obj.get("dir_info").unwrap())?
    };

    Ok(DirectUrlJson {
        url,
        subdirectory,
        info,
    })
}

fn decode_vcs(v: &Value) -> Result<DirectUrlInfo, IndexError> {
    let t = v.as_object().ok_or_else(|| IndexError::ParseError {
        url: "<direct_url.json>".into(),
        detail: "direct_url.json: vcs_info must be a JSON object".into(),
    })?;
    let vcs = t
        .get("vcs")
        .and_then(|v| v.as_str())
        .ok_or_else(|| IndexError::ParseError {
            url: "<direct_url.json>".into(),
            detail: "direct_url.json: vcs_info missing required string field `vcs`".into(),
        })?
        .to_string();
    let requested_revision = t
        .get("requested_revision")
        .and_then(|v| v.as_str())
        .map(String::from);
    let commit_id = t.get("commit_id").and_then(|v| v.as_str()).map(String::from);
    let resolved_revision = t
        .get("resolved_revision")
        .and_then(|v| v.as_str())
        .map(String::from);
    Ok(DirectUrlInfo::Vcs {
        vcs,
        requested_revision,
        commit_id,
        resolved_revision,
    })
}

fn decode_archive(v: &Value) -> Result<DirectUrlInfo, IndexError> {
    let t = v.as_object().ok_or_else(|| IndexError::ParseError {
        url: "<direct_url.json>".into(),
        detail: "direct_url.json: archive_info must be a JSON object".into(),
    })?;
    let mut hashes: Vec<(String, String)> = Vec::new();

    if let Some(map) = t.get("hashes").and_then(|v| v.as_object()) {
        for (k, val) in map {
            let s = val.as_str().ok_or_else(|| IndexError::ParseError {
                url: "<direct_url.json>".into(),
                detail: format!(
                    "direct_url.json: archive_info.hashes.{k} must be a string"
                ),
            })?;
            hashes.push((k.clone(), s.to_string()));
        }
        // Deterministic order regardless of JSON object key order.
        hashes.sort_by(|a, b| a.0.cmp(&b.0));
    } else if let Some(single) = t.get("hash").and_then(|v| v.as_str()) {
        // Older `hash = "sha256=..."` form. Split on '=' once.
        let (algo, digest) = single.split_once('=').ok_or_else(|| IndexError::ParseError {
            url: "<direct_url.json>".into(),
            detail: format!(
                "direct_url.json: archive_info.hash {single:?} must be `algo=digest`"
            ),
        })?;
        hashes.push((algo.to_string(), digest.to_string()));
    }

    Ok(DirectUrlInfo::Archive { hashes })
}

fn decode_dir(v: &Value) -> Result<DirectUrlInfo, IndexError> {
    let t = v.as_object().ok_or_else(|| IndexError::ParseError {
        url: "<direct_url.json>".into(),
        detail: "direct_url.json: dir_info must be a JSON object".into(),
    })?;
    let editable = t
        .get("editable")
        .map(|v| {
            v.as_bool().ok_or_else(|| IndexError::ParseError {
                url: "<direct_url.json>".into(),
                detail: "direct_url.json: dir_info.editable must be a boolean".into(),
            })
        })
        .transpose()?
        .unwrap_or(false);
    Ok(DirectUrlInfo::Dir { editable })
}

/// Render a `DirectUrlJson` to its on-disk JSON form. Output is
/// canonical: `url` first, then `subdirectory` if set, then the
/// info subtable. Within `archive_info.hashes` the algorithms are
/// sorted alphabetically so two installs of the same artifact
/// produce byte-identical files.
pub fn render_direct_url_json(d: &DirectUrlJson) -> String {
    let mut root = Map::new();
    root.insert("url".into(), Value::String(d.url.clone()));
    if let Some(sub) = &d.subdirectory {
        root.insert("subdirectory".into(), Value::String(sub.clone()));
    }
    match &d.info {
        DirectUrlInfo::Vcs {
            vcs,
            requested_revision,
            commit_id,
            resolved_revision,
        } => {
            let mut vcs_map = Map::new();
            vcs_map.insert("vcs".into(), Value::String(vcs.clone()));
            if let Some(rr) = requested_revision {
                vcs_map.insert("requested_revision".into(), Value::String(rr.clone()));
            }
            if let Some(cid) = commit_id {
                vcs_map.insert("commit_id".into(), Value::String(cid.clone()));
            }
            if let Some(rev) = resolved_revision {
                vcs_map.insert("resolved_revision".into(), Value::String(rev.clone()));
            }
            root.insert("vcs_info".into(), Value::Object(vcs_map));
        }
        DirectUrlInfo::Archive { hashes } => {
            let mut a = Map::new();
            if !hashes.is_empty() {
                let mut sorted = hashes.clone();
                sorted.sort_by(|a, b| a.0.cmp(&b.0));
                let mut h = Map::new();
                for (k, v) in sorted {
                    h.insert(k, Value::String(v));
                }
                a.insert("hashes".into(), Value::Object(h));
            }
            root.insert("archive_info".into(), Value::Object(a));
        }
        DirectUrlInfo::Dir { editable } => {
            root.insert("dir_info".into(), json!({ "editable": *editable }));
        }
    }
    serde_json::to_string(&Value::Object(root)).expect("serde_json::to_string never fails on Map")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vcs_sample() -> DirectUrlJson {
        DirectUrlJson {
            url: "https://github.com/example/foo.git".into(),
            subdirectory: None,
            info: DirectUrlInfo::Vcs {
                vcs: "git".into(),
                requested_revision: Some("v1.2.3".into()),
                commit_id: Some("a1b2c3d4".into()),
                resolved_revision: None,
            },
        }
    }

    #[test]
    fn parse_archive_with_hashes_map() {
        let src = r#"{
            "url": "https://example.test/foo-1.0.tar.gz",
            "archive_info": { "hashes": { "sha256": "abc", "md5": "def" } }
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        assert_eq!(d.url, "https://example.test/foo-1.0.tar.gz");
        match d.info {
            DirectUrlInfo::Archive { hashes } => {
                // Deterministic — alphabetical sort.
                assert_eq!(hashes, vec![
                    ("md5".to_string(), "def".to_string()),
                    ("sha256".to_string(), "abc".to_string()),
                ]);
            }
            other => panic!("expected Archive, got {other:?}"),
        }
    }

    #[test]
    fn parse_archive_with_deprecated_single_hash_field() {
        let src = r#"{
            "url": "https://example.test/foo-1.0.tar.gz",
            "archive_info": { "hash": "sha256=abcdef" }
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        match d.info {
            DirectUrlInfo::Archive { hashes } => {
                assert_eq!(hashes, vec![("sha256".to_string(), "abcdef".to_string())]);
            }
            other => panic!("expected Archive, got {other:?}"),
        }
    }

    #[test]
    fn parse_archive_with_no_hashes_yields_empty_vec() {
        let src = r#"{
            "url": "https://example.test/foo-1.0.tar.gz",
            "archive_info": {}
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        assert!(matches!(d.info, DirectUrlInfo::Archive { hashes } if hashes.is_empty()));
    }

    #[test]
    fn parse_vcs_full_payload() {
        let src = r#"{
            "url": "https://github.com/example/foo.git",
            "vcs_info": {
                "vcs": "git",
                "requested_revision": "v1.2.3",
                "commit_id": "a1b2c3d4",
                "resolved_revision": "deadbeef"
            }
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        match d.info {
            DirectUrlInfo::Vcs {
                vcs,
                requested_revision,
                commit_id,
                resolved_revision,
            } => {
                assert_eq!(vcs, "git");
                assert_eq!(requested_revision.as_deref(), Some("v1.2.3"));
                assert_eq!(commit_id.as_deref(), Some("a1b2c3d4"));
                assert_eq!(resolved_revision.as_deref(), Some("deadbeef"));
            }
            other => panic!("expected Vcs, got {other:?}"),
        }
    }

    #[test]
    fn parse_dir_with_explicit_editable_true() {
        let src = r#"{
            "url": "file:///abs/path/to/foo",
            "dir_info": { "editable": true }
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        assert!(matches!(d.info, DirectUrlInfo::Dir { editable: true }));
    }

    #[test]
    fn parse_dir_defaults_editable_to_false_when_missing() {
        let src = r#"{
            "url": "file:///abs/path/to/foo",
            "dir_info": {}
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        assert!(matches!(d.info, DirectUrlInfo::Dir { editable: false }));
    }

    #[test]
    fn parse_with_top_level_subdirectory() {
        let src = r#"{
            "url": "https://example.test/m.tar.gz",
            "subdirectory": "pkgs/foo",
            "archive_info": {}
        }"#;
        let d = parse_direct_url_json(src).unwrap();
        assert_eq!(d.subdirectory.as_deref(), Some("pkgs/foo"));
    }

    #[test]
    fn parse_rejects_invalid_json() {
        let err = parse_direct_url_json("not json").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("invalid json"));
    }

    #[test]
    fn parse_rejects_non_object_root() {
        let err = parse_direct_url_json("[]").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("root must be a JSON object"));
    }

    #[test]
    fn parse_rejects_missing_url() {
        let src = r#"{ "dir_info": {} }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing required string field `url`"));
    }

    #[test]
    fn parse_rejects_missing_info_subtable() {
        let src = r#"{ "url": "https://e.test/x" }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("must contain exactly one"));
    }

    #[test]
    fn parse_rejects_multiple_info_subtables() {
        let src = r#"{
            "url": "https://e.test/x",
            "archive_info": {},
            "dir_info": {}
        }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("found multiple"));
    }

    #[test]
    fn parse_rejects_non_object_info_subtable() {
        let src = r#"{ "url": "x", "vcs_info": 42 }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("vcs_info must be a JSON object"));
    }

    #[test]
    fn parse_rejects_missing_vcs_kind() {
        let src = r#"{ "url": "x", "vcs_info": {} }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("vcs_info missing required string field `vcs`"));
    }

    #[test]
    fn parse_rejects_non_string_hash_value() {
        let src = r#"{
            "url": "x",
            "archive_info": { "hashes": { "sha256": 42 } }
        }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("must be a string"));
    }

    #[test]
    fn parse_rejects_non_bool_editable() {
        let src = r#"{ "url": "x", "dir_info": { "editable": "yes" } }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("dir_info.editable must be a boolean"));
    }

    #[test]
    fn parse_rejects_malformed_deprecated_hash_field() {
        let src = r#"{
            "url": "x",
            "archive_info": { "hash": "no-equals-sign" }
        }"#;
        let err = parse_direct_url_json(src).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("must be `algo=digest`"));
    }

    #[test]
    fn render_archive_emits_hashes_in_alphabetical_order() {
        let d = DirectUrlJson {
            url: "https://e.test/foo.tar.gz".into(),
            subdirectory: None,
            info: DirectUrlInfo::Archive {
                hashes: vec![
                    ("sha256".into(), "z".into()),
                    ("md5".into(), "a".into()),
                ],
            },
        };
        let body = render_direct_url_json(&d);
        let md5 = body.find("\"md5\"").unwrap();
        let sha = body.find("\"sha256\"").unwrap();
        assert!(md5 < sha, "md5 must appear before sha256: {body}");
    }

    #[test]
    fn render_omits_subdirectory_when_none() {
        let d = vcs_sample();
        let body = render_direct_url_json(&d);
        assert!(!body.contains("subdirectory"));
    }

    #[test]
    fn render_includes_subdirectory_when_some() {
        let mut d = vcs_sample();
        d.subdirectory = Some("pkgs/foo".into());
        let body = render_direct_url_json(&d);
        assert!(body.contains("\"subdirectory\":\"pkgs/foo\""));
    }

    #[test]
    fn render_omits_vcs_fields_that_are_none() {
        let d = DirectUrlJson {
            url: "https://e.test/foo.git".into(),
            subdirectory: None,
            info: DirectUrlInfo::Vcs {
                vcs: "git".into(),
                requested_revision: None,
                commit_id: None,
                resolved_revision: None,
            },
        };
        let body = render_direct_url_json(&d);
        assert!(!body.contains("requested_revision"));
        assert!(!body.contains("commit_id"));
        assert!(!body.contains("resolved_revision"));
        assert!(body.contains("\"vcs\":\"git\""));
    }

    #[test]
    fn round_trip_archive() {
        let d = DirectUrlJson {
            url: "https://e.test/foo.tar.gz".into(),
            subdirectory: Some("sub".into()),
            info: DirectUrlInfo::Archive {
                hashes: vec![("sha256".into(), "abc".into())],
            },
        };
        let body = render_direct_url_json(&d);
        let parsed = parse_direct_url_json(&body).unwrap();
        assert_eq!(parsed, d);
    }

    #[test]
    fn round_trip_vcs() {
        let d = vcs_sample();
        let body = render_direct_url_json(&d);
        let parsed = parse_direct_url_json(&body).unwrap();
        assert_eq!(parsed, d);
    }

    #[test]
    fn round_trip_dir_editable() {
        let d = DirectUrlJson {
            url: "file:///abs/path".into(),
            subdirectory: None,
            info: DirectUrlInfo::Dir { editable: true },
        };
        let body = render_direct_url_json(&d);
        let parsed = parse_direct_url_json(&body).unwrap();
        assert_eq!(parsed, d);
    }

    // ---------- DirectUrl <-> DirectUrlJson bridge ----------

    #[test]
    fn from_direct_url_archive() {
        let url = DirectUrl::Archive {
            url: "https://e.test/x.tar.gz".into(),
            subdirectory: Some("s".into()),
        };
        let j = DirectUrlJson::from_direct_url(&url);
        assert_eq!(j.url, "https://e.test/x.tar.gz");
        assert_eq!(j.subdirectory.as_deref(), Some("s"));
        assert!(matches!(j.info, DirectUrlInfo::Archive { hashes } if hashes.is_empty()));
    }

    #[test]
    fn from_direct_url_git_uses_requested_revision() {
        let url = DirectUrl::Git {
            url: "https://github.com/x/y.git".into(),
            rev: Some("main".into()),
            subdirectory: None,
        };
        let j = DirectUrlJson::from_direct_url(&url);
        match j.info {
            DirectUrlInfo::Vcs {
                vcs,
                requested_revision,
                ..
            } => {
                assert_eq!(vcs, "git");
                assert_eq!(requested_revision.as_deref(), Some("main"));
            }
            other => panic!("expected Vcs, got {other:?}"),
        }
    }

    #[test]
    fn from_direct_url_localpath_wraps_in_file_url() {
        let url = DirectUrl::LocalPath {
            path: "/abs/path".into(),
            subdirectory: None,
        };
        let j = DirectUrlJson::from_direct_url(&url);
        assert_eq!(j.url, "file:///abs/path");
        assert!(matches!(j.info, DirectUrlInfo::Dir { editable: false }));
    }

    #[test]
    fn to_direct_url_round_trips_archive() {
        let original = DirectUrl::Archive {
            url: "https://e.test/x.tar.gz".into(),
            subdirectory: Some("s".into()),
        };
        let lowered = DirectUrlJson::from_direct_url(&original).to_direct_url();
        assert_eq!(lowered, original);
    }

    #[test]
    fn to_direct_url_round_trips_git() {
        let original = DirectUrl::Git {
            url: "https://github.com/x/y.git".into(),
            rev: Some("v1".into()),
            subdirectory: Some("pkgs/y".into()),
        };
        let lowered = DirectUrlJson::from_direct_url(&original).to_direct_url();
        assert_eq!(lowered, original);
    }

    #[test]
    fn to_direct_url_strips_file_url_prefix_for_localpath() {
        let original = DirectUrl::LocalPath {
            path: "/abs/path".into(),
            subdirectory: None,
        };
        let lowered = DirectUrlJson::from_direct_url(&original).to_direct_url();
        assert_eq!(lowered, original);
    }

    #[test]
    fn render_dir_info_includes_editable_field_explicitly() {
        let d = DirectUrlJson {
            url: "file:///x".into(),
            subdirectory: None,
            info: DirectUrlInfo::Dir { editable: false },
        };
        let body = render_direct_url_json(&d);
        assert!(body.contains("\"editable\":false"));
    }
}
