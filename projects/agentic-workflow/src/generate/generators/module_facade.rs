//! Module-facade codegen primitive.
//!
//! Emits `pub mod <name>;` declarations and `pub use <name>::<Symbol>;`
//! re-exports for a Rust module hierarchy described by the `exports:` list
//! in a spec change entry. Replaces the hand-written boilerplate that
//! previously sat under `<HANDWRITE gap="missing-generator:module-facade">`
//! markers in files like `projects/agentic-workflow/src/td_ast/mod.rs`.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// A single module-facade export pair: one pub mod declaration and one or more pub use re-exports (R1, R2).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportEntry {
    /// Module name used in pub mod <module>; and pub use <module>::...
    pub module: String,
    /// Symbols to re-export. Each emits one pub use <module>::<Symbol>; line.
    pub symbols: Vec<String>,
}

/// Result of running the module-facade generator. Contains the generated lines to be inserted inside CODEGEN-BEGIN/CODEGEN-END markers (R4).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleFacadeOutput {
    /// Generated source lines (pub mod + pub use statements).
    pub lines: Vec<String>,
    /// SPEC-REF anchor string for the CODEGEN marker header.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_ref: Option<String>,
}

/// Input descriptor for the module-facade generator, sourced from the exports: field of a spec change entry (R1, R2, R3).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleFacadeSpec {
    /// Optional raw module preamble emitted before pub use/pub mod lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preamble: Option<String>,
    /// External pub-use paths emitted as `pub use <path>;`.
    #[serde(default)]
    pub pub_uses: Vec<String>,
    /// Ordered list of module-symbol pairs. Empty list emits no output (R3).
    #[serde(default)]
    pub exports: Vec<ExportEntry>,
}
// CODEGEN-END

#[cfg(test)]
mod tests {
    use super::*;

    /// R3 — empty exports list produces empty output.
    #[test]
    fn empty_exports_produces_empty_output() {
        let spec = ModuleFacadeSpec::default();
        let out = run_module_facade(&spec, None);
        assert!(out.lines.is_empty());
        assert!(out.spec_ref.is_none());
    }

    /// Module preamble + external pub-use paths are emitted before module facade declarations.
    #[test]
    fn preamble_and_external_pub_uses_precede_exports() {
        let spec = ModuleFacadeSpec {
            preamble: Some("//! docs\n// comment".to_string()),
            pub_uses: vec!["cap_core::{client, paths}".to_string()],
            exports: vec![ExportEntry {
                module: "cli".into(),
                symbols: Vec::new(),
            }],
        };
        let out = run_module_facade(&spec, None);
        assert_eq!(
            out.lines,
            vec![
                "//! docs".to_string(),
                "// comment".to_string(),
                "".to_string(),
                "pub use cap_core::{client, paths};".to_string(),
                "".to_string(),
                "pub mod cli;".to_string(),
            ],
        );
    }

    /// R1 + R2 — single entry with single symbol emits one pub mod + one pub use.
    #[test]
    fn single_module_single_symbol() {
        let spec = ModuleFacadeSpec {
            exports: vec![ExportEntry {
                module: "types".into(),
                symbols: vec!["TDAst".into()],
            }],
            ..Default::default()
        };
        let out = run_module_facade(&spec, None);
        assert_eq!(
            out.lines,
            vec![
                "pub mod types;".to_string(),
                "pub use types::TDAst;".to_string()
            ],
        );
    }

    /// R2 — single entry with multiple symbols emits all pub use lines.
    #[test]
    fn single_module_multi_symbol() {
        let spec = ModuleFacadeSpec {
            exports: vec![ExportEntry {
                module: "parse".into(),
                symbols: vec!["parse_td".into(), "parse_td_str".into()],
            }],
            ..Default::default()
        };
        let out = run_module_facade(&spec, None);
        assert_eq!(
            out.lines,
            vec![
                "pub mod parse;".to_string(),
                "pub use parse::parse_td;".to_string(),
                "pub use parse::parse_td_str;".to_string(),
            ],
        );
    }

    /// R1 + R2 — multiple entries each emit their own pub mod + pub use lines.
    #[test]
    fn multi_module() {
        let spec = ModuleFacadeSpec {
            exports: vec![
                ExportEntry {
                    module: "types".into(),
                    symbols: vec!["TDAst".into()],
                },
                ExportEntry {
                    module: "parse".into(),
                    symbols: vec!["parse_td".into()],
                },
            ],
            ..Default::default()
        };
        let out = run_module_facade(&spec, None);
        assert_eq!(
            out.lines,
            vec![
                "pub mod types;".to_string(),
                "pub use types::TDAst;".to_string(),
                "pub mod parse;".to_string(),
                "pub use parse::parse_td;".to_string(),
            ],
        );
    }

    /// R4 — spec_ref threads through to output for SPEC-REF marker emission.
    #[test]
    fn spec_ref_threads_through() {
        let spec = ModuleFacadeSpec {
            exports: vec![ExportEntry {
                module: "x".into(),
                symbols: vec!["Y".into()],
            }],
            ..Default::default()
        };
        let out = run_module_facade(&spec, Some("path/to/spec.md#schema".to_string()));
        assert_eq!(out.spec_ref.as_deref(), Some("path/to/spec.md#schema"));
    }

    /// Snapshot: representative two-module fixture exercising the full
    /// pub-mod-then-pub-use ordering contract (R1 + R2).
    #[test]
    fn snapshot_representative_fixture() {
        let spec = ModuleFacadeSpec {
            exports: vec![
                ExportEntry {
                    module: "entities".into(),
                    symbols: vec!["EntityRef".into(), "SectionEntities".into()],
                },
                ExportEntry {
                    module: "types".into(),
                    symbols: vec![
                        "MermaidPlusPayload".into(),
                        "SectionKind".into(),
                        "TDAst".into(),
                    ],
                },
            ],
            ..Default::default()
        };
        let out = run_module_facade(&spec, None);
        let rendered = out.lines.join("\n");
        assert_eq!(
            rendered,
            "pub mod entities;\n\
             pub use entities::EntityRef;\n\
             pub use entities::SectionEntities;\n\
             pub mod types;\n\
             pub use types::MermaidPlusPayload;\n\
             pub use types::SectionKind;\n\
             pub use types::TDAst;",
        );
    }
}
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#logic
// CODEGEN-BEGIN
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/module-facade.md#logic
pub fn run_module_facade(spec: &ModuleFacadeSpec, spec_ref: Option<String>) -> ModuleFacadeOutput {
    let mut lines: Vec<String> = Vec::new();
    if let Some(preamble) = spec.preamble.as_deref() {
        lines.extend(preamble.trim_end().lines().map(str::to_string));
    }
    if !lines.is_empty() && (!spec.pub_uses.is_empty() || !spec.exports.is_empty()) {
        lines.push(String::new());
    }
    for path in &spec.pub_uses {
        lines.push(format!("pub use {path};"));
    }
    if !spec.pub_uses.is_empty() && !spec.exports.is_empty() {
        lines.push(String::new());
    }
    for entry in &spec.exports {
        lines.push(format!("pub mod {};", entry.module));
        for sym in &entry.symbols {
            lines.push(format!("pub use {}::{};", entry.module, sym));
        }
    }
    ModuleFacadeOutput { lines, spec_ref }
}
// CODEGEN-END
