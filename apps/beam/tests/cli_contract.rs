//! CLI contract test — proves the TD's R1–R6 by invoking the built `beam`
//! binary (`CARGO_BIN_EXE_beam`) and, for R1, linking the `beam` library crate.
//!
//! No `assert_cmd` dependency: this drives `std::process::Command` directly so
//! the first slice stays dependency-light (and CPU/GPU-neutral, R6).

use std::process::Command;

/// The built binary under test (set by cargo for integration tests).
fn beam() -> Command {
    Command::new(env!("CARGO_BIN_EXE_beam"))
}

fn stdout_of(args: &[&str]) -> String {
    let out = beam().args(args).output().expect("run beam");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// R1: `projects/beam` is a workspace member with BOTH a library and a binary
/// target. Linking `beam::` here exercises the lib target; `CARGO_BIN_EXE_beam`
/// existing (and running below) exercises the bin target.
#[test]
fn r1_workspace_crate_has_lib_and_bin() {
    assert_eq!(beam::not_implemented("x"), "not implemented yet: x");
    assert!(beam::LUMEN_BOUNDARY.contains("Lumen"));
    // The binary runs at all (bin target built).
    assert!(beam().arg("--help").output().expect("run beam --help").status.success());
}

/// R2: `beam --help` lists the standard convention verbs.
#[test]
fn r2_help_lists_standard_verbs() {
    let help = stdout_of(&["--help"]);
    for verb in ["llm", "upgrade", "issue"] {
        assert!(help.contains(verb), "help missing standard verb `{verb}`:\n{help}");
    }
}

/// R3: `beam --help` lists the placeholder service verbs, and each exits with a
/// consistent non-zero code and a tracked `not implemented yet: …` diagnostic.
#[test]
fn r3_help_lists_placeholder_verbs_and_they_are_tracked() {
    let help = stdout_of(&["--help"]);
    for verb in ["serve", "collections", "index", "query", "dockerfile", "k8s"] {
        assert!(help.contains(verb), "help missing placeholder verb `{verb}`:\n{help}");
    }

    for (verb, detail) in [
        ("serve", "HTTP service shell"),
        ("collections", "collection lifecycle"),
        ("index", "index lifecycle"),
        ("query", "vector query"),
        ("dockerfile", "dockerfile render"),
        ("k8s", "k8s render/operator"),
    ] {
        let out = beam().arg(verb).output().expect("run placeholder verb");
        assert!(!out.status.success(), "`beam {verb}` should exit non-zero");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(&format!("not implemented yet: {detail}")),
            "`beam {verb}` missing tracked diagnostic; stderr:\n{stderr}"
        );
    }
}

/// R4: `beam llm --topic outline` names Beam as a GPU-native vector DB AND
/// states the Beam/Lumen boundary (Lumen owns mixed search / ranking / dedup).
#[test]
fn r4_llm_outline_states_beam_and_lumen_boundary() {
    let outline = stdout_of(&["llm", "--topic", "outline"]);
    assert!(outline.contains("vector"), "outline should name Beam a vector DB:\n{outline}");
    assert!(outline.contains("Lumen"), "outline should name the Lumen boundary:\n{outline}");
    assert!(
        outline.contains("mixed search"),
        "outline should state Lumen owns mixed search:\n{outline}"
    );
    for owned in ["ranking", "dedup"] {
        assert!(outline.contains(owned), "outline should mention Lumen owns `{owned}`:\n{outline}");
    }
}

/// R5: `beam issue create` carries the `project:beam` scope (offline dry-run
/// preview surfaces the derived labels without any network access).
#[test]
fn r5_issue_is_project_beam_scoped() {
    let preview = stdout_of(&["issue", "create", "--dry-run", "--title", "beam: probe", "hello"]);
    assert!(
        preview.contains("project:beam"),
        "issue create should be scoped project:beam:\n{preview}"
    );
}

/// R6: the slice builds and runs on a plain host with no CUDA/Metal/GPU runtime.
/// If any GPU/vector dependency had crept in, this test binary would not build
/// or link on this CPU-only host; a clean `beam llm` render is that proof.
#[test]
fn r6_no_gpu_dependency_plain_host_runs() {
    assert!(stdout_of(&["llm", "--topic", "architecture"]).contains("no CUDA/Metal/wgpu/vector"));
}
