// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/react_preamble.md#source
// CODEGEN-BEGIN
//! React component scaffold generator
//!
//! Generates a React functional component scaffold from a [`WireframeSpec`]
//! (wireframe section type):
//!
//! | Output file                   | Description                                 |
//! |-------------------------------|---------------------------------------------|
//! | `{ComponentName}.tsx`         | React functional component (TypeScript)     |
//! | `{ComponentName}.types.ts`    | TypeScript props interface                  |
//! | `index.ts`                    | Barrel re-export                            |
//!
//! The generator implements [`SpecIRGenerator`] and only accepts
//! [`SpecIR::Wireframe`] variants.

use super::common::{
    GeneratedFile, GeneratorError, GeneratorSettings, Manifest, OverwritePolicy, SpecIRGenerator,
};
use crate::generate::engine::TemplateEngine;
use crate::generate::spec_ir::{PropDef, SpecIR, WireframeNode, WireframeSpec};
use serde::Serialize;
use std::path::Path;

// ---------------------------------------------------------------------------
// ReactGenerator
// ---------------------------------------------------------------------------
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/react.md#schema
// CODEGEN-BEGIN
/// React generator (unit struct).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/react.md#schema
pub struct ReactGenerator;
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/react_runtime.md#source
// CODEGEN-BEGIN

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/react_runtime.md#source
impl ReactGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/react_runtime.md#source
impl Default for ReactGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Render one React wireframe output file for TD-to-code apply.
///
/// `ReactGenerator` normally returns a manifest and content hashes. The apply
/// path needs the concrete body for the single `Changes` target it is writing,
/// so this helper reuses the same context/rendering functions without inventing
/// a second TSX emitter.
/// @spec projects/agentic-workflow/tech-design/core/specs/typescript-frontend-emitter.md#ts-emit-component
pub fn render_react_wireframe_file(spec: &WireframeSpec, target_path: &Path) -> Option<String> {
    let file_name = target_path.file_name()?.to_str()?;
    let settings = GeneratorSettings {
        name: wireframe_settings_name(spec, target_path),
        ..Default::default()
    };
    let ctx = build_context(spec, &settings);

    if file_name == "index.ts" {
        return Some(generate_index_ts(&ctx));
    }
    if file_name.ends_with(".types.ts") {
        return Some(generate_types_ts(&ctx));
    }
    if target_path.extension().and_then(|ext| ext.to_str()) == Some("tsx") {
        return Some(generate_component_tsx(&ctx));
    }
    None
}

fn wireframe_settings_name(spec: &WireframeSpec, target_path: &Path) -> String {
    if !spec.name.trim().is_empty() {
        return spec.name.clone();
    }
    target_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("app")
        .to_string()
}

// ---------------------------------------------------------------------------
// Template context
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ReactContext {
    /// PascalCase component name
    component_name: String,
    /// Props interface name (e.g. `"UserCardProps"`)
    props_type: String,
    /// High-level component type hint
    component_type: String,
    /// Typed props list
    props: Vec<PropContext>,
    /// JSX body lines (simplified render tree)
    jsx_body: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PropContext {
    name: String,
    ts_type: String,
    required: bool,
    default_value: Option<String>,
    description: Option<String>,
}

// ---------------------------------------------------------------------------
// SpecIRGenerator impl
// ---------------------------------------------------------------------------

/// @spec projects/agentic-workflow/tech-design/core/generate/generators/react_runtime.md#source
impl SpecIRGenerator for ReactGenerator {
    fn can_generate(&self, spec: &SpecIR) -> bool {
        matches!(spec, SpecIR::Wireframe { .. })
    }

    fn template_dir(&self) -> &'static str {
        "react"
    }

    fn generate_from_ir(
        &self,
        spec: &SpecIR,
        settings: &GeneratorSettings,
        engine: &TemplateEngine,
    ) -> Result<Manifest, GeneratorError> {
        let wireframe_spec = match spec {
            SpecIR::Wireframe { spec, .. } => spec,
            _ => {
                return Err(GeneratorError::SchemaError(
                    "ReactGenerator: expected SpecIR::Wireframe variant".into(),
                ))
            }
        };

        let mut manifest = Manifest::new();
        let ctx = build_context(wireframe_spec, settings);

        let component_name = ctx.component_name.clone();

        let files: Vec<(String, String, Box<dyn Fn(&ReactContext) -> String>)> = vec![
            (
                format!("{}.tsx.j2", component_name),
                format!("{}.tsx", component_name),
                Box::new(|c| generate_component_tsx(c)),
            ),
            (
                format!("{}.types.ts.j2", component_name),
                format!("{}.types.ts", component_name),
                Box::new(|c| generate_types_ts(c)),
            ),
            (
                "index.ts.j2".to_string(),
                "index.ts".to_string(),
                Box::new(|c| generate_index_ts(c)),
            ),
        ];

        for (template, output, inline_gen) in &files {
            let output_path = settings.output_dir.join(output);

            if output_path.exists() {
                match settings.overwrite_policy {
                    OverwritePolicy::Error => {
                        return Err(GeneratorError::OverwriteNotAllowed(output_path));
                    }
                    OverwritePolicy::Skip => {
                        manifest.add(GeneratedFile::skipped(output_path));
                        continue;
                    }
                    OverwritePolicy::Overwrite => {}
                }
            }

            let template_name = format!("{}/{}", self.template_dir(), template);
            let content = if engine.has_template(&template_name) {
                engine.render(&template_name, &ctx).map_err(|e| {
                    GeneratorError::TemplateRenderError {
                        template: template_name.clone(),
                        message: e.to_string(),
                    }
                })?
            } else {
                inline_gen(&ctx)
            };

            manifest.add(GeneratedFile::written(output_path, &content));
        }

        Ok(manifest)
    }
}

