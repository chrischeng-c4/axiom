//! Kubernetes structural-schema normalization shared by service CRDs.

use serde_json::{json, Value};

/// Rewrite schemars' unsigned integer formats into Kubernetes-compatible
/// integer schemas while preserving their non-negative contract.
///
/// Kubernetes structural OpenAPI does not recognize `uint32` or `uint64`.
/// Schemars emits them for Rust unsigned types, so every service CRD must
/// remove the format and retain unsigned semantics with `minimum: 0`.
pub fn normalize_unsigned_integer_formats(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if matches!(
                map.get("format").and_then(|value| value.as_str()),
                Some("uint32" | "uint64")
            ) {
                map.remove("format");
                map.entry("minimum").or_insert_with(|| json!(0));
            }
            for child in map.values_mut() {
                normalize_unsigned_integer_formats(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                normalize_unsigned_integer_formats(child);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_unsigned_formats_recursively_and_keeps_nonnegative_semantics() {
        let mut schema = json!({
            "properties": {
                "replicas": { "type": "integer", "format": "uint32" },
                "retention": { "type": "integer", "format": "uint64", "minimum": 1 },
            },
            "items": [{ "type": "integer", "format": "uint32" }],
        });

        normalize_unsigned_integer_formats(&mut schema);

        assert_eq!(schema["properties"]["replicas"]["type"], "integer");
        assert!(schema["properties"]["replicas"].get("format").is_none());
        assert_eq!(schema["properties"]["replicas"]["minimum"], 0);
        assert_eq!(schema["properties"]["retention"]["minimum"], 1);
        assert!(schema["items"][0].get("format").is_none());
        assert_eq!(schema["items"][0]["minimum"], 0);
    }
}
