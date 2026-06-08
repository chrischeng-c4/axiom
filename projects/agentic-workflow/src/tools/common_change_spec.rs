//! Shared helpers for change-spec tools.
//!
//! Provides spec sub-state resolution, skeleton generation, section-level
//! operations, and TODO pruning used across create/review.

use crate::state::StateManager;
use crate::workflow::helpers;
use crate::Result;
use std::path::{Path, PathBuf};

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
// CODEGEN-BEGIN
// ─── Universal Skeleton ─────────────────────────────────────────────────────

// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R6
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R7
// @spec projects/agentic-workflow/tech-design/core/logic/spec-format-unification.md#R8
/// Universal skeleton template with ALL possible sections.
/// Sections are annotated with `<!-- type: xxx lang: yyy -->`.
/// Agent decides which to fill; prune removes unfilled sections.
///
/// Format decisions (D1-D9 from issue):
/// - 3 langs only: markdown, yaml, mermaid. JSON removed.
/// - requirements/unit-test: Mermaid Plus requirementDiagram (YAML frontmatter inside mermaid block)
/// - e2e-test: YAML journey plus machine-verifiable assertions
/// - scenarios: YAML GWT structured format {id, given, when, then, diagram_ref?}
/// - schema/rpc-api/config/component/design-token: yaml (not json)
/// - all diagram sections: Mermaid Plus (YAML frontmatter inside mermaid block)
/// - changes: optional satisfies: [R-id] field for requirement traceability
pub const UNIVERSAL_SKELETON: &str = r#"---
id: {spec_id}
main_spec_ref: ~
merge_strategy: new
---

# {title}

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Requirements
<!-- type: requirements lang: mermaid -->

<!-- TODO: Use Mermaid Plus requirementDiagram (SysML v1.6). Example:
```mermaid
---
id: requirements
---
requirementDiagram

requirement R1 {
  id: R1
  text: "Description of requirement 1"
  risk: low
  verifymethod: test
}

requirement R2 {
  id: R2
  text: "Description of requirement 2"
  risk: medium
  verifymethod: analysis
}
```
-->

## Scenarios
<!-- type: scenarios lang: yaml -->

<!-- TODO: Use YAML GWT structured format. Example:
```yaml
- id: S1
  given: Initial state description
  when: Action or event that triggers the scenario
  then: Expected outcome

- id: S2
  given: Another initial state
  when: Another action
  then: Another expected outcome
  diagram_ref: interaction-S2
```
-->

## Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

## State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

## Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO: Use Mermaid Plus sequenceDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: interaction
---
sequenceDiagram
    actor User
    User->>System: action
```
-->

## Logic
<!-- type: logic lang: mermaid -->
<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
```mermaid
---
id: logic
---
flowchart TD
    A([Start]) --> B{Decision}
```
-->

## Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: dependency
---
classDiagram
    class ComponentA
    class ComponentB
    ComponentA --> ComponentB
```
-->

## Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: db-model
---
erDiagram
    ENTITY {
        string id PK
    }
```
-->

## REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

## RPC API
<!-- type: rpc-api lang: yaml -->
<!-- TODO: OpenRPC 1.3 as YAML. Example:
```yaml
openrpc: "1.3.2"
info:
  title: Service Name
  version: "1.0.0"
methods: []
```
-->

## Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

## CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

## Schema
<!-- type: schema lang: yaml -->
<!-- TODO: JSON Schema as YAML. Example:
```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
type: object
properties:
  id:
    type: string
required: [id]
```
-->

## Config
<!-- type: config lang: yaml -->
<!-- TODO -->

## Unit Test
<!-- type: unit-test lang: mermaid -->

<!-- TODO: Use Mermaid Plus requirementDiagram with element nodes and verifies relationships.
```mermaid
---
id: unit-test
---
requirementDiagram

element T1 {
  type: "Test"
}

element T2 {
  type: "Test"
}

T1 - verifies -> R1
T2 - verifies -> R2
```
-->

## E2E Test
<!-- type: e2e-test lang: yaml -->

<!-- TODO: Use YAML to describe product journeys and machine-verifiable assertions. Example:
```yaml
e2e_tests:
  - name: cli_prints_help
    entrypoint: cli
    command: "./target/debug/aw --help"
    expect:
      exit_code: 0
      stdout_contains: ["Usage: aw"]
    cleanup:
      - "true"
