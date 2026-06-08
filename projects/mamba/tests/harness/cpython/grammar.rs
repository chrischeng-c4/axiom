//! CPython 3.12 compliance test harness for the Mamba parser.
//!
//! Discovers `.py` fixtures under `tests/cpython/_regression/core/grammar/` and
//! runs each through the Mamba parser. (The dimension-first migration moved the
//! no-record grammar syntax-probe regression fixtures under `_regression/`.)
//! Tests listed in `cpython_known_failures.toml` are
//! treated as *expected failures* (xfail): they are skipped without failing
//! CI, and a warning is printed when they unexpectedly pass.
//!
//! Fixture conventions:
//!   `# RUN: parse`   -- run through parser only (default for all cpython tests)
//!   `# XFAIL`        -- mark this individual file as expected to fail
//!   `# REASON: ...`  -- human-readable reason for xfail
//!
//! #2546: this harness is **parser-only**. Every fixture is fed to the
//! Mamba parser; nothing is ever lowered, type-checked, or executed.
//! A passing fixture here is NOT evidence that mamba executes the
//! corresponding CPython behavior — only that the parser accepts the
//! syntax. The runtime half of the CPython compatibility surface
//! lives in `tests/cpython_lib_test_runner.rs` and emits its own
//! `harness_kind = "runtime"` summary JSON (schema_version 2). Each
//! fixture in this file emits a `[cpython_compat:parser-only]`
//! banner on stderr so downstream tools cannot conflate parser
//! acceptance counts with runtime assertion counts.

use datatest_stable::harness;
use mamba::parser;
use mamba::source::span::FileId;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

// -- xfail manifest ----------------------------------------------------------

/// Map from fixture stem (e.g. "test_fstring/nested_fstrings") to reason string.
static XFAIL_MAP: OnceLock<HashMap<String, String>> = OnceLock::new();

fn load_xfail_map() -> HashMap<String, String> {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cpython_known_failures.toml");
    let Ok(contents) = std::fs::read_to_string(&manifest_path) else {
        eprintln!(
            "Warning: cpython_known_failures.toml not found at {}",
            manifest_path.display()
        );
        return HashMap::new();
    };

    let doc: toml::Value = match contents.parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Warning: failed to parse cpython_known_failures.toml: {e}");
            return HashMap::new();
        }
    };

    let mut map = HashMap::new();
    if let Some(failures) = doc.get("failures").and_then(|v| v.as_table()) {
        for (key, entry) in failures {
            let reason = entry
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("no reason given")
                .to_string();
            map.insert(key.clone(), reason);
        }
    }
    map
}

fn xfail_map() -> &'static HashMap<String, String> {
    XFAIL_MAP.get_or_init(load_xfail_map)
}

// -- Fixture key ------------------------------------------------------------

/// Derive the xfail manifest key from the fixture path.
/// E.g. `tests/cpython/core/grammar/test_fstring/nested_fstrings.py`
///   -> `test_fstring/nested_fstrings`
fn fixture_key(path: &Path) -> String {
    let mut parts = path.components().peekable();
    while let Some(c) = parts.next() {
        if c.as_os_str() == "cpython" {
            let remaining: Vec<_> = parts
                .map(|p| p.as_os_str().to_string_lossy().into_owned())
                .collect();
            let joined = remaining.join("/");
            return joined.trim_end_matches(".py").to_string();
        }
    }
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

// -- Inline directive parsing -----------------------------------------------

struct Directives {
    inline_xfail: bool,
    inline_reason: Option<String>,
}

fn parse_directives(src: &str) -> Directives {
    let mut inline_xfail = false;
    let mut inline_reason = None;

    for line in src.lines() {
        let t = line.trim();
        if t == "# XFAIL" || t == "# XFAIL:" {
            inline_xfail = true;
        } else if let Some(rest) = t.strip_prefix("# REASON:") {
            inline_reason = Some(rest.trim().to_string());
        }
    }

    Directives {
        inline_xfail,
        inline_reason,
    }
}

// -- Runner -----------------------------------------------------------------

fn run_cpython_fixture(path: &Path) -> datatest_stable::Result<()> {
    let src = std::fs::read_to_string(path)?;
    let key = fixture_key(path);

    // #2546: every fixture stamps `[cpython_compat:parser-only]` so a
    // CI log scraper that ingests both harnesses can bucket counts
    // unambiguously. Acceptance here means "parser accepted syntax",
    // not "mamba executed CPython behavior".
    eprintln!("[cpython_compat:parser-only] {key}");

    // Determine xfail status (manifest takes priority, then inline directive).
    let xfail_reason: Option<String> = xfail_map().get(&key).cloned().or_else(|| {
        let d = parse_directives(&src);
        if d.inline_xfail {
            Some(
                d.inline_reason
                    .unwrap_or_else(|| "inline XFAIL".to_string()),
            )
        } else {
            None
        }
    });

    let parse_result = parser::parse(&src, FileId(0));

    match (parse_result, xfail_reason) {
        (Ok(_), None) => Ok(()),
        (Err(_), Some(reason)) => {
            eprintln!("  [xfail] {key}: {reason}");
            Ok(())
        }
        (Ok(_), Some(reason)) => {
            eprintln!(
                "  [xpass] {key} passed unexpectedly (xfail reason: {reason}). \
                 Consider removing from cpython_known_failures.toml."
            );
            Ok(())
        }
        (Err(e), None) => Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{}: parse failed: {e}", path.display()),
        ))),
    }
}

harness!(
    run_cpython_fixture,
    "tests/cpython/_regression/core/grammar",
    r".*\.py$"
);
