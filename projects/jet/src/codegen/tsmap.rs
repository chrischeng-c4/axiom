// SPEC-MANAGED: .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
// <HANDWRITE gap="standardize:projects-jet-src-codegen-tsmap-rs" tracker="standardize-gap-projects-jet-src-codegen-tsmap-rs" reason="Existing hand-written code in projects/jet/src/codegen/tsmap.rs requires tracked generator coverage.">
//! Maps an OpenAPI [`Schema`] to a TypeScript type expression.
//!
//! Named declarations for top-level component schemas are produced by
//! `types_emit`; this module returns the *expression* form used inside those
//! declarations, function signatures, and hook generics.

use crate::codegen::names::{self, to_pascal};
use crate::codegen::openapi::{AdditionalProperties, RefOr, Schema};
use std::collections::BTreeMap;

/// Maps an OpenAPI component-schema key to its final TypeScript type name.
#[derive(Debug, Default)]
pub struct TypeMap {
    pub names: BTreeMap<String, String>,
}

impl TypeMap {
    /// Resolve a `#/components/schemas/<key>` reference to its TS type name.
    /// Non-schema or external references yield `None`.
    pub fn resolve_ref(&self, reference: &str) -> Option<String> {
        let key = reference.strip_prefix("#/components/schemas/")?;
        Some(
            self.names
                .get(key)
                .cloned()
                .unwrap_or_else(|| to_pascal(key)),
        )
    }
}

/// TypeScript type expression for a schema (or `$ref`), including nullability.
///
/// @spec .aw/tech-design/projects/jet/interfaces/cli/openapi-client-codegen-types-fetch-client-react-query-hooks.md#logic
pub fn type_expr(node: &RefOr<Schema>, tm: &TypeMap) -> String {
    match node {
        RefOr::Ref(r) => match tm.resolve_ref(&r.reference) {
            Some(name) => name,
            // Out-of-subset reference (parameters/responses/external): fail soft.
            None => format!("unknown /* unsupported $ref {} */", r.reference),
        },
        RefOr::Item(schema) => {
            let base = base_expr(schema, tm);
            if schema.is_nullable() && base != "null" && !base.starts_with("unknown") {
                format!("{base} | null")
            } else {
                base
            }
        }
    }
}

fn base_expr(schema: &Schema, tm: &TypeMap) -> String {
    if !schema.all_of.is_empty() {
        return join(&schema.all_of, " & ", tm);
    }
    if !schema.one_of.is_empty() {
        return join(&schema.one_of, " | ", tm);
    }
    if !schema.any_of.is_empty() {
        return join(&schema.any_of, " | ", tm);
    }
    if !schema.enum_values.is_empty() {
        return enum_union(schema);
    }

    let types = schema.type_names();
    if types.len() > 1 {
        return types
            .iter()
            .map(|t| scalar_expr(t, schema))
            .collect::<Vec<_>>()
            .join(" | ");
    }

    match types.first().map(String::as_str) {
        Some("object") => object_expr(schema, tm),
        Some("array") => array_expr(schema, tm),
        Some("string") => string_expr(schema),
        Some("integer") | Some("number") => "number".to_string(),
        Some("boolean") => "boolean".to_string(),
        Some("null") => "null".to_string(),
        Some(_) => "unknown".to_string(),
        None => {
            if !schema.properties.is_empty() || schema.additional_properties.is_some() {
                object_expr(schema, tm)
            } else {
                "unknown".to_string()
            }
        }
    }
}

/// Scalar mapping for a single type name (used for 3.1 multi-type arrays).
fn scalar_expr(ty: &str, schema: &Schema) -> String {
    match ty {
        "string" => string_expr(schema),
        "integer" | "number" => "number".to_string(),
        "boolean" => "boolean".to_string(),
        "null" => "null".to_string(),
        _ => "unknown".to_string(),
    }
}

fn string_expr(schema: &Schema) -> String {
    match schema.format.as_deref() {
        Some("binary") => "Blob".to_string(),
        _ => "string".to_string(),
    }
}

fn array_expr(schema: &Schema, tm: &TypeMap) -> String {
    match &schema.items {
        Some(items) => {
            let inner = type_expr(items, tm);
            if inner.contains(" | ") || inner.contains(" & ") {
                format!("({inner})[]")
            } else {
                format!("{inner}[]")
            }
        }
        None => "unknown[]".to_string(),
    }
}

