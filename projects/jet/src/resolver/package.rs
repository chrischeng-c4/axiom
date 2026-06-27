// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
// CODEGEN-BEGIN
use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Simplified package.json structure
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
#[derive(Debug, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub main: Option<String>,
    pub module: Option<String>,
    pub exports: Option<serde_json::Value>,
    /// `files` — the npm publish allowlist. When present, `jet pack`/`jet
    /// publish` include ONLY paths matching these entries (plus the always-
    /// included package.json / README / LICENSE), matching npm pack semantics.
    pub files: Option<Vec<String>>,
    pub dependencies: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<serde_json::Map<String, serde_json::Value>>,
    /// `peerDependencies` — packages the consumer is expected to provide.
    /// Like `dependencies`, these must stay external in a library build.
    /// @issue #170
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Read package.json from path
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
pub fn read_package_json(path: &Path) -> Result<PackageJson> {
    let content = fs::read_to_string(path)?;
    let package: PackageJson = serde_json::from_str(&content)?;
    Ok(package)
}

/// Get the main entry point from package.json
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
pub fn get_package_main(path: &Path) -> Result<String> {
    let package = read_package_json(path)?;

    if let Some(module) = package.module {
        return Ok(module);
    }

    if let Some(main) = package.main {
        return Ok(main);
    }

    Ok("index.js".to_string())
}

/// A single library entry discovered from `package.json`.
///
/// `subpath` is the public export key (`.`, `./client`, …) the entry is
/// published under; `source` is the relative file path the bundler reads.
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
/// @issue #170
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Public export subpath, e.g. `.` or `./client`.
    pub subpath: String,
    /// Relative source path the bundler reads, e.g. `./src/index.ts`.
    pub source: String,
}

/// Enumerate the publishable library entries from a `package.json`.
///
/// Resolution order:
///   1. Every concrete (non-wildcard) `exports` subpath, resolved with the
///      supplied `conditions` (e.g. `["import", "default"]`).
///   2. If `exports` yields nothing, fall back to a single `.` entry from
///      `module` (preferred, ESM) or `main`.
///
/// Wildcard export patterns (keys containing `*`) are skipped — they require
/// a filesystem glob and are out of scope for the first library-build pass.
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
/// @issue #170
pub fn library_entries(package_json_path: &Path, conditions: &[&str]) -> Result<Vec<LibraryEntry>> {
    let package = read_package_json(package_json_path)?;
    let mut entries: Vec<LibraryEntry> = Vec::new();

    if let Some(serde_json::Value::Object(map)) = &package.exports {
        for (subpath, _) in map.iter() {
            if subpath.contains('*') {
                // Wildcard patterns need a filesystem glob — deferred.
                continue;
            }
            if let Some(source) = resolve_exports(package_json_path, Some(subpath), conditions)? {
                entries.push(LibraryEntry {
                    subpath: subpath.clone(),
                    source,
                });
            }
        }
    } else if let Some(serde_json::Value::String(source)) = &package.exports {
        // Shorthand: `"exports": "./dist/index.js"` is the `.` entry.
        entries.push(LibraryEntry {
            subpath: ".".to_string(),
            source: source.clone(),
        });
    }

    if entries.is_empty() {
        let source = package
            .module
            .clone()
            .or_else(|| package.main.clone())
            .unwrap_or_else(|| "index.js".to_string());
        entries.push(LibraryEntry {
            subpath: ".".to_string(),
            source,
        });
    }

    Ok(entries)
}

/// Collect the package names that must stay external in a library build:
/// every key of `dependencies` and `peerDependencies`. `devDependencies`
/// are intentionally excluded — they are not shipped to consumers.
/// @spec .aw/tech-design/projects/jet/semantic/jet-resolver.md#schema
/// @issue #170
pub fn external_package_names(
    package_json_path: &Path,
) -> Result<std::collections::HashSet<String>> {
    let package = read_package_json(package_json_path)?;
    let mut names = std::collections::HashSet::new();

    for map in [&package.dependencies, &package.peer_dependencies]
        .into_iter()
        .flatten()
    {
        for key in map.keys() {
            names.insert(key.clone());
        }
    }

    Ok(names)
}

