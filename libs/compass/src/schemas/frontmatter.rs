//! Frontmatter schema definitions for Hugo, Jekyll, Docusaurus, and Generic.
//!
//! Provides JSON Schema validators for YAML frontmatter used in static site
//! generators. Validators are compiled once and cached via `OnceLock`.

use jsonschema::Validator;
use serde_json::{json, Value};
use std::sync::OnceLock;

/// Frontmatter framework type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontmatterFramework {
    Hugo,
    Jekyll,
    Docusaurus,
    Generic,
}

/// Get the frontmatter validator for a framework
pub fn frontmatter_validator(framework: FrontmatterFramework) -> &'static Validator {
    match framework {
        FrontmatterFramework::Hugo => hugo_validator(),
        FrontmatterFramework::Jekyll => jekyll_validator(),
        FrontmatterFramework::Docusaurus => docusaurus_validator(),
        FrontmatterFramework::Generic => generic_validator(),
    }
}

fn hugo_validator() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema = hugo_schema();
        Validator::new(&schema).expect("valid Hugo schema")
    })
}

fn jekyll_validator() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema = jekyll_schema();
        Validator::new(&schema).expect("valid Jekyll schema")
    })
}

fn docusaurus_validator() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema = docusaurus_schema();
        Validator::new(&schema).expect("valid Docusaurus schema")
    })
}

fn generic_validator() -> &'static Validator {
    static V: OnceLock<Validator> = OnceLock::new();
    V.get_or_init(|| {
        let schema = generic_schema();
        Validator::new(&schema).expect("valid Generic schema")
    })
}

// ---------------------------------------------------------------------------
// Schema definitions
// ---------------------------------------------------------------------------

fn hugo_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "date": {"type": "string"},
            "draft": {"type": "boolean"},
            "tags": {
                "type": "array",
                "items": {"type": "string"}
            },
            "categories": {
                "type": "array",
                "items": {"type": "string"}
            },
            "weight": {"type": "integer"},
            "description": {"type": "string"},
            "slug": {"type": "string"},
            "aliases": {
                "type": "array",
                "items": {"type": "string"}
            },
            "layout": {"type": "string"},
            "url": {"type": "string"},
            "lastmod": {"type": "string"},
            "publishDate": {"type": "string"},
            "expiryDate": {"type": "string"},
            "type": {"type": "string"},
            "series": {
                "type": "array",
                "items": {"type": "string"}
            },
            "author": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "array", "items": {"type": "string"}}
                ]
            }
        },
        "required": ["title"]
    })
}

fn jekyll_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "layout": {"type": "string"},
            "title": {"type": "string"},
            "date": {"type": "string"},
            "categories": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "array", "items": {"type": "string"}}
                ]
            },
            "tags": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "array", "items": {"type": "string"}}
                ]
            },
            "permalink": {"type": "string"},
            "published": {"type": "boolean"},
            "excerpt_separator": {"type": "string"},
            "author": {"type": "string"},
            "description": {"type": "string"},
            "image": {"type": "string"},
            "redirect_from": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "array", "items": {"type": "string"}}
                ]
            }
        },
        "required": ["layout"]
    })
}

fn docusaurus_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "title": {"type": "string"},
            "sidebar_label": {"type": "string"},
            "sidebar_position": {"type": "number"},
            "description": {"type": "string"},
            "slug": {"type": "string"},
            "tags": {
                "type": "array",
                "items": {"type": "string"}
            },
            "draft": {"type": "boolean"},
            "hide_title": {"type": "boolean"},
            "hide_table_of_contents": {"type": "boolean"},
            "pagination_next": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "null"}
                ]
            },
            "pagination_prev": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "null"}
                ]
            },
            "custom_edit_url": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "null"}
                ]
            },
            "keywords": {
                "type": "array",
                "items": {"type": "string"}
            },
            "image": {"type": "string"},
            "toc_min_heading_level": {"type": "integer"},
            "toc_max_heading_level": {"type": "integer"}
        }
    })
}

