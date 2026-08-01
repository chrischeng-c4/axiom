"""Black-box contract for the end-to-end Python artifact goal flow (#3298)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, git_commit_fixture, project_fixture, run_aw


CASE_ID = "workflow-root-runner-python-artifact-goal-flow"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "python-artifact-goal-flow"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-python-artifact-goal-flow"
)
ASSERTIONS = (
    "a freshly validated work item's live aw goal wi next.command starts EC-first (a real aw ec check invocation), and relabeling that same real, on-disk work item through ec_checked then ec_reviewed makes the identical live surface progress to a real aw ec review then a real aw td check invocation, tracing the direct-EC-check-through-TD-compilation half of the progression",
    "continuing the same relabeling sequence through td_compiled, ec_td_green, and both real spellings of the generated-target phase (cb_generated and cb_genned) makes the identical live surface progress through a real aw ec verify --stage td, a real aw cb gen --target rust invocation, and a real aw cb fill invocation for each spelling, tracing the TD-verification-through-target-generation half of the progression",
    "continuing the same sequence through cb_filled, cb_checked, and ec_cb_green makes the identical live surface progress through a real aw cb check, a real aw ec verify --stage cb, and finally a real aw wi close --push invocation, tracing the CB-checks-through-terminal-EC-verification tail of the progression so every stage the claim names is a live, reachable command in one connected table",
)


def _change_body() -> str:
    return (
        "## Problem\n\nTrace the full EC-first Python artifact progression end to end.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing phase-routing table\n"
        "Progress Evidence: the public goal wi envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace the full Python artifact goal flow.\n\n"
        "## Scope\n\n### In Scope\n- trace the full EC-first Python progression.\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: every named stage is a live, reachable command.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| goal-flow-trace | update | complete-platform.md |\n"
    )


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str, state: str) -> Path:
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues" / state / f"{slug}.md"


def _relabel_phase(root: Path, slug: str, new_phase: str) -> None:
    path = _issue_path(root, slug, "open")
    assert path.is_file(), path
    original = path.read_text(encoding="utf-8")

    field_pattern = re.compile(r"(?m)^phase: .*$")
    assert field_pattern.search(original), original
    updated = field_pattern.sub(f"phase: {new_phase}", original, count=1)
    assert updated != original, original

    label_pattern = re.compile(r"(?m)^- phase:.*$")
    assert label_pattern.search(updated), updated
    updated = label_pattern.sub(f"- phase:{new_phase}", updated, count=1)

    assert f"phase: {new_phase}" in updated, updated
    assert f"- phase:{new_phase}" in updated, updated
    path.write_text(updated, encoding="utf-8")


def _next_command(root: Path, slug: str) -> str:
    envelope = final_json(run_aw(root, "goal", "wi", slug))
    return str(envelope["next"]["command"])


def _write_minimal_td_module(root: Path) -> None:
    # Target generation (`ec_td_green` onward) canonicalizes the Python TD
    # source root and lowers its modules for the Rust target, so it needs at
    # least one real, minimally declared module -- the same fixture shape
    # `apps/agentic-workflow/src/cli/run.rs`'s own `python_project_root_with_target`
    # test helper uses (a module id plus one non-empty class declaration).
    src_dir = root / "tech-design" / "src" / "demo"
    src_dir.mkdir(parents=True, exist_ok=True)
    (src_dir / "policy.py").write_text(
        '__aw_artifact_id__ = "artifact:policy/evaluate"\n\nclass Policy:\n    pass\n',
        encoding="utf-8",
    )


def verify() -> list[str]:
    with project_fixture() as root:
        _write_minimal_td_module(root)
        created = create(root, "Trace Python artifact goal flow", "change", "--body", _change_body())
        slug = created["slug"]
        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated
        # Target generation resolves native ownership through `git log`.
        git_commit_fixture(root)

        # Stage 1: a fresh work item is EC-first.
        command = _next_command(root, slug)
        assert command.startswith(f"aw ec check --project demo --wi {slug}"), command

        # Stage 2: EC checked -> EC review.
        _relabel_phase(root, slug, "ec_checked")
        command = _next_command(root, slug)
        assert command.startswith(f"aw ec review --project demo --wi {slug}"), command

        # Stage 3: EC reviewed -> TD compilation begins.
        _relabel_phase(root, slug, "ec_reviewed")
        command = _next_command(root, slug)
        assert command.startswith("aw td check "), command
        assert command.endswith(f"--project demo --wi {slug}"), command

        # Stage 4: TD compiled -> EC/TD verification.
        _relabel_phase(root, slug, "td_compiled")
        command = _next_command(root, slug)
        assert command.startswith(
            f"aw ec verify --project demo --required-only --stage td --wi {slug}"
        ), command

        # Stage 5: EC/TD green -> target generation begins.
        _relabel_phase(root, slug, "ec_td_green")
        command = _next_command(root, slug)
        assert command.startswith("aw cb gen --target rust --source-root"), command
        assert command.endswith(f"--project demo --wi {slug}"), command

        # Stage 6: generated target -> fill, for both real phase spellings.
        for generated_phase in ("cb_generated", "cb_genned"):
            _relabel_phase(root, slug, generated_phase)
            command = _next_command(root, slug)
            assert command.startswith(f"aw cb fill {slug}"), (generated_phase, command)

        # Stage 7: filled -> CB check.
        _relabel_phase(root, slug, "cb_filled")
        command = _next_command(root, slug)
        assert command.startswith(f"aw cb check {slug}"), command

        # Stage 8: CB checked -> EC/CB (terminal) verification.
        _relabel_phase(root, slug, "cb_checked")
        command = _next_command(root, slug)
        assert command.startswith(
            f"aw ec verify --project demo --required-only --stage cb --wi {slug}"
        ), command

        # Stage 9: EC/CB green -> terminal close.
        _relabel_phase(root, slug, "ec_cb_green")
        command = _next_command(root, slug)
        assert command.startswith(f"aw wi close {slug} --push"), command

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
