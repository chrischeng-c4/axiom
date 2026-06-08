// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
// CODEGEN-BEGIN
//! JSON tree-shaking: dead code elimination for JSON imports.
//!
//! Named imports (`import { name } from './pkg.json'`) keep only used keys.
//! Default imports (`import data from './config.json'`) keep all keys.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::tree_shake::{extract_imported_names, extract_specifier, find_module_by_specifier};

fn is_json(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == "json")
        .unwrap_or(false)
}

/// Analyze JSON imports across all modules and determine which keys are used.
///
/// For each JSON module:
/// - Named imports (`import { name, version } from './pkg.json'`) use only those keys
/// - Default imports (`import data from './config.json'`) use all keys
/// - Namespace imports (`import * as pkg from './pkg.json'`) use all keys
///
/// Returns a map of JSON module path -> set of used top-level keys.
/// An empty set means "use all keys" (default/namespace import).
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn analyze_json_imports(modules: &[(PathBuf, String)]) -> HashMap<PathBuf, JsonImportUsage> {
    let mut json_usage: HashMap<PathBuf, JsonImportUsage> = HashMap::new();

    // First, identify all JSON modules
    for (path, _source) in modules {
        if is_json(path) {
            json_usage.insert(path.clone(), JsonImportUsage::NoImporters);
        }
    }

    if json_usage.is_empty() {
        return json_usage;
    }

    // Scan all non-JSON modules for imports from JSON files
    for (_path, source) in modules {
        for line in source.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("import ") {
                continue;
            }

            let specifier = extract_specifier(trimmed);
            if !specifier.ends_with(".json") {
                continue;
            }

            // Find the matching JSON module
            let target = find_module_by_specifier(&specifier, modules);
            if let Some((target_path, _)) = target {
                if !is_json(target_path) {
                    continue;
                }

                let names = extract_imported_names(trimmed);

                let entry = json_usage
                    .entry(target_path.clone())
                    .or_insert(JsonImportUsage::NoImporters);

                if names.contains(&"*".to_string()) || names.contains(&"default".to_string()) {
                    // Default or namespace import -> use all keys
                    *entry = JsonImportUsage::UseAll;
                } else if !names.is_empty() {
                    match entry {
                        JsonImportUsage::UseAll => {
                            // Already using all, no change
                        }
                        JsonImportUsage::NoImporters => {
                            *entry = JsonImportUsage::NamedKeys(names.into_iter().collect());
                        }
                        JsonImportUsage::NamedKeys(existing) => {
                            for name in names {
                                existing.insert(name);
                            }
                        }
                    }
                }
            }
        }
    }

    json_usage
}

/// How a JSON module's keys are used by importers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
#[derive(Debug, Clone, PartialEq)]
pub enum JsonImportUsage {
    /// No module imports this JSON file.
    NoImporters,
    /// At least one importer uses default/namespace import -> keep all keys.
    UseAll,
    /// Only named imports -> keep only these keys.
    NamedKeys(HashSet<String>),
}