/// Inline object literal type: `{ a: T; b?: U; [key: string]: V }`.
pub fn object_expr(schema: &Schema, tm: &TypeMap) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (key, prop) in &schema.properties {
        let optional = !schema.required.iter().any(|r| r == key);
        let t = type_expr(prop, tm);
        parts.push(format!(
            "{}{}: {}",
            names::prop_key(key),
            if optional { "?" } else { "" },
            t
        ));
    }
    match &schema.additional_properties {
        Some(AdditionalProperties::Bool(true)) => parts.push("[key: string]: unknown".to_string()),
        Some(AdditionalProperties::Schema(s)) => {
            parts.push(format!("[key: string]: {}", type_expr(s, tm)));
        }
        _ => {}
    }
    if parts.is_empty() {
        "Record<string, unknown>".to_string()
    } else {
        format!("{{ {} }}", parts.join("; "))
    }
}

fn enum_union(schema: &Schema) -> String {
    let mut members: Vec<String> = schema
        .enum_values
        .iter()
        .map(|v| match v {
            serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "null".to_string(),
            _ => "unknown".to_string(),
        })
        .collect();
    if members.is_empty() {
        members.push("never".to_string());
    }
    members.join(" | ")
}

fn join(items: &[RefOr<Schema>], sep: &str, tm: &TypeMap) -> String {
    items
        .iter()
        .map(|i| type_expr(i, tm))
        .collect::<Vec<_>>()
        .join(sep)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(json: &str) -> RefOr<Schema> {
        serde_json::from_str(json).unwrap()
    }

    fn tm() -> TypeMap {
        let mut names = BTreeMap::new();
        names.insert("Pet".to_string(), "Pet".to_string());
        TypeMap { names }
    }

    #[test]
    fn primitives_and_formats() {
        assert_eq!(
            type_expr(&schema(r##"{"type":"string"}"##), &tm()),
            "string"
        );
        assert_eq!(
            type_expr(&schema(r##"{"type":"integer"}"##), &tm()),
            "number"
        );
        assert_eq!(
            type_expr(&schema(r##"{"type":"boolean"}"##), &tm()),
            "boolean"
        );
        assert_eq!(
            type_expr(&schema(r##"{"type":"string","format":"binary"}"##), &tm()),
            "Blob"
        );
        assert_eq!(type_expr(&schema(r##"{}"##), &tm()), "unknown");
    }

    #[test]
    fn ref_resolves_to_named_type() {
        assert_eq!(
            type_expr(&schema(r##"{"$ref":"#/components/schemas/Pet"}"##), &tm()),
            "Pet"
        );
    }

    #[test]
    fn array_and_nullable() {
        assert_eq!(
            type_expr(
                &schema(r##"{"type":"array","items":{"$ref":"#/components/schemas/Pet"}}"##),
                &tm()
            ),
            "Pet[]"
        );
        assert_eq!(
            type_expr(&schema(r##"{"type":"string","nullable":true}"##), &tm()),
            "string | null"
        );
        assert_eq!(
            type_expr(&schema(r##"{"type":["string","null"]}"##), &tm()),
            "string | null"
        );
        // array of nullable union is parenthesized
        assert_eq!(
            type_expr(
                &schema(r##"{"type":"array","items":{"type":"string","nullable":true}}"##),
                &tm()
            ),
            "(string | null)[]"
        );
    }

    #[test]
    fn enum_one_of_all_of() {
        assert_eq!(
            type_expr(&schema(r##"{"type":"string","enum":["a","b"]}"##), &tm()),
            "\"a\" | \"b\""
        );
        assert_eq!(
            type_expr(
                &schema(r##"{"oneOf":[{"type":"string"},{"type":"integer"}]}"##),
                &tm()
            ),
            "string | number"
        );
        assert_eq!(
            type_expr(
                &schema(
                    r##"{"allOf":[{"$ref":"#/components/schemas/Pet"},{"type":"object","properties":{"x":{"type":"integer"}}}]}"##
                ),
                &tm()
            ),
            "Pet & { x?: number }"
        );
    }

    #[test]
    fn object_required_and_additional_properties() {
        assert_eq!(
            type_expr(
                &schema(
                    r##"{"type":"object","properties":{"id":{"type":"integer"},"tag":{"type":"string"}},"required":["id"]}"##
                ),
                &tm()
            ),
            "{ id: number; tag?: string }"
        );
        assert_eq!(
            type_expr(
                &schema(r##"{"type":"object","additionalProperties":{"type":"integer"}}"##),
                &tm()
            ),
            "{ [key: string]: number }"
        );
    }
}
// </HANDWRITE>