// ---------------------------------------------------------------------------
// Context builder
// ---------------------------------------------------------------------------

fn build_context(spec: &WireframeSpec, settings: &GeneratorSettings) -> ReactContext {
    let component_name = if spec.name.is_empty() {
        to_pascal_case(&settings.name)
    } else {
        to_pascal_case(&spec.name)
    };

    let props_type = format!("{}Props", component_name);

    let props: Vec<PropContext> = spec.props.iter().map(|p| prop_context(p)).collect();

    let jsx_body = render_jsx_body(&spec.layout, 2);

    ReactContext {
        component_name,
        props_type,
        component_type: spec.component_type.clone(),
        props,
        jsx_body,
    }
}

fn prop_context(p: &PropDef) -> PropContext {
    PropContext {
        name: p.name.clone(),
        ts_type: p.prop_type.clone(),
        required: p.required,
        default_value: p.default_value.clone(),
        description: p.description.clone(),
    }
}

/// Recursively render a wireframe layout tree into JSX line strings.
///
/// Labels are rendered as `data-label` attributes on container elements so
/// that the generated TSX is valid JSX syntax (C-style comments inside opening
/// tags are not valid JSX).
fn render_jsx_body(nodes: &[WireframeNode], indent: usize) -> Vec<String> {
    let pad = "  ".repeat(indent);
    let mut lines = Vec::new();

    for node in nodes {
        // Render label as a data-label attribute (valid JSX) rather than an
        // inline C-style comment which would produce invalid JSX syntax.
        let label_attr = node
            .label
            .as_deref()
            .map(|l| format!(" data-label=\"{}\"", l))
            .unwrap_or_default();

        match node.kind.as_str() {
            "text" => {
                // For text nodes the label IS the content — no extra attribute needed.
                lines.push(format!(
                    "{}<span>{}</span>",
                    pad,
                    node.label.as_deref().unwrap_or("TODO")
                ));
            }
            "button" => {
                // For button nodes the label IS the button text.
                lines.push(format!(
                    "{}<button type=\"button\">{}</button>",
                    pad,
                    node.label.as_deref().unwrap_or("Action")
                ));
            }
            "input" => {
                lines.push(format!(
                    "{}<input placeholder=\"{}\" />",
                    pad,
                    node.label.as_deref().unwrap_or("")
                ));
            }
            "list" => {
                lines.push(format!("{}<ul{}>", pad, label_attr));
                lines.push(format!("{}  {{/* TODO: render items */}}", pad));
                lines.push(format!("{}</ul>", pad));
            }
            kind => {
                // Generic container
                let tag = match kind {
                    "section" | "article" | "header" | "footer" | "main" | "nav" | "aside" => {
                        kind.to_string()
                    }
                    "form" => "form".to_string(),
                    _ => "div".to_string(),
                };
                if node.children.is_empty() {
                    lines.push(format!("{}<{}{} />", pad, tag, label_attr));
                } else {
                    lines.push(format!("{}<{}{}>", pad, tag, label_attr));
                    lines.extend(render_jsx_body(&node.children, indent + 1));
                    lines.push(format!("{}</{}>", pad, tag));
                }
            }
        }
    }

    lines
}

