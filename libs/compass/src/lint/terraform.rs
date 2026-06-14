//! Terraform/HCL lint checker (tree-sitter-hcl AST)

use super::terraform_rules;
use crate::checker::LintConfig;
use crate::diagnostic::{Diagnostic, DiagnosticCategory, DiagnosticSeverity, Range};
use crate::syntax::{Language, ParsedFile};
use std::collections::HashSet;

/// Terraform/HCL checker
pub struct TerraformChecker;

impl TerraformChecker {
    pub fn new() -> Self {
        Self
    }

    /// TF001: Syntax errors from tree-sitter parse
    fn check_syntax_errors(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for err in file.collect_errors() {
            diagnostics.push(Diagnostic::error(
                Range::new(
                    crate::diagnostic::Position::new(
                        err.start_position.0.saturating_sub(1) as u32,
                        err.start_position.1.saturating_sub(1) as u32,
                    ),
                    crate::diagnostic::Position::new(
                        err.end_position.0.saturating_sub(1) as u32,
                        err.end_position.1.saturating_sub(1) as u32,
                    ),
                ),
                "TF001",
                DiagnosticCategory::Syntax,
                "HCL syntax error",
            ));
        }
        diagnostics
    }

    /// TF004: Hardcoded secrets in attributes
    fn check_hardcoded_secrets(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        const SECRET_KEYWORDS: &[&str] = &[
            "password",
            "secret",
            "token",
            "api_key",
            "apikey",
            "private_key",
            "access_key",
            "credential",
        ];

        file.walk(|node, _depth| {
            if node.kind() == "attribute" {
                let name_node = node.child(0);
                let value_node = node.child(2);

                if let (Some(name_n), Some(val_n)) = (name_node, value_node) {
                    let attr_name = file.node_text(&name_n).to_lowercase();
                    let is_secret = SECRET_KEYWORDS.iter().any(|kw| attr_name.contains(kw));

                    if is_secret {
                        let val_kind = val_n.kind();
                        if val_kind == "string_lit"
                            || val_kind == "template_expr"
                            || val_kind == "literal_value"
                        {
                            let val_text = file.node_text(&val_n);
                            if val_text.len() > 2
                                && !val_text.contains("var.")
                                && !val_text.contains("data.")
                                && !val_text.contains("local.")
                            {
                                diagnostics.push(Diagnostic::new(
                                    Range::from_node(node),
                                    DiagnosticSeverity::Error,
                                    "TF004",
                                    DiagnosticCategory::Security,
                                    format!(
                                        "Hardcoded secret in attribute '{}' — use a variable or secrets manager",
                                        attr_name,
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            true
        });

        diagnostics
    }

    /// TF005: Missing description on variable blocks
    fn check_variable_description(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() != "block" {
                return true;
            }
            if self.get_block_type(node, file).as_deref() != Some("variable") {
                return true;
            }

            if !self.block_has_attribute(node, file, "description") {
                let var_name = self
                    .get_block_label(node, file)
                    .unwrap_or_else(|| "<unknown>".to_string());
                diagnostics.push(Diagnostic::warning(
                    Range::from_node(node),
                    "TF005",
                    DiagnosticCategory::Style,
                    format!(
                        "Variable '{}' is missing a 'description' attribute",
                        var_name
                    ),
                ));
            }

            true
        });

        diagnostics
    }

    /// TF006: Missing terraform.required_version
    fn check_required_version(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut found_required_version = false;

        file.walk(|node, _depth| {
            if node.kind() == "block" {
                if self.get_block_type(node, file).as_deref() == Some("terraform") {
                    if self.block_has_attribute(node, file, "required_version") {
                        found_required_version = true;
                        return false;
                    }
                }
            }
            true
        });

        let has_tf_content = self.source_has_block_type(file, "terraform")
            || self.source_has_block_type(file, "resource")
            || self.source_has_block_type(file, "provider");

        if has_tf_content && !found_required_version {
            vec![Diagnostic::warning(
                Range::new(
                    crate::diagnostic::Position::new(0, 0),
                    crate::diagnostic::Position::new(0, 1),
                ),
                "TF006",
                DiagnosticCategory::Logic,
                "Missing 'required_version' in terraform block — pin the Terraform version",
            )]
        } else {
            Vec::new()
        }
    }

    /// TF008: Unused variables
    fn check_unused_variables(&self, file: &ParsedFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut declared_vars: Vec<(String, Range)> = Vec::new();

        file.walk(|node, _depth| {
            if node.kind() == "block" {
                if self.get_block_type(node, file).as_deref() == Some("variable") {
                    if let Some(name) = self.get_block_label(node, file) {
                        declared_vars.push((name, Range::from_node(node)));
                    }
                }
            }
            true
        });

        if declared_vars.is_empty() {
            return diagnostics;
        }

        let referenced: HashSet<String> = declared_vars
            .iter()
            .filter(|(name, _)| {
                let pattern = format!("var.{}", name);
                file.source.contains(&pattern)
            })
            .map(|(name, _)| name.clone())
            .collect();

        for (name, range) in &declared_vars {
            if !referenced.contains(name) {
                diagnostics.push(Diagnostic::new(
                    range.clone(),
                    DiagnosticSeverity::Hint,
                    "TF008",
                    DiagnosticCategory::Names,
                    format!(
                        "Variable '{}' is declared but never referenced as var.{}",
                        name, name
                    ),
                ));
            }
        }

        diagnostics
    }

    // ===== Helpers (pub(super) for terraform_rules) =====

    /// Get the block type identifier (first identifier child of a block node)
    pub(super) fn get_block_type<'a>(
        &self,
        node: &tree_sitter::Node<'a>,
        file: &'a ParsedFile,
    ) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(file.node_text(&child).to_string());
            }
        }
        None
    }

    /// Get the block label (string_lit after the block type identifier)
    pub(super) fn get_block_label(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Option<String> {
        let mut cursor = node.walk();
        let mut found_type = false;
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" && !found_type {
                found_type = true;
                continue;
            }
            if found_type && (child.kind() == "string_lit" || child.kind() == "identifier") {
                let text = file.node_text(&child);
                return Some(text.trim_matches('"').trim_matches('\'').to_string());
            }
        }
        None
    }

    /// Get the second label of a block (resource name in `resource "type" "name"`)
    pub(super) fn get_second_block_label(
        &self,
        node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
    ) -> Option<String> {
        let mut cursor = node.walk();
        let mut found_type = false;
        let mut found_first_label = false;
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" && !found_type {
                found_type = true;
                continue;
            }
            if found_type && (child.kind() == "string_lit" || child.kind() == "identifier") {
                if !found_first_label {
                    found_first_label = true;
                    continue;
                }
                let text = file.node_text(&child);
                return Some(text.trim_matches('"').trim_matches('\'').to_string());
            }
        }
        None
    }

    /// Check if a block contains a specific attribute by name
    pub(super) fn block_has_attribute(
        &self,
        block_node: &tree_sitter::Node<'_>,
        file: &ParsedFile,
        attr_name: &str,
    ) -> bool {
        let mut found = false;
        let mut cursor = block_node.walk();
        for child in block_node.children(&mut cursor) {
            if child.kind() == "body" {
                let mut body_cursor = child.walk();
                for body_child in child.children(&mut body_cursor) {
                    if body_child.kind() == "attribute" {
                        if let Some(name_node) = body_child.child(0) {
                            if file.node_text(&name_node) == attr_name {
                                found = true;
                            }
                        }
                    }
                }
            }
        }
        found
    }

    /// Check if source contains a block of the given type
    pub(super) fn source_has_block_type(&self, file: &ParsedFile, block_type: &str) -> bool {
        let mut found = false;
        file.walk(|node, _depth| {
            if node.kind() == "block" {
                if self.get_block_type(node, file).as_deref() == Some(block_type) {
                    found = true;
                    return false;
                }
            }
            true
        });
        found
    }
}