```
-->

## Changes
<!-- type: changes lang: yaml -->

<!-- TODO -->

## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: yaml -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: yaml -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
"#;

// @spec projects/agentic-workflow/tech-design/core/logic/spec-structure.md#R2
/// All fillable section names (used for analyze step).
///
/// Ordered by `SectionType::fill_order()` — top-down human reasoning order.
/// Must match `SectionType::as_str()` values.
pub const ALL_SECTIONS: &[&str] = &[
    "overview",      // 0
    "requirements",  // 1
    "scenarios",     // 2
    "mindmap",       // 3
    "state-machine", // 4
    "interaction",   // 5
    "logic",         // 6
    "dependency",    // 7
    "db-model",      // 8
    "schema",        // 9
    "rest-api",      // 10
    "rpc-api",       // 11
    "async-api",     // 12
    "cli",           // 13
    "wireframe",     // 14
    "component",     // 15
    "design-token",  // 16
    "config",        // 17
    "unit-test",     // 18
    "e2e-test",      // 19
    "changes",       // 20
    "doc",           // 21
];

// ─── Spec Path Helpers ──────────────────────────────────────────────────────

/// Get the specs directory for a change, respecting group structure.
///
/// - If `group_id` is `Some(gid)`, returns `change_dir/groups/{gid}/specs/`
/// - If `group_id` is `None`, returns `change_dir/specs/` (backward compat)
///
/// This is the canonical path helper — all tools must use this function
/// instead of hardcoding paths.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
pub fn get_specs_dir(change_dir: &std::path::Path, group_id: Option<&str>) -> std::path::PathBuf {
    match group_id {
        Some(gid) => change_dir.join("groups").join(gid).join("specs"),
        None => change_dir.join("specs"),
    }
}

/// Get the spec file path for a given spec_id within a change.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
pub fn get_spec_path(
    change_dir: &std::path::Path,
    group_id: Option<&str>,
    spec_id: &str,
) -> std::path::PathBuf {
    get_specs_dir(change_dir, group_id).join(format!("{}.md", spec_id))
}

/// Find a spec file by ID, searching groups/*/specs/ first, then specs/.
///
/// Returns the path to the spec file if found, or falls back to the legacy
/// `change_dir/specs/{spec_id}.md` path (which may not exist) so callers
/// get a deterministic path for creation.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
pub fn find_spec_path(change_dir: &std::path::Path, spec_id: &str) -> std::path::PathBuf {
    // Search groups/*/specs/ first
    let groups_dir = change_dir.join("groups");
    if groups_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            sorted.sort_by_key(|e| e.file_name());
            for entry in sorted {
                if entry.path().is_dir() {
                    let candidate = entry.path().join("specs").join(format!("{}.md", spec_id));
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }
    }
    // Fallback: legacy specs/ path
    change_dir.join("specs").join(format!("{}.md", spec_id))
}

/// Get the effective specs directory for a change (groups-aware, for legacy compat).
///
/// Returns the first group's specs dir if `groups/` exists and has subdirs,
/// otherwise returns `change_dir/specs/`.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
pub fn get_primary_specs_dir(change_dir: &std::path::Path) -> std::path::PathBuf {
    let groups_dir = change_dir.join("groups");
    if groups_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&groups_dir) {
            let mut sorted: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            sorted.sort_by_key(|e| e.file_name());
            for entry in sorted {
                if entry.path().is_dir() {
                    let group_specs = entry.path().join("specs");
                    if group_specs.is_dir() {
                        return group_specs;
                    }
                }
            }
        }
    }
    change_dir.join("specs")
}

/// Resolve the group_id for a spec in a multi-group change.
///
/// Searches in this order:
/// 1. `groups/*/specs/{spec_id}.md` — spec already created in a group
/// 2. `groups/*/spec_plan.yaml` — spec assigned to group in spec plan
/// 3. `None` — spec belongs to root layout or single-group change
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#source
pub fn resolve_group_id_for_spec(change_dir: &std::path::Path, spec_id: &str) -> Option<String> {
    let groups_dir = change_dir.join("groups");
    if !groups_dir.is_dir() {
        return None;
    }

    let mut entries: Vec<_> = std::fs::read_dir(&groups_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    // Pass 1: check if spec already exists in a group's specs dir
    for entry in &entries {
        if entry.path().is_dir() {
            let spec_path = entry.path().join("specs").join(format!("{}.md", spec_id));
            if spec_path.exists() {
                return entry.file_name().to_str().map(String::from);
            }
        }
    }

    // Pass 2: check spec_plan.yaml for group membership
    for entry in &entries {
        if entry.path().is_dir() {
            let plan_path = entry.path().join("spec_plan.yaml");
            if let Ok(content) = std::fs::read_to_string(&plan_path) {
                if content.contains(&format!("spec_id: {}", spec_id)) {
                    return entry.file_name().to_str().map(String::from);
                }
            }
        }
    }

    None
}
// CODEGEN-END
// ─── Spec Sub-State ─────────────────────────────────────────────────────────

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_spec.md#schema
// CODEGEN-BEGIN
/// Per-spec sub-state within the change-spec lifecycle.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec.md#schema
#[derive(Debug)]
pub enum SpecSubState {
    /// No spec file — needs skeleton + create loop.
    Create {
        spec_id: String,
        depends: Vec<String>,
    },
    /// Spec exists with create_complete, no review — needs review.
    Review { spec_id: String },
    /// Reviewed with issues — re-fill flagged sections.
    Revise { spec_id: String },
    /// REJECTED after revision limit — mainthread must intervene.
    MainthreadMustFix { spec_id: String },
    /// All specs created + approved.
    AdvanceToImplementation,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
// CODEGEN-BEGIN
/// Resolve the next spec's sub-state for change-spec processing.
///
/// Wraps `helpers::analyze_specs()` + verdict logic from the old
/// `workflow/spec.rs` into a clean enum.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn resolve_next_spec(change_dir: &Path, change_id: &str) -> Result<SpecSubState> {
    let proposal_path = change_dir.join("proposal.md");
    // Use group-aware specs dir (checks groups/*/specs/ first, falls back to specs/)
    let specs_dir = get_primary_specs_dir(change_dir);
    let has_proposal = proposal_path.exists();
    let has_specs_dir = specs_dir.exists();

    let spec_count = if has_specs_dir {
        helpers::count_spec_files(&specs_dir)
    } else {
        0
    };

    let (missing_specs, pending_review_spec) = if has_proposal {
        helpers::analyze_specs(&proposal_path, &specs_dir)?
    } else {
        (vec![], None)
    };

    let last_review_verdict = helpers::get_last_review_verdict(change_dir);
    let default_spec_id = format!("{}-spec", change_id);

    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();

    use crate::models::state::StatePhase;

    match &phase {
        StatePhase::ChangeInited | StatePhase::ChangeSpecCreated => {
            // Check for pending review first (ChangeSpecCreated)
            if phase == StatePhase::ChangeSpecCreated {
                if let Some(spec_id) = &pending_review_spec {
                    // Check if spec has create_complete — if not, still in Create
                    let spec_path = find_spec_path(change_dir, spec_id);
                    if spec_path.exists() {
                        let content = std::fs::read_to_string(&spec_path).unwrap_or_default();
                        if !is_create_complete(&content) {
                            return Ok(SpecSubState::Create {
                                spec_id: spec_id.clone(),
                                depends: vec![],
                            });
                        }
                    }
                    return Ok(SpecSubState::Review {
                        spec_id: spec_id.clone(),
                    });
                }
            }
            // Then check missing specs
            if let Some(spec) = missing_specs.first() {
                Ok(SpecSubState::Create {
                    spec_id: spec.id.clone(),
                    depends: spec.depends.clone(),
                })
            } else if spec_count > 0 {
                // Check if any existing spec is incomplete
                if let Some(incomplete) = find_incomplete_spec(&specs_dir) {
                    return Ok(SpecSubState::Create {
                        spec_id: incomplete,
                        depends: vec![],
                    });
                }
                // REQ: change-spec.md#CRR1 — CRR is mandatory.
                // Complete specs without APPROVED verdict MUST route to Review.
                if let Some(unreviewed) = find_unreviewed_complete_spec(change_dir, &specs_dir) {
                    return Ok(SpecSubState::Review {
                        spec_id: unreviewed,
                    });
                }
                Ok(SpecSubState::AdvanceToImplementation)
            } else {
                Ok(SpecSubState::Create {
                    spec_id: default_spec_id,
                    depends: vec![],
                })
            }
        }
        StatePhase::ChangeSpecReviewed => {
            match last_review_verdict.as_deref() {
                Some("APPROVED") | Some("PASS") => next_or_advance(
                    &missing_specs,
                    spec_count,
                    &default_spec_id,
                    change_dir,
                    &specs_dir,
                ),
                Some("REVIEWED") | Some("NEEDS_REVISION") => {
                    if let Some(spec_id) = &pending_review_spec {
                        let key = format!("spec:{}", spec_id);
                        if sm.revision_count(&key) >= 1 {
                            // Auto-approve after revision limit
                            next_or_advance(
                                &missing_specs,
                                spec_count,
                                &default_spec_id,
                                change_dir,
                                &specs_dir,
                            )
                        } else {
                            Ok(SpecSubState::Revise {
                                spec_id: spec_id.clone(),
                            })
                        }
                    } else {
                        next_or_advance(
                            &missing_specs,
                            spec_count,
                            &default_spec_id,
                            change_dir,
                            &specs_dir,
                        )
                    }
                }
                Some("REJECTED") => {
                    if let Some(spec_id) = &pending_review_spec {
                        let key = format!("spec:{}", spec_id);
                        if sm.revision_count(&key) >= 1 {
                            return Ok(SpecSubState::MainthreadMustFix {
                                spec_id: spec_id.clone(),
                            });
                        }
                        Ok(SpecSubState::Revise {
                            spec_id: spec_id.clone(),
                        })
                    } else {
                        next_or_advance(
                            &missing_specs,
                            spec_count,
                            &default_spec_id,
                            change_dir,
                            &specs_dir,
                        )
                    }
                }
                _ => {
                    // No verdict or unknown — check if pending review
                    if let Some(spec_id) = &pending_review_spec {
                        Ok(SpecSubState::Review {
                            spec_id: spec_id.clone(),
                        })
                    } else {
                        next_or_advance(
                            &missing_specs,
                            spec_count,
                            &default_spec_id,
                            change_dir,
                            &specs_dir,
                        )
                    }
                }
            }
        }
        StatePhase::ChangeSpecRevised => {
            if let Some(spec_id) = &pending_review_spec {
                Ok(SpecSubState::Review {
                    spec_id: spec_id.clone(),
                })
            } else {
                next_or_advance(
                    &missing_specs,
                    spec_count,
                    &default_spec_id,
                    change_dir,
                    &specs_dir,
                )
            }
        }
        _ => {
            // Fallback for other phases
            next_or_advance(
                &missing_specs,
                spec_count,
                &default_spec_id,
                change_dir,
                &specs_dir,
            )
        }
    }
}

/// REQ: change-spec.md#CRR1 — find the first spec that is `create_complete: true`
/// but lacks an APPROVED review verdict (either inline in frontmatter or in a
/// separate `review_spec_<id>.md`). Returns `None` only when every complete
/// spec is APPROVED; that is the only state allowed to `AdvanceToImplementation`.
fn find_unreviewed_complete_spec(change_dir: &Path, specs_dir: &Path) -> Option<String> {
    if !specs_dir.exists() {
        return None;
    }
    let mut entries: Vec<_> = std::fs::read_dir(specs_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().and_then(|ext| ext.to_str()) == Some("md")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".base.md"))
                    .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let content = std::fs::read_to_string(&path).ok()?;
        if !is_create_complete(&content) {
            continue;
        }
        let spec_id = path.file_stem()?.to_str()?.to_string();

        // Inline review verdict wins if present.
        if let Some(verdict) = helpers::extract_verdict(&path) {
            if verdict == "APPROVED" || verdict == "PASS" {
                continue;
            }
            return Some(spec_id);
        }
        // Fall back to sibling review files.
        let review_new = change_dir.join(format!("review_spec_{}.md", spec_id));
        let review_legacy = change_dir.join(format!("REVIEW_SPEC_{}.md", spec_id));
        let review_path = if review_new.exists() {
            Some(review_new)
        } else if review_legacy.exists() {
            Some(review_legacy)
        } else {
            None
        };
        match review_path.and_then(|p| helpers::extract_verdict(&p)) {
            Some(v) if v == "APPROVED" || v == "PASS" => continue,
            _ => return Some(spec_id),
        }
    }
    None
}

/// Find the first spec file that doesn't have `create_complete: true`.
fn find_incomplete_spec(specs_dir: &Path) -> Option<String> {
    if !specs_dir.exists() {
        return None;
    }
    let mut entries: Vec<_> = std::fs::read_dir(specs_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let path = e.path();
            path.extension().map(|ext| ext == "md").unwrap_or(false)
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".base.md"))
                    .unwrap_or(false)
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let content = std::fs::read_to_string(entry.path()).ok()?;
        if !is_create_complete(&content) {
            let spec_id = entry.path().file_stem()?.to_str()?.to_string();
            return Some(spec_id);
        }
    }
    None
}

/// Helper: return next Create, Review (if any complete spec is unreviewed),
/// or AdvanceToImplementation.
///
/// REQ: change-spec.md#CRR2 — AdvanceToImplementation is reachable only when
/// every complete spec has an APPROVED verdict on record.
fn next_or_advance(
    missing_specs: &[helpers::SpecInfo],
    spec_count: usize,
    default_spec_id: &str,
    change_dir: &Path,
    specs_dir: &Path,
) -> Result<SpecSubState> {
    if let Some(spec) = missing_specs.first() {
        Ok(SpecSubState::Create {
            spec_id: spec.id.clone(),
            depends: spec.depends.clone(),
        })
    } else if spec_count > 0 {
        if let Some(unreviewed) = find_unreviewed_complete_spec(change_dir, specs_dir) {
            return Ok(SpecSubState::Review {
                spec_id: unreviewed,
            });
        }
        Ok(SpecSubState::AdvanceToImplementation)
    } else {
        Ok(SpecSubState::Create {
            spec_id: default_spec_id.to_string(),
            depends: vec![],
        })
    }
}

// ─── Skeleton Generation ────────────────────────────────────────────────────

/// Generate a skeleton for a new spec.
///
/// Two sources:
/// - **Modify existing spec**: Copy from `.aw/tech-design/{group}/{spec_id}.md`
///   (found via `main_spec_ref`)
/// - **New spec**: Universal template with all possible sections
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn generate_skeleton(
    spec_id: &str,
    title: &str,
    main_spec_ref: Option<&str>,
    merge_strategy: Option<&str>,
    project_root: &Path,
) -> String {
    // Try to copy from main spec if main_spec_ref is provided
    if let Some(ref_id) = main_spec_ref {
        if let Some((content, rel_path)) = find_and_copy_main_spec(ref_id, project_root) {
            // Update frontmatter with change-spec metadata
            let mut result = content;
            result = super::review_helpers::upsert_frontmatter_field(&result, "id", spec_id);
            // Store the full relative path (from .aw/tech-design/) as main_spec_ref
            result = super::review_helpers::upsert_frontmatter_field(
                &result,
                "main_spec_ref",
                &rel_path,
            );
            let strategy = merge_strategy.unwrap_or("extend");
            result = super::review_helpers::upsert_frontmatter_field(
                &result,
                "merge_strategy",
                strategy,
            );
            // Ensure Reviews section exists
            if !result.contains("\n# Reviews") {
                result.push_str("\n\n# Reviews\n");
            }
            return result;
        }
    }

    // Fall back to universal template
    let strategy = merge_strategy.unwrap_or("new");
    UNIVERSAL_SKELETON
        .replace("{spec_id}", spec_id)
        .replace("{title}", title)
        .replace(
            "merge_strategy: new",
            &format!("merge_strategy: {}", strategy),
        )
}

/// Find a main spec by ref (ID or relative path) across all spec groups.
///
/// Returns `(content, relative_path)` where `relative_path` is relative to
/// `.aw/tech-design/` (e.g., `sdd/workflow/auth-flow.md`).
///
/// The `spec_ref` can be:
/// - A bare ID like `auth-flow` (searches all spec groups)
/// - A relative path like `sdd/workflow/auth-flow.md` (direct lookup)
fn find_and_copy_main_spec(spec_ref: &str, project_root: &Path) -> Option<(String, String)> {
    let spec_roots = main_spec_roots(project_root);
    if spec_roots.is_empty() {
        return None;
    }

    // If spec_ref looks like a relative path (contains /), try direct lookup first
    if spec_ref.contains('/') {
        for (group, root) in &spec_roots {
            let direct_ref = spec_ref
                .strip_prefix(&format!("{group}/"))
                .unwrap_or(spec_ref);
            let direct_path = root.join(direct_ref);
            if direct_path.exists() {
                let content = std::fs::read_to_string(&direct_path).ok()?;
                return Some((content, spec_ref.to_string()));
            }
        }
    }

    // Search by bare ID in all spec groups
    for (group, root) in spec_roots {
        if let Some((content, rel_path)) = search_spec_in_dir(&root, spec_ref, &root) {
            let rel_path = if rel_path.is_empty() {
                group
            } else {
                format!("{group}/{rel_path}")
            };
            return Some((content, rel_path));
        }
    }
    None
}

fn main_spec_roots(project_root: &Path) -> Vec<(String, PathBuf)> {
    let mut roots = Vec::new();
    let specs_root = crate::shared::workspace::tech_design_path(project_root);
    if specs_root.exists() {
        if let Ok(entries) = std::fs::read_dir(&specs_root) {
            roots.extend(entries.flatten().filter_map(|entry| {
                if !entry.file_type().ok()?.is_dir() {
                    return None;
                }
                Some((
                    entry.file_name().to_string_lossy().to_string(),
                    entry.path(),
                ))
            }));
        }
    }
    roots.extend(
        crate::shared::workspace::project_tech_design_paths(project_root)
            .into_iter()
            .filter(|(_, root)| root.exists()),
    );
    roots.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    roots.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    roots
}

/// Recursively search for a spec file in a directory.
///
/// Returns `(content, relative_path)` where relative_path is from `specs_root`.
fn search_spec_in_dir(dir: &Path, spec_id: &str, specs_root: &Path) -> Option<(String, String)> {
    let target = format!("{}.md", spec_id);
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(result) = search_spec_in_dir(&path, spec_id, specs_root) {
                return Some(result);
            }
        } else if path.file_name()?.to_str()? == target {
            let content = std::fs::read_to_string(&path).ok()?;
            let rel_path = path.strip_prefix(specs_root).ok()?;
            let rel_str = rel_path.to_str()?.to_string();
            return Some((content, rel_str));
        }
    }
    None
}

// ─── Section Operations ─────────────────────────────────────────────────────

/// Replace a section's content in a spec markdown file.
///
/// Sections are identified by their H2 heading (e.g., `## Overview`).
/// For sub-sections under Diagrams/API Spec, the `section` param uses
/// the H2 name — the agent fills the entire H2 block.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn replace_section(content: &str, section: &str, new_content: &str) -> String {
    let heading = section_to_heading(section);
    let annotation = section_annotation(section);
    let clean_content = strip_leading_annotation(new_content);
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<String> = Vec::new();
    let mut in_target = false;
    let mut target_level = 0;
    let mut found = false;

    for line in &lines {
        if is_heading(line) {
            let level = heading_level(line);
            if line.trim().eq_ignore_ascii_case(&heading) {
                in_target = true;
                found = true;
                target_level = level;
                // Write heading + annotation + content
                result.push(line.to_string());
                if let Some(ref ann) = annotation {
                    result.push(ann.clone());
                }
                result.push(String::new());
                for new_line in clean_content.lines() {
                    result.push(new_line.to_string());
                }
                continue;
            } else if in_target && level <= target_level {
                // Exiting target section
                in_target = false;
            }
        }

        if !in_target {
            result.push(line.to_string());
        }
    }

    // If heading not found, insert before "# Reviews" (or append at end).
    if !found {
        let reviews_idx = result.iter().position(|l| l.trim() == "# Reviews");
        let mut insert: Vec<String> = vec![String::new(), heading.clone()];
        if let Some(ref ann) = annotation {
            insert.push(ann.clone());
        }
        insert.push(String::new());
        for l in clean_content.lines() {
            insert.push(l.to_string());
        }
        insert.push(String::new());

        if let Some(idx) = reviews_idx {
            let suffix = result.split_off(idx);
            result.extend(insert);
            result.extend(suffix);
        } else {
            result.extend(insert);
        }
    }

    result.join("\n")
}

