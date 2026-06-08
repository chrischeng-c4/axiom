---
id: sdd-gen-rust-cli-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# CLI Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/cli.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CliArgDef` | projects/agentic-workflow/src/generate/gen/rust/cli.rs | struct | pub | 23 |  |
| `CliCommandDef` | projects/agentic-workflow/src/generate/gen/rust/cli.rs | struct | pub | 34 |  |
| `CliGenOutput` | projects/agentic-workflow/src/generate/gen/rust/cli.rs | struct | pub | 16 |  |
| `generate_cli` | projects/agentic-workflow/src/generate/gen/rust/cli.rs | function | pub | 46 | generate_cli(cli_yaml: &Value, config: &RustConfig) -> CliGenOutput |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  CliGenOutput:
    type: object
    required: [code]
    description: Output from CLI codegen.
    properties:
      code: { type: string }
    x-rust-struct:
      derive: [Debug, Clone]

  CliArgDef:
    type: object
    required: [name, required, arg_type]
    description: A single CLI argument definition.
    properties:
      name: { type: string }
      description: { type: string }
      required: { type: boolean }
      short:
        type: string
        x-rust-type: "Option<char>"
      arg_type: { type: string }
    x-rust-struct:
      derive: [Debug, Clone]

  CliCommandDef:
    type: object
    required: [name, args, subcommands]
    description: A CLI command definition (recursive via subcommands).
    properties:
      name: { type: string }
      description: { type: string }
      args:
        type: array
        items: { $ref: "#/definitions/CliArgDef" }
      subcommands:
        type: array
        items: { $ref: "#/definitions/CliCommandDef" }
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/cli.rs -->
```rust
//! CLI structural generator.
//!
//! Produces a clap `#[derive(Subcommand)]` enum from CLI tree YAML frontmatter.
//! 100% deterministic coverage for cli section types.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R2

use crate::generate::types::RustConfig;
use serde_yaml::Value;

/// Output from CLI codegen.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/cli.md#schema
#[derive(Debug, Clone)]
pub struct CliGenOutput {
    pub code: String,
}

/// A single CLI argument definition.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/cli.md#schema
#[derive(Debug, Clone)]
pub struct CliArgDef {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub short: Option<char>,
    pub arg_type: String,
}

/// A CLI command definition (recursive via subcommands).
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/cli.md#schema
#[derive(Debug, Clone)]
pub struct CliCommandDef {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<CliArgDef>,
    pub subcommands: Vec<CliCommandDef>,
}

/// Generate a clap Subcommand enum from CLI tree YAML.
///
/// The YAML should have a `commands` list or `name`/`args`/`subcommands` structure.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R2
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R6
pub fn generate_cli(cli_yaml: &Value, config: &RustConfig) -> CliGenOutput {
    let config = config.merge_overrides(cli_yaml);
    let vis = config.vis_prefix();

    // Collect top-level commands
    let commands = parse_commands(cli_yaml);

    let enum_name = cli_yaml
        .get("name")
        .and_then(|v| v.as_str())
        .map(to_pascal_case)
        .unwrap_or_else(|| "Commands".to_string());

    let mut lines = Vec::new();
    lines.push("#[derive(Subcommand)]".to_string());
    lines.push(format!("{}enum {} {{", vis, enum_name));

    for cmd in &commands {
        generate_command_variant(cmd, &mut lines, vis.trim_end());
        lines.push(String::new());
    }

    lines.push("}".to_string());

    CliGenOutput {
        code: lines.join("\n"),
    }
}