impl Default for TerraformChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl super::Checker for TerraformChecker {
    fn language(&self) -> Language {
        Language::Hcl
    }

    fn check(&self, file: &ParsedFile, _config: &LintConfig) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(self.check_syntax_errors(file));
        diagnostics.extend(self.check_hardcoded_secrets(file));
        diagnostics.extend(self.check_variable_description(file));
        diagnostics.extend(self.check_required_version(file));
        diagnostics.extend(self.check_unused_variables(file));
        // Delegated rules (terraform_rules module)
        diagnostics.extend(terraform_rules::check_deprecated_attributes(self, file));
        diagnostics.extend(terraform_rules::check_empty_resource(self, file));
        diagnostics.extend(terraform_rules::check_required_providers(self, file));
        diagnostics.extend(terraform_rules::check_missing_tags(self, file));
        diagnostics.extend(terraform_rules::check_s3_encryption(self, file));

        diagnostics
    }

    fn available_rules(&self) -> Vec<&'static str> {
        vec![
            "TF001", // Syntax errors
            "TF004", // Hardcoded secrets
            "TF005", // Missing variable description
            "TF006", // Missing required_version
            "TF008", // Unused variables
            "TF002", // Deprecated resource attributes
            "TF003", // Missing required attributes
            "TF007", // Missing required_providers
            "TF009", // Missing tags on resources
            "TF010", // S3 bucket without encryption
        ]
    }
}
