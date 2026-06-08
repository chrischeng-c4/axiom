// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types_preamble_source.md#source
// CODEGEN-BEGIN
//! Issue artifact types — uniform across all backends.
//!
//! These types are the wire format for the `IssueBackend` trait and the
//! serialization format for temp-backed issue working-copy frontmatter.

use std::collections::HashMap;
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// An issue artifact — the same shape whether it comes from local files, GitHub, GitLab, Jira, or an agent-authored draft.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue kind. Maps 1:1 to GitHub type:* labels.
    #[serde(rename = "type")]
    pub issue_type: IssueType,
    /// Full title (preserves prefixes like 'score(2.5):' for human context).
    pub title: String,
    /// Lifecycle state.
    pub state: IssueState,
    /// Local UUID — stable identifier across worktrees. Auto-generated on create.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// GitHub issue number. Backfilled after push/sync.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub github_id: Option<u64>,
    /// GitLab issue iid. Backfilled after push/sync.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gitlab_id: Option<u64>,
    /// Web URL where the issue lives on the tracker (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Author username.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// All labels (including the type:* one — kept for fidelity to the backend).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
    /// ISO 8601 creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 last-update timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Slug — the canonical local identifier (filename stem for local backend). Not serialized to frontmatter because it lives in the filename.
    #[serde(skip)]
    pub slug: String,
    /// Markdown body (everything after frontmatter). Not stored in the frontmatter itself.
    #[serde(skip)]
    pub body: String,
    /// Soft see-also references to other issues, BRDs, PRDs (slugs or paths).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    /// Hard references to changes or tech designs that realize this issue.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,
    /// SDD workflow phase. None means no active SDD change for this issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Git branch for this change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Target branch to merge this issue's worktree into. Optional override per issue-merge-target; when None, resolution falls through to current branch then config.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_branch: Option<String>,
    /// How the branch was created: 'new_branch' or 'in_place'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_workflow: Option<String>,
    /// Change identifier — links this issue to .aw/changes/{change_id}/.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change_id: Option<String>,
    /// Re-proposal iteration counter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iteration: Option<u32>,
    /// Current spec being implemented (per-task workflow).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_task_id: Option<String>,
    /// Per-spec implementation phase: spec_id to 'code' or 'tests'.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub impl_spec_phase: Option<HashMap<String, String>>,
    /// Per-task revision counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_revisions: Option<HashMap<String, u32>>,
    /// Per-phase revision counts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision_counts: Option<HashMap<String, u32>>,
    /// Last workflow action performed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_action: Option<String>,
    /// Session ID for agent resume-by-index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// CRR validation errors — set by aw wi validate, cleared on pass.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<String>,
    /// CRRR review count — number of score-issue-reviewer runs on this issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_count: Option<u8>,
    /// Sections flagged by the most recent needs-revision review.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flagged_sections: Option<Vec<IssueSection>>,
    /// Loop-fill retry counter. Reset to None on successful phase advance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fill_retry_count: Option<u8>,
    /// Ship lifecycle status for merged issues.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ship_status: Option<ShipStatus>,
    /// Git commit hash written by validate when phase first advances to td_merged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ship_commit: Option<String>,
    /// Git commit hash recorded when validate confirmed regen byte-equivalence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regen_verified_at: Option<String>,
}

/// Structured error codes for agent-first JSON error output.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueErrorCode {
    /// Resource not found.
    NotFound,
    /// Validation failure.
    Validation,
    /// Backend error.
    Backend,
}

/// Filter criteria for IssueBackend::list. All fields are conjunctive.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    /// Filter by lifecycle state.
    pub state: Option<IssueState>,
    /// Filter by issue kind.
    pub issue_type: Option<IssueType>,
    /// Match if the issue has this label (exact).
    pub label: Option<String>,
    /// Match if the issue's author equals this.
    pub author: Option<String>,
}

