---
id: sdd-tools-validate-proposal-error-accumulator
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd validate proposal error accumulator

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
impl ErrorAccumulator {
    fn new() -> Self {
        Self {
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            errors: Vec::new(),
            validation_errors: Vec::new(),
        }
    }

    fn process_result(
        &mut self,
        result: &ValidationResult,
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        for error in &result.errors {
            self.process_error(error, file_label, indent, options);
        }
    }

    fn process_error(
        &mut self,
        error: &ValidationError,
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        self.validation_errors.push(error.clone());
        match error.severity {
            Severity::High => {
                self.high_count += 1;
                self.errors
                    .push(format!("[{}] {}", file_label, error.message));
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
            Severity::Medium => {
                self.medium_count += 1;
                if options.strict {
                    self.errors
                        .push(format!("[{}] {}", file_label, error.message));
                }
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
            Severity::Low => {
                self.low_count += 1;
                if options.strict {
                    self.errors
                        .push(format!("[{}] {}", file_label, error.message));
                }
                if !options.json {
                    print_error(error, indent, options.verbose);
                }
            }
        }
    }

    fn process_errors_slice(
        &mut self,
        errors: &[ValidationError],
        file_label: &str,
        indent: &str,
        options: &ValidationOptions,
    ) {
        for error in errors {
            self.process_error(error, file_label, indent, options);
        }
    }
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
      - "<handwrite-tracker:standardize-gap-sdd-validate-proposal-error-accumulator>"
    description: "ErrorAccumulator behavior for counting and reporting validation errors."
```
