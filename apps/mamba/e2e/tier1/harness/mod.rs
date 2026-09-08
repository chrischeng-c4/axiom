//! Shared discovery, PEP 723 header parsing, spawning, and judging for the
//! T1 conformance rollup (issue #4242) and the later per-area cases that
//! restrict the same discovery to one area prefix
//! (`tier1_cross_thread_identity`, `tier1_readiness_evidence`,
//! `tier1_free_threaded`). This module is a fixture under the engine's own
//! rule (`apps/aw/src/aw/scripts/e2e.py:234`: only a `.rs` file directly
//! under `e2e/` is a case), included by each case with
//! `#[path = "tier1/harness/mod.rs"] mod harness;` rather than declared as
//! its own `[[test]]` stanza.
//!
//! Every fixture is a PEP 723 script: a `# /// script` … `# ///` header
//! block whose body is TOML, carrying a `[tool.mamba]` table with a `kind`
//! field that selects the judge:
//!
//! - `"oracle"` — `mamba run --compile <fixture>` and `python3.12 <fixture>`
//!   must both exit 0 and print byte-identical stdout.
//! - `"type-strict"` — mamba must either reject the fixture at compile time
//!   with a `TypeError`/`type error` message on stdout or stderr, or exit 0
//!   having printed a `typeerror:`-prefixed stdout line.
//! - `"self-verdict"` — mamba alone must exit 0 and print a
//!   `concurrency: PASS` stdout line; there is no CPython oracle for a
//!   concurrency fixture because CPython's own thread interleaving is not
//!   the contract being judged.
//!
//! A fixture whose header is missing, malformed, or names an unknown `kind`
//! is a judge failure, never a skip: the harness carries no expected-failure
//! or allowlist mechanism of any kind (Frozen decisions, issue #4242).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// One discovered `*.py` fixture under an area directory
/// (`core`, `type`, or `concurrency`) of `e2e/tier1/`.
pub struct Fixture {
    pub path: PathBuf,
    /// The first path component under the discovery root — the top-level
    /// area directory this fixture lives in.
    pub area: String,
}

/// The three judges a fixture's `[tool.mamba] kind` field selects.
enum Kind {
    Oracle,
    TypeStrict,
    SelfVerdict,
}

/// Resolves the built `mamba` binary under test — the `CARGO_BIN_EXE_mamba`
/// Cargo hands integration test binaries, falling back to the workspace
/// target directory. Never `PATH`, matching
/// `apps/mamba/tests/harness/cpython/harness_common.rs:62`.
pub fn mamba_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_mamba")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mamba")
        })
}

/// The CPython oracle, resolved from `PATH` (Frozen decisions, issue #4242).
pub fn python3_bin() -> PathBuf {
    PathBuf::from("python3.12")
}

/// Asserts the resolved `python3_bin()` actually reports a 3.12 interpreter,
/// before any `oracle` fixture is judged against it — a wrong or missing
/// interpreter would silently turn every oracle comparison into noise
/// instead of a defect signal.
pub fn check_python312() -> Result<(), String> {
    let bin = python3_bin();
    let output = Command::new(&bin)
        .arg("--version")
        .output()
        .map_err(|e| format!("failed to spawn {bin:?} --version: {e}"))?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if output.status.success() && combined.contains("Python 3.12") {
        Ok(())
    } else {
        Err(format!(
            "{bin:?} --version did not report a 3.12 interpreter (exit {:?}): {combined:?}",
            output.status.code()
        ))
    }
}

/// Recursively walks `root` for `*.py` files, tagging each with the first
/// path component under `root` as its area. Returns fixtures sorted by path
/// so a run's failure report is stable across invocations.
pub fn discover_fixtures(root: &Path) -> Vec<Fixture> {
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<Fixture>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            walk(root, &path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
            let area = path
                .strip_prefix(root)
                .ok()
                .and_then(|rel| rel.components().next())
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push(Fixture { path, area });
        }
    }
}

/// Reads `path`, parses its PEP 723 `[tool.mamba] kind` field, and runs the
/// judge that field selects. Any failure — an unreadable file, a malformed
/// or missing header, an unknown `kind`, or the judge itself rejecting the
/// fixture — is `Err(reason)`; there is no skip outcome.
pub fn judge_fixture(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| format!("cannot read fixture: {e}"))?;
    match parse_kind(&text)? {
        Kind::Oracle => judge_oracle(path),
        Kind::TypeStrict => judge_type_strict(path),
        Kind::SelfVerdict => judge_self_verdict(path),
    }
}