/// Generate the `<!-- type: X lang: Y -->` annotation for a section.
///
/// The alignment checker requires this annotation after every `## Section` heading.
/// Returns `None` for unrecognized sections.
fn section_annotation(section: &str) -> Option<String> {
    let (type_name, lang) = match section {
        "overview" => ("overview", "markdown"),
        "requirements" => ("requirements", "mermaid"),
        "scenarios" => ("scenarios", "yaml"),
        "unit-test" | "unit_test" => ("unit-test", "mermaid"),
        "e2e-test" | "e2e_test" | "e2e" => ("e2e-test", "yaml"),
        "changes" => ("changes", "yaml"),
        "doc" => ("doc", "markdown"),
        "db-model" | "erd" => ("db-model", "mermaid"),
        "dependency" | "class" => ("dependency", "mermaid"),
        "state-machine" | "state" => ("state-machine", "mermaid"),
        "logic" | "flowchart" => ("logic", "mermaid"),
        "interaction" | "sequence" => ("interaction", "mermaid"),
        "mindmap" => ("mindmap", "mermaid"),
        "rest-api" | "openapi" => ("rest-api", "yaml"),
        "rpc-api" | "openrpc" => ("rpc-api", "yaml"),
        "async-api" | "asyncapi" => ("async-api", "yaml"),
        "wireframe" | "frontend" => ("wireframe", "yaml"),
        "cli" => ("cli", "yaml"),
        "schema" => ("schema", "yaml"),
        "config" => ("config", "yaml"),
        "component" => ("component", "yaml"),
        "design-token" => ("design-token", "yaml"),
        "runtime-image" | "container-image" | "container" | "dockerfile" => {
            ("runtime-image", "yaml")
        }
        "deployment" | "deploy" | "kustomize" | "kubernetes" | "k8s" => ("deployment", "yaml"),
        "diagrams" => ("diagrams", "mermaid"),
        "api_spec" => ("api-spec", "yaml"),
        _ => return None,
    };
    Some(format!("<!-- type: {} lang: {} -->", type_name, lang))
}

