// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
// CODEGEN-BEGIN
//! `jet config schema` — JSON Schema export for `jet.toml`.
//!
//! @spec `.aw/tech-design/projects/jet/config/jet-config-validation.md`
//!     §"Slice 5 — `schemas/jet.schema.json` export".
//! @issue #1233 — Slice 5 (this commit). Derives the schema from
//!     [`crate::task_runner::config::JetConfig`]'s `schemars::JsonSchema`
//!     so the on-disk artifact stays in lockstep with the full Rust source
//!     of truth (R1). CI gate: `jet config schema --check` exits
//!     non-zero on drift.
//!
//! Subcommand modes:
//!
//! - default (no flag) — print the generated schema to stdout.
//! - `--write` — write to `<workspace_root>/schemas/jet.schema.json`.
//! - `--check` — read the on-disk artifact and exit non-zero if it
//!   differs from a fresh generation. Exit codes: 0 = up-to-date,
//!   1 = drift, 2 = on-disk file missing or malformed.
//!
//! The schema is derived from [`crate::task_runner::config::JetConfig`], the
//! full `jet.toml` shape used by `jet dev`, `jet build`, task runner config,
//! and library mode. WASM-specific loading still consumes `[wasm]` through
//! `wasm_build::config`, but the public schema must describe the whole file.

use crate::task_runner::config::JetConfig;
use schemars::schema::RootSchema;
use schemars::schema_for;
use std::path::Path;

/// Outcome of one schema-export run. Public so the CLI dispatcher and
/// tests share the same exit-code mapping; [`SchemaOutcome::to_exit_code`]
/// is the sole conversion point.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, PartialEq, Eq)]
pub enum SchemaOutcome {
    /// Default mode (printed to stdout) or `--write` succeeded.
    Ok,
    /// `--check` and the on-disk artifact matches.
    Match,
    /// `--check` and the on-disk artifact differs from a fresh generation.
    Drift,
    /// `--check` and the on-disk artifact is missing / unreadable / not
    /// valid JSON.
    Missing,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl SchemaOutcome {
    pub fn to_exit_code(&self) -> i32 {
        match self {
            SchemaOutcome::Ok | SchemaOutcome::Match => 0,
            SchemaOutcome::Drift => 1,
            SchemaOutcome::Missing => 2,
        }
    }
}

/// Top-of-file artifact path, relative to the workspace root.
pub const SCHEMA_REL_PATH: &str = "schemas/jet.schema.json";

/// Build the JSON Schema for the full `jet.toml` file.
///
/// Derives the public schema from [`JetConfig`] so the artifact validates the
/// whole on-disk file shape. Returns the [`RootSchema`] so callers can either
/// serialize it themselves or hand it to [`render`].
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
/// @spec .aw/tech-design/projects/jet/config/jet-build-lib-lib-config-section-css-merge-raw-copy-referenced-i.md#logic
pub fn build_schema() -> RootSchema {
    let mut schema = schema_for!(JetConfig);
    schema.schema.metadata().title = Some("jet.toml".into());
    schema.schema.metadata().description = Some(
        "Schema for the `jet.toml` file. Generated from the Rust \
         source of truth in `projects/jet/src/task_runner/config.rs`. Run \
         `jet config schema --write` to regenerate."
            .into(),
    );
    schema
}

/// Render the schema as a stable, pretty-printed JSON string with a
/// trailing newline. Used by both `--write` and the in-memory
/// comparison in `--check` so byte-equality is meaningful.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn render(schema: &RootSchema) -> String {
    let mut out =
        serde_json::to_string_pretty(schema).expect("RootSchema serialization is infallible");
    out.push('\n');
    out
}

