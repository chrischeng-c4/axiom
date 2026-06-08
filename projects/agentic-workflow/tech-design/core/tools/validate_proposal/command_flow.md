---
id: sdd-tools-validate-proposal-command-flow
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd validate proposal command flow

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/validate_proposal.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ValidationSummary` | projects/agentic-workflow/src/tools/validate_proposal.rs | struct | pub | 19 |  |
| `has_warnings` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 49 | has_warnings(&self) -> bool |
| `is_valid` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 40 | is_valid(&self) -> bool |
| `is_valid_strict` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 45 | is_valid_strict(&self) -> bool |
| `run` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 165 | run(change_id: &str, options: &ValidationOptions) -> Result<()> |
| `to_json_output` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 54 | to_json_output(&self, strict: bool) -> ValidationJsonOutput |
| `validate_proposal` | projects/agentic-workflow/src/tools/validate_proposal.rs | function | pub | 308 | validate_proposal(     change_id: &str,     project_root: &PathBuf,     options: &ValidationOptions, ) -> Result<ValidationSummary> |
## Source
<!-- type: source lang: rust -->

````rust
/// @spec projects/agentic-workflow/tech-design/core/tools/validate_proposal.md#changes
pub fn validate_proposal(
    change_id: &str,
    project_root: &PathBuf,
    options: &ValidationOptions,
) -> Result<ValidationSummary> {
    // Load config for validation rules
    let _config = SddConfig::load_validated(project_root)?;

    // Check if change exists
    let change_dir = super::workflow_common::resolve_change_dir(project_root, change_id);
    if !change_dir.exists() {
        anyhow::bail!(
            "Change '{}' not found. Run 'score run-change --change-id {}' first.",
            change_id,
            change_id
        );
    }

    // Create Change object and validate structure
    let change = Change::new(change_id, "");
    change.validate_structure(project_root)?;

    // Create validators with type-specific rules
    // PRD validator for proposal.md (lenient rules)
    let prd_rules = ValidationRules::for_prd();
    let prd_format_validator = SpecFormatValidator::new(prd_rules);

    // Task validator for tasks.md (lenient rules)
    let task_rules = ValidationRules::for_task();
    let task_format_validator = SpecFormatValidator::new(task_rules);

    // Spec validator for specs/*.md (strict rules - use central format rules)
    let spec_rules = ValidationRules::for_spec();
    let spec_format_validator = SpecFormatValidator::new(spec_rules.clone());
    let semantic_validator = SemanticValidator::new(spec_rules);

    // Schema validator for frontmatter validation
    let schemas_dir = project_root.join("cclab/schemas");
    let mut schema_validator = if schemas_dir.exists() {
        Some(SchemaValidator::new(&schemas_dir))
    } else {
        if !options.json {
            println!(
                "   {}",
                "⚠ Schema directory not found, skipping frontmatter validation".yellow()
            );
        }
        None
    };

    let mut acc = ErrorAccumulator::new();

    // Validate proposal.md
    let proposal_path = change_dir.join("proposal.md");
    if proposal_path.exists() {
        if !options.json {
            println!("   Checking proposal.md...");
        }

        // Check if file has frontmatter
        let content = std::fs::read_to_string(&proposal_path).unwrap_or_default();
        let has_fm = has_frontmatter(&content);

        // Schema validation (only if frontmatter exists and schemas available)
        let mut schema_valid = true;
        if has_fm {
            if let Some(ref mut validator) = schema_validator {
                let schema_result = validator.validate_file(&proposal_path);
                if !schema_result.is_valid() {
                    schema_valid = false;
                    if !options.json {
                        println!("      {} Frontmatter schema validation:", "📋".cyan());
                    }
                    acc.process_result(&schema_result, "proposal.md", "      ", options);
                }
            }
        } else if !options.json && options.verbose {
            println!(
                "      {} No frontmatter found (optional for now)",
                "ℹ".bright_black()
            );
        }

        // Format validation (PRD rules - lenient)
        let result = prd_format_validator.validate(&proposal_path);
        acc.process_result(&result, "proposal.md", "      ", options);

        if result.is_valid() && schema_valid && !options.json {
            println!("      {}", "✓ OK".green());
        }
    }

    // Validate tasks.md
    let tasks_path = change_dir.join("tasks.md");
    if tasks_path.exists() {
        if !options.json {
            println!("   Checking tasks.md...");
        }

        // Check if file has frontmatter
        let content = std::fs::read_to_string(&tasks_path).unwrap_or_default();
        let has_fm = has_frontmatter(&content);

        // Schema validation
        let mut schema_valid = true;
        if has_fm {
            if let Some(ref mut validator) = schema_validator {
                let schema_result = validator.validate_file(&tasks_path);
                if !schema_result.is_valid() {
                    schema_valid = false;
                    if !options.json {
                        println!("      {} Frontmatter schema validation:", "📋".cyan());
                    }
                    acc.process_result(&schema_result, "tasks.md", "      ", options);
                }
            }
        } else if !options.json && options.verbose {
            println!(
                "      {} No frontmatter found (optional for now)",
                "ℹ".bright_black()
            );
        }

        // Format validation (Task rules - lenient)
        let result = task_format_validator.validate(&tasks_path);
        acc.process_result(&result, "tasks.md", "      ", options);

        if result.is_valid() && schema_valid && !options.json {
            println!("      {}", "✓ OK".green());
        }
    }

    // Validate spec files in specs/
    let specs_dir = change_dir.join("specs");
    if specs_dir.exists() {
        if !options.json {
            println!("   Checking specs/...");
        }
        let spec_files: Vec<_> = walkdir::WalkDir::new(&specs_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
            // Skip template/skeleton files (files starting with underscore)
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map_or(true, |name| !name.starts_with('_'))
            })
            .collect();

        for entry in spec_files {
            let rel_path = entry
                .path()
                .strip_prefix(&change_dir)
                .unwrap_or(entry.path());
            let file_label = rel_path.display().to_string();
            if !options.json {
                println!("      {}...", rel_path.display());
            }

            // Check if file has frontmatter
            let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
            let has_fm = has_frontmatter(&content);

            // Schema validation
            let mut schema_valid = true;
            if has_fm {
                if let Some(ref mut validator) = schema_validator {
                    let schema_result = validator.validate_file(entry.path());
                    if !schema_result.is_valid() {
                        schema_valid = false;
                        acc.process_result(&schema_result, &file_label, "         ", options);
                    }
                }
            }

            // Format validation (Spec rules - strict)
            let format_result = spec_format_validator.validate(entry.path());
            // Semantic validation
            let semantic_result = semantic_validator.validate(entry.path());

            // Combine and process errors
            let combined_errors: Vec<_> = format_result
                .errors
                .iter()
                .chain(semantic_result.errors.iter())
                .cloned()
                .collect();
            acc.process_errors_slice(&combined_errors, &file_label, "         ", options);

            if format_result.is_valid()
                && semantic_result.is_valid()
                && schema_valid
                && !options.json
            {
                println!("         {}", "✓ OK".green());
            }
        }
    }

    // Cross-file consistency validation
    if !options.json {
        println!("   Checking cross-file consistency...");
    }
    let consistency_validator = ConsistencyValidator::new(&change_dir);

    // Validate task spec_refs
    if let Ok(spec_ref_errors) = consistency_validator.validate_task_spec_refs() {
        acc.process_errors_slice(&spec_ref_errors, "consistency", "      ", options);
        if spec_ref_errors.is_empty() && !options.json {
            println!("      {} Task spec_refs", "✓".green());
        }
    }

    // Validate proposal spec alignment
    if let Ok(alignment_errors) = consistency_validator.validate_proposal_spec_alignment() {
        acc.process_errors_slice(&alignment_errors, "consistency", "      ", options);
        if alignment_errors.is_empty() && !options.json {
            println!("      {} Proposal spec alignment", "✓".green());
        }
    }

    // Validate task dependencies
    if let Ok(dep_errors) = consistency_validator.validate_task_dependencies() {
        acc.process_errors_slice(&dep_errors, "consistency", "      ", options);
        if dep_errors.is_empty() && !options.json {
            println!("      {} Task dependencies", "✓".green());
        }
    }

    if !options.json {
        println!();
        println!("{}", "📊 Summary:".cyan());
        println!(
            "   {} HIGH, {} MEDIUM, {} LOW",
            if acc.high_count > 0 {
                acc.high_count.to_string().red().to_string()
            } else {
                "0".to_string()
            },
            if acc.medium_count > 0 {
                acc.medium_count.to_string().yellow().to_string()
            } else {
                "0".to_string()
            },
            acc.low_count
        );
    }

    // Record validation to STATE.yaml
    let mut state_manager = StateManager::load(&change_dir)?;

    // Check staleness first
    let staleness = state_manager.check_staleness()?;
    let stale_files = staleness.stale_files.clone();

    if staleness.has_stale() && !options.json {
        println!();
        println!("{}", "⚠️  Stale files detected:".yellow());
        for file in &staleness.stale_files {
            println!("   • {} (modified since last validation)", file);
        }
    }

    // Record validation result
    let validation_mode = if options.strict {
        ValidationMode::Strict
    } else {
        ValidationMode::Normal
    };

    let warnings: Vec<String> = Vec::new();
    state_manager.record_validation(
        "validate-proposal",
        validation_mode,
        acc.high_count == 0,
        acc.high_count as u32,
        acc.medium_count as u32,
        acc.low_count as u32,
        acc.errors.clone(),
        warnings,
    );

    // Update checksums if validation passed
    if acc.high_count == 0 {
        state_manager.update_all_checksums()?;
        state_manager.set_last_action("validate-proposal");
    }

    // Save state
    state_manager.save()?;

    if !options.json {
        println!();
        println!("   {} STATE.yaml updated", "💾".bright_black());
    }

    Ok(ValidationSummary {
        high_count: acc.high_count,
        medium_count: acc.medium_count,
        low_count: acc.low_count,
        errors: acc.errors,
        validation_errors: acc.validation_errors,
        stale_files,
    })
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/validate_proposal.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:standardize-gap-sdd-validate-proposal-command-flow>"
    description: "validate_proposal command flow and state update behavior."
```
