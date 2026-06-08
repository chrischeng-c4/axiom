---
id: projects-sdd-src-tools-analyze-suggestions-rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Analyze-code tool TDs support brownfield semantic coverage and standardization readiness."
---

# Standardized projects/agentic-workflow/src/tools/analyze/suggestions.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/analyze/suggestions.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `detect_spec_type` | projects/agentic-workflow/src/tools/analyze/suggestions.rs | function | pub | 10 | detect_spec_type(     functions: &[FunctionInfo],     classes: &[ClassInfo],     patterns: &[String], ) -> String |
| `generate_diagram_inputs` | projects/agentic-workflow/src/tools/analyze/suggestions.rs | function | pub | 170 | generate_diagram_inputs(     spec_type: &str,     functions: &[FunctionInfo],     classes: &[ClassInfo], ) -> Value |
| `generate_enrichment_prompt` | projects/agentic-workflow/src/tools/analyze/suggestions.rs | function | pub | 260 | generate_enrichment_prompt(     spec_type: &str,     functions: &[FunctionInfo],     classes: &[ClassInfo], ) -> String |
| `generate_suggestions` | projects/agentic-workflow/src/tools/analyze/suggestions.rs | function | pub | 50 | generate_suggestions(     spec_type: &str,     functions: &[FunctionInfo],     classes: &[ClassInfo],     patterns: &[String], ) -> Value |
## Source
<!-- type: source lang: rust -->

````rust
//! Suggestion generation, spec type detection, diagram inputs, and LLM enrichment prompts

use super::{ClassInfo, FunctionInfo};
use serde_json::{json, Value};

/// Detect spec type based on analysis results
pub fn detect_spec_type(
    functions: &[FunctionInfo],
    classes: &[ClassInfo],
    patterns: &[String],
) -> String {
    if patterns.contains(&"http-api".to_string()) {
        return "http-api".to_string();
    }
    if patterns.contains(&"event-driven".to_string()) {
        return "event-driven".to_string();
    }
    if patterns.contains(&"data-model".to_string()) {
        return "data-model".to_string();
    }

    let has_many_classes = classes.len() > functions.len();
    let has_async_funcs = functions.iter().any(|f| f.is_async);
    let has_crud_funcs = functions.iter().any(|f| {
        let name_lower = f.name.to_lowercase();
        name_lower.contains("create")
            || name_lower.contains("read")
            || name_lower.contains("update")
            || name_lower.contains("delete")
            || name_lower.contains("get")
            || name_lower.contains("set")
    });

    if has_many_classes {
        "data-model".to_string()
    } else if has_async_funcs && has_crud_funcs {
        "http-api".to_string()
    } else if functions.len() > 5 {
        "algorithm".to_string()
    } else {
        "utility".to_string()
    }
}

