// SPEC-MANAGED: projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-registry-rs.md#source
// CODEGEN-BEGIN
//! Prompt template registry for managing multiple templates

use super::template::PromptTemplate;
use std::collections::HashMap;
use std::io;
use std::path::Path;

/// Registry for managing prompt templates
#[derive(Debug, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-registry-rs.md#source
pub struct PromptRegistry {
    templates: HashMap<String, PromptTemplate>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-agent-eval-prompt-registry-rs.md#source
impl PromptRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register(&mut self, template: PromptTemplate) {
        let key = format!("{}@{}", template.name, template.version);
        self.templates.insert(key, template);
    }

    /// Get a template by name and version
    pub fn get(&self, name: &str, version: &str) -> Option<&PromptTemplate> {
        let key = format!("{}@{}", name, version);
        self.templates.get(&key)
    }

    /// Get the latest version of a template by name
    pub fn get_latest(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates
            .values()
            .filter(|t| t.name == name)
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// List all template names
    pub fn list_templates(&self) -> Vec<String> {
        let mut names: Vec<_> = self.templates.values().map(|t| t.name.clone()).collect();
        names.sort();
        names.dedup();
        names
    }

    /// List all versions of a template
    pub fn list_versions(&self, name: &str) -> Vec<String> {
        let mut versions: Vec<_> = self
            .templates
            .values()
            .filter(|t| t.name == name)
            .map(|t| t.version.clone())
            .collect();
        versions.sort();
        versions
    }

    /// Load a template from YAML file
    pub fn load_from_file(&mut self, path: impl AsRef<Path>) -> io::Result<String> {
        let content = std::fs::read_to_string(path)?;
        let template: PromptTemplate = serde_yaml::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let key = format!("{}@{}", template.name, template.version);
        self.register(template);

        Ok(key)
    }

    /// Load all templates from a directory
    pub fn load_from_directory(&mut self, dir: impl AsRef<Path>) -> io::Result<Vec<String>> {
        let mut loaded = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                match self.load_from_file(&path) {
                    Ok(key) => loaded.push(key),
                    Err(e) => eprintln!("Failed to load {:?}: {}", path, e),
                }
            }
        }

        Ok(loaded)
    }

    /// Save a template to YAML file
    pub fn save_to_file(
        &self,
        name: &str,
        version: &str,
        path: impl AsRef<Path>,
    ) -> io::Result<()> {
        let template = self
            .get(name, version)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Template not found"))?;

        let yaml = serde_yaml::to_string(template)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        std::fs::write(path, yaml)
    }

    /// Count registered templates
    pub fn count(&self) -> usize {
        self.templates.len()
    }

    /// Clear all templates
    pub fn clear(&mut self) {
        self.templates.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_eval::prompt::template::PromptSection;

    #[test]
    fn test_register_and_get() {
        let mut registry = PromptRegistry::new();

        let template =
            PromptTemplate::basic("test").with_section(PromptSection::new("Input", "{{input}}"));

        registry.register(template);

        let retrieved = registry.get("test", "1.0.0");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "test");
    }

    #[test]
    fn test_get_latest() {
        let mut registry = PromptRegistry::new();

        // Register v1.0.0
        let mut template1 = PromptTemplate::basic("test");
        template1.version = "1.0.0".to_string();
        registry.register(template1);

        // Register v2.0.0
        let mut template2 = PromptTemplate::basic("test");
        template2.version = "2.0.0".to_string();
        registry.register(template2);

        let latest = registry.get_latest("test");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().version, "2.0.0");
    }

    #[test]
    fn test_list_templates() {
        let mut registry = PromptRegistry::new();

        registry.register(PromptTemplate::basic("template1"));
        registry.register(PromptTemplate::basic("template2"));

        let mut template3 = PromptTemplate::basic("template1");
        template3.version = "2.0.0".to_string();
        registry.register(template3);

        let templates = registry.list_templates();
        assert_eq!(templates.len(), 2);
        assert!(templates.contains(&"template1".to_string()));
        assert!(templates.contains(&"template2".to_string()));
    }

    #[test]
    fn test_list_versions() {
        let mut registry = PromptRegistry::new();

        let mut template1 = PromptTemplate::basic("test");
        template1.version = "1.0.0".to_string();
        registry.register(template1);

        let mut template2 = PromptTemplate::basic("test");
        template2.version = "2.0.0".to_string();
        registry.register(template2);

        let versions = registry.list_versions("test");
        assert_eq!(versions.len(), 2);
        assert!(versions.contains(&"1.0.0".to_string()));
        assert!(versions.contains(&"2.0.0".to_string()));
    }

    #[test]
    fn test_count_and_clear() {
        let mut registry = PromptRegistry::new();

        registry.register(PromptTemplate::basic("template1"));
        registry.register(PromptTemplate::basic("template2"));

        assert_eq!(registry.count(), 2);

        registry.clear();
        assert_eq!(registry.count(), 0);
    }
}
// CODEGEN-END
