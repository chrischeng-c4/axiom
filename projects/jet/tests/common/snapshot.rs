// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
// CODEGEN-BEGIN
//! Snapshot-based assertions for conformance tests.
//!
//! Assertion style the `conformance.md` policy calls for — compare
//! a serde_json::Value against a canonical JSON blob on disk. Drift
//! produces a diff instead of a one-off `assert_eq!` failure.
//!
//! Snapshots live at
//! `projects/jet/tests/__snapshots__/<name>.json` (relative to this
//! file). First run: writes the snapshot. Subsequent runs: reads
//! and compares. Set `JET_SNAPSHOT_UPDATE=1` before running to
//! overwrite.
//!
//! Deliberately tiny — no `insta` dependency. If we ever outgrow
//! this, the call sites (`snapshot_eq!("name", &value)`) swap
//! to `insta::assert_json_snapshot!` in a one-line edit per test.

use serde_json::Value;
use std::path::PathBuf;

/// Return the directory that holds committed snapshot files. Lives
/// under the `jet` crate's `tests/` dir so the snapshots ship with
/// the test binary source.
fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("__snapshots__")
}

/// Core comparison. Prefer the `snapshot_eq!` macro at call sites
/// so test names line up with snapshot names automatically.
/// @spec .aw/tech-design/projects/jet/semantic/jet-tests-common.md#tests
pub fn snapshot_eq_impl(name: &str, actual: &Value) -> Result<(), String> {
    let dir = snapshot_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    let path = dir.join(format!("{name}.json"));

    let serialized = canonicalize(actual);
    let update = std::env::var("JET_SNAPSHOT_UPDATE").is_ok();

    match (path.exists(), update) {
        (false, _) | (true, true) => {
            // First run, or user asked to overwrite — write + pass.
            std::fs::write(&path, &serialized)
                .map_err(|e| format!("writing {}: {e}", path.display()))?;
            eprintln!(
                "[snapshot] wrote {} ({} bytes)",
                path.display(),
                serialized.len()
            );
            Ok(())
        }
        (true, false) => {
            let expected = std::fs::read_to_string(&path)
                .map_err(|e| format!("reading {}: {e}", path.display()))?;
            if expected == serialized {
                return Ok(());
            }
            Err(format!(
                "snapshot drift at {}\n\
                 → `JET_SNAPSHOT_UPDATE=1 cargo test ...` to accept the new shape\n\
                 \n\
                 expected ({} bytes):\n{}\n\n\
                 actual ({} bytes):\n{}\n",
                path.display(),
                expected.len(),
                expected,
                serialized.len(),
                serialized,
            ))
        }
    }
}

/// Canonical pretty-printed JSON — 2-space indent, sorted object
/// keys, trailing newline. Stable output so diffs are minimal and
/// human-readable.
fn canonicalize(v: &Value) -> String {
    let sorted = sort_keys(v);
    let mut out = serde_json::to_string_pretty(&sorted)
        .expect("pretty-printing a serde_json::Value can't fail");
    out.push('\n');
    out
}

fn sort_keys(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut pairs: Vec<(String, Value)> =
                map.iter().map(|(k, v)| (k.clone(), sort_keys(v))).collect();
            pairs.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = serde_json::Map::new();
            for (k, v) in pairs {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.iter().map(sort_keys).collect()),
        other => other.clone(),
    }
}

/// `snapshot_eq!("name", &value)` — compare a value to the
/// committed snapshot. Panics (test failure) on drift.
#[macro_export]
macro_rules! snapshot_eq {
    ($name:expr, $value:expr) => {{
        if let Err(msg) = $crate::common::snapshot::snapshot_eq_impl($name, $value) {
            panic!("{msg}");
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sort_keys_recurses_into_objects_and_arrays() {
        let input = json!({
            "z": 1,
            "a": [{"y": 2, "x": 1}, {"b": 3, "a": 4}],
            "m": {"d": 1, "c": 2},
        });
        let got = canonicalize(&input);
        let expected = "\
{
  \"a\": [
    {
      \"x\": 1,
      \"y\": 2
    },
    {
      \"a\": 4,
      \"b\": 3
    }
  ],
  \"m\": {
    \"c\": 2,
    \"d\": 1
  },
  \"z\": 1
}
";
        assert_eq!(
            got, expected,
            "canonicalize must sort keys deterministically"
        );
    }

    #[test]
    fn canonicalize_is_stable_across_map_insertion_order() {
        let a = json!({"x": 1, "y": 2, "z": 3});
        let b = json!({"z": 3, "y": 2, "x": 1});
        assert_eq!(canonicalize(&a), canonicalize(&b));
    }
}
// CODEGEN-END
