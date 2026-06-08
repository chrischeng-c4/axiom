// pep740.rs — PEP 740 attestation envelope decoder.
//
// PEP 740 ("Index support for digital attestations") lets a Simple
// Repository index serve a JSON document of cryptographic
// attestations alongside every release file. The document is keyed
// off the file's URL with a `.publish-attestations.v1` suffix or
// surfaced via PEP 691 JSON as the `attestations` array on each
// file record.
//
// Wire shape (PEP 740 §"The provenance format"):
//
//   {
//     "version": 1,
//     "attestation_bundles": [
//       {
//         "publisher": {
//           "kind": "GitHub",
//           "repository": "owner/repo",
//           "workflow": "release.yml",
//           "environment": "pypi"
//         },
//         "attestations": [
//           {
//             "version": 1,
//             "verification_material": { "certificate": "...", "transparency_entries": [...] },
//             "envelope": { "statement": "<base64>", "signature": "<base64>" }
//           }
//         ]
//       }
//     ]
//   }
//
// This module covers the **structural decoding** only — i.e., it
// surfaces the publisher identity + attestation payloads so the
// caller can hand them to a Sigstore verifier (well outside the
// scope of mamba's package-manager subsystem). Cryptographic
// verification needs `sigstore` / a TUF root, both of which we
// intentionally don't pull in.

use serde::{Deserialize, Serialize};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Top-level provenance document. `version` is currently `1` in
/// the spec; future versions will need a separate parser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceDocument {
    pub version: u32,
    #[serde(default)]
    pub attestation_bundles: Vec<AttestationBundle>,
}

/// One publisher + attestations group. A file may have multiple
/// bundles if it was published by more than one trusted workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationBundle {
    pub publisher: Publisher,
    #[serde(default)]
    pub attestations: Vec<Attestation>,
}

/// Identity of the entity that published the file. PEP 740
/// currently enumerates `GitHub`, `GitLab`, `Google`, `ActiveState`
/// but other values are allowed (forward-compatibility); we keep
/// the typed shape for the four named ones and let other publishers
/// pass through as `Other`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Publisher {
    #[serde(rename = "GitHub")]
    GitHub {
        repository: String,
        workflow: String,
        #[serde(default)]
        environment: Option<String>,
    },
    #[serde(rename = "GitLab")]
    GitLab {
        repository: String,
        workflow_filepath: String,
        #[serde(default)]
        environment: Option<String>,
    },
    #[serde(rename = "Google")]
    Google {
        email: String,
    },
    #[serde(rename = "ActiveState")]
    ActiveState {
        organization: String,
        project: String,
        actor: String,
    },
    /// Forward-compat catch-all for unknown publisher kinds. We
    /// preserve the raw `kind` string so callers can surface it in
    /// `mamba tree` / advisory output.
    #[serde(untagged)]
    Other(serde_json::Value),
}

/// One DSSE-shaped attestation. The `envelope` is the signed
/// payload + signature; `verification_material` carries the X.509
/// certificate + Sigstore transparency log entries needed to
/// validate it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    pub version: u32,
    pub verification_material: VerificationMaterial,
    pub envelope: Envelope,
}

/// X.509 certificate + transparency log entries. We keep both as
/// opaque structures: the cert is a base64 DER blob, the
/// transparency entries are nested objects whose internal shape
/// only matters to a Sigstore verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationMaterial {
    pub certificate: String,
    #[serde(default)]
    pub transparency_entries: Vec<serde_json::Value>,
}

/// DSSE envelope. `statement` is base64-encoded in-toto v0.1
/// statement bytes; `signature` is base64 raw signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    pub statement: String,
    pub signature: String,
}

impl ProvenanceDocument {
    /// True iff the document declares zero attestation bundles.
    /// Surface this so callers don't have to dig into the
    /// `attestation_bundles` field directly.
    pub fn is_empty(&self) -> bool {
        self.attestation_bundles.is_empty()
    }

    /// Total number of `Attestation` records across all bundles.
    /// Used by display layers ("3 attestations from 2 publishers").
    pub fn attestation_count(&self) -> usize {
        self.attestation_bundles
            .iter()
            .map(|b| b.attestations.len())
            .sum()
    }
}

