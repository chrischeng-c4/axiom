---
id: sdd-runtime-mainthread
fill_sections: [overview, schema, scenarios, source, changes]
capability_refs:
  - id: workflow-root-runner
    role: primary
    gap: root-envelope-completion-contract
    claim: root-envelope-completion-contract
    coverage: full
    rationale: "Runtime envelope and session logic define the root-runner completion and HITL contract."
---

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for 3 target files generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `SessionEvent` | projects/agentic-workflow/src/runtime/event.rs | enum | pub | 21 |  |
| `TurnId` | projects/agentic-workflow/src/runtime/event.rs | struct | pub | 16 |  |
| `MainthreadDecision` | projects/agentic-workflow/src/runtime/mainthread.rs | enum | pub | 40 |  |
| `parse_decision` | projects/agentic-workflow/src/runtime/mainthread.rs | function | pub | 59 | parse_decision(text: &str) -> Option<MainthreadDecision> |
| `ModelChoice` | projects/agentic-workflow/src/runtime/router.rs | struct | pub | 17 |  |
| `StaticRouter` | projects/agentic-workflow/src/runtime/router.rs | struct | pub | 59 |  |
| `Task` | projects/agentic-workflow/src/runtime/router.rs | enum | pub | 24 |  |
| `as_str` | projects/agentic-workflow/src/runtime/router.rs | function | pub | 38 | as_str(self) -> &'static str |
| `defaults` | projects/agentic-workflow/src/runtime/router.rs | function | pub | 68 | defaults() -> Self |
| `empty` | projects/agentic-workflow/src/runtime/router.rs | function | pub | 101 | empty() -> Self |
| `with_route` | projects/agentic-workflow/src/runtime/router.rs | function | pub | 107 | with_route(mut self, task: Task, choice: ModelChoice) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
$schema: "https://json-schema.org/draft/2020-12/schema"
$id: sdd-runtime-mainthread-schema

definitions:
  MainthreadDecision:
    description: >
      Wire-protocol discriminated union emitted by the mainthread LLM turn.
      The `action` field is the tag. Slice 1 ships two variants; future
      variants (ContextInject, Override, Pause, Status) extend this oneOf
      without changing the parse_decision contract.
    type: object
    required: [action]
    oneOf:
      - title: NewIssue
        description: >
          Dev wants to start a new SDD issue. `title` is passed verbatim to
          `aw wi create`. After the slug is returned the runtime drives
          the full lifecycle loop without waiting for additional dev input.
        properties:
          action:
            type: string
            const: new_issue
          title:
            type: string
            description: Short, slug-friendly issue title.
            minLength: 1
        required: [action, title]
        additionalProperties: false

      - title: Reply
        description: >
          Dev asked a question, requested status, or made a comment that does
          not start a new lifecycle. The runtime surfaces `content` in the chat
          bubble and performs no score dispatch.
        properties:
          action:
            type: string
            const: reply
          content:
            type: string
            description: Human-readable message to display in the chat bubble.
            minLength: 1
        required: [action, content]
        additionalProperties: false

  Task:
    description: >
      Enumeration of LLM task roles used by StaticRouter / ConfigRouter.
      `Mainthread` is the orchestrator role added in Slice 1; it carries the
      highest model-strength requirement because a mis-classified intent makes
      the product feel broken. The default route targets claude-opus-4-7.
    type: string
    enum: [mainthread, author, review, revise]
    x-rust-variants:
      mainthread:
        description: >
          Conversational orchestrator. Routes to
          provider=anthropic, model=claude-opus-4-7 by default.
      author:
        description: >
          Per-phase Requirements/Scope/ReferenceContext authoring.
          Routes to provider=gemini, model=gemini-2.5-pro by default.
      review:
        description: >
          Section reviewer. Routes to provider=openai, model=gpt-5 by default.
      revise:
        description: >
          Section reviser after needs-revision verdict.
          Routes to provider=anthropic, model=claude-opus-4-7 by default.

  SessionEventMainthreadDecision:
    description: >
      SessionEvent variant emitted after every mainthread LLM turn in which
      parse_decision succeeds. The TUI runner maps this to
      Action::MainthreadDecided so App::apply can rewrite the raw JSON bubble
      to a human-readable string before the user sees it.
    type: object
    required: [kind, decision]
    properties:
      kind:
        type: string
        const: mainthread_decision
      decision:
        $ref: "#/definitions/MainthreadDecision"
    additionalProperties: false
