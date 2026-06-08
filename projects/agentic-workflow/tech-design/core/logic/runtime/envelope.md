---
id: projects-sdd-src-runtime-envelope-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: root-envelope-completion-contract
    claim: root-envelope-completion-contract
    coverage: full
    rationale: "Runtime envelope and session logic define the root-runner completion and HITL contract."
---

# Standardized projects/agentic-workflow/src/runtime/envelope.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/runtime/envelope.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Envelope` | projects/agentic-workflow/src/runtime/envelope.rs | enum | pub | 14 |  |
| `Invoke` | projects/agentic-workflow/src/runtime/envelope.rs | struct | pub | 40 |  |
| `parse` | projects/agentic-workflow/src/runtime/envelope.rs | function | pub | 47 | parse(raw: &str) -> serde_json::Result<Envelope> |
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/envelope.rs -->
```rust
//! SDD CLI envelope protocol — shared by all frontends (score, cue, conductor).
//!
//! Mirrors the schema in `projects/agentic-workflow/tech-design/surface/specs/issue-cli-envelope.md`.

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
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/runtime/envelope.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template for the shared SDD CLI envelope protocol, invoke payload,
      parser helper, and protocol unit tests.
```
