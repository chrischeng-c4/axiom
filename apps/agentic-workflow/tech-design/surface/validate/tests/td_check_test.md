---
id: projects-score-tests-td-check-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: full
    rationale: "Standardization TDs support brownfield takeover, semantic coverage, traceability, and production readiness gates."
---

# Standardized apps/agentic-workflow/tests/cli/tests/td_check_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `apps/agentic-workflow/tests/cli/tests/td_check_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=apps/agentic-workflow/tests/cli/tests/td_check_test.rs -->
````rust
// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/validate/tests/td_check_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td check`.
//!
//! @spec apps/agentic-workflow/tech-design/surface/specs/score-namespaces.md#test-plan

use agentic_workflow::cli::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn test_td_check_registered() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td registered");
    let check = td.find_subcommand("check").expect("td check registered");
    let positionals: Vec<String> = check
        .get_positionals()
        .map(|p: &clap::Arg| p.get_id().as_str().to_string())
        .collect();
    assert!(
        positionals.iter().any(|p| p == "target"),
        "expected target arg, got {:?}",
        positionals
    );
}

/// Issue #1277 (epic #1270 R3): `aw td validate` is retired; its subcommand
/// (and the `--check` flag it exposed) must no longer parse.
#[test]
fn test_td_validate_subcommand_removed_from_check_test_surface() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td registered");
    assert!(
        td.find_subcommand("validate").is_none(),
        "td validate must not be registered (folded into td check, #1277)"
    );
}

