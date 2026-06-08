// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/requirement.md#source
// CODEGEN-BEGIN
//! Requirements documentation generator.
//!
//! Reads requirements frontmatter YAML from spec files and injects
//! `// REQ: R1` comment annotations at `impl_at` locations in code files.
//!
//! Missing `impl_at` → warning emitted, injection skipped.
//! This is a Category C (documentation) generator — no executable code produced.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R1

use crate::generate::marker::Lang;
use serde_yaml::Value;
use std::collections::HashMap;

/// A single impl_at location (file + optional symbol).
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/requirement.md#schema
#[derive(Debug, Clone)]
pub struct ImplAtLocation {
    pub file: String,
    pub symbol: Option<String>,
}

/// A single requirement with optional impl_at locations.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/requirement.md#schema
#[derive(Debug, Clone)]
pub struct RequirementAnnotation {
    pub id: String,
    pub text: String,
    pub spec_path: String,
    pub impl_at: Vec<ImplAtLocation>,
}

/// Output from the requirements annotation generator.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/requirement.md#schema
#[derive(Debug, Clone)]
pub struct RequirementAnnotationOutput {
    pub injections: HashMap<String, Vec<String>>,
    pub skipped: Vec<String>,
    pub warnings: Vec<String>,
}

/// Parse requirement annotations from a requirements YAML frontmatter value.
///
/// The frontmatter should have a `requirements` map with optional `impl_at` fields.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R1
pub fn parse_requirement_annotations(
    frontmatter: &Value,
    spec_path: &str,
) -> Vec<RequirementAnnotation> {
    let reqs = match frontmatter.get("requirements").and_then(|v| v.as_mapping()) {
        Some(r) => r,
        None => return Vec::new(),
    };

    let mut result = Vec::new();
    let mut req_ids: Vec<&str> = reqs.keys().filter_map(|k| k.as_str()).collect();
    req_ids.sort();

    for req_id in req_ids {
        let req = match reqs.get(req_id) {
            Some(v) => v,
            None => continue,
        };

        let text = req
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let impl_at: Vec<ImplAtLocation> = req
            .get("impl_at")
            .and_then(|v| v.as_sequence())
            .map(|locs| {
                locs.iter()
                    .filter_map(|loc| {
                        let file = loc.get("file")?.as_str()?.to_string();
                        let symbol = loc
                            .get("symbol")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        Some(ImplAtLocation { file, symbol })
                    })
                    .collect()
            })
            .unwrap_or_default();

        result.push(RequirementAnnotation {
            id: req_id.to_string(),
            text,
            spec_path: spec_path.to_string(),
            impl_at,
        });
    }

    result
}

/// Generate requirement annotation injections from parsed annotations.
///
/// For each requirement with `impl_at` locations, builds the injection map.
/// Emits warnings for requirements without `impl_at`.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R1
pub fn generate_requirement_annotations(
    annotations: &[RequirementAnnotation],
    lang: Lang,
) -> RequirementAnnotationOutput {
    let pfx = lang.line_comment();
    let mut injections: HashMap<String, Vec<String>> = HashMap::new();
    let mut skipped = Vec::new();
    let mut warnings = Vec::new();

    for ann in annotations {
        if ann.impl_at.is_empty() {
            skipped.push(ann.id.clone());
            warnings.push(format!(
                "WARNING: Requirement {} ('{}') has no impl_at — skipping annotation injection",
                ann.id, ann.text
            ));
            continue;
        }

        for loc in &ann.impl_at {
            let comment = format!(
                "{}REQ: {} — {} ({}#{})",
                pfx, ann.id, ann.text, ann.spec_path, ann.id
            );
            let spec_annotation = format!("{}@spec {}#{}", pfx, ann.spec_path, ann.id);

            let entry = injections.entry(loc.file.clone()).or_default();
            if let Some(symbol) = &loc.symbol {
                entry.push(format!("// At symbol: {}", symbol));
            }
            entry.push(comment);
            entry.push(spec_annotation);
        }
    }

    RequirementAnnotationOutput {
        injections,
        skipped,
        warnings,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_annotations() -> Vec<RequirementAnnotation> {
        vec![
            RequirementAnnotation {
                id: "R1".to_string(),
                text: "System shall process requests".to_string(),
                spec_path: "projects/agentic-workflow/tech-design/core/tools/spec.md".to_string(),
                impl_at: vec![ImplAtLocation {
                    file: "src/handler.rs".to_string(),
                    symbol: Some("process_request".to_string()),
                }],
            },
            RequirementAnnotation {
                id: "R2".to_string(),
                text: "System shall validate input".to_string(),
                spec_path: "projects/agentic-workflow/tech-design/core/tools/spec.md".to_string(),
                impl_at: vec![], // No impl_at — should produce warning
            },
        ]
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R1
    #[test]
    fn test_generates_injection_for_impl_at_location() {
        let annotations = make_annotations();
        let output = generate_requirement_annotations(&annotations, Lang::Rust);

        assert!(
            output.injections.contains_key("src/handler.rs"),
            "Should inject for R1's impl_at file"
        );
        let lines = &output.injections["src/handler.rs"];
        let joined = lines.join("\n");
        assert!(joined.contains("REQ: R1"), "Should contain REQ comment");
        assert!(
            joined.contains("@spec projects/agentic-workflow/tech-design/core/tools/spec.md#R1"),
            "Should contain @spec annotation"
        );
        assert!(
            joined.contains("process_request"),
            "Should reference the symbol"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R1
    #[test]
    fn test_emits_warning_for_missing_impl_at() {
        let annotations = make_annotations();
        let output = generate_requirement_annotations(&annotations, Lang::Rust);

        assert!(
            output.skipped.contains(&"R2".to_string()),
            "R2 should be skipped"
        );
        assert!(
            !output.warnings.is_empty(),
            "Should emit warnings for missing impl_at"
        );
        assert!(
            output.warnings[0].contains("R2"),
            "Warning should mention R2"
        );
    }

    #[test]
    fn test_parse_requirement_annotations_from_yaml() {
        let yaml_str = r#"
requirements:
  R1:
    text: "The system shall process requests"
    impl_at:
      - file: src/handler.rs
        symbol: process_request
  R2:
    text: "The system shall validate"
"#;
        let value: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let annotations = parse_requirement_annotations(
            &value,
            "projects/agentic-workflow/tech-design/core/tools/spec.md",
        );

        assert_eq!(annotations.len(), 2);
        let r1 = annotations.iter().find(|a| a.id == "R1").unwrap();
        assert_eq!(r1.impl_at.len(), 1);
        assert_eq!(r1.impl_at[0].file, "src/handler.rs");

        let r2 = annotations.iter().find(|a| a.id == "R2").unwrap();
        assert!(r2.impl_at.is_empty(), "R2 has no impl_at");
    }
}

// CODEGEN-END