/// Partial update descriptor for IssueBackend::update. Only non-None or non-empty fields are applied.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Default)]
pub struct IssuePatch {
    /// New title to set.
    pub title: Option<String>,
    /// New state to set.
    pub state: Option<IssueState>,
    /// Labels to add.
    pub add_labels: Vec<String>,
    /// Labels to remove.
    pub remove_labels: Vec<String>,
    /// New body to set.
    pub body: Option<String>,
    /// Update SDD phase (issue-centric workflow).
    pub phase: Option<String>,
    /// Clear SDD phase without writing a replacement.
    pub clear_phase: bool,
    /// Update branch (issue-centric workflow).
    pub branch: Option<String>,
    /// Update target merge branch.
    pub target_branch: Option<String>,
    /// Update git_workflow (issue-centric workflow).
    pub git_workflow: Option<String>,
    /// Transient: change identifier.
    pub change_id: Option<String>,
    /// Transient: re-proposal iteration counter.
    pub iteration: Option<u32>,
    /// Transient: current task ID.
    pub current_task_id: Option<String>,
    /// Transient: per-spec implementation phase.
    pub impl_spec_phase: Option<HashMap<String, String>>,
    /// Transient: per-task revision counts.
    pub task_revisions: Option<HashMap<String, u32>>,
    /// Transient: per-phase revision counts.
    pub revision_counts: Option<HashMap<String, u32>>,
    /// Transient: last workflow action performed.
    pub last_action: Option<String>,
    /// Transient: session ID.
    pub session_id: Option<String>,
    /// Transient: validation errors to set.
    pub validation_errors: Option<Vec<String>>,
    /// CRRR review counter — Some(n) writes, None leaves untouched.
    pub review_count: Option<u8>,
    /// CRRR flagged sections — Some(vec![]) to clear, Some(non-empty) to set, None untouched.
    pub flagged_sections: Option<Vec<IssueSection>>,
    /// Loop-fill retry counter — Some(n) writes, Some(0) resets, None untouched.
    pub fill_retry_count: Option<u8>,
    /// Clear all transient SDD fields at once (used by merge).
    pub clear_transient: bool,
    /// Ship lifecycle status — Some(status) writes, None leaves untouched.
    pub ship_status: Option<ShipStatus>,
    /// Git commit hash written at step1_shipped — Some(hash) writes, None leaves untouched.
    pub ship_commit: Option<String>,
    /// Git commit hash written at loop_closed — Some(hash) writes, None leaves untouched.
    pub regen_verified_at: Option<String>,
}

/// Coarse phases of the CRRR loop, mirrored as Lifecycle-Stage git trailers.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuePhase {
    /// Issue created.
    Created,
    /// Requirements section being filled.
    FillRequirements,
    /// Scope section being filled.
    FillScope,
    /// Reference context section being filled.
    FillReferenceContext,
    /// Review complete.
    Reviewed,
    /// Revision applied.
    Revised,
    /// Issue merged.
    Merged,
}

/// Issue body sections targetable by --section and flaggable by reviewer.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSection {
    /// Problem statement section.
    Problem,
    /// Requirements section.
    Requirements,
    /// Scope section.
    Scope,
    /// Reference context section.
    ReferenceContext,
}

/// Issue lifecycle state.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    /// Open and actionable.
    Open,
    /// Closed (done, wontfix, duplicate, etc).
    Closed,
    /// Local-only draft, not yet pushed to a tracker.
    Draft,
}

/// Issue kind. Must match GitHub type:* labels 1:1.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    /// Large multi-issue initiative (may omit crate: label).
    Epic,
    /// A defect in existing behavior.
    Bug,
    /// New capability or improvement.
    Enhancement,
    /// Code restructuring with no behavior change.
    Refactor,
    /// Test coverage work.
    Test,
}

/// Tracks the ship lifecycle of a merged issue.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShipStatus {
    /// No td_merged yet (default).
    #[default]
    NotStarted,
    /// Validate advanced phase to td_merged and recorded the merge commit.
    Step1Shipped,
    /// Validate verified that gen-code output is byte-equivalent to current source.
    LoopClosed,
    /// Issue was closed without merging.
    Rejected,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types_phase_namespaces_source.md#source
// CODEGEN-BEGIN

