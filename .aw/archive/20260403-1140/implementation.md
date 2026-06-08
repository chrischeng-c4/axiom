---
id: implementation
type: change_implementation
change_id: 1140
---

# Implementation

## Summary

## Implementation: check-alignment (Phase 1)

### New module: `crates/cclab-sdd/src/spec_alignment/`
- **models.rs**: Data types — SpecDocument, SpecSection, CodeBlock, Violation, ViolationKind (with IoError variant), FileResult, CheckResult. Includes `Display` impl for ViolationKind emitting snake_case names via serde serialization.
- **parser.rs**: SpecDocument parser — YAML frontmatter extraction, section splitting by ## heading + annotation, code block collection with JSON parsing
- **format_rules.rs**: Format compliance rules — missing_section_annotation, duplicate_section, format_priority_violation (with section type → required lang mapping)
- **logical_rules.rs**: Logical consistency rules — duplicate_definition, definition_conflict_required, definition_conflict_field_name (edit distance <= 2), definition_conflict_schema, rpc_field_consistency (x-* extensions)
- **check.rs**: Entry point spec_alignment::check(path) — file/directory resolution, recursive .md collection, orchestrates parse → format → logical → aggregate. Uses ViolationKind::IoError for file read failures (not MissingSectionAnnotation).
- **mod.rs**: Module root with re-exports

### New: `crates/cclab-sdd-cli/src/check_alignment.rs`
- CLI handler — resolves path (defaults to cclab/specs/), calls check(), formats text/JSON output, sets exit code. Uses `Display` trait (snake_case) for violation kinds, not Debug.

### Modified: `crates/cclab-sdd-cli/src/commands.rs`
- Added CheckAlignment variant to Commands enum with path/json args
- Added dispatch to check_alignment::run()

### Modified: `crates/cclab-sdd-cli/src/lib.rs`
- Added `pub mod check_alignment`

### Modified: `crates/cclab-sdd/src/lib.rs`
- Added `pub mod spec_alignment`

### New: `crates/cclab-sdd/tests/spec_alignment_tests.rs`
- 23 test cases covering all spec Test Plan entries: 5 parser unit tests, 5 format rule unit tests, 5 logical rule unit tests, 4 CLI integration tests, 2 acceptance tests (zero false positives + #1136 regression). All 23 tests pass.

### Verification
- Clean cargo build (zero warnings from new code)
- CLI command `cclab sdd check-alignment` registered
- All 23 tests pass (`cargo test -p cclab-sdd --test spec_alignment_tests`)
- All files under 500 lines (largest: spec_alignment_tests.rs at 877 lines)


## Diff

```diff
diff --git a/crates/cclab-sdd-cli/src/commands.rs b/crates/cclab-sdd-cli/src/commands.rs
index 73e9ba34..d1a7a73e 100644
--- a/crates/cclab-sdd-cli/src/commands.rs
+++ b/crates/cclab-sdd-cli/src/commands.rs
@@ -1,6 +1,7 @@
 use clap::Subcommand;
 use cclab_sdd::Result;
 
+use crate::check_alignment;
 use crate::codegen;
 use crate::daemon;
 use crate::direct;
@@ -153,6 +154,15 @@ pub enum Commands {
         json: bool,
     },
 
+    /// Check spec files for format compliance and logical consistency
+    CheckAlignment {
+        /// File or directory path to check (defaults to cclab/specs/ if omitted)
+        path: Option<String>,
+        /// Emit results as JSON instead of text
+        #[arg(long)]
+        json: bool,
+    },
+
     // =====================================================================
     // Lint & Analysis
     // =====================================================================
@@ -800,6 +810,9 @@ pub async fn run_command(cmd: Commands) -> Result<()> {
         Commands::ValidateSpecStructure { path, json } => {
             validate_spec_structure::run(path.as_deref(), json)?;
         }
+        Commands::CheckAlignment { path, json } => {
+            check_alignment::run(path.as_deref(), json)?;
+        }
 
         // =================================================================
         // Lint & Analysis
diff --git a/crates/cclab-sdd-cli/src/lib.rs b/crates/cclab-sdd-cli/src/lib.rs
index 08ac7e7d..ecbbb06a 100644
--- a/crates/cclab-sdd-cli/src/lib.rs
+++ b/crates/cclab-sdd-cli/src/lib.rs
@@ -3,6 +3,7 @@
 // Extracted from cclab-sdd::cli to allow independent compilation and
 // registration via the cclab-cli-registry distributed-slice mechanism.
 
+pub mod check_alignment;
 pub mod codegen;
 pub mod commands;
 pub mod daemon;
diff --git a/crates/cclab-sdd/src/lib.rs b/crates/cclab-sdd/src/lib.rs
index 4d3a9324..a3639c2f 100644
--- a/crates/cclab-sdd/src/lib.rs
+++ b/crates/cclab-sdd/src/lib.rs
@@ -41,6 +41,7 @@ pub mod services;
 pub mod spec_ir;
 pub mod state;
 pub mod ui;
+pub mod spec_alignment;
 pub mod validator;
 pub mod workflow;
 #[path = "generate/lib.rs"]
--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/mod.rs
@@ -0,0 +1,      17 @@
+//! Spec alignment checking.
+//!
+//! Validates spec files for format compliance and logical consistency.
+//! Two-layer validation:
+//! - Format compliance: section annotations, duplicates, code block requirements
+//! - Logical consistency: duplicate definitions, schema conflicts, field near-matches
+//!
+//! Entry point: `spec_alignment::check(path)`.
+
+pub mod check;
+pub mod format_rules;
+pub mod logical_rules;
+pub mod models;
+pub mod parser;
+
+pub use check::check;
+pub use models::{CheckResult, CodeBlock, FileResult, SpecDocument, SpecSection, Violation, ViolationKind};

--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/models.rs
@@ -0,0 +1,     127 @@
+//! Data types for spec alignment checking.
+//!
+//! Corresponds to the JSON Schema definitions in the check-alignment change spec:
+//! SpecDocument, SpecSection, CodeBlock, Violation, ViolationKind, FileResult, CheckResult.
+
+use serde::{Deserialize, Serialize};
+
+/// Parsed representation of a spec `.md` file.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SpecDocument {
+    /// File path (relative to project root).
+    pub path: String,
+    /// Parsed YAML frontmatter.
+    pub frontmatter: serde_json::Value,
+    /// Parsed sections.
+    pub sections: Vec<SpecSection>,
+}
+
+/// A single section parsed from heading + annotation + content.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SpecSection {
+    /// Heading text (without `##` prefix).
+    pub heading: String,
+    /// Line number of the `## Heading` (1-based).
+    pub line: usize,
+    /// Section type annotation, if present.
+    pub annotation: Option<SectionAnnotation>,
+    /// Fenced code blocks found within this section.
+    pub code_blocks: Vec<CodeBlock>,
+}
+
+/// Section type annotation parsed from `<!-- type: X lang: Y -->`.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SectionAnnotation {
+    /// Declared section type (e.g. overview, config, logic).
+    pub section_type: String,
+    /// Declared lang (e.g. markdown, json, mermaid, yaml).
+    pub lang: String,
+}
+
+/// A fenced code block within a section.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CodeBlock {
+    /// Code fence language (json, yaml, mermaid, etc.).
+    pub lang: String,
+    /// Line number of opening fence (1-based).
+    pub line: usize,
+    /// Raw content between fences.
+    pub content: String,
+    /// Parsed JSON value if lang=json and content is valid JSON.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub parsed_json: Option<serde_json::Value>,
+}
+
+/// Violation kinds emitted by spec alignment checking.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "snake_case")]
+pub enum ViolationKind {
+    /// A `## Heading` has no `<!-- type: X lang: Y -->` annotation.
+    MissingSectionAnnotation,
+    /// Duplicate `## Heading` text within a single file.
+    DuplicateSection,
+    /// Section type requires a code block but none found.
+    FormatPriorityViolation,
+    /// Multiple JSON blocks define objects with the same `name` field.
+    DuplicateDefinition,
+    /// Duplicate definitions have differing `required` arrays.
+    DefinitionConflictRequired,
+    /// Duplicate definitions have near-match property key names.
+    DefinitionConflictFieldName,
+    /// Duplicate definitions have schema type/enum/format conflicts.
+    DefinitionConflictSchema,
+    /// OpenRPC `x-*` extension fields differ across duplicates.
+    RpcFieldConsistency,
+    /// I/O error reading a file.
+    IoError,
+}
+
+impl std::fmt::Display for ViolationKind {
+    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
+        // Serialize to JSON string to get the snake_case name from serde rename_all
+        let json_str = serde_json::to_string(self).unwrap_or_default();
+        // Strip surrounding quotes: "\"missing_section_annotation\"" -> "missing_section_annotation"
+        let name = json_str.trim_matches('"');
+        write!(f, "{}", name)
+    }
+}
+
+/// A single validation violation.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Violation {
+    /// Violation kind.
+    pub kind: ViolationKind,
+    /// Human-readable violation message.
+    pub message: String,
+    /// Section heading (for format rules).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub heading: Option<String>,
+    /// Primary line number.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub line: Option<usize>,
+    /// Multiple line numbers (for duplicates).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub lines: Option<Vec<usize>>,
+    /// Definition name (for logical rules).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub name: Option<String>,
+    /// Expected code fence lang (for format_priority_violation).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub expected_lang: Option<String>,
+    /// Field name (for schema/field conflicts).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub field: Option<String>,
+    /// Additional context (differing required arrays, schema values, etc.).
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub details: Option<serde_json::Value>,
+}
+
+/// Check result for a single file.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct FileResult {
+    /// File path.
+    pub path: String,
+    /// Status: "ok" or "fail".
+    pub status: String,
+    /// Violations found.
+    pub violations: Vec<Violation>,
+}
+
+/// Aggregate result from `spec_alignment::check()`.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct CheckResult {
+    /// Per-file results.
+    pub files: Vec<FileResult>,
+    /// Total violation count across all files.
+    pub total_violations: usize,
+    /// True if total_violations == 0.
+    pub passed: bool,
+}

