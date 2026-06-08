// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
// CODEGEN-BEGIN
//! `jet config lint` — structured diagnostic formatter for
//! `jet.config.toml` validation.
//!
//! @spec `.aw/tech-design/projects/jet/config/jet-config-validation.md`
//!     §"Slice 4 — `jet config lint` subcommand".
//! @issue #1233 — Slice 4 (this commit): the CLI verb that consumes
//!     the typed [`ConfigError`] / [`DeprecatedKeyWarning`] surface
//!     shipped in Slices 2 + 3 and prints a structured diagnostic
//!     block per issue.
//!
//! Exit code semantics (per the spec):
//!   - 0 — no errors and no warnings (or warnings present without
//!         `--strict-warn`)
//!   - 1 — warnings present and `--strict-warn` set
//!   - 2 — errors

use crate::wasm_build::config::{ConfigError, ConfigSpan, DeprecatedKeyWarning, WasmConfig};
use std::path::{Path, PathBuf};

/// Outcome of one lint run. Public so the CLI dispatcher and the
/// test harness share the same exit-code mapping; `to_exit_code`
/// is the sole conversion point.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug, PartialEq, Eq)]
pub enum LintOutcome {
    Ok,
    Warnings,
    Errors,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl LintOutcome {
    /// Map the outcome to the process exit code, taking
    /// `--strict-warn` into account. The `Errors` arm always
    /// wins (warnings + errors → 2).
    pub fn to_exit_code(&self, strict_warn: bool) -> i32 {
        match self {
            LintOutcome::Ok => 0,
            LintOutcome::Warnings => {
                if strict_warn {
                    1
                } else {
                    0
                }
            }
            LintOutcome::Errors => 2,
        }
    }
}

/// CLI entry point. Resolves `<project_root>/jet.config.toml`,
/// runs typed validation, and prints diagnostics in the requested
/// format. Returns the process exit code.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn run(project_root: &Path, format: &str, strict_warn: bool) -> i32 {
    let path = project_root.join("jet.config.toml");
    let report = lint_path(&path);
    print_report(&report, format);
    report.outcome().to_exit_code(strict_warn)
}

/// Internal: lint a specific config path (test entry too).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn lint_path(path: &Path) -> LintReport {
    let body = match std::fs::read_to_string(path) {
        Ok(b) => b,
        // GH #3456 — distinguish a truly missing config from a chmod
        // / EIO / mid-write read failure. Lumping both into NotFound
        // produced misleading "no jet.config.toml" diagnostics when the
        // file existed but was unreadable.
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            return LintReport {
                path: path.to_path_buf(),
                error: Some(ConfigError::NotFound(path.to_path_buf())),
                warnings: Vec::new(),
            };
        }
        Err(err) => {
            return LintReport {
                path: path.to_path_buf(),
                error: Some(ConfigError::Io {
                    path: path.to_path_buf(),
                    source: err,
                }),
                warnings: Vec::new(),
            };
        }
    };
    match WasmConfig::parse_str_with_warnings(&body, path) {
        Ok((_cfg, warnings)) => LintReport {
            path: path.to_path_buf(),
            error: None,
            warnings,
        },
        Err(err) => LintReport {
            path: path.to_path_buf(),
            error: Some(err),
            warnings: Vec::new(),
        },
    }
}