/// Strip a leading `<!-- type: ... -->` annotation from content to prevent duplication.
fn strip_leading_annotation(content: &str) -> &str {
    let trimmed = content.trim_start();
    if trimmed.starts_with("<!-- type:") {
        if let Some(end) = trimmed.find("-->") {
            let after = &trimmed[end + 3..];
            return after.trim_start_matches('\n');
        }
    }
    content
}

/// Map section name to markdown heading.
fn section_to_heading(section: &str) -> String {
    match section {
        "overview" => "## Overview".to_string(),
        "requirements" => "## Requirements".to_string(),
        "scenarios" => "## Scenarios".to_string(),
        "unit-test" | "unit_test" => "## Unit Test".to_string(),
        "e2e-test" | "e2e_test" | "e2e" => "## E2E Test".to_string(),
        "changes" => "## Changes".to_string(),
        "doc" => "## Doc".to_string(),
        "db-model" | "erd" => "## Data Model".to_string(),
        "dependency" | "class" => "## Dependencies".to_string(),
        "state-machine" | "state" => "## State Machine".to_string(),
        "logic" | "flowchart" => "## Logic".to_string(),
        "interaction" | "sequence" => "## Interaction".to_string(),
        "mindmap" => "## Mindmap".to_string(),
        "rest-api" | "openapi" => "## REST API".to_string(),
        "rpc-api" | "openrpc" => "## RPC API".to_string(),
        "async-api" | "asyncapi" => "## Async API".to_string(),
        "wireframe" | "frontend" => "## Wireframe".to_string(),
        "cli" => "## CLI".to_string(),
        "schema" => "## Schema".to_string(),
        "config" => "## Config".to_string(),
        "component" => "## Component".to_string(),
        "design-token" => "## Design Token".to_string(),
        "runtime-image" | "container-image" | "container" | "dockerfile" => {
            "## Runtime Image".to_string()
        }
        "deployment" | "deploy" | "kustomize" | "kubernetes" | "k8s" => "## Deployment".to_string(),
        _ => format!("## {}", section),
    }
}

