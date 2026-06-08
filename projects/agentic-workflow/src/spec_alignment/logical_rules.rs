// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/logical_rules.md#source
// CODEGEN-BEGIN
//! Logical consistency rules for spec alignment checking.
//!
//! Five rules operating on JSON blocks with `name` fields:
//! - `duplicate_definition`: same `name` across multiple JSON blocks
//! - `definition_conflict_required`: differing `required` arrays
//! - `definition_conflict_field_name`: near-match property keys (edit distance <= 2)
//! - `definition_conflict_schema`: type/enum/format conflicts on same field
//! - `rpc_field_consistency`: `x-*` extension mismatches

use std::collections::HashMap;

use super::models::{SpecDocument, Violation, ViolationKind};

/// A named JSON definition extracted from a code block.
struct NamedDefinition {
    /// The `name` field value.
    name: String,
    /// Line number of the code block.
    line: usize,
    /// The parsed JSON value.
    value: serde_json::Value,
}

/// Run all logical consistency rules against a parsed `SpecDocument`.
///
/// Returns a list of violations found.
/// @spec projects/agentic-workflow/tech-design/core/interfaces/spec_alignment/logical_rules.md#source
pub fn check(doc: &SpecDocument) -> Vec<Violation> {
    let definitions = collect_named_definitions(doc);

    // Group by name
    let mut groups: HashMap<String, Vec<&NamedDefinition>> = HashMap::new();
    for def in &definitions {
        groups.entry(def.name.clone()).or_default().push(def);
    }

    let mut violations = Vec::new();

    for (name, defs) in &groups {
        if defs.len() < 2 {
            continue;
        }

        // R5: duplicate_definition
        let lines: Vec<usize> = defs.iter().map(|d| d.line).collect();
        violations.push(Violation {
            kind: ViolationKind::DuplicateDefinition,
            message: format!("Duplicate definition '{}' found at lines {:?}", name, lines),
            heading: None,
            line: Some(lines[0]),
            lines: Some(lines),
            name: Some(name.clone()),
            expected_lang: None,
            field: None,
            details: None,
        });

        // R6: definition_conflict_required
        check_required_conflicts(name, defs, &mut violations);

        // R7: definition_conflict_field_name
        check_field_name_near_matches(name, defs, &mut violations);

        // R8: definition_conflict_schema
        check_schema_conflicts(name, defs, &mut violations);

        // R9: rpc_field_consistency
        check_rpc_extension_fields(name, defs, &mut violations);

        // R19: nested schema conflicts (result.schema, params[*].schema)
        check_nested_schema_conflicts(name, defs, &mut violations);
    }

    violations
}

/// Collect all JSON code blocks with a top-level `name` field.
fn collect_named_definitions(doc: &SpecDocument) -> Vec<NamedDefinition> {
    let mut defs = Vec::new();

    for section in &doc.sections {
        for block in &section.code_blocks {
            if let Some(json) = &block.parsed_json {
                if let Some(name) = json.get("name").and_then(|n| n.as_str()) {
                    defs.push(NamedDefinition {
                        name: name.to_string(),
                        line: block.line,
                        value: json.clone(),
                    });
                }
            }
        }
    }

    defs
}