/// All diagnostics from one lint pass. The CLI converts this into
/// either human-formatted text or a JSON envelope.
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
#[derive(Debug)]
pub struct LintReport {
    pub path: PathBuf,
    pub error: Option<ConfigError>,
    pub warnings: Vec<DeprecatedKeyWarning>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
impl LintReport {
    pub fn outcome(&self) -> LintOutcome {
        if self.error.is_some() {
            LintOutcome::Errors
        } else if !self.warnings.is_empty() {
            LintOutcome::Warnings
        } else {
            LintOutcome::Ok
        }
    }
}

/// Print the report to stdout in the requested format. Unknown
/// formats fall back to `human` — the CLI's `value_parser` already
/// rejects garbage values upstream, so this is just defensive.
fn print_report(report: &LintReport, format: &str) {
    match format {
        "json" => println!("{}", format_json(report)),
        _ => print!("{}", format_human(report)),
    }
}

/// Human-readable format. One block per diagnostic; final line
/// summarizes (`✓ ok`, `N warnings`, `error: ...`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn format_human(report: &LintReport) -> String {
    let mut out = String::new();
    if let Some(err) = &report.error {
        out.push_str(&format!("error: {err}\n"));
    }
    for w in &report.warnings {
        out.push_str(&format!("warning: {w}\n"));
    }
    match report.outcome() {
        LintOutcome::Ok => {
            out.push_str(&format!("ok: {} is valid\n", report.path.display()));
        }
        LintOutcome::Warnings => {
            out.push_str(&format!(
                "{} warning(s) in {}\n",
                report.warnings.len(),
                report.path.display(),
            ));
        }
        LintOutcome::Errors => {
            // `error: ...` already printed above.
        }
    }
    out
}

/// JSON format — agent / CI consumer. Hand-rolled so we don't take
/// a `serde_json` dep here just for output (the existing dep tree
/// already has it via tauri-shell, but we keep this module
/// dep-free to stay drop-in).
/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-build.md#schema
pub fn format_json(report: &LintReport) -> String {
    let mut out = String::from("{");
    out.push_str(&format!(
        "\"path\":{}",
        json_string(&report.path.display().to_string())
    ));
    out.push_str(&format!(",\"ok\":{}", report.error.is_none()));
    out.push_str(",\"errors\":[");
    if let Some(err) = &report.error {
        out.push_str(&error_to_json(err));
    }
    out.push_str("],\"warnings\":[");
    for (i, w) in report.warnings.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&warning_to_json(w));
    }
    out.push_str("]}");
    out
}

fn error_to_json(err: &ConfigError) -> String {
    match err {
        ConfigError::NotFound(p) => format!(
            "{{\"kind\":\"not_found\",\"path\":{}}}",
            json_string(&p.display().to_string()),
        ),
        ConfigError::Io { path, source } => format!(
            "{{\"kind\":\"io\",\"path\":{},\"message\":{}}}",
            json_string(&path.display().to_string()),
            json_string(&source.to_string()),
        ),
        ConfigError::MissingWasmSection(p) => format!(
            "{{\"kind\":\"missing_wasm_section\",\"path\":{}}}",
            json_string(&p.display().to_string()),
        ),
        ConfigError::UnknownKey {
            path,
            key,
            suggestion,
            span,
        } => format!(
            "{{\"kind\":\"unknown_key\",\"path\":{},\"key\":{},\"suggestion\":{},{}}}",
            json_string(&path.display().to_string()),
            json_string(key),
            match suggestion {
                Some(s) => json_string(s),
                None => "null".to_string(),
            },
            span_json(span),
        ),
        ConfigError::InvalidValue {
            path,
            message,
            span,
        } => format!(
            "{{\"kind\":\"invalid_value\",\"path\":{},\"message\":{},{}}}",
            json_string(&path.display().to_string()),
            json_string(message),
            span_json(span),
        ),
    }
}

fn warning_to_json(w: &DeprecatedKeyWarning) -> String {
    format!(
        "{{\"kind\":\"deprecated_key\",\"path\":{},\"key\":{},\"replacement\":{},\"removal_version\":{},{}}}",
        json_string(&w.path.display().to_string()),
        json_string(&w.key),
        json_string(&w.replacement),
        json_string(&w.removal_version),
        span_json(&w.span),
    )
}

fn span_json(s: &ConfigSpan) -> String {
    format!("\"line\":{},\"column\":{}", s.line, s.column)
}