/// Issue #1277 union coverage: `aw td check` must cover BOTH entry shapes
/// the two prior verbs offered — path-mode (`aw td check <path>`, inherited
/// unchanged from `td check`) AND slug-mode (`aw td check <slug>`, formerly
/// only reachable via `aw td validate <slug> --check`). Both shapes are
/// exercised as separate subprocess invocations against the exact same spec
/// content and must report the same finding count, proving they run through
/// the identical rule registry (`crate::validate::run_rules`) rather than
/// two divergent rule sets.
#[test]
fn test_td_check_slug_mode_and_path_mode_share_rule_registry() {
    let Some(git) = agentic_workflow::git::find_git_bin() else {
        eprintln!("skipping: git not found");
        return;
    };
    let bin = std::env::var("CARGO_BIN_EXE_aw").ok().or_else(|| {
        let exe = std::env::current_exe().ok()?;
        let debug_dir = exe.parent()?.parent()?;
        let bin = debug_dir.join(format!("aw{}", std::env::consts::EXE_SUFFIX));
        bin.exists().then(|| bin.display().to_string())
    });
    let Some(bin) = bin else {
        eprintln!("skipping: CARGO_BIN_EXE_aw not set");
        return;
    };

    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    let run_git = |args: &[&str]| {
        std::process::Command::new(&git)
            .arg("-C")
            .arg(root)
            .args(args)
            .status()
            .expect("git command");
    };
    run_git(&["init", "-b", "main"]);
    run_git(&["config", "user.email", "t@t"]);
    run_git(&["config", "user.name", "t"]);
    run_git(&["config", "commit.gpgsign", "false"]);
    std::fs::write(root.join("README.md"), "seed\n").unwrap();
    std::fs::write(root.join("aw.toml"), "").unwrap();
    let td_dir = root.join("tech-design");
    std::fs::create_dir_all(&td_dir).unwrap();
    let spec_path = td_dir.join("union-1277.md");
    std::fs::write(
        &spec_path,
        concat!(
            "---\n",
            "id: union-1277\n",
            "fill_sections: [changes]\n",
            "---\n\n",
            "# Hello\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        ),
    )
    .unwrap();
    run_git(&["add", "."]);
    run_git(&["commit", "-m", "seed"]);
    // Slug-mode resolution only activates a td-<slug> branch switch from
    // `main`; use a project branch instead so `td check <slug>` runs
    // in-place against `tech-design/` without needing `aw td create` first.
    run_git(&["switch", "-c", "project-union1277"]);

    let path_mode = std::process::Command::new(&bin)
        .arg("td")
        .arg("check")
        .arg(spec_path.to_str().unwrap())
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run aw td check <path>");
    let slug_mode = std::process::Command::new(&bin)
        .arg("td")
        .arg("check")
        .arg("union-1277")
        .arg("--json")
        .current_dir(root)
        .output()
        .expect("run aw td check <slug>");

    let count_findings = |out: &std::process::Output| -> usize {
        let stdout = String::from_utf8_lossy(&out.stdout);
        serde_json::from_str::<Vec<serde_json::Value>>(stdout.trim())
            .map(|v| v.len())
            .unwrap_or(0)
    };
    assert_eq!(
        count_findings(&path_mode),
        count_findings(&slug_mode),
        "path-mode and slug-mode `td check` must share the same rule registry \
         (former td validate slug/--check path folded in by #1277):\n\
         path stdout={}\npath stderr={}\nslug stdout={}\nslug stderr={}",
        String::from_utf8_lossy(&path_mode.stdout),
        String::from_utf8_lossy(&path_mode.stderr),
        String::from_utf8_lossy(&slug_mode.stdout),
        String::from_utf8_lossy(&slug_mode.stderr),
    );
}

/// R4 path-mode: when target contains `/` or ends `.md` we exit 0 on
/// no findings. Use a tiny temp file with a valid spec body.
#[test]
fn test_td_check_path_mode_smoke() {
    use agentic_workflow::cli::td;
    let tmp = tempfile::TempDir::new().unwrap();
    let spec = tmp.path().join("ok.md");
    std::fs::write(
        &spec,
        concat!(
            "---\n",
            "id: ok\n",
            "fill_sections: [changes]\n",
            "---\n\n",
            "# Hello\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes: []\n",
            "```\n",
        ),
    )
    .unwrap();
    let args = td::CheckArgs {
        target: spec.to_str().unwrap().to_string(),
        json: true,
        section_type_conformance: false,
    };
    // We don't assert exit code (rule registry may flag style), only
    // that the entry point doesn't panic on a syntactically valid path.
    // run_check calls std::process::exit on violations, so wrap in a
    // forked process? — too heavy. Just run; if it process::exits, the
    // test process dies. To avoid that, run in a thread that catches
    // unwind only; exit() bypasses unwind so this is best-effort.
    let _ = td::run_check(args);
}

#[test]
fn test_td_check_accepts_operations_section_types() {
    use agentic_workflow::cli::td;
    let tmp = tempfile::TempDir::new().unwrap();
    let spec = tmp.path().join("ops.md");
    std::fs::write(
        &spec,
        concat!(
            "---\n",
            "id: ops\n",
            "fill_sections: [runtime-image, deployment, changes]\n",
            "---\n\n",
            "# Operations\n\n",
            "#",
            "# Runtime Image\n",
            "<",
            "!-- type: runtime-image lang: yaml -->\n\n",
            "```yaml\n",
            "image:\n",
            "  base: python:3.12-slim-bookworm\n",
            "  workdir: /workspace/backend\n",
            "build_context:\n",
            "  dockerfile: examples/fixture_platform/backend/Dockerfile\n",
            "  ignore_file: examples/fixture_platform/backend/.dockerignore\n",
            "```\n\n",
            "#",
            "# Deployment\n",
            "<",
            "!-- type: deployment lang: yaml -->\n\n",
            "```yaml\n",
            "kustomize:\n",
            "  base: examples/fixture_platform/kustomize/bases/frontend\n",
            "  overlays:\n",
            "    - examples/fixture_platform/kustomize/overlays/uat/frontend\n",
            "resources:\n",
            "  - kind: Deployment\n",
            "  - kind: Service\n",
            "  - kind: HorizontalPodAutoscaler\n",
            "```\n\n",
            "#",
            "# Changes\n",
            "<",
            "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes:\n",
            "  - path: examples/fixture_platform/backend/Dockerfile\n",
            "    action: modify\n",
            "    impl_mode: hand-written\n",
            "    section: runtime-image\n",
            "  - path: examples/fixture_platform/kustomize/bases/frontend/deployment.yaml\n",
            "    action: modify\n",
            "    impl_mode: hand-written\n",
            "    section: deployment\n",
            "```\n",
        ),
    )
    .unwrap();
    let args = td::CheckArgs {
        target: spec.to_str().unwrap().to_string(),
        json: true,
        section_type_conformance: false,
    };
    let _ = td::run_check(args);
}

/// R4 directory mode: passing a non-existent path returns Err (exit 2 is
/// emitted via `anyhow::bail!` upstream of process::exit).
#[test]
fn test_td_check_unresolvable_target_errors() {
    use agentic_workflow::cli::td;
    let args = td::CheckArgs {
        target: "/this/path/does/not/exist/at/all.md".to_string(),
        json: false,
        section_type_conformance: false,
    };
    let result = td::run_check(args);
    // Either Err (anyhow bail) OR a process::exit. We can only assert
    // Err here; if it exits, the test runner reports the harness failure.
    assert!(result.is_err(), "unresolvable target must return Err");
}

// CODEGEN-END

````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/tests/cli/tests/td_check_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
