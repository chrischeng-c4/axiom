// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
// CODEGEN-BEGIN
//! CRRCycle — generic Create-Review-Revise orchestration pattern.
//!
//! Wires a creator agent, a reviewer, and a reviser agent into a loop:
//!
//! ```text
//! Create → Review → verdict
//!   ↑                  ↓
//!   └── Revise ← NEEDS_REVISION
//!               APPROVED  → done
//!               REJECTED  → Err
//!               max_revisions exceeded → Err(MaxRevisionsExceeded)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::{CRRCycle, ReviewAgent, ReviewType, CodingAgent, ClaudeProvider};
//!
//! let review_agent = ReviewAgent::builder()
//!     .with_provider(provider.clone())
//!     .with_review_type(ReviewType::Spec)
//!     .build()?;
//!
//! let crr = CRRCycle::new()
//!     .creator(spec_agent.clone())
//!     .reviewer(review_agent)
//!     .reviser(spec_agent)
//!     .max_revisions(2)
//!     .on_event(|event| eprintln!("[CRR] {:?}", event))
//!     .build()?;
//!
//! let result = crr.run("Create a spec for the login flow").await?;
//! println!("Artifact:\n{}", result.artifact);
//! println!("Revisions: {}", result.revision_count);
//! ```

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
use crate::agents::{
    review::{ReviewIssue, ReviewVerdict, Reviewer},
    Agent,
};
use agent::error::{NovaError, NovaResult};
use std::sync::Arc;

// ============================================================
// CRR events
// ============================================================

/// Compact discriminant carried in CRREvent::Reviewed.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CRRVerdictType {
    Approved,
    NeedsRevision,
    Rejected,
}

/// Events emitted during a CRRCycle run.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#schema
#[derive(Debug, Clone)]
pub enum CRREvent {
    /// Creator agent is about to run.
    Creating,
    /// Creator produced an artifact.
    Created { artifact: String },
    /// Reviewer is about to review the current artifact.
    Reviewing { revision: u32 },
    /// Reviewer produced a verdict.
    Reviewed { verdict_type: CRRVerdictType },
    /// Reviser is about to address issues from a revision.
    Revising { revision: u32, issue_count: usize },
    /// Reviser produced a revised artifact.
    Revised { artifact: String },
    /// Cycle completed successfully.
    Completed {
        artifact: String,
        revision_count: u32,
    },
}

/// Output of a completed CRRCycle run.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#schema
#[derive(Debug)]
pub struct CRRResult {
    /// Final review verdict.
    pub verdict: ReviewVerdict,
    /// The final artifact string produced by the cycle.
    pub artifact: String,
    /// Number of revisions performed.
    pub revision_count: u32,
}

/// Generic Create-Review-Revise orchestrator.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#schema
pub struct CRRCycle {
    /// Creator agent.
    creator: Arc<dyn Agent>,
    /// Reviewer.
    reviewer: Arc<dyn Reviewer>,
    /// Reviser agent.
    reviser: Arc<dyn Agent>,
    /// Maximum revision rounds.
    max_revisions: u32,
    /// Optional event handler closure.
    event_handler: Option<Arc<dyn Fn(CRREvent) + Send + Sync>>,
}

/// Builder for CRRCycle.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#schema
pub struct CRRCycleBuilder {
    /// Optional creator agent.
    creator: Option<Arc<dyn Agent>>,
    /// Optional reviewer.
    reviewer: Option<Arc<dyn Reviewer>>,
    /// Optional reviser agent.
    reviser: Option<Arc<dyn Agent>>,
    /// Maximum revision rounds.
    max_revisions: u32,
    /// Optional event handler closure.
    event_handler: Option<Arc<dyn Fn(CRREvent) + Send + Sync>>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
impl From<&ReviewVerdict> for CRRVerdictType {
    fn from(v: &ReviewVerdict) -> Self {
        match v {
            ReviewVerdict::Approved => CRRVerdictType::Approved,
            ReviewVerdict::NeedsRevision { .. } => CRRVerdictType::NeedsRevision,
            ReviewVerdict::Rejected { .. } => CRRVerdictType::Rejected,
        }
    }
}

// ============================================================
// CRRResult
// ============================================================

// ============================================================
// CRRCycle
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
impl std::fmt::Debug for CRRCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CRRCycle")
            .field("max_revisions", &self.max_revisions)
            .finish_non_exhaustive()
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
impl CRRCycle {
    /// Start building a [`CRRCycle`].  Returns a [`CRRCycleBuilder`].
    pub fn new() -> CRRCycleBuilder {
        CRRCycleBuilder::default()
    }