```
## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: new_issue_path
    title: Developer submits a feature request — mainthread classifies as new_issue
    description: >
      When a dev types free-form text that describes a feature or bug,
      the mainthread LLM turn classifies it as new_issue and drives the
      full SDD lifecycle loop autonomously. The first LLM call uses the
      mainthread model (claude-opus-4-7); subsequent per-phase calls use
      the task-specific routed models.
    given:
      - The Session is initialised with a StaticRouter using defaults()
      - The LLM provider mock returns '{"action":"new_issue","title":"metrics dashboard"}'
        for the mainthread turn
    when:
      - Action::SubmitChat("add a metrics dashboard to the dev sidebar") is dispatched
      - runner.rs delegates to Session::decide(user_input)
    then:
      - SessionEvent::TurnStart { role: "mainthread", model: "claude-opus-4-7" } is emitted
      - parse_decision extracts MainthreadDecision::NewIssue { title: "metrics dashboard" }
      - SessionEvent::MainthreadDecision { decision: NewIssue { .. } } is emitted
      - Action::MainthreadDecided { decision: NewIssue { .. } } reaches App::apply
      - App::apply rewrites the JSON bubble to "Creating issue: metrics dashboard"
      - run_create_issue is called with title="metrics dashboard"
      - aw wi create is invoked and returns a slug
      - drive_lifecycle_loop runs Author / Reviewer / Reviser phases without further dev input
    acceptance:
      - test: e2e_lifecycle.rs::slice1_step_through_create_then_fill_section
      - assertion: first llm call model starts_with("claude-")
      - assertion: subsequent phase models match per-task router defaults

  - id: reply_path
    title: Developer asks a status question — mainthread classifies as reply
    description: >
      When a dev types a question or comment that does not start a new
      lifecycle, the mainthread LLM classifies it as reply and returns
      the content as a chat bubble. No score call is made and the
      lifecycle state remains Idle.
    given:
      - The Session is initialised with a StaticRouter using defaults()
      - The LLM provider mock returns '{"action":"reply","content":"still drafting Requirements"}'
        for the mainthread turn
    when:
      - Action::SubmitChat("are we still revising?") is dispatched
      - runner.rs delegates to Session::decide(user_input)
    then:
      - SessionEvent::TurnStart { role: "mainthread", model: "claude-opus-4-7" } is emitted
      - parse_decision extracts MainthreadDecision::Reply { content: "still drafting Requirements" }
      - SessionEvent::MainthreadDecision { decision: Reply { .. } } is emitted
      - Action::MainthreadDecided { decision: Reply { .. } } reaches App::apply
      - App::apply rewrites the JSON bubble to "still drafting Requirements"
      - score_calls() is empty — no score dispatch occurred
      - lifecycle_state remains Idle
    acceptance:
      - test: e2e_lifecycle_ux.rs::scenario_mainthread_reply_no_lifecycle
      - assertion: score_calls().is_empty()
      - assertion: lifecycle_state == Idle
```
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/event.rs -->
```rust
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
```

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/mainthread.rs -->
````rust
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
````

