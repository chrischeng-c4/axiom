"""Black-box contract for closed-workflow lock release (#3298)."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-closed-workflow-lock-release"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "closed-workflow-lock-release"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-closed-workflow-lock-release"
)
ASSERTIONS = (
    "a Change work item carrying the real score:locked label and a locked score:workflow-state projection block in its body is independently counted by the live aw health surface as one additional active blocker naming that exact work item and its expected command, while the work item remains open",
    "closing that same work item through the real aw wi close command leaves its on-disk lock label and locked projection block byte-for-byte intact -- proving any later change in aw health's answer comes from reading the tracker's issue state at read time, not from silently scrubbing the stale projection at close time",
    "after the close, aw health --project demo --json reports exactly the pre-lock baseline blocker count again and no blocker naming that work item, even though its stale locked projection is still physically present on disk under the closed/ issue file, matching the production behavior that closed/rejected items cannot retain a stale local projection that blocks unrelated TD/CB work",
)

_STALE_LOCK_BLOCK = """<!-- score:workflow-state
version: 1
issue_id: {slug}
locked: true
owner: td
expected_command: 'aw td create {slug} --apply --spec-path tech-design --project demo'
active_phase: td_inited
updated_at: '2026-08-01T00:00:00Z'
-->
"""


def _change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nDemonstrate that a closed work item releases its workflow lock.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing lock-view read path\n"
        "Progress Evidence: aw health's blocker count is the evidence\n\n"
        "## Requirements\n\n- R1: trace lock visibility across close.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: a closed WI's stale lock never blocks unrelated work.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| lock-release-trace | update | complete-platform.md |\n"
    )


def _workspace_slug(root: Path) -> str:
    resolved = str(root.resolve())
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", resolved)
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str, state: str) -> Path:
    return Path("/tmp/aw/workspaces") / _workspace_slug(root) / "issues" / state / f"{slug}.md"


def _health_report(root: Path) -> dict[str, object]:
    # The compact stdout summary's `blockers.blockers_preview` is truncated to
    # a small fixed window (`HEALTH_COMPACT_PREVIEW_LIMIT`), so its exact
    # membership depends on how many *other*, unrelated blockers happen to
    # exist and in what order -- fragile to assert against directly. The
    # summary's `payload_path` instead names an on-disk JSON file holding the
    # complete, untruncated `ProjectHealthReport` (see
    # `write_health_payload`/`health_payload_bytes` in
    # apps/agentic-workflow/src/cli/project.rs), whose own `blockers` field is
    # the full `Vec<String>`. Reading that file is the robust, independent way
    # to assert on a specific blocker string's exact presence or absence.
    completed = run_aw(root, "health", "--project", "demo", "--json", expect_success=None)
    payload = final_json(completed)
    compact_blockers = payload["blockers"]
    assert isinstance(compact_blockers, dict) and "blocker_count" in compact_blockers, payload
    payload_path = Path(str(payload["payload_path"]))
    assert payload_path.is_file(), payload
    full_report = json.loads(payload_path.read_text(encoding="utf-8"))
    assert isinstance(full_report.get("blockers"), list), full_report
    return {
        "blocker_count": compact_blockers["blocker_count"],
        "blockers": full_report["blockers"],
    }


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(
            root,
            "Trace lock release on close",
            "change",
            "--body",
            _change_body("trace lock release across close"),
        )
        slug = created["slug"]

        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        baseline = _health_report(root)
        baseline_count = baseline["blocker_count"]
        expected_blocker = (
            f"workflow lock: {slug} owned by td expects "
            f"`aw td create {slug} --apply --spec-path tech-design --project demo`"
        )
        assert expected_blocker not in baseline["blockers"], baseline

        # Simulate a stale workflow lock: exactly the shape `create_issue_lock`
        # writes (a `score:locked` label plus a locked `score:workflow-state`
        # projection block), applied directly to the on-disk local-backend
        # issue file. This mirrors the real TD lock-acquisition write without
        # needing to drive a full Python TD authoring session end to end, and
        # matches the "stale local projection" wording in the claim's own
        # promise text -- the scenario under test is specifically a lock left
        # behind by an interrupted or externally-closed workflow, not a lock
        # actively held by a live in-progress session.
        open_path = _issue_path(root, slug, "open")
        assert open_path.is_file(), open_path
        original = open_path.read_text(encoding="utf-8")
        assert "score:locked" not in original, original
        assert original.count("labels:\n") == 1, original
        locked_frontmatter = original.replace(
            "labels:\n", "labels:\n- score:locked\n- score:lock:td\n", 1
        )
        assert "score:locked" in locked_frontmatter, locked_frontmatter
        locked_body = (
            locked_frontmatter.rstrip("\n") + "\n\n" + _STALE_LOCK_BLOCK.format(slug=slug)
        )
        open_path.write_text(locked_body, encoding="utf-8")

        # Positive control: while still open, the real aw health surface must
        # independently recognize this hand-placed lock as one additional
        # active blocker -- proving the synthetic lock is genuinely potent
        # against the real read path, not an inert no-op.
        with_lock = _health_report(root)
        assert with_lock["blocker_count"] == baseline_count + 1, (baseline, with_lock)
        assert expected_blocker in with_lock["blockers"], with_lock

        # Close through the real, public `aw wi close` command.
        closed = final_json(
            run_aw(root, "wi", "close", slug, "--reason", "fixture teardown", "--json")
        )
        assert closed["state"] == "closed", closed

        # The stale label and locked projection remain byte-for-byte intact
        # on disk after close -- `LocalBackend::close` only flips `state`,
        # proving what follows is a read-time fix, not a write-time scrub.
        closed_path = _issue_path(root, slug, "closed")
        assert closed_path.is_file(), closed_path
        assert not open_path.exists(), open_path
        after_close = closed_path.read_text(encoding="utf-8")
        # `close()` legitimately flips the frontmatter `state` field and, as
        # an ordinary side effect of re-serializing the issue back to disk,
        # bumps its `updated_at` bookkeeping timestamp and may reflow
        # incidental blank lines around the body -- none of that is part of
        # the *lock* itself. What must be byte-for-byte intact is the lock
        # artifact proper: every stale lock label and the entire
        # `score:workflow-state` projection block, verbatim, proving
        # `close()` performs no lock-specific scrub of its own.
        assert "\nstate: closed\n" in after_close, after_close
        assert "\nstate: open\n" not in after_close, after_close
        after_close_lines = after_close.splitlines()
        assert "- score:locked" in after_close_lines, after_close
        assert "- score:lock:td" in after_close_lines, after_close
        assert _STALE_LOCK_BLOCK.format(slug=slug) in after_close, after_close

        # The real aw health surface now reports the pre-lock baseline count
        # again and no blocker naming this work item, even though its stale
        # projection is still physically present on disk.
        after_close_blockers = _health_report(root)
        assert after_close_blockers["blocker_count"] == baseline_count, (
            baseline,
            after_close_blockers,
        )
        assert expected_blocker not in after_close_blockers["blockers"], after_close_blockers
        assert not any(
            slug in blocker and "workflow lock" in blocker
            for blocker in after_close_blockers["blockers"]
        ), after_close_blockers

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