/// Resolve using package.json "exports" field (modern Node.js).
///
/// `conditions` controls which export conditions are accepted (e.g.
/// `["import", "browser", "default"]`).  The caller supplies the ordered
/// list; conditions are evaluated in object-key insertion order, matching
/// the Node.js PACKAGE_EXPORTS_RESOLVE specification.
///
// @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R1
pub fn resolve_exports(
    package_json_path: &Path,
    subpath: Option<&str>,
    conditions: &[&str],
) -> Result<Option<String>> {
    let package = read_package_json(package_json_path)?;

    let exports = match package.exports {
        Some(exports) => exports,
        None => return Ok(None),
    };

    let subpath = subpath.unwrap_or(".");

    match &exports {
        serde_json::Value::String(path) if subpath == "." => {
            return Ok(Some(path.clone()));
        }

        serde_json::Value::Object(map) => {
            if let Some(value) = map.get(subpath) {
                return resolve_export_value(value, conditions);
            }

            for (pattern, value) in map.iter() {
                if let Some(matched) = match_export_pattern(pattern, subpath) {
                    if let Some(resolved) = resolve_export_value(value, conditions)? {
                        let final_path = resolved.replace('*', &matched);
                        return Ok(Some(final_path));
                    }
                }
            }

            if subpath == "." && map.contains_key(".") {
                return resolve_export_value(&map["."], conditions);
            }
        }

        _ => {}
    }

    Ok(None)
}