<!-- source-snapshot: path=projects/agentic-workflow/src/runtime/router.rs -->
```rust
//! Model routing — which `(provider, model)` to use for a given task type.
//!
//! Slice 1 ships a `StaticRouter` (in-memory map) sufficient for tests and
//! a default routing table aligned with the team's per-task model strengths
//! (Gemini for authoring, GPT for review, Claude for revise/default).
//! Cue's `.cue/config.toml` lookup wraps this trait via its own impl.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#schema
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelChoice {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub enum Task {
    /// Conversational orchestrator — receives free-form dev input,
    /// decides intent (new issue / reply / pause / override), and
    /// dispatches the per-phase agents. The dev's actual chat
    /// counterpart. **Highest model-strength requirement** because a
    /// mis-classified intent makes the product feel broken.
    Mainthread,
    Author,
    Review,
    Revise,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl Task {
    pub fn as_str(self) -> &'static str {
        match self {
            Task::Mainthread => "mainthread",
            Task::Author => "author",
            Task::Review => "review",
            Task::Revise => "revise",
        }
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub trait ModelRouter: Send + Sync {
    /// Resolve the `(provider, model)` for the given task.
    /// Returns None if the router has no route configured.
    async fn route(&self, task: Task) -> Option<ModelChoice>;
}

/// In-memory router used by tests and as a sensible default. Production code
/// (cue TUI, conductor) wraps `ConfigRouter` (in cue::config) over user TOML.
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
pub struct StaticRouter {
    table: BTreeMap<&'static str, ModelChoice>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl StaticRouter {
    /// Default routing aligned with the team's per-task strengths memo:
    ///   mainthread → Claude (orchestration / conversational),
    ///   author → Gemini, review → GPT, revise → Claude.
    pub fn defaults() -> Self {
        let mut table = BTreeMap::new();
        table.insert(
            "mainthread",
            ModelChoice {
                provider: "anthropic".into(),
                model: "claude-opus-4-7".into(),
            },
        );
        table.insert(
            "author",
            ModelChoice {
                provider: "gemini".into(),
                model: "gemini-2.5-pro".into(),
            },
        );
        table.insert(
            "review",
            ModelChoice {
                provider: "openai".into(),
                model: "gpt-5".into(),
            },
        );
        table.insert(
            "revise",
            ModelChoice {
                provider: "anthropic".into(),
                model: "claude-opus-4-7".into(),
            },
        );
        Self { table }
    }

    pub fn empty() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }

    pub fn with_route(mut self, task: Task, choice: ModelChoice) -> Self {
        self.table.insert(task.as_str(), choice);
        self
    }
}

#[async_trait]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#source
impl ModelRouter for StaticRouter {
    async fn route(&self, task: Task) -> Option<ModelChoice> {
        self.table.get(task.as_str()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn defaults_route_author_to_gemini() {
        let r = StaticRouter::defaults();
        let choice = r.route(Task::Author).await.unwrap();
        assert_eq!(choice.provider, "gemini");
        assert!(choice.model.starts_with("gemini-"));
    }

    #[tokio::test]
    async fn defaults_route_review_to_openai() {
        let r = StaticRouter::defaults();
        let choice = r.route(Task::Review).await.unwrap();
        assert_eq!(choice.provider, "openai");
    }

    #[tokio::test]
    async fn defaults_route_revise_to_anthropic() {
        let r = StaticRouter::defaults();
        let choice = r.route(Task::Revise).await.unwrap();
        assert_eq!(choice.provider, "anthropic");
    }

    #[tokio::test]
    async fn empty_returns_none() {
        let r = StaticRouter::empty();
        assert!(r.route(Task::Author).await.is_none());
    }

    #[tokio::test]
    async fn with_route_overrides() {
        let r = StaticRouter::empty().with_route(
            Task::Author,
            ModelChoice {
                provider: "p".into(),
                model: "m".into(),
            },
        );
        assert_eq!(r.route(Task::Author).await.unwrap().model, "m");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
# Cue ratatui entries remain impl_mode: hand-written — cue ratatui codegen gap.
# Commit: f01d776e7 on the cue branch.
changes:
  - path: projects/agentic-workflow/src/runtime/mainthread.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Defines MainthreadDecision (NewIssue | Reply) tagged
      enum with serde snake_case discriminant. Implements parse_decision
      to extract the first JSON object from LLM output, tolerating raw
      JSON, prose-wrapped JSON, and fenced code blocks. Ships 6 unit
      tests covering all three input shapes for both variants.

  - path: projects/agentic-workflow/src/runtime/router.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for ModelChoice, Task, ModelRouter, and StaticRouter,
      including the Task::Mainthread default route to the strongest
      conversational model slot.

  - path: projects/agentic-workflow/src/runtime/event.rs
    action: modify
    section: source
    impl_mode: codegen
    description: >
      Source template for TurnId and SessionEvent, including the
      MainthreadDecision event variant emitted after parse_decision succeeds.
      TUI runner translates this event to Action::MainthreadDecided.

  - path: projects/agentic-workflow/src/runtime/session.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds MAINTHREAD_SYSTEM_PROMPT constant (JSON-only instruction with
      action examples). Adds Session::decide(user_input) public method that
      spawns run_mainthread_decide. Adds run_mainthread_decide free function
      that streams the mainthread LLM turn, parses the decision, emits
      SessionEvent::MainthreadDecision, and conditionally dispatches
      run_create_issue for NewIssue decisions.

  - path: projects/cue/src/tui/actions.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds Action::MainthreadDecided { decision: MainthreadDecision } variant.
      Imported via agentic_workflow::runtime::{Envelope, MainthreadDecision}.

  - path: projects/cue/src/tui/app.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      App::apply handles Action::MainthreadDecided: rewrites the most-recent
      assistant chat bubble's content from raw JSON to human-readable text
      ("Creating issue: <title>" for NewIssue, plain content for Reply) so
      the developer never sees raw JSON in the chat pane.

  - path: projects/cue/src/tui/runner.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Action::SubmitChat branch now calls Session::decide(text) instead of
      Session::create_issue(text). The event-dispatch loop maps
      SessionEvent::MainthreadDecision to Action::MainthreadDecided.

  - path: projects/cue/tests/support/harness.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds with_mainthread_new_issue and with_mainthread_reply test helpers
      that pre-configure the mock LLM provider with canned mainthread
      responses for use in e2e scenarios.

  - path: projects/agentic-workflow/src/runtime/mod.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Registers the new mainthread module (pub mod mainthread) and
      re-exports MainthreadDecision and parse_decision so callers can
      import them via agentic_workflow::runtime::{MainthreadDecision, parse_decision}
      without knowing the internal submodule layout.

  - path: projects/cue/tests/e2e_lifecycle_ux.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds scenario_mainthread_reply_no_lifecycle: the primary acceptance
      test for the reply_path scenario. Asserts that a Reply decision
      produces no score dispatch and leaves lifecycle_state as Idle.

  - path: projects/cue/tests/e2e_lifecycle.rs
    action: modify
    section: source
    impl_mode: hand-written
    description: >
      Adds slice1_step_through_create_then_fill_section: the acceptance
      test for the new_issue_path scenario. Asserts that a NewIssue
      decision triggers run_create_issue and drives the full lifecycle
      loop through Author / Reviewer / Reviser phases.
  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 2
<!-- type: review lang: markdown -->

**Verdict:** approved

- [changes] All three previously flagged entries are present and correct: `projects/agentic-workflow/src/runtime/mod.rs` (registers `pub mod mainthread` and re-exports), `projects/cue/tests/e2e_lifecycle_ux.rs` (adds `scenario_mainthread_reply_no_lifecycle`), and `projects/cue/tests/e2e_lifecycle.rs` (adds `slice1_step_through_create_then_fill_section`). All carry `impl_mode: hand-written`. Traceability from scenarios to changes is now complete.
- [schema] Untouched and correct. No regressions.
- [scenarios] Untouched and correct. No regressions.

## Review 1
<!-- type: review lang: markdown -->

**Verdict:** needs-revision

- [changes] `projects/agentic-workflow/src/runtime/mod.rs` is missing from the changes list. The commit f01d776e7 modifies this file to register `pub mod mainthread;`. Without this entry the spec does not fully account for how the new module is wired into the crate. Add an entry: `path: projects/agentic-workflow/src/runtime/mod.rs, action: modify, impl_mode: hand-written, description: "Registers the new mainthread module (pub mod mainthread) and re-exports MainthreadDecision."`.
- [changes] `projects/cue/tests/e2e_lifecycle_ux.rs` is missing from the changes list despite being modified in f01d776e7 (51 lines added). This file contains `scenario_mainthread_reply_no_lifecycle`, the primary acceptance-criterion test cited in the `reply_path` scenario. Add an entry: `path: projects/cue/tests/e2e_lifecycle_ux.rs, action: modify, impl_mode: hand-written, description: "Adds scenario_mainthread_reply_no_lifecycle asserting Reply decision produces no lifecycle dispatch."`.
- [changes] `projects/cue/tests/e2e_lifecycle.rs` is modified in f01d776e7 (54 lines changed) and is cited as the acceptance test for the `new_issue_path` scenario (`slice1_step_through_create_then_fill_section`), yet it is absent from the changes list. Add an entry to close the traceability gap between scenarios and changes.