fn is_heading(line: &str) -> bool {
    line.starts_with('#')
}

fn heading_level(line: &str) -> usize {
    line.chars().take_while(|c| *c == '#').count()
}

// ─── Prune Placeholder Sections ─────────────────────────────────────────────

/// Return `true` if `body` is a placeholder sentinel that should be pruned.
///
/// REQ: change-spec.md#NAP3 — two sentinels are recognised:
/// - `<!-- TODO -->` (with optional inline hints inside the same comment)
/// - bare `N/A` (author-declared not-applicable)
fn is_placeholder_body(body: &str) -> bool {
    body == "<!-- TODO -->" || body == "N/A"
}

/// Return `true` if `line` is a section type annotation comment such as
/// `<!-- type: schema lang: yaml -->`. Trimmed leading whitespace expected.
fn is_annotation_line(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("<!-- type:") && t.ends_with("-->")
}

/// Remove sections that still contain only `<!-- TODO -->` or `N/A` placeholders.
///
/// Walks all H2 and H3 sections; if a section's **direct body** (up to the
/// next heading at same or higher level) trims to a placeholder sentinel
/// (see [`is_placeholder_body`]), the heading and its direct body are removed.
/// Child headings are processed independently.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn prune_todo_sections(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result: Vec<&str> = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];

        if is_heading(line) {
            let level = heading_level(line);
            // Only prune H2 and H3 sections
            if level == 2 || level == 3 {
                // Collect ONLY the direct body (stop at any heading)
                let mut body_lines: Vec<&str> = Vec::new();
                let mut j = i + 1;
                while j < lines.len() {
                    let next = lines[j];
                    if is_heading(next) {
                        break;
                    }
                    body_lines.push(next);
                    j += 1;
                }

                // Check if direct body is only a placeholder. The section
                // annotation (`<!-- type: X lang: Y -->`) is not considered
                // part of the body — it is structural metadata, not content.
                let body: String = body_lines
                    .iter()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !is_annotation_line(l))
                    .collect::<Vec<_>>()
                    .join(" ");

                if is_placeholder_body(&body) {
                    // Skip this heading + its direct body
                    i = j;
                    continue;
                }

                // Keep the heading and its direct body
                result.push(line);
                for bl in &body_lines {
                    result.push(bl);
                }
                i = j;
                continue;
            }
        }

        result.push(line);
        i += 1;
    }

    // Clean up consecutive blank lines
    let joined = result.join("\n");
    let mut clean = String::new();
    let mut blank_count = 0;
    for line in joined.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                clean.push('\n');
            }
        } else {
            blank_count = 0;
            if !clean.is_empty() {
                clean.push('\n');
            }
            clean.push_str(line);
        }
    }
    clean.push('\n');
    clean
}

// ─── Frontmatter Helpers ────────────────────────────────────────────────────

/// Read `fill_sections` from spec frontmatter (YAML list).
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn read_fill_sections(content: &str) -> Vec<String> {
    if !content.starts_with("---\n") {
        return vec![];
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return vec![],
    };
    let fm = &content[4..closing];

    let mut in_fill = false;
    let mut sections = Vec::new();
    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("fill_sections:") {
            in_fill = true;
            // Handle inline format: fill_sections: [a, b, c]
            let after = trimmed.trim_start_matches("fill_sections:").trim();
            if after.starts_with('[') && after.ends_with(']') {
                let inner = &after[1..after.len() - 1];
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"').trim_matches('\'');
                    if !s.is_empty() {
                        sections.push(s.to_string());
                    }
                }
                return sections;
            }
            continue;
        }
        if in_fill {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                sections.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break; // End of list
            }
        }
    }
    sections
}

