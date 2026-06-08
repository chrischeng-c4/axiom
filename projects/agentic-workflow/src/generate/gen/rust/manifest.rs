// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/manifest.md#source
// CODEGEN-BEGIN
//! Manifest generator — Cargo.toml `[dependencies]` fragment from a `manifest`
//! section.
//!
//! Section contract (YAML inside the spec's `## Manifest` section):
//!
//! ```yaml
//! dependencies:
//!   - { name: serde, spec: workspace, features: [derive] }
//!   - { name: thiserror, spec: workspace }
//!   - { name: once_cell, spec: version, version: "1.20" }
//!   - { name: cclab-mamba-registry, spec: path, path: "../../crates/cclab-mamba-registry" }
//! ```
//!
//! Output is a TOML fragment (one `key = value` per line) suitable for wrapping
//! inside a CODEGEN block under `[dependencies]` in the target `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! # SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/spec.md#manifest
//! # CODEGEN-BEGIN
//! serde = { workspace = true, features = ["derive"] }
//! thiserror.workspace = true
//! once_cell = { version = "1.20" }
//! cclab-mamba-registry = { path = "../../crates/cclab-mamba-registry" }
//! # CODEGEN-END
//! ```
//!
//! The CODEGEN block markers use `#` (TOML-compatible); the rest of the manifest
//! — including other `[dependencies]` entries — stays hand-editable outside the
//! block.

use serde::Serialize;
use serde_yaml::Value;

use crate::generate::engine::TemplateEngine;

const TPL_CARGO_DEPS: &str = include_str!("templates/manifest/cargo_deps.tera");

#[derive(Debug, Clone)]
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/manifest.md#source
pub struct ManifestGenOutput {
    /// The rendered `key = value` fragment (no `[dependencies]` header, no
    /// CODEGEN markers — apply.rs wraps the markers).
    pub code: String,
    /// Whether the generator produced content. `false` when the `manifest`
    /// section is absent or empty.
    pub emitted: bool,
}

/// Render a Cargo.toml `[dependencies]` fragment from the `## Manifest` YAML
/// section of a spec file.
///
/// Returns `emitted: false` when the spec has no `## Manifest` block or the
/// `dependencies` list is empty.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/manifest.md#source
pub fn generate_manifest(spec_content: &str) -> ManifestGenOutput {
    let Some(yaml_text) = crate::generate::apply::extract_section_yaml(spec_content, "Manifest")
    else {
        return ManifestGenOutput {
            code: String::new(),
            emitted: false,
        };
    };
    let yaml: Value = match serde_yaml::from_str(&yaml_text) {
        Ok(v) => v,
        Err(_) => {
            return ManifestGenOutput {
                code: String::new(),
                emitted: false,
            }
        }
    };

    let deps = parse_dependencies(&yaml);
    if deps.is_empty() {
        return ManifestGenOutput {
            code: String::new(),
            emitted: false,
        };
    }

    let mut engine = TemplateEngine::empty();
    engine
        .add_template("cargo_deps.tera", TPL_CARGO_DEPS)
        .expect("cargo_deps.tera parse");

    let ctx = ManifestContext { dependencies: deps };
    let code = engine
        .render("cargo_deps.tera", &ctx)
        .expect("cargo_deps.tera render");
    ManifestGenOutput {
        code: code.trim_end().to_string(),
        emitted: true,
    }
}

// ── context ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ManifestContext {
    dependencies: Vec<DependencyContext>,
}

#[derive(Debug, Serialize)]
struct DependencyContext {
    name: String,
    /// "workspace" | "version" | "path"
    kind: String,
    version: Option<String>,
    path: Option<String>,
    features: Vec<String>,
}

fn parse_dependencies(yaml: &Value) -> Vec<DependencyContext> {
    let Some(seq) = yaml.get("dependencies").and_then(|v| v.as_sequence()) else {
        return Vec::new();
    };
    seq.iter()
        .filter_map(|entry| {
            let m = entry.as_mapping()?;
            let name = m.get("name").and_then(|v| v.as_str())?.to_string();
            let spec = m
                .get("spec")
                .and_then(|v| v.as_str())
                .unwrap_or("workspace");
            let version = m.get("version").and_then(|v| v.as_str()).map(String::from);
            let path = m.get("path").and_then(|v| v.as_str()).map(String::from);
            let features = m
                .get("features")
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let kind = match spec {
                "workspace" => "workspace",
                "version" => "version",
                "path" => "path",
                _ => "workspace",
            }
            .to_string();
            Some(DependencyContext {
                name,
                kind,
                version,
                path,
                features,
            })
        })
        .collect()
}

// ── tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_manifest_section_emits_nothing() {
        let spec = "## Overview\nNo manifest.\n";
        let out = generate_manifest(spec);
        assert!(!out.emitted);
        assert!(out.code.is_empty());
    }

    #[test]
    fn workspace_dep_no_features_uses_shorthand_workspace() {
        let spec = r#"
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: thiserror, spec: workspace }
```
"#;
        let out = generate_manifest(spec);
        assert!(out.emitted);
        assert_eq!(out.code.trim(), "thiserror.workspace = true");
    }

    #[test]
    fn workspace_dep_with_features_uses_table_form() {
        let spec = r#"
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: serde, spec: workspace, features: [derive] }
```
"#;
        let out = generate_manifest(spec);
        assert!(out.emitted);
        assert_eq!(
            out.code.trim(),
            "serde = { workspace = true, features = [\"derive\"] }"
        );
    }

    #[test]
    fn version_dep_renders_inline_table() {
        let spec = r#"
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: once_cell, spec: version, version: "1.20" }
```
"#;
        let out = generate_manifest(spec);
        assert_eq!(out.code.trim(), "once_cell = { version = \"1.20\" }");
    }

    #[test]
    fn path_dep_renders_inline_table() {
        let spec = r#"
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: cclab-mamba-registry, spec: path, path: "../../crates/cclab-mamba-registry" }
```
"#;
        let out = generate_manifest(spec);
        assert_eq!(
            out.code.trim(),
            "cclab-mamba-registry = { path = \"../../crates/cclab-mamba-registry\" }"
        );
    }

    #[test]
    fn multiple_deps_emit_one_per_line() {
        let spec = r#"
## Manifest
<!-- type: manifest lang: yaml -->

```yaml
dependencies:
  - { name: serde, spec: workspace, features: [derive] }
  - { name: thiserror, spec: workspace }
  - { name: linkme, spec: workspace }
```
"#;
        let out = generate_manifest(spec);
        let lines: Vec<&str> = out.code.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("serde"));
        assert!(lines[1].contains("thiserror"));
        assert!(lines[2].contains("linkme"));
    }
}

// CODEGEN-END
