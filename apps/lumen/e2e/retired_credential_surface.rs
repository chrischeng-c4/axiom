// HANDWRITE gap="missing-generator:e2e-test:retired-credential-surface" tracker="2875" reason="A residue gate has to read the repository as text, hold a reviewed exclusion list, and avoid matching its own source; no generator primitive emits that shape."
//! #2875: phase 1's closing gate — the retired credential model leaves no
//! mention behind that reads as current behaviour.
//!
//! Phase 1 removed the bearer credential model from five surfaces in five
//! separate work items. Retirement split that way finishes at ninety percent,
//! and the ten percent is never logic — it is a README paragraph, an overlay,
//! a design note. Nothing fails when a document still tells a reader to set an
//! environment variable that no longer exists, so nothing does, and the
//! surviving mention becomes the documentation of record for whoever finds it
//! next.
//!
//! This file turns "we deleted it" into a machine answer.
//!
//! # What an exclusion means here
//!
//! A surviving mention is only acceptable when it is *about* the removal: a
//! negative assertion whose deletion would delete the proof, or a migration
//! note describing behaviour that is gone.
//!
//! Two exact multisets hold the reviewed production and evidence sites. A row
//! contains its repository path, matched symbols, and full line after
//! whitespace normalization. An extra, missing, moved, changed, or duplicate
//! row fails. This makes every change to the surviving evidence visible.
//!
//! # Scope
//!
//! `apps/lumen` and `acceptance`. Deliberately not swept:
//!
//! - `libs/service-auth` — `apps/courier`, `apps/defer` and `apps/keep` link
//!   its registry types. Those symbols are legitimately alive.
//! - `libs/cli-std` — `apps/beam` still consumes the credential helpers. lumen
//!   stopped calling them; nothing was removed.
//! - `libs/service-k8s` — shared render helpers serve other apps and are
//!   checked by that library's own tests. This app gate does not own them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Every retired symbol is assembled from fragments at runtime, so this file
/// contains none of the strings it forbids. A residue gate that matches its own
/// source is a gate whose only stable state is red, and the fix a maintainer
/// reaches for is to exclude the gate — which is how these checks die.
fn retired_symbols() -> Vec<Symbol> {
    vec![
        // The env vars. The bare token variable is a prefix of the registry
        // variable below it, so it matches on a word boundary: a naive
        // substring match reports one site under two names and can never tell
        // you that one of the two is gone and the other is not.
        Symbol::word(&["LUMEN", "_", "TOKEN"]),
        Symbol::plain(&["LUMEN", "_", "TOKEN", "_REGISTRY_FILE"]),
        Symbol::plain(&["LUMEN", "_IDENTITY", "_REGISTRY_FILE"]),
        Symbol::plain(&["LUMEN", "_AUTH_GOOGLE", "_AUDIENCES"]),
        // The CRD fields, in both the schema spelling and the Rust spelling.
        Symbol::plain(&["tokens", "Secret"]),
        Symbol::plain(&["tokens", "_secret"]),
        Symbol::plain(&["identity", "Audiences"]),
        Symbol::plain(&["identity", "_audiences"]),
        // The CSI projection the fields drove.
        Symbol::plain(&["tokens", "Secret", "ProviderClass"]),
        Symbol::plain(&["tokens", "Secret", "CsiDriver"]),
        // The types behind them.
        Symbol::plain(&["Identity", "Grant"]),
        Symbol::plain(&["Grant", "Role"]),
        Symbol::plain(&["Metadata", "Token", "Source"]),
    ]
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Row {
    path: String,
    symbols: Vec<String>,
    line: String,
}

#[derive(Clone)]
struct Inventory {
    production: Vec<Row>,
    evidence: Vec<Row>,
}

// Symbol markers use their retired_symbols() indexes. The source therefore
// contains no literal retired symbol and cannot match itself.
const EXPECTED_ROWS: &str = r#"
P|apps/lumen/src/bin/lumen.rs|5|// shared module (`select_token`, `cr_@5@`, `secret_data_bytes`)
P|apps/lumen/src/operator/fleet.rs|4|json!({ "@4@": "lumen-tokens" }),
P|apps/lumen/src/operator/fleet.rs|6|json!({ "@6@": ["https://lumen.example.com"] }),
P|apps/lumen/src/operator/mod.rs|4,6|for retired in ["@6@", "identities", "@4@"] {
P|apps/lumen/src/operator/mod.rs|4,8|assert!(!yaml.contains("@8@"), "{yaml}");
P|apps/lumen/src/spec.rs|4|The CRD configures **no** credential source. `spec.@4@`,
P|apps/lumen/src/spec.rs|6|`spec.identities` and `spec.@6@` are gone (#2872): a Lumen CR
E|apps/lumen/e2e/cli_convention.rs|4|"@4@",
E|apps/lumen/e2e/cli_convention.rs|4,8|"@8@",
E|apps/lumen/e2e/cli_convention.rs|6|"@6@",
E|apps/lumen/e2e/cli_credential_paths_retired.rs|0|/// telling readers to send `Authorization: Bearer <@0@>` for as long as
E|apps/lumen/e2e/cli_credential_paths_retired.rs|5|needle(&["resolve_cr_@5@", "("]),
E|apps/lumen/e2e/operator_render.rs|1|assert!(!names.contains(&"@1@".to_string()));
E|apps/lumen/e2e/operator_render.rs|1|"@1@",
E|apps/lumen/e2e/operator_render.rs|2|"@2@",
E|apps/lumen/e2e/operator_render.rs|4|// #2870: the CRD no longer teaches the retired registry. `@4@` and
E|apps/lumen/e2e/operator_render.rs|1|"@1@",
E|apps/lumen/e2e/operator_render.rs|2|"@2@",
E|apps/lumen/e2e/operator_render.rs|6|/// checked for came from `spec.@6@`, and both are gone. The
E|apps/lumen/e2e/operator_render.rs|3|"@3@",
E|apps/lumen/e2e/operator_render.rs|1|"@1@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|6|"@6@": ["https://lumen.acme.internal"],
E|apps/lumen/e2e/operator_retired_credential_projection.rs|4|json!({ "auth": auth, "@4@": "lumen-tokens" }),
E|apps/lumen/e2e/operator_retired_credential_projection.rs|4|"@4@": "lumen-tokens",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|6|"@6@": ["https://lumen.acme.internal"],
E|apps/lumen/e2e/operator_retired_credential_projection.rs|4,8|"@8@": "lumen-tokens-spc",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|2|"@2@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|1|"@1@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|3|"@3@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|1|"@1@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|2|"@2@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|3|"@3@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|4|"@4@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|4,8|"@8@",
E|apps/lumen/e2e/operator_retired_credential_projection.rs|6|"@6@",
E|apps/lumen/e2e/spec_cli.rs|4|"@4@",
E|apps/lumen/e2e/spec_cli.rs|1|"@1@",
E|apps/lumen/e2e/spec_cli.rs|3|!storage.contains("@3@"),
E|apps/lumen/e2e/spec_cli.rs|1|for retired in ["Authorization: Bearer", "@1@"] {
E|apps/lumen/e2e/spec_cli.rs|1|!q.contains("@1@"),
E|apps/lumen/examples/lumen-cr.yaml|4|# SubjectAccessReview. A CR that still set `@4@`, `identities` or
E|apps/lumen/examples/lumen-cr.yaml|6|# `@6@` is rejected by the API server, not silently ignored.
"#;

fn expected_inventory() -> Inventory {
    let symbols = retired_symbols();
    let mut inventory = Inventory {
        production: Vec::new(),
        evidence: Vec::new(),
    };

    for source in EXPECTED_ROWS.lines().filter(|line| !line.is_empty()) {
        let mut columns = source.splitn(4, '|');
        let category = columns.next().expect("expected row category");
        let path = columns.next().expect("expected row path");
        let indexes = columns.next().expect("expected row symbol indexes");
        let mut line = columns
            .next()
            .expect("expected normalized line")
            .to_string();
        let mut matched = Vec::new();
        for raw_index in indexes.split(',') {
            let index: usize = raw_index.parse().expect("valid expected symbol index");
            let symbol = symbols.get(index).expect("known expected symbol index");
            line = line.replace(&format!("@{index}@"), &symbol.text);
            matched.push(symbol.text.clone());
        }
        assert!(!line.contains('@'), "unresolved symbol marker in {path}");
        let row = Row {
            path: path.to_string(),
            symbols: matched,
            line,
        };
        match category {
            "P" => inventory.production.push(row),
            "E" => inventory.evidence.push(row),
            other => panic!("unknown expected row category {other}"),
        }
    }

    assert_eq!(inventory.production.len(), 7, "production inventory");
    assert_eq!(inventory.evidence.len(), 35, "evidence inventory");
    assert!(
        inventory
            .production
            .iter()
            .all(|row| row.path.starts_with("apps/lumen/src/")),
        "production inventory path"
    );
    assert!(
        inventory
            .evidence
            .iter()
            .all(|row| !row.path.starts_with("apps/lumen/src/")),
        "evidence inventory path"
    );
    inventory
}

fn counts(rows: &[Row]) -> BTreeMap<Row, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(row.clone()).or_default() += 1;
    }
    counts
}

fn compare_category(name: &str, actual: &[Row], expected: &[Row], errors: &mut Vec<String>) {
    let actual = counts(actual);
    let expected = counts(expected);
    for (row, actual_count) in &actual {
        let expected_count = expected.get(row).copied().unwrap_or(0);
        if *actual_count > expected_count {
            errors.push(format!(
                "extra {name} row x{}: {} | [{}] | {}",
                actual_count - expected_count,
                row.path,
                row.symbols.join(", "),
                row.line
            ));
        }
    }
    for (row, expected_count) in &expected {
        let actual_count = actual.get(row).copied().unwrap_or(0);
        if *expected_count > actual_count {
            errors.push(format!(
                "missing {name} row x{}: {} | [{}] | {}",
                expected_count - actual_count,
                row.path,
                row.symbols.join(", "),
                row.line
            ));
        }
    }
}

fn verify_inventory(actual: &[Row], expected: &Inventory) -> Result<(), String> {
    let (production, evidence): (Vec<_>, Vec<_>) = actual
        .iter()
        .cloned()
        .partition(|row| row.path.starts_with("apps/lumen/src/"));
    let mut errors = Vec::new();
    compare_category("production", &production, &expected.production, &mut errors);
    compare_category("evidence", &evidence, &expected.evidence, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

// ---------------------------------------------------------------------------

struct Symbol {
    text: String,
    /// When set, a match is only real if neither neighbouring character is a
    /// word character.
    word_boundary: bool,
}

impl Symbol {
    fn plain(parts: &[&str]) -> Self {
        Self {
            text: parts.concat(),
            word_boundary: false,
        }
    }
    fn word(parts: &[&str]) -> Self {
        Self {
            text: parts.concat(),
            word_boundary: true,
        }
    }

    fn found_in(&self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut from = 0;
        while let Some(rel) = line[from..].find(&self.text) {
            let start = from + rel;
            let end = start + self.text.len();
            if !self.word_boundary {
                return true;
            }
            let before_ok = start == 0 || !is_word_byte(bytes[start - 1]);
            let after_ok = end == bytes.len() || !is_word_byte(bytes[end]);
            if before_ok && after_ok {
                return true;
            }
            from = start + 1;
        }
        false
    }
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn repo_root() -> PathBuf {
    // `apps/lumen` -> `apps` -> repository root.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("apps/lumen lives two levels below the repository root")
        .to_path_buf()
}

/// Text files under the swept roots. Binary and build output are skipped; a
/// residue gate that tries to read `target/` measures the compiler, not the
/// repository.
fn swept_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for sub in ["apps/lumen", "acceptance"] {
        collect(&root.join(sub), &mut out);
    }
    out.sort();
    out
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(name.as_ref(), "target" | ".git" | "node_modules") {
                continue;
            }
            collect(&path, out);
        } else if is_text(&path) {
            out.push(path);
        }
    }
}

fn is_text(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(
            "rs" | "md"
                | "yaml"
                | "yml"
                | "toml"
                | "json"
                | "sh"
                | "txt"
                | "tf"
                | "tfvars"
                | "env"
                | "dockerfile"
                | "sql"
                | "py"
        )
    ) || matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some("Dockerfile" | "Dockerfile.release" | "Dockerfile.test" | "Makefile")
    )
}

