---
id: libs-compass-src-lint-terraform-rules-rs
summary: Lossless rust-source-unit coverage for `libs/compass/src/lint/terraform_rules.rs`.
capability_refs:
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: multi-language-parser-and-checker-dispatch-contract
  gap: multi-language-parser-and-checker-dispatch-contract
  coverage: full
  rationale: "Multi-language parser and checker dispatch contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: codebase-check-and-lint-pipeline
  role: primary
  claim: agent-diagnostic-output-contract
  gap: agent-diagnostic-output-contract
  coverage: full
  rationale: "Agent diagnostic output contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: symbol-outline-and-propagated-type-query-contract
  gap: symbol-outline-and-propagated-type-query-contract
  coverage: full
  rationale: "Symbol outline and propagated type query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: semantic-search-and-graph-query-contract
  gap: semantic-search-and-graph-query-contract
  coverage: full
  rationale: "Semantic search and graph query contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: semantic-navigation-search-and-refactoring
  role: primary
  claim: structured-refactoring-contract
  gap: structured-refactoring-contract
  coverage: full
  rationale: "Structured refactoring contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: spec-parser-and-state-machine-validation-contract
  gap: spec-parser-and-state-machine-validation-contract
  coverage: full
  rationale: "Spec parser and state-machine validation contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: spec-parsing-and-code-generation
  role: primary
  claim: python-and-rust-generator-registry-contract
  gap: python-and-rust-generator-registry-contract
  coverage: full
  rationale: "Python and Rust generator registry contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: argus-daemon-protocol-and-request-handling-contract
  gap: argus-daemon-protocol-and-request-handling-contract
  coverage: full
  rationale: "Argus daemon protocol and request handling contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
- id: daemon-watch-and-incremental-analysis
  role: primary
  claim: watch-bridge-and-incremental-dirty-file-contract
  gap: watch-bridge-and-incremental-dirty-file-contract
  coverage: full
  rationale: "Watch bridge and incremental dirty-file contract is implemented by the existing Compass library surface and covered by the configured smoke gate."
fill_sections: [overview, source, changes]
---

