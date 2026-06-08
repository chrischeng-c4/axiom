//! Trait-impl codegen primitive.
//!
//! Emits an `impl <Trait> for <Type> { fn <method>(...) { match self { ... } } }`
//! block from a spec change entry's `trait_impl:` field. Closes the
//! trait-implementation gap that previously forced hand-written `impl` blocks
//! at sites like `projects/agentic-workflow/src/td_ast/entities.rs`.
//!
//! @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// One `<pattern> => <expression>,` line in a match self body.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MatchArm {
    pub pattern: String,
    pub expression: String,
}

/// Result of running the trait-impl generator. lines plug into CODEGEN-BEGIN/CODEGEN-END markers (R4).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraitImplOutput {
    /// Generated source lines (impl block declaration + body).
    pub lines: Vec<String>,
    /// SPEC-REF anchor string for the CODEGEN marker header.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_ref: Option<String>,
}

/// Input descriptor for the trait-impl generator, sourced from the trait_impl: field of a spec change entry (R1, R2).
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraitImplSpec {
    /// Trait identifier used in `impl <trait_name> for <type_name>`.
    pub trait_name: String,
    /// Type identifier used in `impl <trait_name> for <type_name>`.
    pub type_name: String,
    /// Ordered list of methods inside the impl block (R2, R3).
    pub methods: Vec<TraitMethod>,
}

/// One method inside a TraitImplSpec. The signature carries the full Rust fn header (everything from fn through the opening { brace). body_lookup is an ordered map from match-arm pattern (key) to expression (value); each entry emits one `<key> => <value>,` line in the match body.
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraitMethod {
    /// Method identifier, used for documentation only (the signature carries it too).
    pub name: String,
    /// Full Rust fn signature from `fn` through the opening `{`.
    pub signature: String,
    /// Ordered list of (pattern, expression) match arms.
    pub body_lookup: Vec<MatchArm>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#source
// CODEGEN-BEGIN
/// Run the trait-impl generator: produce one `impl <Trait> for <Type>` block
/// containing one fn body per `TraitMethod`. Each fn body is a `match self
/// { ... }` whose arms come from the method's `body_lookup`.
///
/// Implements the Logic flowchart from
/// `projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#logic`.
/// The `spec_ref` argument is threaded through to the output so apply-side
/// callers can populate the `SPEC-REF` line of the surrounding
/// `CODEGEN-BEGIN`/`CODEGEN-END` block (R4).
///
/// An empty `methods` list emits an `impl` block with an empty body (R3).
///
/// @spec projects/agentic-workflow/tech-design/core/generate/generators/trait-impl.md#logic
pub fn run_trait_impl(spec: &TraitImplSpec, spec_ref: Option<String>) -> TraitImplOutput {
    let mut lines: Vec<String> = Vec::new();
    if let Some(spec_ref) = spec_ref.as_deref() {
        lines.push(format!("/// @spec {}", spec_ref));
    }
    lines.push(format!(
        "impl {} for {} {{",
        spec.trait_name, spec.type_name
    ));
    for method in &spec.methods {
        lines.push(format!("    {}", method.signature));
        lines.push("        match self {".to_string());
        for arm in &method.body_lookup {
            lines.push(format!(
                "            {} => {},",
                arm.pattern, arm.expression
            ));
        }
        lines.push("        }".to_string());
        lines.push("    }".to_string());
    }
    lines.push("}".to_string());
    TraitImplOutput { lines, spec_ref }
}
// CODEGEN-END
#[cfg(test)]
mod tests {
    use super::*;

    /// R3 — empty methods list emits an impl block with empty body.
    #[test]
    fn empty_methods_emit_empty_impl_body() {
        let spec = TraitImplSpec {
            trait_name: "MyTrait".into(),
            type_name: "MyType".into(),
            methods: Vec::new(),
        };
        let out = run_trait_impl(&spec, None);
        assert_eq!(
            out.lines,
            vec!["impl MyTrait for MyType {".to_string(), "}".to_string()],
        );
    }

    /// R1 + R2 — single method, single arm: emits impl + fn + match dispatch.
    #[test]
    fn single_method_single_arm() {
        let spec = TraitImplSpec {
            trait_name: "Greeting".into(),
            type_name: "Greeter".into(),
            methods: vec![TraitMethod {
                name: "hello".into(),
                signature: "fn hello(&self) -> &'static str {".into(),
                body_lookup: vec![MatchArm {
                    pattern: "Greeter::English".into(),
                    expression: "\"hi\"".into(),
                }],
            }],
        };
        let out = run_trait_impl(&spec, None);
        assert_eq!(
            out.lines.join("\n"),
            "impl Greeting for Greeter {\n\
             \x20\x20\x20\x20fn hello(&self) -> &'static str {\n\
             \x20\x20\x20\x20\x20\x20\x20\x20match self {\n\
             \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20Greeter::English => \"hi\",\n\
             \x20\x20\x20\x20\x20\x20\x20\x20}\n\
             \x20\x20\x20\x20}\n\
             }",
        );
    }

    /// R2 — single method, multiple arms: each arm emits its own line.
    #[test]
    fn single_method_multi_arm() {
        let spec = TraitImplSpec {
            trait_name: "Greeting".into(),
            type_name: "Greeter".into(),
            methods: vec![TraitMethod {
                name: "hello".into(),
                signature: "fn hello(&self) -> &'static str {".into(),
                body_lookup: vec![
                    MatchArm {
                        pattern: "Greeter::English".into(),
                        expression: "\"hi\"".into(),
                    },
                    MatchArm {
                        pattern: "Greeter::Spanish".into(),
                        expression: "\"hola\"".into(),
                    },
                ],
            }],
        };
        let out = run_trait_impl(&spec, None);
        let rendered = out.lines.join("\n");
        assert!(rendered.contains("Greeter::English => \"hi\","));
        assert!(rendered.contains("Greeter::Spanish => \"hola\","));
    }

    /// R2 — multiple methods, each with its own match body.
    #[test]
    fn multi_method() {
        let spec = TraitImplSpec {
            trait_name: "Pair".into(),
            type_name: "P".into(),
            methods: vec![
                TraitMethod {
                    name: "a".into(),
                    signature: "fn a(&self) -> u8 {".into(),
                    body_lookup: vec![MatchArm {
                        pattern: "P::X".into(),
                        expression: "1".into(),
                    }],
                },
                TraitMethod {
                    name: "b".into(),
                    signature: "fn b(&self) -> u8 {".into(),
                    body_lookup: vec![MatchArm {
                        pattern: "P::X".into(),
                        expression: "2".into(),
                    }],
                },
            ],
        };
        let out = run_trait_impl(&spec, None);
        let rendered = out.lines.join("\n");
        assert!(rendered.contains("fn a(&self) -> u8 {"));
        assert!(rendered.contains("fn b(&self) -> u8 {"));
        // Each fn opens its own match block — count match-self openings.
        let match_count = rendered.matches("match self {").count();
        assert_eq!(match_count, 2, "each method must open its own match block");
    }

    /// R4 — spec_ref threads through to output for SPEC-REF marker emission.
    #[test]
    fn spec_ref_threads_through() {
        let spec = TraitImplSpec {
            trait_name: "T".into(),
            type_name: "Y".into(),
            methods: Vec::new(),
        };
        let out = run_trait_impl(&spec, Some("path/to/spec.md#schema".to_string()));
        assert_eq!(out.spec_ref.as_deref(), Some("path/to/spec.md#schema"));
    }
}