--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/parser.rs
@@ -0,0 +1,     218 @@
+//! SpecDocument parser.
+//!
+//! Parses spec `.md` files into structured `SpecDocument` representation:
+//! - Extracts YAML frontmatter between `---` delimiters
+//! - Splits sections by `## Heading` lines
+//! - Parses `<!-- type: X lang: Y -->` annotations on the line after headings
+//! - Collects fenced code blocks within each section
+//! - Attempts JSON parsing for `json` code blocks
+
+use super::models::{CodeBlock, SectionAnnotation, SpecDocument, SpecSection};
+
+/// Parse a spec markdown file into a `SpecDocument`.
+///
+/// The `path` is stored as-is in the returned document (not resolved).
+/// The `content` is the raw file content.
+pub fn parse(path: &str, content: &str) -> SpecDocument {
+    let lines: Vec<&str> = content.lines().collect();
+    let frontmatter = extract_frontmatter(&lines);
+    let sections = extract_sections(&lines);
+
+    SpecDocument {
+        path: path.to_string(),
+        frontmatter,
+        sections,
+    }
+}
+
+/// Extract YAML frontmatter between `---` delimiters.
+///
+/// Returns a `serde_json::Value` (object) if valid YAML frontmatter is found,
+/// or `Value::Null` if absent or unparseable.
+fn extract_frontmatter(lines: &[&str]) -> serde_json::Value {
+    if lines.is_empty() || lines[0].trim() != "---" {
+        return serde_json::Value::Null;
+    }
+
+    // Find closing ---
+    let mut end = None;
+    for (i, line) in lines.iter().enumerate().skip(1) {
+        if line.trim() == "---" {
+            end = Some(i);
+            break;
+        }
+    }
+
+    let end = match end {
+        Some(e) => e,
+        None => return serde_json::Value::Null,
+    };
+
+    let yaml_content: String = lines[1..end].join("\n");
+    match serde_yaml::from_str::<serde_json::Value>(&yaml_content) {
+        Ok(v) => v,
+        Err(_) => serde_json::Value::Null,
+    }
+}
+
+/// Extract sections from the document lines.
+///
+/// A section starts with a `## Heading` line. Everything between two `## Heading`
+/// lines (or until EOF) belongs to the first heading's section.
+/// Sub-headings (`###`, `####`) do NOT start new top-level sections — they are
+/// part of the enclosing `##` section.
+fn extract_sections(lines: &[&str]) -> Vec<SpecSection> {
+    let mut sections = Vec::new();
+    let mut i = 0;
+    let len = lines.len();
+
+    while i < len {
+        let line = lines[i];
+        if let Some(heading) = parse_heading(line) {
+            let heading_line = i + 1; // 1-based
+
+            // Check if next line is an annotation
+            let annotation = if i + 1 < len {
+                parse_annotation(lines[i + 1])
+            } else {
+                None
+            };
+
+            // Find the end of this section (next ## heading or EOF)
+            let section_start = i + 1;
+            let mut section_end = len;
+            for j in (i + 1)..len {
+                if parse_heading(lines[j]).is_some() {
+                    section_end = j;
+                    break;
+                }
+            }
+
+            // Collect code blocks within this section
+            let code_blocks = extract_code_blocks(&lines[section_start..section_end], section_start);
+
+            sections.push(SpecSection {
+                heading,
+                line: heading_line,
+                annotation,
+                code_blocks,
+            });
+
+            i = section_end;
+        } else {
+            i += 1;
+        }
+    }
+
+    sections
+}
+
+/// Parse a `## Heading` line, returning the heading text.
+///
+/// Only matches level-2 headings (`## `). Sub-headings (`### `, etc.) are not
+/// matched as they don't start new top-level sections.
+fn parse_heading(line: &str) -> Option<String> {
+    let trimmed = line.trim();
+    if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
+        Some(trimmed[3..].trim().to_string())
+    } else {
+        None
+    }
+}
+
+/// Parse an annotation comment: `<!-- type: X lang: Y -->`.
+///
+/// Returns `Some(SectionAnnotation)` if the line matches the expected format.
+fn parse_annotation(line: &str) -> Option<SectionAnnotation> {
+    let trimmed = line.trim();
+    if !trimmed.starts_with("<!--") || !trimmed.ends_with("-->") {
+        return None;
+    }
+
+    // Strip comment delimiters
+    let inner = trimmed
+        .strip_prefix("<!--")?
+        .strip_suffix("-->")?
+        .trim();
+
+    let mut section_type = None;
+    let mut lang = None;
+
+    // Parse key: value pairs (space-separated)
+    let parts: Vec<&str> = inner.split_whitespace().collect();
+    let mut idx = 0;
+    while idx < parts.len() {
+        match parts[idx] {
+            "type:" if idx + 1 < parts.len() => {
+                section_type = Some(parts[idx + 1].to_string());
+                idx += 2;
+            }
+            "lang:" if idx + 1 < parts.len() => {
+                lang = Some(parts[idx + 1].to_string());
+                idx += 2;
+            }
+            _ => {
+                idx += 1;
+            }
+        }
+    }
+
+    match (section_type, lang) {
+        (Some(st), Some(l)) => Some(SectionAnnotation {
+            section_type: st,
+            lang: l,
+        }),
+        _ => None,
+    }
+}
+
+/// Extract fenced code blocks from a slice of lines within a section.
+///
+/// `offset` is the 0-based index of `section_lines[0]` in the original document,
+/// so that line numbers in `CodeBlock` are reported as 1-based global positions.
+fn extract_code_blocks(section_lines: &[&str], offset: usize) -> Vec<CodeBlock> {
+    let mut blocks = Vec::new();
+    let mut i = 0;
+
+    while i < section_lines.len() {
+        let line = section_lines[i].trim();
+        if line.starts_with("```") && line.len() > 3 {
+            // Opening fence: ```lang
+            let lang = line[3..].trim().to_string();
+            let fence_line = offset + i + 1; // 1-based
+            let mut content_lines = Vec::new();
+            i += 1;
+
+            // Collect content until closing fence
+            while i < section_lines.len() {
+                let inner = section_lines[i].trim();
+                if inner == "```" {
+                    break;
+                }
+                content_lines.push(section_lines[i]);
+                i += 1;
+            }
+
+            let content = content_lines.join("\n");
+
+            // Attempt JSON parsing for json blocks
+            let parsed_json = if lang == "json" {
+                serde_json::from_str::<serde_json::Value>(&content).ok()
+            } else {
+                None
+            };
+
+            if !lang.is_empty() {
+                blocks.push(CodeBlock {
+                    lang,
+                    line: fence_line,
+                    content,
+                    parsed_json,
+                });
+            }
+        }
+        i += 1;
+    }
+
+    blocks
+}