/// Resolve an export value, applying caller-supplied `conditions`.
///
/// Iterates object keys in JSON insertion order (preserved by `serde_json::Map`).
/// For each key that is a member of `conditions`, if the mapped value is a
/// string it is returned directly; if it is a nested object the function
/// recurses with the same `conditions` slice.  This matches the Node.js
/// PACKAGE_EXPORTS_RESOLVE spec (first-matching-condition-wins, nested
/// conditions narrow the resolution path).
///
/// Object key insertion order determines precedence; `conditions` acts as a
/// membership filter (which conditions are active), not a precedence list.
///
// @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R2
// @spec .aw/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R3
fn resolve_export_value(value: &serde_json::Value, conditions: &[&str]) -> Result<Option<String>> {
    match value {
        serde_json::Value::String(path) => Ok(Some(path.clone())),

        serde_json::Value::Object(map) => {
            // Iterate object keys in JSON insertion order (serde_json::Map with
            // preserve_order feature).  For each key, check membership in the
            // caller-supplied conditions set.  First matching key wins — this
            // matches the Node.js PACKAGE_EXPORTS_RESOLVE spec.
            for (key, v) in map.iter() {
                if conditions.contains(&key.as_str()) {
                    match v {
                        serde_json::Value::String(path) => return Ok(Some(path.clone())),
                        serde_json::Value::Object(_) => {
                            if let Some(result) = resolve_export_value(v, conditions)? {
                                return Ok(Some(result));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Ok(None)
        }

        _ => Ok(None),
    }
}

/// Match export pattern (e.g., "./features/*" matches "./features/foo")
fn match_export_pattern(pattern: &str, subpath: &str) -> Option<String> {
    if !pattern.contains('*') {
        return None;
    }

    let pattern_parts: Vec<&str> = pattern.split('*').collect();
    if pattern_parts.len() != 2 {
        return None;
    }

    let (prefix, suffix) = (pattern_parts[0], pattern_parts[1]);

    if subpath.starts_with(prefix) && subpath.ends_with(suffix) {
        let start = prefix.len();
        let end = subpath.len() - suffix.len();
        if start <= end {
            return Some(subpath[start..end].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Default conditions slice matching dev mode defaults.
    const DEV_CONDITIONS: &[&str] = &["import", "browser", "default"];

    #[test]
    fn test_read_package_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"name": "test-package", "version": "1.0.0", "main": "dist/index.js"}}"#
        )
        .unwrap();

        let package = read_package_json(file.path()).unwrap();
        assert_eq!(package.name, Some("test-package".to_string()));
        assert_eq!(package.version, Some("1.0.0".to_string()));
        assert_eq!(package.main, Some("dist/index.js".to_string()));
    }

    #[test]
    fn test_get_package_main() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "test", "main": "lib/index.js"}}"#).unwrap();

        let main = get_package_main(file.path()).unwrap();
        assert_eq!(main, "lib/index.js");
    }

    // REQ: R1
    #[test]
    fn test_resolve_exports_string() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "test", "exports": "./dist/index.js"}}"#).unwrap();

        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
        assert_eq!(result, Some("./dist/index.js".to_string()));
    }

    // REQ: R1
    #[test]
    fn test_resolve_exports_object() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": "./dist/index.js",
                    "./package.json": "./package.json"
                }}
            }}"#
        )
        .unwrap();

        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
        assert_eq!(result, Some("./dist/index.js".to_string()));

        let result2 = resolve_exports(file.path(), Some("./package.json"), DEV_CONDITIONS).unwrap();
        assert_eq!(result2, Some("./package.json".to_string()));
    }

    // REQ: R1
    #[test]
    fn test_resolve_exports_conditional() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "import": "./dist/esm/index.js",
                        "require": "./dist/cjs/index.js",
                        "default": "./dist/index.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        // With default dev conditions (import first) → selects ESM entry
        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
        assert_eq!(result, Some("./dist/esm/index.js".to_string()));
    }

    // REQ: R1
    #[test]
    fn test_resolve_exports_pattern() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    "./features/*": "./dist/features/*.js"
                }}
            }}"#
        )
        .unwrap();

        let result = resolve_exports(file.path(), Some("./features/auth"), DEV_CONDITIONS).unwrap();
        assert_eq!(result, Some("./dist/features/auth.js".to_string()));
    }

    #[test]
    fn test_match_export_pattern() {
        assert_eq!(
            match_export_pattern("./features/*", "./features/auth"),
            Some("auth".to_string())
        );
        assert_eq!(
            match_export_pattern("./lib/*.js", "./lib/utils.js"),
            Some("utils".to_string())
        );
        assert_eq!(match_export_pattern("./foo/*", "./bar/baz"), None);
    }

    // ─── T1: resolve_import_condition ────────────────────────────────────────────

    /// T1: S1 exports + conditions=[import, default] → ./esm.mjs
    // REQ: R1
    #[test]
    fn test_resolve_import_condition() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "import": "./esm.mjs",
                        "require": "./cjs.js",
                        "default": "./index.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        let result = resolve_exports(file.path(), Some("."), &["import", "default"]).unwrap();
        assert_eq!(result, Some("./esm.mjs".to_string()));
    }

    // ─── T2: resolve_require_condition ───────────────────────────────────────────

    /// T2: S1 exports + conditions=[require, default] → ./cjs.js
    // REQ: R1
    #[test]
    fn test_resolve_require_condition() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "import": "./esm.mjs",
                        "require": "./cjs.js",
                        "default": "./index.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        let result = resolve_exports(file.path(), Some("."), &["require", "default"]).unwrap();
        assert_eq!(result, Some("./cjs.js".to_string()));
    }

    // ─── T3: resolve_browser_condition ───────────────────────────────────────────

    /// T3: browser-specific exports + conditions=[browser, default] → browser entry
    // REQ: R1
    #[test]
    fn test_resolve_browser_condition() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "browser": "./browser.js",
                        "node": "./node.js",
                        "default": "./index.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        let result = resolve_exports(file.path(), Some("."), &["browser", "default"]).unwrap();
        assert_eq!(result, Some("./browser.js".to_string()));
    }

    // ─── T4: nested_node_import ──────────────────────────────────────────────────

    /// T4: S4 nested exports + conditions=[node, import, default] → recurse into
    /// node object, return ./node.mjs
    // REQ: R2
    #[test]
    fn test_nested_node_import() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "node": {{
                            "import": "./node.mjs",
                            "require": "./node.cjs"
                        }},
                        "browser": "./browser.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        let result =
            resolve_exports(file.path(), Some("."), &["node", "import", "default"]).unwrap();
        assert_eq!(result, Some("./node.mjs".to_string()));
    }

    // ─── T5: nested_skip_unmatched_block ─────────────────────────────────────────

    /// T5: S6 exports with node block, conditions=[import, default] →
    /// skip node block, return ./fallback.js via default
    // REQ: R2
    #[test]
    fn test_nested_skip_unmatched_block() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "node": {{
                            "import": "./node.mjs",
                            "require": "./node.cjs"
                        }},
                        "default": "./fallback.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        // conditions=[import, default] — "node" is not in the conditions list so
        // the node block is skipped; "default" matches at the top level.
        let result = resolve_exports(file.path(), Some("."), &["import", "default"]).unwrap();
        assert_eq!(result, Some("./fallback.js".to_string()));
    }

    // ─── T6: no_matching_condition_error ─────────────────────────────────────────

    /// T6: S3 exports — import+require only, conditions=[browser, default] →
    /// no match, returns Ok(None)
    // REQ: R5
    #[test]
    fn test_no_matching_condition_returns_none() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "import": "./esm.mjs",
                        "require": "./cjs.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        // Neither "browser" nor "default" are present in the exports object.
        let result = resolve_exports(file.path(), Some("."), &["browser", "default"]).unwrap();
        assert_eq!(result, None, "Should return None when no condition matches");
    }

    // ─── T7: string_shorthand_ignores_conditions ─────────────────────────────────

    /// T7: string exports shorthand — any conditions → return string directly
    // REQ: R1
    #[test]
    fn test_string_shorthand_ignores_conditions() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, r#"{{"name": "test", "exports": "./dist/index.js"}}"#).unwrap();

        // Even unusual conditions — string shorthand always resolves
        for conds in [
            &["require"] as &[&str],
            &["node"],
            &["browser", "default"],
            &[],
        ] {
            let result = resolve_exports(file.path(), Some("."), conds).unwrap();
            assert_eq!(
                result,
                Some("./dist/index.js".to_string()),
                "String shorthand must resolve regardless of conditions: {:?}",
                conds
            );
        }
    }

    // ─── T10: object_key_order_tiebreak ──────────────────────────────────────────

    /// T10: Object key insertion order determines precedence, conditions is
    /// just a membership filter. Given exports { "import": "./esm.mjs",
    /// "require": "./cjs.js" } (import key appears first in JSON), both
    /// [require,import] and [import,require] return ./esm.mjs because
    /// "import" is the first key in object insertion order that matches.
    // REQ: R3
    #[test]
    fn test_object_key_order_drives_precedence() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "import": "./esm.mjs",
                        "require": "./cjs.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        // Both orderings return ./esm.mjs — object key order wins
        let result_import_first =
            resolve_exports(file.path(), Some("."), &["import", "require"]).unwrap();
        assert_eq!(
            result_import_first,
            Some("./esm.mjs".to_string()),
            "Object key order wins: import appears first in JSON"
        );

        let result_require_first =
            resolve_exports(file.path(), Some("."), &["require", "import"]).unwrap();
        assert_eq!(
            result_require_first,
            Some("./esm.mjs".to_string()),
            "Conditions order is irrelevant: import still first in JSON object"
        );
    }

    // ─── T10b: conditions_membership_filter ──────────────────────────────────────

    /// T10b: Conditions acts as a membership filter — only conditions in the
    /// supplied slice are accepted. A condition key present in the exports
    /// object but absent from the conditions slice is skipped.
    // REQ: R3
    #[test]
    fn test_conditions_membership_filter() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "name": "test",
                "exports": {{
                    ".": {{
                        "node": "./node.js",
                        "import": "./esm.mjs",
                        "default": "./index.js"
                    }}
                }}
            }}"#
        )
        .unwrap();

        // "node" is not in conditions — skipped. "import" is active — wins.
        let result = resolve_exports(file.path(), Some("."), &["import", "default"]).unwrap();
        assert_eq!(
            result,
            Some("./esm.mjs".to_string()),
            "node key skipped (not in conditions); import matched"
        );
    }
}
// CODEGEN-END