    /// Run the full CRR loop starting from `input`.
    ///
    /// # Errors
    ///
    /// - [`NovaError::MaxRevisionsExceeded`] if the reviewer keeps returning
    ///   `NeedsRevision` after `max_revisions` rounds.
    /// - [`NovaError::Other`] if the reviewer returns `Rejected`.
    pub async fn run(&self, input: &str) -> NovaResult<CRRResult> {
        self.emit(CRREvent::Creating);

        // 1. Create initial artifact
        let initial = self.creator.run(input).await?;
        self.emit(CRREvent::Created {
            artifact: initial.clone(),
        });

        let mut artifact = initial;
        let mut revision_count: u32 = 0;

        loop {
            // 2. Review current artifact
            self.emit(CRREvent::Reviewing {
                revision: revision_count,
            });
            let verdict = self.reviewer.review(&artifact).await?;
            self.emit(CRREvent::Reviewed {
                verdict_type: CRRVerdictType::from(&verdict),
            });

            match verdict {
                ReviewVerdict::Approved => {
                    self.emit(CRREvent::Completed {
                        artifact: artifact.clone(),
                        revision_count,
                    });
                    return Ok(CRRResult {
                        verdict: ReviewVerdict::Approved,
                        artifact,
                        revision_count,
                    });
                }

                ReviewVerdict::Rejected { ref reason } => {
                    return Err(NovaError::Other(anyhow::anyhow!(
                        "CRR cycle rejected after {} revision(s): {}",
                        revision_count,
                        reason
                    )));
                }

                ReviewVerdict::NeedsRevision { ref issues } => {
                    if revision_count >= self.max_revisions {
                        return Err(NovaError::MaxRevisionsExceeded(self.max_revisions));
                    }

                    let issue_count = issues.len();
                    self.emit(CRREvent::Revising {
                        revision: revision_count,
                        issue_count,
                    });

                    // 3. Revise: build a prompt that carries artifact + issues
                    let revise_prompt = build_revision_prompt(&artifact, issues);
                    let revised = self.reviser.run(&revise_prompt).await?;

                    self.emit(CRREvent::Revised {
                        artifact: revised.clone(),
                    });

                    artifact = revised;
                    revision_count += 1;
                }
            }
        }
    }

    // ---- private helpers ----

    fn emit(&self, event: CRREvent) {
        if let Some(ref handler) = self.event_handler {
            handler(event);
        }
    }
}

/// Build a revision prompt that instructs the reviser agent what to fix.
fn build_revision_prompt(artifact: &str, issues: &[ReviewIssue]) -> String {
    let mut prompt = String::from(
        "Revise the following artifact based on the review issues listed below.\n\n\
         ## Original Artifact\n\n",
    );
    prompt.push_str(artifact);
    prompt.push_str("\n\n## Review Issues\n");

    for (i, issue) in issues.iter().enumerate() {
        prompt.push_str(&format!(
            "\n{}. [{}] {}\n   Suggestion: {}\n",
            i + 1,
            issue.severity,
            issue.description,
            issue.suggestion,
        ));
        if let Some(ref loc) = issue.location {
            prompt.push_str(&format!("   Location: {}\n", loc));
        }
    }

    prompt.push_str("\nPlease address every issue above and return the fully revised artifact.");
    prompt
}

// ============================================================
// CRRCycleBuilder
// ============================================================

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
impl Default for CRRCycleBuilder {
    fn default() -> Self {
        Self {
            creator: None,
            reviewer: None,
            reviser: None,
            max_revisions: 3,
            event_handler: None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/agents/crr.md#source
impl CRRCycleBuilder {
    /// Set the creator agent (produces the initial artifact from the input).
    pub fn creator<A: Agent + 'static>(mut self, agent: A) -> Self {
        self.creator = Some(Arc::new(agent));
        self
    }

    /// Set the creator agent from an `Arc`.
    pub fn creator_arc(mut self, agent: Arc<dyn Agent>) -> Self {
        self.creator = Some(agent);
        self
    }

    /// Set the reviewer (returns a [`ReviewVerdict`]).
    pub fn reviewer<R: Reviewer + 'static>(mut self, reviewer: R) -> Self {
        self.reviewer = Some(Arc::new(reviewer));
        self
    }

    /// Set the reviewer from an `Arc`.
    pub fn reviewer_arc(mut self, reviewer: Arc<dyn Reviewer>) -> Self {
        self.reviewer = Some(reviewer);
        self
    }

    /// Set the reviser agent (receives original artifact + issues, returns revised artifact).
    pub fn reviser<A: Agent + 'static>(mut self, agent: A) -> Self {
        self.reviser = Some(Arc::new(agent));
        self
    }