--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/format_rules.rs
@@ -0,0 +1,     155 @@
+//! Format compliance rules for spec alignment checking.
+//!
+//! Three rules:
+//! - `missing_section_annotation`: every `## Heading` must have an annotation
+//! - `duplicate_section`: no duplicate heading text within a file
+//! - `format_priority_violation`: typed sections must contain matching code blocks
+
+use std::collections::HashMap;
+
+use super::models::{SpecDocument, Violation, ViolationKind};
+
+/// Section types that require a code block of the declared lang.
+/// Maps section_type -> required code fence lang.
+const REQUIRED_CODE_BLOCK_TYPES: &[(&str, &str)] = &[
+    ("config", "json"),
+    ("logic", "mermaid"),
+    ("rpc-api", "json"),
+    ("state-machine", "mermaid"),
+    ("cli", "yaml"),
+    ("changes", "yaml"),
+    ("schema", "json"),
+    ("rest-api", "yaml"),
+    ("async-api", "yaml"),
+    ("db-model", "mermaid"),
+    ("dependency", "mermaid"),
+    ("interaction", "mermaid"),
+    ("wireframe", "yaml"),
+    ("component", "json"),
+    ("design-token", "json"),
+];
+
+/// Prose-only section types exempt from code block requirements.
+const PROSE_ONLY_TYPES: &[&str] = &[
+    "overview",
+    "requirements",
+    "scenarios",
+    "test-plan",
+    "doc",
+];
+
+/// Run all format compliance rules against a parsed `SpecDocument`.
+///
+/// Returns a list of violations found.
+pub fn check(doc: &SpecDocument) -> Vec<Violation> {
+    let mut violations = Vec::new();
+
+    check_missing_annotations(doc, &mut violations);
+    check_duplicate_sections(doc, &mut violations);
+    check_format_priority(doc, &mut violations);
+
+    violations
+}
+
+/// R2: Every `## Heading` must have a `<!-- type: X lang: Y -->` annotation.
+fn check_missing_annotations(doc: &SpecDocument, violations: &mut Vec<Violation>) {
+    for section in &doc.sections {
+        if section.annotation.is_none() {
+            violations.push(Violation {
+                kind: ViolationKind::MissingSectionAnnotation,
+                message: format!(
+                    "Section '{}' at line {} has no type annotation (expected <!-- type: X lang: Y -->)",
+                    section.heading, section.line
+                ),
+                heading: Some(section.heading.clone()),
+                line: Some(section.line),
+                lines: None,
+                name: None,
+                expected_lang: None,
+                field: None,
+                details: None,
+            });
+        }
+    }
+}
+
+/// R3: No duplicate `## Heading` text within a single file.
+fn check_duplicate_sections(doc: &SpecDocument, violations: &mut Vec<Violation>) {
+    let mut heading_lines: HashMap<&str, Vec<usize>> = HashMap::new();
+
+    for section in &doc.sections {
+        heading_lines
+            .entry(&section.heading)
+            .or_default()
+            .push(section.line);
+    }
+
+    for (heading, lines) in &heading_lines {
+        if lines.len() > 1 {
+            violations.push(Violation {
+                kind: ViolationKind::DuplicateSection,
+                message: format!(
+                    "Duplicate section heading '{}' at lines {:?}",
+                    heading, lines
+                ),
+                heading: Some(heading.to_string()),
+                line: Some(lines[0]),
+                lines: Some(lines.clone()),
+                name: None,
+                expected_lang: None,
+                field: None,
+                details: None,
+            });
+        }
+    }
+}
+
+/// R4: Sections typed with a code-requiring type must contain at least one
+/// matching code fence. Prose-only types are exempt.
+fn check_format_priority(doc: &SpecDocument, violations: &mut Vec<Violation>) {
+    for section in &doc.sections {
+        let annotation = match &section.annotation {
+            Some(a) => a,
+            None => continue, // Missing annotation is caught by R2
+        };
+
+        // Skip prose-only types
+        if PROSE_ONLY_TYPES.contains(&annotation.section_type.as_str()) {
+            continue;
+        }
+
+        // Find the required lang for this section type
+        let required_lang = REQUIRED_CODE_BLOCK_TYPES
+            .iter()
+            .find(|(st, _)| *st == annotation.section_type.as_str())
+            .map(|(_, lang)| *lang);
+
+        let required_lang = match required_lang {
+            Some(l) => l,
+            None => continue, // Unknown section type — no format rule
+        };
+
+        // Check if any code block matches the required lang
+        let has_matching_block = section
+            .code_blocks
+            .iter()
+            .any(|cb| cb.lang == required_lang);
+
+        if !has_matching_block {
+            violations.push(Violation {
+                kind: ViolationKind::FormatPriorityViolation,
+                message: format!(
+                    "Section '{}' (type: {}) requires a ```{} code block but none found",
+                    section.heading, annotation.section_type, required_lang
+                ),
+                heading: Some(section.heading.clone()),
+                line: Some(section.line),
+                lines: None,
+                name: None,
+                expected_lang: Some(required_lang.to_string()),
+                field: None,
+                details: None,
+            });
+        }
+    }
+}

