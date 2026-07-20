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

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in cli_contract.rs is hand-written pending codegen support">
/// R1: `apps/beam` is a workspace member with BOTH a library and a binary
/// target. Linking `beam::` here exercises the lib target; `CARGO_BIN_EXE_beam`
/// existing (and running below) exercises the bin target.
#[test]
fn r1_workspace_crate_has_lib_and_bin() {
    assert_eq!(beam::not_implemented("x"), "not implemented yet: x");
    assert!(beam::LUMEN_BOUNDARY.contains("Lumen"));
    // The binary runs at all (bin target built).
    assert!(beam().arg("--help").output().expect("run beam --help").status.success());
}
// </HANDWRITE>

/// R2: `beam --help` lists the standard convention verbs.
#[test]
fn r2_help_lists_standard_verbs() {
    let help = stdout_of(&["--help"]);
    for verb in ["llm", "upgrade", "issue"] {
        assert!(help.contains(verb), "help missing standard verb `{verb}`:\n{help}");
    }
}

/// R3: `beam --help` lists the service verbs. `serve`, `query`, `dockerfile`, `k8s`
/// are real subcommands whose `--help` exits successfully.
#[test]
fn r3_service_verbs_are_implemented() {
    let help = stdout_of(&["--help"]);
    for verb in ["serve", "query", "dockerfile", "k8s", "connect", "backup", "spec"] {
        assert!(help.contains(verb), "help missing service verb `{verb}`:\n{help}");
    }

    for verb in ["query", "dockerfile", "k8s"] {
        let out = beam().arg(verb).arg("--help").output().expect("run service verb help");
        assert!(out.status.success(), "`beam {verb} --help` should exit successfully");
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

/// R6 (superseded): the shell slice was GPU-neutral, but beam now ships a real
/// wgpu/Metal GPU flat k-NN index (see `beam bench`). This asserts the
/// architecture topic tells that truth — it names the wgpu/Metal GPU engine.
#[test]
fn r6_architecture_topic_names_gpu_engine() {
    let arch = stdout_of(&["llm", "--topic", "architecture"]);
    assert!(arch.contains("wgpu"), "architecture topic must name the wgpu backend");
    assert!(arch.contains("Metal"), "architecture topic must name the Metal (Apple) backend");
}
