// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#source
// CODEGEN-BEGIN
//! SpecIR YAML manifest types.
//!
//! Follows k8s-style resource model: apiVersion, kind, metadata, spec.
//! Each manifest is a single YAML document that can be read by any tool.
//!
//! ## Relationship to `generate::spec_ir`
//!
//! - **`SpecManifest`** is the YAML envelope (apiVersion, kind, metadata, spec).
//!   It carries an opaque `serde_yaml::Value` payload and is used for
//!   serialisation / disk I/O.
//! - **`SpecIR`** (in [`crate::generate::spec_ir`]) is the typed payload.
//!   Each variant wraps a concrete struct (e.g. `FlowchartDef`, `DeploySpec`).
//! - **`SpecKind`** maps 1:1 to the first 6 variants of `SpecIR`. The
//!   remaining `SpecIR` variants (`Deploy`, `Wireframe`, `Component`,
//!   `DesignToken`) are newer section types not yet surfaced in the
//!   manifest envelope.

use std::path::Path;

// Re-export the typed payload types from generate::spec_ir so consumers
// that import from this module can access both the envelope and the payload.
pub use crate::generate::spec_ir::{
    BundleMetadata, ComponentSpec, DeploySpec, DesignTokenSpec, SpecBundle, SpecIR, SpecMetadata,
    WireframeSpec,
};

/// Current API version for SpecIR manifests.
pub const API_VERSION: &str = "cclab.dev/v1";

use serde::{Deserialize, Serialize};

/// Valid spec kinds (maps to SDD diagram types).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecKind {
    #[serde(rename = "Api")]
    Api,
    #[serde(rename = "FlowchartPlus")]
    FlowchartPlus,
    #[serde(rename = "SequencePlus")]
    SequencePlus,
    #[serde(rename = "ClassPlus")]
    ClassPlus,
    #[serde(rename = "ErdPlus")]
    ErdPlus,
    #[serde(rename = "RequirementPlus")]
    RequirementPlus,
}

/// Manifest metadata (k8s-style).
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestMetadata {
    /// Human-readable name for this spec artifact.
    pub name: String,
    /// Change ID this manifest belongs to.
    pub change_id: String,
    /// Source spec file path (relative to project root).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    /// Spec group (e.g. "sdd").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_group: Option<String>,
    /// Spec ID within the group.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    /// Tags for filtering / routing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// A SpecIR YAML manifest — the k8s-style envelope.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpecManifest {
    /// API version (always "cclab.dev/v1").
    pub api_version: String,
    /// Spec kind.
    pub kind: SpecKind,
    /// Manifest metadata.
    pub metadata: ManifestMetadata,
    /// Kind-specific payload (opaque YAML).
    pub spec: serde_yaml::Value,
}
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#source
impl SpecKind {
    /// Return the kind as a snake_case string for file naming.
    pub fn as_snake(&self) -> &'static str {
        match self {
            SpecKind::Api => "api",
            SpecKind::FlowchartPlus => "flowchart_plus",
            SpecKind::SequencePlus => "sequence_plus",
            SpecKind::ClassPlus => "class_plus",
            SpecKind::ErdPlus => "erd_plus",
            SpecKind::RequirementPlus => "requirement_plus",
        }
    }

    /// Map this `SpecKind` to the corresponding `SpecIR` variant name.
    ///
    /// `SpecKind` maps 1:1 to `SpecIR` variants: `Api` → `"api"`,
    /// `FlowchartPlus` → `"flowchart_plus"`, etc.
    pub fn spec_ir_kind(&self) -> &'static str {
        // Intentionally identical to as_snake — documents the mapping.
        self.as_snake()
    }

    /// Construct a `SpecKind` from a `SpecIR` variant tag.
    ///
    /// Returns `None` for `SpecIR` variants that have no corresponding
    /// `SpecKind` (e.g. `deploy`, `wireframe`, `component`, `design_token`).
    pub fn from_spec_ir_kind(tag: &str) -> Option<Self> {
        match tag {
            "api" => Some(SpecKind::Api),
            "flowchart_plus" => Some(SpecKind::FlowchartPlus),
            "sequence_plus" => Some(SpecKind::SequencePlus),
            "class_plus" => Some(SpecKind::ClassPlus),
            "erd_plus" => Some(SpecKind::ErdPlus),
            "requirement_plus" => Some(SpecKind::RequirementPlus),
            _ => None,
        }
    }

    /// Parse from string (case-insensitive snake_case or PascalCase).
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "api" => Some(SpecKind::Api),
            "flowchart_plus" | "flowchartplus" => Some(SpecKind::FlowchartPlus),
            "sequence_plus" | "sequenceplus" => Some(SpecKind::SequencePlus),
            "class_plus" | "classplus" => Some(SpecKind::ClassPlus),
            "erd_plus" | "erdplus" => Some(SpecKind::ErdPlus),
            "requirement_plus" | "requirementplus" => Some(SpecKind::RequirementPlus),
            _ => None,
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/types.md#source
impl SpecManifest {
    /// Create a new manifest with the given kind and spec payload.
    pub fn new(
        kind: SpecKind,
        name: impl Into<String>,
        change_id: impl Into<String>,
        spec: serde_yaml::Value,
    ) -> Self {
        Self {
            api_version: API_VERSION.to_string(),
            kind,
            metadata: ManifestMetadata {
                name: name.into(),
                change_id: change_id.into(),
                ..Default::default()
            },
            spec,
        }
    }