/// Tech-design lifecycle phases, stored as raw strings in `Issue.phase`.
///
/// Phase 1 of the aw binary namespace split renames `td_gen_coded` to
/// `cb_genned`. Readers MUST accept both for one release; writers MUST
/// always emit the canonical form. Use [`canonical_td_phase`] when writing
/// and [`normalize_td_phase`] when comparing arbitrary input.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
pub mod td_phase {
    /// Canonical phase strings (Phase 1+). Always use these for writes.
    ///
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
    pub const TD_INITED: &str = "td_inited";
    pub const TD_CREATED: &str = "td_created";
    pub const TD_REVIEWED: &str = "td_reviewed";
    pub const TD_REVISED: &str = "td_revised";
    /// Canonical post-gen-code phase (Phase 1+). Replaces legacy `td_gen_coded`.
    pub const CB_GENNED: &str = "cb_genned";
    /// Phase 3: all manual code markers filled via `aw cb fill`.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#schema
    pub const CB_FILLED: &str = "cb_filled";
    /// Phase 3 CRRR: filled code passed `aw cb review` (verdict committed).
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_REVIEWED: &str = "cb_reviewed";
    /// Phase 3 CRRR: flagged markers re-filled by `aw cb revise`.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_REVISED: &str = "cb_revised";
    /// Phase 3 CRRR: 2-review ceiling exceeded; awaiting human arbitration.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_ARBITRATED: &str = "cb_arbitrated";
    pub const TD_MERGED: &str = "td_merged";

    /// Legacy alias — readers MUST accept it; writers MUST NOT emit it.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
    pub const LEGACY_TD_GEN_CODED: &str = "td_gen_coded";

    /// Normalize a phase string to its canonical form. Maps the legacy
    /// `td_gen_coded` alias to `cb_genned`; passes everything else
    /// through unchanged.
    ///
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
    pub fn normalize(phase: &str) -> &str {
        match phase {
            LEGACY_TD_GEN_CODED => CB_GENNED,
            other => other,
        }
    }

    /// True if `phase` represents the post-gen-code phase, regardless of
    /// whether it's the canonical `cb_genned` or the legacy `td_gen_coded`.
    ///
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
    pub fn is_post_gen(phase: &str) -> bool {
        matches!(phase, CB_GENNED | LEGACY_TD_GEN_CODED)
    }

    /// True if `phase` is acceptable as a pre-merge phase. Includes the
    /// Phase 3 `cb_filled` / `cb_reviewed` / `cb_revised` phases in
    /// addition to `cb_genned` / `td_gen_coded`.
    ///
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub fn is_mergeable(phase: &str) -> bool {
        matches!(
            phase,
            CB_GENNED | LEGACY_TD_GEN_CODED | CB_FILLED | CB_REVIEWED | CB_REVISED
        )
    }
}

/// Lifecycle-Stage trailer values written by `aw td` / `aw cb` to
/// the worktree git log. Phase 1 renames `Td-GenCode` to `Cb-Gen`;
/// readers MUST accept both for one release, writers MUST always emit
/// the canonical form.
///
/// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
pub mod lifecycle_trailer {
    pub const TD_INIT: &str = "Td-Init";
    pub const TD_CREATE: &str = "Td-Create";
    pub const TD_VALIDATE: &str = "Td-Validate";
    pub const TD_REVIEW: &str = "Td-Review";
    pub const TD_REVISE: &str = "Td-Revise";
    /// Canonical post-gen-code trailer (Phase 1+). Replaces `Td-GenCode`.
    pub const CB_GEN: &str = "Cb-Gen";
    /// Legacy trailer — readers MUST accept it; writers MUST NOT emit it.
    pub const LEGACY_TD_GEN_CODE: &str = "Td-GenCode";
    pub const TD_MERGE: &str = "Td-Merge";
    /// Phase 2: TD spec adopted from disk; phase bypassed to td_reviewed.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs.md#schema
    pub const TD_CLAIM: &str = "Td-Claim";
    /// Phase 2: existing code adopted; TD spec generated by fillback.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-recovery-verbs.md#schema
    pub const CB_CLAIM: &str = "Cb-Claim";
    /// Phase 3: all manual code markers filled via `aw cb fill`.
    /// Committed after the `cb check` gate passes.
    /// Note: spec calls for a `LifecycleTrailer` enum with `CbFill` variant,
    /// but the codebase uses module-level consts (Phase 1+ pattern); we
    /// follow the existing pattern.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-fill-workflow.md#schema
    pub const CB_FILL: &str = "Cb-Fill";
    /// Phase 3 CRRR: written by `aw cb review --apply` on verdict commit.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_REVIEW: &str = "Cb-Review";
    /// Phase 3 CRRR: written by `aw cb revise --apply` on revision commit.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_REVISE: &str = "Cb-Revise";
    /// Phase 3 CRRR: written by `aw cb arbitrate` after the 2-review ceiling.
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-cb-review-revise-crrr.md#schema
    pub const CB_ARBITRATE: &str = "Cb-Arbitrate";

