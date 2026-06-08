// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rule-rs.md#source
// CODEGEN-BEGIN
//! `Rule` trait + `Finding` type.
//!
//! A `Rule` takes a spec's text content and emits zero or more `Finding`s.
//! Rules are stateless; they parse the slice they need per-invocation.
//! The router (in `crate::validate::rules::mod::all_rules`) runs every rule
//! against every spec in scope.
//!
//! Type definitions (`RuleId`, `Severity`, `Finding`, `RuleReport`) and their
//! inherent codegen-owned methods (constructors, builders, dispatch arms) are
//! produced by `aw td gen-code` from
//! `projects/agentic-workflow/tech-design/core/validate/rule.md` and live inside the
//! `CODEGEN-BEGIN` / `CODEGEN-END` block at the bottom of this file. The
//! items kept hand-written above the marker are:
//!
//! - `Finding::format` — single-line human output, exact string layout is
//!   not derivable from the schema.
//! - `RuleReport` inherent methods — the bodies (`self.findings.push(...)`
//!   etc.) are one-liners the spec does not capture.
//! - `Rule` trait — pure-function interface owned by each concrete rule
//!   implementation, not the schema.
//! - `#[cfg(test)] mod tests` — preserved verbatim; `aw td gen-code` does
//!   not touch it on `action: modify` files.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Hand-written method impls for codegen-declared types.
//
// The types live inside the CODEGEN block below. These `impl` blocks
// intentionally sit OUTSIDE CODEGEN-BEGIN/END and are owned by humans.
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rule-rs.md#source
impl Finding {
    /// Format for single-line human output.
    ///
    /// Tag format is `[{category}:{slug}]` — category gives the agent a
    /// one-word "what kind of problem" (codegen / structure / format /
    /// logic), slug names the specific rule. The R-id (e.g. R3a) is kept in
    /// the JSON `rule` field for traceability back to issue Requirements.
    pub fn format(&self) -> String {
        let loc = match (self.line, self.path.as_deref()) {
            (Some(l), Some(p)) => format!("{}:{} ({})", self.file.display(), l, p),
            (Some(l), None) => format!("{}:{}", self.file.display(), l),
            (None, Some(p)) => format!("{} ({})", self.file.display(), p),
            (None, None) => format!("{}", self.file.display()),
        };
        format!(
            "[{}:{}] {} \u{2014} {}",
            self.rule.category(),
            self.rule.slug(),
            loc,
            self.message,
        )
    }
}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rule-rs.md#source
impl RuleId {
    /// Category bucket: one-word "what kind of problem". Read by
    /// `Finding::format` to produce the `[{category}:{slug}]` text tag.
    pub fn category(&self) -> &'static str {
        match self {
            // Rust-type / codegen sanity (R3a–R3g).
            RuleId::DoubleOption
            | RuleId::NullableRequired
            | RuleId::OrphanBinding
            | RuleId::LowercaseEnum
            | RuleId::ImplModeMisuse
            | RuleId::CodegenReady
            | RuleId::RustTypeConsistency => "codegen",
            // Section format / annotation / fence (R3h, R7a–R7c).
            RuleId::SectionFormat
            | RuleId::MissingSectionAnnotation
            | RuleId::FormatPriorityViolation
            | RuleId::DuplicateSection => "format",
            // TD directory shape (R6a, R6b).
            RuleId::LooseRootFile | RuleId::UnexpectedSubdir => "structure",
            // Cross-section logical consistency (R7d, R7e, R7f).
            RuleId::OrphanRequirement | RuleId::SchemaConflict | RuleId::FieldNearMatch => "logic",
        }
    }

    /// Just the slug part of the existing `short()` ("double-Option",
    /// "unexpected-subdir", ...) — the readable tail without the R-id
    /// prefix. Implemented by parsing `short()` so this stays in sync with
    /// codegen's `short()` arm.
    pub fn slug(&self) -> &'static str {
        let s = self.short();
        match s.split_once(':') {
            Some((_, slug)) => slug,
            None => s,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rule-rs.md#source
impl RuleReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, f: Finding) {
        self.findings.push(f);
    }

    pub fn extend(&mut self, other: RuleReport) {
        self.findings.extend(other.findings);
    }

    /// True iff at least one finding has `Severity::Error`.
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    pub fn is_empty(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Pure-function rule trait.
///
/// Each rule takes the spec's `(path, raw text)` and appends its findings
/// into a shared `RuleReport`. Rules MUST NOT mutate state outside the
/// report; no I/O beyond reading the provided text.
/// @spec projects/agentic-workflow/tech-design/core/validate/source/projects-sdd-src-validate-rule-rs.md#source
pub trait Rule {
    fn id(&self) -> RuleId;
    fn check(&self, spec_path: &std::path::Path, content: &str, report: &mut RuleReport);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_format_uses_category_slug_tag() {
        let f = Finding::error(RuleId::DoubleOption, "spec.md", "bad type")
            .with_line(42)
            .with_path("schemas[0].rust_type");
        let out = f.format();
        // Tag is [category:slug] — agent reads category first.
        assert!(out.contains("[codegen:double-Option]"), "got: {}", out);
        assert!(out.contains("spec.md:42"));
        assert!(out.contains("schemas[0].rust_type"));
    }

    #[test]
    fn category_buckets_match_design() {
        assert_eq!(RuleId::DoubleOption.category(), "codegen");
        assert_eq!(RuleId::SectionFormat.category(), "format");
        assert_eq!(RuleId::MissingSectionAnnotation.category(), "format");
        assert_eq!(RuleId::LooseRootFile.category(), "structure");
        assert_eq!(RuleId::UnexpectedSubdir.category(), "structure");
        assert_eq!(RuleId::OrphanRequirement.category(), "logic");
        assert_eq!(RuleId::SchemaConflict.category(), "logic");
        assert_eq!(RuleId::FieldNearMatch.category(), "logic");
    }

    #[test]
    fn slug_strips_r_id_prefix() {
        assert_eq!(RuleId::DoubleOption.slug(), "double-Option");
        assert_eq!(RuleId::UnexpectedSubdir.slug(), "unexpected-subdir");
    }

    #[test]
    fn report_has_errors_returns_false_when_empty() {
        let r = RuleReport::new();
        assert!(!r.has_errors());
        assert!(r.is_empty());
    }

    #[test]
    fn report_has_errors_returns_true_on_error() {
        let mut r = RuleReport::new();
        r.push(Finding::error(RuleId::DoubleOption, "x.md", "m"));
        assert!(r.has_errors());
        assert!(!r.is_empty());
    }
}
use serde::{Deserialize, Serialize};

/// Rule identifier. Mirrors the R-ids used in issue Requirements tables
/// so a failing rule points authors at the exact requirement it enforces.
/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuleId {
    /// R3a — reject Option<Option<T>> in any rust_type.
    #[serde(rename = "DoubleOption")]
    DoubleOption,
    /// R3b — reject required: true with nullable rust_type, or vice versa.
    #[serde(rename = "NullableRequired")]
    NullableRequired,
    /// R3c — reject x-mamba-binding pointing at an absent schema/section.
    #[serde(rename = "OrphanBinding")]
    OrphanBinding,
    /// R3d — reject lowercase enum rust_type (must be PascalCase).
    #[serde(rename = "LowercaseEnum")]
    LowercaseEnum,
    /// R3e — reject impl_mode on sections where it has no meaning, or values outside {codegen, hand-written}.
    #[serde(rename = "ImplModeMisuse")]
    ImplModeMisuse,
    /// R3f — codegen-ready gate. Mermaid Plus frontmatter required on codegen sections.
    #[serde(rename = "CodegenReady")]
    CodegenReady,
    /// R3g — cross-section consistency. rust_type in changes must match rust_type in schemas.
    #[serde(rename = "RustTypeConsistency")]
    RustTypeConsistency,
    /// R3h — enforce section annotation/fence format and Mermaid Plus frontmatter.
    #[serde(rename = "SectionFormat")]
    SectionFormat,
    /// R6a — reject loose .md files at crate spec roots and directly under interfaces/.
    #[serde(rename = "LooseRootFile")]
    LooseRootFile,
    /// R6b — reject spec files inside unexpected top-level subdirectories.
    #[serde(rename = "UnexpectedSubdir")]
    UnexpectedSubdir,
    /// R7a — reject H2 sections missing a type/lang annotation.
    #[serde(rename = "MissingSectionAnnotation")]
    MissingSectionAnnotation,
    /// R7b — reject sections whose annotated type lacks the required fenced format.
    #[serde(rename = "FormatPriorityViolation")]
    FormatPriorityViolation,
    /// R7c — reject duplicate section headings within the same spec file.
    #[serde(rename = "DuplicateSection")]
    DuplicateSection,
    /// R7d — reject requirements not referenced by scenarios or unit-test coverage.
    #[serde(rename = "OrphanRequirement")]
    OrphanRequirement,
    /// R7e — reject conflicting schema definitions for the same named entity.
    #[serde(rename = "SchemaConflict")]
    SchemaConflict,
    /// R7f — reject near-match field names that likely indicate schema typos.
    #[serde(rename = "FieldNearMatch")]
    FieldNearMatch,
}

/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema.impls
impl RuleId {
    /// Short human name used in CLI output and JSON.
    pub fn short(&self) -> &'static str {
        match self {
            RuleId::DoubleOption => "R3a:double-Option",
            RuleId::NullableRequired => "R3b:nullable-required",
            RuleId::OrphanBinding => "R3c:orphan-binding",
            RuleId::LowercaseEnum => "R3d:lowercase-enum",
            RuleId::ImplModeMisuse => "R3e:impl_mode-misuse",
            RuleId::CodegenReady => "R3f:codegen-ready",
            RuleId::RustTypeConsistency => "R3g:rust_type-consistency",
            RuleId::SectionFormat => "R3h:section-format",
            RuleId::LooseRootFile => "R6a:loose-root-file",
            RuleId::UnexpectedSubdir => "R6b:unexpected-subdir",
            RuleId::MissingSectionAnnotation => "R7a:missing-section-annotation",
            RuleId::FormatPriorityViolation => "R7b:format-priority-violation",
            RuleId::DuplicateSection => "R7c:duplicate-section",
            RuleId::OrphanRequirement => "R7d:orphan-requirement",
            RuleId::SchemaConflict => "R7e:schema-conflict",
            RuleId::FieldNearMatch => "R7f:field-near-match",
        }
    }
}