/// Read `filled_sections` from spec frontmatter (YAML list).
pub fn read_filled_sections(content: &str) -> Vec<String> {
    if !content.starts_with("---\n") {
        return vec![];
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return vec![],
    };
    let fm = &content[4..closing];

    let mut in_filled = false;
    let mut sections = Vec::new();
    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("filled_sections:") {
            in_filled = true;
            let after = trimmed.trim_start_matches("filled_sections:").trim();
            if after.starts_with('[') && after.ends_with(']') {
                let inner = &after[1..after.len() - 1];
                for item in inner.split(',') {
                    let s = item.trim().trim_matches('"').trim_matches('\'');
                    if !s.is_empty() {
                        sections.push(s.to_string());
                    }
                }
                return sections;
            }
            continue;
        }
        if in_filled {
            if trimmed.starts_with("- ") {
                let item = trimmed
                    .trim_start_matches("- ")
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                sections.push(item.to_string());
            } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
        }
    }
    sections
}

/// Parse a fill_sections entry into base section name and optional flag.
///
/// Supports both formats:
/// - `"component (optional)"` → `("component", true)`
/// - `"overview"` → `("overview", false)`
///
/// This is the canonical function for interpreting fill_sections entries.
/// Use this when comparing with `filled_sections` (which stores bare names).
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn parse_fill_section(s: &str) -> (&str, bool) {
    crate::models::spec_rules::parse_fill_section_str(s)
}

/// Extract base section name from a fill_sections entry, stripping `(optional)`.
///
/// Convenience wrapper around `parse_fill_section` for contexts that only
/// need the name.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn fill_section_base_name(s: &str) -> &str {
    parse_fill_section(s).0
}

/// Check if a fill_sections entry is marked optional.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn is_fill_section_optional(s: &str) -> bool {
    parse_fill_section(s).1
}

/// Read `main_spec_ref` from spec frontmatter.
///
/// Returns `None` if the field is missing or set to `~` (YAML null).
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn read_main_spec_ref(content: &str) -> Option<String> {
    if !content.starts_with("---\n") {
        return None;
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return None,
    };
    let fm = &content[4..closing];
    for line in fm.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("main_spec_ref:") {
            let val = trimmed.trim_start_matches("main_spec_ref:").trim();
            if val == "~" || val.is_empty() {
                return None;
            }
            // Strip YAML quotes (single or double) from the value
            let val = val.trim_matches('"').trim_matches('\'');
            return Some(val.to_string());
        }
    }
    None
}

/// Strip change-spec-only frontmatter fields for merge to main spec.
///
/// Removes: `fill_sections`, `filled_sections`, `create_complete`,
/// `review_verdict`, `review_iteration`, `problem_sections`.
/// Keeps: `id`, `main_spec_ref`, and all other fields.
pub fn strip_change_spec_fields(content: &str) -> String {
    let fields_to_strip = [
        "fill_sections",
        "filled_sections",
        "create_complete",
        "review_verdict",
        "review_iteration",
        "problem_sections",
    ];
    let mut result = content.to_string();
    for field in &fields_to_strip {
        result = super::review_helpers::remove_frontmatter_field(&result, field);
    }
    // Also strip the Reviews section (change-spec artifact, not for main spec)
    result = super::review_helpers::strip_review_section(&result);
    result
}

/// Read `create_complete` flag from spec frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/tools/common_change_spec/helpers.md#source
pub fn is_create_complete(content: &str) -> bool {
    if !content.starts_with("---\n") {
        return false;
    }
    let closing = match content[4..].find("\n---") {
        Some(pos) => 4 + pos,
        None => return false,
    };
    let fm = &content[4..closing];
    fm.lines().any(|l| l.trim() == "create_complete: true")
}

// ─── Spec Plan (re-exports from spec_plan module) ───────────────────────────

