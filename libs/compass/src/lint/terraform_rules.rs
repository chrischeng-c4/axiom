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
