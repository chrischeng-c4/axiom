//! Schema gate for the MVP release-gate summary — closes #2820.
//!
//! Acceptance (issue #2820):
//!
//!   1. Missing required summary fields fail validation. The schema
//!      declares `required` arrays at every level; this test walks
//!      the schema, picks each `required` field, and asserts a
//!      duplicate-of-the-fixture-with-that-field-removed fails
//!      validation. Catches schema drift and validator gaps.
//!   2. Summary groups blockers by MVP objective. The schema's
//!      `blockers_by_objective` property is an object keyed by
//!      objective id; this test asserts the example fixture uses
//!      objective keys (not flat arrays) and the schema requires
//!      `profile` / `fixture_id` / `kind` / `reason` per blocker.
//!   3. Schema is documented near the release profile manifests.
//!      Schema file lives at
//!      `projects/mamba/validation/schemas/release_summary.schema.json`,
//!      sibling to `validation/profiles/`.
//!
//! Validator is hand-rolled (no new dependency): walks the schema's
//! `required` + `type` + `enum` + `properties` constraints. Covers
//! the subset the schema actually uses today.
//!
//! Cheap test — single schema read + example walk. Runs in well
//! under a second; stays in the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn schemas_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("schemas")
}

fn profiles_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
}

fn schema_path() -> PathBuf {
    schemas_root().join("release_summary.schema.json")
}

fn example_path() -> PathBuf {
    schemas_root().join("release_summary.example.json")
}

fn load_json(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {} failed: {e}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {} failed: {e}", path.display()))
}

/// Returns Ok(()) iff `value` satisfies the JSON Schema `schema`.
///
/// Hand-rolled walker covering the constraints this schema uses today:
/// `type` (object, array, string, integer, boolean), `enum`, `required`,
/// `properties`, `additionalProperties` (only when `false`),
/// `minLength`, `minimum`, `minProperties`, `items`. Returns Err with a
/// human-readable path-prefixed message on the first failure.
fn validate(value: &Value, schema: &Value, path: &str) -> Result<(), String> {
    let schema_obj = schema
        .as_object()
        .ok_or_else(|| format!("{path}: schema must be an object"))?;

    if let Some(ty) = schema_obj.get("type") {
        let expected_types: Vec<&str> = match ty {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(|v| v.as_str()).collect(),
            _ => return Err(format!("{path}: schema `type` malformed")),
        };
        let actual = match value {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        };
        let matches_int_as_number = expected_types.contains(&"number") && actual == "integer";
        if !expected_types.contains(&actual) && !matches_int_as_number {
            return Err(format!(
                "{path}: expected type {expected_types:?}, got {actual:?}"
            ));
        }
    }

    if let Some(en) = schema_obj.get("enum").and_then(|v| v.as_array()) {
        if !en.iter().any(|allowed| allowed == value) {
            return Err(format!(
                "{path}: value {value} not in enum {en:?}"
            ));
        }
    }

    if let Some(min) = schema_obj.get("minimum").and_then(|v| v.as_i64()) {
        if let Some(n) = value.as_i64() {
            if n < min {
                return Err(format!("{path}: {n} < minimum {min}"));
            }
        }
    }

    if let Some(min) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
        if let Some(s) = value.as_str() {
            if (s.chars().count() as u64) < min {
                return Err(format!("{path}: string shorter than minLength {min}"));
            }
        }
    }

    if let Some(min_props) = schema_obj.get("minProperties").and_then(|v| v.as_u64()) {
        if let Some(obj) = value.as_object() {
            if (obj.len() as u64) < min_props {
                return Err(format!(
                    "{path}: object has {} properties; minProperties = {min_props}",
                    obj.len()
                ));
            }
        }
    }

    if let Some(obj) = value.as_object() {
        let properties = schema_obj
            .get("properties")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let required: BTreeSet<&str> = schema_obj
            .get("required")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        for r in &required {
            if !obj.contains_key(*r) {
                return Err(format!("{path}: missing required field {r:?}"));
            }
        }

        let additional_allowed = schema_obj
            .get("additionalProperties")
            .map(|v| !matches!(v, Value::Bool(false)))
            .unwrap_or(true);
        let additional_schema = schema_obj
            .get("additionalProperties")
            .and_then(|v| v.as_object());

        for (k, v) in obj {
            if let Some(prop_schema) = properties.get(k) {
                validate(v, prop_schema, &format!("{path}/{k}"))?;
            } else if let Some(addl) = additional_schema {
                validate(v, &Value::Object(addl.clone()), &format!("{path}/{k}"))?;
            } else if !additional_allowed {
                return Err(format!("{path}: unexpected property {k:?}"));
            }
        }
    }

    if let Some(arr) = value.as_array() {
        if let Some(items_schema) = schema_obj.get("items") {
            for (i, item) in arr.iter().enumerate() {
                validate(item, items_schema, &format!("{path}[{i}]"))?;
            }
        }
    }

    Ok(())
}

/// Walks the schema and yields every JSON Pointer path that names a
/// required field (top-level and nested through `properties`). Used to
/// drive the "missing field fails" negative cases.
fn collect_required_paths(
    schema: &Value,
    parent: &str,
    out: &mut Vec<(String, String)>,
) {
    let Some(obj) = schema.as_object() else { return };
    if let (Some(required), Some(props)) = (
        obj.get("required").and_then(|v| v.as_array()),
        obj.get("properties").and_then(|v| v.as_object()),
    ) {
        for r in required {
            if let Some(name) = r.as_str() {
                out.push((parent.to_string(), name.to_string()));
            }
        }
        for (k, v) in props {
            collect_required_paths(v, &format!("{parent}/{k}"), out);
        }
    } else if let Some(props) = obj.get("properties").and_then(|v| v.as_object()) {
        for (k, v) in props {
            collect_required_paths(v, &format!("{parent}/{k}"), out);
        }
    }
}

