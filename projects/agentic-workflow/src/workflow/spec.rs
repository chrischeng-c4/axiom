// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/workflow/spec.md#source
// CODEGEN-BEGIN
//! Spec flow: CreateSpec + ReviewSpec + ReviseSpec.
//!
//! Handles phases: PostClarificationsCreated, ChangeSpecCreated, ChangeSpecReviewed, ChangeSpecRevised.

use super::helpers;
use crate::models::state::StatePhase;
use crate::state::StateManager;
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;

/// Handle the spec flow.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/workflow/spec.md#source
pub fn handle(change_dir: &Path, change_id: &str, project_path: &str) -> Result<Value> {
    let sm = StateManager::load(change_dir)?;
    let phase = sm.phase().clone();

    let proposal_path = change_dir.join("proposal.md");
    let specs_dir = change_dir.join("specs");
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

    let action = match &phase {
        StatePhase::ChangeInited => {
            if let Some(spec) = missing_specs.first() {
                Action::CreateSpec {
                    spec_id: spec.id.clone(),
                    depends: spec.depends.clone(),
                }
            } else if spec_count == 0 {
                eprintln!(
                    "[sdd] warn: no specs found and none parsed from proposal — \
                     falling through to tasks (change: {})",
                    change_id
                );
                return super::tasks::handle(change_dir, change_id, project_path);
            } else {
                return super::tasks::handle(change_dir, change_id, project_path);
            }
        }
        StatePhase::ChangeSpecCreated => {
            if let Some(spec_id) = &pending_review_spec {
                Action::ReviewSpec {
                    spec_id: spec_id.clone(),
                }
            } else if let Some(spec) = missing_specs.first() {
                Action::CreateSpec {
                    spec_id: spec.id.clone(),
                    depends: spec.depends.clone(),
                }
            } else {
                return super::tasks::handle(change_dir, change_id, project_path);
            }
        }
        StatePhase::ChangeSpecReviewed => {
            match last_review_verdict.as_deref() {
                Some("APPROVED") | Some("PASS") => {
                    if let Some(spec) = missing_specs.first() {
                        Action::CreateSpec {
                            spec_id: spec.id.clone(),
                            depends: spec.depends.clone(),
                        }
                    } else {
                        return super::tasks::handle(change_dir, change_id, project_path);
                    }
                }
                Some("REVIEWED") | Some("NEEDS_REVISION") => {
                    if let Some(spec_id) = &pending_review_spec {
                        let key = format!("change_spec:{}", spec_id);
                        if sm.revision_count(&key) >= 1 {
                            if let Some(spec) = missing_specs.first() {
                                Action::CreateSpec {
                                    spec_id: spec.id.clone(),
                                    depends: spec.depends.clone(),
                                }
                            } else {
                                return super::tasks::handle(change_dir, change_id, project_path);
                            }
                        } else {
                            Action::ReviseSpec {
                                spec_id: spec_id.clone(),
                            }
                        }
                    } else if let Some(spec) = missing_specs.first() {
                        Action::CreateSpec {
                            spec_id: spec.id.clone(),
                            depends: spec.depends.clone(),
                        }
                    } else {
                        return super::tasks::handle(change_dir, change_id, project_path);
                    }
                }
                Some("REJECTED") => {
                    // REJECTED verdict handled in ChangeSpecReviewed state
                    if let Some(spec_id) = &pending_review_spec {
                        let key = format!("change_spec:{}", spec_id);
                        if sm.revision_count(&key) >= 1 {
                            return Ok(helpers::mainthread_must_fix(
                                change_id,
                                &phase,
                                &format!("change_spec:{}", spec_id),
                                &format!("review_change_spec_{}", spec_id),
                            ));
                        }
                        Action::ReviseSpec {
                            spec_id: spec_id.clone(),
                        }
                    } else if let Some(spec) = missing_specs.first() {
                        Action::CreateSpec {
                            spec_id: spec.id.clone(),
                            depends: spec.depends.clone(),
                        }
                    } else {
                        return super::tasks::handle(change_dir, change_id, project_path);
                    }
                }
                _ => {
                    if let Some(spec_id) = &pending_review_spec {
                        Action::ReviewSpec {
                            spec_id: spec_id.clone(),
                        }
                    } else if let Some(spec) = missing_specs.first() {
                        Action::CreateSpec {
                            spec_id: spec.id.clone(),
                            depends: spec.depends.clone(),
                        }
                    } else {
                        return super::tasks::handle(change_dir, change_id, project_path);
                    }
                }
            }
        }
        StatePhase::ChangeSpecRevised => {
            if let Some(spec_id) = &pending_review_spec {
                Action::ReviewSpec {
                    spec_id: spec_id.clone(),
                }
            } else if let Some(spec) = missing_specs.first() {
                Action::CreateSpec {
                    spec_id: spec.id.clone(),
                    depends: spec.depends.clone(),
                }
            } else {
                return super::tasks::handle(change_dir, change_id, project_path);
            }
        }
        // ChangeSpecApproved removed — all-specs-approved routing is handled by
        // APPROVED verdict in ChangeSpecReviewed, which falls through to tasks::handle
        _ => {
            if let Some(spec) = missing_specs.first() {
                Action::CreateSpec {
                    spec_id: spec.id.clone(),
                    depends: spec.depends.clone(),
                }
            } else {
                return super::tasks::handle(change_dir, change_id, project_path);
            }
        }
    };

    let phase_str = helpers::phase_to_string(&phase);
    let mut base = json!({
        "change_id": change_id,
        "current_phase": phase_str,
        "has_proposal": has_proposal,
        "has_specs_dir": has_specs_dir,
        "spec_count": spec_count,
        "missing_specs_count": missing_specs.len(),
    });

    match action {
        Action::CreateSpec {
            ref spec_id,
            ref depends,
        } => {
            base["action"] = json!("create_change_spec");
            base["message"] = json!(format!("Create spec: {}", spec_id));
            base["next_phase"] = json!("change_spec_created");
            base["spec_id"] = json!(spec_id);
            base["depends"] = json!(depends);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Create Spec '{sid}' for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read context: try `score workflow read-artifact` with scope=\"proposal\" for spec_plan routing,\n   \
                    or scope=\"reference_context\" if no proposal exists\n\
                 2. Read context artifacts referenced by context_refs\n\
                 3. Read dependency specs (if this spec has `depends`)\n\
                 4. Read gap artifacts for gap_repairs assigned to this spec\n\
                 5. Determine spec_type and compositional tags\n\
                 6. Write requirements (each gap_repair → requirement), scenarios, diagrams\n\
                 7. Call `score artifact write-artifact` with artifact='spec', action='create'\n\n\
                 ## Compositional Tag System\n\n\
                 spec_type auto-expands to tags. Union of all tags determines required elements:\n\
                 - api → sequence diagram\n\
                 - http → OpenAPI 3.1\n\
                 - rpc → class diagram + OpenRPC 1.3\n\
                 - events → sequence diagram + AsyncAPI 2.6\n\
                 - data → ERD/class diagram + JSON Schema\n\
                 - logic → flowchart/state diagram\n\
                 - state → state diagram + Serverless Workflow 0.8\n\
                 - external → sequence diagram\n\n\
                 ## Payload Schema\n\n\
                 requirements (required, array, minItems: 1):\n  \
                   Each item: {{id: \"R1\", title: \"...\", description: \"...\"}}\n  \
                   Optional: priority (high/medium/low)\n\
                 scenarios (required, array, minItems: 1):\n  \
                   Each item: {{name: \"...\", given: \"...\", when: \"...\", then: \"...\"}}\n\
                 overview: required, minLength 50 chars\n\
                 diagrams (array): each {{type: \"flowchart|sequence|class|state|erd|mindmap\", title: \"...\", input: {{...}}}}\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"proposal\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"spec_context|knowledge_context|codebase_context\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id, sid = spec_id
            ));
        }
        Action::ReviewSpec { ref spec_id } => {
            base["action"] = json!("review_change_spec");
            base["message"] = json!(format!("Review spec: {}", spec_id));
            base["next_phase"] = json!("change_spec_reviewed or change_spec_approved");
            base["spec_id"] = json!(spec_id);
            base["mcp_tool"] = json!("sdd_write_artifact");
            base["prompt"] = json!(format!(
                "# Task: Review Spec '{sid}' for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Run automated validation:\n   \
                    `score workflow validate-spec-completeness {cid} {sid}`\n\
                 2. Read the spec: `score workflow read-artifact {cid}` with scope=\"{sid}\"\n\
                 3. Read the proposal for context routing\n\
                 4. Read dependency specs (if spec has `depends`)\n\
                 5. Evaluate against review checklist\n\
                 6. Determine verdict: APPROVED / REVIEWED / REJECTED\n\
                 7. Call `score artifact write-artifact` with artifact='spec', action='review'\n\n\
                 ## Review Checklist\n\n\
                 - Validation passes (validate-spec-completeness returns is_complete: true)\n\
                 - Correct spec_type for the nature of changes\n\
                 - Required diagrams present per tag requirements\n\
                 - Required API spec present per tag requirements\n\
                 - Requirements cover all relevant context_refs from proposal\n\
                 - Gap repairs assigned to this spec are addressed as requirements\n\
                 - At least one scenario per requirement\n\
                 - Overview is substantive (>= 50 chars)\n\
                 - Consistent with dependency specs\n\n\
                 ## Verdict Guidelines\n\n\
                 - APPROVED (use verdict='PASS'): Passes validation AND all checklist items\n\
                 - REVIEWED: Missing elements or unclear requirements\n\
                 - REJECTED: Fundamental design problems\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow validate-spec-completeness {cid} {sid}\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"{sid}\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"proposal\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id,
                sid = spec_id
            ));
        }
        Action::ReviseSpec { ref spec_id } => {
            base["action"] = json!("revise_change_spec");
            base["message"] = json!(format!("Revise spec based on review: {}", spec_id));
            base["next_phase"] = json!("change_spec_revised");
            base["spec_id"] = json!(spec_id);
            base["prompt"] = json!(format!(
                "# Task: Revise Spec '{sid}' for Change '{cid}'\n\n\
                 ## Instructions\n\n\
                 1. Read the spec and its review via `score workflow read-artifact`\n\
                 2. Address each issue from the review\n\
                 3. Call `score artifact write-artifact` with artifact='spec', action='revise'\n\n\
                 ## CLI Commands\n\n\
                 ```\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"{sid}\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"review_spec_{sid}\"}}'\n\
                 score workflow read-artifact {cid} '{{\"scope\":\"proposal\"}}'\n\
                 score artifact write-artifact {cid} <payload_path>\n\
                 ```",
                cid = change_id,
                sid = spec_id
            ));
        }
    }

    Ok(base)
}

#[derive(Debug)]
enum Action {
    CreateSpec {
        spec_id: String,
        depends: Vec<String>,
    },
    ReviewSpec {
        spec_id: String,
    },
    ReviseSpec {
        spec_id: String,
    },
}

// CODEGEN-END
