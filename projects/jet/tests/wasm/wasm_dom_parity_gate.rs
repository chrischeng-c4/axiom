// <HANDWRITE gap="codegen:external-tool-benchmark-gate" tracker="jet-wasm-dom-parity-gate" reason="Aggregate gate driving the advanced WASM verification script; becomes CODEGEN with the gate generator primitive.">
//! Replacement gate: `jet build --wasm` behaves like `jet build`.
//!
//! This test is the tests-tree owner of the block claim in
//! `tests/wasm/README.md`: for the same app and gesture, the FE-on-WASM
//! output must match the FE-on-DOM oracle's externally observable
//! behavior — rendered content, interaction outcomes, and visual
//! surface, all captured through `jet bb`.
//!
//! It drives `scripts/verify-advanced-wasm-gates.sh`, which aggregates:
//! WASM build config/manifest tests, the jet-wasm runtime subset,
//! renderer output, WebGPU default scaffold + runtime status/visual
//! probe, TSX→Rust library lowering fixtures, the MUI and AntD visual
//! DOM-vs-WASM parity gates (`jet bb capture/screenshot`, pHash
//! comparison), and the DOM oracle parity skeleton.
//!
//! The aggregated tests skip themselves gracefully when a browser or
//! WASM toolchain is unavailable; a red anywhere is a behavior gap
//! between `jet build --wasm` and the DOM oracle and fails this gate.

#[path = "../harness/mod.rs"]
mod harness;

use std::process::Command;

#[test]
fn jet_build_wasm_matches_dom_build_behavior() {
    const GATE: &str = "wasm-dom-parity-gate";
    let root = harness::repo_root();
    if !harness::require_tools(GATE, &["bash", "cargo"]) {
        return;
    }

    // The aggregate script runs nested cargo test invocations; keep the
    // toolchain environment correct for this repo (the `cc` shim on PATH
    // shadows the system C compiler and breaks cargo link steps).
    let status = Command::new("bash")
        .arg("projects/jet/scripts/verify-advanced-wasm-gates.sh")
        .current_dir(&root)
        .env("CC", "/usr/bin/cc")
        .env(
            "PATH",
            format!("/usr/bin:{}", std::env::var("PATH").unwrap_or_default()),
        )
        .status()
        .expect("spawning verify-advanced-wasm-gates.sh");

    assert!(
        status.success(),
        "[{GATE}] verify-advanced-wasm-gates.sh reported a DOM/WASM behavior gap \
         (exit {status:?}); see the failing section in the output above"
    );
}
// </HANDWRITE>
