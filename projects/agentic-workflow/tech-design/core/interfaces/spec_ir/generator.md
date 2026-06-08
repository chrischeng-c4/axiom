---
id: sdd-spec-ir-generator
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Spec IR interfaces drive code artifact generation from TD/spec manifests in the TD/CB lifecycle."
---

# Spec IR Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/spec_ir/generator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ApiSpecEntry` | projects/agentic-workflow/src/spec_ir/generator.rs | struct | pub | 49 |  |
| `DiagramEntry` | projects/agentic-workflow/src/spec_ir/generator.rs | struct | pub | 37 |  |
| `GenerateResult` | projects/agentic-workflow/src/spec_ir/generator.rs | struct | pub | 59 |  |
| `SpecIrInput` | projects/agentic-workflow/src/spec_ir/generator.rs | struct | pub | 17 |  |
| `generate` | projects/agentic-workflow/src/spec_ir/generator.rs | function | pub | 109 | generate(change_dir: &Path, input: &SpecIrInput) -> crate::Result<GenerateResult> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  SpecIrInput:
    type: object
    required: [spec_id, change_id, spec_group, source_file, tags, diagrams, api_spec]
    description: |
      Input for generating YAML IR from a spec.
    properties:
      spec_id:
        type: string
        description: "Spec identifier (e.g. \"user-service-api\")."
      change_id:
        type: string
        description: "Change ID this spec belongs to."
      spec_group:
        type: string
        x-rust-type: "Option<String>"
        description: "Spec group (e.g. \"sdd\")."
      source_file:
        type: string
        x-rust-type: "Option<String>"
        description: "Source spec file path (relative to project root)."
      tags:
        type: array
        items: { type: string }
        x-rust-type: "Vec<String>"
        description: "Tags from the spec."
      diagrams:
        type: array
        items: { $ref: "#/definitions/DiagramEntry" }
        x-rust-type: "Vec<DiagramEntry>"
        description: "Diagram definitions from the spec."
      api_spec:
        type: object
        x-rust-type: "Option<ApiSpecEntry>"
        description: "API spec content, if any."
    x-rust-struct:
      derive: [Debug, Clone]

  DiagramEntry:
    type: object
    required: [diagram_type, title, content]
    description: A diagram from a spec.
    properties:
      diagram_type:
        type: string
        description: "Diagram type (flowchart, sequence, class, erd, etc.)."
      title:
        type: string
        description: "Diagram title."
      content:
        type: object
        x-rust-type: "serde_yaml::Value"
        description: "Rendered content (Mermaid code or structured data)."
    x-rust-struct:
      derive: [Debug, Clone]

  ApiSpecEntry:
    type: object
    required: [api_type, content]
    description: An API spec from a spec document.
    properties:
      api_type:
        type: string
        description: "API spec type (openapi-3.1, asyncapi-2.6, etc.)."
      content:
        type: object
        x-rust-type: "serde_yaml::Value"
        description: "The spec content as YAML value."
    x-rust-struct:
      derive: [Debug, Clone]

  GenerateResult:
    type: object
    required: [files]
    description: Result of generating YAML IR files.
    properties:
      files:
        type: array
        items: { type: string }
        x-rust-type: "Vec<std::path::PathBuf>"
        description: "Paths to the generated YAML IR files."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/spec_ir/generator.rs -->
