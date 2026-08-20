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
//! note describing behaviour that is gone. Those are listed below, each with
//! the reason it survives and the exact number of lines it may occupy.
//!
//! The line count is the part that matters. A bare path exclusion turns a file
//! into a blind spot — the very first place a re-grown credential path would
//! land, because the gate already agreed not to look there. Pinning the count
//! means a file that is allowed two retirement notes cannot quietly acquire a
//! third line that hands someone a credential recipe. Changing a count is a
//! deliberate edit that shows up in review; that is the whole point. A stale
//! entry — one that matches nothing — fails too, so the list shrinks as the
//! notes age out. An empty list is the goal.
//!
//! # Scope
//!
//! `apps/lumen` and `acceptance`. Deliberately not swept:
//!
//! - `libs/service-auth` — `apps/courier`, `apps/defer` and `apps/keep` link
//!   its registry types. Those symbols are legitimately alive.
//! - `libs/cli-std` — `apps/beam` still consumes the credential helpers. lumen
//!   stopped calling them; nothing was removed.
//! - `libs/service-k8s` — one test fixture uses a retired variable name as
//!   example env, not product wiring. It also belongs to a paused session's
//!   modified set and must not be touched from this chain (#2880).

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

/// A file that may keep a bounded number of retirement mentions, and why.
struct Allowance {
    /// Repository-relative path. A trailing `/` makes it a directory prefix.
    path: &'static str,
    /// Number of *lines* — not matches — the site may contain. A line naming
    /// two retired symbols is one line.
    lines: usize,
    reason: &'static str,
}

const ALLOWANCES: &[Allowance] = &[
    Allowance {
        path: "apps/lumen/src/operator/mod.rs",
        lines: 2,
        reason: "Negative assertions: the rendered operator manifest is checked for the absence \
                 of the retired field and of the CSI projection it drove. Deleting these deletes \
                 the proof.",
    },
    Allowance {
        path: "apps/lumen/src/operator/fleet.rs",
        lines: 2,
        reason: "Negative assertions. The fleet is the one path that takes free-form spec JSON, \
                 so a platform team's stale defaults would otherwise be merged in and dropped \
                 without a word. This test feeds each retired field in and requires a rejection.",
    },
    Allowance {
        path: "apps/lumen/src/spec.rs",
        lines: 2,
        reason: "Migration note in the shipped specification: the fields are named so a reader \
                 with an old CR learns their apply will be rejected by strict decoding rather \
                 than silently ignored. Naming them is the service the note performs.",
    },
    Allowance {
        path: "apps/lumen/src/bin/lumen.rs",
        lines: 1,
        reason: "A comment pointing at the shared cli-std helpers, which still exist and still \
                 have their own tests there because another app consumes them. It records that \
                 lumen no longer calls them, and names the gate that keeps it that way.",
    },
    Allowance {
        path: "apps/lumen/docs/deployment-handoff.md",
        lines: 2,
        reason: "Migration note for operators holding a CR written against the old schema.",
    },
    Allowance {
        path: "apps/lumen/examples/lumen-cr.yaml",
        lines: 2,
        reason: "Migration note in the example CR, where someone copying an old manifest will \
                 actually read it.",
    },
    Allowance {
        path: "apps/lumen/e2e/cli_convention.rs",
        lines: 3,
        reason: "Negative assertions on the CLI's help surface for the snapshot verbs.",
    },
    Allowance {
        path: "apps/lumen/e2e/cli_credential_paths_retired.rs",
        lines: 2,
        reason: "#2873's gate. It names the retired header recipe in prose to record what the \
                 committed OpenAPI snapshot used to publish, and forbids it in code.",
    },
    Allowance {
        path: "apps/lumen/e2e/operator_render.rs",
        lines: 9,
        reason: "Negative assertions across the rendered StatefulSet, its env, and the checked-in \
                 CRD YAML.",
    },
    Allowance {
        path: "apps/lumen/e2e/operator_retired_credential_projection.rs",
        lines: 14,
        reason: "#2870's gate — the file whose entire purpose is proving the projection is gone. \
                 It necessarily names every symbol it forbids.",
    },
    Allowance {
        path: "apps/lumen/e2e/spec_cli.rs",
        lines: 5,
        reason: "Negative assertions on the generated specification and its committed snapshot.",
    },
];

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

/// One line that names at least one retired symbol.
struct Hit {
    path: String,
    line_no: usize,
    line: String,
    symbols: Vec<String>,
}

fn scan(root: &Path) -> Vec<Hit> {
    let symbols = retired_symbols();
    let mut hits = Vec::new();
    for file in swept_files(root) {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        let rel = file
            .strip_prefix(root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        for (idx, line) in text.lines().enumerate() {
            let found: Vec<String> = symbols
                .iter()
                .filter(|s| s.found_in(line))
                .map(|s| s.text.clone())
                .collect();
            if !found.is_empty() {
                hits.push(Hit {
                    path: rel.clone(),
                    line_no: idx + 1,
                    line: line.trim().chars().take(160).collect(),
                    symbols: found,
                });
            }
        }
    }
    hits
}

fn allowance_for(path: &str) -> Option<&'static Allowance> {
    ALLOWANCES.iter().find(|a| {
        if let Some(prefix) = a.path.strip_suffix('/') {
            path.starts_with(prefix)
        } else {
            path == a.path
        }
    })
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

/// AC3 + AC4: nothing survives that is not a named, reasoned, bounded
/// exclusion — and every exclusion still describes something real.
#[test]
fn no_retired_credential_symbol_survives_unaccounted() {
    let root = repo_root();
    let hits = scan(&root);

    let mut per_path: BTreeMap<String, Vec<&Hit>> = BTreeMap::new();
    for hit in &hits {
        per_path.entry(hit.path.clone()).or_default().push(hit);
    }

    let mut unaccounted = Vec::new();
    let mut counted: BTreeMap<&'static str, usize> = BTreeMap::new();
    for (path, path_hits) in &per_path {
        match allowance_for(path) {
            Some(allowance) => *counted.entry(allowance.path).or_default() += path_hits.len(),
            None => {
                for hit in path_hits {
                    unaccounted.push(format!(
                        "  {}:{} [{}]\n      {}",
                        hit.path,
                        hit.line_no,
                        hit.symbols.join(", "),
                        hit.line
                    ));
                }
            }
        }
    }

    assert!(
        unaccounted.is_empty(),
        "the retired credential model survives at {} site(s) with no stated reason.\n\
         Delete the site, or add it to ALLOWANCES with the reason it is about the removal \
         rather than a live instruction:\n{}",
        unaccounted.len(),
        unaccounted.join("\n")
    );

    let mut budget_errors = Vec::new();
    for allowance in ALLOWANCES {
        let actual = counted.get(allowance.path).copied().unwrap_or(0);
        if actual == 0 {
            budget_errors.push(format!(
                "  {} is excluded but names nothing any more — delete the entry.\n      (was: {})",
                allowance.path, allowance.reason
            ));
        } else if actual != allowance.lines {
            budget_errors.push(format!(
                "  {} is allowed {} line(s) but has {}.\n      reason on file: {}",
                allowance.path, allowance.lines, actual, allowance.reason
            ));
        }
    }

    assert!(
        budget_errors.is_empty(),
        "the exclusion list no longer matches the tree. Every entry must name something real \
         and pin how much of it there is:\n{}",
        budget_errors.join("\n")
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