fn scan(root: &Path) -> Vec<Row> {
    let symbols = retired_symbols();
    let mut rows = Vec::new();
    for file in swept_files(root) {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        for line in text.lines() {
            let found: Vec<String> = symbols
                .iter()
                .filter(|s| s.found_in(line))
                .map(|s| s.text.clone())
                .collect();
            if !found.is_empty() {
                rows.push(Row {
                    path: rel.clone(),
                    symbols: found,
                    line: line.split_whitespace().collect::<Vec<_>>().join(" "),
                });
            }
        }
    }
    rows
}

// ---------------------------------------------------------------------------

/// AC1: the gate covers every symbol the retirement removed.
///
/// Asserted by name rather than by count so that deleting one from the list is
/// a visible edit, not a number that still looks plausible.
#[test]
fn the_gate_covers_every_retired_symbol() {
    let have: Vec<String> = retired_symbols().into_iter().map(|s| s.text).collect();
    let want = [
        ["LUMEN", "_", "TOKEN"].concat(),
        ["LUMEN", "_", "TOKEN", "_REGISTRY_FILE"].concat(),
        ["LUMEN", "_IDENTITY", "_REGISTRY_FILE"].concat(),
        ["LUMEN", "_AUTH_GOOGLE", "_AUDIENCES"].concat(),
        ["tokens", "Secret"].concat(),
        ["tokens", "_secret"].concat(),
        ["identity", "Audiences"].concat(),
        ["identity", "_audiences"].concat(),
        ["tokens", "Secret", "ProviderClass"].concat(),
        ["tokens", "Secret", "CsiDriver"].concat(),
        ["Identity", "Grant"].concat(),
        ["Grant", "Role"].concat(),
        ["Metadata", "Token", "Source"].concat(),
    ];
    for symbol in want {
        assert!(have.contains(&symbol), "the gate stopped covering {symbol}");
    }
    assert_eq!(have.len(), 13, "covered symbols: {have:?}");
}

