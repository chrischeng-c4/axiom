//! Generator registry — dispatches SpecIR to the correct generator.
//!
//! Instead of ad-hoc generator selection, consumers call
//! `registry.generate(spec_ir, ctx)` and the registry finds the first
//! generator whose `can_generate()` returns `true`.
//!
//! SpecIR types now live in `sdd::generate`. This registry accepts
//! `serde_json::Value` to avoid a circular crate dependency.

use super::traits::{CodeGenerator, GenContext, GenError, GenResult, GeneratedCode};

/// Registry holding all registered [`CodeGenerator`] implementations.
pub struct GeneratorRegistry {
    generators: Vec<Box<dyn CodeGenerator>>,
}

impl GeneratorRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            generators: Vec::new(),
        }
    }

    /// Register a generator.
    pub fn register(&mut self, gen: Box<dyn CodeGenerator>) {
        self.generators.push(gen);
    }

    /// Find the first generator that can handle the given SpecIR (as JSON value).
    pub fn find(&self, spec: &serde_json::Value) -> Option<&dyn CodeGenerator> {
        self.generators
            .iter()
            .find(|g| g.can_generate(spec))
            .map(|g| g.as_ref())
    }

    /// Generate code by dispatching to the first matching generator.
    ///
    /// The `spec` parameter is a serialized SpecIR value. The registry
    /// finds the first generator whose `can_generate()` returns `true`
    /// and delegates to it.
    pub fn generate(
        &self,
        spec: &serde_json::Value,
        ctx: &GenContext,
    ) -> GenResult<Vec<GeneratedCode>> {
        let kind = spec
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let gen = self.find(spec).ok_or_else(|| {
            GenError::UnsupportedFeature(format!(
                "no generator registered for SpecIR kind '{}'",
                kind
            ))
        })?;
        gen.generate(spec, ctx)
    }

    /// List names of all registered generators.
    pub fn list(&self) -> Vec<&str> {
        self.generators.iter().map(|g| g.name()).collect()
    }

    /// Number of registered generators.
    pub fn len(&self) -> usize {
        self.generators.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.generators.is_empty()
    }
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::ir::DataModelSpec;

    /// Stub generator for testing
    struct StubApiGen;

    impl CodeGenerator for StubApiGen {
        fn name(&self) -> &str {
            "stub-api"
        }

        fn can_generate(&self, spec: &serde_json::Value) -> bool {
            spec.get("kind").and_then(|v| v.as_str()) == Some("api")
        }

        fn generate(
            &self,
            _spec: &serde_json::Value,
            _ctx: &GenContext,
        ) -> GenResult<Vec<GeneratedCode>> {
            Ok(vec![GeneratedCode::new(
                "stub",
                "// generated",
                crate::gen::traits::Language::Rust,
            )])
        }

        fn generate_data_models(
            &self,
            _spec: &DataModelSpec,
            _ctx: &GenContext,
        ) -> GenResult<Vec<GeneratedCode>> {
            Err(GenError::UnsupportedFeature("use generate()".into()))
        }
    }

    #[test]
    fn test_registry_dispatch() {
        let mut registry = GeneratorRegistry::new();
        registry.register(Box::new(StubApiGen));

        let spec = serde_json::json!({
            "kind": "api",
            "schema": { "title": "Test" }
        });
        let ctx = GenContext::default();

        let result = registry.generate(&spec, &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_registry_no_match() {
        let registry = GeneratorRegistry::new();
        let spec = serde_json::json!({
            "kind": "api",
            "schema": {}
        });
        let ctx = GenContext::default();

        let result = registry.generate(&spec, &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_list() {
        let mut registry = GeneratorRegistry::new();
        registry.register(Box::new(StubApiGen));
        assert_eq!(registry.list(), vec!["stub-api"]);
        assert_eq!(registry.len(), 1);
    }
}