/// Extracts the TOML body of a PEP 723 `# /// script` … `# ///` header:
/// each body line is either the bare marker `#` (a blank TOML line) or
/// `# ` followed by the line's real content; anything else inside the block
/// is a malformed header.
fn extract_pep723_toml(text: &str) -> Result<String, String> {
    let mut body = String::new();
    let mut in_block = false;
    for line in text.lines() {
        if !in_block {
            if line == "# /// script" {
                in_block = true;
            }
            continue;
        }
        if line == "# ///" {
            return Ok(body);
        }
        if line == "#" {
            body.push('\n');
        } else if let Some(rest) = line.strip_prefix("# ") {
            body.push_str(rest);
            body.push('\n');
        } else {
            return Err(format!("malformed PEP 723 header line: {line:?}"));
        }
    }
    Err("no `# /// script` … `# ///` header block found".to_string())
}

fn parse_kind(text: &str) -> Result<Kind, String> {
    let body = extract_pep723_toml(text)?;
    let value: toml::Value =
        toml::from_str(&body).map_err(|e| format!("invalid PEP 723 TOML header: {e}"))?;
    let kind_str = value
        .get("tool")
        .and_then(|t| t.get("mamba"))
        .and_then(|m| m.get("kind"))
        .and_then(|k| k.as_str())
        .ok_or_else(|| "no [tool.mamba] kind field in header".to_string())?;
    match kind_str {
        "oracle" => Ok(Kind::Oracle),
        "type-strict" => Ok(Kind::TypeStrict),
        "self-verdict" => Ok(Kind::SelfVerdict),
        other => Err(format!("unknown [tool.mamba] kind {other:?}")),
    }
}

fn judge_oracle(path: &Path) -> Result<(), String> {
    let mamba_out = Command::new(mamba_bin())
        .args(["run", "--compile"])
        .arg(path)
        .output()
        .map_err(|e| format!("failed to spawn mamba: {e}"))?;
    if !mamba_out.status.success() {
        return Err(format!(
            "mamba exited {:?}: stdout={:?} stderr={:?}",
            mamba_out.status.code(),
            String::from_utf8_lossy(&mamba_out.stdout),
            String::from_utf8_lossy(&mamba_out.stderr)
        ));
    }
    let python_out = Command::new(python3_bin())
        .arg(path)
        .output()
        .map_err(|e| format!("failed to spawn python3.12: {e}"))?;
    if !python_out.status.success() {
        return Err(format!(
            "oracle python3.12 exited {:?}: stdout={:?} stderr={:?}",
            python_out.status.code(),
            String::from_utf8_lossy(&python_out.stdout),
            String::from_utf8_lossy(&python_out.stderr)
        ));
    }
    let mamba_stdout = String::from_utf8_lossy(&mamba_out.stdout);
    let python_stdout = String::from_utf8_lossy(&python_out.stdout);
    if mamba_stdout == python_stdout {
        Ok(())
    } else {
        Err(format!(
            "stdout mismatch: mamba printed {mamba_stdout:?}, python3.12 printed {python_stdout:?}"
        ))
    }
}

fn judge_type_strict(path: &Path) -> Result<(), String> {
    let out = Command::new(mamba_bin())
        .args(["run", "--compile"])
        .arg(path)
        .output()
        .map_err(|e| format!("failed to spawn mamba: {e}"))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        let names_type_error = stdout.contains("TypeError")
            || stdout.contains("type error")
            || stderr.contains("TypeError")
            || stderr.contains("type error");
        return if names_type_error {
            Ok(())
        } else {
            Err(format!(
                "mamba exited {:?} without a TypeError/type error message: stdout={stdout:?} stderr={stderr:?}",
                out.status.code()
            ))
        };
    }
    if stdout.lines().any(|l| l.starts_with("typeerror:")) {
        Ok(())
    } else {
        Err(format!(
            "mamba exited 0 without a `typeerror:` stdout line: stdout={stdout:?} stderr={stderr:?}"
        ))
    }
}

fn judge_self_verdict(path: &Path) -> Result<(), String> {
    let out = Command::new(mamba_bin())
        .args(["run", "--compile"])
        .arg(path)
        .output()
        .map_err(|e| format!("failed to spawn mamba: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "mamba exited {:?}: stdout={:?} stderr={:?}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    if stdout.lines().any(|l| l.trim() == "concurrency: PASS") {
        Ok(())
    } else {
        Err(format!(
            "mamba exited 0 without a `concurrency: PASS` stdout line: stdout={stdout:?}"
        ))
    }
}