    /// Normalize a trailer to its canonical form. Maps the legacy
    /// `Td-GenCode` to `Cb-Gen`; everything else passes through.
    ///
    /// @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#schema
    pub fn normalize(trailer: &str) -> &str {
        match trailer {
            LEGACY_TD_GEN_CODE => CB_GEN,
            other => other,
        }
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
// CODEGEN-BEGIN
/// Coarse phases of the CRRR loop, mirrored as `Lifecycle-Stage` git trailers.
/// Stored as the snake_case string in `Issue.phase` (shared field with the
/// change workflow, which uses `change_*` values once the issue moves to
/// `state: open`).
// REQ: issue-crrr-state-machine#R2
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
impl IssuePhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssuePhase::Created => "created",
            IssuePhase::FillRequirements => "fill_requirements",
            IssuePhase::FillScope => "fill_scope",
            IssuePhase::FillReferenceContext => "fill_reference_context",
            IssuePhase::Reviewed => "reviewed",
            IssuePhase::Revised => "revised",
            IssuePhase::Merged => "merged",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "created" => Some(IssuePhase::Created),
            "fill_requirements" => Some(IssuePhase::FillRequirements),
            "fill_scope" => Some(IssuePhase::FillScope),
            "fill_reference_context" => Some(IssuePhase::FillReferenceContext),
            "reviewed" => Some(IssuePhase::Reviewed),
            "revised" => Some(IssuePhase::Revised),
            "merged" => Some(IssuePhase::Merged),
            _ => None,
        }
    }
}