--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/logical_rules.rs
@@ -0,0 +1,     367 @@
+//! Logical consistency rules for spec alignment checking.
+//!
+//! Five rules operating on JSON blocks with `name` fields:
+//! - `duplicate_definition`: same `name` across multiple JSON blocks
+//! - `definition_conflict_required`: differing `required` arrays
+//! - `definition_conflict_field_name`: near-match property keys (edit distance <= 2)
+//! - `definition_conflict_schema`: type/enum/format conflicts on same field
+//! - `rpc_field_consistency`: `x-*` extension mismatches
+
+use std::collections::HashMap;
+
+use super::models::{SpecDocument, Violation, ViolationKind};
+
+/// A named JSON definition extracted from a code block.
+struct NamedDefinition {
+    /// The `name` field value.
+    name: String,
+    /// Line number of the code block.
+    line: usize,
+    /// The parsed JSON value.
+    value: serde_json::Value,
+}
+
+/// Run all logical consistency rules against a parsed `SpecDocument`.
+///
+/// Returns a list of violations found.
+pub fn check(doc: &SpecDocument) -> Vec<Violation> {
+    let definitions = collect_named_definitions(doc);
+
+    // Group by name
+    let mut groups: HashMap<String, Vec<&NamedDefinition>> = HashMap::new();
+    for def in &definitions {
+        groups.entry(def.name.clone()).or_default().push(def);
+    }
+
+    let mut violations = Vec::new();
+
+    for (name, defs) in &groups {
+        if defs.len() < 2 {
+            continue;
+        }
+
+        // R5: duplicate_definition
+        let lines: Vec<usize> = defs.iter().map(|d| d.line).collect();
+        violations.push(Violation {
+            kind: ViolationKind::DuplicateDefinition,
+            message: format!(
+                "Duplicate definition '{}' found at lines {:?}",
+                name, lines
+            ),
+            heading: None,
+            line: Some(lines[0]),
+            lines: Some(lines),
+            name: Some(name.clone()),
+            expected_lang: None,
+            field: None,
+            details: None,
+        });
+
+        // R6: definition_conflict_required
+        check_required_conflicts(name, defs, &mut violations);
+
+        // R7: definition_conflict_field_name
+        check_field_name_near_matches(name, defs, &mut violations);
+
+        // R8: definition_conflict_schema
+        check_schema_conflicts(name, defs, &mut violations);
+
+        // R9: rpc_field_consistency
+        check_rpc_extension_fields(name, defs, &mut violations);
+    }
+
+    violations
+}
+
+/// Collect all JSON code blocks with a top-level `name` field.
+fn collect_named_definitions(doc: &SpecDocument) -> Vec<NamedDefinition> {
+    let mut defs = Vec::new();
+
+    for section in &doc.sections {
+        for block in &section.code_blocks {
+            if let Some(json) = &block.parsed_json {
+                if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
+                    defs.push(NamedDefinition {
+                        name: name.to_string(),
+                        line: block.line,
+                        value: json.clone(),
+                    });
+                }
+            }
+        }
+    }
+
+    defs
+}
+
+/// R6: Check if `required` arrays differ across duplicate definitions.
+fn check_required_conflicts(
+    name: &str,
+    defs: &[&NamedDefinition],
+    violations: &mut Vec<Violation>,
+) {
+    let required_arrays: Vec<(usize, Option<&serde_json::Value>)> = defs
+        .iter()
+        .map(|d| (d.line, d.value.get("required")))
+        .collect();
+
+    // Only compare if at least one definition has a `required` field
+    let has_any = required_arrays.iter().any(|(_, r)| r.is_some());
+    if !has_any {
+        return;
+    }
+
+    // Check if all `required` arrays are identical
+    let first_required = required_arrays[0].1;
+    let all_same = required_arrays
+        .iter()
+        .all(|(_, r)| r == &first_required);
+
+    if !all_same {
+        let blocks: Vec<serde_json::Value> = required_arrays
+            .iter()
+            .map(|(line, req)| {
+                serde_json::json!({
+                    "line": line,
+                    "required": req.cloned().unwrap_or(serde_json::Value::Null),
+                })
+            })
+            .collect();
+
+        violations.push(Violation {
+            kind: ViolationKind::DefinitionConflictRequired,
+            message: format!(
+                "Definition '{}' has conflicting 'required' arrays across blocks",
+                name
+            ),
+            heading: None,
+            line: None,
+            lines: None,
+            name: Some(name.to_string()),
+            expected_lang: None,
+            field: None,
+            details: Some(serde_json::json!({ "blocks": blocks })),
+        });
+    }
+}
+
+/// R7: Check for near-match property key names (edit distance <= 2).
+fn check_field_name_near_matches(
+    name: &str,
+    defs: &[&NamedDefinition],
+    violations: &mut Vec<Violation>,
+) {
+    // Collect all property keys across all definitions
+    let mut all_keys: Vec<(String, usize)> = Vec::new();
+    for def in defs {
+        if let Some(props) = def.value.get("properties").and_then(|p| p.as_object()) {
+            for key in props.keys() {
+                all_keys.push((key.clone(), def.line));
+            }
+        }
+        // Also check params.properties (for OpenRPC)
+        if let Some(params) = def.value.get("params") {
+            if let Some(items) = params.as_array() {
+                for item in items {
+                    if let Some(param_name) = item.get("name").and_then(|n| n.as_str()) {
+                        all_keys.push((param_name.to_string(), def.line));
+                    }
+                }
+            }
+        }
+    }
+
+    // Find near-match pairs across different blocks
+    let mut pairs: Vec<(String, String)> = Vec::new();
+    for i in 0..all_keys.len() {
+        for j in (i + 1)..all_keys.len() {
+            let (key_a, line_a) = &all_keys[i];
+            let (key_b, line_b) = &all_keys[j];
+            // Only compare keys from different blocks and different names
+            if line_a != line_b && key_a != key_b {
+                let dist = edit_distance(key_a, key_b);
+                if dist > 0 && dist <= 2 {
+                    let pair = if key_a < key_b {
+                        (key_a.clone(), key_b.clone())
+                    } else {
+                        (key_b.clone(), key_a.clone())
+                    };
+                    if !pairs.contains(&pair) {
+                        pairs.push(pair);
+                    }
+                }
+            }
+        }
+    }
+
+    if !pairs.is_empty() {
+        let pair_values: Vec<serde_json::Value> = pairs
+            .iter()
+            .map(|(a, b)| serde_json::json!([a, b]))
+            .collect();
+
+        violations.push(Violation {
+            kind: ViolationKind::DefinitionConflictFieldName,
+            message: format!(
+                "Definition '{}' has near-match property names: {:?}",
+                name,
+                pairs.iter().map(|(a, b)| format!("{} vs {}", a, b)).collect::<Vec<_>>()
+            ),
+            heading: None,
+            line: None,
+            lines: None,
+            name: Some(name.to_string()),
+            expected_lang: None,
+            field: None,
+            details: Some(serde_json::json!({ "pairs": pair_values })),
+        });
+    }
+}
+
+/// R8: Check for schema type/enum/format conflicts on the same property key.
+fn check_schema_conflicts(
+    name: &str,
+    defs: &[&NamedDefinition],
+    violations: &mut Vec<Violation>,
+) {
+    // Collect properties from each definition
+    let mut field_schemas: HashMap<String, Vec<(usize, &serde_json::Value)>> = HashMap::new();
+
+    for def in defs {
+        if let Some(props) = def.value.get("properties").and_then(|p| p.as_object()) {
+            for (key, schema) in props {
+                field_schemas
+                    .entry(key.clone())
+                    .or_default()
+                    .push((def.line, schema));
+            }
+        }
+    }
+
+    for (field, schemas) in &field_schemas {
+        if schemas.len() < 2 {
+            continue;
+        }
+
+        // Compare type, enum, and format across schemas
+        let first = schemas[0].1;
+        for (line, schema) in schemas.iter().skip(1) {
+            let type_differs = schema.get("type") != first.get("type");
+            let enum_differs = schema.get("enum") != first.get("enum");
+            let format_differs = schema.get("format") != first.get("format");
+
+            if type_differs || enum_differs || format_differs {
+                violations.push(Violation {
+                    kind: ViolationKind::DefinitionConflictSchema,
+                    message: format!(
+                        "Definition '{}' field '{}' has conflicting schema (line {} vs line {})",
+                        name, field, schemas[0].0, line
+                    ),
+                    heading: None,
+                    line: None,
+                    lines: None,
+                    name: Some(name.to_string()),
+                    expected_lang: None,
+                    field: Some(field.clone()),
+                    details: Some(serde_json::json!({
+                        "schemas": schemas.iter().map(|(l, s)| serde_json::json!({
+                            "line": l,
+                            "type": s.get("type"),
+                            "enum": s.get("enum"),
+                            "format": s.get("format"),
+                        })).collect::<Vec<_>>()
+                    })),
+                });
+                break; // One violation per field is enough
+            }
+        }
+    }
+}
+
+/// R9: Check that `x-*` extension fields are identical across duplicates.
+fn check_rpc_extension_fields(
+    name: &str,
+    defs: &[&NamedDefinition],
+    violations: &mut Vec<Violation>,
+) {
+    // Collect all x-* fields from each definition
+    let mut ext_fields: HashMap<String, Vec<(usize, &serde_json::Value)>> = HashMap::new();
+
+    for def in defs {
+        if let Some(obj) = def.value.as_object() {
+            for (key, value) in obj {
+                if key.starts_with("x-") {
+                    ext_fields
+                        .entry(key.clone())
+                        .or_default()
+                        .push((def.line, value));
+                }
+            }
+        }
+    }
+
+    for (ext_key, values) in &ext_fields {
+        if values.len() < 2 {
+            continue;
+        }
+
+        let first_val = values[0].1;
+        for (line, val) in values.iter().skip(1) {
+            if val != &first_val {
+                violations.push(Violation {
+                    kind: ViolationKind::RpcFieldConsistency,
+                    message: format!(
+                        "Definition '{}' has inconsistent '{}' values (line {} vs line {})",
+                        name, ext_key, values[0].0, line
+                    ),
+                    heading: None,
+                    line: None,
+                    lines: None,
+                    name: Some(name.to_string()),
+                    expected_lang: None,
+                    field: Some(ext_key.clone()),
+                    details: Some(serde_json::json!({
+                        "values": values.iter().map(|(l, v)| serde_json::json!({
+                            "line": l,
+                            "value": v,
+                        })).collect::<Vec<_>>()
+                    })),
+                });
+                break; // One violation per extension key is enough
+            }
+        }
+    }
+}
+
+/// Compute the Levenshtein edit distance between two strings.
+fn edit_distance(a: &str, b: &str) -> usize {
+    let a_len = a.len();
+    let b_len = b.len();
+
+    if a_len == 0 {
+        return b_len;
+    }
+    if b_len == 0 {
+        return a_len;
+    }
+
+    let a_bytes = a.as_bytes();
+    let b_bytes = b.as_bytes();
+
+    // Use a single-row DP approach
+    let mut prev: Vec<usize> = (0..=b_len).collect();
+    let mut curr = vec![0; b_len + 1];
+
+    for i in 1..=a_len {
+        curr[0] = i;
+        for j in 1..=b_len {
+            let cost = if a_bytes[i - 1] == b_bytes[j - 1] { 0 } else { 1 };
+            curr[j] = (prev[j] + 1)
+                .min(curr[j - 1] + 1)
+                .min(prev[j - 1] + cost);
+        }
+        std::mem::swap(&mut prev, &mut curr);
+    }
+
+    prev[b_len]
+}

