//! Governance lock for #1121: the C-API feasibility spike must stay present,
//! scoped as a decision artifact, and explicit about per-package routing.

use std::path::PathBuf;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn spike_doc() -> PathBuf {
    project_root().join("docs/native-extensions/c-api-feasibility-spike.md")
}

fn load_spike_doc() -> String {
    std::fs::read_to_string(spike_doc()).expect("read #1121 C-API feasibility spike doc")
}

#[test]
fn c_api_feasibility_spike_doc_exists() {
    assert!(spike_doc().is_file(), "#1121 spike doc must exist");
}

#[test]
fn c_api_feasibility_spike_declares_scope_and_non_goals() {
    let doc = load_spike_doc();
    for needle in [
        "# C-API Subset Emulation Feasibility Spike",
        "## Scope",
        "## Non-Goals",
        "No CPython ABI loader is implemented by this issue.",
        "No promise of `numpy`, `lxml`, `grpcio`, or `psycopg2` compatibility is made",
        "existing `ctypes` surface is sufficient",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1121 scope/non-goal marker: {needle}"
        );
    }
}

#[test]
fn c_api_feasibility_spike_keeps_required_repo_evidence_points() {
    let doc = load_spike_doc();
    for needle in [
        "projects/mamba/README.md",
        "projects/mamba/ecosystem_fixture_manifest.toml",
        "#2526 mamba native-extension loader not ready for numpy C core",
        "#2526 mamba native-extension loader not ready for cryptography Rust _rust extension",
        "projects/mamba/src/pkgmanage/pkgmgr/maturin_compat.rs",
        "bindings = \"pyo3\"",
        "projects/mamba/src/runtime/stdlib/ctypes_mod.rs",
        "viable C-extension ABI",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1121 repo evidence point: {needle}"
        );
    }
}

#[test]
fn c_api_feasibility_spike_covers_required_packages_and_decision_vocabulary() {
    let doc = load_spike_doc();
    for needle in [
        "| `numpy` |",
        "| `psycopg` |",
        "| `psycopg2` |",
        "| `lxml` |",
        "| `cryptography` |",
        "| `pydantic-core` |",
        "| `orjson` |",
        "| `protobuf` |",
        "| `grpcio` |",
        "`native mambalib replacement`",
        "`PyO3-native backend`",
        "`bridge/subprocess CPython`",
        "`no-go/defer`",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1121 package row or routing vocabulary: {needle}"
        );
    }
}

#[test]
fn c_api_feasibility_spike_preserves_requested_conclusion() {
    let doc = load_spike_doc();
    for needle in [
        "`numpy` is not a good candidate for partial CPython C-API emulation as the",
        "`lxml` is also not a good candidate for partial CPython C-API emulation as",
        "prefer Rust-native `pg` / mambalib or a",
        "partial CPython C-API emulation is not the recommended first path for the",
        "`numpy` / `psycopg` / `lxml` class",
        "#1120",
        "#1119",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1121 conclusion marker: {needle}"
        );
    }
}
