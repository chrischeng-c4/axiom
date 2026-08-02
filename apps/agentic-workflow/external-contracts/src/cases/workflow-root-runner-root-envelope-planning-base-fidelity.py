"""Black-box contract for the root envelope's real planning_base (#3327)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-root-envelope-planning-base-fidelity"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "root-envelope-completion-contract"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-root-envelope-planning-base-fidelity"
)
ASSERTIONS = (
    "before any pending epic planning artifact exists, aw goal capability for an open capability gap with no active WI dispatches (status continue, next.kind run_command) with no payload_path anywhere in the envelope",
    "planting a genuinely pending epicize artifact under the fixture's own real, disposable project root -- never under /tmp/aw/test/root -- flips the identical live command to a blocked, requires_hitl envelope (status blocked, next.kind hitl) whose next.payload_path resolves byte-for-byte to that real on-disk artifact",
    "the blocked envelope's hitl_question independently repeats the identical real artifact path in its own target and question text and freeform_prompt, and carries exact, independently-computable project-derived id and resume_command strings that match next.command",
    "completion.missing grows by exactly the one new real-path-bearing reason while preserving the prior gap messages verbatim, and the literal stub path /tmp/aw/test/root never appears anywhere in either captured envelope",
)

_CAPABILITY_DOCUMENT = """# Demo Capabilities

## Brief

Isolated planning-base fidelity fixture for the workflow root runner's root
envelope completion contract (#3327).

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | planned | none | smoke | not-ready | one open gap awaits a real WI |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw goal capability demo-capability --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box planning-base fidelity contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one open work root with no active WI so the capability action envelope
reaches its create_wi remediation branch.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Real planning base fixture | change | - | planned | none | smoke | `true` |
"""

_EPICIZE_ARTIFACT = (
    "---\nkind: epicize\nagent_review_required: true\nreview_status: pending\n"
    "issue_count: 1\n---\n# plan\n"
)


def verify() -> list[str]:
    with project_fixture() as root:
        (root / "CAPABILITIES.md").write_text(_CAPABILITY_DOCUMENT, encoding="utf-8")

        # Before any planning artifact exists: the open gap with no active WI
        # dispatches straight to epicize. This is the negative control -- the
        # very same command, against the very same real root, with nothing
        # yet planted for it to find.
        before = final_json(
            run_aw(root, "goal", "capability", "demo-capability", "--project", "demo")
        )
        assert before["schema_version"] == "aw.cli.v1", before
        assert before["status"] == "continue", before
        assert before["action"] == "dispatch", before
        assert before["completion"]["requires_hitl"] is False, before
        assert before["next"]["kind"] == "run_command", before
        assert before["next"]["command"] == "aw wi epicize --project demo", before
        assert "payload_path" not in before["next"], before
        assert "payload_path" not in before, before
        assert before["completion"]["missing"] == [
            "gap `real-planning-base-fixture` is Open",
            "open capability gap has no active WI in README",
        ], before["completion"]
        assert "/tmp/aw/test/root" not in json.dumps(before), before

        # Plant a genuinely pending epicize artifact under the fixture's own
        # real, on-disk project root -- the exact frontmatter shape
        # `create_wi_blocks_on_pending_epicize_artifact` (apps/agentic-workflow/
        # src/cli/run.rs) proves the reader accepts.
        epics_dir = root / "aw" / "demo" / "epics"
        epics_dir.mkdir(parents=True)
        artifact_path = epics_dir / "20260101000000-demo-plan.md"
        artifact_path.write_text(_EPICIZE_ARTIFACT, encoding="utf-8")
        real_artifact_path = artifact_path.resolve()

        after = final_json(
            run_aw(
                root,
                "goal",
                "capability",
                "demo-capability",
                "--project",
                "demo",
                expect_success=None,
            )
        )

        # The identical live command against the identical real root now
        # blocks on review, and next.payload_path resolves byte-for-byte to
        # the real artifact this fixture planted a moment ago -- a hardcoded
        # `/tmp/aw/test/root` stub could never resolve to a path living
        # inside this disposable temp directory, so this is only reachable
        # if the CLI genuinely threaded the real project root through.
        assert after["schema_version"] == "aw.cli.v1", after
        assert after["status"] == "blocked", after
        assert after["action"] == "blocked", after
        assert after["completion"]["requires_hitl"] is True, after
        assert after["next"]["kind"] == "hitl", after
        assert after["next"]["command"] == (
            "aw goal capability --project demo --non-interactive --max-ticks 1"
        ), after
        reported_path = Path(after["next"]["payload_path"])
        assert reported_path.resolve() == real_artifact_path, (reported_path, real_artifact_path)
        assert root.name in after["next"]["payload_path"], after
        assert "/tmp/aw/test/root" not in after["next"]["payload_path"], after
        # The envelope also duplicates payload_path as a top-level field --
        # independently re-check it matches, rather than trusting one spot.
        assert after.get("payload_path") == after["next"]["payload_path"], after
        assert after["next"]["reason"] == (
            "pending epic planning artifact requires review before creating "
            "or linking WIs: " + after["next"]["payload_path"]
        ), after

        # The HITL question independently repeats the same real path in a
        # second and third field, plus exact, independently-computable
        # project-derived strings for its id and resume command.
        question = after["hitl_question"]
        assert question["id"] == "planning:demo:epicize_review", question
        assert question["interaction"]["kind"] == "user_question", question
        assert question["resume_command"] == after["next"]["command"], (question, after)
        assert question["target"] == after["next"]["payload_path"], (question, after)
        assert question["question"] == (
            "Review pending epic planning artifact `"
            + after["next"]["payload_path"]
            + "` before Agentic Workflow creates or links WIs?"
        ), question
        assert question["freeform_prompt"] == after["next"]["reason"], (question, after)
        assert question["default_choice"] == "approve_epic_plan", question
        assert {choice["id"] for choice in question["choices"]} == {
            "approve_epic_plan",
            "revise_epic_plan",
            "regenerate_epic_plan",
        }, question
        assert "/tmp/aw/test/root" not in question["question"], question

        # completion.missing grows by exactly the one new real-path-bearing
        # reason, preserving the prior gap messages verbatim, and the
        # historical stub never appears anywhere in the full envelope.
        assert after["completion"]["missing"] == (
            before["completion"]["missing"] + [after["next"]["reason"]]
        ), after["completion"]
        assert "/tmp/aw/test/root" not in json.dumps(after), after

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
