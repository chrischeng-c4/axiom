//! Governance lock for #1102: the held-WIP braid landing plan must stay
//! explicit, durable, and machine-checkable before the main thread lands it.

use std::path::PathBuf;

fn project_root() -> PathBuf {
    crate::common::project_root()
}

fn landing_doc() -> PathBuf {
    project_root().join("docs/landing/braid-held-wip-landing-plan.md")
}

fn load_doc() -> String {
    std::fs::read_to_string(landing_doc()).expect("read #1102 braid landing plan")
}

#[test]
fn braid_landing_plan_doc_exists() {
    assert!(landing_doc().is_file(), "#1102 landing plan doc must exist");
}

#[test]
fn braid_landing_plan_keeps_hold_invariants() {
    let doc = load_doc();
    for needle in [
        "# Braid Held-WIP Landing Plan",
        "Issue: #1102",
        "## HOLD Invariants",
        "HOLD: never `git add -A`.",
        "Do not clean up, revert, or restack unrelated user-owned WIP.",
        "Preserve the user's dirty state exactly as found",
        "Do not strand a dependency where a committed file references an uncommitted symbol",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1102 HOLD invariant: {needle}"
        );
    }
}

#[test]
fn braid_landing_plan_keeps_preflight_and_surgical_commands() {
    let doc = load_doc();
    for needle in [
        "## Preflight Inventory",
        "git status --short",
        "git diff --name-only",
        "gh issue view 897 --comments",
        "gh issue view 953 --comments",
        "## Surgical Staging Recipe",
        "git diff > /tmp/1102-full.diff",
        "git apply --cached /tmp/1102-filtered.patch",
        "git diff --cached --stat",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1102 command or recipe marker: {needle}"
        );
    }
}

#[test]
fn braid_landing_plan_keeps_lockstep_issue_refs_and_artifacts() {
    let doc = load_doc();
    for needle in [
        "## Lockstep Landing Checklist",
        "`lower:: walrus=WIP-owned`",
        "`lower:: mutated-defaults=#897`",
        "`lower:: raw-int-param-ordering=#953`",
        "`VENDORED_MODULES` + no-op registers + `py_src` files + curated",
        "`#953/#976/#977/#1014/#1015`",
        "#897",
        "#953",
        "#976",
        "#977",
        "#1014",
        "#1015",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1102 lockstep marker: {needle}"
        );
    }
}

#[test]
fn braid_landing_plan_keeps_committed_state_and_acceptance_markers() {
    let doc = load_doc();
    for needle in [
        "## Committed-State Verification",
        "git worktree add /tmp/mamba-1102-verify <commit-sha>",
        "cargo build -p mamba",
        "cargo test -p mamba lower:: -- --nocapture",
        "## Post-Landing Gates",
        "## Abort / Rollback",
        "git restore --staged <path>",
        "## Acceptance Criteria",
        "AC-1: committed state builds clean in a fresh worktree.",
        "AC-2: baseline post-landing conformance sweep is back to expected state.",
        "AC-3: the 3 lockstep `lower::` tests are green together.",
    ] {
        assert!(
            doc.contains(needle),
            "missing #1102 verification or acceptance marker: {needle}"
        );
    }
}
