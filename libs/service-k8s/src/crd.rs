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

/// Quote plain scalars that Kubernetes' YAML 1.1 decoder would coerce to a
/// boolean even though the CRD JSON schema declares them as strings.
///
/// `serde_yaml` follows YAML 1.2 and therefore emits strings such as `off`
/// without quotes. Kubernetes still accepts YAML 1.1 input, where `off`,
/// `on`, `yes`, and `no` (including their single-letter forms) are booleans.
/// The API server then rejects a string schema default such as `default: off`
/// because it arrives as `false`. JSON booleans serialize as `true`/`false`,
/// so quoting these legacy spellings cannot change a real boolean value.
pub fn quote_yaml_1_1_boolean_like_strings(yaml: &str) -> String {
    let trailing_newline = yaml.ends_with('\n');
    let mut normalized = yaml
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let scalar = if let Some((_, value)) = trimmed.split_once(": ") {
                Some(value)
            } else {
                trimmed.strip_prefix("- ")
            };

            match scalar {
                Some(value)
                    if matches!(
                        value.to_ascii_lowercase().as_str(),
                        "y" | "yes" | "n" | "no" | "on" | "off"
                    ) =>
                {
                    let prefix_len = line.len() - value.len();
                    format!("{}\"{value}\"", &line[..prefix_len])
                }
                _ => line.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if trailing_newline {
        normalized.push('\n');
    }
    normalized
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

    #[test]
    fn quotes_yaml_1_1_boolean_like_strings_without_touching_real_booleans() {
        let yaml = "properties:\n  auth:\n    default: off\n    enum:\n    - off\n    - required\n    description: mode defaults to: off\n  enabled:\n    default: false\n";

        let normalized = quote_yaml_1_1_boolean_like_strings(yaml);

        assert!(normalized.contains("default: \"off\""));
        assert!(normalized.contains("- \"off\""));
        assert!(normalized.contains("description: mode defaults to: off"));
        assert!(normalized.contains("default: false"));
        assert!(normalized.ends_with('\n'));
    }
}