fn generate_command_variant(cmd: &CliCommandDef, lines: &mut Vec<String>, _vis: &str) {
    // Doc comment
    if let Some(desc) = &cmd.description {
        lines.push(format!("    /// {}", desc));
    }

    let variant_name = to_pascal_case(&cmd.name);

    if !cmd.subcommands.is_empty() {
        // Subcommand variant
        let sub_enum = format!("{}Commands", variant_name);
        lines.push(format!("    #[command(subcommand)]"));
        lines.push(format!("    {}({}),", variant_name, sub_enum));
    } else if cmd.args.is_empty() {
        lines.push(format!("    {},", variant_name));
    } else {
        lines.push(format!("    {} {{", variant_name));
        for arg in &cmd.args {
            if let Some(desc) = &arg.description {
                lines.push(format!("        /// {}", desc));
            }
            let mut arg_attrs = Vec::new();
            if arg.required {
                arg_attrs.push("required = true".to_string());
            } else {
                arg_attrs.push("long".to_string());
            }
            if let Some(short) = arg.short {
                arg_attrs.push(format!("short = '{}'", short));
            }
            if !arg_attrs.is_empty() {
                lines.push(format!("        #[arg({})]", arg_attrs.join(", ")));
            }
            let arg_name = to_snake_case(&arg.name);
            let arg_type = if arg.required {
                arg.arg_type.clone()
            } else {
                format!("Option<{}>", arg.arg_type)
            };
            lines.push(format!("        {}: {},", arg_name, arg_type));
        }
        lines.push("    },".to_string());
    }
}

fn parse_commands(yaml: &Value) -> Vec<CliCommandDef> {
    if let Some(commands) = yaml.get("commands").and_then(|v| v.as_sequence()) {
        return commands.iter().filter_map(parse_command).collect();
    }

    if let Some(subcommands) = yaml.get("subcommands").and_then(|v| v.as_sequence()) {
        return subcommands.iter().filter_map(parse_command).collect();
    }

    Vec::new()
}

fn parse_command(value: &Value) -> Option<CliCommandDef> {
    let name = value.get("name")?.as_str()?.to_string();
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let args = value
        .get("args")
        .and_then(|v| v.as_sequence())
        .map(|seq| seq.iter().filter_map(parse_arg).collect())
        .unwrap_or_default();

    let subcommands = value
        .get("subcommands")
        .and_then(|v| v.as_sequence())
        .map(|seq| seq.iter().filter_map(parse_command).collect())
        .unwrap_or_default();

    Some(CliCommandDef {
        name,
        description,
        args,
        subcommands,
    })
}

fn parse_arg(value: &Value) -> Option<CliArgDef> {
    let name = value.get("name")?.as_str()?.to_string();
    let description = value
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let required = value
        .get("required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let short = value
        .get("short")
        .and_then(|v| v.as_str())
        .and_then(|s| s.chars().next());
    let arg_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("String")
        .to_string();

    Some(CliArgDef {
        name,
        description,
        required,
        short,
        arg_type,
    })
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

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R2
    #[test]
    fn test_generates_clap_enum_from_cli_tree() {
        let yaml_str = r#"
name: GenCommands
commands:
  - name: diff
    description: Diff spec against target files
    args:
      - name: spec-path
        type: String
        required: false
  - name: apply
    description: Apply codegen to target files
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_cli(&yaml, &config);

        assert!(
            output.code.contains("#[derive(Subcommand)]"),
            "Should have Subcommand derive"
        );
        assert!(
            output.code.contains("enum GenCommands"),
            "Should generate GenCommands enum"
        );
        assert!(output.code.contains("Diff"), "Should have Diff variant");
        assert!(output.code.contains("Apply"), "Should have Apply variant");
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-structural.md#R2
    #[test]
    fn test_generates_args_for_commands() {
        let yaml_str = r#"
name: Cmds
commands:
  - name: create
    args:
      - name: title
        type: String
        required: true
        description: Title of the item
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let config = RustConfig::default();
        let output = generate_cli(&yaml, &config);

        assert!(
            output.code.contains("title: String"),
            "Should have title arg"
        );
        assert!(
            output.code.contains("Title of the item"),
            "Should have doc comment"
        );
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/cli.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete CLI structural generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