/// Severity level. Error blocks aw td validate; Warning is advisory.
/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    #[serde(rename = "Error")]
    Error,
    #[serde(rename = "Warning")]
    Warning,
}

/// Single rule-violation finding.
/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Which rule fired.
    pub rule: RuleId,
    /// Spec file where the violation lives (PathBuf in Rust).
    pub file: PathBuf,
    /// Optional 1-indexed line number if the rule can pinpoint it.
    #[serde(default)]
    pub line: Option<usize>,
    /// Optional YAML path hint (e.g. schemas[0].properties.status_code).
    #[serde(default)]
    pub path: Option<String>,
    /// Human-readable violation message.
    pub message: String,
    /// Severity — blocks validate if Error.
    pub severity: Severity,
}

/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema.impls
impl Finding {
    /// Construct an error-severity finding.
    pub fn error(rule: RuleId, file: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self {
            rule,
            file: file.into(),
            message: message.into(),
            line: None,
            path: None,
            severity: Severity::Error,
        }
    }

    /// Attach a line number.
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Attach a YAML path hint.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Aggregate of findings across a single spec file or a path-prefix walk.
/// @spec projects/agentic-workflow/tech-design/core/validate/rule.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleReport {
    /// Accumulated rule-violation findings.
    pub findings: Vec<Finding>,
}

// CODEGEN-END
