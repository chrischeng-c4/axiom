// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#source
// CODEGEN-BEGIN
//! YAML-based code generation dispatch.
//!
//! Implements lens-yaml-codegen spec:
//! - R1: YAML Reader — read manifests from disk via SpecManifest::from_file
//! - R2: Generic Generator Input — ManifestGenerator trait for SpecManifest
//! - R3: Generator Dispatch — ManifestDispatcher routes by kind
//!
//! ## Relationship to `generate::generators::common`
//!
//! - [`Generator`] — takes `JsonSchema` + settings + engine (low-level, schema-only).
//! - [`SpecIRGenerator`] — takes `SpecIR` + settings + engine (typed payload).
//! - [`ManifestGenerator`] — takes `SpecManifest` (YAML envelope, higher-level
//!   convenience). Implementors typically parse the opaque `spec` field and
//!   delegate to a `SpecIRGenerator`.
//!
//! All three traits are re-exported here for discoverability.

use std::path::PathBuf;

use super::{SpecKind, SpecManifest};

// Re-export the generate module's traits so consumers of spec_ir::codegen
// can access the full generator trait hierarchy from one place.
pub use crate::generate::generators::{
    GeneratedFile, Generator, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy,
    SpecIRGenerator,
};

/// Generated code output from a manifest generator.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#schema
#[derive(Debug, Clone)]
pub struct ManifestOutput {
    /// Source manifest path.
    pub source: PathBuf,
    /// Generated file path (relative to project root).
    pub output_path: String,
    /// Generated content.
    pub content: String,
    /// Kind of manifest that produced this output.
    pub kind: SpecKind,
}

/// Dispatcher that routes manifests to registered generators.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#schema
pub struct ManifestDispatcher {
    /// Registered generators.
    generators: Vec<Box<dyn ManifestGenerator>>,
}
/// Trait for generators that consume SpecManifest input (R2).
///
/// Each implementation handles one or more SpecKind variants.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#source
pub trait ManifestGenerator: Send + Sync {
    /// Generator name for display and error messages.
    fn name(&self) -> &str;

    /// Check if this generator can handle the given manifest kind.
    fn can_handle(&self, kind: &SpecKind) -> bool;

    /// Generate code from a manifest.
    fn generate(&self, manifest: &SpecManifest) -> crate::Result<Vec<ManifestOutput>>;
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#source
impl ManifestDispatcher {
    /// Create a new dispatcher with no registered generators.
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    /// Register a generator.
    pub fn register(&mut self, generator: Box<dyn ManifestGenerator>) {
        self.generators.push(generator);
    }

    /// Find a generator that can handle the given kind.
    pub fn find(&self, kind: &SpecKind) -> Option<&dyn ManifestGenerator> {
        self.generators
            .iter()
            .find(|g| g.can_handle(kind))
            .map(|g| g.as_ref())
    }

    /// Generate code from a single manifest (R3).
    ///
    /// Routes to the appropriate generator based on the manifest's `kind`.
    /// Returns an error if no generator is registered for the kind.
    pub fn dispatch(&self, manifest: &SpecManifest) -> crate::Result<Vec<ManifestOutput>> {
        let generator = self.find(&manifest.kind).ok_or_else(|| {
            anyhow::anyhow!(
                "No generator found for kind {:?}. Register a ManifestGenerator \
                 that handles this kind.",
                manifest.kind
            )
        })?;
        generator.generate(manifest)
    }

    /// Read and generate from YAML manifest files (R1 + R3).
    ///
    /// Reads each path as a SpecManifest, dispatches to generators,
    /// and collects all outputs. Stops on first error.
    pub fn generate_from_paths(&self, paths: &[PathBuf]) -> crate::Result<Vec<ManifestOutput>> {
        let mut outputs = Vec::new();
        for path in paths {
            let manifest = SpecManifest::from_file(path)?;
            let mut result = self.dispatch(&manifest)?;
            // Tag outputs with source path
            for output in &mut result {
                output.source = path.clone();
            }
            outputs.extend(result);
        }
        Ok(outputs)
    }
}

/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_ir/codegen.md#source
impl Default for ManifestDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Test generator that handles Api kind
    struct TestApiGenerator;

    impl ManifestGenerator for TestApiGenerator {
        fn name(&self) -> &str {
            "test-api"
        }

        fn can_handle(&self, kind: &SpecKind) -> bool {
            matches!(kind, SpecKind::Api)
        }

        fn generate(&self, manifest: &SpecManifest) -> crate::Result<Vec<ManifestOutput>> {
            Ok(vec![ManifestOutput {
                source: PathBuf::new(),
                output_path: format!("src/{}.rs", manifest.metadata.name),
                content: format!("// Generated from {}", manifest.metadata.name),
                kind: manifest.kind.clone(),
            }])
        }
    }

    /// Test generator that handles FlowchartPlus kind
    struct TestFlowGenerator;

