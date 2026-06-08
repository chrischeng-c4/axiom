// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
// CODEGEN-BEGIN
//! Mainthread agent — the orchestrator that translates dev intent
//! into lifecycle actions.
//!
//! Why a separate role: per-phase agents (Author / Reviewer / Reviser)
//! are workers triggered by score dispatch envelopes. The
//! **mainthread agent** is what the dev actually talks to: it receives
//! free-form chat input, decides what to do, and dispatches the
//! per-phase workers. Without it, cue can only "submit title → run
//! lifecycle" — no conversation, no override, no status reply.
//!
//! ## Wire protocol
//!
//! The mainthread LLM turn produces a JSON object on stdout:
//!
//! ```json
//! {"action": "new_issue", "title": "metrics dashboard"}
//! {"action": "reply", "content": "still drafting Requirements..."}
//! ```
//!
//! `parse_decision` extracts the first `{...}` JSON object from the
//! assistant's free-form text — tolerates preamble / postamble (real
//! LLMs sometimes wrap JSON in prose). For the mock provider in tests,
//! the canned response is just the raw JSON.
//!
//! Slice 1 ships two variants (`NewIssue`, `Reply`); future variants
//! land alongside the corresponding runner / app handlers:
//!   - `ContextInject { text }` — augment current Author turn's prompt
//!   - `Override { decision }`  — bypass reviewer verdict
//!   - `Pause`                  — suspend in-flight lifecycle
//!   - `Status`                 — render current phase / progress

use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#schema
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#logic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum MainthreadDecision {
    /// Dev asked for a new issue. `title` becomes the aw binary
    /// `create` arg; the lifecycle loop runs after.
    NewIssue { title: String },
    /// Dev asked something the mainthread can answer in chat without
    /// touching the lifecycle (status / question / clarification).
    Reply { content: String },
}

/// Extract the first JSON object from `text` and parse it as a
/// `MainthreadDecision`. Returns `None` if no parseable JSON is found —
/// caller surfaces this as a `SessionEvent::Error` so the dev sees the
/// raw output and can recover.
///
/// Handles three formats real LLMs produce:
///   1. raw JSON (mock fixture path): `{"action":"reply",...}`
///   2. JSON wrapped in prose: `Sure! {"action":"new_issue",...} done.`
///   3. JSON in a fenced code block: ` ```json\n{...}\n``` `
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub fn parse_decision(text: &str) -> Option<MainthreadDecision> {
    let stripped = strip_fence(text).unwrap_or(text);
    let start = stripped.find('{')?;
    // Find matching closing brace by depth-tracking — `text.rfind('}')`
    // would over-match if the JSON has nested objects followed by trailing
    // prose with another `}`.
    let bytes = stripped.as_bytes();
    let mut depth = 0i32;
    let mut end = None;
    for (i, b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    serde_json::from_str(&stripped[start..=end]).ok()
}

fn strip_fence(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let inner = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))?;
    let body = inner.strip_suffix("```")?;
    Some(body.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_raw_new_issue_json() {
        let d = parse_decision(r#"{"action":"new_issue","title":"foo"}"#).unwrap();
        assert_eq!(
            d,
            MainthreadDecision::NewIssue {
                title: "foo".into()
            }
        );
    }

    #[test]
    fn parses_raw_reply_json() {
        let d = parse_decision(r#"{"action":"reply","content":"hello"}"#).unwrap();
        assert_eq!(
            d,
            MainthreadDecision::Reply {
                content: "hello".into()
            }
        );
    }

    #[test]
    fn parses_json_with_preamble_and_postamble() {
        let body = r#"Sure thing! {"action":"new_issue","title":"x"} Let me know."#;
        let d = parse_decision(body).unwrap();
        assert_eq!(d, MainthreadDecision::NewIssue { title: "x".into() });
    }

    #[test]
    fn parses_fenced_json_code_block() {
        let body = "```json\n{\"action\":\"reply\",\"content\":\"hi\"}\n```";
        let d = parse_decision(body).unwrap();
        assert_eq!(
            d,
            MainthreadDecision::Reply {
                content: "hi".into()
            }
        );
    }

    #[test]
    fn no_json_returns_none() {
        assert!(parse_decision("just plain text").is_none());
    }

    #[test]
    fn nested_json_picks_outer_object() {
        let body = r#"{"action":"reply","content":"see {nested:true}"}"#;
        let d = parse_decision(body).unwrap();
        assert_eq!(
            d,
            MainthreadDecision::Reply {
                content: "see {nested:true}".into()
            }
        );
    }
}

// CODEGEN-END