# Standardized libs/compass/src/lint/terraform_rules.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `libs/compass/src/lint/terraform_rules.rs` captured during libs codegen standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `check_deprecated_attributes` | libs/compass/src/lint/terraform_rules.rs | function | pub | 8 | pub(super) fn check_deprecated_attributes( |
| `check_empty_resource` | libs/compass/src/lint/terraform_rules.rs | function | pub | 46 | pub(super) fn check_empty_resource( |
| `check_required_providers` | libs/compass/src/lint/terraform_rules.rs | function | pub | 96 | pub(super) fn check_required_providers( |
| `check_missing_tags` | libs/compass/src/lint/terraform_rules.rs | function | pub | 145 | pub(super) fn check_missing_tags(checker: &TerraformChecker, file: &ParsedFile) -> Vec<Diagnostic> { |
| `check_s3_encryption` | libs/compass/src/lint/terraform_rules.rs | function | pub | 177 | pub(super) fn check_s3_encryption( |


## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! Additional Terraform lint rules (TF002, TF003, TF007, TF009, TF010)

use super::terraform::TerraformChecker;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, Range};
use crate::syntax::ParsedFile;

/// TF002: Deprecated resource attributes (lifecycle.prevent_destroy syntax)
pub(super) fn check_deprecated_attributes(
    checker: &TerraformChecker,
    file: &ParsedFile,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() == "attribute" {
            if let Some(name_node) = node.child(0) {
                let attr_name = file.node_text(&name_node);
                if attr_name == "prevent_destroy" {
                    if let Some(parent) = node.parent() {
                        if parent.kind() == "body" {
                            if let Some(gp) = parent.parent() {
                                if gp.kind() == "block"
                                    && checker.get_block_type(&gp, file).as_deref()
                                        == Some("lifecycle")
                                {
                                    diagnostics.push(Diagnostic::warning(
                                        Range::from_node(node),
                                        "TF002",
                                        DiagnosticCategory::Style,
                                        "Deprecated: 'prevent_destroy' in lifecycle — use the lifecycle meta-argument directly on the resource",
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        true
    });

    diagnostics
}

/// TF003: Missing required attributes (resource blocks with no attributes)
pub(super) fn check_empty_resource(
    checker: &TerraformChecker,
    file: &ParsedFile,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() != "block" {
            return true;
        }
        if checker.get_block_type(node, file).as_deref() != Some("resource") {
            return true;
        }

        let mut has_attributes = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "body" {
                let mut body_cursor = child.walk();
                for body_child in child.children(&mut body_cursor) {
                    if body_child.kind() == "attribute" || body_child.kind() == "block" {
                        has_attributes = true;
                        break;
                    }
                }
            }
        }

        if !has_attributes {
            let label = checker
                .get_block_label(node, file)
                .unwrap_or_else(|| "<unknown>".to_string());
            diagnostics.push(Diagnostic::warning(
                Range::from_node(node),
                "TF003",
                DiagnosticCategory::Logic,
                format!(
                    "Resource '{}' has no attributes — this is likely incomplete",
                    label
                ),
            ));
        }

        true
    });

    diagnostics
}

/// TF007: Missing required_providers in terraform block
pub(super) fn check_required_providers(
    checker: &TerraformChecker,
    file: &ParsedFile,
) -> Vec<Diagnostic> {
    let mut has_terraform_block = false;
    let mut has_required_providers = false;

    file.walk(|node, _depth| {
        if node.kind() == "block"
            && checker.get_block_type(node, file).as_deref() == Some("terraform")
        {
            has_terraform_block = true;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "body" {
                    let mut bc = child.walk();
                    for bc_child in child.children(&mut bc) {
                        if bc_child.kind() == "block"
                            && checker.get_block_type(&bc_child, file).as_deref()
                                == Some("required_providers")
                        {
                            has_required_providers = true;
                        }
                    }
                }
            }
        }
        true
    });

    let has_providers = checker.source_has_block_type(file, "provider")
        || checker.source_has_block_type(file, "resource");

    if (has_terraform_block || has_providers) && !has_required_providers {
        vec![Diagnostic::warning(
            Range::new(
                crate::diagnostic::Position::new(0, 0),
                crate::diagnostic::Position::new(0, 1),
            ),
            "TF007",
            DiagnosticCategory::Logic,
            "Missing 'required_providers' in terraform block — pin provider versions for reproducibility",
        )]
    } else {
        Vec::new()
    }
}

/// TF009: Missing tags on resource blocks
pub(super) fn check_missing_tags(checker: &TerraformChecker, file: &ParsedFile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() != "block" {
            return true;
        }
        if checker.get_block_type(node, file).as_deref() != Some("resource") {
            return true;
        }

        if !checker.block_has_attribute(node, file, "tags") {
            let label = checker.get_block_label(node, file)
                .unwrap_or_else(|| "<unknown>".to_string());
            diagnostics.push(Diagnostic::warning(
                Range::from_node(node),
                "TF009",
                DiagnosticCategory::Style,
                format!(
                    "Resource '{}' has no 'tags' attribute — tagging aids cost tracking and organization",
                    label,
                ),
            ));
        }

        true
    });

    diagnostics
}

/// TF010: S3 bucket without encryption configuration
pub(super) fn check_s3_encryption(
    checker: &TerraformChecker,
    file: &ParsedFile,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    file.walk(|node, _depth| {
        if node.kind() != "block" {
            return true;
        }
        if checker.get_block_type(node, file).as_deref() != Some("resource") {
            return true;
        }

        let label = checker.get_block_label(node, file).unwrap_or_default();
        if label != "aws_s3_bucket" {
            return true;
        }

        let mut has_encryption = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "body" {
                let mut bc = child.walk();
                for bc_child in child.children(&mut bc) {
                    if bc_child.kind() == "block"
                        && checker.get_block_type(&bc_child, file).as_deref()
                            == Some("server_side_encryption_configuration")
                    {
                        has_encryption = true;
                    }
                }
            }
        }

        if !has_encryption {
            let resource_name = checker.get_second_block_label(node, file)
                .unwrap_or_else(|| "<unnamed>".to_string());
            diagnostics.push(Diagnostic::warning(
                Range::from_node(node),
                "TF010",
                DiagnosticCategory::Security,
                format!(
                    "S3 bucket '{}' missing 'server_side_encryption_configuration' — data at rest should be encrypted",
                    resource_name,
                ),
            ));
        }

        true
    });

    diagnostics
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/compass/src/lint/terraform_rules.rs"
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `libs/compass/src/lint/terraform_rules.rs` captured during libs codegen standardization.
```
