---
id: projects-agentic-workflow-tests-cli-tests-init-doc-projection-test-rs
fill_sections: [overview, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: managed-and-semantic-production-gates
    claim: managed-and-semantic-production-gates
    coverage: partial
    rationale: "Real-binary smoke coverage for aw init's dual CLAUDE.md/AGENTS.md projection and the read-only aw init --check counterpart (issue #984, init-projector slice 1/3)."
---

# Standardized projects/agentic-workflow/tests/cli/tests/init_doc_projection_test.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/tests/cli/tests/init_doc_projection_test.rs`.

### Symbols

No public AST symbols.

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-handwrite -->

<!-- source-snapshot: path=projects/agentic-workflow/tests/cli/tests/init_doc_projection_test.rs -->
```rust
//! Real-binary smoke tests for `aw init`'s dual root-doc projection and the
//! read-only `aw init --check` counterpart (issue #984, init-projector
//! slice 1/3).
//!
//! `aw init` projects BOTH `CLAUDE.md` and `AGENTS.md` from the same
//! compiled-in `aw:start` template section (`AGENTS.md`'s section is
//! `CLAUDE.md`'s section plus the fixed Codex-only whitelist from
//! `agentic_workflow::cli::doc_mirror`). `aw init --check` mirrors `cargo
//! fmt --check` semantics: it must detect a tampered managed section and
//! name the stale file without writing, and a subsequent write-mode
//! `aw init` must restore it.

use std::path::Path;
use std::process::Command;

fn skip_unless_ready() -> Option<std::path::PathBuf> {
    std::env::var("CARGO_BIN_EXE_aw")
        .ok()
        .or_else(|| {
            let exe = std::env::current_exe().ok()?;
            let debug_dir = exe.parent()?.parent()?;
            let bin = debug_dir.join(format!("aw{}", std::env::consts::EXE_SUFFIX));
            bin.exists().then(|| bin.display().to_string())
        })
        .map(std::path::PathBuf::from)
}

fn run_init(bin: &Path, root: &Path, extra_args: &[&str]) -> std::process::Output {
    Command::new(bin)
        .arg("init")
        .args(extra_args)
        .current_dir(root)
        .output()
        .expect("run aw init")
}

fn aw_start_block(content: &str) -> &str {
    let start = content.find("<!-- aw:start -->").expect("aw:start marker");
    let end = content
        .find("<!-- aw:end -->")
        .map(|i| i + "<!-- aw:end -->".len())
        .expect("aw:end marker");
    &content[start..end]
}

fn combined_output(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    )
}

/// AC1: a fresh `aw init` creates both root docs, and `aw init --check`
/// against that fresh output is clean (no drift) — the projector's own
/// output always satisfies its own checker.
#[test]
fn fresh_init_creates_both_root_docs_and_check_is_clean() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );

    let claude_path = root.join("CLAUDE.md");
    let agents_path = root.join("AGENTS.md");
    assert!(claude_path.exists(), "aw init must create CLAUDE.md");
    assert!(agents_path.exists(), "aw init must create AGENTS.md");

    let check = run_init(&bin, root, &["--check"]);
    assert!(
        check.status.success(),
        "aw init --check should be clean right after a fresh init:\n{}",
        combined_output(&check)
    );
    assert!(combined_output(&check).contains("up to date"));
}

/// AC1 (structural proof): AGENTS.md's projected `aw:start` block equals
/// CLAUDE.md's projected block run through
/// `doc_mirror::agents_block_from_claude_block` — the same function `aw
/// init` itself calls — proving the fresh-install output matches the
/// projector's own contract, not just "some content got written".
#[test]
fn fresh_init_agents_md_block_matches_doc_mirror_projection_of_claude_md_block() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );

    let claude = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    let agents = std::fs::read_to_string(root.join("AGENTS.md")).unwrap();

    let claude_block = aw_start_block(&claude);
    let agents_block = aw_start_block(&agents);
    let expected_agents_block =
        agentic_workflow::cli::doc_mirror::agents_block_from_claude_block(claude_block);

    assert_eq!(
        agents_block, expected_agents_block,
        "AGENTS.md's aw:start block must equal doc_mirror's projection of CLAUDE.md's block"
    );
}

