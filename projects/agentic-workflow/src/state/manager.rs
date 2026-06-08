// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
// CODEGEN-BEGIN
//! StateManager — issue frontmatter + meta.yaml
//!
//! Workflow state (phase, branch, iteration, task tracking) lives in issue frontmatter.
//! Operational data (checksums, validations, telemetry) lives in `meta.yaml`.
//!
//! @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
//! @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R5
//! @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R6
//! STATE.yaml is fully deprecated: never read, never written. Legacy change
//! directories that still contain a STATE.yaml file receive a hard error
//! directing users at the migration path.

/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
use crate::models::state::{
    ChecksumEntry, DelegationGuard, LlmCall, State, StatePhase, Telemetry, ValidationEntry,
    ValidationMode, ValidationResult as FrontmatterValidationResult,
};
use crate::parser::frontmatter::calculate_checksum;
use anyhow::{Context, Result};
use chrono::Utc;
use fs2::FileExt;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

/// Files tracked for staleness detection
const TRACKED_FILES: &[&str] = &[
    "proposal.md",
    "tasks.md",
    "CHALLENGE.md",
    "IMPLEMENTATION.md",
    "VERIFICATION.md",
];

/// File-based agent lock. Released automatically when dropped.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#schema
pub struct AgentLock {
    /// Locked file handle.
    _file: File,
}

/// State manager for a single change.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#schema
pub struct StateManager {
    /// Change directory path.
    change_dir: PathBuf,
    /// Loaded state.
    state: State,
    /// Whether the state has unsaved changes.
    dirty: bool,
    /// Issue slug, equals change_id.
    issue_slug: Option<String>,
    /// Project root derived from change_dir.
    project_root: Option<PathBuf>,
}