fn pointer_mut<'a>(value: &'a mut Value, pointer: &str) -> Option<&'a mut Value> {
    if pointer.is_empty() {
        return Some(value);
    }
    let mut cur = value;
    for token in pointer.trim_start_matches('/').split('/') {
        let token = token.replace("~1", "/").replace("~0", "~");
        match cur {
            Value::Object(map) => cur = map.get_mut(&token)?,
            Value::Array(arr) => cur = arr.get_mut(token.parse::<usize>().ok()?)?,
            _ => return None,
        }
    }
    Some(cur)
}

#[test]
fn schema_lives_next_to_profile_manifests() {
    // Acceptance: "Schema is documented near the release profile manifests."
    assert!(
        schema_path().exists(),
        "schema must live at {} (sibling to validation/profiles/)",
        schema_path().display()
    );
    assert!(
        example_path().exists(),
        "example fixture must live at {}",
        example_path().display()
    );
    assert!(
        profiles_root().exists(),
        "profile manifests must live at {} (schema is documented next to them)",
        profiles_root().display()
    );

    // Both schemas and profiles share validation/ as the parent so
    // workers resolve one root for everything.
    assert_eq!(
        schemas_root().parent(),
        profiles_root().parent(),
        "schemas/ and profiles/ must share the validation/ root"
    );
}

#[test]
fn example_summary_validates_against_schema() {
    let schema = load_json(&schema_path());
    let example = load_json(&example_path());
    validate(&example, &schema, "$")
        .expect("example summary must validate against the schema");
}

#[test]
fn dropping_any_required_field_fails_validation() {
    // Acceptance: "Missing required summary fields fail validation."
    let schema = load_json(&schema_path());
    let example = load_json(&example_path());

    let mut required_paths: Vec<(String, String)> = Vec::new();
    collect_required_paths(&schema, "", &mut required_paths);
    assert!(
        !required_paths.is_empty(),
        "schema declared no required fields — something is wrong"
    );

    for (parent, field) in &required_paths {
        let mut mutated = example.clone();
        let target = match pointer_mut(&mut mutated, parent) {
            Some(t) => t,
            None => continue,
        };
        let removed = if let Value::Object(map) = target {
            map.remove(field)
        } else {
            continue;
        };
        if removed.is_none() {
            continue;
        }
        let err = validate(&mutated, &schema, "$");
        assert!(
            err.is_err(),
            "removing required field {parent}/{field} did not fail \
             validation — schema is not enforcing the requirement"
        );
        let err_msg = err.unwrap_err();
        assert!(
            err_msg.contains(field),
            "validator error must mention the missing field name \
             ({field}); got: {err_msg}"
        );
    }
}

#[test]
fn blockers_are_grouped_by_mvp_objective() {
    // Acceptance: "Summary groups blockers by MVP objective."
    let schema = load_json(&schema_path());
    let blockers_schema = schema
        .pointer("/properties/blockers_by_objective")
        .expect("schema missing blockers_by_objective");

    let ty = blockers_schema
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    assert_eq!(
        ty, "object",
        "blockers_by_objective must be an object (keyed by objective), \
         not an array; got type {ty:?}"
    );

    let entry_required: BTreeSet<&str> = blockers_schema
        .pointer("/additionalProperties/items/required")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in ["profile", "fixture_id", "kind", "reason"] {
        assert!(
            entry_required.contains(required),
            "blockers_by_objective entries must require {required:?}; \
             got {entry_required:?}"
        );
    }

    // The example fixture exercises the grouping shape with at least
    // one objective key carrying at least one blocker entry.
    let example = load_json(&example_path());
    let blockers = example
        .get("blockers_by_objective")
        .and_then(|v| v.as_object())
        .expect("example missing blockers_by_objective object");
    assert!(
        !blockers.is_empty(),
        "example must demonstrate at least one objective grouping so the \
         schema gate exercises the structure"
    );
    for (objective, entries) in blockers {
        let arr = entries
            .as_array()
            .unwrap_or_else(|| panic!("blockers_by_objective[{objective}] must be an array"));
        assert!(
            !arr.is_empty(),
            "blockers_by_objective[{objective}] must list at least one blocker"
        );
    }
}

#[test]
fn schema_id_and_version_are_stable() {
    let schema = load_json(&schema_path());
    let id = schema
        .get("$id")
        .and_then(|v| v.as_str())
        .expect("schema missing $id");
    assert!(
        id.starts_with("https://"),
        "schema $id must be a stable URL (got {id:?}); workers pin against this id"
    );

    // Top-level schema version field must be required and ≥ 1.
    let example = load_json(&example_path());
    let version = example
        .get("schema_version")
        .and_then(|v| v.as_i64())
        .expect("example missing schema_version");
    assert!(version >= 1, "schema_version must be a positive integer");

    // Smoke check that the validator rejects a different major.
    // Constructed as a copy of the example with schema_version = 0.
    let mut broken = example.clone();
    broken["schema_version"] = json!(0);
    let err = validate(&broken, &schema, "$");
    assert!(
        err.is_err(),
        "schema_version below the floor must fail validation; \
         got Ok with broken summary"
    );
}