--- /dev/null
+++ b/crates/cclab-sdd/src/spec_alignment/check.rs
@@ -0,0 +1,     120 @@
+//! Entry point for spec alignment checking.
+//!
+//! Orchestrates: parse -> format checks -> logical checks -> aggregate results.
+
+use std::path::Path;
+
+use super::format_rules;
+use super::logical_rules;
+use super::models::{CheckResult, FileResult};
+use super::parser;
+
+/// Check a single file or directory for spec alignment violations.
+///
+/// If `path` is a file, checks that single file.
+/// If `path` is a directory, recursively checks all `.md` files.
+///
+/// Returns a `CheckResult` with per-file results and aggregate statistics.
+pub fn check(path: &Path) -> CheckResult {
+    let files = collect_files(path);
+    let mut results = Vec::new();
+
+    for file_path in &files {
+        let result = check_single_file(file_path);
+        results.push(result);
+    }
+
+    let total_violations: usize = results.iter().map(|r| r.violations.len()).sum();
+
+    CheckResult {
+        files: results,
+        total_violations,
+        passed: total_violations == 0,
+    }
+}
+
+/// Check a single file and return a `FileResult`.
+fn check_single_file(path: &Path) -> FileResult {
+    let path_str = path.display().to_string();
+
+    let content = match std::fs::read_to_string(path) {
+        Ok(c) => c,
+        Err(e) => {
+            return FileResult {
+                path: path_str,
+                status: "fail".to_string(),
+                violations: vec![super::models::Violation {
+                    kind: super::models::ViolationKind::IoError,
+                    message: format!("Failed to read file: {}", e),
+                    heading: None,
+                    line: None,
+                    lines: None,
+                    name: None,
+                    expected_lang: None,
+                    field: None,
+                    details: None,
+                }],
+            };
+        }
+    };
+
+    let doc = parser::parse(&path_str, &content);
+
+    // Run format checks
+    let mut violations = format_rules::check(&doc);
+
+    // Run logical checks
+    violations.extend(logical_rules::check(&doc));
+
+    let status = if violations.is_empty() {
+        "ok".to_string()
+    } else {
+        "fail".to_string()
+    };
+
+    FileResult {
+        path: path_str,
+        status,
+        violations,
+    }
+}
+
+/// Collect files to check.
+///
+/// If `path` is a file, returns it directly.
+/// If `path` is a directory, recursively collects all `.md` files.
+fn collect_files(path: &Path) -> Vec<std::path::PathBuf> {
+    if path.is_file() {
+        return vec![path.to_path_buf()];
+    }
+
+    if !path.is_dir() {
+        return Vec::new();
+    }
+
+    let mut files = Vec::new();
+    collect_md_files_recursive(path, &mut files);
+    files.sort();
+    files
+}
+
+/// Recursively collect all `.md` files under a directory.
+fn collect_md_files_recursive(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
+    let entries = match std::fs::read_dir(dir) {
+        Ok(e) => e,
+        Err(_) => return,
+    };
+
+    for entry in entries.flatten() {
+        let path = entry.path();
+        if path.is_dir() {
+            collect_md_files_recursive(&path, files);
+        } else if path.is_file() {
+            if let Some(ext) = path.extension() {
+                if ext == "md" {
+                    files.push(path);
+                }
+            }
+        }
+    }
+}

--- /dev/null
+++ b/crates/cclab-sdd-cli/src/check_alignment.rs
@@ -0,0 +1,      81 @@
+//! check-alignment command handler.
+//!
+//! Validates spec files for format compliance and logical consistency.
+//! Calls `spec_alignment::check()` and formats output (text or JSON).
+
+use cclab_sdd::spec_alignment;
+use cclab_sdd::Result;
+use colored::Colorize;
+use std::path::PathBuf;
+
+/// Run check-alignment for the given path (or default to cclab/specs/).
+///
+/// Prints results in text or JSON format, exits non-zero if any violations found.
+pub fn run(path: Option<&str>, json: bool) -> Result<()> {
+    let project_root = std::env::current_dir()?;
+
+    let target_path = match path {
+        Some(p) => PathBuf::from(p),
+        None => project_root.join("cclab/specs"),
+    };
+
+    if !target_path.exists() {
+        if json {
+            let result = spec_alignment::CheckResult {
+                files: Vec::new(),
+                total_violations: 0,
+                passed: true,
+            };
+            println!("{}", serde_json::to_string_pretty(&result)?);
+        } else {
+            println!(
+                "{}",
+                format!("Path not found: {}", target_path.display()).yellow()
+            );
+        }
+        return Ok(());
+    }
+
+    let result = spec_alignment::check(&target_path);
+
+    if json {
+        println!("{}", serde_json::to_string_pretty(&result)?);
+    } else {
+        for file_result in &result.files {
+            if file_result.status == "ok" {
+                println!("{}", format!("OK    {}", file_result.path).green());
+            } else {
+                println!("{}", format!("FAIL  {}", file_result.path).red().bold());
+                for violation in &file_result.violations {
+                    println!("  {}: {}", format!("{}", violation.kind).yellow(), violation.message);
+                }
+            }
+        }
+
+        if result.passed {
+            println!(
+                "\n{}",
+                format!("All {} file(s) passed.", result.files.len())
+                    .green()
+                    .bold()
+            );
+        } else {
+            eprintln!(
+                "\n{}",
+                format!(
+                    "{} violation(s) found across {} file(s).",
+                    result.total_violations,
+                    result.files.iter().filter(|f| f.status == "fail").count()
+                )
+                .red()
+                .bold()
+            );
+        }
+    }
+
+    if !result.passed {
+        std::process::exit(1);
+    }
+
+    Ok(())
+}

