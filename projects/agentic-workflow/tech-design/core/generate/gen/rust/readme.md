---
id: sdd-generate-gen-rust-readme
fill_sections: [overview, schema, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# README Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/readme.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ReadmeGenOutput` | projects/agentic-workflow/src/generate/gen/rust/readme.rs | struct | pub | 39 |  |
| `SymbolEntry` | projects/agentic-workflow/src/generate/gen/rust/readme.rs | struct | pub | 29 |  |
| `generate_readme_symbols` | projects/agentic-workflow/src/generate/gen/rust/readme.rs | function | pub | 55 | generate_readme_symbols(tech_design_dir: &Path, project_root: &Path) -> ReadmeGenOutput |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SymbolEntry:
    type: object
    required: [symbol, spec_path]
    description: |
      One registered symbol row in the README's symbols table.
    properties:
      symbol:
        type: string
        description: "The symbol name."
      spec_path:
        type: string
        description: "Spec path, relative to project_root. Rendered as a markdown link."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  ReadmeGenOutput:
    type: object
    required: [code, emitted, symbols]
    description: |
      Full output of the README symbols generator.
    properties:
      code:
        type: string
        description: "Rendered markdown table (no CODEGEN markers — apply.rs wraps them)."
      emitted:
        type: boolean
        description: "True when at least one symbol was found."
      symbols:
        type: array
        items: { type: object }
        x-rust-type: "Vec<SymbolEntry>"
        description: "The symbol entries collected, exposed for tests / diagnostics."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/readme.rs -->
~~~rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/readme.md#source
// CODEGEN-BEGIN
//! README generator — aggregates `x-mamba-binding.symbol` entries from every
//! spec under a project's `.aw/tech-design/<project>/` directory into a
//! single Registered Symbols markdown table.
//!
//! Output lives inside a markdown CODEGEN block in the project's `README.md`
//! (e.g. `projects/mamba/mambalibs/httpkit/README.md`). The block is anchored under a `##
//! Registered symbols` H2 header; first-time insertion seeds both the header
//! and the block.
//!
//! Unlike the schema / manifest generators, this one is a **crate-level
//! aggregator** — it doesn't consume a single spec's `## Readme` section.
//! Instead, it scans the tech-design directory that matches the README's
//! project and pulls `x-mamba-binding.symbol` from every spec it finds.

use serde_yaml::Value;
use std::path::Path;

use crate::generate::engine::TemplateEngine;

const TPL_SYMBOL_TABLE: &str = include_str!("templates/readme/symbol_table.tera");

use serde::Serialize;

/// One registered symbol row in the README's symbols table.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/readme.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct SymbolEntry {
    /// The symbol name.
    pub symbol: String,
    /// Spec path, relative to project_root. Rendered as a markdown link.
    pub spec_path: String,
}

/// Full output of the README symbols generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/readme.md#schema
#[derive(Debug, Clone)]
pub struct ReadmeGenOutput {
    /// Rendered markdown table (no CODEGEN markers — apply.rs wraps them).
    pub code: String,
    /// True when at least one symbol was found.
    pub emitted: bool,
    /// The symbol entries collected, exposed for tests / diagnostics.
    pub symbols: Vec<SymbolEntry>,
}

/// Scan a tech-design directory for `x-mamba-binding.symbol` entries and
/// render a sorted markdown table.
///
/// `tech_design_dir` is the absolute path to the project's tech-design
/// directory (e.g. `<root>/.aw/tech-design/projects/httpkit/`).
/// `project_root` is the repo root used to render relative spec paths.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/readme.md#source
pub fn generate_readme_symbols(tech_design_dir: &Path, project_root: &Path) -> ReadmeGenOutput {
    let mut symbols = collect_symbols(tech_design_dir, project_root);
    symbols.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    symbols.dedup_by(|a, b| a.symbol == b.symbol);

    if symbols.is_empty() {
        return ReadmeGenOutput {
            code: String::new(),
            emitted: false,
            symbols,
        };
    }

    let mut engine = TemplateEngine::empty();
    engine
        .add_template("symbol_table.tera", TPL_SYMBOL_TABLE)
        .expect("symbol_table.tera parse");

    #[derive(Serialize)]
    struct Ctx<'a> {
        symbols: &'a [SymbolEntry],
    }
    let code = engine
        .render("symbol_table.tera", &Ctx { symbols: &symbols })
        .expect("symbol_table.tera render")
        .trim_end()
        .to_string();

    ReadmeGenOutput {
        code,
        emitted: true,
        symbols,
    }
}

fn collect_symbols(tech_design_dir: &Path, project_root: &Path) -> Vec<SymbolEntry> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(tech_design_dir) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let symbols = extract_mamba_symbols(&content);
        if symbols.is_empty() {
            continue;
        }
        let spec_path = path
            .strip_prefix(project_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        for symbol in symbols {
            out.push(SymbolEntry {
                symbol,
                spec_path: spec_path.clone(),
            });
        }
    }
    out
}

