//! Maps an OpenAPI [`Schema`] to a Rust type expression (serde-flavored).

use crate::ir::openapi::{AdditionalProperties, RefOr, Schema};
use crate::ir::typemap::TypeMap;

/// Free-form / unsupported fallback type.
const ANY: &str = "serde_json::Value";

/// Rust type expression for a schema (or `$ref`), including `Option`.
pub fn type_expr(node: &RefOr<Schema>, tm: &TypeMap) -> String {
    match node {
        RefOr::Ref(r) => tm
            .resolve_ref(&r.reference)
            .unwrap_or_else(|| ANY.to_string()),
        RefOr::Item(schema) => {
            let base = base_expr(schema, tm);
            if schema.is_nullable() && base != ANY {
                format!("Option<{base}>")
            } else {
                base
            }
        }
    }
}

/// Wrap in `Option<...>` unless already optional or free-form.
pub fn optional(ty: &str) -> String {
    if ty == ANY || ty.starts_with("Option<") {
        ty.to_string()
    } else {
        format!("Option<{ty}>")
    }
}

fn base_expr(schema: &Schema, tm: &TypeMap) -> String {
    if !schema.all_of.is_empty() {
        return schema
            .all_of
            .first()
            .map(|s| type_expr(s, tm))
            .unwrap_or_else(|| ANY.to_string());
    }
    // Untagged unions don't map to a single Rust type → free-form value.
    if !schema.one_of.is_empty() || !schema.any_of.is_empty() {
        return ANY.to_string();
    }
    // String enums deserialize fine as a plain `String` (constraint dropped).
    if !schema.enum_values.is_empty() {
        return "String".to_string();
    }

    let types = schema.type_names();
    if types.len() > 1 {
        return ANY.to_string();
    }

    match types.first().map(String::as_str) {
        Some("object") => object_expr(schema, tm),
        Some("array") => array_expr(schema, tm),
        Some("string") => string_expr(schema),
        Some("integer") => "i64".to_string(),
        Some("number") => "f64".to_string(),
        Some("boolean") => "bool".to_string(),
        Some("null") => ANY.to_string(),
        Some(_) => ANY.to_string(),
        None => {
            if !schema.properties.is_empty() || schema.additional_properties.is_some() {
                object_expr(schema, tm)
            } else {
                ANY.to_string()
            }
        }
    }
}

fn string_expr(schema: &Schema) -> String {
    match schema.format.as_deref() {
        Some("binary") => "Vec<u8>".to_string(),
        _ => "String".to_string(),
    }
}

fn array_expr(schema: &Schema, tm: &TypeMap) -> String {
    match &schema.items {
        Some(items) => format!("Vec<{}>", type_expr(items, tm)),
        None => format!("Vec<{ANY}>"),
    }
}

/// Inline object → a typed map (`additionalProperties`) or a free-form value.
pub fn object_expr(schema: &Schema, tm: &TypeMap) -> String {
    match &schema.additional_properties {
        Some(AdditionalProperties::Schema(s)) => {
            format!("std::collections::HashMap<String, {}>", type_expr(s, tm))
        }
        _ => ANY.to_string(),
    }
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
        assert_eq!(type_expr(&s(r##"{"type":"integer"}"##), &tm()), "i64");
        assert_eq!(type_expr(&s(r##"{"type":"number"}"##), &tm()), "f64");
        assert_eq!(type_expr(&s(r##"{"type":"string"}"##), &tm()), "String");
        assert_eq!(
            type_expr(
                &s(r##"{"type":"array","items":{"$ref":"#/components/schemas/Pet"}}"##),
                &tm()
            ),
            "Vec<Pet>"
        );
        assert_eq!(
            type_expr(&s(r##"{"type":"string","nullable":true}"##), &tm()),
            "Option<String>"
        );
        assert_eq!(
            type_expr(&s(r##"{"$ref":"#/components/schemas/Pet"}"##), &tm()),
            "Pet"
        );
    }
}