    /// Serialize to YAML string.
    pub fn to_yaml(&self) -> crate::Result<String> {
        serde_yaml::to_string(self).map_err(|e| anyhow::anyhow!("YAML serialization failed: {e}"))
    }

    /// Deserialize from YAML string.
    pub fn from_yaml(yaml: &str) -> crate::Result<Self> {
        let manifest: Self =
            serde_yaml::from_str(yaml).map_err(|e| anyhow::anyhow!("YAML parse failed: {e}"))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Read a manifest from a YAML file.
    pub fn from_file(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", path.display()))?;
        Self::from_yaml(&content)
    }

    /// Write this manifest to a YAML file.
    pub fn write_to(&self, path: &Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("Failed to create dir: {e}"))?;
        }
        let yaml = self.to_yaml()?;
        std::fs::write(path, yaml)
            .map_err(|e| anyhow::anyhow!("Failed to write {}: {e}", path.display()))?;
        Ok(())
    }

    /// Validate manifest structure.
    pub fn validate(&self) -> crate::Result<()> {
        if self.api_version != API_VERSION {
            anyhow::bail!(
                "Unsupported apiVersion '{}', expected '{}'",
                self.api_version,
                API_VERSION
            );
        }
        if self.metadata.name.is_empty() {
            anyhow::bail!("metadata.name is required");
        }
        if self.metadata.change_id.is_empty() {
            anyhow::bail!("metadata.change_id is required");
        }
        Ok(())
    }

    /// Generate the canonical filename for this manifest.
    ///
    /// Pattern: `<group>_<name>_<kind>.yaml`
    pub fn filename(&self) -> String {
        let group = self.metadata.spec_group.as_deref().unwrap_or("default");
        format!(
            "{}_{}_{}.yaml",
            group,
            self.metadata.name,
            self.kind.as_snake()
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> SpecManifest {
        SpecManifest::new(
            SpecKind::Api,
            "user-service",
            "genesis-372",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        )
    }

    #[test]
    fn test_manifest_roundtrip() {
        let manifest = sample_manifest();
        let yaml = manifest.to_yaml().unwrap();
        let parsed = SpecManifest::from_yaml(&yaml).unwrap();
        assert_eq!(parsed.api_version, API_VERSION);
        assert_eq!(parsed.kind, SpecKind::Api);
        assert_eq!(parsed.metadata.name, "user-service");
        assert_eq!(parsed.metadata.change_id, "genesis-372");
    }

    #[test]
    fn test_manifest_validation_bad_version() {
        let mut m = sample_manifest();
        m.api_version = "v2".into();
        let yaml = serde_yaml::to_string(&m).unwrap();
        let err = SpecManifest::from_yaml(&yaml).unwrap_err();
        assert!(err.to_string().contains("Unsupported apiVersion"));
    }

    #[test]
    fn test_manifest_validation_missing_name() {
        let mut m = sample_manifest();
        m.metadata.name = String::new();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("metadata.name is required"));
    }

    #[test]
    fn test_manifest_validation_missing_change_id() {
        let mut m = sample_manifest();
        m.metadata.change_id = String::new();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("metadata.change_id is required"));
    }

    #[test]
    fn test_manifest_filename() {
        let mut m = sample_manifest();
        m.metadata.spec_group = Some("sdd".into());
        assert_eq!(m.filename(), "sdd_user-service_api.yaml");
    }

    #[test]
    fn test_manifest_filename_default_group() {
        let m = sample_manifest();
        assert_eq!(m.filename(), "default_user-service_api.yaml");
    }

    #[test]
    fn test_spec_kind_from_str_loose() {
        assert_eq!(SpecKind::from_str_loose("Api"), Some(SpecKind::Api));
        assert_eq!(
            SpecKind::from_str_loose("flowchart_plus"),
            Some(SpecKind::FlowchartPlus)
        );
        assert_eq!(
            SpecKind::from_str_loose("FlowchartPlus"),
            Some(SpecKind::FlowchartPlus)
        );
        assert_eq!(
            SpecKind::from_str_loose("ERD_PLUS"),
            Some(SpecKind::ErdPlus)
        );
        assert_eq!(SpecKind::from_str_loose("unknown"), None);
    }

    #[test]
    fn test_manifest_with_spec_payload() {
        let mut spec_map = serde_yaml::Mapping::new();
        spec_map.insert(
            serde_yaml::Value::String("title".into()),
            serde_yaml::Value::String("User API".into()),
        );
        let manifest = SpecManifest::new(
            SpecKind::Api,
            "user-api",
            "genesis-372",
            serde_yaml::Value::Mapping(spec_map),
        );
        let yaml = manifest.to_yaml().unwrap();
        assert!(yaml.contains("title: User API"));
        assert!(yaml.contains("kind: Api"));
        assert!(yaml.contains("apiVersion: cclab.dev/v1"));
    }

    #[test]
    fn test_manifest_file_roundtrip() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("spec_ir/test.yaml");
        let manifest = sample_manifest();
        manifest.write_to(&path).unwrap();
        let loaded = SpecManifest::from_file(&path).unwrap();
        assert_eq!(loaded.kind, SpecKind::Api);
        assert_eq!(loaded.metadata.name, "user-service");
    }

    #[test]
    fn test_manifest_error_on_missing_kind() {
        let yaml = r#"
apiVersion: cclab.dev/v1
metadata:
  name: test
  change_id: test-123
spec: {}
"#;
        let err = SpecManifest::from_yaml(yaml).unwrap_err();
        assert!(err.to_string().contains("YAML parse failed"));
    }

    #[test]
    fn test_reject_unknown_fields() {
        let yaml = r#"
apiVersion: cclab.dev/v1
kind: Api
metadata:
  name: test
  change_id: test-123
spec: {}
extra_field: should_fail
"#;
        let err = SpecManifest::from_yaml(yaml).unwrap_err();
        assert!(err.to_string().contains("YAML parse failed"));
    }

    #[test]
    fn test_reject_unknown_metadata_fields() {
        let yaml = r#"
apiVersion: cclab.dev/v1
kind: Api
metadata:
  name: test
  change_id: test-123
  unknown_meta: bad
spec: {}
"#;
        let err = SpecManifest::from_yaml(yaml).unwrap_err();
        assert!(err.to_string().contains("YAML parse failed"));
    }

    #[test]
    fn test_all_spec_kinds() {
        for kind in [
            SpecKind::Api,
            SpecKind::FlowchartPlus,
            SpecKind::SequencePlus,
            SpecKind::ClassPlus,
            SpecKind::ErdPlus,
            SpecKind::RequirementPlus,
        ] {
            let m = SpecManifest::new(
                kind.clone(),
                "test",
                "change-1",
                serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
            );
            let yaml = m.to_yaml().unwrap();
            let parsed = SpecManifest::from_yaml(&yaml).unwrap();
            assert_eq!(parsed.kind, kind);
        }
    }
}

// CODEGEN-END