```rust
//! Spec IR generator: produce YAML manifest files from spec content.
//!
//! Implements genesis-spec-generation spec:
//! - R1: YAML Generation — create YAML IR files alongside markdown specs
//! - R2: File Naming — `<group>_<name>_<kind>.yaml` pattern
//! - R3: Content Mapping — map diagrams/API defs into SpecManifest

use std::path::Path;

use super::{ManifestMetadata, SpecKind, SpecManifest};

/// Input for generating YAML IR from a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#schema
#[derive(Debug, Clone)]
pub struct SpecIrInput {
    /// Spec identifier (e.g. "user-service-api").
    pub spec_id: String,
    /// Change ID this spec belongs to.
    pub change_id: String,
    /// Spec group (e.g. "sdd").
    pub spec_group: Option<String>,
    /// Source spec file path (relative to project root).
    pub source_file: Option<String>,
    /// Tags from the spec.
    pub tags: Vec<String>,
    /// Diagram definitions from the spec.
    pub diagrams: Vec<DiagramEntry>,
    /// API spec content, if any.
    pub api_spec: Option<ApiSpecEntry>,
}

/// A diagram from a spec.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#schema
#[derive(Debug, Clone)]
pub struct DiagramEntry {
    /// Diagram type (flowchart, sequence, class, erd, etc.).
    pub diagram_type: String,
    /// Diagram title.
    pub title: String,
    /// Rendered content (Mermaid code or structured data).
    pub content: serde_yaml::Value,
}

/// An API spec from a spec document.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#schema
#[derive(Debug, Clone)]
pub struct ApiSpecEntry {
    /// API spec type (openapi-3.1, asyncapi-2.6, etc.).
    pub api_type: String,
    /// The spec content as YAML value.
    pub content: serde_yaml::Value,
}

/// Result of generating YAML IR files.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#schema
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Paths to the generated YAML IR files.
    pub files: Vec<std::path::PathBuf>,
}
/// Map a diagram type string to a SpecKind.
///
/// Returns None for diagram types that don't have a SpecKind mapping.
/// Currently unmapped types (mindmap, journey) are skipped — extend
/// SpecKind when Lens adds generators for these.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#source
fn map_diagram_to_kind(diagram_type: &str) -> Option<SpecKind> {
    match diagram_type.to_lowercase().as_str() {
        "flowchart" | "flowchart_plus" | "flowchartplus" => Some(SpecKind::FlowchartPlus),
        "sequence" | "sequence_plus" | "sequenceplus" => Some(SpecKind::SequencePlus),
        "class" | "class_plus" | "classplus" => Some(SpecKind::ClassPlus),
        "erd" | "erd_plus" | "erdplus" => Some(SpecKind::ErdPlus),
        "requirement" | "requirement_plus" | "requirementplus" => Some(SpecKind::RequirementPlus),
        // State diagrams map to FlowchartPlus (control-flow family)
        "state" => Some(SpecKind::FlowchartPlus),
        // mindmap, journey: no SpecKind yet — extend registry when needed
        _ => None,
    }
}

/// Map an API spec type string to a SpecKind.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#source
fn map_api_to_kind(api_type: &str) -> Option<SpecKind> {
    match api_type.to_lowercase().replace('-', "_").as_str() {
        "openapi_3.1"
        | "openapi"
        | "asyncapi_2.6"
        | "asyncapi"
        | "openrpc_1.3"
        | "openrpc"
        | "serverless_workflow_0.8"
        | "serverless_workflow"
        | "json_schema" => Some(SpecKind::Api),
        _ => None,
    }
}

/// Generate YAML IR manifest files from spec input (R1).
///
/// Creates one manifest per diagram kind and one for API specs.
/// Files are written to `<change_dir>/spec_ir/` using the canonical
/// naming pattern (R2). Spec content is mapped into the manifest
/// payload (R3).
///
/// Returns the list of generated file paths.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/generator.md#source
pub fn generate(change_dir: &Path, input: &SpecIrInput) -> crate::Result<GenerateResult> {
    let spec_ir_dir = change_dir.join("spec_ir");
    std::fs::create_dir_all(&spec_ir_dir)?;

    let mut files = Vec::new();

    let base_metadata = ManifestMetadata {
        name: input.spec_id.clone(),
        change_id: input.change_id.clone(),
        source_file: input.source_file.clone(),
        spec_group: input.spec_group.clone(),
        spec_id: Some(input.spec_id.clone()),
        tags: input.tags.clone(),
    };

    // R3: Map each diagram into a SpecManifest
    for diagram in &input.diagrams {
        let kind = match map_diagram_to_kind(&diagram.diagram_type) {
            Some(k) => k,
            None => continue, // Skip unsupported diagram types
        };

        let mut payload = serde_yaml::Mapping::new();
        payload.insert(
            serde_yaml::Value::String("title".into()),
            serde_yaml::Value::String(diagram.title.clone()),
        );
        payload.insert(
            serde_yaml::Value::String("diagram_type".into()),
            serde_yaml::Value::String(diagram.diagram_type.clone()),
        );
        payload.insert(
            serde_yaml::Value::String("content".into()),
            diagram.content.clone(),
        );

        let manifest = SpecManifest {
            api_version: super::API_VERSION.to_string(),
            kind,
            metadata: base_metadata.clone(),
            spec: serde_yaml::Value::Mapping(payload),
        };

        // R2: canonical filename
        let path = spec_ir_dir.join(manifest.filename());
        manifest.write_to(&path)?;
        files.push(path);
    }

    // R3: Map API spec into a SpecManifest
    if let Some(ref api_spec) = input.api_spec {
        if let Some(kind) = map_api_to_kind(&api_spec.api_type) {
            let mut payload = serde_yaml::Mapping::new();
            payload.insert(
                serde_yaml::Value::String("api_type".into()),
                serde_yaml::Value::String(api_spec.api_type.clone()),
            );
            payload.insert(
                serde_yaml::Value::String("content".into()),
                api_spec.content.clone(),
            );

            let manifest = SpecManifest {
                api_version: super::API_VERSION.to_string(),
                kind,
                metadata: base_metadata,
                spec: serde_yaml::Value::Mapping(payload),
            };

            let path = spec_ir_dir.join(manifest.filename());
            manifest.write_to(&path)?;
            files.push(path);
        }
    }

    Ok(GenerateResult { files })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn base_input() -> SpecIrInput {
        SpecIrInput {
            spec_id: "user-service".into(),
            change_id: "genesis-372".into(),
            spec_group: Some("sdd".into()),
            source_file: Some("specs/sdd/user-service.md".into()),
            tags: vec!["api".into()],
            diagrams: vec![],
            api_spec: None,
        }
    }

    // -- R1: YAML Generation --

    #[test]
    fn test_generate_creates_spec_ir_dir() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "flowchart".into(),
            title: "Main Flow".into(),
            content: serde_yaml::Value::String("graph TD; A-->B".into()),
        });

        let result = generate(change_dir, &input).unwrap();
        assert_eq!(result.files.len(), 1);
        assert!(change_dir.join("spec_ir").is_dir());
    }

    #[test]
    fn test_generate_creates_yaml_file() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "sequence".into(),
            title: "Auth Flow".into(),
            content: serde_yaml::Value::String("A->>B: hello".into()),
        });

        let result = generate(change_dir, &input).unwrap();
        assert_eq!(result.files.len(), 1);
        assert!(result.files[0].exists());

        // Verify it's valid YAML and can be parsed as SpecManifest
        let manifest = SpecManifest::from_file(&result.files[0]).unwrap();
        assert_eq!(manifest.kind, SpecKind::SequencePlus);
    }

    #[test]
    fn test_generate_overwrites_existing() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "flowchart".into(),
            title: "V1".into(),
            content: serde_yaml::Value::String("old".into()),
        });
        generate(change_dir, &input).unwrap();

        // Update content
        input.diagrams[0].title = "V2".into();
        input.diagrams[0].content = serde_yaml::Value::String("new".into());
        let result = generate(change_dir, &input).unwrap();

        let manifest = SpecManifest::from_file(&result.files[0]).unwrap();
        let title = manifest.spec["title"].as_str().unwrap();
        assert_eq!(title, "V2");
    }

    // -- R2: File Naming --

    #[test]
    fn test_file_naming_pattern() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "erd".into(),
            title: "Data Model".into(),
            content: serde_yaml::Value::Null,
        });

        let result = generate(change_dir, &input).unwrap();
        let filename = result.files[0].file_name().unwrap().to_str().unwrap();
        // Pattern: <group>_<name>_<kind>.yaml
        assert_eq!(filename, "sdd_user-service_erd_plus.yaml");
    }

    #[test]
    fn test_file_naming_default_group() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.spec_group = None;
        input.diagrams.push(DiagramEntry {
            diagram_type: "class".into(),
            title: "Class Diagram".into(),
            content: serde_yaml::Value::Null,
        });

        let result = generate(change_dir, &input).unwrap();
        let filename = result.files[0].file_name().unwrap().to_str().unwrap();
        assert_eq!(filename, "default_user-service_class_plus.yaml");
    }

    // -- R3: Content Mapping --

    #[test]
    fn test_content_mapping_diagram() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "flowchart".into(),
            title: "Main Flow".into(),
            content: serde_yaml::Value::String("graph TD; Start-->End".into()),
        });

        let result = generate(change_dir, &input).unwrap();
        let manifest = SpecManifest::from_file(&result.files[0]).unwrap();

        assert_eq!(manifest.kind, SpecKind::FlowchartPlus);
        assert_eq!(manifest.metadata.name, "user-service");
        assert_eq!(manifest.metadata.change_id, "genesis-372");
        assert_eq!(manifest.spec["title"].as_str().unwrap(), "Main Flow");
        assert_eq!(manifest.spec["diagram_type"].as_str().unwrap(), "flowchart");
    }

    #[test]
    fn test_content_mapping_api_spec() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        let mut api_content = serde_yaml::Mapping::new();
        api_content.insert(
            serde_yaml::Value::String("openapi".into()),
            serde_yaml::Value::String("3.1.0".into()),
        );
        input.api_spec = Some(ApiSpecEntry {
            api_type: "openapi-3.1".into(),
            content: serde_yaml::Value::Mapping(api_content),
        });

        let result = generate(change_dir, &input).unwrap();
        let manifest = SpecManifest::from_file(&result.files[0]).unwrap();

        assert_eq!(manifest.kind, SpecKind::Api);
        assert_eq!(manifest.spec["api_type"].as_str().unwrap(), "openapi-3.1");
    }

    #[test]
    fn test_multi_diagram_generates_multiple_files() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "flowchart".into(),
            title: "Flow".into(),
            content: serde_yaml::Value::Null,
        });
        input.diagrams.push(DiagramEntry {
            diagram_type: "sequence".into(),
            title: "Seq".into(),
            content: serde_yaml::Value::Null,
        });
        input.diagrams.push(DiagramEntry {
            diagram_type: "erd".into(),
            title: "ERD".into(),
            content: serde_yaml::Value::Null,
        });

        let result = generate(change_dir, &input).unwrap();
        assert_eq!(result.files.len(), 3);

        // Verify each has the correct kind
        let kinds: Vec<SpecKind> = result
            .files
            .iter()
            .map(|f| SpecManifest::from_file(f).unwrap().kind)
            .collect();
        assert!(kinds.contains(&SpecKind::FlowchartPlus));
        assert!(kinds.contains(&SpecKind::SequencePlus));
        assert!(kinds.contains(&SpecKind::ErdPlus));
    }

    #[test]
    fn test_unsupported_diagram_type_skipped() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "unknown_type".into(),
            title: "Unknown".into(),
            content: serde_yaml::Value::Null,
        });

        let result = generate(change_dir, &input).unwrap();
        assert!(result.files.is_empty());
    }

    #[test]
    fn test_metadata_fields_preserved() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let mut input = base_input();
        input.diagrams.push(DiagramEntry {
            diagram_type: "requirement".into(),
            title: "Reqs".into(),
            content: serde_yaml::Value::Null,
        });

        let result = generate(change_dir, &input).unwrap();
        let manifest = SpecManifest::from_file(&result.files[0]).unwrap();

        assert_eq!(manifest.metadata.spec_group.as_deref(), Some("sdd"));
        assert_eq!(manifest.metadata.spec_id.as_deref(), Some("user-service"));
        assert_eq!(
            manifest.metadata.source_file.as_deref(),
            Some("specs/sdd/user-service.md")
        );
        assert_eq!(manifest.metadata.tags, vec!["api"]);
    }

    #[test]
    fn test_empty_input_no_files() {
        let tmp = TempDir::new().unwrap();
        let change_dir = tmp.path();
        let input = base_input();

        let result = generate(change_dir, &input).unwrap();
        assert!(result.files.is_empty());
        // spec_ir dir is still created
        assert!(change_dir.join("spec_ir").is_dir());
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/spec_ir/generator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete SpecIR YAML generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Four data carriers; mix of foreign types (Value, PathBuf) and Options.
- [schema] All in `required:`; foreign types via x-rust-type.
- [changes] Standard split with all four in `replaces`.