fn generic_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "date": {"type": "string"},
            "description": {"type": "string"},
            "tags": {
                "type": "array",
                "items": {"type": "string"}
            },
            "author": {
                "oneOf": [
                    {"type": "string"},
                    {"type": "array", "items": {"type": "string"}}
                ]
            },
            "draft": {"type": "boolean"}
        }
    })
}

// ---------------------------------------------------------------------------
// Framework detection
// ---------------------------------------------------------------------------

/// Detect frontmatter framework from project root by checking well-known config files.
///
/// Detection order:
/// 1. Hugo   — `hugo.toml`, `config.toml`, `hugo.yaml`, `hugo.json`
/// 2. Jekyll — `_config.yml`, `_config.yaml`
/// 3. Docusaurus — `docusaurus.config.js`, `docusaurus.config.ts`
/// 4. Generic — fallback
pub fn detect_framework(project_root: &std::path::Path) -> FrontmatterFramework {
    // Hugo
    if project_root.join("hugo.toml").exists()
        || project_root.join("config.toml").exists()
        || project_root.join("hugo.yaml").exists()
        || project_root.join("hugo.json").exists()
        || project_root.join("config").join("_default").exists()
    {
        return FrontmatterFramework::Hugo;
    }

    // Jekyll
    if project_root.join("_config.yml").exists() || project_root.join("_config.yaml").exists() {
        return FrontmatterFramework::Jekyll;
    }

    // Docusaurus
    if project_root.join("docusaurus.config.js").exists()
        || project_root.join("docusaurus.config.ts").exists()
        || project_root.join("docusaurus.config.mjs").exists()
    {
        return FrontmatterFramework::Docusaurus;
    }

    FrontmatterFramework::Generic
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_hugo_validator_accepts_valid_frontmatter() {
        let validator = frontmatter_validator(FrontmatterFramework::Hugo);
        let data = json!({
            "title": "My Post",
            "date": "2024-01-01",
            "draft": false,
            "tags": ["rust", "web"],
            "weight": 10
        });
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_hugo_validator_rejects_missing_title() {
        let validator = frontmatter_validator(FrontmatterFramework::Hugo);
        let data = json!({
            "date": "2024-01-01",
            "draft": false
        });
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(!errors.is_empty(), "expected required 'title' error");
    }

    #[test]
    fn test_jekyll_validator_accepts_valid_frontmatter() {
        let validator = frontmatter_validator(FrontmatterFramework::Jekyll);
        let data = json!({
            "layout": "post",
            "title": "Hello World",
            "date": "2024-01-01",
            "categories": ["tutorial"],
            "published": true
        });
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_jekyll_validator_rejects_missing_layout() {
        let validator = frontmatter_validator(FrontmatterFramework::Jekyll);
        let data = json!({
            "title": "My Page"
        });
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(!errors.is_empty(), "expected required 'layout' error");
    }

    #[test]
    fn test_docusaurus_validator_accepts_valid_frontmatter() {
        let validator = frontmatter_validator(FrontmatterFramework::Docusaurus);
        let data = json!({
            "id": "my-doc",
            "title": "My Document",
            "sidebar_label": "My Doc",
            "sidebar_position": 1,
            "tags": ["guide"]
        });
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(errors.is_empty(), "expected no errors, got: {:?}", errors);
    }

    #[test]
    fn test_generic_validator_accepts_empty_frontmatter() {
        let validator = frontmatter_validator(FrontmatterFramework::Generic);
        let data = json!({});
        let errors: Vec<_> = validator.iter_errors(&data).collect();
        assert!(errors.is_empty(), "generic should accept empty frontmatter");
    }

    #[test]
    fn test_detect_framework_generic_fallback() {
        // A temp dir with no config files → Generic
        let dir = std::env::temp_dir();
        let framework = detect_framework(&dir);
        assert_eq!(framework, FrontmatterFramework::Generic);
    }

    #[test]
    fn test_validators_are_cached() {
        // Calling twice returns the same pointer — confirms OnceLock caching
        let v1 = frontmatter_validator(FrontmatterFramework::Hugo);
        let v2 = frontmatter_validator(FrontmatterFramework::Hugo);
        assert!(
            std::ptr::eq(v1, v2),
            "validator should be the same cached instance"
        );
    }
}