/// Generate suggestions based on analysis
pub fn generate_suggestions(
    spec_type: &str,
    functions: &[FunctionInfo],
    classes: &[ClassInfo],
    patterns: &[String],
) -> Value {
    let mut suggested_diagrams = Vec::new();
    let mut suggested_requirements = Vec::new();

    match spec_type {
        "http-api" => {
            suggested_diagrams
                .push(json!({"type": "sequence", "reason": "Show API request/response flow"}));
            if !classes.is_empty() {
                suggested_diagrams
                    .push(json!({"type": "class", "reason": "Document request/response models"}));
            }
        }
        "event-driven" => {
            suggested_diagrams.push(
                json!({"type": "sequence", "reason": "Show event publishing and handling flow"}),
            );
        }
        "data-model" => {
            suggested_diagrams.push(json!({"type": "erd", "reason": "Show entity relationships"}));
            suggested_diagrams
                .push(json!({"type": "class", "reason": "Show class structure and attributes"}));
        }
        "algorithm" => {
            suggested_diagrams
                .push(json!({"type": "flowchart", "reason": "Show algorithm logic flow"}));
            suggested_diagrams
                .push(json!({"type": "state", "reason": "Show state transitions if applicable"}));
        }
        _ => {
            if !functions.is_empty() {
                suggested_diagrams
                    .push(json!({"type": "flowchart", "reason": "Show module logic flow"}));
            }
        }
    }

    let mut req_id = 1;
    for func in functions {
        if func.doc.is_some() || !func.params.is_empty() {
            suggested_requirements.push(json!({
                "id": format!("R{}", req_id),
                "title": format!("{} function", func.name),
                "description": func.doc.clone().unwrap_or_else(|| {
                    let params_str = func.params.iter()
                        .map(|p| match &p.type_annotation {
                            Some(t) => format!("{}: {}", p.name, t),
                            None => p.name.clone(),
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret = func.return_type.clone().unwrap_or_else(|| "void".to_string());
                    format!("Function {}({}) -> {}", func.name, params_str, ret)
                }),
                "priority": if func.is_async { "high" } else { "medium" }
            }));
            req_id += 1;
        }
    }

    for class in classes {
        suggested_requirements.push(json!({
            "id": format!("R{}", req_id),
            "title": format!("{} data model", class.name),
            "description": class.doc.clone().unwrap_or_else(|| {
                let fields_str = class.fields.iter()
                    .map(|f| match &f.type_annotation {
                        Some(t) => format!("{}: {}", f.name, t),
                        None => f.name.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Data model {} with fields: {}", class.name, fields_str)
            }),
            "priority": "high"
        }));
        req_id += 1;
    }

    let extracted_types: Vec<Value> = classes.iter().map(|c| {
        json!({
            "name": c.name,
            "kind": if c.bases.is_empty() { "class" } else { "derived" },
            "bases": c.bases,
            "fields": c.fields.iter().map(|f| json!({"name": f.name, "type": f.type_annotation})).collect::<Vec<_>>()
        })
    }).collect();

    let extracted_functions: Vec<Value> = functions.iter().map(|f| {
        json!({
            "name": f.name,
            "is_async": f.is_async,
            "params": f.params.iter().map(|p| json!({"name": p.name, "type": p.type_annotation})).collect::<Vec<_>>(),
            "return_type": f.return_type,
            "decorators": f.decorators
        })
    }).collect();

    json!({
        "suggested_spec_type": spec_type,
        "detected_patterns": patterns,
        "extracted_types": extracted_types,
        "extracted_functions": extracted_functions,
        "suggested_requirements": suggested_requirements,
        "suggested_diagrams": suggested_diagrams,
        "api_spec_recommendation": match spec_type {
            "http-api" => Some("Include OpenAPI 3.1 specification"),
            "event-driven" => Some("Include AsyncAPI 2.6 specification"),
            _ => None
        }
    })
}

/// Generate structured diagram inputs based on spec_type
pub fn generate_diagram_inputs(
    spec_type: &str,
    functions: &[FunctionInfo],
    classes: &[ClassInfo],
) -> Value {
    let mut inputs = Vec::new();

    match spec_type {
        "http-api" => {
            let participants: Vec<Value> = vec![
                json!({"name": "Client", "type": "actor"}),
                json!({"name": "API", "type": "participant"}),
                json!({"name": "Service", "type": "participant"}),
            ];
            let messages: Vec<Value> = functions.iter()
                .filter(|f| f.decorators.iter().any(|d| {
                    d.contains("get") || d.contains("post") || d.contains("put") || d.contains("delete")
                }))
                .take(10)
                .map(|f| {
                    let method = f.decorators.iter()
                        .find(|d| d.contains("get") || d.contains("post") || d.contains("put") || d.contains("delete"))
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "request".to_string());
                    json!({"from": "Client", "to": "API", "message": format!("{} → {}", method, f.name)})
                })
                .collect();
            inputs.push(json!({
                "diagram_type": "sequence",
                "title": "API Request Flow",
                "participants": participants,
                "messages": messages,
            }));
        }
        "data-model" => {
            let entities: Vec<Value> = classes.iter()
                .map(|c| json!({
                    "name": c.name,
                    "fields": c.fields.iter().map(|f| json!({
                        "name": f.name,
                        "type": f.type_annotation.clone().unwrap_or_else(|| "unknown".to_string()),
                    })).collect::<Vec<_>>(),
                    "bases": c.bases,
                }))
                .collect();
            inputs.push(json!({"diagram_type": "erd", "title": "Entity Relationship Diagram", "entities": entities}));
            inputs.push(json!({
                "diagram_type": "class",
                "title": "Class Hierarchy",
                "classes": classes.iter().map(|c| json!({
                    "name": c.name,
                    "extends": c.bases,
                    "methods": c.methods.iter().map(|m| m.name.clone()).collect::<Vec<_>>(),
                    "fields": c.fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>(),
                })).collect::<Vec<Value>>(),
            }));
        }
        "algorithm" => {
            let steps: Vec<Value> = functions.iter()
                .take(10)
                .enumerate()
                .map(|(i, f)| json!({"id": format!("step{}", i), "label": f.name, "description": f.doc.clone().unwrap_or_default()}))
                .collect();
            inputs.push(
                json!({"diagram_type": "flowchart", "title": "Algorithm Flow", "steps": steps}),
            );
        }
        "event-driven" => {
            inputs.push(json!({
                "diagram_type": "sequence",
                "title": "Event Flow",
                "participants": [
                    {"name": "Publisher", "type": "participant"},
                    {"name": "EventBus", "type": "participant"},
                    {"name": "Handler", "type": "participant"},
                ],
            }));
        }
        _ => {
            if !functions.is_empty() {
                inputs.push(json!({"diagram_type": "flowchart", "title": "Module Flow"}));
            }
        }
    }

    json!(inputs)
}

/// Generate an enrichment prompt for LLM to transform AST data into rich specs
pub fn generate_enrichment_prompt(
    spec_type: &str,
    functions: &[FunctionInfo],
    classes: &[ClassInfo],
) -> String {
    let mut prompt = String::new();
    prompt
        .push_str("Based on the following code analysis, generate rich specification content:\n\n");
    prompt.push_str(&format!("Detected spec_type: {}\n\n", spec_type));

    prompt.push_str("## Extracted Functions\n\n");
    for f in functions.iter().take(20) {
        let params_str: String = f
            .params
            .iter()
            .map(|p| match &p.type_annotation {
                Some(t) => format!("{}: {}", p.name, t),
                None => p.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        let ret = f.return_type.as_deref().unwrap_or("void");
        let async_marker = if f.is_async { "async " } else { "" };
        prompt.push_str(&format!(
            "- {}fn {}({}) -> {}",
            async_marker, f.name, params_str, ret
        ));
        if let Some(ref doc) = f.doc {
            prompt.push_str(&format!(" // {}", doc));
        }
        prompt.push('\n');
    }

    prompt.push_str("\n## Extracted Types\n\n");
    for c in classes.iter().take(20) {
        prompt.push_str(&format!("- class {}", c.name));
        if !c.bases.is_empty() {
            prompt.push_str(&format!(" extends {}", c.bases.join(", ")));
        }
        let fields_str: String = c
            .fields
            .iter()
            .map(|f| match &f.type_annotation {
                Some(t) => format!("{}: {}", f.name, t),
                None => f.name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ");
        if !fields_str.is_empty() {
            prompt.push_str(&format!(" {{ {} }}", fields_str));
        }
        prompt.push('\n');
    }

    prompt.push_str("\n## Instructions\n\n");
    prompt.push_str("Generate the following for each module/class:\n");
    prompt.push_str("1. A semantic description (what it does, not just what it contains)\n");
    prompt.push_str("2. Requirements (R1, R2, ...) with clear descriptions\n");
    prompt.push_str("3. Acceptance Criteria with GIVEN/WHEN/THEN scenarios\n");
    prompt.push_str("4. Priority classification (high/medium/low)\n");

    prompt
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/analyze/suggestions.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "<handwrite-tracker:projects-sdd-src-tools-analyze-suggestions-rs>"
    description: "Analyze-code suggestion generation and enrichment helpers."
```