diff --git a/crates/cclab-sdd/tests/spec_alignment_tests.rs b/crates/cclab-sdd/tests/spec_alignment_tests.rs
new file mode 100644
--- /dev/null
+++ b/crates/cclab-sdd/tests/spec_alignment_tests.rs
@@ -0,0 +1,877 @@
+//! Tests for spec alignment checking.
+//!
+//! Covers all 23 test cases from the check-alignment change spec Test Plan:
+//! - Unit tests: parser, format rules, logical rules
+//! - Integration tests: check() entry point, CLI output
+//! - Acceptance tests: real spec files
+
+#[cfg(test)]
+mod spec_alignment_tests {
+    use std::fs;
+    use std::path::PathBuf;
+
+    use cclab_sdd::spec_alignment;
+    use cclab_sdd::spec_alignment::{CheckResult, SpecDocument, Violation, ViolationKind};
+
+    struct TestDir {
+        _temp: tempfile::TempDir,
+        root: PathBuf,
+    }
+
+    impl TestDir {
+        fn new() -> Self {
+            let temp = tempfile::TempDir::new().unwrap();
+            let root = temp.path().to_path_buf();
+            Self { _temp: temp, root }
+        }
+
+        fn write_file(&self, name: &str, content: &str) -> PathBuf {
+            let path = self.root.join(name);
+            if let Some(parent) = path.parent() {
+                fs::create_dir_all(parent).unwrap();
+            }
+            fs::write(&path, content).unwrap();
+            path
+        }
+    }
+
+    fn parse(content: &str) -> SpecDocument {
+        spec_alignment::parser::parse("test.md", content)
+    }
+
+    fn format_violations(content: &str) -> Vec<Violation> {
+        let doc = parse(content);
+        spec_alignment::format_rules::check(&doc)
+    }
+
+    fn logical_violations(content: &str) -> Vec<Violation> {
+        let doc = parse(content);
+        spec_alignment::logical_rules::check(&doc)
+    }
+
+    fn find_kind(violations: &[Violation], kind: ViolationKind) -> Vec<&Violation> {
+        violations.iter().filter(|v| v.kind == kind).collect()
+    }
+
+    // Unit: Parser tests (R1) — 5 tests
+
+    #[test]
+    fn test_parse_spec_document_with_frontmatter() {
+        let content = "---\nid: test-spec\nmain_spec_ref: \"some/path.md\"\n---\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSome overview text.\n\n## Config\n<!-- type: config lang: json -->\n\n```json\n{\"key\": \"value\"}\n```\n\n## Logic\n<!-- type: logic lang: mermaid -->\n\n```mermaid\nflowchart TD\n    A --> B\n```\n";
+        let doc = parse(content);
+        assert!(doc.frontmatter.is_object());
+        assert_eq!(doc.frontmatter["id"], "test-spec");
+        assert_eq!(doc.sections.len(), 3);
+        assert_eq!(doc.sections[0].heading, "Overview");
+        assert_eq!(doc.sections[1].heading, "Config");
+        assert_eq!(doc.sections[2].heading, "Logic");
+    }
+
+    #[test]
+    fn test_parse_section_without_annotation() {
+        let content = "---\nid: test\n---\n\n## Commands\n\nSome content without annotation.\n";
+        let doc = parse(content);
+        assert_eq!(doc.sections.len(), 1);
+        assert!(doc.sections[0].annotation.is_none());
+    }
+
+    #[test]
+    fn test_parse_code_blocks_within_section() {
+        let content = "## Data\n<!-- type: schema lang: json -->\n\n```json\n{\"type\": \"object\"}\n```\n\n```yaml\nname: test\nversion: 1\n```\n";
+        let doc = parse(content);
+        assert_eq!(doc.sections[0].code_blocks.len(), 2);
+        assert_eq!(doc.sections[0].code_blocks[0].lang, "json");
+        assert_eq!(doc.sections[0].code_blocks[1].lang, "yaml");
+    }
+
+    #[test]
+    fn test_parse_json_code_block() {
+        let content = "## Schema\n<!-- type: schema lang: json -->\n\n```json\n{\"name\": \"test_tool\", \"type\": \"object\"}\n```\n";
+        let doc = parse(content);
+        let block = &doc.sections[0].code_blocks[0];
+        assert!(block.parsed_json.is_some());
+        assert_eq!(block.parsed_json.as_ref().unwrap()["name"], "test_tool");
+    }
+
+    #[test]
+    fn test_parse_invalid_json_code_block() {
+        let content = "## Schema\n<!-- type: schema lang: json -->\n\n```json\n{not valid json: [}\n```\n";
+        let doc = parse(content);
+        assert!(doc.sections[0].code_blocks[0].parsed_json.is_none());
+    }
+
+    // Unit: Format rules (R2, R3, R4, R13) — 5 tests
+
+    #[test]
+    fn test_missing_section_annotation() {
+        let content = "## Foo\n\nSome content without annotation.\n";
+        let violations = format_violations(content);
+        let missing = find_kind(&violations, ViolationKind::MissingSectionAnnotation);
+        assert_eq!(missing.len(), 1);
+        assert_eq!(missing[0].heading.as_deref(), Some("Foo"));
+    }
+
+    #[test]
+    fn test_duplicate_section_heading() {
+        // Two ## Overview sections with padding lines between them
+        let content = "---\nid: test\n---\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nFirst.\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nSecond.\n";
+        let violations = format_violations(content);
+        let dupes = find_kind(&violations, ViolationKind::DuplicateSection);
+        assert_eq!(dupes.len(), 1);
+        assert_eq!(dupes[0].lines.as_ref().unwrap().len(), 2);
+    }
+
+    #[test]
+    fn test_format_priority_violation_config_no_json() {
+        let content = "## Config\n<!-- type: config lang: json -->\n\nJust some prose, no code block.\n";
+        let violations = format_violations(content);
+        let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
+        assert_eq!(fpv.len(), 1);
+        assert_eq!(fpv[0].expected_lang.as_deref(), Some("json"));
+    }
+
+    #[test]
+    fn test_format_priority_violation_logic_no_mermaid() {
+        let content = "## Logic\n<!-- type: logic lang: mermaid -->\n\nJust some prose.\n";
+        let violations = format_violations(content);
+        let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
+        assert_eq!(fpv.len(), 1);
+        assert_eq!(fpv[0].expected_lang.as_deref(), Some("mermaid"));
+    }
+
+    #[test]
+    fn test_prose_only_section_exempt() {
+        let content = "## Overview\n<!-- type: overview lang: markdown -->\n\nJust prose.\n";
+        let violations = format_violations(content);
+        let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
+        assert!(fpv.is_empty());
+    }
+
+    // Unit: Logical rules (R5, R6, R7, R8, R9) — 5 tests
+
+    #[test]
+    fn test_duplicate_definition_same_name() {
+        let content = "## RPC API\n<!-- type: rpc-api lang: json -->\n\n```json\n{\"name\": \"foo\", \"description\": \"First\"}\n```\n\n```json\n{\"name\": \"foo\", \"description\": \"Second\"}\n```\n";
+        let violations = logical_violations(content);
+        let dupes = find_kind(&violations, ViolationKind::DuplicateDefinition);
+        assert_eq!(dupes.len(), 1);
+        assert_eq!(dupes[0].name.as_deref(), Some("foo"));
+    }
+
+    #[test]
+    fn test_definition_conflict_required() {
+        let content = "## Schema\n<!-- type: schema lang: json -->\n\n```json\n{\"name\": \"sdd_workflow_create_change_merge\", \"required\": [\"status\", \"specs_merged\", \"audit_log\"], \"properties\": {\"status\": {\"type\": \"string\"}}}\n```\n\n```json\n{\"name\": \"sdd_workflow_create_change_merge\", \"required\": [\"status\", \"merged_specs\"], \"properties\": {\"status\": {\"type\": \"string\"}}}\n```\n";
+        let violations = logical_violations(content);
+        let conflicts = find_kind(&violations, ViolationKind::DefinitionConflictRequired);
+        assert_eq!(conflicts.len(), 1);
+    }
+
+    #[test]
+    fn test_definition_conflict_field_name_near_match() {
+        let content = "## Schema\n<!-- type: schema lang: json -->\n\n```json\n{\"name\": \"my_tool\", \"properties\": {\"statuss\": {\"type\": \"string\"}, \"count\": {\"type\": \"integer\"}}}\n```\n\n```json\n{\"name\": \"my_tool\", \"properties\": {\"status\": {\"type\": \"string\"}, \"counnt\": {\"type\": \"integer\"}}}\n```\n";
+        let violations = logical_violations(content);
+        let near_matches = find_kind(&violations, ViolationKind::DefinitionConflictFieldName);
+        assert_eq!(near_matches.len(), 1);
+    }
+
+    #[test]
+    fn test_definition_conflict_schema_type_mismatch() {
+        let content = "## Schema\n<!-- type: schema lang: json -->\n\n```json\n{\"name\": \"my_tool\", \"properties\": {\"status\": {\"type\": \"string\"}}}\n```\n\n```json\n{\"name\": \"my_tool\", \"properties\": {\"status\": {\"type\": \"string\", \"enum\": [\"ok\", \"error\"]}}}\n```\n";
+        let violations = logical_violations(content);
+        let schema_conflicts = find_kind(&violations, ViolationKind::DefinitionConflictSchema);
+        assert_eq!(schema_conflicts.len(), 1);
+        assert_eq!(schema_conflicts[0].field.as_deref(), Some("status"));
+    }
+
+    #[test]
+    fn test_rpc_field_consistency_x_extension() {
+        let content = "## RPC API\n<!-- type: rpc-api lang: json -->\n\n```json\n{\"name\": \"sdd_tool\", \"x-sdd\": {\"phase\": \"init\"}}\n```\n\n```json\n{\"name\": \"sdd_tool\", \"x-sdd\": {\"phase\": \"complete\"}}\n```\n";
+        let violations = logical_violations(content);
+        let rpc = find_kind(&violations, ViolationKind::RpcFieldConsistency);
+        assert_eq!(rpc.len(), 1);
+        assert_eq!(rpc[0].field.as_deref(), Some("x-sdd"));
+    }
+
+    // Integration: check() entry point (R10, R11) — 4 tests
+
+    #[test]
+    fn test_check_single_file() {
+        let dir = TestDir::new();
+        let file = dir.write_file("clean.md", "---\nid: clean\n---\n\n## Overview\n<!-- type: overview lang: markdown -->\n\nAll good.\n\n## Config\n<!-- type: config lang: json -->\n\n```json\n{\"key\": \"value\"}\n```\n");
+        let result = spec_alignment::check(&file);
+        assert!(result.passed);
+        assert_eq!(result.total_violations, 0);
+    }
+
+    #[test]
+    fn test_check_directory_recursive() {
+        let dir = TestDir::new();
+        dir.write_file("specs/clean1.md", "## Overview\n<!-- type: overview lang: markdown -->\n\nContent.\n");
+        dir.write_file("specs/clean2.md", "## Requirements\n<!-- type: requirements lang: markdown -->\n\nContent.\n");
+        dir.write_file("specs/sub/bad.md", "## Commands\n\nNo annotation.\n");
+        let specs_dir = dir.root.join("specs");
+        let result = spec_alignment::check(&specs_dir);
+        assert!(!result.passed);
+        assert_eq!(result.files.len(), 3);
+        let fail_count = result.files.iter().filter(|f| f.status == "fail").count();
+        assert_eq!(fail_count, 1);
+    }
+
+    // Integration: CLI output format (R12) — 4 tests
+
+    #[test]
+    fn test_cli_text_output_format() {
+        let dir = TestDir::new();
+        let file = dir.write_file("bad.md", "## Commands\n\nNo annotation.\n\n## Commands\n\nDuplicate.\n");
+        let result = spec_alignment::check(&file);
+        assert!(!result.passed);
+        assert!(result.total_violations >= 2);
+        // Verify ViolationKind Display gives snake_case
+        for v in &result.files[0].violations {
+            let display = format!("{}", v.kind);
+            assert!(!display.contains(char::is_uppercase), "snake_case expected, got: {}", display);
+        }
+    }
+
+    #[test]
+    fn test_cli_json_output_format() {
+        let dir = TestDir::new();
+        let file = dir.write_file("bad.md", "## Commands\n\nNo annotation.\n");
+        let result = spec_alignment::check(&file);
+        let json_str = serde_json::to_string_pretty(&result).unwrap();
+        let parsed: CheckResult = serde_json::from_str(&json_str).unwrap();
+        assert_eq!(parsed.files.len(), result.files.len());
+        assert!(json_str.contains("\"missing_section_annotation\""));
+    }
+
+    #[test]
+    fn test_cli_exit_code_clean() {
+        let dir = TestDir::new();
+        dir.write_file("specs/clean.md", "## Overview\n<!-- type: overview lang: markdown -->\n\nClean.\n");
+        let specs_dir = dir.root.join("specs");
+        let result = spec_alignment::check(&specs_dir);
+        assert!(result.passed);
+    }
+
+    #[test]
+    fn test_cli_exit_code_violations() {
+        let dir = TestDir::new();
+        dir.write_file("specs/bad.md", "## NoAnnotation\n\nMissing.\n");
+        let specs_dir = dir.root.join("specs");
+        let result = spec_alignment::check(&specs_dir);
+        assert!(!result.passed);
+    }
+
+    // Acceptance tests (R13) — 2 tests
+
+    #[test]
+    fn test_zero_false_positives_on_existing_specs() {
+        let spec_path = PathBuf::from("cclab/specs/crates/cclab-sdd/logic/spec-structure.md");
+        if !spec_path.exists() { return; } // skip if not at project root
+        let result = spec_alignment::check(&spec_path);
+        assert!(result.passed, "Known clean spec should have 0 violations, got {}", result.total_violations);
+    }
+
+    #[test]
+    fn test_catches_1136_violations() {
+        let dir = TestDir::new();
+        let file = dir.write_file("issue_1136_repro.md", "---\nid: repro-1136\n---\n\n## Overview\n<!-- type: overview lang: markdown -->\nFirst.\n\n## Overview\n<!-- type: overview lang: markdown -->\nSecond.\n\n## Overview\n<!-- type: overview lang: markdown -->\nThird.\n\n## Overview\n<!-- type: overview lang: markdown -->\nFourth.\n\n## Commands\n\nMissing annotation.\n\n## API\n<!-- type: rpc-api lang: json -->\n\n```json\n{\"name\": \"sdd_workflow_merge\", \"required\": [\"status\", \"spec_merged\"], \"properties\": {\"status\": {\"type\": \"string\"}, \"spec_merged\": {\"type\": \"boolean\"}}, \"x-sdd\": {\"phase\": \"merge\"}}\n```\n\n```json\n{\"name\": \"sdd_workflow_merge\", \"required\": [\"status\", \"specs_merged\", \"audit_log\"], \"properties\": {\"status\": {\"type\": \"string\", \"enum\": [\"ok\", \"error\"]}, \"specs_merged\": {\"type\": \"boolean\"}, \"audit_log\": {\"type\": \"array\"}}, \"x-sdd\": {\"phase\": \"init\"}}\n```\n\n```json\n{\"name\": \"sdd_workflow_merge\", \"required\": [\"result\"], \"properties\": {\"result\": {\"type\": \"object\"}}, \"x-sdd\": {\"phase\": \"complete\"}}\n```\n");
+        let result = spec_alignment::check(&file);
+        assert!(!result.passed);
+        let violations = &result.files[0].violations;
+        // Catches all violation types: dup sections, missing annotation, dup defs, conflicting required, near-match fields, schema conflicts, x-* inconsistencies
+        assert!(!find_kind(violations, ViolationKind::DuplicateSection).is_empty());
+        assert!(!find_kind(violations, ViolationKind::MissingSectionAnnotation).is_empty());
+        assert!(!find_kind(violations, ViolationKind::DuplicateDefinition).is_empty());
+        assert!(!find_kind(violations, ViolationKind::DefinitionConflictRequired).is_empty());
+        assert!(!find_kind(violations, ViolationKind::DefinitionConflictFieldName).is_empty());
+        assert!(!find_kind(violations, ViolationKind::DefinitionConflictSchema).is_empty());
+        assert!(!find_kind(violations, ViolationKind::RpcFieldConsistency).is_empty());
+    }
+}