/// Re-export: deduplicate spec_plan entries across groups.
pub use super::spec_plan::deduplicate_spec_plans;
/// Re-export: prepare spec files from spec_plan entries across all groups.
pub use super::spec_plan::prepare_specs_from_plan;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_spec/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_skeleton_universal() {
        let skeleton = generate_skeleton("my-spec", "My Spec Title", None, None, Path::new("/tmp"));
        assert!(skeleton.contains("id: my-spec"));
        assert!(skeleton.contains("# My Spec Title"));
        assert!(skeleton.contains("## Overview"));
        assert!(skeleton.contains("## Requirements"));
        assert!(skeleton.contains("## Scenarios"));
        assert!(skeleton.contains("## Interaction"));
        assert!(skeleton.contains("## Logic"));
        assert!(skeleton.contains("## REST API"));
        assert!(skeleton.contains("## Schema"));
        assert!(skeleton.contains("## Unit Test"));
        assert!(skeleton.contains("## E2E Test"));
        assert!(skeleton.contains("## Changes"));
        assert!(!skeleton.contains("## Diagrams"));
        assert!(!skeleton.contains("## API Spec"));
        assert!(skeleton.contains("# Reviews"));
        assert!(skeleton.contains("merge_strategy: new"));
    }

    #[test]
    fn test_skeleton_has_section_annotations() {
        let skeleton = generate_skeleton(
            "annotated-spec",
            "Annotated Spec",
            None,
            None,
            Path::new("/tmp"),
        );
        // Verify section type annotations are present (new format: 3 langs only)
        assert!(skeleton.contains("<!-- type: overview lang: markdown -->"));
        assert!(skeleton.contains("<!-- type: changes lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: requirements lang: mermaid -->")); // was markdown
        assert!(skeleton.contains("<!-- type: scenarios lang: yaml -->")); // was markdown
        assert!(skeleton.contains("<!-- type: unit-test lang: mermaid -->"));
        assert!(skeleton.contains("<!-- type: e2e-test lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: interaction lang: mermaid -->"));
        assert!(skeleton.contains("<!-- type: rest-api lang: yaml -->"));
        assert!(skeleton.contains("<!-- type: rpc-api lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: schema lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: config lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: component lang: yaml -->")); // was json
        assert!(skeleton.contains("<!-- type: design-token lang: yaml -->")); // was json
    }

    #[test]
    fn test_get_specs_dir_no_group() {
        let base = Path::new("/tmp/change");
        let dir = get_specs_dir(base, None);
        assert_eq!(dir, Path::new("/tmp/change/specs"));
    }

    #[test]
    fn test_get_specs_dir_with_group() {
        let base = Path::new("/tmp/change");
        let dir = get_specs_dir(base, Some("feature-a"));
        assert_eq!(dir, Path::new("/tmp/change/groups/feature-a/specs"));
    }

    #[test]
    fn test_get_spec_path_no_group() {
        let base = Path::new("/tmp/change");
        let path = get_spec_path(base, None, "my-spec");
        assert_eq!(path, Path::new("/tmp/change/specs/my-spec.md"));
    }

    #[test]
    fn test_get_spec_path_with_group() {
        let base = Path::new("/tmp/change");
        let path = get_spec_path(base, Some("group-1"), "my-spec");
        assert_eq!(
            path,
            Path::new("/tmp/change/groups/group-1/specs/my-spec.md")
        );
    }

    // REQ: change-spec.md#NAP3 — `N/A` body sections are pruned alongside TODOs.
    #[test]
    fn test_prune_na_sections() {
        let content = r#"---
id: test
---

# Test Spec

## Overview

Real overview content.

## Schema
<!-- type: schema lang: yaml -->

N/A

## Config
<!-- type: config lang: yaml -->

```yaml
key: value
```

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(pruned.contains("## Overview"), "real section kept");
        assert!(!pruned.contains("## Schema"), "N/A section must be pruned");
        assert!(pruned.contains("## Config"), "filled section kept");
        assert!(pruned.contains("# Reviews"), "reviews heading kept");
    }

    #[test]
    fn test_prune_mixed_na_and_todo() {
        let content = r#"---
id: test
---

# Test Spec

## Overview
<!-- type: overview lang: markdown -->

<!-- TODO -->

## Schema
<!-- type: schema lang: yaml -->

N/A

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
flowchart TD
    A --> B
```

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(!pruned.contains("## Overview"), "TODO pruned");
        assert!(!pruned.contains("## Schema"), "N/A pruned");
        assert!(pruned.contains("## Logic"), "real section kept");
    }

    #[test]
    fn test_prune_na_prose_is_not_pruned() {
        let content = r#"---
id: test
---

# Test Spec

## Overview
<!-- type: overview lang: markdown -->

N/A because historically we skipped this; new owners: fill it in.

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(
            pruned.contains("## Overview"),
            "section with N/A-prefixed prose must be kept (only bare N/A sentinel prunes)"
        );
    }

    #[test]
    fn test_prune_todo_sections() {
        let content = r#"---
id: test
---

# Test Spec

## Overview

Some real content here.

## Requirements

<!-- TODO -->

## Scenarios

Real scenarios here.

## Diagrams

### Sequence Diagram
<!-- TODO -->

### Flowchart

Real flowchart here.

### Class Diagram
<!-- TODO -->

## Unit Test

<!-- TODO -->

## Changes

<!-- TODO -->

# Reviews
"#;
        let pruned = prune_todo_sections(content);
        assert!(pruned.contains("## Overview"));
        assert!(pruned.contains("Some real content here."));
        assert!(!pruned.contains("## Requirements"));
        assert!(pruned.contains("## Scenarios"));
        assert!(!pruned.contains("### Sequence Diagram"));
        assert!(pruned.contains("### Flowchart"));
        assert!(!pruned.contains("### Class Diagram"));
        assert!(!pruned.contains("## Unit Test"));
        assert!(!pruned.contains("## Changes"));
        assert!(pruned.contains("# Reviews"));
    }

    #[test]
    fn test_replace_section() {
        let content = r#"---
id: test
---

# Test

## Overview

Old overview.

## Requirements

Old requirements.

## Scenarios

Old scenarios.
"#;
        let result = replace_section(
            content,
            "overview",
            "New overview content.\n\nMore details.",
        );
        assert!(result.contains("New overview content."));
        assert!(result.contains("More details."));
        assert!(!result.contains("Old overview."));
        assert!(result.contains("Old requirements."));
        assert!(result.contains("Old scenarios."));
    }

    #[test]
    fn test_read_fill_sections_yaml_list() {
        let content = "---\nid: test\nfill_sections:\n- overview\n- requirements\n- scenarios\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(sections, vec!["overview", "requirements", "scenarios"]);
    }

    #[test]
    fn test_read_fill_sections_inline() {
        let content = "---\nid: test\nfill_sections: [overview, requirements]\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(sections, vec!["overview", "requirements"]);
    }

    #[test]
    fn test_read_main_spec_ref_unquoted() {
        let content = "---\nid: test\nmain_spec_ref: foo/bar.md\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_double_quoted() {
        let content = "---\nid: test\nmain_spec_ref: \"foo/bar.md\"\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_single_quoted() {
        let content = "---\nid: test\nmain_spec_ref: 'foo/bar.md'\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), Some("foo/bar.md".to_string()));
    }

    #[test]
    fn test_read_main_spec_ref_null() {
        let content = "---\nid: test\nmain_spec_ref: ~\n---\n# Body\n";
        assert_eq!(read_main_spec_ref(content), None);
    }

    #[test]
    fn test_is_create_complete() {
        let content = "---\nid: test\ncreate_complete: true\n---\n\n# Body\n";
        assert!(is_create_complete(content));

        let content2 = "---\nid: test\n---\n\n# Body\n";
        assert!(!is_create_complete(content2));
    }

    #[test]
    fn test_resolve_group_id_for_spec_from_existing_file() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let group_dir = change_dir.join("groups/my-group/specs");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(group_dir.join("my-spec.md"), "---\nid: my-spec\n---\n").unwrap();
        assert_eq!(
            resolve_group_id_for_spec(change_dir, "my-spec"),
            Some("my-group".to_string())
        );
    }

    #[test]
    fn test_resolve_group_id_for_spec_from_spec_plan() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let group_dir = change_dir.join("groups/plan-group");
        std::fs::create_dir_all(&group_dir).unwrap();
        std::fs::write(
            group_dir.join("spec_plan.yaml"),
            "- spec_id: plan-spec\n  action: create\n",
        )
        .unwrap();
        assert_eq!(
            resolve_group_id_for_spec(change_dir, "plan-spec"),
            Some("plan-group".to_string())
        );
    }

    #[test]
    fn test_resolve_group_id_for_spec_no_groups_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        assert_eq!(resolve_group_id_for_spec(change_dir, "any-spec"), None);
    }

    #[test]
    fn test_resolve_group_id_for_spec_not_found_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        std::fs::create_dir_all(change_dir.join("groups/group-a")).unwrap();
        assert_eq!(resolve_group_id_for_spec(change_dir, "missing-spec"), None);
    }

    // ── replace_section: insert-before-Reviews for missing headings ───────────

    #[test]
    fn test_replace_section_inserts_before_reviews_when_missing() {
        // When the target section heading does not exist in the document,
        // replace_section must insert it immediately before "# Reviews".
        // This handles modify-action specs copied from main specs that lack the section.
        let content =
            "---\nid: test\n---\n\n# Test\n\n## Overview\n\nExisting content.\n\n# Reviews\n";
        let result = replace_section(content, "requirements", "R1: Must do something.");
        assert!(
            result.contains("## Requirements"),
            "inserted heading must be present"
        );
        assert!(
            result.contains("R1: Must do something."),
            "inserted content must be present"
        );
        assert!(
            result.contains("## Overview"),
            "original overview must be preserved"
        );
        assert!(
            result.contains("Existing content."),
            "original overview content must be preserved"
        );
        // Inserted section must appear before # Reviews
        let reqs_pos = result.find("## Requirements").unwrap();
        let reviews_pos = result.find("# Reviews").unwrap();
        assert!(
            reqs_pos < reviews_pos,
            "inserted section must come before # Reviews, got:\n{}",
            result
        );
    }

    #[test]
    fn test_replace_section_appends_at_end_when_no_reviews_and_missing() {
        // When neither the target section nor a "# Reviews" section exists,
        // replace_section must append the new section at the end of the document.
        let content = "---\nid: test\n---\n\n# Test\n\n## Overview\n\nExisting content.\n";
        let result = replace_section(content, "requirements", "R1: Must work.");
        assert!(
            result.contains("## Requirements"),
            "appended heading must be present"
        );
        assert!(
            result.contains("R1: Must work."),
            "appended content must be present"
        );
        assert!(
            result.contains("## Overview"),
            "original section must be preserved"
        );
    }

    // ── strip_change_spec_fields: merge_strategy is no longer stripped ────────

    // ─── parse_fill_section / fill_section_base_name / is_fill_section_optional ─

    #[test]
    fn test_parse_fill_section_required() {
        let (name, optional) = parse_fill_section("overview");
        assert_eq!(name, "overview");
        assert!(!optional);
    }

    #[test]
    fn test_parse_fill_section_optional() {
        let (name, optional) = parse_fill_section("component (optional)");
        assert_eq!(name, "component");
        assert!(optional);
    }

    #[test]
    fn test_parse_fill_section_design_token_optional() {
        let (name, optional) = parse_fill_section("design-token (optional)");
        assert_eq!(name, "design-token");
        assert!(optional);
    }

    #[test]
    fn test_fill_section_base_name_required() {
        assert_eq!(fill_section_base_name("overview"), "overview");
    }

    #[test]
    fn test_fill_section_base_name_strips_optional() {
        assert_eq!(fill_section_base_name("component (optional)"), "component");
    }

    #[test]
    fn test_fill_section_base_name_design_token() {
        assert_eq!(
            fill_section_base_name("design-token (optional)"),
            "design-token"
        );
    }

    #[test]
    fn test_is_fill_section_optional_false() {
        assert!(!is_fill_section_optional("overview"));
        assert!(!is_fill_section_optional("changes"));
        assert!(!is_fill_section_optional("wireframe"));
    }

    #[test]
    fn test_is_fill_section_optional_true() {
        assert!(is_fill_section_optional("component (optional)"));
        assert!(is_fill_section_optional("design-token (optional)"));
    }

    #[test]
    fn test_read_fill_sections_with_optional_markers() {
        let content = "---\nid: test\nfill_sections: [overview, component (optional), design-token (optional), changes]\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(
            sections,
            vec![
                "overview",
                "component (optional)",
                "design-token (optional)",
                "changes",
            ]
        );
    }

    #[test]
    fn test_read_fill_sections_yaml_list_with_optional_markers() {
        let content = "---\nid: test\nfill_sections:\n- overview\n- wireframe\n- component (optional)\n- design-token (optional)\n- changes\n---\n\n# Body\n";
        let sections = read_fill_sections(content);
        assert_eq!(
            sections,
            vec![
                "overview",
                "wireframe",
                "component (optional)",
                "design-token (optional)",
                "changes",
            ]
        );
    }

    #[test]
    fn test_strip_change_spec_fields_preserves_merge_strategy() {
        // merge_strategy is dead code but is no longer in the stripped fields list.
        // After strip it must remain in the frontmatter (not silently removed).
        let content = "---\n\
            id: test\n\
            main_spec_ref: foo/bar.md\n\
            merge_strategy: extend\n\
            fill_sections: [overview]\n\
            filled_sections: [overview]\n\
            create_complete: true\n\
            ---\n\n# Test\n\nContent.\n";
        let stripped = strip_change_spec_fields(content);
        // Change-spec-only lifecycle fields must be removed
        assert!(
            !stripped.contains("fill_sections"),
            "fill_sections must be stripped"
        );
        assert!(
            !stripped.contains("filled_sections"),
            "filled_sections must be stripped"
        );
        assert!(
            !stripped.contains("create_complete"),
            "create_complete must be stripped"
        );
        // merge_strategy is NOT a change-spec-only field — must be preserved
        assert!(
            stripped.contains("merge_strategy: extend"),
            "merge_strategy must NOT be stripped; it belongs to the main spec"
        );
        // Core fields must be preserved
        assert!(stripped.contains("id: test"), "id must be preserved");
        assert!(
            stripped.contains("main_spec_ref: foo/bar.md"),
            "main_spec_ref must be preserved"
        );
    }

    // ── find_unreviewed_complete_spec: CRR is mandatory ───────────────────────

    #[test]
    fn test_find_unreviewed_complete_spec_with_approved_inline_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let content = "---\n\
            id: foo-spec\n\
            create_complete: true\n\
            review_verdict: APPROVED\n\
            ---\n\n# Foo\n";
        std::fs::write(specs_dir.join("foo-spec.md"), content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }

    #[test]
    fn test_find_unreviewed_complete_spec_without_verdict_returns_id() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let content = "---\n\
            id: bar-spec\n\
            create_complete: true\n\
            ---\n\n# Bar\n";
        std::fs::write(specs_dir.join("bar-spec.md"), content).unwrap();
        assert_eq!(
            find_unreviewed_complete_spec(change_dir, &specs_dir),
            Some("bar-spec".to_string())
        );
    }

    #[test]
    fn test_find_unreviewed_complete_spec_with_sibling_review_approved_returns_none() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        let spec_content = "---\n\
            id: baz-spec\n\
            create_complete: true\n\
            ---\n\n# Baz\n";
        std::fs::write(specs_dir.join("baz-spec.md"), spec_content).unwrap();
        let review_content = "---\n\
            verdict: APPROVED\n\
            ---\n\n# Review\n";
        std::fs::write(change_dir.join("review_spec_baz-spec.md"), review_content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }

    #[test]
    fn test_find_unreviewed_complete_spec_skips_incomplete() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let change_dir = temp_dir.path();
        let specs_dir = change_dir.join("specs");
        std::fs::create_dir_all(&specs_dir).unwrap();
        // Incomplete spec must NOT be flagged for review
        let content = "---\n\
            id: qux-spec\n\
            create_complete: false\n\
            ---\n\n# Qux\n";
        std::fs::write(specs_dir.join("qux-spec.md"), content).unwrap();
        assert_eq!(find_unreviewed_complete_spec(change_dir, &specs_dir), None);
    }
}
// CODEGEN-END
