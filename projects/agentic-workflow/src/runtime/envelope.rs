// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/envelope.md#source
// CODEGEN-BEGIN
//! SDD CLI envelope protocol — shared by all frontends (score, cue, conductor).
//!
//! Mirrors the schema in `projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md`.

use crate::models::artifact_quality::ArtifactQualityProfile;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "lowercase")]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/envelope.md#source
pub enum Envelope {
    Dispatch {
        #[serde(default)]
        agent: Option<String>,
        slug: String,
        #[serde(default)]
        invoke: Option<Invoke>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        artifact_quality_profile: Option<ArtifactQualityProfile>,
    },
    Done {
        slug: String,
        #[serde(default)]
        message: Option<String>,
    },
    Error {
        slug: String,
        message: String,
    },
    Batch {
        recommendations: Vec<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/envelope.md#source
pub struct Invoke {
    pub command: String,
    #[serde(default)]
    pub args: Value,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/envelope.md#source
pub fn parse(raw: &str) -> serde_json::Result<Envelope> {
    serde_json::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dispatch_author() {
        let raw = r#"{"action":"dispatch","agent":"score-issue-author","slug":"foo","invoke":{"command":"aw wi author","args":{"slug":"foo","section":"requirements"}}}"#;
        let env = parse(raw).unwrap();
        match env {
            Envelope::Dispatch {
                agent,
                slug,
                invoke,
                ..
            } => {
                assert_eq!(agent.as_deref(), Some("score-issue-author"));
                assert_eq!(slug, "foo");
                let inv = invoke.unwrap();
                assert_eq!(inv.command, "aw wi author");
            }
            _ => panic!("expected Dispatch"),
        }
    }

    #[test]
    fn parses_done() {
        let raw = r#"{"action":"done","slug":"baz","message":"ok"}"#;
        let env = parse(raw).unwrap();
        match env {
            Envelope::Done { slug, message } => {
                assert_eq!(slug, "baz");
                assert_eq!(message.as_deref(), Some("ok"));
            }
            _ => panic!("expected Done"),
        }
    }

    #[test]
    fn parses_error() {
        let raw = r#"{"action":"error","slug":"qux","message":"boom"}"#;
        let env = parse(raw).unwrap();
        match env {
            Envelope::Error { slug, message } => {
                assert_eq!(slug, "qux");
                assert_eq!(message, "boom");
            }
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn parses_batch() {
        let raw = r#"{"action":"batch","recommendations":[{"kind":"skip","slug":"a"}]}"#;
        let env = parse(raw).unwrap();
        match env {
            Envelope::Batch { recommendations } => {
                assert_eq!(recommendations.len(), 1);
            }
            _ => panic!("expected Batch"),
        }
    }

    #[test]
    fn envelope_profile_deserializes_without_profile() {
        let raw =
            r#"{"action":"dispatch","slug":"foo","invoke":{"command":"aw td create","args":{}}}"#;
        let env = parse(raw).unwrap();
        match env {
            Envelope::Dispatch {
                artifact_quality_profile,
                ..
            } => assert!(artifact_quality_profile.is_none()),
            _ => panic!("expected Dispatch"),
        }
    }

    #[test]
    fn envelope_profile_roundtrips_when_present() {
        let profile = ArtifactQualityProfile::default_for_kind(
            crate::models::artifact_quality::ArtifactKind::CliSurface,
        );
        let env = Envelope::Dispatch {
            agent: None,
            slug: "3903".to_string(),
            invoke: Some(Invoke {
                command: "aw run".to_string(),
                args: serde_json::json!({ "wi": "3903" }),
            }),
            artifact_quality_profile: Some(profile.clone()),
        };
        let encoded = serde_json::to_string(&env).unwrap();
        let decoded = parse(&encoded).unwrap();
        match decoded {
            Envelope::Dispatch {
                artifact_quality_profile,
                ..
            } => assert_eq!(artifact_quality_profile, Some(profile)),
            _ => panic!("expected Dispatch"),
        }
    }
}

// CODEGEN-END
