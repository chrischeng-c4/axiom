---
id: sdd-tools-validate-proposal-summary-behavior
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd validate proposal summary behavior

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
impl ValidationSummary {
    /// Check if valid (no blocking errors)
    pub fn is_valid(&self) -> bool {
        self.high_count == 0
    }

    /// Check if valid with strict mode (no errors at all)
    pub fn is_valid_strict(&self) -> bool {
        self.high_count == 0 && self.medium_count == 0 && self.low_count == 0
    }

    pub fn has_warnings(&self) -> bool {
        self.medium_count > 0 || self.low_count > 0
    }

    /// Convert to JSON output format
    pub fn to_json_output(&self, strict: bool) -> ValidationJsonOutput {
        ValidationJsonOutput {
            valid: if strict {
                self.is_valid_strict()
            } else {
                self.is_valid()
            },
            counts: ValidationCounts {
                high: self.high_count,
                medium: self.medium_count,
                low: self.low_count,
            },
            errors: self
                .validation_errors
                .iter()
                .map(JsonValidationError::from)
                .collect(),
            stale_files: self.stale_files.clone(),
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
      - "<handwrite-tracker:standardize-gap-sdd-validate-proposal-summary-impl>"
    description: "ValidationSummary helper methods and JSON output projection."
```