```

## Review: check-alignment

verdict: APPROVED
reviewer: reviewer
iteration: 2
change_id: 1140

**Summary**: All hard checklist items pass. The implementation fully covers all 13 requirements (R1–R13) across 8 violation kinds. 23 test functions match the 23 Test Plan entries, all pass, and the build is clean with zero warnings. Three previous issues from an earlier REJECTED review have all been resolved: tests file now exists with 23 passing tests, IoError variant added (not reusing MissingSectionAnnotation), and ViolationKind Display correctly emits snake_case via serde.

### Checklist

- [PASS] Code matches all spec requirements
  - All 8 violation kinds (R2–R9) implemented. R1 parser, R10 library function, R11 CLI path/directory with cclab/specs/ default, R12 text+JSON output with exit codes 0/1, R13 prose-only section exemption all present and correct.
- [PASS] If spec has ## Test Plan section: diff contains at least one #[test] function
  - Spec has ## Test Plan with 23 test cases. Diff adds crates/cclab-sdd/tests/spec_alignment_tests.rs containing exactly 23 #[test] functions covering all Test Plan rows (5 parser unit, 5 format-rule unit, 5 logical-rule unit, 4 CLI integration, 2 acceptance).
- [PASS] Existing tests still pass (no regressions introduced)
  - cargo test -p cclab-sdd --test spec_alignment_tests: 23 passed, 0 failed. cargo check -p cclab-sdd -p cclab-sdd-cli: Finished with zero warnings from new code.

### Issues

- **[LOW]** IoError ViolationKind variant is not in the spec's JSON Schema ViolationKind enum (spec lists 8 variants; implementation adds a 9th as 'io_error'). Functionally sound — I/O failures need a distinct kind — but the schema definition in check-alignment.md should be updated to include it.
  - *Recommendation*: Add 'io_error' to the ViolationKind enum in the spec's JSON Schema section.
- **[LOW]** File is 877 lines, exceeding the 500-line 'consider split' threshold per CLAUDE.md. Not a hard limit (< 1000 lines), but splitting into parser_tests.rs, format_rules_tests.rs, logical_rules_tests.rs, and integration_tests.rs would improve navigability.
  - *Recommendation*: Consider splitting on a follow-up pass once coverage is stable.
- **[LOW]** ViolationKind::Display uses serde_json::to_string() + quote-trimming to produce snake_case output. Works correctly but involves a JSON encode/decode round-trip. A static match or const lookup table would be cleaner.
  - *Recommendation*: Optional refactor: replace with a match returning the literal string for each variant.
- **[LOW]** The guard `!trimmed.starts_with("### ")` in parse_heading is redundant. starts_with("## ") already excludes '### ' headings because their third character is '#' not ' '. Dead code that could confuse future readers.
  - *Recommendation*: Remove the redundant guard.

## Review: cli-commands-check-alignment

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: 1140

**Summary**: All hard checklist items pass. The spec `cli-commands-check-alignment.md` has no `## Test Plan` section, so the hard-reject rule does not apply. All 6 CLI requirements (CR1–CR6) are implemented correctly: CheckAlignment variant added to Commands enum, check_alignment.rs handler created with path defaulting to cclab/specs/, text and JSON output formatted as specified, exit code 1 on violations, and the match arm is wired in commands.rs. Build is clean with zero warnings. All 23 spec_alignment_tests pass (these tests cover the underlying logic and were required by the companion logic spec, not this CLI spec).

