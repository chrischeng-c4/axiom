// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/commands/proposal.md#source
// CODEGEN-BEGIN
//! Proposal CLI commands

use agentic_workflow::services::proposal_service::{
    create_proposal, AffectedSpec, CreateProposalInput, ImpactData, SpecPlanInput,
};
use agentic_workflow::Result;
use clap::Subcommand;
use std::env;
use std::path::PathBuf;

/// Available subcommands for `score proposal`.
/// @spec projects/agentic-workflow/tech-design/surface/commands/proposal.md#schema
#[derive(Subcommand)]
pub enum ProposalCommands {
    /// Create a new proposal from JSON file.
    Create {
        change_id: String,
        #[arg(long)]
        json_file: PathBuf,
    },
    /// Add review to proposal from JSON file.
    Review {
        change_id: String,
        #[arg(long)]
        json_file: PathBuf,
    },
}
/// @spec projects/agentic-workflow/tech-design/surface/commands/proposal.md#source
pub fn run(cmd: ProposalCommands) -> Result<()> {
    let project_root = crate::find_project_root()?;

    match cmd {
        ProposalCommands::Create {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract fields from JSON
            let summary = json
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'summary' field"))?
                .to_string();

            let why = json
                .get("why")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'why' field"))?
                .to_string();

            let what_changes: Vec<String> = json
                .get("what_changes")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing 'what_changes' field"))?
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();

            let impact = json
                .get("impact")
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact' field"))?;

            let scope = impact
                .get("scope")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact.scope' field"))?
                .to_string();

            let affected_files = impact
                .get("affected_files")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'impact.affected_files' field"))?;

            let new_files = impact
                .get("new_files")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let affected_specs: Vec<AffectedSpec> = impact
                .get("affected_specs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            // Support both old format (string) and new format (object)
                            if let Some(s) = v.as_str() {
                                // Old format: just a string ID
                                Some(AffectedSpec {
                                    id: s.to_string(),
                                    depends: vec![],
                                })
                            } else if let Some(obj) = v.as_object() {
                                // New format: object with id and depends
                                let id = obj.get("id")?.as_str()?.to_string();
                                let depends = obj
                                    .get("depends")
                                    .and_then(|d| d.as_array())
                                    .map(|deps| {
                                        deps.iter()
                                            .filter_map(|d| d.as_str().map(|s| s.to_string()))
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                Some(AffectedSpec { id, depends })
                            } else {
                                None
                            }
                        })
                        .collect()
                })
                .unwrap_or_default();

            let affected_code: Vec<String> = impact
                .get("affected_code")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let breaking_changes = impact
                .get("breaking_changes")
                .and_then(|v| v.as_str())
                .map(String::from);

            // Convert affected_specs to spec_plan entries for v1 compat
            let spec_plan: Vec<SpecPlanInput> = affected_specs
                .iter()
                .map(|s| SpecPlanInput {
                    id: s.id.trim_end_matches(".md").to_string(),
                    title: s.id.trim_end_matches(".md").to_string(),
                    depends: s
                        .depends
                        .iter()
                        .map(|d| d.trim_end_matches(".md").to_string())
                        .collect(),
                    context_refs: None,
                    gap_repairs: Vec::new(),
                    affected_code: Vec::new(),
                })
                .collect();

            // Create input struct (v1 legacy format from CLI)
            let input = CreateProposalInput {
                change_id,
                version: 1,
                scope: scope.clone(),
                spec_plan,
                scope_areas: Vec::new(),
                iteration: 1,
                summary: Some(summary),
                why: Some(why),
                what_changes,
                impact: Some(ImpactData {
                    scope,
                    affected_files,
                    new_files,
                    affected_specs,
                    affected_code,
                    breaking_changes,
                }),
                agent: None,
                duration_secs: None,
            };

            // Create proposal
            let result = create_proposal(input, &project_root)?;
            println!("{}", result);
        }

        ProposalCommands::Review {
            change_id,
            json_file,
        } => {
            // Read and parse JSON file
            let json_content = std::fs::read_to_string(&json_file)?;
            let json: serde_json::Value = serde_json::from_str(&json_content)?;

            // Extract fields from JSON
            let summary = json
                .get("summary")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'summary' field"))?;

            let verdict = json
                .get("verdict")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'verdict' field"))?;

            let iteration = json
                .get("iteration")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'iteration' field"))?
                as u32;

            let next_steps = json.get("next_steps").and_then(|v| v.as_str());

            let issues = json
                .get("issues")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            // Validate verdict (accept both old and new names)
            if !["APPROVED", "REVIEWED", "REJECTED", "NEEDS_REVISION"].contains(&verdict) {
                anyhow::bail!("verdict must be 'APPROVED', 'REVIEWED', or 'REJECTED'");
            }

            // Normalize legacy verdict name
            let verdict = if verdict == "NEEDS_REVISION" {
                "REVIEWED"
            } else {
                verdict
            };

            // Get change directory
            let change_dir = project_root.join(".aw/changes").join(&change_id);
            if !change_dir.exists() {
                anyhow::bail!(
                    "Change '{}' not found. Run create proposal first.",
                    change_id
                );
            }

            // Build REVIEW_PROPOSAL.md content
            let mut content = String::new();
            content.push_str(&format!("# Proposal Review (Iteration {})\n\n", iteration));
            content.push_str(&format!("**Change ID**: {}\n\n", change_id));

            content.push_str("## Summary\n\n");
            content.push_str(summary);
            content.push_str("\n\n");

            content.push_str("## Issues\n\n");
            if issues.is_empty() {
                content.push_str("No issues found.\n\n");
            } else {
                for issue in &issues {
                    let severity = issue
                        .get("severity")
                        .and_then(|v| v.as_str())
                        .unwrap_or("MEDIUM");
                    let description = issue
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let recommendation = issue.get("recommendation").and_then(|v| v.as_str());

                    content.push_str(&format!("- **[{}]** {}\n", severity, description));
                    if let Some(rec) = recommendation {
                        content.push_str(&format!("  - *Recommendation*: {}\n", rec));
                    }
                }
                content.push('\n');
            }

            content.push_str("## Verdict\n\n");
            match verdict {
                "APPROVED" => {
                    content.push_str("- [x] APPROVED - Proposal is clear, complete, and ready for spec creation\n");
                    content.push_str("- [ ] REVIEWED - Has issues that need fixing\n");
                    content.push_str("- [ ] REJECTED - Fundamental problems with the proposal\n");
                }
                "REVIEWED" => {
                    content.push_str("- [ ] APPROVED - Proposal is clear, complete, and ready for spec creation\n");
                    content.push_str("- [x] REVIEWED - Has issues that need fixing\n");
                    content.push_str("- [ ] REJECTED - Fundamental problems with the proposal\n");
                }
                "REJECTED" => {
                    content.push_str("- [ ] APPROVED - Proposal is clear, complete, and ready for spec creation\n");
                    content.push_str("- [ ] REVIEWED - Has issues that need fixing\n");
                    content.push_str("- [x] REJECTED - Fundamental problems with the proposal\n");
                }
                _ => {}
            }
            content.push('\n');

            if let Some(steps) = next_steps {
                content.push_str(&format!("**Next Steps**: {}\n", steps));
            }

            // Write REVIEW_PROPOSAL.md
            let review_path = change_dir.join("REVIEW_PROPOSAL.md");
            std::fs::write(&review_path, &content)?;

            // Count issues by severity
            let high_count = issues
                .iter()
                .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("HIGH"))
                .count();
            let medium_count = issues
                .iter()
                .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("MEDIUM"))
                .count();
            let low_count = issues
                .iter()
                .filter(|i| i.get("severity").and_then(|v| v.as_str()) == Some("LOW"))
                .count();

            println!(
                "✓ REVIEW_PROPOSAL.md written: {}\n  Verdict: {}\n  Issues: {} high, {} medium, {} low",
                review_path.display(),
                verdict,
                high_count,
                medium_count,
                low_count
            );
        }
    }

    Ok(())
}

// CODEGEN-END