/// R6: Check if `required` arrays differ across duplicate definitions.
fn check_required_conflicts(
    name: &str,
    defs: &[&NamedDefinition],
    violations: &mut Vec<Violation>,
) {
    let required_arrays: Vec<(usize, Option<&serde_json::Value>)> = defs
        .iter()
        .map(|d| (d.line, d.value.get("required")))
        .collect();

    // Only compare if at least one definition has a `required` field
    let has_any = required_arrays.iter().any(|(_, r)| r.is_some());
    if !has_any {
        return;
    }

    // Check if all `required` arrays are identical
    let first_required = required_arrays[0].1;
    let all_same = required_arrays.iter().all(|(_, r)| r == &first_required);

    if !all_same {
        let blocks: Vec<serde_json::Value> = required_arrays
            .iter()
            .map(|(line, req)| {
                serde_json::json!({
                    "line": line,
                    "required": req.cloned().unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();

        violations.push(Violation {
            kind: ViolationKind::DefinitionConflictRequired,
            message: format!(
                "Definition '{}' has conflicting 'required' arrays across blocks",
                name
            ),
            heading: None,
            line: None,
            lines: None,
            name: Some(name.to_string()),
            expected_lang: None,
            field: None,
            details: Some(serde_json::json!({ "blocks": blocks })),
        });
    }
}

/// R7: Check for near-match property key names (edit distance <= 2).
fn check_field_name_near_matches(
    name: &str,
    defs: &[&NamedDefinition],
    violations: &mut Vec<Violation>,
) {
    // Collect all property keys across all definitions
    let mut all_keys: Vec<(String, usize)> = Vec::new();
    for def in defs {
        if let Some(props) = def.value.get("properties").and_then(|p| p.as_object()) {
            for key in props.keys() {
                all_keys.push((key.clone(), def.line));
            }
        }
        // Also check params.properties (for OpenRPC)
        if let Some(params) = def.value.get("params") {
            if let Some(items) = params.as_array() {
                for item in items {
                    if let Some(param_name) = item.get("name").and_then(|n| n.as_str()) {
                        all_keys.push((param_name.to_string(), def.line));
                    }
                }
            }
        }
    }

    // Find near-match pairs across different blocks
    let mut pairs: Vec<(String, String)> = Vec::new();
    for i in 0..all_keys.len() {
        for j in (i + 1)..all_keys.len() {
            let (key_a, line_a) = &all_keys[i];
            let (key_b, line_b) = &all_keys[j];
            // Only compare keys from different blocks and different names
            if line_a != line_b && key_a != key_b {
                let dist = edit_distance(key_a, key_b);
                if dist > 0 && dist <= 2 {
                    let pair = if key_a < key_b {
                        (key_a.clone(), key_b.clone())
                    } else {
                        (key_b.clone(), key_a.clone())
                    };
                    if !pairs.contains(&pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
    }

    if !pairs.is_empty() {
        let pair_values: Vec<serde_json::Value> = pairs
            .iter()
            .map(|(a, b)| serde_json::json!([a, b]))
            .collect();

        violations.push(Violation {
            kind: ViolationKind::DefinitionConflictFieldName,
            message: format!(
                "Definition '{}' has near-match property names: {:?}",
                name,
                pairs
                    .iter()
                    .map(|(a, b)| format!("{} vs {}", a, b))
                    .collect::<Vec<_>>()
            ),
            heading: None,
            line: None,
            lines: None,
            name: Some(name.to_string()),
            expected_lang: None,
            field: None,
            details: Some(serde_json::json!({ "pairs": pair_values })),
        });
    }
}

/// R8: Check for schema type/enum/format conflicts on the same property key.
fn check_schema_conflicts(name: &str, defs: &[&NamedDefinition], violations: &mut Vec<Violation>) {
    // Collect properties from each definition
    let mut field_schemas: HashMap<String, Vec<(usize, &serde_json::Value)>> = HashMap::new();

    for def in defs {
        if let Some(props) = def.value.get("properties").and_then(|p| p.as_object()) {
            for (key, schema) in props {
                field_schemas
                    .entry(key.clone())
                    .or_default()
                    .push((def.line, schema));
            }
        }
    }

    for (field, schemas) in &field_schemas {
        if schemas.len() < 2 {
            continue;
        }

        // Compare type, enum, and format across schemas
        let first = schemas[0].1;
        for (line, schema) in schemas.iter().skip(1) {
            let type_differs = schema.get("type") != first.get("type");
            let enum_differs = schema.get("enum") != first.get("enum");
            let format_differs = schema.get("format") != first.get("format");

            if type_differs || enum_differs || format_differs {
                violations.push(Violation {
                    kind: ViolationKind::DefinitionConflictSchema,
                    message: format!(
                        "Definition '{}' field '{}' has conflicting schema (line {} vs line {})",
                        name, field, schemas[0].0, line
                    ),
                    heading: None,
                    line: None,
                    lines: None,
                    name: Some(name.to_string()),
                    expected_lang: None,
                    field: Some(field.clone()),
                    details: Some(serde_json::json!({
                        "schemas": schemas.iter().map(|(l, s)| serde_json::json!({
                            "line": l,
                            "type": s.get("type"),
                            "enum": s.get("enum"),
                            "format": s.get("format"),
                        })).collect::<Vec<_>>()
                    })),
                });
                break; // One violation per field is enough
            }
        }
    }
}

/// R9: Check that `x-*` extension fields are identical across duplicates.
fn check_rpc_extension_fields(
    name: &str,
    defs: &[&NamedDefinition],
    violations: &mut Vec<Violation>,
) {
    // Collect all x-* fields from each definition
    let mut ext_fields: HashMap<String, Vec<(usize, &serde_json::Value)>> = HashMap::new();

    for def in defs {
        if let Some(obj) = def.value.as_object() {
            for (key, value) in obj {
                if key.starts_with("x-") {
                    ext_fields
                        .entry(key.clone())
                        .or_default()
                        .push((def.line, value));
                }
            }
        }
    }

    for (ext_key, values) in &ext_fields {
        if values.len() < 2 {
            continue;
        }

        let first_val = values[0].1;
        for (line, val) in values.iter().skip(1) {
            if val != &first_val {
                violations.push(Violation {
                    kind: ViolationKind::RpcFieldConsistency,
                    message: format!(
                        "Definition '{}' has inconsistent '{}' values (line {} vs line {})",
                        name, ext_key, values[0].0, line
                    ),
                    heading: None,
                    line: None,
                    lines: None,
                    name: Some(name.to_string()),
                    expected_lang: None,
                    field: Some(ext_key.clone()),
                    details: Some(serde_json::json!({
                        "values": values.iter().map(|(l, v)| serde_json::json!({
                            "line": l,
                            "value": v,
                        })).collect::<Vec<_>>()
                    })),
                });
                break; // One violation per extension key is enough
            }
        }
    }
}

/// R19: Check for conflicts within nested `result.schema` and `params[*].schema` structures.
///
/// Extracts embedded schemas from OpenRPC method definitions and applies the same
/// conflict detection as R6/R7/R8 (required, field name, schema type) but emits
/// `NestedSchemaConflict*` variants.
fn check_nested_schema_conflicts(
    name: &str,
    defs: &[&NamedDefinition],
    violations: &mut Vec<Violation>,
) {
    // Collect nested schemas: (line, schema_value) from result.schema and params[*].schema
    let mut nested_schemas: Vec<(usize, &serde_json::Value)> = Vec::new();

    for def in defs {
        // result.schema
        if let Some(schema) = def.value.get("result").and_then(|r| r.get("schema")) {
            nested_schemas.push((def.line, schema));
        }

        // params[*].schema
        if let Some(params) = def.value.get("params").and_then(|p| p.as_array()) {
            for param in params {
                if let Some(schema) = param.get("schema") {
                    nested_schemas.push((def.line, schema));
                }
            }
        }
    }

    if nested_schemas.len() < 2 {
        return;
    }

    // Check nested required conflicts
    check_nested_required_conflicts(name, &nested_schemas, violations);

    // Check nested schema type conflicts
    check_nested_schema_type_conflicts(name, &nested_schemas, violations);

    // Check nested field name near-matches
    check_nested_field_name_near_matches(name, &nested_schemas, violations);
}

/// Check for conflicting `required` arrays across nested schemas.
fn check_nested_required_conflicts(
    name: &str,
    schemas: &[(usize, &serde_json::Value)],
    violations: &mut Vec<Violation>,
) {
    let required_arrays: Vec<(usize, Option<&serde_json::Value>)> = schemas
        .iter()
        .map(|(line, s)| (*line, s.get("required")))
        .collect();

    let has_any = required_arrays.iter().any(|(_, r)| r.is_some());
    if !has_any {
        return;
    }

    let first_required = required_arrays[0].1;
    let all_same = required_arrays.iter().all(|(_, r)| r == &first_required);

    if !all_same {
        let blocks: Vec<serde_json::Value> = required_arrays
            .iter()
            .map(|(line, req)| {
                serde_json::json!({
                    "line": line,
                    "required": req.cloned().unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();

        violations.push(Violation {
            kind: ViolationKind::NestedSchemaConflictRequired,
            message: format!(
                "Definition '{}' has conflicting nested 'required' arrays",
                name
            ),
            heading: None,
            line: None,
            lines: None,
            name: Some(name.to_string()),
            expected_lang: None,
            field: None,
            details: Some(serde_json::json!({ "blocks": blocks })),
        });
    }
}

/// Check for type/enum/format conflicts across nested schema properties.
fn check_nested_schema_type_conflicts(
    name: &str,
    schemas: &[(usize, &serde_json::Value)],
    violations: &mut Vec<Violation>,
) {
    let mut field_schemas: HashMap<String, Vec<(usize, &serde_json::Value)>> = HashMap::new();

    for (line, schema) in schemas {
        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, prop_schema) in props {
                field_schemas
                    .entry(key.clone())
                    .or_default()
                    .push((*line, prop_schema));
            }
        }
    }

    for (field, field_defs) in &field_schemas {
        if field_defs.len() < 2 {
            continue;
        }

        let first = field_defs[0].1;
        for (line, schema) in field_defs.iter().skip(1) {
            let type_differs = schema.get("type") != first.get("type");
            let enum_differs = schema.get("enum") != first.get("enum");
            let format_differs = schema.get("format") != first.get("format");

            if type_differs || enum_differs || format_differs {
                violations.push(Violation {
                    kind: ViolationKind::NestedSchemaConflictSchema,
                    message: format!(
                        "Definition '{}' nested schema field '{}' has conflicting type (line {} vs line {})",
                        name, field, field_defs[0].0, line
                    ),
                    heading: None,
                    line: None,
                    lines: None,
                    name: Some(name.to_string()),
                    expected_lang: None,
                    field: Some(field.clone()),
                    details: Some(serde_json::json!({
                        "schemas": field_defs.iter().map(|(l, s)| serde_json::json!({
                            "line": l,
                            "type": s.get("type"),
                            "enum": s.get("enum"),
                            "format": s.get("format"),
                        })).collect::<Vec<_>>()
                    })),
                });
                break;
            }
        }
    }
}

/// Check for near-match property names across nested schemas.
fn check_nested_field_name_near_matches(
    name: &str,
    schemas: &[(usize, &serde_json::Value)],
    violations: &mut Vec<Violation>,
) {
    let mut all_keys: Vec<(String, usize)> = Vec::new();
    for (line, schema) in schemas {
        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            for key in props.keys() {
                all_keys.push((key.clone(), *line));
            }
        }
    }

    let mut pairs: Vec<(String, String)> = Vec::new();
    for i in 0..all_keys.len() {
        for j in (i + 1)..all_keys.len() {
            let (key_a, line_a) = &all_keys[i];
            let (key_b, line_b) = &all_keys[j];
            if line_a != line_b && key_a != key_b {
                let dist = edit_distance(key_a, key_b);
                if dist > 0 && dist <= 2 {
                    let pair = if key_a < key_b {
                        (key_a.clone(), key_b.clone())
                    } else {
                        (key_b.clone(), key_a.clone())
                    };
                    if !pairs.contains(&pair) {
                        pairs.push(pair);
                    }
                }
            }
        }
    }

    if !pairs.is_empty() {
        let pair_values: Vec<serde_json::Value> = pairs
            .iter()
            .map(|(a, b)| serde_json::json!([a, b]))
            .collect();

        violations.push(Violation {
            kind: ViolationKind::NestedSchemaConflictFieldName,
            message: format!(
                "Definition '{}' has near-match nested property names: {:?}",
                name,
                pairs
                    .iter()
                    .map(|(a, b)| format!("{} vs {}", a, b))
                    .collect::<Vec<_>>()
            ),
            heading: None,
            line: None,
            lines: None,
            name: Some(name.to_string()),
            expected_lang: None,
            field: None,
            details: Some(serde_json::json!({ "pairs": pair_values })),
        });
    }
}

/// Compute the Levenshtein edit distance between two strings.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    // Use a single-row DP approach
    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0; b_len + 1];

    for i in 1..=a_len {
        curr[0] = i;
        for j in 1..=b_len {
            let cost = if a_bytes[i - 1] == b_bytes[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}
// CODEGEN-END
