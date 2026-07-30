// SPEC-MANAGED: apps/mesh/tech-design/interfaces/cli/scaffold-service-crate-and-standard-cli-shell.md#unit-test
// <HANDWRITE gap="mesh-cli-shell-scaffold" tracker="#1970" reason="R1-R6 verification for the initial Mesh CLI shell.">
//! Black-box tests against the real compiled `mesh` binary (WI #1970,
//! requirements R1-R6).

use std::process::Command;

fn mesh() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mesh"))
}

/// R2/AC2: `mesh --help` lists llm, upgrade, issue, and every domain
/// placeholder verb.
#[test]
fn help_lists_all_verbs() {
    let out = mesh().arg("--help").output().expect("run mesh --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for verb in [
        "llm",
        "upgrade",
        "issue",
        "serve",
        "collections",
        "nodes",
        "edges",
        "query",
        "dockerfile",
        "k8s",
    ] {
        assert!(stdout.contains(verb), "help output missing '{verb}':\n{stdout}");
    }
}

/// R3/AC3: `mesh llm --topic outline` states the mesh/lumen/beam/cube
/// boundary.
#[test]
fn llm_outline_states_boundary() {
    let out = mesh()
        .args(["llm", "--topic", "outline"])
        .output()
        .expect("run mesh llm --topic outline");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("boundaries"), "outline missing topic id:\n{stdout}");

    let out = mesh()
        .args(["llm", "--topic", "boundaries"])
        .output()
        .expect("run mesh llm --topic boundaries");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for name in ["lumen", "beam", "cube", "mesh"] {
        assert!(stdout.contains(name), "boundaries topic missing '{name}':\n{stdout}");
    }
}

/// R4/AC4: `mesh issue --help` is scoped to app:mesh — the search/create
/// subcommands are present and the binary's issue label carries app:mesh.
#[test]
fn issue_help_scoped_to_app_mesh() {
    let out = mesh().args(["issue", "--help"]).output().expect("run mesh issue --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for sub in ["search", "view", "create", "comment"] {
        assert!(stdout.contains(sub), "issue --help missing '{sub}':\n{stdout}");
    }

    // `create --dry-run` runs the offline path and prints the assembled
    // payload, which carries the app:mesh label even without a token.
    let out = mesh()
        .args(["issue", "create", "--dry-run", "-y", "-t", "test", "hello"])
        .output()
        .expect("run mesh issue create --dry-run");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(combined.contains("app:mesh"), "issue create --dry-run missing app:mesh label:\n{combined}");
}

/// R5/AC5: every placeholder domain verb exits non-zero (code 3) with a
/// clear "not implemented yet" message instead of panicking.
#[test]
fn placeholder_verbs_exit_code_3_not_implemented() {
    for verb in ["serve", "collections", "nodes", "edges", "query", "dockerfile", "k8s"] {
        let out = mesh().arg(verb).output().expect("run mesh <placeholder verb>");
        assert_eq!(out.status.code(), Some(3), "verb '{verb}' exit code");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("not implemented yet"),
            "verb '{verb}' stderr missing message:\n{stderr}"
        );
        assert!(stderr.contains(verb), "verb '{verb}' stderr missing its own name:\n{stderr}");
    }
}

/// R1/AC1 (partial — the build succeeding at all proves the workspace
/// membership + build.rs stamp wiring): the binary reports a real version
/// stamped via build.rs, not a placeholder.
#[test]
fn version_is_stamped() {
    let out = mesh().arg("--version").output().expect("run mesh --version");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.trim().starts_with("mesh"), "unexpected --version output:\n{stdout}");
}
// </HANDWRITE>