/// Pull every `x-mamba-binding.symbol` from the spec's schema-section YAML.
///
/// Handles two layouts:
///
/// - **Top-level binding** — the schema itself has `x-mamba-binding` (the
///   single-type convention used by `http-exception.md`).
/// - **Per-definition binding** — the schema uses JSON Schema `definitions:`
///   and each definition carries its own `x-mamba-binding` (the multi-type
///   convention used by `health.md`).
///
/// Returns an empty `Vec` when no bindings are declared or the YAML fails to
/// parse.
fn extract_mamba_symbols(spec_content: &str) -> Vec<String> {
    let Some(yaml_text) = crate::generate::apply::extract_section_yaml(spec_content, "Schema")
    else {
        return Vec::new();
    };
    let Ok(yaml): Result<Value, _> = serde_yaml::from_str(&yaml_text) else {
        return Vec::new();
    };

    let mut symbols: Vec<String> = Vec::new();

    if let Some(sym) = yaml
        .get("x-mamba-binding")
        .and_then(|b| b.get("symbol"))
        .and_then(|v| v.as_str())
    {
        symbols.push(sym.to_string());
    }

    if let Some(defs) = yaml.get("definitions").and_then(|v| v.as_mapping()) {
        for (_name, def) in defs {
            if let Some(sym) = def
                .get("x-mamba-binding")
                .and_then(|b| b.get("symbol"))
                .and_then(|v| v.as_str())
            {
                symbols.push(sym.to_string());
            }
        }
    }

    symbols
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&p, content).unwrap();
    }

    fn spec_with_symbol(symbol: &str) -> String {
        format!(
            concat!(
                "---\n",
                "id: dummy\n",
                "fill_sections: [schema, changes]\n",
                "---\n\n",
                "## Schema\n",
                "<!-- type: schema lang: yaml -->\n\n",
                "```yaml\n",
                "title: Dummy\n",
                "type: object\n",
                "x-mamba-binding:\n",
                "  symbol: {}\n",
                "  extern_fn: dummy_new\n",
                "  signature: \"dummy()\"\n",
                "```\n\n",
                "## Changes\n",
                "<!-- type: changes lang: yaml -->\n\n",
                "```yaml\n",
                "changes: []\n",
                "```\n",
            ),
            symbol,
        )
    }

    #[test]
    fn empty_dir_produces_no_emission() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".aw/tech-design/projects/httpkit");
        std::fs::create_dir_all(&dir).unwrap();
        let out = generate_readme_symbols(&dir, tmp.path());
        assert!(!out.emitted);
        assert!(out.symbols.is_empty());
    }

    #[test]
    fn aggregates_symbols_from_multiple_specs_sorted() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".aw/tech-design/projects/httpkit");
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/http-exception.md",
            &spec_with_symbol("HTTPException"),
        );
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/response.md",
            &spec_with_symbol("Response"),
        );
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/app.md",
            &spec_with_symbol("App"),
        );
        // A spec without x-mamba-binding — should be skipped silently.
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/nonbinding.md",
            "## Schema\n<!-- type: schema lang: yaml -->\n\n```yaml\ntitle: Nope\n```\n",
        );

        let out = generate_readme_symbols(&dir, tmp.path());
        assert!(out.emitted);
        assert_eq!(out.symbols.len(), 3);
        // Sorted alphabetically
        assert_eq!(out.symbols[0].symbol, "App");
        assert_eq!(out.symbols[1].symbol, "HTTPException");
        assert_eq!(out.symbols[2].symbol, "Response");
        // Spec paths are relative to project_root
        assert_eq!(
            out.symbols[0].spec_path,
            ".aw/tech-design/projects/httpkit/app.md"
        );
        // Table format
        assert!(out.code.contains("| Symbol | Spec |"));
        assert!(out.code.contains("| `HTTPException` |"));
        assert!(out
            .code
            .contains("[.aw/tech-design/projects/httpkit/app.md]"));
    }

    #[test]
    fn dedupes_duplicate_symbol_names() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join(".aw/tech-design/projects/httpkit");
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/a.md",
            &spec_with_symbol("Dup"),
        );
        write(
            tmp.path(),
            ".aw/tech-design/projects/httpkit/b.md",
            &spec_with_symbol("Dup"),
        );
        let out = generate_readme_symbols(&dir, tmp.path());
        assert_eq!(out.symbols.len(), 1);
    }
}

// CODEGEN-END
~~~

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/readme.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete README symbol-table generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Two pure data carriers; mixed-derive shape covered.
- [schema] Both definitions well-formed; `Vec<SymbolEntry>` uses `x-rust-type` to skip Option wrap.
- [changes] Standard codegen+hand-written split with both structs in `replaces`.
