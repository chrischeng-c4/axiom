---
id: implementation
type: change_implementation
change_id: agent-crr-review
---

# Implementation

## Summary

New ReviewAgent and CRRCycle orchestrator in crates/cclab-agent across 5 files: (1) agents/review.rs (new, 554 lines): LLM-powered reviewer implementing the Reviewer trait; ReviewVerdict discriminated union (Approved/NeedsRevision/Rejected), ReviewIssue with Severity, two review modes (ReviewType::Spec checks format/completeness/consistency, ReviewType::Code checks spec compliance/security/tests/style), structured JSON output via complete_structured(), ReviewAgentBuilder with fluent API, 11 unit tests covering verdict helpers, serde round-trips, and schema validation; (2) agents/crr.rs (new, 585 lines): generic Create-Review-Revise orchestrator; CRRCycle wires creator Agent + Reviewer + reviser Agent into a loop — creates initial artifact, reviews it, on NeedsRevision builds a structured revision prompt and calls reviser, repeats up to max_revisions (default 3), returns Err(MaxRevisionsExceeded) or Err on Rejected; CRREvent enum for observability (Creating/Created/Reviewing/Reviewed/Revising/Revised/Completed), CRRCycleBuilder with fluent API including on_event(), 8 unit tests with mock agents and scripted reviewer (approved-first, two-revisions, max-exceeded, rejected, event emission, builder errors); (3) error.rs: added MaxRevisionsExceeded(u32) variant; (4) agents/mod.rs: added pub mod crr/review and re-exports for all public types; (5) lib.rs: re-exported CRRCycle, ReviewAgent, and all associated types at crate root.

## Diff

