// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#tests
// CODEGEN-BEGIN
//! Temp-backed issue store dirty-tree gate coverage (R1/R2/R3).
//!
//! Exercises `td_activate_inplace_allowing_dirty_spec_path` via the
//! `aw td review --apply` CLI surface (same gate, accessible verb):
//!
//! - R1: dirty spec_path AND temp issue working-copy mutation → accepted.
//! - R2: dirty spec_path + temp issue mutation + dirty unrelated file → rejected.
//! - R3: dirty spec_path only (issue file clean) → still accepted (regression
//!       guard for the previous single-path allowance).
//!
//! The companion file `inplace_mode_test.rs` is CODEGEN-managed; these
//! cases live here as hand-written until the codegen surface for the
//! gate widens to cover the multi-allowed assertions.

use agentic_workflow::issues::LocalBackend;
use std::process::Command;

fn skip_unless_ready() -> Option<(std::path::PathBuf, String)> {
    let git = agentic_workflow::git::find_git_bin()?;
    let bin = std::env::var("CARGO_BIN_EXE_aw").ok().or_else(|| {
        let exe = std::env::current_exe().ok()?;
        let debug_dir = exe.parent()?.parent()?;
        let bin = debug_dir.join(format!("aw{}", std::env::consts::EXE_SUFFIX));
        bin.exists().then(|| bin.display().to_string())
    })?;
    Some((git, bin))
}

fn run(git: &std::path::Path, root: &std::path::Path, args: &[&str]) {
    let status = Command::new(git)
        .arg("-C")
        .arg(root)
        .args(args)
        .status()
        .unwrap();
    assert!(status.success(), "git {:?} failed", args);
}

fn bootstrap_repo(git: &std::path::Path, root: &std::path::Path) {
    run(git, root, &["init", "-b", "main"]);
    run(git, root, &["config", "user.email", "t@t"]);
    run(git, root, &["config", "user.name", "t"]);
    run(git, root, &["config", "commit.gpgsign", "false"]);
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    run(git, root, &["add", "."]);
    run(git, root, &["commit", "-m", "seed"]);
}

fn issue_path(root: &std::path::Path, slug: &str) -> std::path::PathBuf {
    LocalBackend::from_project_root(root)
        .issues_dir()
        .join("open")
        .join(format!("{slug}.md"))
}

fn write_issue_fixture(root: &std::path::Path, slug: &str, body: impl AsRef<str>) {
    let path = issue_path(root, slug);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, body.as_ref()).unwrap();
}

fn write_fixture(root: &std::path::Path, slug: &str, spec_rel: &str) {
    std::fs::create_dir_all(root.join("projects/agentic-workflow/tech-design/surface/specs"))
        .unwrap();
    std::fs::create_dir_all(root.join(".aw")).unwrap();
    std::fs::write(root.join(".aw/config.toml"), "").unwrap();
    write_issue_fixture(
        root,
        slug,
        format!(
            "---\n\
             type: bug\n\
             title: dirty gate fixture\n\
             state: open\n\
             labels: [\"type:bug\", \"project:agentic-workflow\"]\n\
             phase: td_created\n\
             branch: td-{slug}\n\
             ---\n\n\
             ## Problem\n\nFixture for dirty-gate tests.\n"
        ),
    );
    std::fs::write(
        root.join(spec_rel),
        "---\n\
         id: dirty-gate-fixture\n\
         fill_sections: [logic, changes]\n\
         ---\n\n\
         # Dirty Gate Fixture\n\n\
         ## Logic\n\
         <!-- type: logic lang: mermaid -->\n\n\
         ```mermaid\n\
         ---\n\
         id: dirty-gate-fixture-logic\n\
         entry: start\n\
         nodes:\n\
           start:\n\
             kind: start\n\
             label: \"start\"\n\
           done:\n\
             kind: terminal\n\
             label: \"done\"\n\
         edges:\n\
           - from: start\n\
             to: done\n\
             label: ok\n\
         ---\n\
         flowchart TD\n\
             start([start]) --> done([done])\n\
         ```\n\n\
         ## Changes\n\
         <!-- type: changes lang: yaml -->\n\n\
         ```yaml\n\
         changes:\n\
           - path: projects/agentic-workflow/src/cli/td.rs\n\
             action: modify\n\
             impl_mode: hand-written\n\
             description: Exercise the multi-allowed dirty gate.\n\
         ```\n",
    )
    .unwrap();
}

