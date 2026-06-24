//! Maps an OpenAPI [`Schema`] to a Python type expression (pydantic-flavored).

use crate::ir::openapi::{AdditionalProperties, RefOr, Schema};
use crate::ir::typemap::TypeMap;

/// Python type expression for a schema (or `$ref`), including `Optional`.
pub fn type_expr(node: &RefOr<Schema>, tm: &TypeMap) -> String {
    match node {
        RefOr::Ref(r) => tm
            .resolve_ref(&r.reference)
            .unwrap_or_else(|| "Any".to_string()),
        RefOr::Item(schema) => {
            let base = base_expr(schema, tm);
            if schema.is_nullable() {
                optional(&base)
            } else {
                base
            }
        }
    }
}

/// Wrap in `Optional[...]` unless it already is (or is `Any`/`None`).
pub fn optional(ty: &str) -> String {
    if ty == "Any" || ty == "None" || ty.starts_with("Optional[") {
        ty.to_string()
    } else {
        format!("Optional[{ty}]")
    }
}

fn base_expr(schema: &Schema, tm: &TypeMap) -> String {
    if !schema.all_of.is_empty() {
        // pydantic has no intersection type; fall back to the first member.
        return schema
            .all_of
            .first()
            .map(|s| type_expr(s, tm))
            .unwrap_or_else(|| "Any".to_string());
    }
    if !schema.one_of.is_empty() {
        return union(&schema.one_of, tm);
    }
    if !schema.any_of.is_empty() {
        return union(&schema.any_of, tm);
    }
    if !schema.enum_values.is_empty() {
        return enum_literal(schema);
    }

    let types = schema.type_names();
    if types.len() > 1 {
        return types
            .iter()
            .map(|t| scalar(t, schema))
            .collect::<Vec<_>>()
            .join(" | ");
    }

    match types.first().map(String::as_str) {
        Some("object") => object_expr(schema, tm),
        Some("array") => array_expr(schema, tm),
        Some("string") => string_expr(schema),
        Some("integer") => "int".to_string(),
        Some("number") => "float".to_string(),
        Some("boolean") => "bool".to_string(),
        Some("null") => "None".to_string(),
        Some(_) => "Any".to_string(),
        None => {
            if !schema.properties.is_empty() || schema.additional_properties.is_some() {
                object_expr(schema, tm)
            } else {
                "Any".to_string()
            }
        }
    }
}

fn scalar(ty: &str, schema: &Schema) -> String {
    match ty {
        "string" => string_expr(schema),
        "integer" => "int".to_string(),
        "number" => "float".to_string(),
        "boolean" => "bool".to_string(),
        "null" => "None".to_string(),
        _ => "Any".to_string(),
    }
}

fn string_expr(schema: &Schema) -> String {
    match schema.format.as_deref() {
        Some("binary") => "bytes".to_string(),
        _ => "str".to_string(),
    }
}

fn array_expr(schema: &Schema, tm: &TypeMap) -> String {
    match &schema.items {
        Some(items) => format!("list[{}]", type_expr(items, tm)),
        None => "list[Any]".to_string(),
    }
}

/// Inline object → a typed mapping (pydantic can't synthesize a nested model
/// inline; `additionalProperties` drives the value type, else `Any`).
pub fn object_expr(schema: &Schema, tm: &TypeMap) -> String {
    match &schema.additional_properties {
        Some(AdditionalProperties::Schema(s)) => format!("dict[str, {}]", type_expr(s, tm)),
        _ => "dict[str, Any]".to_string(),
    }
}

fn enum_literal(schema: &Schema) -> String {
    let members: Vec<String> = schema
        .enum_values
        .iter()
        .map(|v| match v {
            serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            serde_json::Value::Null => "None".to_string(),
            _ => "Any".to_string(),
        })
        .collect();
    if members.is_empty() {
        "Any".to_string()
    } else {
        format!("Literal[{}]", members.join(", "))
    }
}

fn union(items: &[RefOr<Schema>], tm: &TypeMap) -> String {
    items
        .iter()
        .map(|i| type_expr(i, tm))
        .collect::<Vec<_>>()
        .join(" | ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(json: &str) -> RefOr<Schema> {
        serde_json::from_str(json).unwrap()
    }
    fn tm() -> TypeMap {
        let mut names = std::collections::BTreeMap::new();
        names.insert("Pet".to_string(), "Pet".to_string());
        TypeMap { names }
    }

    #[test]
    fn primitives_array_ref_optional() {
        assert_eq!(type_expr(&s(r##"{"type":"integer"}"##), &tm()), "int");
        assert_eq!(type_expr(&s(r##"{"type":"number"}"##), &tm()), "float");
        assert_eq!(
            type_expr(&s(r##"{"type":"string","format":"binary"}"##), &tm()),
            "bytes"
        );
        assert_eq!(
            type_expr(
                &s(r##"{"type":"array","items":{"$ref":"#/components/schemas/Pet"}}"##),
                &tm()
            ),
            "list[Pet]"
        );
        assert_eq!(
            type_expr(&s(r##"{"$ref":"#/components/schemas/Pet"}"##), &tm()),
            "Pet"
        );
        assert_eq!(
            type_expr(&s(r##"{"type":"string","nullable":true}"##), &tm()),
            "Optional[str]"
        );
        assert_eq!(
            type_expr(&s(r##"{"type":"string","enum":["a","b"]}"##), &tm()),
            "Literal[\"a\", \"b\"]"
        );
    }
}