```diff
diff --git a/crates/cclab-agent/src/agents/mod.rs b/crates/cclab-agent/src/agents/mod.rs
index ebf139d1..a243664a 100644
--- a/crates/cclab-agent/src/agents/mod.rs
+++ b/crates/cclab-agent/src/agents/mod.rs
@@ -2,14 +2,21 @@
 
 mod analyst;
 mod coding;
+pub mod crr;
+pub mod review;
 mod restructure;
 
 pub use analyst::{AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig};
 pub use coding::{CodingAgent, CodingAgentBuilder, CodingAgentConfig};
+pub use crr::{CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType};
 pub use restructure::{
     Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
 };
+pub use review::{
+    ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,
+    Reviewer, Severity,
+};
 
 use crate::error::NovaResult;
 use crate::stream::StreamHandler;
diff --git a/crates/cclab-agent/src/error.rs b/crates/cclab-agent/src/error.rs
index 730322f2..6591d18c 100644
--- a/crates/cclab-agent/src/error.rs
+++ b/crates/cclab-agent/src/error.rs
@@ -31,6 +31,9 @@ pub enum NovaError {
     #[error("Maximum turns reached: {0}")]
     MaxTurnsReached(u32),
 
+    #[error("Maximum revisions exceeded: {0}")]
+    MaxRevisionsExceeded(u32),
+
     #[error("Context overflow: token budget exceeded")]
     ContextOverflow,
 
diff --git a/crates/cclab-agent/src/lib.rs b/crates/cclab-agent/src/lib.rs
index 13dc766b..484c2b6b 100644
--- a/crates/cclab-agent/src/lib.rs
+++ b/crates/cclab-agent/src/lib.rs
@@ -81,8 +81,11 @@ mod types;
 pub use agents::{
     Agent, AnalystAgent, AnalystAgentBuilder, AnalystAgentConfig, ApprovalHandler,
     AutoApproveHandler, CodingAgent, CodingAgentBuilder, CodingAgentConfig,
+    CRRCycle, CRRCycleBuilder, CRREvent, CRRResult, CRRVerdictType,
     Clarification, Question, RestructureAgent, RestructureAgentBuilder, RestructureAgentConfig,
     RestructureInput, RestructureOutput, SpecExcerpt, SpecStore, StructuredIssue,
+    ReviewAgent, ReviewAgentBuilder, ReviewAgentConfig, ReviewIssue, ReviewType, ReviewVerdict,
+    Reviewer, Severity,
 };
 
 // Re-export context
diff --git a/crates/cclab-agent/src/agents/crr.rs b/crates/cclab-agent/src/agents/crr.rs
new file mode 100644
index 00000000..9433041b
--- /dev/null
+++ b/crates/cclab-agent/src/agents/crr.rs
@@ -0,0 +1,585 @@
+//! CRRCycle — generic Create-Review-Revise orchestration pattern.
+//!
+//! Wires a creator agent, a reviewer, and a reviser agent into a loop:
+//!
+//! ```text
+//! Create → Review → verdict
+//!   ↑                  ↓
+//!   └── Revise ← NEEDS_REVISION
+//!               APPROVED  → done
+//!               REJECTED  → Err
+//!               max_revisions exceeded → Err(MaxRevisionsExceeded)
+//! ```
+//!
+//! # Example
+//!
+//! ```rust,ignore
+//! use cclab_agent::{CRRCycle, ReviewAgent, ReviewType, CodingAgent, ClaudeProvider};
+//!
+//! let review_agent = ReviewAgent::builder()
+//!     .with_provider(provider.clone())
+//!     .with_review_type(ReviewType::Spec)
+//!     .build()?;
+//!
+//! let crr = CRRCycle::new()
+//!     .creator(spec_agent.clone())
+//!     .reviewer(review_agent)
+//!     .reviser(spec_agent)
+//!     .max_revisions(2)
+//!     .on_event(|event| eprintln!("[CRR] {:?}", event))
+//!     .build()?;
+//!
+//! let result = crr.run("Create a spec for the login flow").await?;
+//! println!("Artifact:\n{}", result.artifact);
+//! println!("Revisions: {}", result.revision_count);
+//! ```
+
+use crate::agents::{Agent, review::{ReviewIssue, ReviewVerdict, Reviewer}};
+use crate::error::{NovaError, NovaResult};
+use std::sync::Arc;
+
+// ============================================================
+// CRR events
+// ============================================================
+
+/// Events emitted during a [`CRRCycle`] run for logging or SSE forwarding.
+#[derive(Debug, Clone)]
+pub enum CRREvent {
+    /// Creator agent is about to run.
+    Creating,
+    /// Creator produced an artifact.
+    Created { artifact: String },
+    /// Reviewer is about to review the current artifact (revision 0 = initial).
+    Reviewing { revision: u32 },
+    /// Reviewer produced a verdict.
+    Reviewed { verdict_type: CRRVerdictType },
+    /// Reviser is about to address issues from revision `revision`.
+    Revising { revision: u32, issue_count: usize },
+    /// Reviser produced a revised artifact.
+    Revised { artifact: String },
+    /// Cycle completed successfully.
+    Completed { artifact: String, revision_count: u32 },
+}
+
+/// Compact discriminant carried in [`CRREvent::Reviewed`].
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub enum CRRVerdictType {
+    Approved,
+    NeedsRevision,
+    Rejected,
+}
+
+impl From<&ReviewVerdict> for CRRVerdictType {
+    fn from(v: &ReviewVerdict) -> Self {
+        match v {
+            ReviewVerdict::Approved => CRRVerdictType::Approved,
+            ReviewVerdict::NeedsRevision { .. } => CRRVerdictType::NeedsRevision,
+            ReviewVerdict::Rejected { .. } => CRRVerdictType::Rejected,
+        }
+    }
+}
+
+// ============================================================
+// CRRResult
+// ============================================================
+
+/// Output of a completed [`CRRCycle`] run.
+#[derive(Debug)]
+pub struct CRRResult {
+    /// Final review verdict (always [`ReviewVerdict::Approved`] on success).
+    pub verdict: ReviewVerdict,
+    /// The final artifact string produced by the cycle.
+    pub artifact: String,
+    /// Number of revisions performed (0 = approved on first review).
+    pub revision_count: u32,
+}
+
+// ============================================================
+// CRRCycle
+// ============================================================
+
+/// Generic Create-Review-Revise orchestrator.
+///
+/// Build with [`CRRCycle::new()`] (returns a [`CRRCycleBuilder`]).
+pub struct CRRCycle {
+    creator: Arc<dyn Agent>,
+    reviewer: Arc<dyn Reviewer>,
+    reviser: Arc<dyn Agent>,
+    max_revisions: u32,
+    event_handler: Option<Arc<dyn Fn(CRREvent) + Send + Sync>>,
+}
+
+impl std::fmt::Debug for CRRCycle {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("CRRCycle")
+            .field("max_revisions", &self.max_revisions)
+            .finish_non_exhaustive()
+    }
+}
+
+impl CRRCycle {
+    /// Start building a [`CRRCycle`].  Returns a [`CRRCycleBuilder`].
+    pub fn new() -> CRRCycleBuilder {
+        CRRCycleBuilder::default()
+    }
+
+    /// Run the full CRR loop starting from `input`.
+    ///
+    /// # Errors
+    ///
+    /// - [`NovaError::MaxRevisionsExceeded`] if the reviewer keeps returning
+    ///   `NeedsRevision` after `max_revisions` rounds.
+    /// - [`NovaError::Other`] if the reviewer returns `Rejected`.
+    pub async fn run(&self, input: &str) -> NovaResult<CRRResult> {
+        self.emit(CRREvent::Creating);
+
+        // 1. Create initial artifact
+        let initial = self.creator.run(input).await?;
+        self.emit(CRREvent::Created {
+            artifact: initial.clone(),
+        });
+
+        let mut artifact = initial;
+        let mut revision_count: u32 = 0;
+
+        loop {
+            // 2. Review current artifact
+            self.emit(CRREvent::Reviewing {
+                revision: revision_count,
+            });
+            let verdict = self.reviewer.review(&artifact).await?;
+            self.emit(CRREvent::Reviewed {
+                verdict_type: CRRVerdictType::from(&verdict),
+            });
+
+            match verdict {
+                ReviewVerdict::Approved => {
+                    self.emit(CRREvent::Completed {
+                        artifact: artifact.clone(),
+                        revision_count,
+                    });
+                    return Ok(CRRResult {
+                        verdict: ReviewVerdict::Approved,
+                        artifact,
+                        revision_count,
+                    });
+                }
+
+                ReviewVerdict::Rejected { ref reason } => {
+                    return Err(NovaError::Other(anyhow::anyhow!(
+                        "CRR cycle rejected after {} revision(s): {}",
+                        revision_count,
+                        reason
+                    )));
+                }
+
+                ReviewVerdict::NeedsRevision { ref issues } => {
+                    if revision_count >= self.max_revisions {
+                        return Err(NovaError::MaxRevisionsExceeded(self.max_revisions));
+                    }
+
+                    let issue_count = issues.len();
+                    self.emit(CRREvent::Revising {
+                        revision: revision_count,
+                        issue_count,
+                    });
+
+                    // 3. Revise: build a prompt that carries artifact + issues
+                    let revise_prompt = build_revision_prompt(&artifact, issues);
+                    let revised = self.reviser.run(&revise_prompt).await?;
+
+                    self.emit(CRREvent::Revised {
+                        artifact: revised.clone(),
+                    });
+
+                    artifact = revised;
+                    revision_count += 1;
+                }
+            }
+        }
+    }
+
+    // ---- private helpers ----
+
+    fn emit(&self, event: CRREvent) {
+        if let Some(ref handler) = self.event_handler {
+            handler(event);
+        }
+    }
+}
+
+/// Build a revision prompt that instructs the reviser agent what to fix.
+fn build_revision_prompt(artifact: &str, issues: &[ReviewIssue]) -> String {
+    let mut prompt = String::from(
+        "Revise the following artifact based on the review issues listed below.\n\n\
+         ## Original Artifact\n\n",
+    );
+    prompt.push_str(artifact);
+    prompt.push_str("\n\n## Review Issues\n");
+
+    for (i, issue) in issues.iter().enumerate() {
+        prompt.push_str(&format!(
+            "\n{}. [{}] {}\n   Suggestion: {}\n",
+            i + 1,
+            issue.severity,
+            issue.description,
+            issue.suggestion,
+        ));
+        if let Some(ref loc) = issue.location {
+            prompt.push_str(&format!("   Location: {}\n", loc));
+        }
+    }
+
+    prompt.push_str(
+        "\nPlease address every issue above and return the fully revised artifact.",
+    );
+    prompt
+}
+
+// ============================================================
+// CRRCycleBuilder
+// ============================================================
+
+/// Builder for [`CRRCycle`].
+///
+/// Obtained via [`CRRCycle::new()`].
+pub struct CRRCycleBuilder {
+    creator: Option<Arc<dyn Agent>>,
+    reviewer: Option<Arc<dyn Reviewer>>,
+    reviser: Option<Arc<dyn Agent>>,
+    max_revisions: u32,
+    event_handler: Option<Arc<dyn Fn(CRREvent) + Send + Sync>>,
+}
+
+impl Default for CRRCycleBuilder {
+    fn default() -> Self {
+        Self {
+            creator: None,
+            reviewer: None,
+            reviser: None,
+            max_revisions: 3,
+            event_handler: None,
+        }
+    }
+}
+
+impl CRRCycleBuilder {
+    /// Set the creator agent (produces the initial artifact from the input).
+    pub fn creator<A: Agent + 'static>(mut self, agent: A) -> Self {
+        self.creator = Some(Arc::new(agent));
+        self
+    }
+
+    /// Set the creator agent from an `Arc`.
+    pub fn creator_arc(mut self, agent: Arc<dyn Agent>) -> Self {
+        self.creator = Some(agent);
+        self
+    }
+
+    /// Set the reviewer (returns a [`ReviewVerdict`]).
+    pub fn reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
+        self.reviewer = Some(Arc::new(reviewer));
+        self
+    }
+
+    /// Set the reviewer from an `Arc`.
+    pub fn reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
+        self.reviewer = Some(reviewer);
+        self
+    }
+
+    /// Set the reviser agent (receives original artifact + issues, returns revised artifact).
+    pub fn reviser<A: Agent + 'static>(mut self, agent: A) -> Self {
+        self.reviser = Some(Arc::new(agent));
+        self
+    }
+
+    /// Set the reviser agent from an `Arc`.
+    pub fn reviser_arc(mut self, agent: Arc<dyn Agent>) -> Self {
+        self.reviser = Some(agent);
+        self
+    }
+
+    /// Maximum number of revise → review rounds before [`NovaError::MaxRevisionsExceeded`].
+    ///
+    /// Default: `3`.
+    pub fn max_revisions(mut self, n: u32) -> Self {
+        self.max_revisions = n;
+        self
+    }
+
+    /// Attach a sync event callback for logging, SSE forwarding, etc.
+    pub fn on_event<F: Fn(CRREvent) + Send + Sync + 'static>(mut self, handler: F) -> Self {
+        self.event_handler = Some(Arc::new(handler));
+        self
+    }
+
+    /// Build the [`CRRCycle`].
+    ///
+    /// # Errors
+    ///
+    /// Returns [`NovaError::ConfigError`] if creator, reviewer, or reviser is missing.
+    pub fn build(self) -> NovaResult<CRRCycle> {
+        let creator = self
+            .creator
+            .ok_or_else(|| NovaError::ConfigError("CRRCycle: creator is required".to_string()))?;
+        let reviewer = self
+            .reviewer
+            .ok_or_else(|| NovaError::ConfigError("CRRCycle: reviewer is required".to_string()))?;
+        let reviser = self
+            .reviser
+            .ok_or_else(|| NovaError::ConfigError("CRRCycle: reviser is required".to_string()))?;
+
+        Ok(CRRCycle {
+            creator,
+            reviewer,
+            reviser,
+            max_revisions: self.max_revisions,
+            event_handler: self.event_handler,
+        })
+    }
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use crate::agents::review::{ReviewIssue, ReviewVerdict, Reviewer, Severity};
+    use crate::error::NovaResult;
+    use async_trait::async_trait;
+
+    // ---- mock agents ----
+
+    struct ConstAgent(String);
+
+    #[async_trait]
+    impl Agent for ConstAgent {
+        async fn run(&self, _input: &str) -> NovaResult<String> {
+            Ok(self.0.clone())
+        }
+        async fn run_with_handler(
+            &self,
+            input: &str,
+            _handler: &dyn crate::stream::StreamHandler,
+        ) -> NovaResult<String> {
+            self.run(input).await
+        }
+    }
+
+    /// A reviewer whose verdict sequence is fixed at construction.
+    struct ScriptedReviewer {
+        verdicts: std::sync::Mutex<Vec<ReviewVerdict>>,
+    }
+
+    impl ScriptedReviewer {
+        fn new(verdicts: Vec<ReviewVerdict>) -> Self {
+            Self {
+                verdicts: std::sync::Mutex::new(verdicts),
+            }
+        }
+    }
+
+    #[async_trait]
+    impl Reviewer for ScriptedReviewer {
+        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
+            let mut v = self.verdicts.lock().unwrap();
+            if v.is_empty() {
+                Ok(ReviewVerdict::Approved)
+            } else {
+                Ok(v.remove(0))
+            }
+        }
+    }
+
+    fn make_issue(msg: &str) -> ReviewIssue {
+        ReviewIssue {
+            severity: Severity::Medium,
+            description: msg.to_string(),
+            suggestion: "fix it".to_string(),
+            location: None,
+        }
+    }
+
+    // ---- tests ----
+
+    #[tokio::test]
+    async fn test_crr_approved_on_first_review() {
+        let crr = CRRCycle::new()
+            .creator(ConstAgent("artifact-v1".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Approved]))
+            .reviser(ConstAgent("should-not-be-called".to_string()))
+            .max_revisions(2)
+            .build()
+            .unwrap();
+
+        let result = crr.run("input").await.unwrap();
+        assert_eq!(result.artifact, "artifact-v1");
+        assert_eq!(result.revision_count, 0);
+        assert!(result.verdict.is_approved());
+    }
+
+    #[tokio::test]
+    async fn test_crr_two_revisions_then_approved() {
+        let crr = CRRCycle::new()
+            .creator(ConstAgent("v1".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![
+                ReviewVerdict::NeedsRevision {
+                    issues: vec![make_issue("issue 1")],
+                },
+                ReviewVerdict::NeedsRevision {
+                    issues: vec![make_issue("issue 2")],
+                },
+                ReviewVerdict::Approved,
+            ]))
+            .reviser(ConstAgent("revised".to_string()))
+            .max_revisions(3)
+            .build()
+            .unwrap();
+
+        let result = crr.run("input").await.unwrap();
+        assert!(result.verdict.is_approved());
+        assert_eq!(result.revision_count, 2);
+    }
+
+    #[tokio::test]
+    async fn test_crr_max_revisions_exceeded() {
+        let crr = CRRCycle::new()
+            .creator(ConstAgent("v1".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![
+                ReviewVerdict::NeedsRevision {
+                    issues: vec![make_issue("persistent issue")],
+                },
+                ReviewVerdict::NeedsRevision {
+                    issues: vec![make_issue("still broken")],
+                },
+                ReviewVerdict::NeedsRevision {
+                    issues: vec![make_issue("never fixed")],
+                },
+            ]))
+            .reviser(ConstAgent("still-broken".to_string()))
+            .max_revisions(2)
+            .build()
+            .unwrap();
+
+        let err = crr.run("input").await.unwrap_err();
+        assert!(
+            matches!(err, NovaError::MaxRevisionsExceeded(2)),
+            "expected MaxRevisionsExceeded(2), got {:?}",
+            err
+        );
+    }
+
+    #[tokio::test]
+    async fn test_crr_rejected() {
+        let crr = CRRCycle::new()
+            .creator(ConstAgent("v1".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Rejected {
+                reason: "fundamentally wrong".to_string(),
+            }]))
+            .reviser(ConstAgent("irrelevant".to_string()))
+            .max_revisions(2)
+            .build()
+            .unwrap();
+
+        let err = crr.run("input").await.unwrap_err();
+        let msg = err.to_string();
+        assert!(msg.contains("rejected"), "expected rejected error, got: {}", msg);
+    }
+
+    #[tokio::test]
+    async fn test_crr_events_emitted() {
+        use std::sync::Mutex;
+        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
+        let events_clone = events.clone();
+
+        let crr = CRRCycle::new()
+            .creator(ConstAgent("artifact".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Approved]))
+            .reviser(ConstAgent("unused".to_string()))
+            .max_revisions(1)
+            .on_event(move |e| {
+                events_clone.lock().unwrap().push(format!("{:?}", e));
+            })
+            .build()
+            .unwrap();
+
+        crr.run("input").await.unwrap();
+
+        let recorded = events.lock().unwrap();
+        // Expect at least: Creating, Created, Reviewing, Reviewed, Completed
+        assert!(recorded.iter().any(|s| s.contains("Creating")));
+        assert!(recorded.iter().any(|s| s.contains("Created")));
+        assert!(recorded.iter().any(|s| s.contains("Reviewing")));
+        assert!(recorded.iter().any(|s| s.contains("Reviewed")));
+        assert!(recorded.iter().any(|s| s.contains("Completed")));
+    }
+
+    #[test]
+    fn test_builder_missing_creator() {
+        let err = CRRCycle::new()
+            .reviewer(ScriptedReviewer::new(vec![]))
+            .reviser(ConstAgent("x".to_string()))
+            .build()
+            .unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_builder_missing_reviewer() {
+        let err = CRRCycle::new()
+            .creator(ConstAgent("x".to_string()))
+            .reviser(ConstAgent("x".to_string()))
+            .build()
+            .unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_builder_missing_reviser() {
+        let err = CRRCycle::new()
+            .creator(ConstAgent("x".to_string()))
+            .reviewer(ScriptedReviewer::new(vec![]))
+            .build()
+            .unwrap_err();
+        assert!(matches!(err, NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_build_revision_prompt_contains_issues() {
+        let issues = vec![
+            ReviewIssue {
+                severity: Severity::High,
+                description: "Missing section".to_string(),
+                suggestion: "Add it".to_string(),
+                location: Some("spec/interfaces.md".to_string()),
+            },
+        ];
+        let prompt = build_revision_prompt("original content", &issues);
+        assert!(prompt.contains("original content"));
+        assert!(prompt.contains("Missing section"));
+        assert!(prompt.contains("Add it"));
+        assert!(prompt.contains("spec/interfaces.md"));
+        assert!(prompt.contains("High"));
+    }
+
+    #[test]
+    fn test_crr_verdict_type_from_verdict() {
+        assert_eq!(
+            CRRVerdictType::from(&ReviewVerdict::Approved),
+            CRRVerdictType::Approved
+        );
+        assert_eq!(
+            CRRVerdictType::from(&ReviewVerdict::NeedsRevision { issues: vec![] }),
+            CRRVerdictType::NeedsRevision
+        );
+        assert_eq!(
+            CRRVerdictType::from(&ReviewVerdict::Rejected {
+                reason: "x".to_string()
+            }),
+            CRRVerdictType::Rejected
+        );
+    }
+}
diff --git a/crates/cclab-agent/src/agents/review.rs b/crates/cclab-agent/src/agents/review.rs
new file mode 100644
index 00000000..2f0c277b
--- /dev/null
+++ b/crates/cclab-agent/src/agents/review.rs
@@ -0,0 +1,554 @@
+//! ReviewAgent — LLM-based spec and code review with structured verdicts.
+//!
+//! Provides a reusable [`ReviewAgent`] for CRR cycles. Supports two review modes:
+//! - [`ReviewType::Spec`]: format, quality, completeness, consistency
+//! - [`ReviewType::Code`]: spec compliance, security, test coverage, style
+
+use crate::agents::Agent;
+use crate::error::{NovaError, NovaResult};
+use crate::llm::{CompletionRequest, LLMProvider};
+use crate::stream::StreamHandler;
+use crate::structured::complete_structured;
+use crate::types::Message;
+use async_trait::async_trait;
+use serde::{Deserialize, Serialize};
+use std::sync::Arc;
+
+// ============================================================
+// Review types
+// ============================================================
+
+/// Severity level of a review issue.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum Severity {
+    High,
+    Medium,
+    Low,
+}
+
+impl std::fmt::Display for Severity {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        match self {
+            Severity::High => write!(f, "High"),
+            Severity::Medium => write!(f, "Medium"),
+            Severity::Low => write!(f, "Low"),
+        }
+    }
+}
+
+/// A single issue found during review.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct ReviewIssue {
+    pub severity: Severity,
+    pub description: String,
+    pub suggestion: String,
+    /// Optional location hint — e.g., `"file.rs:42"` or `"spec/interfaces"`.
+    pub location: Option<String>,
+}
+
+/// Verdict returned by a reviewer.
+///
+/// Serializes with a `"verdict"` discriminant tag:
+/// - `{"verdict": "approved"}`
+/// - `{"verdict": "needs_revision", "issues": [...]}`
+/// - `{"verdict": "rejected", "reason": "..."}`
+#[derive(Debug, Clone, Serialize, Deserialize)]
+#[serde(tag = "verdict", rename_all = "snake_case")]
+pub enum ReviewVerdict {
+    Approved,
+    NeedsRevision { issues: Vec<ReviewIssue> },
+    Rejected { reason: String },
+}
+
+impl ReviewVerdict {
+    pub fn is_approved(&self) -> bool {
+        matches!(self, ReviewVerdict::Approved)
+    }
+
+    pub fn is_rejected(&self) -> bool {
+        matches!(self, ReviewVerdict::Rejected { .. })
+    }
+
+    pub fn needs_revision(&self) -> bool {
+        matches!(self, ReviewVerdict::NeedsRevision { .. })
+    }
+
+    /// Return the issues if verdict is `NeedsRevision`, otherwise an empty slice.
+    pub fn issues(&self) -> &[ReviewIssue] {
+        match self {
+            ReviewVerdict::NeedsRevision { issues } => issues,
+            _ => &[],
+        }
+    }
+}
+
+/// Type of review to perform.
+#[derive(Debug, Clone)]
+pub enum ReviewType {
+    /// Review a spec document for format, quality, completeness, and consistency.
+    Spec,
+    /// Review code for spec compliance, security, test coverage, and style.
+    Code,
+}
+
+// ============================================================
+// Reviewer trait
+// ============================================================
+
+/// Trait for types that can review an artifact and return a structured verdict.
+///
+/// Implemented by [`ReviewAgent`]; also usable as a seam for mock reviewers in tests.
+#[async_trait]
+pub trait Reviewer: Send + Sync {
+    async fn review(&self, artifact: &str) -> NovaResult<ReviewVerdict>;
+}
+
+// ============================================================
+// Internal deserialization helpers
+// ============================================================
+
+/// Raw JSON structure returned by the LLM's structured output.
+#[derive(Debug, Deserialize)]
+struct ReviewResponseRaw {
+    verdict: String,
+    #[serde(default)]
+    issues: Vec<ReviewIssueRaw>,
+    rejection_reason: Option<String>,
+}
+
+#[derive(Debug, Deserialize)]
+struct ReviewIssueRaw {
+    severity: String,
+    description: String,
+    suggestion: String,
+    location: Option<String>,
+}
+
+// ============================================================
+// ReviewAgentConfig
+// ============================================================
+
+/// Configuration for [`ReviewAgent`].
+#[derive(Debug, Clone)]
+pub struct ReviewAgentConfig {
+    pub model: String,
+    pub review_type: ReviewType,
+    pub temperature: Option<f32>,
+    pub max_tokens: Option<u32>,
+    /// Additional instructions appended to the system prompt.
+    pub extra_instructions: Option<String>,
+}
+
+impl Default for ReviewAgentConfig {
+    fn default() -> Self {
+        Self {
+            model: "claude-sonnet-4-20250514".to_string(),
+            review_type: ReviewType::Spec,
+            temperature: Some(0.0),
+            max_tokens: Some(4096),
+            extra_instructions: None,
+        }
+    }
+}
+
+// ============================================================
+// ReviewAgent
+// ============================================================
+
+/// An LLM-powered review agent that returns structured [`ReviewVerdict`]s.
+///
+/// # Example
+///
+/// ```rust,ignore
+/// use cclab_agent::{ReviewAgent, ReviewType, ClaudeProvider};
+///
+/// let provider = ClaudeProvider::new(std::env::var("ANTHROPIC_API_KEY")?)?;
+/// let agent = ReviewAgent::builder()
+///     .with_provider(provider)
+///     .with_review_type(ReviewType::Spec)
+///     .build()?;
+///
+/// let verdict = agent.review("## API Spec\n...").await?;
+/// ```
+pub struct ReviewAgent {
+    config: ReviewAgentConfig,
+    provider: Arc<dyn LLMProvider>,
+}
+
+impl std::fmt::Debug for ReviewAgent {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        f.debug_struct("ReviewAgent")
+            .field("model", &self.config.model)
+            .finish_non_exhaustive()
+    }
+}
+
+impl ReviewAgent {
+    /// Create a new builder.
+    pub fn builder() -> ReviewAgentBuilder {
+        ReviewAgentBuilder::new()
+    }
+
+    /// Review `artifact` and return a structured verdict.
+    ///
+    /// Convenience method — equivalent to calling via the [`Reviewer`] trait.
+    pub async fn review(&self, artifact: &str) -> NovaResult<ReviewVerdict> {
+        self.review_impl(artifact).await
+    }
+
+    // ---- private helpers ----
+
+    async fn review_impl(&self, artifact: &str) -> NovaResult<ReviewVerdict> {
+        let system_prompt = self.build_system_prompt();
+        let schema = Self::review_schema();
+
+        let messages = vec![
+            Message::system(&system_prompt),
+            Message::user(artifact),
+        ];
+        let request = self.build_completion_request(messages);
+
+        let (_response, value) =
+            complete_structured(&*self.provider, request, &schema, 2).await?;
+
+        self.parse_verdict(value)
+    }
+
+    fn build_system_prompt(&self) -> String {
+        let base = match self.config.review_type {
+            ReviewType::Spec => SPEC_REVIEW_SYSTEM_PROMPT,
+            ReviewType::Code => CODE_REVIEW_SYSTEM_PROMPT,
+        };
+        match &self.config.extra_instructions {
+            Some(extra) => format!("{}\n\n{}", base, extra),
+            None => base.to_string(),
+        }
+    }
+
+    fn build_completion_request(&self, messages: Vec<Message>) -> CompletionRequest {
+        let mut request = CompletionRequest::new(messages, &self.config.model);
+        if let Some(temp) = self.config.temperature {
+            request = request.with_temperature(temp);
+        }
+        if let Some(max_tokens) = self.config.max_tokens {
+            request = request.with_max_tokens(max_tokens);
+        }
+        request
+    }
+
+    /// JSON Schema for structured review output.
+    fn review_schema() -> serde_json::Value {
+        serde_json::json!({
+            "type": "object",
+            "properties": {
+                "verdict": {
+                    "type": "string",
+                    "enum": ["approved", "needs_revision", "rejected"]
+                },
+                "issues": {
+                    "type": "array",
+                    "items": {
+                        "type": "object",
+                        "properties": {
+                            "severity": {
+                                "type": "string",
+                                "enum": ["high", "medium", "low"]
+                            },
+                            "description": { "type": "string" },
+                            "suggestion":  { "type": "string" },
+                            "location":    { "type": "string" }
+                        },
+                        "required": ["severity", "description", "suggestion"]
+                    }
+                },
+                "rejection_reason": { "type": "string" }
+            },
+            "required": ["verdict"]
+        })
+    }
+
+    fn parse_verdict(&self, value: serde_json::Value) -> NovaResult<ReviewVerdict> {
+        let raw: ReviewResponseRaw = serde_json::from_value(value)?;
+
+        match raw.verdict.as_str() {
+            "approved" => Ok(ReviewVerdict::Approved),
+
+            "needs_revision" => {
+                let issues = raw
+                    .issues
+                    .into_iter()
+                    .map(|r| {
+                        let severity = match r.severity.as_str() {
+                            "high" => Severity::High,
+                            "medium" => Severity::Medium,
+                            _ => Severity::Low,
+                        };
+                        ReviewIssue {
+                            severity,
+                            description: r.description,
+                            suggestion: r.suggestion,
+                            location: r.location,
+                        }
+                    })
+                    .collect();
+                Ok(ReviewVerdict::NeedsRevision { issues })
+            }
+
+            "rejected" => {
+                let reason = raw
+                    .rejection_reason
+                    .unwrap_or_else(|| "Rejected by reviewer".to_string());
+                Ok(ReviewVerdict::Rejected { reason })
+            }
+
+            other => Err(NovaError::InvalidRequest(format!(
+                "Unknown verdict type: '{}'",
+                other
+            ))),
+        }
+    }
+}
+
+/// [`ReviewAgent`] implements [`Agent`]: `run(artifact)` returns the verdict as JSON.
+#[async_trait]
+impl Agent for ReviewAgent {
+    async fn run(&self, input: &str) -> NovaResult<String> {
+        let verdict = self.review_impl(input).await?;
+        serde_json::to_string(&verdict).map_err(NovaError::SerializationError)
+    }
+
+    async fn run_with_handler(
+        &self,
+        input: &str,
+        _handler: &dyn StreamHandler,
+    ) -> NovaResult<String> {
+        self.run(input).await
+    }
+}
+
+/// [`ReviewAgent`] implements [`Reviewer`].
+#[async_trait]
+impl Reviewer for ReviewAgent {
+    async fn review(&self, artifact: &str) -> NovaResult<ReviewVerdict> {
+        self.review_impl(artifact).await
+    }
+}
+
+// ============================================================
+// System prompts
+// ============================================================
+
+const SPEC_REVIEW_SYSTEM_PROMPT: &str = r#"You are an expert spec reviewer. Review the provided spec document and return a structured JSON verdict.
+
+Evaluate the spec on:
+1. **Format compliance**: OpenRPC / JSON Schema / Mermaid are preferred over prose. Structured formats must be used wherever applicable.
+2. **Diagram correctness**: The right diagram type must be used for the structure being modeled (stateDiagram-v2 for FSMs, flowchart for DAGs, sequenceDiagram for actor interactions, erDiagram for entity relationships, classDiagram for type hierarchies).
+3. **Quality**: Less than 10% of the spec should be plain prose. No real code should be embedded inline.
+4. **Completeness**: All required sections must be present (interfaces, logic, API definitions, etc.).
+5. **Consistency**: Naming conventions must be consistent throughout the spec.
+
+Return one of:
+- `"approved"` — spec meets all standards.
+- `"needs_revision"` — fixable issues found; include a list of issues with severity (high/medium/low), description, suggestion, and optional location.
+- `"rejected"` — fundamental problems requiring a full rewrite; include a rejection_reason."#;
+
+const CODE_REVIEW_SYSTEM_PROMPT: &str = r#"You are an expert code reviewer. Review the provided code and return a structured JSON verdict.
+
+Evaluate the code on:
+1. **Spec compliance**: Does the code implement exactly what the spec defines? Flag missing features, undocumented extras, and incorrect behavior.
+2. **Security**: Check for OWASP Top 10 vulnerabilities (injection, broken auth, XSS, insecure deserialization, etc.).
+3. **Test coverage**: Are edge cases covered? Are tests meaningful and not just smoke tests?
+4. **Style consistency**: Does the code follow existing codebase patterns and naming conventions?
+
+Return one of:
+- `"approved"` — code meets all standards.
+- `"needs_revision"` — fixable issues found; include a list of issues with severity (high/medium/low), description, suggestion, and optional location (file:line or module path).
+- `"rejected"` — fundamental architectural problems requiring a rewrite; include a rejection_reason."#;
+
+// ============================================================
+// ReviewAgentBuilder
+// ============================================================
+
+/// Builder for [`ReviewAgent`].
+pub struct ReviewAgentBuilder {
+    config: ReviewAgentConfig,
+    provider: Option<Arc<dyn LLMProvider>>,
+}
+
+impl ReviewAgentBuilder {
+    pub fn new() -> Self {
+        Self {
+            config: ReviewAgentConfig::default(),
+            provider: None,
+        }
+    }
+
+    pub fn with_provider<P: LLMProvider + 'static>(mut self, provider: P) -> Self {
+        self.provider = Some(Arc::new(provider));
+        self
+    }
+
+    pub fn with_provider_arc(mut self, provider: Arc<dyn LLMProvider>) -> Self {
+        self.provider = Some(provider);
+        self
+    }
+
+    pub fn with_review_type(mut self, review_type: ReviewType) -> Self {
+        self.config.review_type = review_type;
+        self
+    }
+
+    pub fn with_model(mut self, model: impl Into<String>) -> Self {
+        self.config.model = model.into();
+        self
+    }
+
+    pub fn with_temperature(mut self, temperature: f32) -> Self {
+        self.config.temperature = Some(temperature);
+        self
+    }
+
+    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
+        self.config.max_tokens = Some(max_tokens);
+        self
+    }
+
+    pub fn with_extra_instructions(mut self, instructions: impl Into<String>) -> Self {
+        self.config.extra_instructions = Some(instructions.into());
+        self
+    }
+
+    pub fn build(self) -> NovaResult<ReviewAgent> {
+        let provider = self
+            .provider
+            .ok_or_else(|| NovaError::ConfigError("LLM provider is required".to_string()))?;
+        Ok(ReviewAgent {
+            config: self.config,
+            provider,
+        })
+    }
+}
+
+impl Default for ReviewAgentBuilder {
+    fn default() -> Self {
+        Self::new()
+    }
+}
+
+// ============================================================
+// Tests
+// ============================================================
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_review_schema_has_verdict_enum() {
+        let schema = ReviewAgent::review_schema();
+        let verdicts = &schema["properties"]["verdict"]["enum"];
+        assert!(verdicts.is_array());
+        let arr = verdicts.as_array().unwrap();
+        assert!(arr.iter().any(|v| v == "approved"));
+        assert!(arr.iter().any(|v| v == "needs_revision"));
+        assert!(arr.iter().any(|v| v == "rejected"));
+    }
+
+    #[test]
+    fn test_review_schema_required_verdict() {
+        let schema = ReviewAgent::review_schema();
+        assert_eq!(schema["required"][0], "verdict");
+    }
+
+    #[test]
+    fn test_verdict_helpers_approved() {
+        let v = ReviewVerdict::Approved;
+        assert!(v.is_approved());
+        assert!(!v.is_rejected());
+        assert!(!v.needs_revision());
+        assert!(v.issues().is_empty());
+    }
+
+    #[test]
+    fn test_verdict_helpers_needs_revision() {
+        let issue = ReviewIssue {
+            severity: Severity::High,
+            description: "Missing interface section".to_string(),
+            suggestion: "Add interfaces/ directory with OpenRPC definitions".to_string(),
+            location: Some("cclab/specs/foo/".to_string()),
+        };
+        let v = ReviewVerdict::NeedsRevision { issues: vec![issue] };
+        assert!(v.needs_revision());
+        assert!(!v.is_approved());
+        assert!(!v.is_rejected());
+        assert_eq!(v.issues().len(), 1);
+    }
+
+    #[test]
+    fn test_verdict_helpers_rejected() {
+        let v = ReviewVerdict::Rejected {
+            reason: "Fundamental design flaw".to_string(),
+        };
+        assert!(v.is_rejected());
+        assert!(!v.is_approved());
+        assert!(!v.needs_revision());
+    }
+
+    #[test]
+    fn test_severity_display() {
+        assert_eq!(Severity::High.to_string(), "High");
+        assert_eq!(Severity::Medium.to_string(), "Medium");
+        assert_eq!(Severity::Low.to_string(), "Low");
+    }
+
+    #[test]
+    fn test_verdict_serialization_approved() {
+        let v = ReviewVerdict::Approved;
+        let json = serde_json::to_string(&v).unwrap();
+        assert!(json.contains("\"verdict\":\"approved\""));
+    }
+
+    #[test]
+    fn test_verdict_serialization_needs_revision() {
+        let v = ReviewVerdict::NeedsRevision { issues: vec![] };
+        let json = serde_json::to_string(&v).unwrap();
+        assert!(json.contains("\"verdict\":\"needs_revision\""));
+    }
+
+    #[test]
+    fn test_verdict_serialization_rejected() {
+        let v = ReviewVerdict::Rejected {
+            reason: "bad".to_string(),
+        };
+        let json = serde_json::to_string(&v).unwrap();
+        assert!(json.contains("\"verdict\":\"rejected\""));
+        assert!(json.contains("\"reason\":\"bad\""));
+    }
+
+    #[test]
+    fn test_builder_missing_provider() {
+        let result = ReviewAgentBuilder::new()
+            .with_review_type(ReviewType::Code)
+            .build();
+        assert!(result.is_err());
+        assert!(matches!(result.unwrap_err(), NovaError::ConfigError(_)));
+    }
+
+    #[test]
+    fn test_verdict_deserialization_round_trip() {
+        let original = ReviewVerdict::NeedsRevision {
+            issues: vec![ReviewIssue {
+                severity: Severity::Medium,
+                description: "desc".to_string(),
+                suggestion: "fix it".to_string(),
+                location: Some("src/lib.rs:10".to_string()),
+            }],
+        };
+        let json = serde_json::to_string(&original).unwrap();
+        let restored: ReviewVerdict = serde_json::from_str(&json).unwrap();
+        assert!(restored.needs_revision());
+        assert_eq!(restored.issues().len(), 1);
+        assert_eq!(restored.issues()[0].severity, Severity::Medium);
+    }
+}

```

## Review: agent-crr-review-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: agent-crr-review

**Summary**: Implementation complete. CRRCycle with creator/reviewer/reviser, verdict routing, max_revisions, event callbacks. ReviewAgent with ReviewType (Spec/Code), structured verdict output. 19 new tests all passing. 124/124 total tests pass.

