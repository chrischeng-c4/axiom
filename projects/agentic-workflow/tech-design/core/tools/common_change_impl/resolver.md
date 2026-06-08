---
id: sdd-tools-common-change-impl-resolver
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools common change impl resolver

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/common_change_impl.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ImplSubState` | projects/agentic-workflow/src/tools/common_change_impl.rs | enum | pub | 26 |  |
| `MAX_SPEC_REVISIONS` | projects/agentic-workflow/src/tools/common_change_impl.rs | constant | pub | 15 |  |
| `build_spec_execution_order` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 249 | build_spec_execution_order(specs_dir: &Path) -> Vec<String> |
| `build_spec_execution_order_from_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 409 | build_spec_execution_order_from_paths(paths: &[std::path::PathBuf]) -> Vec<String> |
| `collect_all_spec_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 354 | collect_all_spec_paths(change_dir: &Path) -> Vec<std::path::PathBuf> |
| `find_inline_reviews` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 524 | find_inline_reviews(impl_path: &Path) -> (HashSet<String>, HashSet<String>) |
| `is_codegen_eligible_for_spec` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 583 | is_codegen_eligible_for_spec(specs_dir: &Path, spec_id: &str) -> bool |
| `is_codegen_eligible_in_paths` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 433 | is_codegen_eligible_in_paths(paths: &[std::path::PathBuf], spec_id: &str) -> bool |
| `parse_refs_frontmatter` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 458 | parse_refs_frontmatter(path: &Path) -> Vec<String> |
| `resolve_next_impl` | projects/agentic-workflow/src/tools/common_change_impl.rs | function | pub | 64 | resolve_next_impl(     change_dir: &Path,     _change_id: &str, ) -> Result<(ImplSubState, Option<String>, Option<String>)> |
## Source
<!-- type: source lang: rust -->

````rust
/// Resolve the current implementation sub-state from change directory.
///
/// Reads STATE.yaml, builds spec execution order, checks implementation.md
/// existence and inline reviews, then determines the next action.
///
/// Returns `(sub_state, new_current_spec_id, increment_revision_for_spec)`.
pub fn resolve_next_impl(
    change_dir: &Path,
    _change_id: &str,
) -> Result<(ImplSubState, Option<String>, Option<String>)> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();
    let current_spec_id = sm.state().current_task_id.clone();
    let spec_revisions = sm.state().task_revisions.clone();
    let impl_spec_phase = sm.state().impl_spec_phase.clone();
    drop(sm);

    let spec_paths = collect_all_spec_paths(change_dir);
    let spec_ids = build_spec_execution_order_from_paths(&spec_paths);
    let impl_path = change_dir.join("implementation.md");
    let impl_written = impl_path.exists();

    if spec_ids.is_empty() {
        return Ok((ImplSubState::NoSpecs, None, None));
    }

    let (reviewed_specs, approved_specs) = if impl_written {
        find_inline_reviews(&impl_path)
    } else {
        (HashSet::new(), HashSet::new())
    };

    let just_revised = matches!(phase, StatePhase::ChangeImplementationRevised);

    determine_sub_state(
        &spec_ids,
        &spec_paths,
        &current_spec_id,
        &spec_revisions,
        impl_written,
        &reviewed_specs,
        &approved_specs,
        change_dir,
        just_revised,
        &impl_spec_phase,
    )
}

/// Pure logic: determine sub-state from implementation context.
fn determine_sub_state(
    spec_ids: &[String],
    spec_paths: &[std::path::PathBuf],
    current_spec_id: &Option<String>,
    spec_revisions: &HashMap<String, u32>,
    impl_written: bool,
    reviewed_specs: &HashSet<String>,
    approved_specs: &HashSet<String>,
    _change_dir: &Path,
    just_revised: bool,
    impl_spec_phase: &HashMap<String, String>,
) -> Result<(ImplSubState, Option<String>, Option<String>)> {
    if !impl_written {
        // IMPLEMENTATION LOOP: implement each spec in order

        // Check if current spec has an impl_spec_phase entry (phase dispatched but not yet verified)
        if let Some(current) = current_spec_id.as_ref() {
            if let Some(phase) = impl_spec_phase.get(current.as_str()) {
                match phase.as_str() {
                    "code" => {
                        return Ok((
                            ImplSubState::BuildCheck {
                                spec_id: current.clone(),
                            },
                            None,
                            None,
                        ))
                    }
                    "tests" => {
                        return Ok((
                            ImplSubState::TestCountCheck {
                                spec_id: current.clone(),
                            },
                            None,
                            None,
                        ))
                    }
                    _ => {}
                }
            }
        }

        if current_spec_id.is_none() {
            let first = spec_ids[0].clone();
            return Ok((
                ImplSubState::ImplementSpecCode {
                    spec_id: first.clone(),
                    is_first: true,
                },
                Some(first),
                None,
            ));
        }

        let current = current_spec_id.as_ref().unwrap();
        let current_idx = spec_ids.iter().position(|s| s == current);

        match current_idx {
            Some(idx) if idx + 1 < spec_ids.len() => {
                let next = spec_ids[idx + 1].clone();
                let sub_state = if is_codegen_eligible_in_paths(spec_paths, &next) {
                    ImplSubState::ImplementSpecWithCodegen {
                        spec_id: next.clone(),
                    }
                } else {
                    ImplSubState::ImplementSpecCode {
                        spec_id: next.clone(),
                        is_first: false,
                    }
                };
                return Ok((sub_state, Some(next), None));
            }
            _ => {
                return Ok((ImplSubState::WriteDiff, None, None));
            }
        }
    }

    // REVIEW LOOP: implementation.md exists, find first non-approved spec
    for spec_id in spec_ids {
        if approved_specs.contains(spec_id) {
            continue;
        }

        let revisions = spec_revisions.get(spec_id.as_str()).copied().unwrap_or(0);

        if reviewed_specs.contains(spec_id) {
            // Just revised for this spec -> force re-review
            if just_revised && current_spec_id.as_deref() == Some(spec_id.as_str()) {
                return Ok((
                    ImplSubState::ReviewSpec {
                        spec_id: spec_id.clone(),
                    },
                    Some(spec_id.clone()),
                    None,
                ));
            }
            // Has a review but not approved -> revise or terminal failure
            if revisions >= MAX_SPEC_REVISIONS {
                return Ok((
                    ImplSubState::TerminalFailure {
                        spec_id: spec_id.clone(),
                        revisions,
                    },
                    None,
                    None,
                ));
            }
            return Ok((
                ImplSubState::ReviseSpec {
                    spec_id: spec_id.clone(),
                },
                Some(spec_id.clone()),
                Some(spec_id.clone()), // increment revision count
            ));
        }

        // No review yet -> schedule review
        return Ok((
            ImplSubState::ReviewSpec {
                spec_id: spec_id.clone(),
            },
            Some(spec_id.clone()),
            None,
        ));
    }

    Ok((ImplSubState::AdvanceToMerge, None, None))
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/common_change_impl.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "resolve_next_impl"
      - "determine_sub_state"
    description: "Implementation sub-state resolver and pure transition logic."
```