/// The word-boundary rule, asserted directly. Without it the shorter env var
/// matches inside the longer one and the two can never be told apart.
#[test]
fn the_short_env_var_does_not_match_inside_the_long_one() {
    let short = Symbol::word(&["LUMEN", "_", "TOKEN"]);
    let long_line = format!(
        "  {} = /etc/x",
        ["LUMEN", "_", "TOKEN", "_REGISTRY_FILE"].concat()
    );
    assert!(
        !short.found_in(&long_line),
        "the short variable matched inside the long one: {long_line}"
    );
    let own_line = format!("  {}: value", ["LUMEN", "_", "TOKEN"].concat());
    assert!(
        short.found_in(&own_line),
        "the short variable stopped matching itself"
    );
}

/// AC3 + AC4: every surviving production and evidence row is exact.
#[test]
fn no_retired_credential_symbol_survives_unaccounted() {
    let root = repo_root();
    let rows = scan(&root);
    let expected = expected_inventory();
    if let Err(error) = verify_inventory(&rows, &expected) {
        panic!("the retired credential inventory does not match the tree:\n{error}");
    }
}

#[test]
fn exact_inventory_rejects_required_mutations() {
    let baseline = scan(&repo_root());
    let expected = expected_inventory();
    assert!(verify_inventory(&baseline, &expected).is_ok());

    let mut eighth_production = baseline.clone();
    let mut extra = expected.production[0].clone();
    extra.path = "apps/lumen/src/extra.rs".to_string();
    eighth_production.push(extra);
    assert_ne!(eighth_production, baseline, "production fixture changed");
    assert!(
        verify_inventory(&eighth_production, &expected).is_err(),
        "an eighth production row must fail"
    );

    let mut moved_evidence = baseline.clone();
    let evidence = moved_evidence
        .iter_mut()
        .find(|row| !row.path.starts_with("apps/lumen/src/"))
        .expect("live evidence row");
    evidence.path = "apps/lumen/e2e/moved_surface.rs".to_string();
    assert_ne!(moved_evidence, baseline, "evidence fixture changed");
    assert!(
        verify_inventory(&moved_evidence, &expected).is_err(),
        "moving evidence to a new path must fail"
    );

    let mut stale_expected = expected.clone();
    let mut stale = stale_expected.evidence[0].clone();
    stale.path = "apps/lumen/e2e/stale_surface.rs".to_string();
    stale_expected.evidence.push(stale);
    assert_ne!(
        stale_expected.evidence.len(),
        expected.evidence.len(),
        "stale expected fixture changed"
    );
    assert!(
        verify_inventory(&baseline, &stale_expected).is_err(),
        "an expected row with no live counterpart must fail"
    );
}