/// Minimal JSON-string encoder: backslash-escapes the four
/// characters that JSON requires (`"`, `\`, control chars, and
/// the BMP escapes). Anything else passes through untouched —
/// our paths and messages are ASCII / UTF-8 already.
fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &std::path::Path, body: &str) -> PathBuf {
        let p = dir.join("jet.config.toml");
        std::fs::write(&p, body).unwrap();
        p
    }

    #[test]
    fn outcome_ok_for_valid_config_yields_exit_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write(
            tmp.path(),
            "[wasm]\nentry = \"src/index.tsx\"\nroot_component = \"App\"\n",
        );
        let report = lint_path(&p);
        assert_eq!(report.outcome(), LintOutcome::Ok);
        assert_eq!(report.outcome().to_exit_code(false), 0);
        assert_eq!(report.outcome().to_exit_code(true), 0);
    }

    #[test]
    fn outcome_errors_for_unknown_key_yields_exit_two() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write(
            tmp.path(),
            "[wasm]\nentry = \"x\"\nroot_component = \"X\"\nroot_propz = []\n",
        );
        let report = lint_path(&p);
        assert_eq!(report.outcome(), LintOutcome::Errors);
        assert_eq!(report.outcome().to_exit_code(false), 2);
        assert_eq!(report.outcome().to_exit_code(true), 2);
    }

    #[test]
    fn missing_file_classifies_as_not_found_error() {
        let tmp = tempfile::tempdir().unwrap();
        let report = lint_path(&tmp.path().join("jet.config.toml"));
        assert_eq!(report.outcome(), LintOutcome::Errors);
        match report.error.as_ref().unwrap() {
            ConfigError::NotFound(_) => {}
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn dev_port_anchor_emits_unknown_key_dev_error() {
        // Post-#1403: `[dev]` is an accepted top-level section in the
        // WASM loader's `ConfigFile`, so the `dev-port` deprecation
        // remap now lands cleanly. The lint report no longer surfaces
        // an `UnknownKey on dev` error — it surfaces the deprecation
        // warning alone, with no error.
        let tmp = tempfile::tempdir().unwrap();
        let p = write(
            tmp.path(),
            "dev-port = 5173\n[wasm]\nentry = \"x\"\nroot_component = \"App\"\n",
        );
        let report = lint_path(&p);
        assert!(
            report.error.is_none(),
            "expected no error, got {:?}",
            report.error
        );
        assert!(
            report
                .warnings
                .iter()
                .any(|w| w.key == "dev-port" && w.replacement == "dev.port"),
            "expected deprecation warning for dev-port → dev.port, got {:?}",
            report.warnings
        );
    }

    #[test]
    fn human_format_renders_ok_summary() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: None,
            warnings: Vec::new(),
        };
        let out = format_human(&report);
        assert!(out.contains("ok:"));
        assert!(out.contains("/x/jet.config.toml"));
    }

    #[test]
    fn human_format_renders_error_block() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: Some(ConfigError::UnknownKey {
                path: PathBuf::from("/x/jet.config.toml"),
                key: "root_propz".into(),
                suggestion: Some("root_props".into()),
                span: ConfigSpan { line: 4, column: 1 },
            }),
            warnings: Vec::new(),
        };
        let out = format_human(&report);
        assert!(out.starts_with("error:"));
        assert!(out.contains("\"root_propz\""));
        assert!(out.contains("did you mean"));
    }

    #[test]
    fn human_format_renders_warning_block_and_count() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: None,
            warnings: vec![DeprecatedKeyWarning {
                path: PathBuf::from("/x/jet.config.toml"),
                key: "dev-port".into(),
                replacement: "dev.port".into(),
                removal_version: "0.4.0".into(),
                span: ConfigSpan { line: 2, column: 1 },
            }],
        };
        let out = format_human(&report);
        assert!(out.contains("warning:"));
        assert!(out.contains("\"dev-port\""));
        assert!(out.contains("1 warning(s)"));
    }

    #[test]
    fn json_format_envelope_for_ok_report() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: None,
            warnings: Vec::new(),
        };
        let out = format_json(&report);
        assert!(out.contains("\"ok\":true"));
        assert!(out.contains("\"errors\":[]"));
        assert!(out.contains("\"warnings\":[]"));
    }

    #[test]
    fn json_format_envelope_for_unknown_key_error() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: Some(ConfigError::UnknownKey {
                path: PathBuf::from("/x/jet.config.toml"),
                key: "root_propz".into(),
                suggestion: Some("root_props".into()),
                span: ConfigSpan { line: 4, column: 1 },
            }),
            warnings: Vec::new(),
        };
        let out = format_json(&report);
        assert!(out.contains("\"ok\":false"));
        assert!(out.contains("\"kind\":\"unknown_key\""));
        assert!(out.contains("\"key\":\"root_propz\""));
        assert!(out.contains("\"suggestion\":\"root_props\""));
        assert!(out.contains("\"line\":4"));
    }

    #[test]
    fn json_format_envelope_for_deprecated_warning() {
        let report = LintReport {
            path: PathBuf::from("/x/jet.config.toml"),
            error: None,
            warnings: vec![DeprecatedKeyWarning {
                path: PathBuf::from("/x/jet.config.toml"),
                key: "dev-port".into(),
                replacement: "dev.port".into(),
                removal_version: "0.4.0".into(),
                span: ConfigSpan { line: 2, column: 1 },
            }],
        };
        let out = format_json(&report);
        assert!(out.contains("\"kind\":\"deprecated_key\""));
        assert!(out.contains("\"replacement\":\"dev.port\""));
        assert!(out.contains("\"removal_version\":\"0.4.0\""));
    }

    #[test]
    fn warnings_outcome_promotes_to_exit_one_under_strict_warn() {
        let outcome = LintOutcome::Warnings;
        assert_eq!(outcome.to_exit_code(false), 0);
        assert_eq!(outcome.to_exit_code(true), 1);
    }

    #[test]
    fn json_string_escapes_quotes_and_backslashes() {
        let s = json_string("a\"b\\c");
        assert_eq!(s, "\"a\\\"b\\\\c\"");
    }

    #[test]
    fn json_string_escapes_control_chars() {
        let s = json_string("x\nz\tw");
        assert_eq!(s, "\"x\\nz\\tw\"");
    }

    // ── GH #3456 lint_path IO error classification ───────────────────

    /// GH #3456 — happy path: valid config produces no error.
    #[test]
    fn gh3456_lint_path_valid_config_no_error() {
        let tmp = tempfile::tempdir().unwrap();
        let p = write(
            tmp.path(),
            "[wasm]\nentry = \"x\"\nroot_component = \"App\"\n",
        );
        let report = lint_path(&p);
        assert!(
            report.error.is_none(),
            "valid config must lint without error: {:?}",
            report.error
        );
    }

    /// GH #3456 — missing file still classifies as NotFound (preserved).
    #[test]
    fn gh3456_lint_path_missing_file_classifies_as_not_found() {
        let tmp = tempfile::tempdir().unwrap();
        let report = lint_path(&tmp.path().join("jet.config.toml"));
        match report.error.as_ref().unwrap() {
            ConfigError::NotFound(_) => {}
            other => panic!("expected NotFound for missing file, got {other:?}"),
        }
    }

    /// GH #3456 — chmod 000 file: must classify as Io, not NotFound.
    #[cfg(unix)]
    #[test]
    fn gh3456_lint_path_unreadable_file_classifies_as_io_not_notfound() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let p = write(
            tmp.path(),
            "[wasm]\nentry = \"x\"\nroot_component = \"App\"\n",
        );
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root may still read 000-mode files — skip if so.
        if std::fs::read_to_string(&p).is_ok() {
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let report = lint_path(&p);

        // Restore perms for tempdir cleanup.
        let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644));

        match report.error.as_ref().unwrap() {
            ConfigError::Io { path, .. } => {
                assert_eq!(path, &p);
            }
            ConfigError::NotFound(_) => {
                panic!("unreadable file must NOT be classified as NotFound")
            }
            other => panic!("expected Io variant, got {other:?}"),
        }
    }
}
// CODEGEN-END
