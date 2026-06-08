---
id: sdd-generate-gen-rust-config
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ConfigGenOutput Type

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/config.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ConfigGenOutput` | projects/agentic-workflow/src/generate/gen/rust/config.rs | struct | pub | 16 |  |
| `generate_config` | projects/agentic-workflow/src/generate/gen/rust/config.rs | function | pub | 28 | generate_config(config_yaml: &Value, rust_config: &RustConfig) -> ConfigGenOutput |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ConfigGenOutput:
    type: object
    required: [code]
    description: |
      Output from config code generation.
    properties:
      code:
        type: string
        description: "The generated Rust config struct + Default impl."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/config.rs -->
```rust
//! Config structural generator.
//!
//! Produces serde config struct with Default impl from config YAML frontmatter.
//! 100% deterministic coverage for config section types.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R5

use crate::generate::types::RustConfig;
use serde_yaml::Value;

/// Output from config code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/config.md#schema
#[derive(Debug, Clone)]
pub struct ConfigGenOutput {
    /// The generated Rust config struct + Default impl.
    pub code: String,
}

/// Generate a Rust config struct with Default from a config schema YAML.
///
/// Each property in the config schema becomes a field with its default value.
/// Generates `impl Default for T` using schema defaults.
/// Includes a `load()` fn for reading the config from file.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R5
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
pub fn generate_config(config_yaml: &Value, rust_config: &RustConfig) -> ConfigGenOutput {
    let rust_config = rust_config.merge_overrides(config_yaml);
    let vis = rust_config.vis_prefix();

    let title = config_yaml
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Config");
    let struct_name = to_pascal_case(title);

    let properties = config_yaml
        .get("properties")
        .and_then(|v| v.as_mapping())
        .cloned()
        .unwrap_or_default();

    let mut lines = Vec::new();

    // Derives
    let derive_attr = rust_config.derive_attr();
    if !derive_attr.is_empty() {
        lines.push(derive_attr);
    }
    lines.push("#[derive(serde::Serialize, serde::Deserialize)]".to_string());

    lines.push(format!("{}struct {} {{", vis, struct_name));

    struct FieldDefault {
        rust_type: String,
        default_expr: String,
        fn_name: String,
    }
    let mut field_defaults: Vec<(String, FieldDefault)> = Vec::new();

    for (key, prop_value) in &properties {
        let field_name = key.as_str().unwrap_or("field");
        let rust_field_name = to_snake_case(field_name);

        if let Some(desc) = prop_value.get("description").and_then(|v| v.as_str()) {
            lines.push(format!("    /// {}", desc));
        }

        let rust_type = infer_config_type(prop_value);
        let default_val = prop_value.get("default");

        let fn_name = format!("default_{}", rust_field_name);
        let (has_default, default_expr) = build_default_expr(&rust_type, default_val);

        if has_default {
            lines.push(format!("    #[serde(default = \"{}\")]", fn_name));
            field_defaults.push((
                rust_field_name.clone(),
                FieldDefault {
                    rust_type: rust_type.clone(),
                    default_expr,
                    fn_name,
                },
            ));
        } else {
            lines.push("    #[serde(default)]".to_string());
        }

        lines.push(format!("    {}{}: {},", vis, rust_field_name, rust_type));
    }

    lines.push("}".to_string());
    lines.push(String::new());

    // Default functions for serde
    for (_, fd) in &field_defaults {
        lines.push(format!(
            "fn {}() -> {} {{ {} }}",
            fd.fn_name, fd.rust_type, fd.default_expr
        ));
    }

    if !field_defaults.is_empty() {
        lines.push(String::new());
    }

    // impl Default
    lines.push(format!("impl Default for {} {{", struct_name));
    lines.push("    fn default() -> Self {".to_string());
    lines.push("        Self {".to_string());
    for (key, prop_value) in &properties {
        let field_name = key.as_str().unwrap_or("field");
        let rust_field_name = to_snake_case(field_name);
        let rust_type = infer_config_type(prop_value);
        let default_val = prop_value.get("default");
        let (_, default_expr) = build_default_expr(&rust_type, default_val);
        lines.push(format!(
            "            {}: {},",
            rust_field_name, default_expr
        ));
    }
    lines.push("        }".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());
    lines.push(String::new());

    // load() function
    lines.push(format!("impl {} {{", struct_name));
    lines.push(format!(
        "    {}fn load(path: &std::path::Path) -> anyhow::Result<Self> {{",
        vis
    ));
    lines.push("        let content = std::fs::read_to_string(path)?;".to_string());
    lines.push("        let config: Self = toml::from_str(&content)?;".to_string());
    lines.push("        Ok(config)".to_string());
    lines.push("    }".to_string());
    lines.push("}".to_string());

    ConfigGenOutput {
        code: lines.join("\n"),
    }
}