// ---------------------------------------------------------------------------
// Inline generators
// ---------------------------------------------------------------------------

fn generate_component_tsx(ctx: &ReactContext) -> String {
    let import_line = format!(
        "import type {{ {} }} from \"./{}.types\";",
        ctx.props_type, ctx.component_name
    );

    // Build props destructuring
    let destructure = if ctx.props.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = ctx
            .props
            .iter()
            .map(|p| {
                if let Some(default) = &p.default_value {
                    format!("{} = {}", p.name, default)
                } else {
                    p.name.clone()
                }
            })
            .collect();
        format!("{{ {} }}", parts.join(", "))
    };

    let props_param = if ctx.props.is_empty() {
        // Use the generated props interface even when empty to keep component
        // and types files consistent.
        format!("_props: {}", ctx.props_type)
    } else {
        format!("{}: {}", destructure, ctx.props_type)
    };

    // Build JSX body
    let body = if ctx.jsx_body.is_empty() {
        "    {/* TODO: implement */}".to_string()
    } else {
        ctx.jsx_body.join("\n")
    };

    // Component type comment
    let type_comment = if ctx.component_type.is_empty() {
        String::new()
    } else {
        format!("/** @componentType {} */\n", ctx.component_type)
    };

    format!(
        r#"// Generated by sdd
{import_line}

{type_comment}export function {name}({props_param}) {{
  return (
    <div>
{body}
    </div>
  );
}}

export default {name};
"#,
        import_line = import_line,
        type_comment = type_comment,
        name = ctx.component_name,
        props_param = props_param,
        body = body,
    )
}