/// Tree-shake a JSON string, keeping only the specified top-level keys.
///
/// If `used_keys` is `None` or the JSON is not an object, returns the
/// original JSON unchanged.
/// @spec .aw/tech-design/projects/jet/semantic/jet-bundler.md#schema
pub fn shake_json(json_source: &str, used_keys: Option<&HashSet<String>>) -> String {
    let keys = match used_keys {
        Some(k) if !k.is_empty() => k,
        _ => return json_source.to_string(),
    };

    let parsed: serde_json::Value = match serde_json::from_str(json_source) {
        Ok(v) => v,
        Err(err) => {
            let preview: String = json_source.chars().take(200).collect();
            tracing::warn!(
                target: "jet::bundler::json_shake",
                error = %err,
                source_len = json_source.len(),
                source_preview = %preview,
                "GH #3318 JSON source failed to parse; tree-shaking bypassed \
                 and the original (full) JSON is being returned. The bundle \
                 may be larger than intended — check the upstream JSON for \
                 corruption or invalid escapes."
            );
            return json_source.to_string();
        }
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return json_source.to_string(),
    };

    let filtered: serde_json::Map<String, serde_json::Value> = obj
        .iter()
        .filter(|(k, _)| keys.contains(k.as_str()))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    serde_json::to_string(&filtered).unwrap_or_else(|_| json_source.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_tree_shake_named_import() {
        // T19: Named import should keep only used keys
        let json_source = r#"{"name":"x","version":"1","description":"y"}"#;
        let mut used_keys = HashSet::new();
        used_keys.insert("name".to_string());

        let result = shake_json(json_source, Some(&used_keys));
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = parsed.as_object().unwrap();
        assert!(obj.contains_key("name"), "Used key 'name' should be kept");
        assert!(
            !obj.contains_key("version"),
            "Unused key 'version' should be removed"
        );
        assert!(
            !obj.contains_key("description"),
            "Unused key 'description' should be removed"
        );
        assert_eq!(obj["name"], "x");
    }

    #[test]
    fn test_json_tree_shake_default_import_keeps_all() {
        // T20: Default import should keep all keys
        let json_source = r#"{"name":"x","version":"1","description":"y"}"#;

        // None means use all (default import behavior)
        let result = shake_json(json_source, None);
        assert_eq!(result, json_source, "Default import should keep all keys");
    }

    #[test]
    fn test_json_tree_shake_empty_keys() {
        let json_source = r#"{"a":1,"b":2}"#;
        let empty_keys = HashSet::new();
        let result = shake_json(json_source, Some(&empty_keys));
        assert_eq!(result, json_source, "Empty used_keys should keep all");
    }

    #[test]
    fn test_json_tree_shake_multiple_keys() {
        let json_source =
            r#"{"name":"pkg","version":"1.0","desc":"test","main":"index.js","license":"MIT"}"#;
        let mut used_keys = HashSet::new();
        used_keys.insert("name".to_string());
        used_keys.insert("version".to_string());

        let result = shake_json(json_source, Some(&used_keys));
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("version"));
    }

    #[test]
    fn test_json_tree_shake_non_object() {
        // JSON arrays and primitives should be returned unchanged
        let array_source = r#"[1,2,3]"#;
        let mut used_keys = HashSet::new();
        used_keys.insert("name".to_string());
        let result = shake_json(array_source, Some(&used_keys));
        assert_eq!(result, array_source);
    }

    /// GH #3318 — malformed JSON used to silently bypass tree-shaking; the
    /// fallback is still correct (return original input) but now must also
    /// emit a tracing::warn. Asserts on observable behavior.
    #[test]
    fn gh3318_shake_json_malformed_returns_original() {
        let malformed = r#"{"name":"x","version""#;
        let mut used_keys = HashSet::new();
        used_keys.insert("name".to_string());
        let result = shake_json(malformed, Some(&used_keys));
        assert_eq!(
            result, malformed,
            "malformed JSON must return original input unchanged"
        );
    }

    /// GH #3318 — happy path round-trip to guard against the warn-path
    /// disturbing the success branch.
    #[test]
    fn gh3318_shake_json_valid_still_shakes() {
        let valid = r#"{"name":"x","version":"1","desc":"y"}"#;
        let mut used_keys = HashSet::new();
        used_keys.insert("name".to_string());
        let result = shake_json(valid, Some(&used_keys));
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let obj = parsed.as_object().unwrap();
        assert!(obj.contains_key("name"));
        assert!(!obj.contains_key("version"));
        assert!(!obj.contains_key("desc"));
    }

    #[test]
    fn test_analyze_json_imports_named() {
        // Use paths that match what find_module_by_specifier expects:
        // specifier "./pkg.json" matches path "src/pkg.json" via ends_with
        let modules = vec![
            (
                PathBuf::from("src/app.js"),
                "import { name } from './pkg.json';\n".to_string(),
            ),
            (
                PathBuf::from("src/pkg.json"),
                r#"{"name":"x","version":"1","desc":"y"}"#.to_string(),
            ),
        ];

        let usage = analyze_json_imports(&modules);
        let pkg_usage = usage.get(&PathBuf::from("src/pkg.json")).unwrap();
        match pkg_usage {
            JsonImportUsage::NamedKeys(keys) => {
                assert!(keys.contains("name"), "Should have 'name' key");
                assert!(!keys.contains("version"), "Should not have 'version' key");
            }
            other => panic!("Expected NamedKeys, got {:?}", other),
        }
    }

    #[test]
    fn test_analyze_json_imports_default() {
        let modules = vec![
            (
                PathBuf::from("src/app.js"),
                "import data from './config.json';\n".to_string(),
            ),
            (
                PathBuf::from("src/config.json"),
                r#"{"key":"value"}"#.to_string(),
            ),
        ];

        let usage = analyze_json_imports(&modules);
        let config_usage = usage.get(&PathBuf::from("src/config.json")).unwrap();
        assert_eq!(
            *config_usage,
            JsonImportUsage::UseAll,
            "Default import should use all keys"
        );
    }
}
// CODEGEN-END