/// Issue body sections targetable by `--section` and flaggable by reviewer.
// REQ: issue-crrr-state-machine#R6, R9
/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
impl IssueSection {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueSection::Problem => "problem",
            IssueSection::Requirements => "requirements",
            IssueSection::Scope => "scope",
            IssueSection::ReferenceContext => "reference_context",
        }
    }

    /// Markdown H2 heading line that marks this section in an issue body.
    pub fn heading(&self) -> &'static str {
        match self {
            IssueSection::Problem => "## Problem",
            IssueSection::Requirements => "## Requirements",
            IssueSection::Scope => "## Scope",
            IssueSection::ReferenceContext => "## Reference Context",
        }
    }

    /// Title-cased name as it appears inside `[Section]` review tags.
    pub fn tag_name(&self) -> &'static str {
        match self {
            IssueSection::Problem => "Problem",
            IssueSection::Requirements => "Requirements",
            IssueSection::Scope => "Scope",
            IssueSection::ReferenceContext => "Reference Context",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "problem" => Some(IssueSection::Problem),
            "requirements" => Some(IssueSection::Requirements),
            "scope" => Some(IssueSection::Scope),
            "reference_context" | "reference context" | "ref_context" | "ref context" => {
                Some(IssueSection::ReferenceContext)
            }
            _ => None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
impl IssueState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueState::Open => "open",
            IssueState::Closed => "closed",
            IssueState::Draft => "draft",
        }
    }

    /// Parse from common tracker strings (GitHub returns "OPEN"/"CLOSED").
    pub fn parse_loose(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "open" => Some(IssueState::Open),
            "closed" => Some(IssueState::Closed),
            "draft" => Some(IssueState::Draft),
            _ => None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
impl IssueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueType::Epic => "epic",
            IssueType::Bug => "bug",
            IssueType::Enhancement => "enhancement",
            IssueType::Refactor => "refactor",
            IssueType::Test => "test",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "epic" => Some(IssueType::Epic),
            "change" => Some(IssueType::Enhancement),
            "bug" => Some(IssueType::Bug),
            "enhancement" | "feature" => Some(IssueType::Enhancement),
            "refactor" => Some(IssueType::Refactor),
            "test" => Some(IssueType::Test),
            _ => None,
        }
    }

    pub fn workflow_role(&self) -> &'static str {
        match self {
            IssueType::Epic => "epic",
            IssueType::Bug | IssueType::Enhancement | IssueType::Refactor | IssueType::Test => {
                "change"
            }
        }
    }

    /// Extract the issue type from a list of labels by finding the
    /// `type:*` label. Defaults to `Enhancement` if none found.
    pub fn from_labels(labels: &[String]) -> Self {
        for label in labels {
            if let Some(kind) = label.strip_prefix("type:") {
                if let Some(t) = Self::parse(kind) {
                    return t;
                }
            }
        }
        IssueType::Enhancement
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_runtime_helpers_source.md#source
impl IssueFilter {
    /// Apply the filter to an issue in-memory. Backends that support
    /// server-side filtering should translate the filter fields into their
    /// native query language instead of using this.
    pub fn matches(&self, issue: &Issue) -> bool {
        if let Some(state) = self.state {
            if issue.state != state {
                return false;
            }
        }
        if let Some(t) = self.issue_type {
            if issue.issue_type != t {
                return false;
            }
        }
        if let Some(label) = &self.label {
            if !issue.labels.iter().any(|l| l == label) {
                return false;
            }
        }
        if let Some(author) = &self.author {
            if issue.author.as_deref() != Some(author.as_str()) {
                return false;
            }
        }
        true
    }
}
// CODEGEN-END
/// Partial update descriptor for `IssueBackend::update`.
///
/// Only non-`None` / non-empty fields are applied. This avoids requiring
/// callers to fetch-then-modify-then-write the full `Issue` struct.

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types_patch_apply_source.md#source
// CODEGEN-BEGIN
// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R2
impl IssuePatch {
    /// Apply the patch to an issue, mutating it in place.
    pub fn apply(&self, issue: &mut Issue) {
        let sync_managed_labels = self.touches_managed_labels();
        if let Some(t) = &self.title {
            issue.title = t.clone();
        }
        if let Some(s) = &self.state {
            issue.state = *s;
        }
        for l in &self.add_labels {
            if !issue.labels.contains(l) {
                issue.labels.push(l.clone());
            }
        }
        issue.labels.retain(|l| !self.remove_labels.contains(l));
        if let Some(b) = &self.body {
            issue.body = b.clone();
        }
        if self.clear_phase {
            issue.phase = None;
        }
        if let Some(p) = &self.phase {
            issue.phase = Some(p.clone());
        }
        if let Some(br) = &self.branch {
            issue.branch = Some(br.clone());
        }
        if let Some(tb) = &self.target_branch {
            issue.target_branch = Some(tb.clone());
        }
        if let Some(gw) = &self.git_workflow {
            issue.git_workflow = Some(gw.clone());
        }
        // Transient SDD fields
        if self.clear_transient {
            issue.iteration = None;
            issue.current_task_id = None;
            issue.impl_spec_phase = None;
            issue.task_revisions = None;
            issue.revision_counts = None;
            issue.last_action = None;
            issue.session_id = None;
            issue.validation_errors = vec![];
            issue.review_count = None;
            issue.flagged_sections = None;
            issue.fill_retry_count = None;
        } else {
            if let Some(v) = &self.change_id {
                issue.change_id = Some(v.clone());
            }
            if let Some(v) = self.iteration {
                issue.iteration = Some(v);
            }
            if let Some(v) = &self.current_task_id {
                issue.current_task_id = Some(v.clone());
            }
            if let Some(v) = &self.impl_spec_phase {
                issue.impl_spec_phase = Some(v.clone());
            }
            if let Some(v) = &self.task_revisions {
                issue.task_revisions = Some(v.clone());
            }
            if let Some(v) = &self.revision_counts {
                issue.revision_counts = Some(v.clone());
            }
            if let Some(v) = &self.last_action {
                issue.last_action = Some(v.clone());
            }
            if let Some(v) = &self.session_id {
                issue.session_id = Some(v.clone());
            }
            if let Some(v) = &self.validation_errors {
                issue.validation_errors = v.clone();
            }
            if let Some(v) = self.review_count {
                issue.review_count = Some(v);
            }
            if let Some(v) = &self.flagged_sections {
                issue.flagged_sections = if v.is_empty() { None } else { Some(v.clone()) };
            }
            if let Some(v) = self.fill_retry_count {
                issue.fill_retry_count = if v == 0 { None } else { Some(v) };
            }
        }
        // Ship status fields — applied unconditionally (not gated on clear_transient).
        // @spec projects/agentic-workflow/tech-design/surface/specs/score-td-validate-lifecycle-extension.md#schema
        if let Some(v) = self.ship_status {
            issue.ship_status = Some(v);
        }
        if let Some(ref v) = self.ship_commit {
            issue.ship_commit = Some(v.clone());
        }
        if let Some(ref v) = self.regen_verified_at {
            issue.regen_verified_at = Some(v.clone());
        }
        if sync_managed_labels {
            issue.labels = crate::issues::labels::encode_labels(issue);
        }
    }

    fn touches_managed_labels(&self) -> bool {
        self.phase.is_some()
            || self.clear_phase
            || self.clear_transient
            || self.validation_errors.is_some()
            || self.review_count.is_some()
            || self.flagged_sections.is_some()
            || self.fill_retry_count.is_some()
            || self.ship_status.is_some()
            || self.ship_commit.is_some()
            || self.regen_verified_at.is_some()
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/issues/types_error_slug_tests_source.md#source
// CODEGEN-BEGIN
/// Structured error codes for agent-first JSON error output.
// @spec projects/agentic-workflow/tech-design/core/logic/issues-backend.md#R7
impl IssueErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueErrorCode::NotFound => "NOT_FOUND",
            IssueErrorCode::Validation => "VALIDATION",
            IssueErrorCode::Backend => "BACKEND",
        }
    }

    /// Exit code per spec: 0=ok, 1=not_found, 2=validation, 3=backend.
    pub fn exit_code(&self) -> i32 {
        match self {
            IssueErrorCode::NotFound => 1,
            IssueErrorCode::Validation => 2,
            IssueErrorCode::Backend => 3,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/issues/types_error_slug_tests_source.md#source
impl Issue {
    /// Generate a default filesystem-safe slug.
    ///
    /// Tracker-backed issues use the tracker-local numeric id as the canonical
    /// file key (`github_id` for GitHub, `gitlab_id` for GitLab). Local-only
    /// drafts keep the legacy type/title slug until they are mirrored to a
    /// tracker and backfilled with a native id.
    ///
    /// For local-only drafts, strips common title prefixes (e.g.
    /// `"score(2.5):"`, `"epic:"`) before slugifying the remaining words.
    pub fn default_slug(&self) -> String {
        if let Some(id) = self.github_id.or(self.gitlab_id) {
            return id.to_string();
        }

        let stripped = strip_title_prefix(&self.title);
        let slug = slugify(stripped);
        let combined = format!("{}-{}", self.issue_type.as_str(), slug);
        // Keep local-only legacy slugs short enough to remain usable as
        // temporary `slug:*` aliases if the draft is later mirrored.
        const MAX_SLUG_LEN: usize = 45;
        if combined.len() <= MAX_SLUG_LEN {
            return combined;
        }
        let mut cut = MAX_SLUG_LEN;
        while cut > 0 && !combined.is_char_boundary(cut) {
            cut -= 1;
        }
        combined[..cut].trim_end_matches('-').to_string()
    }
}

/// Strip common organizational prefixes from issue titles:
/// - `epic: ...`
/// - `score(2.5): ...`
/// - `notation(4.3): ...`
/// - `feat(sdd): ...` style conventional-commit prefixes
fn strip_title_prefix(title: &str) -> &str {
    let mut s = title.trim();

    // Strip leading `{word}:` or `{word}({ver}):` patterns, up to 3 levels.
    for _ in 0..3 {
        if let Some(colon_pos) = s.find(':') {
            let before = &s[..colon_pos];
            // Only strip if the prefix is short and looks like a tag
            // (letters, digits, parens, dots only, no spaces).
            let looks_like_tag = before.len() <= 30
                && before
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '(' | ')' | '.' | '-' | '_'));
            if looks_like_tag {
                s = s[colon_pos + 1..].trim_start();
                continue;
            }
        }
        break;
    }
    s
}

/// Slugify: lowercase, non-alphanumeric → `-`, collapse dashes, trim, truncate.
fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_dash = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    // Truncate to 50 chars at a word boundary.
    if trimmed.len() <= 50 {
        return trimmed;
    }
    let mut cut = 50;
    while cut > 0 && !trimmed.is_char_boundary(cut) {
        cut -= 1;
    }
    let short = &trimmed[..cut];
    short.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_prefix_cases() {
        assert_eq!(
            strip_title_prefix("epic: create projects/agentic-workflow"),
            "create projects/agentic-workflow"
        );
        assert_eq!(
            strip_title_prefix("score(2.5): rename user-facing cclab-sdd → score"),
            "rename user-facing cclab-sdd → score"
        );
        assert_eq!(
            strip_title_prefix("notation(4.3): Issue authoring notation"),
            "Issue authoring notation"
        );
        assert_eq!(strip_title_prefix("no prefix here"), "no prefix here");
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Fix the --broken-- thing!"), "fix-the-broken-thing");
        assert_eq!(slugify("Issue #1179: foo"), "issue-1179-foo");
    }

    #[test]
    fn default_slug_full() {
        let issue = Issue {
            issue_type: IssueType::Enhancement,
            title: "notation(4.3): Issue authoring notation + agent".into(),
            state: IssueState::Open,
            id: Some("test-uuid-1179".to_string()),
            github_id: Some(1179),
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec![],
            created_at: None,
            updated_at: None,
            slug: String::new(),
            body: String::new(),
            related: vec![],
            implements: vec![],
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        assert_eq!(issue.default_slug(), "1179");
    }

    #[test]
    fn default_slug_for_local_draft_uses_type_and_title() {
        let mut issue = empty_issue();
        issue.issue_type = IssueType::Enhancement;
        issue.title = "notation(4.3): Issue authoring notation + agent".into();
        issue.github_id = None;
        issue.gitlab_id = None;
        assert_eq!(
            issue.default_slug(),
            "enhancement-issue-authoring-notation-agent"
        );
    }

    #[test]
    fn issue_type_from_labels() {
        let labels = vec![
            "priority:p1".into(),
            "type:epic".into(),
            "project:agentic-workflow".into(),
        ];
        assert_eq!(IssueType::from_labels(&labels), IssueType::Epic);

        let labels = vec!["priority:p1".into()];
        assert_eq!(IssueType::from_labels(&labels), IssueType::Enhancement);
    }

    #[test]
    fn issue_type_change_alias_is_executable_work() {
        assert_eq!(IssueType::parse("change"), Some(IssueType::Enhancement));
        assert_eq!(IssueType::Epic.workflow_role(), "epic");
        assert_eq!(IssueType::Enhancement.workflow_role(), "change");
        assert_eq!(IssueType::Bug.workflow_role(), "change");
    }

    #[test]
    fn filter_matches() {
        let issue = Issue {
            issue_type: IssueType::Bug,
            title: "x".into(),
            state: IssueState::Open,
            id: Some("test-uuid-1".to_string()),
            github_id: Some(1),
            gitlab_id: None,
            url: None,
            author: Some("alice".into()),
            labels: vec!["priority:p0".into(), "project:agentic-workflow".into()],
            created_at: None,
            updated_at: None,
            slug: "bug-x".into(),
            body: String::new(),
            related: vec![],
            implements: vec![],
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        };
        let mut f = IssueFilter::default();
        assert!(f.matches(&issue));

        f.state = Some(IssueState::Closed);
        assert!(!f.matches(&issue));

        f.state = Some(IssueState::Open);
        f.label = Some("project:agentic-workflow".into());
        assert!(f.matches(&issue));

        f.label = Some("project:other".into());
        assert!(!f.matches(&issue));
    }

    fn empty_issue() -> Issue {
        Issue {
            issue_type: IssueType::Enhancement,
            title: "x".into(),
            state: IssueState::Open,
            id: None,
            github_id: None,
            gitlab_id: None,
            url: None,
            author: None,
            labels: vec![],
            created_at: None,
            updated_at: None,
            slug: "x".into(),
            body: String::new(),
            related: vec![],
            implements: vec![],
            phase: None,
            branch: None,
            target_branch: None,
            git_workflow: None,
            change_id: None,
            iteration: None,
            current_task_id: None,
            impl_spec_phase: None,
            task_revisions: None,
            revision_counts: None,
            last_action: None,
            session_id: None,
            validation_errors: vec![],
            review_count: None,
            flagged_sections: None,
            fill_retry_count: None,
            ship_status: None,
            ship_commit: None,
            regen_verified_at: None,
        }
    }

    #[test]
    fn patch_apply_writes_review_count_when_not_clearing() {
        let mut issue = empty_issue();
        let patch = IssuePatch {
            review_count: Some(2),
            ..Default::default()
        };
        patch.apply(&mut issue);
        assert_eq!(issue.review_count, Some(2));
    }

    #[test]
    fn patch_apply_writes_flagged_sections_when_not_clearing() {
        let mut issue = empty_issue();
        let patch = IssuePatch {
            flagged_sections: Some(vec![IssueSection::Problem, IssueSection::Requirements]),
            ..Default::default()
        };
        patch.apply(&mut issue);
        assert_eq!(
            issue.flagged_sections,
            Some(vec![IssueSection::Problem, IssueSection::Requirements])
        );
    }

    #[test]
    fn patch_apply_clears_flagged_sections_when_given_empty_vec() {
        let mut issue = empty_issue();
        issue.flagged_sections = Some(vec![IssueSection::Scope]);
        let patch = IssuePatch {
            flagged_sections: Some(vec![]),
            ..Default::default()
        };
        patch.apply(&mut issue);
        assert_eq!(issue.flagged_sections, None);
    }

    #[test]
    fn patch_apply_clear_transient_resets_review_count() {
        let mut issue = empty_issue();
        issue.review_count = Some(2);
        issue.flagged_sections = Some(vec![IssueSection::Problem]);
        let patch = IssuePatch {
            clear_transient: true,
            ..Default::default()
        };
        patch.apply(&mut issue);
        assert_eq!(issue.review_count, None);
        assert_eq!(issue.flagged_sections, None);
    }

    #[test]
    fn patch_apply_clear_phase_syncs_managed_labels() {
        let mut issue = empty_issue();
        issue.phase = Some("td_inited".to_string());
        issue.labels = vec![
            "type:bug".to_string(),
            "phase:td_inited".to_string(),
            "review:1".to_string(),
            "score:locked".to_string(),
            "score:lock:td".to_string(),
        ];
        issue.review_count = Some(1);

        let patch = IssuePatch {
            clear_phase: true,
            clear_transient: true,
            ship_status: Some(ShipStatus::Rejected),
            ..Default::default()
        };
        patch.apply(&mut issue);

        assert_eq!(issue.phase, None);
        assert_eq!(issue.review_count, None);
        assert_eq!(
            issue.labels,
            vec!["type:bug".to_string(), "ship:rejected".to_string()]
        );
    }
}
// CODEGEN-END