fn generate_types_ts(ctx: &ReactContext) -> String {
    let props_body = if ctx.props.is_empty() {
        "  // No props defined".to_string()
    } else {
        ctx.props
            .iter()
            .map(|p| {
                let optional = if p.required { "" } else { "?" };
                let doc = p
                    .description
                    .as_ref()
                    .map(|d| format!("  /** {} */\n", d))
                    .unwrap_or_default();
                format!("{}  {}{}: {};", doc, p.name, optional, p.ts_type)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"// Generated by sdd

export interface {props_type} {{
{props_body}
}}
"#,
        props_type = ctx.props_type,
        props_body = props_body,
    )
}

fn generate_index_ts(ctx: &ReactContext) -> String {
    format!(
        r#"// Generated by sdd
export {{ {name}, default }} from "./{name}";
export type {{ {props_type} }} from "./{name}.types";
"#,
        name = ctx.component_name,
        props_type = ctx.props_type,
    )
}

// ---------------------------------------------------------------------------
// Case helpers
// ---------------------------------------------------------------------------

fn to_pascal_case(s: &str) -> String {
    use heck::ToPascalCase;
    s.to_pascal_case()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::spec_ir::{PropDef, SpecIR, SpecMetadata, WireframeNode, WireframeSpec};

    fn card_spec() -> SpecIR {
        SpecIR::Wireframe {
            spec: WireframeSpec {
                name: "UserCard".into(),
                component_type: "card".into(),
                props: vec![
                    PropDef {
                        name: "userId".into(),
                        prop_type: "number".into(),
                        required: true,
                        default_value: None,
                        description: Some("The user ID to display".into()),
                    },
                    PropDef {
                        name: "showAvatar".into(),
                        prop_type: "boolean".into(),
                        required: false,
                        default_value: Some("true".into()),
                        description: None,
                    },
                ],
                layout: vec![WireframeNode {
                    kind: "div".into(),
                    label: Some("card-body".into()),
                    children: vec![
                        WireframeNode {
                            kind: "text".into(),
                            label: Some("User name".into()),
                            children: vec![],
                        },
                        WireframeNode {
                            kind: "button".into(),
                            label: Some("Edit".into()),
                            children: vec![],
                        },
                    ],
                }],
            },
            metadata: SpecMetadata::default(),
        }
    }

    #[test]
    fn test_can_generate_wireframe() {
        let gen = ReactGenerator::new();
        assert!(gen.can_generate(&card_spec()));
    }

    #[test]
    fn test_cannot_generate_non_wireframe() {
        use crate::generate::schema::JsonSchema;
        let gen = ReactGenerator::new();
        let api_spec = SpecIR::Api {
            schema: JsonSchema::default(),
            metadata: SpecMetadata::default(),
        };
        assert!(!gen.can_generate(&api_spec));
    }

    #[test]
    fn test_generate_produces_three_files() {
        let spec = card_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_react_gen"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = ReactGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        assert_eq!(manifest.files.len(), 3);
        let names: Vec<String> = manifest
            .files
            .keys()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert!(names.contains(&"UserCard.tsx".to_string()));
        assert!(names.contains(&"UserCard.types.ts".to_string()));
        assert!(names.contains(&"index.ts".to_string()));
    }

    #[test]
    fn test_render_wireframe_file_selects_tsx_body() {
        let spec = match card_spec() {
            SpecIR::Wireframe { spec, .. } => spec,
            _ => unreachable!(),
        };

        let content = render_react_wireframe_file(&spec, std::path::Path::new("src/UserCard.tsx"))
            .expect("render TSX");

        assert!(content.contains("export function UserCard"));
        assert!(content.contains("<button type=\"button\">Edit</button>"));
    }

    #[test]
    fn test_component_tsx_content() {
        let spec = card_spec();
        let settings = GeneratorSettings {
            output_dir: std::path::PathBuf::from("/tmp/test_react_content"),
            ..Default::default()
        };
        let engine = crate::generate::engine::TemplateEngine::empty();
        let gen = ReactGenerator::new();
        let manifest = gen.generate_from_ir(&spec, &settings, &engine).unwrap();

        let tsx = manifest
            .files
            .values()
            .find(|f| f.path.file_name().unwrap() == "UserCard.tsx")
            .expect("UserCard.tsx not found");

        // Content hash should be set (meaning content was written)
        assert!(tsx.content_hash.is_some());
    }

    #[test]
    fn test_component_tsx_structure() {
        // Test structural requirements of the inline generator directly.
        let ctx = build_context(
            match &card_spec() {
                SpecIR::Wireframe { spec, .. } => spec,
                _ => unreachable!(),
            },
            &GeneratorSettings::default(),
        );

        let content = generate_component_tsx(&ctx);
        assert!(content.contains("import type"), "should have import line");
        assert!(
            content.contains("export function UserCard"),
            "should have named export"
        );
        assert!(
            content.contains("export default UserCard"),
            "should have default export"
        );
        assert!(
            content.contains("@componentType"),
            "should have componentType JSDoc"
        );
        // No C-style comments inside JSX opening tags (invalid JSX syntax)
        assert!(
            !content.contains(" /* "),
            "must not have C-style comments inside opening tags"
        );
    }

    #[test]
    fn test_types_ts_props_interface() {
        let ctx = build_context(
            match &card_spec() {
                SpecIR::Wireframe { spec, .. } => spec,
                _ => unreachable!(),
            },
            &GeneratorSettings::default(),
        );

        let types_output = generate_types_ts(&ctx);
        assert!(types_output.contains("interface UserCardProps"));
        assert!(types_output.contains("userId: number"));
        assert!(types_output.contains("showAvatar?: boolean"));
    }

    #[test]
    fn test_index_ts_exports() {
        let ctx = build_context(
            match &card_spec() {
                SpecIR::Wireframe { spec, .. } => spec,
                _ => unreachable!(),
            },
            &GeneratorSettings::default(),
        );

        let index_output = generate_index_ts(&ctx);
        assert!(index_output.contains("UserCard"));
        assert!(index_output.contains("UserCardProps"));
    }

    #[test]
    fn test_jsx_body_render() {
        let nodes = vec![
            WireframeNode {
                kind: "text".into(),
                label: Some("Hello".into()),
                children: vec![],
            },
            WireframeNode {
                kind: "button".into(),
                label: Some("Click me".into()),
                children: vec![],
            },
        ];
        let lines = render_jsx_body(&nodes, 2);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("<span")));
        assert!(lines.iter().any(|l| l.contains("<button")));
    }
}
// CODEGEN-END
