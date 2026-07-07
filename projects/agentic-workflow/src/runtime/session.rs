// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
// CODEGEN-BEGIN
//! `Session` — the per-issue agent loop, shared by all SDD frontends.
//!
//! Slice 1 surface: `create_issue(title)` performs
//!   1. `aw wi create <title>` → slug
//!   2. route Author task → choose provider/model
//!   3. stream LLM completion (Requirements section draft)
//!   4. `aw wi fill-section --apply <slug> requirements <body>`
//! emitting `SessionEvent`s along the way. `aw wi validate` (the
//! commit point) lands in slice 2.

use crate::runtime::envelope::Envelope;
use crate::runtime::event::{SessionEvent, TurnId};
use crate::runtime::issue_backend::{
    BackendKind, IssueBackend, IssueBody, IssueId, IssueRef, ListFilter,
};
use crate::runtime::mainthread::{parse_decision, MainthreadDecision};
use crate::runtime::router::{ModelRouter, Task};
use crate::runtime::score_process::{LocalIssueBackend, ScoreProcess};
use agent::{CompletionRequest, LLMProvider, Message};
use anyhow::{anyhow, Result};
use futures::StreamExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#logic
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/mainthread.md#changes
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/issue_backend.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Phase {
    Draft,
    Requirements,
    Scope,
    ReferenceContext,
    Open,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
pub struct IssueBinding {
    pub slug: String,
    pub phase: Phase,
}

const SESSION_CHANNEL_BUFFER: usize = 64;

const REQUIREMENTS_SYSTEM_PROMPT: &str = "You are an SDD Requirements author. Produce a concise, machine-readable Requirements section in Markdown for the provided issue title. Focus on Problem statement, Goals, and Non-goals. Output Markdown only — no preamble, no fences.";

const REVIEW_SYSTEM_PROMPT: &str = "You are an SDD section reviewer. For each filled section in the issue body, judge whether the content meets the bar (concrete, testable, no filler). Output one verdict per section, plus an overall `approve` or `needs-revision` decision. Markdown only.";

const REVISE_SYSTEM_PROMPT: &str = "You are an SDD section reviser. Given the current issue body and reviewer feedback, rewrite the flagged sections so they address the feedback. Output the revised section(s) in Markdown only — no preamble, no fences.";

const MAINTHREAD_SYSTEM_PROMPT: &str = r#"You are cue's mainthread agent — the dev's conversational counterpart. Classify each dev message into a structured action. Output JSON ONLY (no preamble, no markdown fence) matching one of:

  {"action": "new_issue", "title": "<short slug-friendly title>"}
    when the dev wants to create a new SDD issue.

  {"action": "reply", "content": "<message back to the dev>"}
    when the dev is asking a question, requesting status, or making a comment that does not start a new lifecycle.

Be conservative: only emit "new_issue" when the dev's intent is clearly a new feature/bug to track. Anything ambiguous → "reply" asking for clarification."#;

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
pub struct Session {
    provider: Arc<dyn LLMProvider>,
    score_process: Arc<dyn ScoreProcess>,
    /// Active issue backend selected at construction (per
    /// `.cue/config.toml` `[issue].backend`). `Session::decide` /
    /// `run_create_issue` route `create` calls through this trait
    /// instead of `score_process.create` directly — that keeps the
    /// backend choice observable at runtime and avoids hardcoding
    /// `local`. SDD lifecycle ops (fill_section_apply / review_apply /
    /// validate / merge) still go via `score_process` since slice 1 keeps
    /// CRRR fill semantics scoped to local (issue R9).
    issue_backend: Arc<dyn IssueBackend>,
    router: Arc<dyn ModelRouter>,
    binding: Option<IssueBinding>,
    next_turn: AtomicU64,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
impl Session {
    pub fn builder() -> SessionBuilder {
        SessionBuilder::default()
    }

    pub fn binding(&self) -> Option<&IssueBinding> {
        self.binding.as_ref()
    }

    fn next_turn_id(&self) -> TurnId {
        TurnId(self.next_turn.fetch_add(1, Ordering::SeqCst))
    }

    /// Drive the create-issue + Requirements-authoring slice.
    /// Returns a receiver that yields `SessionEvent`s as work progresses;
    /// the actual work runs in a spawned task so the TUI can render the
    /// stream while it arrives.
    pub async fn create_issue(&mut self, title: &str) -> Result<mpsc::Receiver<SessionEvent>> {
        let (tx, rx) = mpsc::channel(SESSION_CHANNEL_BUFFER);

        let provider = self.provider.clone();
        let score_process = self.score_process.clone();
        let issue_backend = self.issue_backend.clone();
        let router = self.router.clone();
        let _turn_id = self.next_turn_id();
        let title = title.to_string();

        // Optimistically record the binding when we get the slug back below.
        // For now leave self.binding alone; runner.rs will refresh it from
        // the IssueCreated event.

        tokio::spawn(async move {
            let _ =
                run_create_issue(provider, score_process, issue_backend, router, title, tx).await;
        });

        Ok(rx)
    }

    /// Mainthread agent entry point. Receives the dev's free-form chat
    /// input, runs an LLM turn (Task::Mainthread → Claude default),
    /// parses the response as a `MainthreadDecision`, then either
    /// dispatches a full lifecycle (NewIssue) or just lets the reply
    /// stand (Reply).
    ///
    /// `Action::SubmitChat` from the TUI funnels here instead of going
    /// directly to `create_issue` — gives the dev an actual
    /// conversational counterpart that classifies intent before
    /// touching the lifecycle.
    pub async fn decide(&mut self, user_input: &str) -> Result<mpsc::Receiver<SessionEvent>> {
        let (tx, rx) = mpsc::channel(SESSION_CHANNEL_BUFFER);

        let provider = self.provider.clone();
        let score_process = self.score_process.clone();
        let issue_backend = self.issue_backend.clone();
        let router = self.router.clone();
        let _turn_id = self.next_turn_id();
        let user_input = user_input.to_string();

        tokio::spawn(async move {
            let _ = run_mainthread_decide(
                provider,
                score_process,
                issue_backend,
                router,
                user_input,
                tx,
            )
            .await;
        });

        Ok(rx)
    }

    /// Slice-2 placeholder — kept on the surface so callers can be written
    /// against the final shape. Today it just emits an Error event.
    pub async fn turn(&mut self, _prompt: &str) -> Result<mpsc::Receiver<SessionEvent>> {
        let (tx, rx) = mpsc::channel(SESSION_CHANNEL_BUFFER);
        let _ = tx
            .send(SessionEvent::Error {
                message: "Session::turn not implemented (slice 2)".into(),
            })
            .await;
        Ok(rx)
    }

    pub async fn list_issues(&self, filter: &ListFilter) -> Result<Vec<IssueRef>> {
        self.issue_backend
            .list(filter)
            .await
            .map_err(|e| anyhow!("issue backend list: {e}"))
    }

    pub async fn read_issue(&self, id: &IssueId) -> Result<IssueBody> {
        self.issue_backend
            .read(id)
            .await
            .map_err(|e| anyhow!("issue backend read {id}: {e}"))
    }

    pub async fn close_issue(&self, id: &IssueId, message: Option<&str>) -> Result<()> {
        self.issue_backend
            .close(id, message)
            .await
            .map_err(|e| anyhow!("issue backend close {id}: {e}"))
    }

    pub fn set_binding(&mut self, binding: IssueBinding) {
        self.binding = Some(binding);
    }
}

async fn run_create_issue(
    provider: Arc<dyn LLMProvider>,
    score_process: Arc<dyn ScoreProcess>,
    issue_backend: Arc<dyn IssueBackend>,
    router: Arc<dyn ModelRouter>,
    title: String,
    tx: mpsc::Sender<SessionEvent>,
) -> Result<()> {
    let send = |ev: SessionEvent| {
        let tx = tx.clone();
        async move {
            let _ = tx.send(ev).await;
        }
    };

    send(SessionEvent::UserMessage {
        content: title.clone(),
    })
    .await;

    // Branch on backend kind: SDD CRRR fill semantics are scoped to
    // LOCAL in slice 1 (issue R9). Remote backends just create the
    // issue and stop — the dev gets confirmation in chat, no
    // author/reviewer/reviser dispatch runs. That keeps the trait
    // abstraction honest while preserving the existing local-only
    // CRRR contract.
    match issue_backend.backend_kind() {
        BackendKind::Local => {
            // Local path: score_process.create returns the real Dispatch
            // envelope (with the actual `sections` list). Run the
            // full lifecycle loop against it.
            let create_env = match score_process.create(&title).await {
                Ok(env) => env,
                Err(e) => {
                    send(SessionEvent::Error {
                        message: format!("aw wi create failed: {e}"),
                    })
                    .await;
                    return Err(anyhow!(e));
                }
            };
            send(SessionEvent::Envelope(create_env.clone())).await;

            let slug = match &create_env {
                Envelope::Dispatch { slug, .. } | Envelope::Done { slug, .. } => slug.clone(),
                Envelope::Error { slug: _, message } => {
                    send(SessionEvent::Error {
                        message: message.clone(),
                    })
                    .await;
                    return Err(anyhow!("aw wi create returned error envelope"));
                }
                Envelope::Batch { .. } => {
                    send(SessionEvent::Error {
                        message: "unexpected batch envelope from issues create".into(),
                    })
                    .await;
                    return Err(anyhow!("unexpected batch envelope"));
                }
            };

            drive_lifecycle_loop(
                create_env,
                slug,
                title,
                provider.clone(),
                score_process.clone(),
                router.clone(),
                tx.clone(),
            )
            .await
        }
        kind => {
            // Remote path (github / gitlab / jira): create only, no
            // lifecycle loop. Per R9: SDD CRRR fill semantics stay
            // scoped to local — remote backends return the issue id
            // and the dev moves on.
            let issue_id = match issue_backend.create(&title).await {
                Ok(id) => id,
                Err(e) => {
                    send(SessionEvent::Error {
                        message: format!("{kind:?} backend create failed: {e}"),
                    })
                    .await;
                    return Err(anyhow!("backend create: {e}"));
                }
            };
            // Emit a Done envelope so the runner / app surface a
            // terminal-state lifecycle badge ("✓ created").
            send(SessionEvent::Envelope(Envelope::Done {
                slug: issue_id.as_str().to_string(),
                message: Some(format!("created on {kind:?}")),
            }))
            .await;
            Ok(())
        }
    }
}

/// Mainthread agent — receives the dev's free-form chat input, runs an
/// LLM turn (Task::Mainthread), parses the response as a structured
/// `MainthreadDecision`, then either dispatches a full lifecycle
/// (NewIssue) or just emits a Reply event for chat display.
///
/// Errors funnel through `SessionEvent::Error`. The lifecycle dispatch
/// path delegates to `run_create_issue` for slug + lifecycle setup, so
/// no logic is duplicated.
async fn run_mainthread_decide(
    provider: Arc<dyn LLMProvider>,
    score_process: Arc<dyn ScoreProcess>,
    issue_backend: Arc<dyn IssueBackend>,
    router: Arc<dyn ModelRouter>,
    user_input: String,
    tx: mpsc::Sender<SessionEvent>,
) -> Result<()> {
    let _ = tx
        .send(SessionEvent::UserMessage {
            content: user_input.clone(),
        })
        .await;

    let messages = vec![
        Message::system(MAINTHREAD_SYSTEM_PROMPT),
        Message::user(user_input.clone()),
    ];
    let body = run_llm_turn(
        &provider,
        &router,
        Task::Mainthread,
        "mainthread",
        messages,
        &tx,
    )
    .await?;

    let decision = match parse_decision(&body) {
        Some(d) => d,
        None => {
            let _ = tx
                .send(SessionEvent::Error {
                    message: format!(
                        "mainthread agent returned unparseable JSON: {}",
                        body.lines().next().unwrap_or(&body)
                    ),
                })
                .await;
            return Err(anyhow!("unparseable mainthread decision"));
        }
    };

    let _ = tx
        .send(SessionEvent::MainthreadDecision {
            decision: decision.clone(),
        })
        .await;

    match decision {
        MainthreadDecision::NewIssue { title } => {
            // Hand off to the existing lifecycle entry point.
            run_create_issue(provider, score_process, issue_backend, router, title, tx).await
        }
        MainthreadDecision::Reply { .. } => {
            // The reply text already streamed via AssistantDelta /
            // AssistantMessageComplete during the LLM turn; nothing
            // more to do.
            Ok(())
        }
    }
}

/// State-machine loop that consumes envelopes returned by either CLI calls
/// or `*_apply` calls and dispatches the next step. Terminates on a
/// terminal envelope (`Done` / `Error` / `Batch`) or an unsupported
/// dispatch (logged via `SessionEvent::Error`, then graceful exit).
async fn drive_lifecycle_loop(
    initial_env: Envelope,
    slug: String,
    initial_body: String,
    provider: Arc<dyn LLMProvider>,
    score_process: Arc<dyn ScoreProcess>,
    router: Arc<dyn ModelRouter>,
    tx: mpsc::Sender<SessionEvent>,
) -> Result<()> {
    let mut env = initial_env;
    let mut last_body = initial_body;

    loop {
        match env {
            Envelope::Done { .. } => return Ok(()),
            Envelope::Error { ref message, .. } => {
                let _ = tx
                    .send(SessionEvent::Error {
                        message: message.clone(),
                    })
                    .await;
                return Ok(());
            }
            Envelope::Batch { .. } => {
                let _ = tx
                    .send(SessionEvent::Error {
                        message: "lifecycle loop got Batch envelope (unsupported)".into(),
                    })
                    .await;
                return Ok(());
            }
            Envelope::Dispatch {
                agent: None,
                invoke: Some(ref inv),
                ..
            } => {
                let next_env = if inv.command.contains("validate") {
                    score_process.validate(&slug).await
                } else if inv.command.contains("merge") {
                    score_process.merge(&slug).await
                } else {
                    let _ = tx
                        .send(SessionEvent::Error {
                            message: format!("unsupported mainthread verb: {}", inv.command),
                        })
                        .await;
                    return Ok(());
                };
                env = match next_env {
                    Ok(next) => {
                        let _ = tx.send(SessionEvent::Envelope(next.clone())).await;
                        next
                    }
                    Err(e) => {
                        let _ = tx
                            .send(SessionEvent::Error {
                                message: format!("score process invoke failed: {e}"),
                            })
                            .await;
                        return Err(anyhow!(e));
                    }
                };
            }
            Envelope::Dispatch {
                agent: Some(ref role),
                invoke: ref maybe_invoke,
                ..
            } => {
                let role = role.clone();
                let plan = match dispatch_plan(&role) {
                    Some(p) => p,
                    None => {
                        let _ = tx
                            .send(SessionEvent::Error {
                                message: format!("unknown agent role: {role}"),
                            })
                            .await;
                        return Ok(());
                    }
                };

                // Author dispatches carry a `sections` list — the agent
                // loops internally (LLM turn + fill_section per section).
                // Reviewer / Reviser are single-step.
                let sections = match plan.apply {
                    ApplyVerb::FillSection { section } => extract_sections(maybe_invoke.as_ref())
                        .unwrap_or_else(|| vec![section.to_string()]),
                    ApplyVerb::Review => vec![String::new()], // section unused
                };

                let mut last_apply_env: Option<Envelope> = None;
                for section in &sections {
                    let prompt_messages = build_prompt(&plan, &slug, &last_body, Some(section));
                    let body =
                        run_llm_turn(&provider, &router, plan.task, &role, prompt_messages, &tx)
                            .await?;
                    last_body = body.clone();

                    let apply_res = match plan.apply {
                        ApplyVerb::FillSection { .. } => {
                            score_process
                                .fill_section_apply(&slug, section, &body)
                                .await
                        }
                        ApplyVerb::Review => score_process.review_apply(&slug, &body).await,
                    };
                    let next = match apply_res {
                        Ok(next) => {
                            let _ = tx.send(SessionEvent::Envelope(next.clone())).await;
                            next
                        }
                        Err(e) => {
                            let _ = tx
                                .send(SessionEvent::Error {
                                    message: format!("score apply failed: {e}"),
                                })
                                .await;
                            return Err(anyhow!(e));
                        }
                    };
                    last_apply_env = Some(next);
                }
                // The LAST section's apply envelope drives the next loop
                // iteration. Mid-sequence envelopes from fill_section
                // (which would dispatch validate per section if naively
                // followed) are intentionally discarded — the agent
                // loops internally per the score process design.
                env = last_apply_env.expect("dispatch_plan always produces at least one section");
            }
            // Dispatch with `agent: None` AND no invoke is malformed — bail.
            Envelope::Dispatch {
                agent: None,
                invoke: None,
                ..
            } => {
                let _ = tx
                    .send(SessionEvent::Error {
                        message: "Dispatch envelope missing both agent and invoke".into(),
                    })
                    .await;
                return Ok(());
            }
        }
    }
}

/// Per-subagent dispatch plan: which routing task, system prompt, and
/// `*_apply` verb to use after the LLM turn. Slice-by-slice extensible.
struct DispatchPlan {
    task: Task,
    system_prompt: &'static str,
    user_intro: &'static str,
    apply: ApplyVerb,
}

enum ApplyVerb {
    /// `aw wi fill-section --apply --slug S --section X --body Y`.
    /// Slice 3 will read the section name from the dispatch envelope's
    /// `invoke.args.sections` array; for now we only handle requirements.
    FillSection { section: &'static str },
    /// `aw wi review --apply --slug S --body Y`.
    Review,
}

fn dispatch_plan(role: &str) -> Option<DispatchPlan> {
    match role {
        "score-issue-author" => Some(DispatchPlan {
            task: Task::Author,
            system_prompt: REQUIREMENTS_SYSTEM_PROMPT,
            user_intro: "Draft the Requirements section.",
            apply: ApplyVerb::FillSection {
                section: "requirements",
            },
        }),
        "score-issue-reviewer" => Some(DispatchPlan {
            task: Task::Review,
            system_prompt: REVIEW_SYSTEM_PROMPT,
            user_intro:
                "Return one verdict per filled section plus an overall approve/needs-revision.",
            apply: ApplyVerb::Review,
        }),
        "score-issue-reviser" => Some(DispatchPlan {
            task: Task::Revise,
            system_prompt: REVISE_SYSTEM_PROMPT,
            user_intro: "Rewrite the flagged sections to address the reviewer feedback.",
            apply: ApplyVerb::FillSection {
                section: "requirements",
            },
        }),
        _ => None,
    }
}

fn build_prompt(
    plan: &DispatchPlan,
    slug: &str,
    last_body: &str,
    section: Option<&str>,
) -> Vec<Message> {
    let intro = match section {
        Some(s) if !s.is_empty() => format!("Section: {s}\n\n{}", plan.user_intro),
        _ => plan.user_intro.to_string(),
    };
    vec![
        Message::system(plan.system_prompt),
        Message::user(format!(
            "Issue slug: {slug}\n\nCurrent body:\n\n{last_body}\n\n{intro}"
        )),
    ]
}

/// Extract `sections: [...]` from a dispatch envelope's `invoke.args`.
/// Returns None if the field is absent or malformed; callers fall back
/// to the per-role default (e.g., `["requirements"]` for Author).
fn extract_sections(invoke: Option<&crate::runtime::envelope::Invoke>) -> Option<Vec<String>> {
    let inv = invoke?;
    let arr = inv.args.get("sections")?.as_array()?;
    let sections: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if sections.is_empty() {
        None
    } else {
        Some(sections)
    }
}

/// Run one LLM turn end-to-end: route → emit `TurnStart` → open stream →
/// drain chunks → emit `AssistantDelta` / `AssistantMessageComplete`.
/// Returns the full concatenated content. Errors funnel through
/// `SessionEvent::Error` so the TUI surfaces them without panicking.
///
/// `agent_role` is the SDD agent name (`score-issue-author` etc.) — used
/// by the `TurnStart` event so the TUI knows which chat-bubble role +
/// color to open before any deltas arrive.
async fn run_llm_turn(
    provider: &Arc<dyn LLMProvider>,
    router: &Arc<dyn ModelRouter>,
    task: Task,
    agent_role: &str,
    messages: Vec<Message>,
    tx: &mpsc::Sender<SessionEvent>,
) -> Result<String> {
    let choice = match router.route(task).await {
        Some(c) => c,
        None => {
            let _ = tx
                .send(SessionEvent::Error {
                    message: format!("no model route for {} task", task.as_str()),
                })
                .await;
            return Err(anyhow!("no {} route", task.as_str()));
        }
    };

    let _ = tx
        .send(SessionEvent::TurnStart {
            role: agent_role.to_string(),
            model: choice.model.clone(),
        })
        .await;

    let request = CompletionRequest::new(messages, &choice.model).with_stream(true);
    let mut stream = match provider.complete_stream(request).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx
                .send(SessionEvent::Error {
                    message: format!("LLM stream error: {e}"),
                })
                .await;
            return Err(anyhow!("llm stream open failed: {e}"));
        }
    };

    let mut full = String::new();
    while let Some(chunk_res) = stream.next().await {
        match chunk_res {
            Ok(chunk) => {
                if !chunk.content.is_empty() {
                    full.push_str(&chunk.content);
                    let _ = tx
                        .send(SessionEvent::AssistantDelta {
                            content: chunk.content.clone(),
                        })
                        .await;
                }
                if chunk.is_final {
                    break;
                }
            }
            Err(e) => {
                let _ = tx
                    .send(SessionEvent::Error {
                        message: format!("LLM stream error: {e}"),
                    })
                    .await;
                return Err(anyhow!("llm stream chunk failed: {e}"));
            }
        }
    }

    let _ = tx
        .send(SessionEvent::AssistantMessageComplete {
            content: full.clone(),
        })
        .await;

    Ok(full)
}

#[derive(Default)]
/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
pub struct SessionBuilder {
    provider: Option<Arc<dyn LLMProvider>>,
    score_process: Option<Arc<dyn ScoreProcess>>,
    issue_backend: Option<Arc<dyn IssueBackend>>,
    router: Option<Arc<dyn ModelRouter>>,
    binding: Option<IssueBinding>,
}

/// @spec projects/agentic-workflow/tech-design/core/logic/runtime/session.md#source
impl SessionBuilder {
    pub fn provider(mut self, p: Arc<dyn LLMProvider>) -> Self {
        self.provider = Some(p);
        self
    }
    pub fn score_process(mut self, s: Arc<dyn ScoreProcess>) -> Self {
        self.score_process = Some(s);
        self
    }
    /// Override the active issue backend. Defaults to a
    /// `LocalIssueBackend` wrapping the configured `score_process` if not
    /// set — preserves existing test behavior where only `score_process`
    /// is provided.
    pub fn issue_backend(mut self, b: Arc<dyn IssueBackend>) -> Self {
        self.issue_backend = Some(b);
        self
    }
    pub fn router(mut self, r: Arc<dyn ModelRouter>) -> Self {
        self.router = Some(r);
        self
    }
    pub fn binding(mut self, b: IssueBinding) -> Self {
        self.binding = Some(b);
        self
    }
    pub fn build(self) -> Result<Session> {
        let score_process = self
            .score_process
            .ok_or_else(|| anyhow!("SessionBuilder: score_process is required"))?;
        // Default the issue backend to LocalIssueBackend over the
        // provided ScoreProcess — keeps tests that only set score_process
        // working without changes.
        let issue_backend: Arc<dyn IssueBackend> = match self.issue_backend {
            Some(b) => b,
            None => Arc::new(LocalIssueBackend::new(score_process.clone())),
        };
        Ok(Session {
            provider: self
                .provider
                .ok_or_else(|| anyhow!("SessionBuilder: provider is required"))?,
            score_process,
            issue_backend,
            router: self
                .router
                .ok_or_else(|| anyhow!("SessionBuilder: router is required"))?,
            binding: self.binding,
            next_turn: AtomicU64::new(0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::envelope::Envelope;
    use crate::runtime::issue_backend::{IssueBody, IssueId, IssueRef, IssueState, ListFilter};
    use crate::runtime::router::{ModelChoice, StaticRouter};
    use crate::runtime::score_process::{
        MockBackendCall, MockIssueBackend, MockScoreProcess, ScoreCall,
    };
    use agent::{
        CompletionRequest as Req, CompletionResponse as Resp, NovaResult, StreamChunk,
        StreamResponse,
    };
    use async_trait::async_trait;
    use futures::stream;
    use std::sync::Mutex;

    struct ScriptedProvider {
        chunks: Mutex<Vec<NovaResult<StreamChunk>>>,
        last_request: Mutex<Option<Req>>,
    }

    impl ScriptedProvider {
        fn new(chunks: Vec<NovaResult<StreamChunk>>) -> Self {
            Self {
                chunks: Mutex::new(chunks),
                last_request: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl LLMProvider for ScriptedProvider {
        fn provider_name(&self) -> &str {
            "scripted"
        }
        fn supported_models(&self) -> Vec<String> {
            vec!["gemini-2.5-pro".into()]
        }
        async fn complete(&self, _req: Req) -> NovaResult<Resp> {
            unimplemented!()
        }
        async fn complete_stream(&self, req: Req) -> NovaResult<StreamResponse> {
            *self.last_request.lock().unwrap() = Some(req);
            let chunks = std::mem::take(&mut *self.chunks.lock().unwrap());
            Ok(Box::pin(stream::iter(chunks)))
        }
    }

    fn final_chunk(content: &str) -> NovaResult<StreamChunk> {
        Ok(StreamChunk {
            content: content.into(),
            tool_calls: None,
            finish_reason: Some("stop".into()),
            is_final: true,
        })
    }

    fn delta(content: &str) -> NovaResult<StreamChunk> {
        Ok(StreamChunk {
            content: content.into(),
            tool_calls: None,
            finish_reason: None,
            is_final: false,
        })
    }

    #[tokio::test]
    async fn create_issue_full_path_emits_expected_events() {
        let mock = Arc::new(MockScoreProcess::new());
        // create returns a Dispatch to score-issue-author with a single
        // section — the lifecycle loop runs ONE LLM turn + ONE
        // fill_section_apply, then terminates on Done from fill_section.
        mock.enqueue_create(Envelope::Dispatch {
            slug: "add-metrics-dashboard".into(),
            agent: Some("score-issue-author".into()),
            invoke: Some(crate::runtime::envelope::Invoke {
                command: "aw wi fill-section".into(),
                args: serde_json::json!({
                    "slug": "add-metrics-dashboard",
                    "sections": ["requirements"],
                }),
            }),
            artifact_quality_profile: None,
        });
        mock.enqueue_fill_section(Envelope::Done {
            slug: "add-metrics-dashboard".into(),
            message: Some("requirements applied".into()),
        });

        let provider = Arc::new(ScriptedProvider::new(vec![
            delta("## Problem\n\n"),
            final_chunk("Teams lack visibility."),
        ]));

        let mut session = Session::builder()
            .provider(provider)
            .score_process(mock.clone())
            .router(Arc::new(StaticRouter::empty().with_route(
                Task::Author,
                ModelChoice {
                    provider: "gemini".into(),
                    model: "gemini-2.5-pro".into(),
                },
            )))
            .build()
            .unwrap();

        let mut rx = session.create_issue("add metrics dashboard").await.unwrap();

        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }

        // user prompt → create envelope → 2 deltas → complete → fill envelope
        assert!(matches!(events[0], SessionEvent::UserMessage { .. }));
        assert!(matches!(events[1], SessionEvent::Envelope(_)));
        let delta_count = events
            .iter()
            .filter(|e| matches!(e, SessionEvent::AssistantDelta { .. }))
            .count();
        assert_eq!(delta_count, 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, SessionEvent::AssistantMessageComplete { .. })));
        assert!(events
            .last()
            .is_some_and(|e| matches!(e, SessionEvent::Envelope(_))));

        let calls = mock.calls();
        assert_eq!(calls.len(), 2);
        match &calls[0] {
            ScoreCall::Create { title } => assert_eq!(title, "add metrics dashboard"),
            other => panic!("expected Create, got {other:?}"),
        }
        match &calls[1] {
            ScoreCall::FillSectionApply {
                slug,
                section,
                body,
            } => {
                assert_eq!(slug, "add-metrics-dashboard");
                assert_eq!(section, "requirements");
                assert!(body.contains("Teams lack visibility"));
            }
            other => panic!("expected FillSectionApply, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn create_issue_score_failure_emits_error_and_skips_llm() {
        let mock = Arc::new(MockScoreProcess::new());
        // No enqueue → MockExhausted error from `create`.
        let provider = Arc::new(ScriptedProvider::new(vec![]));

        let mut session = Session::builder()
            .provider(provider)
            .score_process(mock.clone())
            .router(Arc::new(StaticRouter::defaults()))
            .build()
            .unwrap();

        let mut rx = session.create_issue("title").await.unwrap();
        let mut events = Vec::new();
        while let Some(ev) = rx.recv().await {
            events.push(ev);
        }

        assert!(events
            .iter()
            .any(|e| matches!(e, SessionEvent::Error { .. })));
        // No fill-section call
        assert_eq!(mock.calls().len(), 1);
    }

    #[tokio::test]
    async fn issue_management_methods_delegate_to_active_backend() {
        let mock_score = Arc::new(MockScoreProcess::new());
        let mock_backend = Arc::new(MockIssueBackend::new(BackendKind::Local));
        mock_backend.enqueue_list(vec![IssueRef {
            id: IssueId::new("alpha"),
            title: "Alpha".into(),
            state: IssueState::Open,
            labels: vec!["project:jet".into()],
        }]);
        mock_backend.enqueue_read(IssueBody {
            id: IssueId::new("alpha"),
            title: "Alpha".into(),
            body_md: "## Problem\n\nBody.".into(),
            frontmatter: Default::default(),
        });
        let session = Session::builder()
            .provider(Arc::new(ScriptedProvider::new(vec![])))
            .score_process(mock_score)
            .issue_backend(mock_backend.clone())
            .router(Arc::new(StaticRouter::defaults()))
            .build()
            .unwrap();

        let refs = session.list_issues(&ListFilter::default()).await.unwrap();
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].id.as_str(), "alpha");

        let body = session.read_issue(&IssueId::new("alpha")).await.unwrap();
        assert!(body.body_md.contains("Body."));

        session
            .close_issue(&IssueId::new("alpha"), Some("done"))
            .await
            .unwrap();

        assert_eq!(
            mock_backend.calls(),
            vec![
                MockBackendCall::List {
                    filter: ListFilter::default()
                },
                MockBackendCall::Read {
                    id: IssueId::new("alpha")
                },
                MockBackendCall::Close {
                    id: IssueId::new("alpha"),
                    message: Some("done".into())
                }
            ]
        );
    }
}

// CODEGEN-END