    /// Set the reviser agent from an `Arc`.
    pub fn reviser_arc(mut self, agent: Arc<dyn Agent>) -> Self {
        self.reviser = Some(agent);
        self
    }

    /// Maximum number of revise → review rounds before [`NovaError::MaxRevisionsExceeded`].
    ///
    /// Default: `3`.
    pub fn max_revisions(mut self, n: u32) -> Self {
        self.max_revisions = n;
        self
    }

    /// Attach a sync event callback for logging, SSE forwarding, etc.
    pub fn on_event<F: Fn(CRREvent) + Send + Sync + 'static>(mut self, handler: F) -> Self {
        self.event_handler = Some(Arc::new(handler));
        self
    }

    /// Build the [`CRRCycle`].
    ///
    /// # Errors
    ///
    /// Returns [`NovaError::ConfigError`] if creator, reviewer, or reviser is missing.
    pub fn build(self) -> NovaResult<CRRCycle> {
        let creator = self
            .creator
            .ok_or_else(|| NovaError::ConfigError("CRRCycle: creator is required".to_string()))?;
        let reviewer = self
            .reviewer
            .ok_or_else(|| NovaError::ConfigError("CRRCycle: reviewer is required".to_string()))?;
        let reviser = self
            .reviser
            .ok_or_else(|| NovaError::ConfigError("CRRCycle: reviser is required".to_string()))?;

        Ok(CRRCycle {
            creator,
            reviewer,
            reviser,
            max_revisions: self.max_revisions,
            event_handler: self.event_handler,
        })
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::review::{ReviewIssue, ReviewVerdict, Reviewer, Severity};
    use agent::error::NovaResult;
    use async_trait::async_trait;

    // ---- mock agents ----

    struct ConstAgent(String);

    #[async_trait]
    impl Agent for ConstAgent {
        async fn run(&self, _input: &str) -> NovaResult<String> {
            Ok(self.0.clone())
        }
        async fn run_with_handler(
            &self,
            input: &str,
            _handler: &dyn agent::stream::StreamHandler,
        ) -> NovaResult<String> {
            self.run(input).await
        }
    }

    /// A reviewer whose verdict sequence is fixed at construction.
    struct ScriptedReviewer {
        verdicts: std::sync::Mutex<Vec<ReviewVerdict>>,
    }

    impl ScriptedReviewer {
        fn new(verdicts: Vec<ReviewVerdict>) -> Self {
            Self {
                verdicts: std::sync::Mutex::new(verdicts),
            }
        }
    }

    #[async_trait]
    impl Reviewer for ScriptedReviewer {
        async fn review(&self, _artifact: &str) -> NovaResult<ReviewVerdict> {
            let mut v = self.verdicts.lock().unwrap();
            if v.is_empty() {
                Ok(ReviewVerdict::Approved)
            } else {
                Ok(v.remove(0))
            }
        }
    }

    fn make_issue(msg: &str) -> ReviewIssue {
        ReviewIssue {
            severity: Severity::Medium,
            description: msg.to_string(),
            suggestion: "fix it".to_string(),
            location: None,
        }
    }

    // ---- tests ----

    #[tokio::test]
    async fn test_crr_approved_on_first_review() {
        let crr = CRRCycle::new()
            .creator(ConstAgent("artifact-v1".to_string()))
            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Approved]))
            .reviser(ConstAgent("should-not-be-called".to_string()))
            .max_revisions(2)
            .build()
            .unwrap();

        let result = crr.run("input").await.unwrap();
        assert_eq!(result.artifact, "artifact-v1");
        assert_eq!(result.revision_count, 0);
        assert!(result.verdict.is_approved());
    }

    #[tokio::test]
    async fn test_crr_two_revisions_then_approved() {
        let crr = CRRCycle::new()
            .creator(ConstAgent("v1".to_string()))
            .reviewer(ScriptedReviewer::new(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_issue("issue 1")],
                },
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_issue("issue 2")],
                },
                ReviewVerdict::Approved,
            ]))
            .reviser(ConstAgent("revised".to_string()))
            .max_revisions(3)
            .build()
            .unwrap();

        let result = crr.run("input").await.unwrap();
        assert!(result.verdict.is_approved());
        assert_eq!(result.revision_count, 2);
    }

    #[tokio::test]
    async fn test_crr_max_revisions_exceeded() {
        let crr = CRRCycle::new()
            .creator(ConstAgent("v1".to_string()))
            .reviewer(ScriptedReviewer::new(vec![
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_issue("persistent issue")],
                },
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_issue("still broken")],
                },
                ReviewVerdict::NeedsRevision {
                    issues: vec![make_issue("never fixed")],
                },
            ]))
            .reviser(ConstAgent("still-broken".to_string()))
            .max_revisions(2)
            .build()
            .unwrap();

        let err = crr.run("input").await.unwrap_err();
        assert!(
            matches!(err, NovaError::MaxRevisionsExceeded(2)),
            "expected MaxRevisionsExceeded(2), got {:?}",
            err
        );
    }

    #[tokio::test]
    async fn test_crr_rejected() {
        let crr = CRRCycle::new()
            .creator(ConstAgent("v1".to_string()))
            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Rejected {
                reason: "fundamentally wrong".to_string(),
            }]))
            .reviser(ConstAgent("irrelevant".to_string()))
            .max_revisions(2)
            .build()
            .unwrap();

        let err = crr.run("input").await.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("rejected"),
            "expected rejected error, got: {}",
            msg
        );
    }

    #[tokio::test]
    async fn test_crr_events_emitted() {
        use std::sync::Mutex;
        let events: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
        let events_clone = events.clone();

        let crr = CRRCycle::new()
            .creator(ConstAgent("artifact".to_string()))
            .reviewer(ScriptedReviewer::new(vec![ReviewVerdict::Approved]))
            .reviser(ConstAgent("unused".to_string()))
            .max_revisions(1)
            .on_event(move |e| {
                events_clone.lock().unwrap().push(format!("{:?}", e));
            })
            .build()
            .unwrap();

        crr.run("input").await.unwrap();

        let recorded = events.lock().unwrap();
        // Expect at least: Creating, Created, Reviewing, Reviewed, Completed
        assert!(recorded.iter().any(|s| s.contains("Creating")));
        assert!(recorded.iter().any(|s| s.contains("Created")));
        assert!(recorded.iter().any(|s| s.contains("Reviewing")));
        assert!(recorded.iter().any(|s| s.contains("Reviewed")));
        assert!(recorded.iter().any(|s| s.contains("Completed")));
    }

    #[test]
    fn test_builder_missing_creator() {
        let err = CRRCycle::new()
            .reviewer(ScriptedReviewer::new(vec![]))
            .reviser(ConstAgent("x".to_string()))
            .build()
            .unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_builder_missing_reviewer() {
        let err = CRRCycle::new()
            .creator(ConstAgent("x".to_string()))
            .reviser(ConstAgent("x".to_string()))
            .build()
            .unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_builder_missing_reviser() {
        let err = CRRCycle::new()
            .creator(ConstAgent("x".to_string()))
            .reviewer(ScriptedReviewer::new(vec![]))
            .build()
            .unwrap_err();
        assert!(matches!(err, NovaError::ConfigError(_)));
    }

    #[test]
    fn test_build_revision_prompt_contains_issues() {
        let issues = vec![ReviewIssue {
            severity: Severity::High,
            description: "Missing section".to_string(),
            suggestion: "Add it".to_string(),
            location: Some("spec/interfaces.md".to_string()),
        }];
        let prompt = build_revision_prompt("original content", &issues);
        assert!(prompt.contains("original content"));
        assert!(prompt.contains("Missing section"));
        assert!(prompt.contains("Add it"));
        assert!(prompt.contains("spec/interfaces.md"));
        assert!(prompt.contains("High"));
    }

    #[test]
    fn test_crr_verdict_type_from_verdict() {
        assert_eq!(
            CRRVerdictType::from(&ReviewVerdict::Approved),
            CRRVerdictType::Approved
        );
        assert_eq!(
            CRRVerdictType::from(&ReviewVerdict::NeedsRevision { issues: vec![] }),
            CRRVerdictType::NeedsRevision
        );
        assert_eq!(
            CRRVerdictType::from(&ReviewVerdict::Rejected {
                reason: "x".to_string()
            }),
            CRRVerdictType::Rejected
        );
    }
}

// CODEGEN-END
