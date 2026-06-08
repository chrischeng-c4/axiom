// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
// CODEGEN-BEGIN
//! Session-level events that flow from `Session::create_issue` / `Session::turn`
//! to consumers (TUI, web, CLI).
//!
//! Slice 1 ships a subset; richer variants (SubagentStart/Stop, PhaseChanged,
//! Refusal, Approval) land alongside `aw wi validate` integration.

use crate::runtime::envelope::Envelope;
use crate::runtime::mainthread::MainthreadDecision;
use serde::{Deserialize, Serialize};

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#schema
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TurnId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub enum SessionEvent {
    UserMessage {
        content: String,
    },
    /// Emitted right before an LLM turn opens. Carries the SDD agent role
    /// (`score-issue-author` / `-reviewer` / `-reviser`) and the routed
    /// model name so the TUI can open a correctly-labeled chat bubble
    /// before any deltas arrive.
    TurnStart {
        role: String,
        model: String,
    },
    AssistantDelta {
        content: String,
    },
    AssistantMessageComplete {
        content: String,
    },
    ToolUse {
        name: String,
        args: serde_json::Value,
    },
    ToolResult {
        name: String,
        output: serde_json::Value,
    },
    Envelope(Envelope),
    /// Mainthread agent's parsed structured decision. Emitted after
    /// the LLM turn completes and `parse_decision` succeeds. Drives the
    /// runner's choice between dispatching a lifecycle (NewIssue) or
    /// just leaving the assistant bubble in place (Reply).
    MainthreadDecision {
        decision: MainthreadDecision,
    },
    Error {
        message: String,
    },
}

// CODEGEN-END