fn append_needs_revision_review(root: &std::path::Path, spec_rel: &str) {
    let spec_path = root.join(spec_rel);
    let mut spec = std::fs::read_to_string(&spec_path).unwrap();
    spec.push_str(
        "\n# Reviews\n\n\
         ## Review 1\n\
         **Verdict:** needs-revision\n\n\
         - [logic] Trip the gate.\n",
    );
    std::fs::write(spec_path, spec).unwrap();
}

fn touch_dirty_issue(root: &std::path::Path, slug: &str) {
    let p = issue_path(root, slug);
    let body = std::fs::read_to_string(&p).unwrap();
    let mutated = body.replace("phase: td_created", "phase: td_reviewed");
    assert_ne!(body, mutated, "fixture must contain 'phase: td_created'");
    std::fs::write(p, mutated).unwrap();
}

/// R1 — dirty spec_path AND temp issue working-copy mutation → accepted.
#[test]
fn dirty_gate_accepts_spec_and_issue() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let slug = "dirty-gate-r1";
    let spec_rel = "projects/agentic-workflow/tech-design/surface/specs/dirty-gate-r1.md";
    write_fixture(root, slug, spec_rel);
    run(&git, root, &["add", "."]);
    run(&git, root, &["commit", "-m", "bootstrap"]);
    run(&git, root, &["switch", "-c", &format!("td-{}", slug)]);

    append_needs_revision_review(root, spec_rel);
    touch_dirty_issue(root, slug);

    let out = Command::new(&bin)
        .arg("td")
        .arg("review")
        .arg(slug)
        .arg("--apply")
        .arg("--spec-path")
        .arg(spec_rel)
        .current_dir(root)
        .output()
        .expect("run aw td review --apply");
    assert!(
        out.status.success(),
        "td review --apply must accept dirty spec plus temp issue working-copy mutation:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

/// R2 — dirty spec_path + temp issue mutation + dirty unrelated file → rejected.
#[test]
fn dirty_gate_rejects_third_dirty() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let slug = "dirty-gate-r2";
    let spec_rel = "projects/agentic-workflow/tech-design/surface/specs/dirty-gate-r2.md";
    write_fixture(root, slug, spec_rel);
    run(&git, root, &["add", "."]);
    run(&git, root, &["commit", "-m", "bootstrap"]);
    run(&git, root, &["switch", "-c", &format!("td-{}", slug)]);

    append_needs_revision_review(root, spec_rel);
    touch_dirty_issue(root, slug);
    std::fs::write(root.join("unrelated.txt"), "dirty\n").unwrap();

    let out = Command::new(&bin)
        .arg("td")
        .arg("review")
        .arg(slug)
        .arg("--apply")
        .arg("--spec-path")
        .arg(spec_rel)
        .current_dir(root)
        .output()
        .expect("run aw td review --apply");
    assert!(
        !out.status.success(),
        "td review --apply must reject when a third (unrelated) file is dirty"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        combined.contains("dirty outside allowed paths")
            || combined.contains("only the dirty spec path"),
        "expected dirty-gate rejection mentioning unrelated file; got:\n{}",
        combined,
    );
}

/// R3 — dirty spec_path only (issue file clean) → still accepted (regression).
#[test]
fn dirty_gate_accepts_spec_only() {
    let Some((git, bin)) = skip_unless_ready() else {
        eprintln!("skipping: git or CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    bootstrap_repo(&git, root);

    let slug = "dirty-gate-r3";
    let spec_rel = "projects/agentic-workflow/tech-design/surface/specs/dirty-gate-r3.md";
    write_fixture(root, slug, spec_rel);
    run(&git, root, &["add", "."]);
    run(&git, root, &["commit", "-m", "bootstrap"]);
    run(&git, root, &["switch", "-c", &format!("td-{}", slug)]);

    append_needs_revision_review(root, spec_rel);

    let out = Command::new(&bin)
        .arg("td")
        .arg("review")
        .arg(slug)
        .arg("--apply")
        .arg("--spec-path")
        .arg(spec_rel)
        .current_dir(root)
        .output()
        .expect("run aw td review --apply");
    assert!(
        out.status.success(),
        "td review --apply must still accept spec-only dirty (regression):\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests-cli-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