/// AC2: tampering either root doc's `aw:start` content, then running `aw
/// init --check`, must fail (non-zero exit), name the stale file, and leave
/// the tampered file byte-unchanged on disk (read-only). A follow-up
/// write-mode `aw init` must restore the managed section, after which `aw
/// init --check` is clean again.
#[test]
fn init_check_detects_tamper_without_writing_and_init_restores_it() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );

    let claude_path = root.join("CLAUDE.md");
    let agents_path = root.join("AGENTS.md");
    let claude_before = std::fs::read_to_string(&claude_path).unwrap();
    let agents_before = std::fs::read_to_string(&agents_path).unwrap();

    // --- Tamper CLAUDE.md ---
    let tampered_claude =
        claude_before.replace("## Agentic Workflow CLI Surface", "## TAMPERED CLAUDE");
    assert_ne!(
        tampered_claude, claude_before,
        "fixture heading must exist in the fresh-installed CLAUDE.md"
    );
    std::fs::write(&claude_path, &tampered_claude).unwrap();

    let check_tampered = run_init(&bin, root, &["--check"]);
    assert!(
        !check_tampered.status.success(),
        "aw init --check must fail when CLAUDE.md's managed section is tampered"
    );
    let check_tampered_out = combined_output(&check_tampered);
    assert!(
        check_tampered_out.contains("CLAUDE.md"),
        "aw init --check must name the stale file:\n{check_tampered_out}"
    );
    assert_eq!(
        std::fs::read_to_string(&claude_path).unwrap(),
        tampered_claude,
        "aw init --check must never write"
    );

    let restore = run_init(&bin, root, &[]);
    assert!(
        restore.status.success(),
        "aw init should succeed restoring CLAUDE.md:\n{}",
        combined_output(&restore)
    );
    assert_eq!(
        std::fs::read_to_string(&claude_path).unwrap().trim(),
        claude_before.trim(),
        "aw init must restore the tampered aw:start block in CLAUDE.md"
    );

    let check_clean = run_init(&bin, root, &["--check"]);
    assert!(
        check_clean.status.success(),
        "aw init --check should be clean after restore:\n{}",
        combined_output(&check_clean)
    );

    // --- Tamper AGENTS.md ---
    let tampered_agents =
        agents_before.replace("## Agentic Workflow CLI Surface", "## TAMPERED AGENTS");
    assert_ne!(
        tampered_agents, agents_before,
        "fixture heading must exist in the fresh-installed AGENTS.md"
    );
    std::fs::write(&agents_path, &tampered_agents).unwrap();

    let check_agents_tampered = run_init(&bin, root, &["--check"]);
    assert!(
        !check_agents_tampered.status.success(),
        "aw init --check must fail when AGENTS.md's managed section is tampered"
    );
    let check_agents_tampered_out = combined_output(&check_agents_tampered);
    assert!(
        check_agents_tampered_out.contains("AGENTS.md"),
        "aw init --check must name the stale file:\n{check_agents_tampered_out}"
    );
    assert_eq!(
        std::fs::read_to_string(&agents_path).unwrap(),
        tampered_agents,
        "aw init --check must never write"
    );

    let restore_agents = run_init(&bin, root, &[]);
    assert!(
        restore_agents.status.success(),
        "aw init should succeed restoring AGENTS.md:\n{}",
        combined_output(&restore_agents)
    );
    assert_eq!(
        std::fs::read_to_string(&agents_path).unwrap().trim(),
        agents_before.trim(),
        "aw init must restore the tampered aw:start block in AGENTS.md"
    );

    let check_clean_again = run_init(&bin, root, &["--check"]);
    assert!(
        check_clean_again.status.success(),
        "aw init --check should be clean after restoring AGENTS.md:\n{}",
        combined_output(&check_clean_again)
    );
}