/// CLI entry point. `mode` is one of `"print"` / `"write"` / `"check"`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn run(workspace_root: &Path, mode: &str) -> i32 {
    let schema = build_schema();
    let rendered = render(&schema);
    let outcome = match mode {
        "print" => {
            print!("{rendered}");
            SchemaOutcome::Ok
        }
        "write" => {
            let target = workspace_root.join(SCHEMA_REL_PATH);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&target, &rendered) {
                Ok(()) => {
                    eprintln!("[jet config schema] wrote {}", target.display());
                    SchemaOutcome::Ok
                }
                Err(err) => {
                    eprintln!(
                        "[jet config schema] failed to write {}: {err}",
                        target.display()
                    );
                    return 2;
                }
            }
        }
        "check" => check_against_disk(workspace_root, &rendered),
        other => {
            eprintln!("[jet config schema] unknown mode {other:?}");
            return 2;
        }
    };
    outcome.to_exit_code()
}

/// Read the on-disk artifact and return the matching [`SchemaOutcome`].
/// Public for tests that want to bypass the CLI dispatch.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn check_against_disk(workspace_root: &Path, fresh: &str) -> SchemaOutcome {
    let target = workspace_root.join(SCHEMA_REL_PATH);
    let on_disk = match std::fs::read_to_string(&target) {
        Ok(b) => b,
        // GH #3474 — distinguish a truly absent artifact (the
        // 'run --write' remediation only helps then) from a chmod /
        // EIO read failure where --write will also fail. Both still
        // map to SchemaOutcome::Missing so the exit-code contract
        // does not change.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "[jet config schema --check] {} missing — run `jet config schema --write`",
                target.display()
            );
            return SchemaOutcome::Missing;
        }
        Err(err) => {
            eprintln!(
                "[jet config schema --check] failed to read {}: {} ({:?}). \
                 Check file permissions — `jet config schema --write` will \
                 likely fail with the same error.",
                target.display(),
                err,
                err.kind()
            );
            return SchemaOutcome::Missing;
        }
    };
    if on_disk == fresh {
        SchemaOutcome::Match
    } else {
        eprintln!(
            "[jet config schema --check] DRIFT — {} differs from a fresh generation. \
             Run `jet config schema --write` to regenerate.",
            target.display()
        );
        SchemaOutcome::Drift
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn schema_outcome_exit_codes() {
        assert_eq!(SchemaOutcome::Ok.to_exit_code(), 0);
        assert_eq!(SchemaOutcome::Match.to_exit_code(), 0);
        assert_eq!(SchemaOutcome::Drift.to_exit_code(), 1);
        assert_eq!(SchemaOutcome::Missing.to_exit_code(), 2);
    }

    #[test]
    fn build_schema_exposes_full_jet_config_with_optional_wasm() {
        let schema = build_schema();
        let json = serde_json::to_value(&schema).unwrap();

        assert_eq!(json["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(json["type"], "object");
        assert!(
            json.get("required")
                .and_then(|v| v.as_array())
                .map_or(true, |required| !required.iter().any(|v| v == "wasm")),
            "`wasm` must be optional in the full jet.toml schema, got {}",
            json.get("required").unwrap_or(&serde_json::Value::Null)
        );
        assert_eq!(json["additionalProperties"], false);

        let props = json["properties"].as_object().unwrap();
        for expected in [
            "pipeline", "dev", "alias", "build", "resolve", "test", "wasm", "lib", "codegen",
        ] {
            assert!(
                props.contains_key(expected),
                "expected top-level property `{expected}` in schema, got {props:?}",
            );
        }
        let schema_text = serde_json::to_string(&json).unwrap();
        assert!(
            schema_text.contains("css_merge"),
            "`[lib].css_merge` must be present in schema"
        );
        assert!(
            schema_text.contains("raw_copy"),
            "`[lib].raw_copy` must be present in schema"
        );
    }

    #[test]
    fn render_is_deterministic_and_newline_terminated() {
        let a = render(&build_schema());
        let b = render(&build_schema());
        assert_eq!(a, b, "render must be deterministic");
        assert!(a.ends_with('\n'), "render must end with a trailing newline");
        assert!(a.starts_with("{\n"), "render must be pretty-printed JSON");
    }

    #[test]
    fn check_against_disk_match_when_file_equals_fresh() {
        let dir = tempdir().unwrap();
        let target = dir.path().join(SCHEMA_REL_PATH);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        let fresh = render(&build_schema());
        std::fs::write(&target, &fresh).unwrap();

        assert_eq!(check_against_disk(dir.path(), &fresh), SchemaOutcome::Match);
    }

    #[test]
    fn check_against_disk_drift_when_file_differs() {
        let dir = tempdir().unwrap();
        let target = dir.path().join(SCHEMA_REL_PATH);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "{}\n").unwrap();
        let fresh = render(&build_schema());

        assert_eq!(check_against_disk(dir.path(), &fresh), SchemaOutcome::Drift);
    }

    #[test]
    fn check_against_disk_missing_when_file_absent() {
        let dir = tempdir().unwrap();
        let fresh = render(&build_schema());
        assert_eq!(
            check_against_disk(dir.path(), &fresh),
            SchemaOutcome::Missing
        );
    }

    #[test]
    fn run_print_mode_returns_zero_and_does_not_touch_disk() {
        let dir = tempdir().unwrap();
        let exit = run(dir.path(), "print");
        assert_eq!(exit, 0);
        // No file should have been created.
        assert!(!dir.path().join(SCHEMA_REL_PATH).exists());
    }

    #[test]
    fn run_write_mode_creates_artifact_round_trip_to_disk() {
        let dir = tempdir().unwrap();
        let exit = run(dir.path(), "write");
        assert_eq!(exit, 0);
        let on_disk = std::fs::read_to_string(dir.path().join(SCHEMA_REL_PATH)).unwrap();
        let fresh = render(&build_schema());
        assert_eq!(on_disk, fresh);

        // A subsequent `--check` against the just-written file matches.
        let exit_check = run(dir.path(), "check");
        assert_eq!(exit_check, 0);
    }

    #[test]
    fn run_check_mode_drift_returns_exit_one() {
        let dir = tempdir().unwrap();
        let target = dir.path().join(SCHEMA_REL_PATH);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "{}\n").unwrap();

        let exit = run(dir.path(), "check");
        assert_eq!(exit, 1);
    }

    #[test]
    fn run_check_mode_missing_returns_exit_two() {
        let dir = tempdir().unwrap();
        let exit = run(dir.path(), "check");
        assert_eq!(exit, 2);
    }

    #[test]
    fn run_unknown_mode_returns_exit_two() {
        let dir = tempdir().unwrap();
        let exit = run(dir.path(), "garbage");
        assert_eq!(exit, 2);
    }

    // ── GH #3474 check_against_disk IO error classification ─────────

    /// GH #3474 — happy path: an on-disk artifact byte-equal to `fresh`
    /// classifies as Match.
    #[test]
    fn gh3474_check_against_disk_matching_returns_match() {
        let dir = tempdir().unwrap();
        let target = dir.path().join(SCHEMA_REL_PATH);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "same\n").unwrap();
        assert_eq!(
            check_against_disk(dir.path(), "same\n"),
            SchemaOutcome::Match
        );
    }

    /// GH #3474 — missing artifact: Missing outcome preserved.
    #[test]
    fn gh3474_check_against_disk_missing_returns_missing() {
        let dir = tempdir().unwrap();
        assert_eq!(
            check_against_disk(dir.path(), "anything"),
            SchemaOutcome::Missing
        );
    }

    /// GH #3474 — chmod 000 artifact: Missing outcome (exit-code contract
    /// preserved). Stderr message differs (verified by code review of the
    /// split arm).
    #[cfg(unix)]
    #[test]
    fn gh3474_check_against_disk_unreadable_returns_missing() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let target = dir.path().join(SCHEMA_REL_PATH);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, "real content\n").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root may still read 000-mode files — skip if so.
        if std::fs::read_to_string(&target).is_ok() {
            let _ = std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let outcome = check_against_disk(dir.path(), "fresh content\n");

        // Restore perms for tempdir cleanup.
        let _ = std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644));

        assert_eq!(
            outcome,
            SchemaOutcome::Missing,
            "unreadable artifact must still map to Missing for exit-code stability"
        );
    }
}
// CODEGEN-END
