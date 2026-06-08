---
id: sdd-tools-common-change-spec-helpers
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change spec helpers

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/common_change_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ALL_SECTIONS` | projects/agentic-workflow/src/tools/common_change_spec.rs | constant | pub | 285 |  |
| `SpecSubState` | projects/agentic-workflow/src/tools/common_change_spec.rs | enum | pub | 439 |  |
| `UNIVERSAL_SKELETON` | projects/agentic-workflow/src/tools/common_change_spec.rs | constant | pub | 30 |  |
| `fill_section_base_name` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1281 | fill_section_base_name(s: &str) -> &str |
| `find_spec_path` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 343 | find_spec_path(change_dir: &std::path::Path, spec_id: &str) -> std::path::PathBuf |
| `generate_skeleton` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 770 | generate_skeleton(     spec_id: &str,     title: &str,     main_spec_ref: Option<&str>,     merge_strategy: Option<&str>,     project_root: &Path, ) -> String |
| `get_primary_specs_dir` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 369 | get_primary_specs_dir(change_dir: &std::path::Path) -> std::path::PathBuf |
| `get_spec_path` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 329 | get_spec_path(     change_dir: &std::path::Path,     group_id: Option<&str>,     spec_id: &str, ) -> std::path::PathBuf |
| `get_specs_dir` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 320 | get_specs_dir(change_dir: &std::path::Path, group_id: Option<&str>) -> std::path::PathBuf |
| `is_create_complete` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1344 | is_create_complete(content: &str) -> bool |
| `is_fill_section_optional` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1287 | is_fill_section_optional(s: &str) -> bool |
| `parse_fill_section` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1272 | parse_fill_section(s: &str) -> (&str, bool) |
| `prune_todo_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1091 | prune_todo_sections(content: &str) -> String |
| `read_fill_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1171 | read_fill_sections(content: &str) -> Vec<String> |
| `read_filled_sections` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1218 | read_filled_sections(content: &str) -> Vec<String> |
| `read_main_spec_ref` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1295 | read_main_spec_ref(content: &str) -> Option<String> |
| `replace_section` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 911 | replace_section(content: &str, section: &str, new_content: &str) -> String |
| `resolve_group_id_for_spec` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 395 | resolve_group_id_for_spec(change_dir: &std::path::Path, spec_id: &str) -> Option<String> |
| `resolve_next_spec` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 463 | resolve_next_spec(change_dir: &Path, change_id: &str) -> Result<SpecSubState> |
| `strip_change_spec_fields` | projects/agentic-workflow/src/tools/common_change_spec.rs | function | pub | 1324 | strip_change_spec_fields(content: &str) -> String |
## Source
<!-- type: source lang: rust -->

````rust
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-common-change-spec-rs-helpers>"
    description: "Change-spec helper runtime excluding the schema enum and regression tests."
```