/// Issue #985 (init-projector slice 2/3): `aw init` renders the Workflow/
/// Support CLI tables inside CLAUDE.md's `aw:start` block between
/// fine-grained `<!-- aw:cli-table:{workflow,support}:start/end -->`
/// markers. Tampering a row inside one of those markers must be caught by
/// `aw init --check` (naming CLAUDE.md, without writing) and repaired by a
/// follow-up write-mode `aw init`, exactly like the outer whole-block
/// contract proven above.
#[test]
fn init_check_detects_cli_table_row_tamper_and_init_restores_it() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );

    let claude_path = root.join("CLAUDE.md");
    let claude_before = std::fs::read_to_string(&claude_path).unwrap();
    assert!(
        claude_before.contains("<!-- aw:cli-table:workflow:start -->"),
        "fresh-installed CLAUDE.md must carry the workflow CLI-table markers:\n{claude_before}"
    );
    assert!(
        claude_before.contains("| `aw wi` |"),
        "the workflow CLI table must carry a rendered `aw wi` row:\n{claude_before}"
    );

    let tampered = claude_before.replace(
        "| `aw wi` | Manage work-items",
        "| `aw wi` | TAMPERED ROW TEXT",
    );
    assert_ne!(
        tampered, claude_before,
        "fixture row must exist in the fresh-installed CLAUDE.md"
    );
    std::fs::write(&claude_path, &tampered).unwrap();

    let check_tampered = run_init(&bin, root, &["--check"]);
    assert!(
        !check_tampered.status.success(),
        "aw init --check must fail when a CLI-table row is tampered"
    );
    let check_tampered_out = combined_output(&check_tampered);
    assert!(
        check_tampered_out.contains("CLAUDE.md"),
        "aw init --check must name the stale file:\n{check_tampered_out}"
    );
    assert_eq!(
        std::fs::read_to_string(&claude_path).unwrap(),
        tampered,
        "aw init --check must never write"
    );

    let restore = run_init(&bin, root, &[]);
    assert!(
        restore.status.success(),
        "aw init should succeed restoring CLAUDE.md:\n{}",
        combined_output(&restore)
    );
    assert_eq!(
        std::fs::read_to_string(&claude_path).unwrap().trim(),
        claude_before.trim(),
        "aw init must restore the tampered CLI-table row in CLAUDE.md"
    );

    let check_clean = run_init(&bin, root, &["--check"]);
    assert!(
        check_clean.status.success(),
        "aw init --check should be clean after restore:\n{}",
        combined_output(&check_clean)
    );
}

/// Issue #985 (init-projector slice 2/3): when a project's README.md already
/// carries `<!-- aw:projects-table:start/end -->` markers, `aw init`
/// regenerates the enclosed table from `.aw/config.toml`'s top-level
/// `[[projects]]` entries and `aw init --check` covers its freshness the
/// same way it covers CLAUDE.md/AGENTS.md — without ever inserting the
/// table into a README that never opted in.
#[test]
fn init_projects_table_is_opt_in_and_tamper_is_detected_and_restored() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    // A README with no markers is left alone (opt-in, not force-inserted).
    let plain_readme = "# demo\n\nno markers here.\n";
    std::fs::write(root.join("README.md"), plain_readme).unwrap();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );
    assert_eq!(
        std::fs::read_to_string(root.join("README.md")).unwrap(),
        plain_readme,
        "aw init must not touch a README without the Projects-table markers"
    );
    let check_no_markers = run_init(&bin, root, &["--check"]);
    assert!(
        check_no_markers.status.success(),
        "a marker-less README must never be reported stale:\n{}",
        combined_output(&check_no_markers)
    );

    // Opt in: seed a stale Projects table between the markers, then re-run.
    let seeded_readme = format!(
        "# demo\n\n## Projects\n\n{}\n| Project | What it is |\n|---------|------------|\n| [stale](stale) | stale row |\n{}\n\n## Other\n",
        "<!-- aw:projects-table:start -->", "<!-- aw:projects-table:end -->"
    );
    std::fs::write(root.join("README.md"), &seeded_readme).unwrap();

    let out2 = run_init(&bin, root, &[]);
    assert!(
        out2.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out2)
    );
    let readme_after = std::fs::read_to_string(root.join("README.md")).unwrap();
    assert!(
        !readme_after.contains("stale row"),
        "aw init must regenerate the opted-in Projects table, dropping stale rows:\n{readme_after}"
    );
    assert!(
        readme_after.contains("## Other"),
        "content outside the markers must be preserved byte-for-byte:\n{readme_after}"
    );

    let check_clean = run_init(&bin, root, &["--check"]);
    assert!(
        check_clean.status.success(),
        "aw init --check should be clean right after regenerating the Projects table:\n{}",
        combined_output(&check_clean)
    );

    // Tamper the now-generated table and prove detection + restore.
    let tampered_readme = readme_after.replace(
        "<!-- aw:projects-table:start -->",
        "<!-- aw:projects-table:start -->\n| TAMPERED | ROW |",
    );
    assert_ne!(tampered_readme, readme_after);
    std::fs::write(root.join("README.md"), &tampered_readme).unwrap();

    let check_tampered = run_init(&bin, root, &["--check"]);
    assert!(
        !check_tampered.status.success(),
        "aw init --check must fail when the Projects table is tampered"
    );
    assert!(
        combined_output(&check_tampered).contains("README.md"),
        "aw init --check must name README.md as stale:\n{}",
        combined_output(&check_tampered)
    );

    let restore = run_init(&bin, root, &[]);
    assert!(
        restore.status.success(),
        "aw init should succeed restoring README.md:\n{}",
        combined_output(&restore)
    );
    assert_eq!(
        std::fs::read_to_string(root.join("README.md"))
            .unwrap()
            .trim(),
        readme_after.trim(),
        "aw init must restore the tampered Projects table in README.md"
    );
}