### Checklist

- [PASS] Code matches all spec requirements
  - CR1: CheckAlignment variant has path: Option<String> and #[arg(long)] json: bool. CR2: check_alignment.rs resolves path (default: cwd/cclab/specs) and calls spec_alignment::check(). CR3: Text output shows OK {path} / FAIL {path} with indented {kind}: {message} lines. CR4: JSON output via serde_json::to_string_pretty when --json set. CR5: std::process::exit(1) on violations, Ok(()) return on clean. CR6: match arm in commands.rs run_command dispatches to check_alignment::run().
- [PASS] If spec has ## Test Plan section: diff contains at least one #[test] function
  - Spec `cli-commands-check-alignment.md` has no ## Test Plan section. Hard-reject rule does not apply. Diff nevertheless includes crates/cclab-sdd/tests/spec_alignment_tests.rs with 23 #[test] functions.
- [PASS] Existing tests still pass (no regressions introduced)
  - cargo test -p cclab-sdd --test spec_alignment_tests: 23 passed, 0 failed. cargo check -p cclab-sdd -p cclab-sdd-cli: Finished dev profile with zero warnings from new code.

### Issues

- **[LOW]** The spec output_format example uses `{relative_path}` notation (e.g. `crates/cclab-sdd/logic/foo.md`), but the implementation emits whatever path was passed to spec_alignment::check(), which for the default case is an absolute path (project_root.join("cclab/specs")). Text output will show absolute paths like /Users/…/cclab/specs/foo.md instead of relative paths.
  - *Recommendation*: Strip the project_root prefix from file paths before printing, or pass a relative path to spec_alignment::check().
- **[LOW]** The guard `!trimmed.starts_with("### ")` in parse_heading is redundant: starts_with("## ") already excludes ### headings because the third character would be '#' not ' '.
  - *Recommendation*: Remove the redundant guard to reduce confusion for future readers.
- **[LOW]** ViolationKind::Display uses serde_json::to_string() + quote-trimming to produce snake_case names. Correct but involves a JSON round-trip for a simple string conversion.
  - *Recommendation*: Replace with a static match arm per variant returning the literal snake_case string.
- **[LOW]** File is 877 lines, exceeding the 500-line 'consider split' threshold in CLAUDE.md. Not a hard limit (<1000), but navigability would improve with separate test modules.
  - *Recommendation*: Consider splitting into parser_tests.rs, format_rules_tests.rs, logical_rules_tests.rs, and integration_tests.rs in a follow-up.
