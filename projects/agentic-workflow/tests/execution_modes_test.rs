// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/validate/execution_modes_test.md#tests
// CODEGEN-BEGIN

//! Integration tests for retired legacy agent execution hooks
//!
//! Covers:
//! - R5: legacy Claude agent bash hooks are retired
//! - R6: stale mainthread Claude hooks are retired
//!
//! R4 (multi_claude_agents agent frontmatter) tests were retired when
//! Phase 2 mainthread-only execution deleted `.claude/agents/score-*.md`.
//! The mainthread now drives every `--apply` step directly; there are no
//! `score-change-*` / `score-review` agent definitions to validate.

use std::path::PathBuf;

/// Project root (2 levels up from the crate directory)
fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn test_legacy_agent_pretooluse_hooks_are_not_required() {
    // R5: retired Claude agent hook paths must not be required by current SDD tests.
    let root = project_root();
    assert!(
        !root
            .join(".claude/hooks/agents/_shared/pretooluse-readonly-bash.sh")
            .exists(),
        "legacy readonly agent hook should stay retired"
    );
    assert!(
        !root
            .join(".claude/hooks/agents/_shared/pretooluse-safe-bash.sh")
            .exists(),
        "legacy safe agent hook should stay retired"
    );
}

#[test]
fn test_mainthread_hooks_are_not_required() {
    // R6: stale Claude framework hooks should not be required by current AW execution.
    let root = project_root();
    for hook in [
        ".claude/hooks/hook1-post-apply-validate.sh",
        ".claude/hooks/hook2-pre-apply-guard.sh",
        ".claude/hooks/hook5-session-start-idle.sh",
    ] {
        assert!(!root.join(hook).exists(), "{hook} should stay retired");
    }
}
// CODEGEN-END
// SPEC-MANAGED: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// CODEGEN-BEGIN
// SPEC-REF: projects/agentic-workflow/tech-design/semantic/agentic-workflow-tests.md#schema
// TODO: Existing source behavior is covered by this feature/domain semantic TD.

// CODEGEN-END
