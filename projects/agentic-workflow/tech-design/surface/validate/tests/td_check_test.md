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

# Standardized projects/agentic-workflow/tests/cli/tests/td_check_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/td_check_test.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/td_check_test.rs -->
````rust
// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/td_check_test.md#source
// CODEGEN-BEGIN
//! Integration tests for `aw td check`.
//!
//! @spec projects/agentic-workflow/tech-design/surface/specs/score-namespaces.md#test-plan

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

/// R5: `aw td validate` no longer surfaces `--check` in help (it's
/// hidden / deprecated) but the flag still parses for backward
/// compatibility.
#[test]
fn test_td_validate_check_flag_hidden() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td registered");
    let validate = td
        .find_subcommand("validate")
        .expect("td validate registered");
    let check_flag = validate
        .get_arguments()
        .find(|a: &&clap::Arg| a.get_id().as_str() == "check");
    let flag = check_flag.expect("--check still parses for compat");
    assert!(flag.is_hide_set(), "--check must be hidden in Phase 1");
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
            "#", "# Changes\n",
            "<", "!-- type: changes lang: yaml -->\n\n",
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
            "#", "# Runtime Image\n",
            "<", "!-- type: runtime-image lang: yaml -->\n\n",
            "```yaml\n",
            "image:\n",
            "  base: python:3.12-slim-bookworm\n",
            "  workdir: /workspace/backend\n",
            "build_context:\n",
            "  dockerfile: examples/fixture_platform/backend/Dockerfile\n",
            "  ignore_file: examples/fixture_platform/backend/.dockerignore\n",
            "```\n\n",
            "#", "# Deployment\n",
            "<", "!-- type: deployment lang: yaml -->\n\n",
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
            "#", "# Changes\n",
            "<", "!-- type: changes lang: yaml -->\n\n",
            "```yaml\n",
            "changes:\n",
            "  - path: examples/fixture_platform/backend/Dockerfile\n",
            "    action: modify\n",
            "    section: runtime-image\n",
            "  - path: examples/fixture_platform/kustomize/bases/frontend/deployment.yaml\n",
            "    action: modify\n",
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
  - path: projects/agentic-workflow/tests/cli/tests/td_check_test.rs
    action: modify
    impl_mode: codegen
    section: source
    description: |
      Whole-file source template generated from the standardized target body.
```
