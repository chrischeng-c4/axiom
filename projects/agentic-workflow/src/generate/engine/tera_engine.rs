// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/tera_engine_preamble.md#source
// CODEGEN-BEGIN
//! Tera template engine wrapper

use super::error::TemplateError;
use super::filters;
use serde::Serialize;
use std::path::Path;
use tera::{Context, Tera};
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/tera_engine.md#schema
// CODEGEN-BEGIN
/// Tera template engine wrapper. Holds the `tera::Tera` engine
/// with custom filters registered. All behaviour is on the
/// hand-written impl block.
/// @spec projects/agentic-workflow/tech-design/core/generate/engine/tera_engine.md#schema
pub struct TemplateEngine {
    /// Underlying tera engine.
    tera: Tera,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/engine/tera_engine_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/engine/tera_engine_runtime.md#source
impl TemplateEngine {
    /// Create a new template engine loading templates from a directory
    ///
    /// Recursively loads all .j2 and .tera files from the directory.
    pub fn new(template_dir: impl AsRef<Path>) -> Result<Self, TemplateError> {
        let dir = template_dir.as_ref();
        if !dir.exists() {
            return Err(TemplateError::DirectoryNotFound(dir.to_path_buf()));
        }

        // Build glob pattern for .j2 and .tera files only
        let pattern = format!("{}/**/*.{{j2,tera}}", dir.display());
        let mut tera = Tera::new(&pattern).map_err(|e| TemplateError::ParseError {
            template: "initialization".to_string(),
            message: e.to_string(),
        })?;

        // Register custom filters
        tera.register_filter("pascal_case", filters::pascal_case);
        tera.register_filter("camel_case", filters::camel_case);
        tera.register_filter("snake_case", filters::snake_case);
        tera.register_filter("kebab_case", filters::kebab_case);

        Ok(Self { tera })
    }

    /// Create an empty template engine (for programmatic template addition)
    pub fn empty() -> Self {
        let mut tera = Tera::default();

        // Register custom filters
        tera.register_filter("pascal_case", filters::pascal_case);
        tera.register_filter("camel_case", filters::camel_case);
        tera.register_filter("snake_case", filters::snake_case);
        tera.register_filter("kebab_case", filters::kebab_case);

        Self { tera }
    }

    /// Add a template from a string
    pub fn add_template(&mut self, name: &str, content: &str) -> Result<(), TemplateError> {
        self.tera
            .add_raw_template(name, content)
            .map_err(|e| TemplateError::ParseError {
                template: name.to_string(),
                message: e.to_string(),
            })
    }

    /// Render a template with the given context
    pub fn render<T: Serialize>(
        &self,
        template: &str,
        context: &T,
    ) -> Result<String, TemplateError> {
        let ctx = Context::from_serialize(context).map_err(|e| TemplateError::TypeMismatch {
            expected: "serializable".to_string(),
            actual: e.to_string(),
        })?;

        self.tera.render(template, &ctx).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                TemplateError::NotFound(template.to_string())
            } else {
                TemplateError::RenderError {
                    template: template.to_string(),
                    message: msg,
                }
            }
        })
    }

    /// Check if a template exists
    pub fn has_template(&self, name: &str) -> bool {
        self.tera.get_template_names().any(|n| n == name)
    }

    /// Get all template names
    pub fn template_names(&self) -> impl Iterator<Item = &str> {
        self.tera.get_template_names()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_empty_engine_with_filters() {
        let mut engine = TemplateEngine::empty();
        engine
            .add_template("test.rs", "{{ name | pascal_case }}")
            .unwrap();

        let result = engine
            .render("test.rs", &json!({"name": "my_module"}))
            .unwrap();
        assert_eq!(result, "MyModule");
    }

    #[test]
    fn test_render_missing_template() {
        let engine = TemplateEngine::empty();
        let result = engine.render("ghost.j2", &json!({}));
        assert!(matches!(result, Err(TemplateError::NotFound(_))));
    }

    #[test]
    fn test_nested_context() {
        let mut engine = TemplateEngine::empty();
        engine
            .add_template("config.rs", "version = {{ config.version }}")
            .unwrap();

        let result = engine
            .render("config.rs", &json!({"config": {"version": "1.0"}}))
            .unwrap();
        assert_eq!(result, "version = 1.0");
    }

    #[test]
    fn test_has_template() {
        let mut engine = TemplateEngine::empty();
        engine.add_template("exists.rs", "").unwrap();

        assert!(engine.has_template("exists.rs"));
        assert!(!engine.has_template("not_exists.rs"));
    }
}
// CODEGEN-END