/// AC2's mechanism, exercised on synthetic input: the scanner reports a line
/// that reintroduces a retired symbol.
///
/// This is the unit-level half. The behavioural half — reintroducing each
/// symbol into the real tree and watching this gate fail — is recorded on
/// #2875 with the sites it named, because a gate first observed passing has
/// never been shown to detect anything.
#[test]
fn the_scanner_reports_a_reintroduced_symbol() {
    for symbol in retired_symbols() {
        let line = format!("  set {} to the value your operator gave you", symbol.text);
        assert!(
            symbol.found_in(&line),
            "a document reintroducing {} would pass unnoticed",
            symbol.text
        );
    }
}

/// The gate must not match its own source: every symbol is assembled at
/// runtime. If this fails, the next maintainer's cheapest fix is to exclude
/// this file, and the gate stops being one.
#[test]
fn the_gate_does_not_match_itself() {
    let own = fs::read_to_string(file!())
        .or_else(|_| fs::read_to_string(repo_root().join(file!())))
        .expect("this test file is readable from the repository root");
    let offenders: Vec<String> = retired_symbols()
        .into_iter()
        .filter(|s| own.lines().any(|l| s.found_in(l)))
        .map(|s| s.text)
        .collect();
    assert!(
        offenders.is_empty(),
        "this file contains the literal strings it forbids: {offenders:?}"
    );
}