impl Publisher {
    /// Display the publisher's kind as a stable string for logs.
    /// Returns the JSON `kind` value for the named variants and
    /// `"Other"` for the catch-all.
    pub fn kind_label(&self) -> &str {
        match self {
            Publisher::GitHub { .. } => "GitHub",
            Publisher::GitLab { .. } => "GitLab",
            Publisher::Google { .. } => "Google",
            Publisher::ActiveState { .. } => "ActiveState",
            Publisher::Other(_) => "Other",
        }
    }
}

/// Decode a provenance document from its JSON serialization. The
/// document MUST have `version == 1`; future versions need a new
/// parser and are rejected here with an explicit error so the
/// caller can surface a "upgrade mamba" hint instead of silently
/// treating the file as unsigned.
pub fn parse_provenance(src: &str) -> Result<ProvenanceDocument, IndexError> {
    let doc: ProvenanceDocument =
        serde_json::from_str(src).map_err(|e| IndexError::ParseError {
            url: String::new(),
            detail: format!("PEP 740 provenance JSON: {e}"),
        })?;
    if doc.version != 1 {
        return Err(IndexError::ParseError {
            url: String::new(),
            detail: format!(
                "PEP 740 provenance version {} is not supported (expected 1)",
                doc.version
            ),
        });
    }
    Ok(doc)
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

    fn sample_envelope() -> Envelope {
        Envelope {
            statement: "eyJfdHlwZSI6ICJodHRwczovL2luLXRvdG8uaW8vU3RhdGVtZW50L3YwLjEifQ=="
                .into(),
            signature: "MEUCIQD6Hxx7TYY=".into(),
        }
    }

    fn sample_verification() -> VerificationMaterial {
        VerificationMaterial {
            certificate: "MIIDXTCCAkWgAwIBAgIJAJC1HiIAZAi2MA0=".into(),
            transparency_entries: vec![serde_json::json!({
                "logIndex": 12345,
                "logId": { "keyId": "abc" }
            })],
        }
    }

    #[test]
    fn parse_minimal_empty_document() {
        let src = r#"{"version": 1}"#;
        let doc = parse_provenance(src).unwrap();
        assert_eq!(doc.version, 1);
        assert!(doc.is_empty());
        assert_eq!(doc.attestation_count(), 0);
    }

    #[test]
    fn parse_github_publisher_with_one_attestation() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": {
                    "kind": "GitHub",
                    "repository": "psf/requests",
                    "workflow": "release.yml",
                    "environment": "pypi"
                },
                "attestations": [{
                    "version": 1,
                    "verification_material": {
                        "certificate": "MIIDXTCC=",
                        "transparency_entries": []
                    },
                    "envelope": {
                        "statement": "stmt",
                        "signature": "sig"
                    }
                }]
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        assert_eq!(doc.attestation_bundles.len(), 1);
        let bundle = &doc.attestation_bundles[0];
        match &bundle.publisher {
            Publisher::GitHub { repository, workflow, environment } => {
                assert_eq!(repository, "psf/requests");
                assert_eq!(workflow, "release.yml");
                assert_eq!(environment.as_deref(), Some("pypi"));
            }
            other => panic!("expected GitHub publisher, got {other:?}"),
        }
        assert_eq!(bundle.attestations.len(), 1);
        assert_eq!(doc.attestation_count(), 1);
    }

    #[test]
    fn parse_github_publisher_without_environment() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": {
                    "kind": "GitHub",
                    "repository": "x/y",
                    "workflow": "ci.yml"
                },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::GitHub { environment, .. } => assert!(environment.is_none()),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_gitlab_publisher() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": {
                    "kind": "GitLab",
                    "repository": "group/sub/project",
                    "workflow_filepath": ".gitlab-ci.yml"
                },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::GitLab { repository, workflow_filepath, environment } => {
                assert_eq!(repository, "group/sub/project");
                assert_eq!(workflow_filepath, ".gitlab-ci.yml");
                assert!(environment.is_none());
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_google_publisher() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": { "kind": "Google", "email": "ci@my-project.iam" },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::Google { email } => assert_eq!(email, "ci@my-project.iam"),
            _ => panic!(),
        }
    }

    #[test]
    fn parse_activestate_publisher() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": {
                    "kind": "ActiveState",
                    "organization": "Acme",
                    "project": "widget",
                    "actor": "build-bot"
                },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::ActiveState { organization, project, actor } => {
                assert_eq!(organization, "Acme");
                assert_eq!(project, "widget");
                assert_eq!(actor, "build-bot");
            }
            _ => panic!(),
        }
    }

    #[test]
    fn parse_unknown_publisher_kind_passes_through() {
        // Future publisher kinds should not break older mambas.
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": { "kind": "Forge", "url": "https://forge.example" },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::Other(v) => {
                assert_eq!(v["kind"], "Forge");
                assert_eq!(v["url"], "https://forge.example");
            }
            other => panic!("expected Other, got {other:?}"),
        }
    }

    #[test]
    fn multiple_bundles_and_attestations() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [
                {
                    "publisher": { "kind": "GitHub", "repository": "a/b", "workflow": "w.yml" },
                    "attestations": [
                        {"version":1,"verification_material":{"certificate":"c1"},"envelope":{"statement":"s1","signature":"g1"}},
                        {"version":1,"verification_material":{"certificate":"c2"},"envelope":{"statement":"s2","signature":"g2"}}
                    ]
                },
                {
                    "publisher": { "kind": "GitLab", "repository": "x/y", "workflow_filepath": "ci.yml" },
                    "attestations": [
                        {"version":1,"verification_material":{"certificate":"c3"},"envelope":{"statement":"s3","signature":"g3"}}
                    ]
                }
            ]
        }"#;
        let doc = parse_provenance(src).unwrap();
        assert_eq!(doc.attestation_bundles.len(), 2);
        assert_eq!(doc.attestation_count(), 3);
    }

    #[test]
    fn rejects_unsupported_version() {
        let src = r#"{"version": 2}"#;
        let err = parse_provenance(src).unwrap_err();
        let detail = err_detail(err);
        assert!(detail.contains("version 2"));
        assert!(detail.contains("not supported"));
    }

    #[test]
    fn rejects_version_zero() {
        let src = r#"{"version": 0}"#;
        let err = parse_provenance(src).unwrap_err();
        assert!(err_detail(err).contains("version 0"));
    }

    #[test]
    fn rejects_invalid_json() {
        let err = parse_provenance("not json").unwrap_err();
        assert!(err_detail(err).contains("provenance JSON"));
    }

    #[test]
    fn rejects_missing_version_field() {
        let src = r#"{"attestation_bundles": []}"#;
        let err = parse_provenance(src).unwrap_err();
        assert!(err_detail(err).contains("provenance JSON"));
    }

    #[test]
    fn github_missing_repository_falls_to_other() {
        // The `#[serde(untagged)] Other(Value)` catch-all means a
        // malformed-known-kind publisher does NOT explode the whole
        // document — it degrades into the opaque variant so
        // older mamba builds keep working. The display layer can
        // still surface the raw `kind` for the user.
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": { "kind": "GitHub", "workflow": "w.yml" },
                "attestations": []
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        match &doc.attestation_bundles[0].publisher {
            Publisher::Other(v) => {
                assert_eq!(v["kind"], "GitHub");
                assert!(v.get("repository").is_none());
            }
            other => panic!("expected Other fallback, got {other:?}"),
        }
    }

    #[test]
    fn transparency_entries_default_to_empty() {
        let src = r#"{
            "version": 1,
            "attestation_bundles": [{
                "publisher": { "kind": "GitHub", "repository": "a/b", "workflow": "w.yml" },
                "attestations": [{
                    "version": 1,
                    "verification_material": { "certificate": "c" },
                    "envelope": { "statement": "s", "signature": "g" }
                }]
            }]
        }"#;
        let doc = parse_provenance(src).unwrap();
        let att = &doc.attestation_bundles[0].attestations[0];
        assert!(att.verification_material.transparency_entries.is_empty());
    }

    // ---- helpers ------------------------------------------------------

    #[test]
    fn kind_label_for_each_variant() {
        let github = Publisher::GitHub {
            repository: "a/b".into(),
            workflow: "w".into(),
            environment: None,
        };
        let gitlab = Publisher::GitLab {
            repository: "a/b".into(),
            workflow_filepath: ".gitlab-ci.yml".into(),
            environment: None,
        };
        let google = Publisher::Google { email: "a@b".into() };
        let active = Publisher::ActiveState {
            organization: "o".into(),
            project: "p".into(),
            actor: "a".into(),
        };
        let other = Publisher::Other(serde_json::json!({"kind": "Forge"}));

        assert_eq!(github.kind_label(), "GitHub");
        assert_eq!(gitlab.kind_label(), "GitLab");
        assert_eq!(google.kind_label(), "Google");
        assert_eq!(active.kind_label(), "ActiveState");
        assert_eq!(other.kind_label(), "Other");
    }

    #[test]
    fn attestation_count_with_zero_bundles() {
        let doc = ProvenanceDocument {
            version: 1,
            attestation_bundles: vec![],
        };
        assert_eq!(doc.attestation_count(), 0);
        assert!(doc.is_empty());
    }

    #[test]
    fn attestation_count_with_empty_bundle() {
        let doc = ProvenanceDocument {
            version: 1,
            attestation_bundles: vec![AttestationBundle {
                publisher: Publisher::Google { email: "a@b".into() },
                attestations: vec![],
            }],
        };
        assert_eq!(doc.attestation_count(), 0);
        assert!(!doc.is_empty());
    }

    // ---- serde round-trip ---------------------------------------------

    #[test]
    fn serde_roundtrip_preserves_full_document() {
        let doc = ProvenanceDocument {
            version: 1,
            attestation_bundles: vec![AttestationBundle {
                publisher: Publisher::GitHub {
                    repository: "psf/requests".into(),
                    workflow: "release.yml".into(),
                    environment: Some("pypi".into()),
                },
                attestations: vec![Attestation {
                    version: 1,
                    verification_material: sample_verification(),
                    envelope: sample_envelope(),
                }],
            }],
        };
        let serialized = serde_json::to_string(&doc).unwrap();
        // Re-parse via the public parser (exercises the version
        // guard too).
        let back = parse_provenance(&serialized).unwrap();
        assert_eq!(back, doc);
    }

    #[test]
    fn realistic_pypi_provenance_record() {
        // Faithful reproduction of the PyPI provenance JSON for a
        // single Trusted Publisher GitHub upload.
        let src = r#"{
            "version": 1,
            "attestation_bundles": [
                {
                    "publisher": {
                        "kind": "GitHub",
                        "repository": "trustypub/example",
                        "workflow": "release.yml",
                        "environment": "release"
                    },
                    "attestations": [
                        {
                            "version": 1,
                            "verification_material": {
                                "certificate": "MIIDXTCCAkWgAwIBAgIJAJC1HiIAZAi2MA0=",
                                "transparency_entries": [
                                    {
                                        "logIndex": "12345",
                                        "logId": {"keyId": "wNI9atQGlz+VWfO6LRygH4QUfY/8W4RFwiT5i5WRgB0="},
                                        "kindVersion": {"kind": "hashedrekord", "version": "0.0.1"},
                                        "integratedTime": "1700000000"
                                    }
                                ]
                            },
                            "envelope": {
                                "statement": "eyJfdHlwZSI6ICJodHRwczovL2luLXRvdG8uaW8vU3RhdGVtZW50L3YwLjEifQ==",
                                "signature": "MEUCIQD6Hxx7TYY="
                            }
                        }
                    ]
                }
            ]
        }"#;
        let doc = parse_provenance(src).unwrap();
        assert_eq!(doc.attestation_count(), 1);
        let bundle = &doc.attestation_bundles[0];
        assert_eq!(bundle.publisher.kind_label(), "GitHub");
        let att = &bundle.attestations[0];
        assert_eq!(att.version, 1);
        assert_eq!(att.verification_material.transparency_entries.len(), 1);
        assert!(att.envelope.statement.starts_with("eyJ"));
    }
}
