//! TDAst-driven generator dispatch (Stage 2 of the TD→AST→Code migration).
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
//! @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Context passed to every generator. Holds the source spec path (for
/// SPEC-REF anchoring) and the workspace target language hint.
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone)]
pub struct DispatchCtx {
    /// Absolute path of the source TD spec.
    pub spec_path: std::path::PathBuf,
    /// Prefix used when forming SPEC-REF anchors (e.g. relative spec path).
    pub spec_ref_prefix: String,
    /// Workspace-selected target language (rs / py / ts / css). None defaults to rs.
    pub target_lang: Option<String>,
}

/// Mixed codegen maturity band used by dispatch and health reports.
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DispatchMaturity {
    #[serde(rename = "semantic_generator")]
    SemanticGenerator,
    #[serde(rename = "structural_generator")]
    StructuralGenerator,
    #[serde(rename = "source_replay")]
    SourceReplay,
    #[serde(rename = "artifact_replay")]
    ArtifactReplay,
    #[serde(rename = "handwrite_gap")]
    HandwriteGap,
    #[serde(rename = "none")]
    None,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema.trait-impls.Default
impl Default for DispatchMaturity {
    fn default() -> Self {
        Self::None
    }
}

/// Per-section dispatch result. Aggregated into [`DispatchReport`] so the
/// `apply.rs` pipeline can decide whether to write file changes.
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOutcome {
    /// Canonical SectionType name (e.g. schema, cli, logic).
    pub section_type: String,
    /// Generator id that handled the section (e.g. rust-schema, cli-subcommand).
    pub generator: String,
    /// Emitted | Skipped | NoGenerator | LegacyFallback | Failed.
    pub status: DispatchStatus,
    /// Codegen route selected for this typed section.
    #[serde(default)]
    pub strategy: DispatchStrategy,
    /// Maturity band for reporting mixed codegen coverage.
    #[serde(default)]
    pub maturity: DispatchMaturity,
    /// True when this route is backed by source replay rather than AST emission.
    #[serde(default)]
    pub source_backed: bool,
    /// Stable generator gap id for unsupported typed shapes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Aggregated dispatcher result for one TD spec. Carries one
/// [`DispatchOutcome`] per visited section.
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DispatchReport {
    pub outcomes: Vec<DispatchOutcome>,
    /// Paths in Changes section that no other typed payload references (R4).
    #[serde(default)]
    pub orphan_changes_paths: Vec<String>,
}

/// Discriminator for [`DispatchOutcome::status`]. `LegacyFallback` means
/// the dispatcher routed through the legacy per-section-string path
/// because the typed generator is not yet wired (Stage 2B follow-up).
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DispatchStatus {
    #[serde(rename = "emitted")]
    Emitted,
    #[serde(rename = "skipped")]
    Skipped,
    #[serde(rename = "no_generator")]
    NoGenerator,
    #[serde(rename = "legacy_fallback")]
    LegacyFallback,
    #[serde(rename = "failed")]
    Failed,
}

/// High-level route chosen after TD AST parsing.
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DispatchStrategy {
    #[serde(rename = "typed_generator")]
    TypedGenerator,
    #[serde(rename = "source_replay")]
    SourceReplay,
    #[serde(rename = "structural_scaffold")]
    StructuralScaffold,
    #[serde(rename = "handwrite_gap")]
    HandwriteGap,
    #[serde(rename = "none")]
    None,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema.trait-impls.Default
impl Default for DispatchStrategy {
    fn default() -> Self {
        Self::None
    }
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#source
// CODEGEN-BEGIN
use crate::models::spec_rules::SectionType;
use crate::td_ast::types::{TDAst, TDSection, TypedBody};

/// Typed generator input contract shared by AST-first emitters.
///
/// Source replay is still handled by the CB/apply layer because `source` is not
/// a first-class `SectionType`; this contract is for typed TD sections only.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#schema
pub struct GeneratorInput<'a> {
    pub ctx: &'a DispatchCtx,
    pub section: &'a TDSection,
    pub body: &'a TypedBody,
    pub strategy: DispatchStrategy,
}

/// Dispatch every section of a parsed TDAst to its registered generator.
///
/// This is the new Stage 2 entry point that consumes typed payloads from
/// `td_ast::payloads` instead of re-parsing raw spec text. Generators that
/// have not yet been migrated to typed input are recorded as
/// [`DispatchStatus::LegacyFallback`] so `apply.rs` can route them through
/// the legacy substring dispatch.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
pub fn dispatch_from_tdast(td_ast: &TDAst, _ctx: &DispatchCtx) -> DispatchReport {
    let mut report = DispatchReport::default();

    for section in &td_ast.sections {
        let outcome = classify_section(section);
        report.outcomes.push(outcome);
    }

    report.orphan_changes_paths = collect_orphan_changes_paths(td_ast);
    report
}

/// Map a single `TDSection` to a [`DispatchOutcome`] without invoking the
/// per-section generators yet. Stage 2 wires the dispatch table; Stage 2B
/// migrations attach the actual `emit_typed` call sites.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
fn classify_section(section: &TDSection) -> DispatchOutcome {
    let section_type = format!("{:?}", section.section_type).to_ascii_lowercase();

    let (generator, status, strategy, maturity, gap_id, message) =
        match (&section.body, section.section_type) {
            (TypedBody::JsonSchema(_), SectionType::Schema) => {
                typed_generator("rust-schema", DispatchMaturity::SemanticGenerator)
            }
            (TypedBody::JsonSchema(_), SectionType::Changes) => structural_scaffold(
                "changes-manifest",
                "typed changes manifest ready for apply.rs target routing",
            ),
            (TypedBody::CliManifest(_), _) => structural_scaffold(
                "cli-subcommand",
                "typed CLI manifest ready for subcommand generation",
            ),
            (TypedBody::ConfigManifest(_), _) => structural_scaffold(
                "rust-config",
                "typed config manifest ready for config generation",
            ),
            (TypedBody::OpenRpc(_), _) => {
                typed_generator("rust-schema", DispatchMaturity::SemanticGenerator)
            }
            (TypedBody::MermaidPlus(_), SectionType::Logic) => {
                typed_generator("rust-logic-emitter", DispatchMaturity::SemanticGenerator)
            }
            (TypedBody::RustSourceUnit(_), SectionType::RustSourceUnit) => structural_scaffold(
                "rust-source-unit",
                "typed Rust source-unit item tree ready for lossless source regeneration",
            ),
            (TypedBody::Placeholder, _) => (
                "none",
                DispatchStatus::Skipped,
                DispatchStrategy::None,
                DispatchMaturity::None,
                None,
                Some("placeholder section has no generator input".to_string()),
            ),
            (TypedBody::Markdown(_), SectionType::Overview | SectionType::Doc) => (
                "none",
                DispatchStatus::Skipped,
                DispatchStrategy::None,
                DispatchMaturity::None,
                None,
                Some("prose section has no generator input".to_string()),
            ),
            (TypedBody::Unsupported(_), _) => {
                handwrite_gap(section.section_type, "unsupported typed payload shape")
            }
            _ => handwrite_gap(section.section_type, "no typed generator registered"),
        };

    DispatchOutcome {
        section_type,
        generator: generator.to_string(),
        status,
        strategy,
        maturity,
        source_backed: matches!(
            maturity,
            DispatchMaturity::SourceReplay | DispatchMaturity::ArtifactReplay
        ),
        gap_id,
        message,
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
fn typed_generator(
    generator: &'static str,
    maturity: DispatchMaturity,
) -> (
    &'static str,
    DispatchStatus,
    DispatchStrategy,
    DispatchMaturity,
    Option<String>,
    Option<String>,
) {
    (
        generator,
        DispatchStatus::Emitted,
        DispatchStrategy::TypedGenerator,
        maturity,
        None,
        Some("typed generator input ready; apply.rs owns writeback".to_string()),
    )
}

/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
fn structural_scaffold(
    generator: &'static str,
    message: &'static str,
) -> (
    &'static str,
    DispatchStatus,
    DispatchStrategy,
    DispatchMaturity,
    Option<String>,
    Option<String>,
) {
    (
        generator,
        DispatchStatus::Emitted,
        DispatchStrategy::StructuralScaffold,
        DispatchMaturity::StructuralGenerator,
        None,
        Some(message.to_string()),
    )
}

/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
fn handwrite_gap(
    section_type: SectionType,
    reason: &'static str,
) -> (
    &'static str,
    DispatchStatus,
    DispatchStrategy,
    DispatchMaturity,
    Option<String>,
    Option<String>,
) {
    let section_name = section_type.as_str();
    (
        "none",
        DispatchStatus::NoGenerator,
        DispatchStrategy::HandwriteGap,
        DispatchMaturity::HandwriteGap,
        Some(format!("typed-generator:{section_name}")),
        Some(reason.to_string()),
    )
}

/// Walk the typed payloads of every section and collect paths in
/// `Changes.path` that no other typed payload references. Replaces the
/// substring heuristic in `validate_td::check_orphan_changes_targets`
/// (R4) — Stage 2B will graduate this into the validator proper.
///
/// @spec projects/agentic-workflow/tech-design/core/generate/from-td-ast.md#logic
fn collect_orphan_changes_paths(td_ast: &TDAst) -> Vec<String> {
    use std::collections::BTreeSet;

    let mut changes_paths: Vec<String> = Vec::new();
    let mut referenced: BTreeSet<String> = BTreeSet::new();

    for section in &td_ast.sections {
        match (&section.body, section.section_type) {
            (TypedBody::JsonSchema(p), SectionType::Changes) => {
                if let Some(arr) = p.extra.get("changes").and_then(|v| v.as_sequence()) {
                    for entry in arr {
                        if let Some(path) = entry.get("path").and_then(|v| v.as_str()) {
                            changes_paths.push(path.to_string());
                        }
                    }
                }
            }
            (TypedBody::JsonSchema(p), _) => {
                for name in p.definitions.keys() {
                    referenced.insert(name.clone());
                }
                for name in p.defs.keys() {
                    referenced.insert(name.clone());
                }
            }
            (TypedBody::OpenRpc(p), _) => {
                for m in &p.methods {
                    referenced.insert(m.name.clone());
                }
            }
            (TypedBody::CliManifest(p), _) => {
                for c in &p.commands {
                    referenced.insert(c.name.clone());
                }
            }
            _ => {}
        }
    }

    changes_paths
        .into_iter()
        .filter(|path| {
            // A changes path is orphan if no typed payload elsewhere mentions it.
            !referenced.iter().any(|sym| path.contains(sym))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(section_type: SectionType, body: TypedBody) -> TDSection {
        TDSection {
            section_type,
            lang: "yaml".to_string(),
            body,
            line_start: 1,
            line_end: 2,
            content_hash: None,
        }
    }

    #[test]
    fn dispatch_report_default_is_empty() {
        let r = DispatchReport::default();
        assert!(r.outcomes.is_empty());
        assert!(r.orphan_changes_paths.is_empty());
    }

    #[test]
    fn dispatch_status_round_trips_via_json() {
        let s = DispatchStatus::LegacyFallback;
        let j = serde_json::to_string(&s).unwrap();
        assert_eq!(j, "\"legacy_fallback\"");
        let back: DispatchStatus = serde_json::from_str(&j).unwrap();
        assert_eq!(back, DispatchStatus::LegacyFallback);
    }

    #[test]
    fn empty_tdast_yields_empty_report() {
        let td = TDAst {
            frontmatter: serde_yaml::Value::Null,
            sections: Vec::new(),
        };
        let ctx = DispatchCtx {
            spec_path: std::path::PathBuf::from("/dev/null"),
            spec_ref_prefix: String::new(),
            target_lang: None,
        };
        let report = dispatch_from_tdast(&td, &ctx);
        assert!(report.outcomes.is_empty());
        assert!(report.orphan_changes_paths.is_empty());
    }

    #[test]
    fn schema_and_cli_sections_route_to_mixed_typed_strategies() {
        let td = TDAst {
            frontmatter: serde_yaml::Value::Null,
            sections: vec![
                section(
                    SectionType::Schema,
                    TypedBody::JsonSchema(crate::td_ast::payloads::JsonSchemaPayload::default()),
                ),
                section(
                    SectionType::Cli,
                    TypedBody::CliManifest(crate::td_ast::payloads::CliManifestPayload::default()),
                ),
            ],
        };
        let ctx = DispatchCtx {
            spec_path: std::path::PathBuf::from("demo.md"),
            spec_ref_prefix: "demo.md".to_string(),
            target_lang: Some("rs".to_string()),
        };
        let report = dispatch_from_tdast(&td, &ctx);

        assert_eq!(report.outcomes.len(), 2);
        assert_eq!(report.outcomes[0].status, DispatchStatus::Emitted);
        assert_eq!(
            report.outcomes[0].strategy,
            DispatchStrategy::TypedGenerator
        );
        assert_eq!(
            report.outcomes[0].maturity,
            DispatchMaturity::SemanticGenerator
        );
        assert_eq!(report.outcomes[1].status, DispatchStatus::Emitted);
        assert_eq!(
            report.outcomes[1].strategy,
            DispatchStrategy::StructuralScaffold
        );
        assert_eq!(
            report.outcomes[1].maturity,
            DispatchMaturity::StructuralGenerator
        );
    }

    #[test]
    fn unsupported_typed_section_reports_stable_handwrite_gap() {
        let td = TDAst {
            frontmatter: serde_yaml::Value::Null,
            sections: vec![section(
                SectionType::RestApi,
                TypedBody::Unsupported("raw".to_string()),
            )],
        };
        let ctx = DispatchCtx {
            spec_path: std::path::PathBuf::from("demo.md"),
            spec_ref_prefix: "demo.md".to_string(),
            target_lang: Some("rs".to_string()),
        };
        let report = dispatch_from_tdast(&td, &ctx);

        let outcome = &report.outcomes[0];
        assert_eq!(outcome.status, DispatchStatus::NoGenerator);
        assert_eq!(outcome.strategy, DispatchStrategy::HandwriteGap);
        assert_eq!(outcome.maturity, DispatchMaturity::HandwriteGap);
        assert_eq!(outcome.gap_id.as_deref(), Some("typed-generator:rest-api"));
    }

    #[test]
    fn rust_source_unit_dispatch_routes_as_structural_generator() {
        let td = TDAst {
            frontmatter: serde_yaml::Value::Null,
            sections: vec![section(
                SectionType::RustSourceUnit,
                TypedBody::RustSourceUnit(
                    crate::generate::rust_source_unit::parse("pub fn demo() {}")
                        .expect("rust source parses"),
                ),
            )],
        };
        let ctx = DispatchCtx {
            spec_path: std::path::PathBuf::from("demo.md"),
            spec_ref_prefix: "demo.md".to_string(),
            target_lang: Some("rs".to_string()),
        };

        let report = dispatch_from_tdast(&td, &ctx);
        let outcome = &report.outcomes[0];
        assert_eq!(outcome.status, DispatchStatus::Emitted);
        assert_eq!(outcome.generator, "rust-source-unit");
        assert_eq!(outcome.strategy, DispatchStrategy::StructuralScaffold);
        assert_eq!(outcome.maturity, DispatchMaturity::StructuralGenerator);
        assert!(!outcome.source_backed);
        assert!(outcome.gap_id.is_none());
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#logic
pub fn enter() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-split_fence
    // TODO: Implement process step: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]
    todo!("process: Locate inner ``---`` markers; split raw into [frontmatter_raw, rendered_body]");
    // Decision: Frontmatter markers found?
    if todo!("decision: Frontmatter markers found?")
    /* no */
    {
        todo!("terminal: Err(MP-FM-001 missing-frontmatter)");
    } else
    /* yes */
    {
        // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-parse_yaml
        // TODO: Implement process step: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)
        todo!("process: serde_yaml::from_str::<MermaidPlusFrontmatter>(frontmatter_raw) (typed deserializer dispatches on `kind`)");
        // Decision: YAML parses into typed frontmatter?
        if todo!("decision: YAML parses into typed frontmatter?")
        /* no */
        {
            todo!("terminal: Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)");
        } else
        /* yes */
        {
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-derive_id
            // TODO: Implement process step: id = frontmatter.id().unwrap_or(format!("{section_type}:{line_start}"))
            todo!("process: id = frontmatter.id().unwrap_or(format!(\"{{section_type}}:{{line_start}}\"))");
            // SPEC-REF: projects/agentic-workflow/tech-design/core/specs/mermaid-plus-ast-and-ir.md#mermaid-plus-ast-and-ir-pipeline-build_block
            // TODO: Implement process step: Construct MermaidPlusBlockTyped { id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }
            todo!("process: Construct MermaidPlusBlockTyped {{ id, span, frontmatter, frontmatter_raw, rendered_body, diagnostics: [] }}");
            todo!("terminal: Ok(MermaidPlusBlockTyped) — parse layer done");
        }
    }
    // Terminal: err_no_fence -> Err(MP-FM-001 missing-frontmatter)
    // Terminal: err_yaml -> Err(MP-FM-002 invalid-yaml | MP-FM-003 unknown-kind | MP-FM-004 schema-mismatch)
    // Terminal: lower_unsupp -> (None, [info: MP-LO-999 unsupported-ir-family]) — mindmap / dependency carry no IR yet
    // Terminal: ret_block -> Ok(MermaidPlusBlockTyped) — parse layer done
    // Terminal: ret_ir -> (Some(IRFamily), diagnostics)
}
// CODEGEN-END