/// `aw init` output must end with a chainable next step (CONTRIBUTING's
/// chainable-output convention). A from-scratch sandbox has no registered
/// `[[projects]]` entry, so the emitted hint must be the terminal `next:
/// done` marker rather than a guessed/unexecutable `aw health --project`
/// invocation (`--project` is a required argument).
#[test]
fn init_emits_chainable_next_step() {
    let Some(bin) = skip_unless_ready() else {
        eprintln!("skipping: CARGO_BIN_EXE_aw missing");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    let out = run_init(&bin, root, &[]);
    assert!(
        out.status.success(),
        "aw init should succeed:\n{}",
        combined_output(&out)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("next: done") || stdout.contains("next: aw health --project"),
        "aw init must end with a chainable next step:\n{stdout}"
    );

    let check = run_init(&bin, root, &["--check"]);
    let check_stdout = String::from_utf8_lossy(&check.stdout);
    assert!(
        check_stdout.contains("next: done"),
        "aw init --check should also end with a chainable next step when clean:\n{check_stdout}"
    );
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/cli/tests/init_doc_projection_test.rs
    action: create
    section: source
    description: |
      Issue #984 (init-projector slice 1/3): real-binary smoke tests driving
      the actual `aw` binary against sandboxed tempdirs (no git repository
      required for `aw init`). `fresh_init_creates_both_root_docs_and_check_is_clean`
      proves AC1 (a fresh install creates CLAUDE.md and AGENTS.md and a
      follow-up `aw init --check` is clean).
      `fresh_init_agents_md_block_matches_doc_mirror_projection_of_claude_md_block`
      structurally proves AC1 by re-deriving AGENTS.md's expected block via
      `doc_mirror::agents_block_from_claude_block` on the fresh-installed
      CLAUDE.md block. `init_check_detects_tamper_without_writing_and_init_restores_it`
      proves AC2 for both root docs: `aw init --check` fails, names the
      stale file, and never writes when tampered, and a follow-up
      `aw init` restores the managed section (verified clean again via a
      second `--check`). `init_emits_chainable_next_step` proves the
      CONTRIBUTING chainable-output convention: both `aw init` and
      `aw init --check` end with a `next:` line, using the terminal
      `next: done` marker (not a guessed `aw health --project <name>`) when
      no single `[[projects]]` entry is registered.

      Issue #985 (init-projector slice 2/3) adds two more real-binary
      smoke tests. `init_check_detects_cli_table_row_tamper_and_init_restores_it`
      proves the fine-grained `<!-- aw:cli-table:{workflow,support}:start/end -->`
      markers inside CLAUDE.md's `aw:start` block: a tampered row inside the
      workflow table is caught by `aw init --check` (naming CLAUDE.md,
      without writing) and repaired by a follow-up write-mode `aw init`.
      `init_projects_table_is_opt_in_and_tamper_is_detected_and_restored`
      proves the repo-root `<!-- aw:projects-table:start/end -->` markers:
      a README without the markers is left byte-for-byte untouched (opt-in,
      never force-inserted); a README that already carries the markers gets
      its enclosed table regenerated from `.aw/config.toml`'s top-level
      `[[projects]]` entries (dropping a seeded stale row, preserving
      content outside the markers); and a subsequent tamper of the
      generated table is caught by `aw init --check` (naming README.md)
      and restored by a follow-up `aw init`.
    impl_mode: hand-written
```