    impl ManifestGenerator for TestFlowGenerator {
        fn name(&self) -> &str {
            "test-flow"
        }

        fn can_handle(&self, kind: &SpecKind) -> bool {
            matches!(kind, SpecKind::FlowchartPlus)
        }

        fn generate(&self, manifest: &SpecManifest) -> crate::Result<Vec<ManifestOutput>> {
            Ok(vec![ManifestOutput {
                source: PathBuf::new(),
                output_path: format!("src/{}_flow.rs", manifest.metadata.name),
                content: "// Flow generated".into(),
                kind: manifest.kind.clone(),
            }])
        }
    }

    fn write_manifest(dir: &Path, kind: SpecKind, name: &str) -> PathBuf {
        let manifest = SpecManifest::new(
            kind,
            name,
            "test-change",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
        let spec_ir_dir = dir.join("spec_ir");
        std::fs::create_dir_all(&spec_ir_dir).unwrap();
        let path = spec_ir_dir.join(manifest.filename());
        manifest.write_to(&path).unwrap();
        path
    }

    // -- R1: YAML Reader --

    #[test]
    fn test_read_manifest_from_file() {
        let tmp = TempDir::new().unwrap();
        let path = write_manifest(tmp.path(), SpecKind::Api, "user-service");
        let manifest = SpecManifest::from_file(&path).unwrap();
        assert_eq!(manifest.kind, SpecKind::Api);
        assert_eq!(manifest.metadata.name, "user-service");
    }

    #[test]
    fn test_read_invalid_yaml_returns_error() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("bad.yaml");
        std::fs::write(&path, "not: valid: yaml: {{{{").unwrap();
        let result = SpecManifest::from_file(&path);
        assert!(result.is_err());
    }

    // -- R2: Generic Generator Input --

    #[test]
    fn test_manifest_generator_trait() {
        let gen = TestApiGenerator;
        assert!(gen.can_handle(&SpecKind::Api));
        assert!(!gen.can_handle(&SpecKind::FlowchartPlus));

        let manifest = SpecManifest::new(
            SpecKind::Api,
            "test",
            "change-1",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
        let outputs = gen.generate(&manifest).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].output_path, "src/test.rs");
    }

    // -- R3: Generator Dispatch --

    #[test]
    fn test_dispatcher_routes_by_kind() {
        let mut dispatcher = ManifestDispatcher::new();
        dispatcher.register(Box::new(TestApiGenerator));
        dispatcher.register(Box::new(TestFlowGenerator));

        let api_manifest = SpecManifest::new(
            SpecKind::Api,
            "api-svc",
            "c1",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
        let flow_manifest = SpecManifest::new(
            SpecKind::FlowchartPlus,
            "flow",
            "c1",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );

        let api_result = dispatcher.dispatch(&api_manifest).unwrap();
        assert_eq!(api_result[0].output_path, "src/api-svc.rs");

        let flow_result = dispatcher.dispatch(&flow_manifest).unwrap();
        assert_eq!(flow_result[0].output_path, "src/flow_flow.rs");
    }

    #[test]
    fn test_dispatcher_error_on_unsupported_kind() {
        let dispatcher = ManifestDispatcher::new(); // No generators registered
        let manifest = SpecManifest::new(
            SpecKind::ErdPlus,
            "data",
            "c1",
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
        let result = dispatcher.dispatch(&manifest);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No generator found"));
    }

    #[test]
    fn test_generate_from_paths() {
        let tmp = TempDir::new().unwrap();
        let path1 = write_manifest(tmp.path(), SpecKind::Api, "svc-a");
        let path2 = write_manifest(tmp.path(), SpecKind::FlowchartPlus, "flow-b");

        let mut dispatcher = ManifestDispatcher::new();
        dispatcher.register(Box::new(TestApiGenerator));
        dispatcher.register(Box::new(TestFlowGenerator));

        let outputs = dispatcher.generate_from_paths(&[path1, path2]).unwrap();
        assert_eq!(outputs.len(), 2);
    }

    #[test]
    fn test_generate_from_paths_stops_on_error() {
        let tmp = TempDir::new().unwrap();
        // ErdPlus has no registered generator
        let path = write_manifest(tmp.path(), SpecKind::ErdPlus, "data");
        let dispatcher = ManifestDispatcher::new();

        let result = dispatcher.generate_from_paths(&[path]);
        assert!(result.is_err());
    }

    #[test]
    fn test_find_generator() {
        let mut dispatcher = ManifestDispatcher::new();
        dispatcher.register(Box::new(TestApiGenerator));

        assert!(dispatcher.find(&SpecKind::Api).is_some());
        assert_eq!(dispatcher.find(&SpecKind::Api).unwrap().name(), "test-api");
        assert!(dispatcher.find(&SpecKind::ErdPlus).is_none());
    }
}

// CODEGEN-END
