//! Current-state schema gate for issue #1104 tier1 stdlib scope correction.
//!
//! This is intentionally a bounded metadata gate, not a new replacement
//! campaign. It locks the repo facts that were re-checked on 2026-07-05:
//!
//! - `urllib.parse missing` is a stale claim and must not be reopened without
//!   first changing the runtime surface in `http_mod.rs`.
//! - Tier1 evidence here is file/registration/API/perf-pin state only.

use std::fs;
use std::path::PathBuf;

fn mamba_root() -> PathBuf {
    crate::common::project_root()
}

fn read_rel(rel: &str) -> String {
    let path = mamba_root().join(rel);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn assert_exists(rel: &str) {
    let path = mamba_root().join(rel);
    assert!(path.is_file(), "expected file to exist: {}", path.display());
}

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} missing required marker `{needle}`"
    );
}

fn perf_pin_names() -> Vec<String> {
    let dir = mamba_root().join("tests/harness/cpython/config/perf/pins");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            path.file_name()?.to_str().map(|s| s.to_string())
        })
        .collect();
    names.sort();
    names
}

fn pins_for_module<'a>(pins: &'a [String], module_stem: &str) -> Vec<&'a str> {
    let prefix = format!("{module_stem}_");
    pins.iter()
        .filter_map(|name| {
            let rest = name.strip_prefix(&prefix)?;
            rest.chars()
                .next()
                .filter(|c| c.is_ascii_digit())
                .map(|_| name.as_str())
        })
        .collect()
}

#[test]
fn issue_1104_tier1_runtime_files_and_registrations_exist() {
    let posixpath_mod = "src/runtime/stdlib/posixpath_mod.rs";
    let os_mod = "src/runtime/stdlib/os_mod.rs";
    let functools_mod = "src/runtime/stdlib/functools_mod.rs";
    let http_mod = "src/runtime/stdlib/http_mod.rs";
    let json_mod = "src/runtime/stdlib/json_mod.rs";
    let typing_mod = "src/runtime/stdlib/typing_mod.rs";
    let enum_mod = "src/runtime/stdlib/enum_mod.rs";

    for rel in [
        posixpath_mod,
        os_mod,
        functools_mod,
        http_mod,
        json_mod,
        typing_mod,
        enum_mod,
    ] {
        assert_exists(rel);
    }

    let posixpath_text = read_rel(posixpath_mod);
    assert_contains(
        &posixpath_text,
        "register_module(\"posixpath\", build_attrs())",
        "posixpath native registration",
    );

    let os_text = read_rel(os_mod);
    assert_contains(
        &os_text,
        "register_module(\"os.path\", path_attrs)",
        "os.path native registration",
    );
    assert_contains(
        &os_text,
        "mods_ref.get(\"os.path\")",
        "os.path wiring back onto os",
    );

    let functools_text = read_rel(functools_mod);
    assert_contains(
        &functools_text,
        "register_module(\"functools\", attrs)",
        "functools registration",
    );
    assert_contains(
        &functools_text,
        "\"lru_cache\"",
        "functools lru_cache public surface",
    );
    assert_contains(
        &functools_text,
        "functools.lru_cache_wrapper",
        "functools lru_cache wrapper surface",
    );

    let http_text = read_rel(http_mod);
    assert_contains(
        &http_text,
        "register_module(\"urllib.parse\", parse_attrs)",
        "urllib.parse registration",
    );

    let json_text = read_rel(json_mod);
    assert_contains(
        &json_text,
        "register_module(\"json\", attrs)",
        "json native registration",
    );

    let typing_text = read_rel(typing_mod);
    assert_contains(
        &typing_text,
        "register_module(\"typing\", attrs)",
        "typing native registration",
    );

    let enum_text = read_rel(enum_mod);
    assert_contains(
        &enum_text,
        "register_module(\"enum\", attrs)",
        "enum native registration",
    );
}

#[test]
fn issue_1104_scope_correction_invariant_2026_07_05_urllib_parse_is_present() {
    let http_text = read_rel("src/runtime/stdlib/http_mod.rs");

    for api_name in [
        "urlparse",
        "urlsplit",
        "urlencode",
        "quote",
        "quote_from_bytes",
        "quote_plus",
        "unquote",
        "unquote_plus",
        "parse_qs",
        "parse_qsl",
        "urlunparse",
        "urljoin",
        "urldefrag",
    ] {
        assert_contains(
            &http_text,
            &format!("\"{api_name}\""),
            "urllib.parse dispatcher table",
        );
    }

    for result_class in [
        "ParseResult",
        "ParseResultBytes",
        "SplitResult",
        "SplitResultBytes",
        "DefragResult",
        "DefragResultBytes",
    ] {
        assert_contains(
            &http_text,
            &format!("\"{result_class}\""),
            "urllib.parse result-class surface",
        );
    }

    assert_contains(
        &http_text,
        "register_module(\"urllib.parse\", parse_attrs)",
        "2026-07-05 scope correction invariant",
    );
}

#[test]
fn issue_1104_perf_pins_match_current_repo_evidence() {
    let pins = perf_pin_names();

    for required in [
        "urllib_parse_1419.toml",
        "functools_1451.toml",
        "enum_1448.toml",
    ] {
        assert!(
            pins.iter().any(|pin| pin == required),
            "missing required #1104 current-state perf evidence pin `{required}`"
        );
    }

    assert_eq!(
        pins_for_module(&pins, "os_path"),
        vec!["os_path_1432.toml"],
        "os.path current-state perf evidence drifted; update #1104 gate if the repo's exact pin set changed",
    );

    let blocked_missing = vec![
        ("json-native", pins_for_module(&pins, "json")),
        ("typing-lightweight", pins_for_module(&pins, "typing")),
        ("posixpath-native", pins_for_module(&pins, "posixpath")),
    ]
    .into_iter()
    .filter_map(|(module, matches)| {
        if matches.is_empty() {
            Some(module)
        } else {
            None
        }
    })
    .collect::<Vec<_>>();

    assert_eq!(
        blocked_missing,
        vec!["json-native", "typing-lightweight", "posixpath-native"],
        "TODO(#1104): do not invent missing perf evidence. Expected current blocked modules are json-native, typing-lightweight, and posixpath-native; if new exact pins were added, update this gate to record them explicitly.",
    );
}