fn infer_config_type(prop: &Value) -> String {
    let type_str = prop.get("type").and_then(|v| v.as_str());
    match type_str {
        Some("string") => "String".to_string(),
        Some("integer") => "i64".to_string(),
        Some("number") => "f64".to_string(),
        Some("boolean") => "bool".to_string(),
        Some("array") => {
            let item = prop
                .get("items")
                .and_then(|i| i.get("type"))
                .and_then(|v| v.as_str())
                .unwrap_or("String");
            format!("Vec<{}>", capitalize(item))
        }
        _ => "serde_json::Value".to_string(),
    }
}

fn build_default_expr(rust_type: &str, default_val: Option<&Value>) -> (bool, String) {
    if let Some(val) = default_val {
        let expr = match rust_type {
            "String" => {
                let s = val.as_str().unwrap_or("");
                (true, format!("\"{}\".to_string()", s))
            }
            "i64" | "i32" => {
                let n = val.as_i64().unwrap_or(0);
                (true, n.to_string())
            }
            "f64" | "f32" => {
                let n = val.as_f64().unwrap_or(0.0);
                (true, format!("{}_f64", n))
            }
            "bool" => {
                let b = val.as_bool().unwrap_or(false);
                (true, b.to_string())
            }
            _ if rust_type.starts_with("Vec<") => (false, format!("{}::new()", rust_type)),
            _ => (false, format!("Default::default()")),
        };
        expr
    } else {
        match rust_type {
            "String" => (false, "String::new()".to_string()),
            "bool" => (false, "false".to_string()),
            "i64" | "i32" => (false, "0".to_string()),
            "f64" | "f32" => (false, "0.0".to_string()),
            _ => (false, "Default::default()".to_string()),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
            result.push(c.to_lowercase().next().unwrap_or(c));
        } else if c == '-' {
            result.push('_');
        } else {
            result.push(c.to_lowercase().next().unwrap_or(c));
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::types::RustConfig;

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R5
    #[test]
    fn test_generates_config_struct_with_default() {
        let yaml_str = r#"
title: AppConfig
properties:
  host:
    type: string
    default: "localhost"
  port:
    type: integer
    default: 8080
  debug:
    type: boolean
    default: false
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let rust_config = RustConfig::default();
        let output = generate_config(&yaml, &rust_config);

        assert!(
            output.code.contains("struct AppConfig"),
            "Should generate AppConfig struct"
        );
        assert!(
            output.code.contains("impl Default for AppConfig"),
            "Should have Default impl"
        );
        assert!(
            output.code.contains("host: String"),
            "Should have host field"
        );
        assert!(output.code.contains("port: i64"), "Should have port field");
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R5
    #[test]
    fn test_generates_load_fn() {
        let yaml_str = "title: MyConfig\nproperties: {}";
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let rust_config = RustConfig::default();
        let output = generate_config(&yaml, &rust_config);

        assert!(output.code.contains("fn load("), "Should have load() fn");
        assert!(
            output.code.contains("std::path::Path"),
            "load() should take Path"
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/config.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete config structural generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Pure data carrier; matches existing gen output struct precedent.
- [schema] Single field, basic Debug+Clone derive. No issues.
- [changes] Standard codegen+hand-written split.