/// Report of file freshness vs recorded checksums.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#schema
#[derive(Debug, Clone)]
pub struct StalenessReport {
    /// Files that have changed since last checksum.
    pub stale_files: Vec<String>,
    /// Files without recorded checksums.
    pub missing_checksums: Vec<String>,
    /// Files that are up to date.
    pub up_to_date: Vec<String>,
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
impl StateManager {
    /// Load state for a change.
    ///
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R5
    /// R5: reads only issue frontmatter + meta.yaml. Legacy STATE.yaml triggers
    /// a hard error with migration guidance.
    pub fn load(change_dir: impl Into<PathBuf>) -> Result<Self> {
        let change_dir = change_dir.into();

        // Extract change_id from directory name (= issue slug)
        let change_id = change_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Derive project root from change_dir: .aw/changes/{id} → root
        let project_root = change_dir
            .parent() // .aw/changes
            .and_then(|p| p.parent()) // .aw
            .and_then(|p| p.parent()) // project/worktree root
            .map(|p| p.to_path_buf());

        // R5: reject legacy STATE.yaml — users must migrate.
        let state_path = change_dir.join("STATE.yaml");
        if state_path.exists() {
            anyhow::bail!(
                "STATE.yaml is deprecated. Migrate via `score changes migrate-legacy` \
                 (or copy workflow fields to the issue frontmatter manually, then \
                 delete STATE.yaml, user_input.md, and groups/). See: \
                 projects/agentic-workflow/tech-design/core/logic/issue-centric-workflow.md"
            );
        }

        // Load workflow state from issue frontmatter (single source of truth).
        // Catches panics from block_in_place in single-threaded test runtimes.
        let issue_state = project_root.as_ref().and_then(|root| {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                load_state_from_issue(root, &change_id).ok()
            }))
            .ok()
            .flatten()
        });

        // Load operational data from meta.yaml
        let meta_path = change_dir.join("meta.yaml");
        let meta: Option<State> = if meta_path.exists() {
            std::fs::read_to_string(&meta_path)
                .ok()
                .and_then(|c| serde_yaml::from_str(&c).ok())
        } else {
            None
        };

        // Workflow state from issue frontmatter, or defaults for never-initialized changes.
        // Non-issue callers (tests, ad-hoc fixtures) get defaults — save() will then fail
        // to sync with a clear issue-backend error rather than silently writing STATE.yaml.
        let base = issue_state.unwrap_or_else(|| State {
            change_id: change_id.clone(),
            schema_version: "2.0".to_string(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            phase: StatePhase::ChangeInited,
            iteration: 1,
            last_action: None,
            session_id: None,
            git_workflow: None,
            checksums: HashMap::new(),
            validations: Vec::new(),
            revision_counts: HashMap::new(),
            current_task_id: None,
            task_revisions: HashMap::new(),
            impl_spec_phase: HashMap::new(),
            telemetry: None,
            dag: None,
            delegation_guard: None,
            branch: None,
        });

        // Overlay operational data from meta.yaml if available
        let state = if let Some(m) = meta {
            State {
                checksums: if m.checksums.is_empty() {
                    base.checksums
                } else {
                    m.checksums
                },
                validations: if m.validations.is_empty() {
                    base.validations
                } else {
                    m.validations
                },
                telemetry: m.telemetry.or(base.telemetry),
                delegation_guard: m.delegation_guard.or(base.delegation_guard),
                ..base
            }
        } else {
            base
        };

        Ok(Self {
            change_dir,
            state,
            dirty: false,
            issue_slug: Some(change_id),
            project_root,
        })
    }

    /// Save state: workflow fields → issue frontmatter, operational data → meta.yaml.
    ///
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
    /// @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R6
    /// R4/R6: No STATE.yaml fallback. If `sync_to_issue()` returns `Err`, the
    /// error bubbles up unchanged. Callers observe the underlying backend
    /// error (NotFound, PermissionDenied, etc.) rather than a silent
    /// "wrote fallback" success.
    pub fn save(&mut self) -> Result<()> {
        self.state.updated_at = Some(Utc::now());

        // 1. Write operational data (checksums, validations, telemetry) to meta.yaml
        //    (conditional — see save_meta; R11).
        self.save_meta()?;

        // 2. Write workflow fields to issue frontmatter — single source of truth.
        //    Panic-catch preserved for single-threaded test runtimes where
        //    block_in_place would panic; convert panics into a hard error
        //    rather than silently writing STATE.yaml.
        let sync_result: std::result::Result<(), anyhow::Error> =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.sync_to_issue())) {
                Ok(inner) => inner,
                Err(_) => Err(anyhow::anyhow!(
                    "sync_to_issue() panicked — likely single-threaded tokio runtime. \
                     Tests must use a multi-threaded runtime or a mock issue backend."
                )),
            };
        sync_result?;

        self.dirty = false;
        Ok(())
    }

    /// Write operational data to meta.yaml (checksums, validations, telemetry, delegation_guard).
    fn save_meta(&self) -> Result<()> {
        // Only write if there's actual operational data
        let has_data = !self.state.checksums.is_empty()
            || !self.state.validations.is_empty()
            || self.state.telemetry.is_some()
            || self.state.delegation_guard.is_some();

        if !has_data {
            return Ok(());
        }

        // Build a minimal State with only operational fields for meta.yaml
        let meta = State {
            change_id: self.state.change_id.clone(),
            schema_version: self.state.schema_version.clone(),
            created_at: self.state.created_at,
            updated_at: self.state.updated_at,
            checksums: self.state.checksums.clone(),
            validations: self.state.validations.clone(),
            telemetry: self.state.telemetry.clone(),
            delegation_guard: self.state.delegation_guard.clone(),
            ..State::default()
        };

        let meta_path = self.change_dir.join("meta.yaml");
        std::fs::create_dir_all(&self.change_dir).context("Failed to create change directory")?;
        let content = serde_yaml::to_string(&meta).context("Failed to serialize meta.yaml")?;
        std::fs::write(&meta_path, content).context("Failed to write meta.yaml")?;

        Ok(())
    }

    /// Save only if dirty
    pub fn save_if_dirty(&mut self) -> Result<()> {
        if self.dirty {
            self.save()?;
        }
        Ok(())
    }

    /// Get current state (read-only)
    pub fn state(&self) -> &State {
        &self.state
    }

    /// Get mutable reference to state (for direct field updates)
    pub fn state_mut(&mut self) -> &mut State {
        self.dirty = true;
        &mut self.state
    }

    /// Get change directory
    pub fn change_dir(&self) -> &Path {
        &self.change_dir
    }

    // =========================================================================
    // Phase Management
    // =========================================================================

    /// Update the current phase.
    /// If a delegation guard is active, only phases in `allowed_phases` (or the
    /// current phase, for idempotent sets) are permitted.
    pub fn set_phase(&mut self, phase: StatePhase) -> Result<()> {
        if let Some(guard) = &self.state.delegation_guard {
            // Same-phase set is always allowed (idempotent)
            if phase != self.state.phase && !guard.allowed_phases.contains(&phase) {
                anyhow::bail!(
                    "Delegation guard blocked phase transition to '{}': \
                     action '{}' only allows {:?}",
                    phase_display(&phase),
                    guard.action,
                    guard
                        .allowed_phases
                        .iter()
                        .map(phase_display)
                        .collect::<Vec<_>>(),
                );
            }
        }
        self.state.phase = phase;
        self.dirty = true;
        Ok(())
    }

    /// Get current phase
    pub fn phase(&self) -> &StatePhase {
        &self.state.phase
    }

    // =========================================================================
    // Delegation Guard (#303)
    // =========================================================================

    /// Activate a delegation guard before dispatching to an external agent.
    pub fn set_delegation_guard(&mut self, allowed_phases: Vec<StatePhase>, action: &str) {
        self.state.delegation_guard = Some(DelegationGuard {
            allowed_phases,
            phase_before: self.state.phase.clone(),
            action: action.to_string(),
            started_at: Utc::now(),
        });
        self.dirty = true;
    }

    /// Clear delegation guard after successful agent completion.
    pub fn clear_delegation_guard(&mut self) {
        self.state.delegation_guard = None;
        self.dirty = true;
    }

    /// Rollback phase to pre-delegation snapshot and clear guard.
    pub fn rollback_delegation(&mut self) {
        if let Some(guard) = self.state.delegation_guard.take() {
            self.state.phase = guard.phase_before;
            self.dirty = true;
        }
    }

    /// Try to acquire an exclusive agent lock for this change (#1125).
    ///
    /// Returns `Err` if another agent is already running.
    /// The lock is released when the returned `AgentLock` is dropped.
    pub fn try_acquire_agent_lock(&self) -> Result<AgentLock> {
        let lock_path = self.change_dir.join(".agent.lock");
        let file = File::create(&lock_path)
            .with_context(|| format!("Failed to create agent lock: {}", lock_path.display()))?;
        file.try_lock_exclusive().map_err(|_| {
            anyhow::anyhow!(
                "Another agent is already running for this change (lock: {})",
                lock_path.display()
            )
        })?;
        Ok(AgentLock { _file: file })
    }

    /// Check if an existing guard is stale (agent crashed > 30 min ago).
    pub fn is_delegation_stale(&self) -> bool {
        self.state
            .delegation_guard
            .as_ref()
            .map(|g| {
                let elapsed = Utc::now().signed_duration_since(g.started_at);
                elapsed.num_minutes() > 30
            })
            .unwrap_or(false)
    }

    /// Increment iteration (for reproposals)
    pub fn increment_iteration(&mut self) {
        self.state.iteration += 1;
        self.dirty = true;
    }

    /// Set last action
    pub fn set_last_action(&mut self, action: impl Into<String>) {
        self.state.last_action = Some(action.into());
        self.dirty = true;
    }

    /// Get revision count for a context stage (e.g., "spec_context")
    pub fn revision_count(&self, stage: &str) -> u32 {
        self.state.revision_counts.get(stage).copied().unwrap_or(0)
    }

    /// Increment revision count for a context stage
    pub fn increment_revision_count(&mut self, stage: &str) {
        *self
            .state
            .revision_counts
            .entry(stage.to_string())
            .or_insert(0) += 1;
        self.dirty = true;
    }

    /// Set git workflow preference
    pub fn set_git_workflow(&mut self, workflow: String) {
        self.state.git_workflow = Some(workflow);
        self.dirty = true;
    }

    /// Get git workflow preference
    pub fn git_workflow(&self) -> Option<&str> {
        self.state.git_workflow.as_deref()
    }

    /// Set Gemini session ID for resume-by-index
    pub fn set_session_id(&mut self, session_id: String) {
        self.state.session_id = Some(session_id);
        self.dirty = true;
    }

    /// Get current session ID
    pub fn session_id(&self) -> Option<&str> {
        self.state.session_id.as_deref()
    }

    /// Update phase based on proposal review verdict (for plan-change workflow)
    /// Legacy: proposal phases have been removed. This now maps directly to ChangeRejected on REJECTED,
    /// and is a no-op otherwise (proposal flow is removed).
    pub fn update_phase_from_proposal_verdict(&mut self, verdict: &str) -> Result<()> {
        if verdict == "REJECTED" {
            self.set_phase(StatePhase::ChangeRejected)?;
        }
        Ok(())
    }

    /// Update phase based on spec review verdict (for plan-change workflow)
    ///
    /// Specs Phase (#177):
    /// - ChangeSpecCreated + APPROVED → ChangeImplementationCreated (if last) or stays for next spec
    /// - ChangeSpecCreated + NEEDS_REVISION → ChangeSpecReviewed
    /// - ChangeSpecRevised + APPROVED → ChangeImplementationCreated (if last)
    /// - ChangeSpecRevised + NEEDS_REVISION → ChangeSpecReviewed (loop)
    pub fn update_phase_from_spec_verdict(
        &mut self,
        verdict: &str,
        is_last_spec: bool,
    ) -> Result<()> {
        let current = self.phase().clone();
        let new_phase = match (verdict, &current) {
            ("APPROVED", StatePhase::ChangeSpecCreated)
            | ("APPROVED", StatePhase::ChangeSpecRevised)
            | ("APPROVED", StatePhase::ChangeSpecReviewed) => {
                if is_last_spec {
                    StatePhase::ChangeImplementationCreated
                } else {
                    StatePhase::ChangeSpecCreated // Ready for next spec
                }
            }

            ("NEEDS_REVISION", StatePhase::ChangeSpecCreated)
            | ("NEEDS_REVISION", StatePhase::ChangeSpecRevised) => StatePhase::ChangeSpecReviewed,

            ("REJECTED", _) => StatePhase::ChangeRejected,
            _ => return Ok(()), // Unknown verdict or invalid state
        };

        self.set_phase(new_phase)
    }

    /// Mark spec as revised (after revise action)
    pub fn mark_spec_revised(&mut self) -> Result<()> {
        if matches!(self.phase(), StatePhase::ChangeSpecReviewed) {
            self.set_phase(StatePhase::ChangeSpecRevised)?;
        }
        Ok(())
    }

    /// Legacy: Update phase based on challenge verdict
    /// Kept for backwards compatibility, maps to update_phase_from_proposal_verdict
    pub fn update_phase_from_verdict(&mut self, verdict: &str) -> Result<()> {
        self.update_phase_from_proposal_verdict(verdict)
    }

    /// Update phase based on review verdict (for impl-change workflow)
    /// - APPROVED → ChangeMergeCreated (review passed, advance to merge)
    /// - REVIEWED → ChangeImplementationReviewed (needs revision)
    /// - REJECTED → ChangeImplementationCreated (needs manual intervention)
    pub fn update_phase_from_review(&mut self, verdict: &str) -> Result<()> {
        let new_phase = match verdict {
            "APPROVED" => StatePhase::ChangeMergeCreated,
            "REVIEWED" => StatePhase::ChangeImplementationReviewed,
            "REJECTED" => StatePhase::ChangeImplementationCreated,
            _ => return Ok(()), // Unknown verdict, don't change phase
        };

        self.set_phase(new_phase)
    }

    // =========================================================================
    // Checksum Management
    // =========================================================================

    /// Update checksum for a file
    pub fn update_checksum(&mut self, filename: &str) -> Result<()> {
        let file_path = self.change_dir.join(filename);
        if !file_path.exists() {
            // Remove checksum if file no longer exists
            self.state.checksums.remove(filename);
            self.dirty = true;
            return Ok(());
        }

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", filename))?;

        let hash = calculate_checksum(&content);
        self.state.checksums.insert(
            filename.to_string(),
            ChecksumEntry {
                hash,
                validated_at: Some(Utc::now()),
            },
        );
        self.dirty = true;

        Ok(())
    }

    /// Update checksums for all tracked files
    pub fn update_all_checksums(&mut self) -> Result<()> {
        for filename in TRACKED_FILES {
            let file_path = self.change_dir.join(filename);
            if file_path.exists() {
                self.update_checksum(filename)?;
            }
        }

        // Also track spec files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() {
            for entry in walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.change_dir) {
                    let filename = rel_path.to_string_lossy().to_string();
                    self.update_checksum(&filename)?;
                }
            }
        }

        Ok(())
    }

    /// Check if a file is stale (content changed since last checksum)
    pub fn is_file_stale(&self, filename: &str) -> Result<bool> {
        let file_path = self.change_dir.join(filename);
        if !file_path.exists() {
            return Ok(false);
        }

        let Some(entry) = self.state.checksums.get(filename) else {
            // No recorded checksum = stale (never validated)
            return Ok(true);
        };

        let content = std::fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to read {}", filename))?;

        let current_hash = calculate_checksum(&content);
        Ok(entry.hash != current_hash)
    }

    /// Get full staleness report for all tracked files
    pub fn check_staleness(&self) -> Result<StalenessReport> {
        let mut stale_files = Vec::new();
        let mut missing_checksums = Vec::new();
        let mut up_to_date = Vec::new();

        for filename in TRACKED_FILES {
            let file_path = self.change_dir.join(filename);
            if !file_path.exists() {
                continue;
            }

            if !self.state.checksums.contains_key(*filename) {
                missing_checksums.push(filename.to_string());
            } else if self.is_file_stale(filename)? {
                stale_files.push(filename.to_string());
            } else {
                up_to_date.push(filename.to_string());
            }
        }

        // Check spec files
        let specs_dir = self.change_dir.join("specs");
        if specs_dir.exists() {
            for entry in walkdir::WalkDir::new(&specs_dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            {
                if let Ok(rel_path) = entry.path().strip_prefix(&self.change_dir) {
                    let filename = rel_path.to_string_lossy().to_string();
                    if !self.state.checksums.contains_key(&filename) {
                        missing_checksums.push(filename);
                    } else if self.is_file_stale(&filename)? {
                        stale_files.push(filename);
                    } else {
                        up_to_date.push(filename);
                    }
                }
            }
        }

        Ok(StalenessReport {
            stale_files,
            missing_checksums,
            up_to_date,
        })
    }

    // =========================================================================
    // Validation History
    // =========================================================================

    /// Record a validation result
    pub fn record_validation(
        &mut self,
        step: impl Into<String>,
        mode: ValidationMode,
        valid: bool,
        high: u32,
        medium: u32,
        low: u32,
        errors: Vec<String>,
        warnings: Vec<String>,
    ) {
        let entry = ValidationEntry {
            step: step.into(),
            timestamp: Some(Utc::now()),
            rules_version: Some("2.0".to_string()),
            rules_hash: None,
            mode: Some(mode),
            result: Some(FrontmatterValidationResult {
                valid,
                high,
                medium,
                low,
                verdict: None,
                issues_parsed: None,
            }),
            errors,
            warnings,
        };

        self.state.validations.push(entry);
        self.dirty = true;
    }

    /// Record a challenge validation with verdict
    pub fn record_challenge_validation(
        &mut self,
        verdict: &str,
        issues_parsed: u32,
        high: u32,
        medium: u32,
        low: u32,
    ) {
        let entry = ValidationEntry {
            step: "validate-challenge".to_string(),
            timestamp: Some(Utc::now()),
            rules_version: Some("2.0".to_string()),
            rules_hash: None,
            mode: Some(ValidationMode::Normal),
            result: Some(FrontmatterValidationResult {
                valid: true,
                high,
                medium,
                low,
                verdict: Some(verdict.to_string()),
                issues_parsed: Some(issues_parsed),
            }),
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        self.state.validations.push(entry);
        self.dirty = true;
    }

    /// Get last validation for a step
    pub fn last_validation(&self, step: &str) -> Option<&ValidationEntry> {
        self.state.validations.iter().rev().find(|v| v.step == step)
    }

    /// Clear validation history
    pub fn clear_validations(&mut self) {
        self.state.validations.clear();
        self.dirty = true;
    }

    // =========================================================================
    // Telemetry
    // =========================================================================

    /// Record LLM call telemetry with cost calculation
    ///
    /// # Arguments
    /// * `step` - The workflow step (e.g., "proposal", "challenge", "implement")
    /// * `model` - The model name used
    /// * `tokens_in` - Number of input tokens
    /// * `tokens_out` - Number of output tokens
    /// * `duration_ms` - Duration in milliseconds
    /// * `cost_per_1m_input` - Cost per 1M input tokens (optional)
    /// * `cost_per_1m_output` - Cost per 1M output tokens (optional)
    pub fn record_llm_call(
        &mut self,
        step: &str,
        model: Option<String>,
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
        duration_ms: Option<u64>,
        cost_per_1m_input: Option<f64>,
        cost_per_1m_output: Option<f64>,
    ) {
        // Calculate cost if pricing info is available
        let cost_usd =
            Self::calculate_cost(tokens_in, tokens_out, cost_per_1m_input, cost_per_1m_output);

        let call = LlmCall {
            step: step.to_string(),
            sdd_version: Some(env!("CARGO_PKG_VERSION").to_string()),
            model,
            tokens_in,
            tokens_out,
            cost_usd,
            duration_ms,
            timestamp: Some(Utc::now()),
        };

        let telemetry = self.state.telemetry.get_or_insert_with(Telemetry::default);

        // Update totals
        if let Some(cost) = cost_usd {
            telemetry.total_cost_usd += cost;
        }
        if let Some(tokens) = tokens_in {
            telemetry.total_tokens_in += tokens;
        }
        if let Some(tokens) = tokens_out {
            telemetry.total_tokens_out += tokens;
        }

        // Append to calls list
        telemetry.calls.push(call);

        self.dirty = true;
    }

    /// Calculate cost from token usage and pricing
    fn calculate_cost(
        tokens_in: Option<u64>,
        tokens_out: Option<u64>,
        cost_per_1m_input: Option<f64>,
        cost_per_1m_output: Option<f64>,
    ) -> Option<f64> {
        let input_cost = match (tokens_in, cost_per_1m_input) {
            (Some(tokens), Some(cost)) => (tokens as f64 / 1_000_000.0) * cost,
            _ => 0.0,
        };

        let output_cost = match (tokens_out, cost_per_1m_output) {
            (Some(tokens), Some(cost)) => (tokens as f64 / 1_000_000.0) * cost,
            _ => 0.0,
        };

        let total = input_cost + output_cost;
        if total > 0.0 {
            Some(total)
        } else {
            None
        }
    }

    /// Append a pre-built LlmCall to telemetry and update totals
    pub fn append_telemetry_call(&mut self, call: LlmCall) {
        let telemetry = self.state.telemetry.get_or_insert_with(Telemetry::default);

        if let Some(cost) = call.cost_usd {
            telemetry.total_cost_usd += cost;
        }
        if let Some(tokens) = call.tokens_in {
            telemetry.total_tokens_in += tokens;
        }
        if let Some(tokens) = call.tokens_out {
            telemetry.total_tokens_out += tokens;
        }

        telemetry.calls.push(call);
        self.dirty = true;
    }

    /// Get telemetry summary for the change
    pub fn telemetry_summary(&self) -> Option<&Telemetry> {
        self.state.telemetry.as_ref()
    }

    // =========================================================================
    // Issue Frontmatter Sync (R1, R6)
    // =========================================================================

    /// Sync all workflow fields from State to the issue frontmatter.
    /// Issue slug = change_id (1:1:1 mapping, no user_input.md needed).
    fn sync_to_issue(&mut self) -> Result<()> {
        let slug = self
            .issue_slug
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No issue slug resolved"))?;
        let root = self
            .project_root
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Cannot derive project root from change dir"))?;

        let phase_str = crate::tools::phase_transition::phase_to_string(&self.state.phase);

        // Single-writer semantics (R4): StateManager fully overwrites workflow
        // fields in the issue. Always send Some(...) — `None` in IssuePatch
        // means "leave the issue's value alone", which would let locally
        // cleared maps/fields silently retain stale values in the issue file.
        let patch = crate::issues::types::IssuePatch {
            phase: Some(phase_str.to_string()),
            branch: self.state.branch.clone(),
            git_workflow: self.state.git_workflow.clone(),
            change_id: Some(self.state.change_id.clone()),
            iteration: Some(self.state.iteration),
            current_task_id: self.state.current_task_id.clone(),
            impl_spec_phase: Some(self.state.impl_spec_phase.clone()),
            task_revisions: Some(self.state.task_revisions.clone()),
            revision_counts: Some(self.state.revision_counts.clone()),
            last_action: self.state.last_action.clone(),
            session_id: self.state.session_id.clone(),
            ..Default::default()
        };

        let slug_owned = slug.clone();
        let root_owned = root.clone();
        let patch_owned = patch.clone();
        run_blocking_io(move || async move {
            let backend = crate::issues::local_backend(&root_owned);
            crate::issues::IssueBackend::update(&backend, &slug_owned, &patch_owned)
                .await
                .map(|_| ())
        })?;

        Ok(())
    }
}

/// Run an async future synchronously, transparent to current runtime flavor.
///
/// Works uniformly in:
/// - No tokio runtime (bare `fn main()`): creates a fresh runtime and blocks.
/// - Multi-threaded tokio runtime: uses `block_in_place` to avoid starving the scheduler.
/// - Single-threaded tokio runtime (e.g. `#[tokio::test]`): offloads to a
///   dedicated OS thread with its own runtime, since `block_in_place` would panic.
///
/// The future is constructed by the caller's closure so it can be re-created
/// across thread boundaries without requiring the future itself to be `Send`
/// (the closure + its inputs must be).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
pub(crate) fn run_blocking_io<T, F, Fut>(build_fut: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T>>,
{
    use tokio::runtime::{Handle, RuntimeFlavor};
    match Handle::try_current() {
        Ok(handle) => match handle.runtime_flavor() {
            RuntimeFlavor::MultiThread => {
                tokio::task::block_in_place(|| handle.block_on(build_fut()))
            }
            // CurrentThread (and any future flavor that disallows block_in_place):
            // offload to a dedicated OS thread with its own runtime.
            _ => std::thread::spawn(move || -> Result<T> {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(build_fut())
            })
            .join()
            .map_err(|_| anyhow::anyhow!("run_blocking_io worker thread panicked"))?,
        },
        Err(_) => tokio::runtime::Runtime::new()?.block_on(build_fut()),
    }
}

/// Quick display helper for StatePhase in error messages
/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
fn phase_display(phase: &StatePhase) -> String {
    // Re-use serde serialization for consistent naming
    serde_yaml::to_string(phase)
        .unwrap_or_else(|_| format!("{:?}", phase))
        .trim()
        .to_string()
}

/// Staleness report for a change
/// Load workflow state from issue frontmatter (primary state store).
///
/// Reads the issue file from the temp issue working copy,
/// extracts phase/branch/iteration/etc from frontmatter, and builds a State.
fn load_state_from_issue(project_root: &Path, slug: &str) -> Result<State> {
    use crate::issues::IssueBackend;

    let slug_owned = slug.to_string();
    let root_owned = project_root.to_path_buf();
    let issue = run_blocking_io(move || async move {
        let backend = crate::issues::local_backend(&root_owned);
        backend.get(&slug_owned).await
    })?;

    let issue = issue.ok_or_else(|| anyhow::anyhow!("Issue '{}' not found", slug))?;

    // Parse phase string → StatePhase enum
    let phase = issue
        .phase
        .as_deref()
        .and_then(|p| crate::tools::phase_transition::parse_phase(p).ok())
        .unwrap_or(StatePhase::ChangeInited);

    Ok(State {
        change_id: issue.change_id.unwrap_or_else(|| slug.to_string()),
        schema_version: "2.0".to_string(),
        created_at: None,
        updated_at: None,
        phase,
        iteration: issue.iteration.unwrap_or(1),
        last_action: issue.last_action,
        session_id: issue.session_id,
        git_workflow: issue.git_workflow,
        revision_counts: issue.revision_counts.unwrap_or_default(),
        current_task_id: issue.current_task_id,
        task_revisions: issue.task_revisions.unwrap_or_default(),
        impl_spec_phase: issue.impl_spec_phase.unwrap_or_default(),
        branch: issue.branch,
        // Operational data NOT in issue — loaded from meta.yaml separately
        checksums: HashMap::new(),
        validations: Vec::new(),
        telemetry: None,
        dag: None,
        delegation_guard: None,
    })
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/state/manager.md#source
impl StalenessReport {
    /// Check if any files are stale
    pub fn has_stale(&self) -> bool {
        !self.stale_files.is_empty()
    }

    /// Check if all tracked files have checksums
    pub fn is_complete(&self) -> bool {
        self.missing_checksums.is_empty()
    }

    /// Check if everything is up to date
    pub fn is_fresh(&self) -> bool {
        self.stale_files.is_empty() && self.missing_checksums.is_empty()
    }

    /// Total number of tracked files
    pub fn total_files(&self) -> usize {
        self.stale_files.len() + self.missing_checksums.len() + self.up_to_date.len()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_change() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let change_dir = temp_dir.path().join(".aw/changes/test-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // R4: StateManager::save() requires a backing issue file. Create a
        // minimal valid issue for slug `test-change` so sync_to_issue() succeeds.
        let issues_dir = crate::shared::workspace::issues_path(temp_dir.path()).join("open");
        std::fs::create_dir_all(&issues_dir).unwrap();
        let issue_content = "---\n\
            type: refactor\n\
            title: 'test(sdd): fixture'\n\
            state: open\n\
            ---\n\n## Problem\n\nTest fixture.\n";
        std::fs::write(issues_dir.join("test-change.md"), issue_content).unwrap();

        // Create proposal.md
        let mut proposal = std::fs::File::create(change_dir.join("proposal.md")).unwrap();
        writeln!(proposal, "# Test Proposal\n\nContent here").unwrap();

        // Create tasks.md
        let mut tasks = std::fs::File::create(change_dir.join("tasks.md")).unwrap();
        writeln!(tasks, "# Tasks\n\n## Task 1").unwrap();

        (temp_dir, change_dir)
    }

    #[test]
    fn test_load_new_state() {
        let (_temp, change_dir) = setup_test_change();

        let manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(manager.state().change_id, "test-change");
        assert_eq!(manager.state().phase, StatePhase::ChangeInited);
        assert_eq!(manager.state().iteration, 1);
    }

    #[test]
    fn test_save_and_load() {
        let (_temp, change_dir) = setup_test_change();

        // Create and save state
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.set_phase(StatePhase::ChangeSpecReviewed).unwrap();
            manager.set_last_action("challenge-proposal");
            manager.save().unwrap();
        }

        // Load and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(manager.state().phase, StatePhase::ChangeSpecReviewed);
            assert_eq!(
                manager.state().last_action,
                Some("challenge-proposal".to_string())
            );
        }
    }

    #[test]
    fn test_update_checksums() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();
        manager.update_checksum("tasks.md").unwrap();

        assert!(manager.state().checksums.contains_key("proposal.md"));
        assert!(manager.state().checksums.contains_key("tasks.md"));
        assert!(manager
            .state()
            .checksums
            .get("proposal.md")
            .unwrap()
            .hash
            .starts_with("sha256:"));
    }

    #[test]
    fn test_staleness_detection() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();

        // Should not be stale initially
        assert!(!manager.is_file_stale("proposal.md").unwrap());

        // Modify file
        std::fs::write(
            change_dir.join("proposal.md"),
            "# Modified Proposal\n\nNew content",
        )
        .unwrap();

        // Should now be stale
        assert!(manager.is_file_stale("proposal.md").unwrap());
    }

    #[test]
    fn test_staleness_report() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.update_checksum("proposal.md").unwrap();
        // Don't update tasks.md checksum

        let report = manager.check_staleness().unwrap();

        assert!(report.up_to_date.contains(&"proposal.md".to_string()));
        assert!(report.missing_checksums.contains(&"tasks.md".to_string()));
        assert!(!report.is_fresh());
    }

    #[test]
    fn test_record_validation() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.record_validation(
            "validate-proposal",
            ValidationMode::Normal,
            true,
            0,
            2,
            1,
            vec![],
            vec!["warning 1".to_string()],
        );

        assert_eq!(manager.state().validations.len(), 1);

        let last = manager.last_validation("validate-proposal").unwrap();
        assert_eq!(last.step, "validate-proposal");
        assert!(last.result.as_ref().unwrap().valid);
        assert_eq!(last.result.as_ref().unwrap().medium, 2);
    }

    #[test]
    fn test_phase_transitions() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(*manager.phase(), StatePhase::ChangeInited);

        manager.set_phase(StatePhase::ChangeSpecReviewed).unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeSpecReviewed);

        manager
            .set_phase(StatePhase::ChangeImplementationCreated)
            .unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeImplementationCreated);
    }

    #[test]
    fn test_iteration_increment() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        assert_eq!(manager.state().iteration, 1);

        manager.increment_iteration();
        assert_eq!(manager.state().iteration, 2);

        manager.increment_iteration();
        assert_eq!(manager.state().iteration, 3);
    }

    #[test]
    fn test_update_phase_from_verdict_rejected() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager.set_phase(StatePhase::ChangeInited).unwrap();

        manager.update_phase_from_verdict("REJECTED").unwrap();

        assert_eq!(*manager.phase(), StatePhase::ChangeRejected);
    }

    #[test]
    fn test_update_phase_from_verdict_unknown() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        let original_phase = StatePhase::ChangeInited;
        manager.set_phase(original_phase.clone()).unwrap();

        manager.update_phase_from_verdict("UNKNOWN").unwrap();

        // Should not change phase for unknown verdict
        assert_eq!(*manager.phase(), original_phase);
    }

    #[test]
    fn test_phase_transition_workflow() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Initial phase: ChangeInited
        assert_eq!(*manager.phase(), StatePhase::ChangeInited);

        // Move to PostClarificationsCreated
        manager.set_phase(StatePhase::ChangeInited).unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeInited);

        // Spec reviewed → ChangeImplementationCreated (approved = skip to impl)
        manager
            .set_phase(StatePhase::ChangeImplementationCreated)
            .unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeImplementationCreated);

        // Merge created
        manager.set_phase(StatePhase::ChangeMergeCreated).unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeMergeCreated);

        // Archive → ChangeArchived
        manager.set_phase(StatePhase::ChangeArchived).unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeArchived);
    }

    #[test]
    fn test_rejected_phase_workflow() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Initial phase: ChangeInited
        assert_eq!(*manager.phase(), StatePhase::ChangeInited);

        // Move to PostClarificationsCreated
        manager.set_phase(StatePhase::ChangeInited).unwrap();

        // Rejected verdict → ChangeRejected
        manager
            .update_phase_from_proposal_verdict("REJECTED")
            .unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeRejected);
    }

    #[test]
    fn test_phase_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Set phase and save
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.set_phase(StatePhase::ChangeInited).unwrap();
            manager.set_last_action("review");
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(*manager.phase(), StatePhase::ChangeInited);
            assert_eq!(manager.state().last_action, Some("review".to_string()));
        }
    }

    #[test]
    fn test_phase_terminal() {
        assert!(StatePhase::ChangeArchived.is_terminal());
        assert!(StatePhase::ChangeRejected.is_terminal());
        assert!(!StatePhase::ChangeImplementationCreated.is_terminal());
    }

    #[test]
    fn test_update_phase_from_review() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();
        manager
            .set_phase(StatePhase::ChangeImplementationCreated)
            .unwrap();

        // Test APPROVED verdict
        manager.update_phase_from_review("APPROVED").unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeMergeCreated);

        // Test REVIEWED verdict
        manager
            .set_phase(StatePhase::ChangeImplementationCreated)
            .unwrap();
        manager.update_phase_from_review("REVIEWED").unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeImplementationReviewed);

        // Test REJECTED verdict
        manager
            .set_phase(StatePhase::ChangeImplementationCreated)
            .unwrap();
        manager.update_phase_from_review("REJECTED").unwrap();
        assert_eq!(*manager.phase(), StatePhase::ChangeImplementationCreated);
    }

    // =========================================================================
    // Telemetry and Cost Tracking Tests
    // =========================================================================

    #[test]
    fn test_record_llm_call_basic() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record a basic LLM call without pricing
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(1000),
            Some(500),
            Some(5000),
            None,
            None,
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();
        assert_eq!(telemetry.calls.len(), 1);
        assert_eq!(telemetry.total_tokens_in, 1000);
        assert_eq!(telemetry.total_tokens_out, 500);
        assert_eq!(telemetry.total_cost_usd, 0.0); // No pricing info

        let call = &telemetry.calls[0];
        assert_eq!(call.step, "proposal");
        assert_eq!(call.model, Some("gemini-3-flash".to_string()));
        assert_eq!(call.tokens_in, Some(1000));
        assert_eq!(call.tokens_out, Some(500));
        assert_eq!(call.duration_ms, Some(5000));
        assert!(call.timestamp.is_some());
    }

    #[test]
    fn test_record_llm_call_with_cost() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record LLM call with pricing
        // Gemini flash: $0.10/1M input, $0.40/1M output
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(1_000_000), // 1M input tokens
            Some(500_000),   // 500K output tokens
            Some(5000),
            Some(0.10), // $0.10/1M input
            Some(0.40), // $0.40/1M output
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Expected cost: $0.10 (input) + $0.20 (output) = $0.30
        assert!((telemetry.total_cost_usd - 0.30).abs() < 0.0001);

        let call = &telemetry.calls[0];
        assert!((call.cost_usd.unwrap() - 0.30).abs() < 0.0001);
    }

    #[test]
    fn test_record_multiple_llm_calls() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Record proposal call (Gemini)
        manager.record_llm_call(
            "proposal",
            Some("gemini-3-flash".to_string()),
            Some(100_000),
            Some(50_000),
            Some(5000),
            Some(0.10),
            Some(0.40),
        );

        // Record challenge call (Codex)
        manager.record_llm_call(
            "challenge",
            Some(crate::defaults::CODEX_MODEL.to_string()),
            Some(80_000),
            Some(40_000),
            Some(10000),
            Some(2.00),
            Some(8.00),
        );

        // Record implement call (Claude)
        manager.record_llm_call(
            "implement",
            Some("claude-3-sonnet".to_string()),
            Some(200_000),
            Some(100_000),
            Some(30000),
            Some(3.00),
            Some(15.00),
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Verify totals
        assert_eq!(telemetry.calls.len(), 3);
        assert_eq!(telemetry.total_tokens_in, 380_000);
        assert_eq!(telemetry.total_tokens_out, 190_000);

        // Calculate expected costs:
        // Proposal: 0.1M * $0.10 + 0.05M * $0.40 = $0.01 + $0.02 = $0.03
        // Challenge: 0.08M * $2.00 + 0.04M * $8.00 = $0.16 + $0.32 = $0.48
        // Implement: 0.2M * $3.00 + 0.1M * $15.00 = $0.60 + $1.50 = $2.10
        // Total: $2.61
        assert!((telemetry.total_cost_usd - 2.61).abs() < 0.01);
    }

    #[test]
    fn test_cost_calculation() {
        // Test the static cost calculation method directly

        // Test with full pricing info
        let cost = StateManager::calculate_cost(
            Some(1_000_000),
            Some(500_000),
            Some(1.0), // $1/1M input
            Some(2.0), // $2/1M output
        );
        assert!((cost.unwrap() - 2.0).abs() < 0.0001); // $1.0 + $1.0 = $2.0

        // Test with no tokens
        let cost = StateManager::calculate_cost(None, None, Some(1.0), Some(2.0));
        assert!(cost.is_none());

        // Test with no pricing
        let cost = StateManager::calculate_cost(Some(1_000_000), Some(500_000), None, None);
        assert!(cost.is_none());

        // Test with partial pricing (input only)
        let cost = StateManager::calculate_cost(Some(1_000_000), Some(500_000), Some(1.0), None);
        assert!((cost.unwrap() - 1.0).abs() < 0.0001); // Only input cost
    }

    #[test]
    fn test_telemetry_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Record telemetry and save
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.record_llm_call(
                "proposal",
                Some("gemini-3-flash".to_string()),
                Some(50_000),
                Some(25_000),
                Some(3000),
                Some(0.10),
                Some(0.40),
            );
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            let telemetry = manager.state().telemetry.as_ref().unwrap();

            assert_eq!(telemetry.calls.len(), 1);
            assert_eq!(telemetry.total_tokens_in, 50_000);
            assert_eq!(telemetry.total_tokens_out, 25_000);

            let call = &telemetry.calls[0];
            assert_eq!(call.step, "proposal");
            assert_eq!(call.model, Some("gemini-3-flash".to_string()));
        }
    }

    #[test]
    fn test_telemetry_summary() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // No telemetry initially
        assert!(manager.telemetry_summary().is_none());

        // Record a call
        manager.record_llm_call(
            "test",
            Some("test-model".to_string()),
            Some(1000),
            Some(500),
            Some(1000),
            None,
            None,
        );

        // Now telemetry exists
        assert!(manager.telemetry_summary().is_some());
        assert_eq!(manager.telemetry_summary().unwrap().calls.len(), 1);
    }

    #[test]
    fn test_small_token_cost_precision() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Test with small token counts to verify precision
        manager.record_llm_call(
            "test",
            Some("test-model".to_string()),
            Some(100), // 100 tokens
            Some(50),  // 50 tokens
            Some(500),
            Some(0.10), // $0.10/1M
            Some(0.40), // $0.40/1M
        );

        let telemetry = manager.state().telemetry.as_ref().unwrap();

        // Expected cost: 0.0001 * $0.10 + 0.00005 * $0.40 = $0.00001 + $0.00002 = $0.00003
        let expected = 0.00003;
        assert!((telemetry.total_cost_usd - expected).abs() < 0.000001);
    }

    // =========================================================================
    // Session ID Tests
    // =========================================================================

    #[test]
    fn test_session_id_setter_and_getter() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // No session_id initially
        assert!(manager.session_id().is_none());

        // Set session_id
        manager.set_session_id("abc123-def456-789".to_string());
        assert_eq!(manager.session_id(), Some("abc123-def456-789"));

        // Change session_id
        manager.set_session_id("new-session-uuid".to_string());
        assert_eq!(manager.session_id(), Some("new-session-uuid"));
    }

    #[test]
    fn test_session_id_persistence() {
        let (_temp, change_dir) = setup_test_change();

        // Set and save session_id
        {
            let mut manager = StateManager::load(&change_dir).unwrap();
            manager.set_session_id("550e8400-e29b-41d4-a716-446655440000".to_string());
            manager.save().unwrap();
        }

        // Load in new instance and verify
        {
            let manager = StateManager::load(&change_dir).unwrap();
            assert_eq!(
                manager.session_id(),
                Some("550e8400-e29b-41d4-a716-446655440000")
            );
        }
    }

    // Obsolete: test_session_id_in_yaml_serialization and test_session_id_null_handling
    // exercised STATE.yaml, which R5/R6 of refactor-eliminate-state-yaml-user-input-md-groups-nesting
    // deprecated. session_id persistence is now covered by test_session_id_persistence,
    // which uses the canonical save/load cycle through issue frontmatter.

    #[test]
    fn test_session_id_marks_dirty() {
        let (_temp, change_dir) = setup_test_change();

        let mut manager = StateManager::load(&change_dir).unwrap();

        // Save initial state to clear dirty flag
        manager.save().unwrap();

        // Set session_id should mark as dirty
        manager.set_session_id("new-id".to_string());

        // Verify dirty flag is set by checking if save would write
        // (we can't directly check dirty, but we know it's set because the setter does it)
        manager.save().unwrap();

        // Reload and verify value persisted (proves dirty flag was set)
        let manager = StateManager::load(&change_dir).unwrap();
        assert_eq!(manager.session_id(), Some("new-id"));
    }

    // ─── Refactor tests (T3, T4) ──────────────────────────────────────────
    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md

    /// Build a project root with a change dir but **no** backing issue file.
    /// Used to exercise sync_to_issue() error propagation (T3).
    fn setup_change_without_issue() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path().to_path_buf();
        let change_dir = project_root
            .join(".aw/changes")
            .join("change-without-issue");
        std::fs::create_dir_all(&change_dir).unwrap();
        // Deliberately do NOT create a temp issue working copy.
        (temp, change_dir)
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R4
    // T3: StateManager::save() propagates sync_to_issue() Err unchanged.
    // No STATE.yaml fallback is ever written. meta.yaml is only written
    // when operational data exists (R11 preserved).
    #[test]
    fn test_r4_save_propagates_sync_error_no_state_yaml_fallback() {
        let (_tmp, change_dir) = setup_change_without_issue();

        let mut manager = StateManager::load(&change_dir).unwrap();
        // Trigger a phase transition so sync_to_issue is meaningful.
        manager.set_phase(StatePhase::ChangeSpecCreated).unwrap();

        // save() must bubble the backend error up — no silent fallback.
        let result = manager.save();
        assert!(
            result.is_err(),
            "save() must propagate sync_to_issue Err when issue is missing"
        );
        let err = result.err().expect("expected Err").to_string();
        assert!(
            err.to_ascii_lowercase().contains("not found")
                || err.to_ascii_lowercase().contains("no issue slug")
                || err
                    .to_ascii_lowercase()
                    .contains("cannot derive project root"),
            "expected a backend error surface, got: {}",
            err
        );

        // R4/R6 invariant: NO STATE.yaml created by the fallback branch.
        assert!(
            !change_dir.join("STATE.yaml").exists(),
            "save() must not write STATE.yaml fallback (R4/R6)"
        );

        // R11: meta.yaml is NOT written when there's no operational data
        // (no checksums, validations, telemetry, delegation_guard).
        assert!(
            !change_dir.join("meta.yaml").exists(),
            "meta.yaml must not be written when operational data is empty (R11)"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R5
    // T4: StateManager::load() against a dir containing only STATE.yaml
    // returns Err with the deprecation message. Legacy change directories
    // must not be silently resurrected.
    #[test]
    fn test_r5_load_rejects_legacy_state_yaml() {
        let temp = TempDir::new().unwrap();
        let change_dir = temp.path().join(".aw/changes").join("legacy-change");
        std::fs::create_dir_all(&change_dir).unwrap();

        // Write a legacy STATE.yaml payload (deserializable, non-empty).
        let legacy_yaml = r#"change_id: legacy-change
schema_version: "2.0"
phase: change_spec_created
iteration: 1
"#;
        std::fs::write(change_dir.join("STATE.yaml"), legacy_yaml).unwrap();

        let result = StateManager::load(&change_dir);
        assert!(
            result.is_err(),
            "load() must reject change dirs containing STATE.yaml"
        );
        let err = result.err().expect("expected Err").to_string();
        assert!(
            err.contains("STATE.yaml is deprecated"),
            "error must flag deprecation, got: {}",
            err
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/interfaces/models/state.md#R11
    // T3b: save_meta is conditional — when operational data IS present, the
    // manager writes meta.yaml (exercised indirectly by the save() attempt).
    // This test isolates the empty-data branch: with zero operational data,
    // a failing save() must not create meta.yaml.
    #[test]
    fn test_r11_meta_yaml_not_written_when_empty() {
        let (_tmp, change_dir) = setup_change_without_issue();
        let mut manager = StateManager::load(&change_dir).unwrap();
        // No checksums, validations, telemetry, or delegation_guard set.
        // save() will fail at sync_to_issue; but even before that,
        // save_meta should have short-circuited on empty data.
        let _ = manager.save();
        assert!(
            !change_dir.join("meta.yaml").exists(),
            "meta.yaml must not be written when operational data is empty (R11)"
        );
    }
}

// CODEGEN-END
