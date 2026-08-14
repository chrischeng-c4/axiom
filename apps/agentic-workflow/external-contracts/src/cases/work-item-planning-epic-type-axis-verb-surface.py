"""Black-box contract for the `aw wi epic` type-axis verb surface.

The work-item type enum is closed (epic | change | spike | report) and each type
declares its own terminal state. For `epic` that terminal state is "all owned
children are terminal". This contract pins that the epic axis is reachable as
`aw wi epic {create,update,validate,close}` and that the terminal-state rule is a
refusal at the mutation point rather than a diagnostic emitted after the fact.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show

CASE_ID = "work-item-planning-epic-type-axis-verb-surface"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-type-axis-verb-surface"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-epic-type-axis-verb-surface"
)
ASSERTIONS = (
    "the live CLI exposes `aw wi epic` as a type-axis group whose command list is exactly "
    "create, update, validate, and close, where create fixes the work-item type from the axis "
    "rather than a flag -- rejecting `--type` outright while still emitting the canonical "
    "`type:epic` label -- validate accepts the created epic, update mutates its labels, and "
    "close drives it to state closed once every owned change child is itself closed",
    "`aw wi epic close` refuses an epic that still owns a non-closed change: it exits non-zero, "
    "names the blocking child by exact id, leaves the epic's on-disk tracker record byte-for-byte "
    "unmodified, and leaves its state not closed -- while `aw wi epic update`, `aw wi epic "
    "validate`, and `aw wi epic close` each independently refuse a `type:change` target without "
    "mutating it, proving the type axis and the children-terminal rollup are enforced refusals "
    "at the mutation point and not post-hoc graph diagnostics, so the first cluster is not vacuous",
)

_EPIC_BODY = (
    "## Requirements\n\n- R1: Own the change children this contract closes.\n\n"
    "## Verification Inventory\n\n"
    "| Requirement | Gate | Oracle | Depends On |\n"
    "|-------------|------|--------|------------|\n"
    "| R1 | `aw wi epic close` | close refuses until every child is closed. | - |\n"
)

_CHANGE_BODY = (
    "## Goal\n\n"
    "Owned child change whose open state must block the parent epic's close.\n\n"
    "## How\n\n"
    "### Verified premises\n\n"
    "- apps/agentic-workflow/src/cli/issues.rs:4052 - R1: the close path resolves the target "
    "work item before mutating tracker state.\n\n"
    "### Change points\n\n"
    "- apps/agentic-workflow/src/cli/issues.rs — the epic close leaf gains the rollup guard.\n\n"
    "### Frozen decisions\n\n"
    "A child is terminal exactly when its tracker state is closed.\n\n"
    "## Acceptance\n\n"
    "| # | command | current | target | why it cannot hold by accident |\n"
    "|---|---------|---------|--------|--------------------------------|\n"
    "| 1 | `aw wi epic close` | closes with an open child | refuses with an open child | "
    "the refusal names the blocking child by id |\n\n"
    "### Negative control\n\n"
    "Under a mutation that drops the rollup guard the gate must go red, restoring to sha256 "
    "23ea20b1513817f0991d6aaaea8f4fb3eaec71181bc63d23db8fb24c457b171c\n\n"
    "## Never\n\n"
    "This addresses the worker implementing the epic axis, not the controller reviewing it.\n\n"
    "### Must not touch\n\n"
    "- apps/agentic-workflow/src/issues/ghan.rs — validator rules are immutable for this change.\n\n"
    "### Must not do\n\n"
    "- Do not satisfy the rollup by reading the graph projection after the close has landed.\n"
)

_EXPECTED_LEAVES = {"create", "update", "validate", "close"}


def _workspace_slug(root: Path) -> str:
    collapsed = re.sub(r"[^a-zA-Z0-9]+", "-", str(root.resolve()))
    return collapsed.strip("-").lower()


def _issue_path(root: Path, slug: str, state: str) -> Path:
    return (
        Path("/tmp/aw/workspaces")
        / _workspace_slug(root)
        / "issues"
        / state
        / f"{slug}.md"
    )


def _help_leaves(help_text: str) -> set[str]:
    """Extract the command names clap prints under the `Commands:` block."""
    lines = help_text.splitlines()
    try:
        start = next(i for i, line in enumerate(lines) if line.strip() == "Commands:")
    except StopIteration as exc:  # pragma: no cover - surfaced as an assertion
        raise AssertionError(f"`aw wi epic --help` has no Commands block:\n{help_text}") from exc
    leaves: set[str] = set()
    for line in lines[start + 1 :]:
        if not line.strip():
            break
        if not line.startswith((" ", "\t")):
            break
        name = line.strip().split()[0]
        if name == "help":
            continue
        leaves.add(name)
    return leaves


def _epic_create(root: Path, title: str) -> str:
    created = final_json(
        run_aw(
            root,
            "wi",
            "epic",
            "create",
            "--title",
            title,
            "--project",
            "demo",
            "--priority",
            "p1",
            "--body",
            _EPIC_BODY,
            "--json",
        )
    )
    return created["slug"]


def _change_create(root: Path, title: str, epic: str) -> str:
    return create(root, title, "change", "--epic", epic, "--body", _CHANGE_BODY)["slug"]


def _assert_refused(completed, *, must_mention: str) -> str:
    combined = f"{completed.stdout}\n{completed.stderr}"
    assert completed.returncode != 0, f"expected a refusal, got rc=0:\n{combined}"
    assert must_mention in combined, (
        f"refusal did not name `{must_mention}`:\n{combined}"
    )
    return combined


def _verify_type_axis_surface(root: Path) -> str:
    leaves = _help_leaves(run_aw(root, "wi", "epic", "--help").stdout)
    assert leaves == _EXPECTED_LEAVES, leaves

    # The axis, not a flag, fixes the type: `--type` is not part of the leaf's surface.
    typed = run_aw(
        root,
        "wi",
        "epic",
        "create",
        "--title",
        "Rejected explicit type",
        "--type",
        "epic",
        "--project",
        "demo",
        "--body",
        _EPIC_BODY,
        "--json",
        expect_success=False,
    )
    assert "--type" in f"{typed.stdout}\n{typed.stderr}"

    epic = _epic_create(root, "Terminal rollup epic")
    assert "type:epic" in show(root, epic)["labels"], show(root, epic)

    assert final_json(run_aw(root, "wi", "epic", "validate", epic))["passed"] is True

    updated = final_json(
        run_aw(root, "wi", "epic", "update", epic, "--add-label", "area:planning", "--json")
    )
    assert "area:planning" in updated["labels"], updated

    return epic


def _verify_rollup_refusal(root: Path, epic: str) -> str:
    child = _change_create(root, "Owned open child", epic)

    before = _issue_path(root, epic, "open").read_bytes()
    refused = run_aw(root, "wi", "epic", "close", epic, "--json", expect_success=False)
    _assert_refused(refused, must_mention=child)
    assert _issue_path(root, epic, "open").read_bytes() == before, (
        "the refused close mutated the epic's tracker record"
    )
    assert show(root, epic)["state"] != "closed", show(root, epic)

    run_aw(root, "wi", "close", child, "--json")
    closed = final_json(run_aw(root, "wi", "epic", "close", epic, "--json"))
    assert closed["state"] == "closed", closed
    return child


def _verify_non_epic_targets_refused(root: Path) -> None:
    host = _epic_create(root, "Type guard host epic")
    change = _change_create(root, "Non-epic target", host)
    before = _issue_path(root, change, "open").read_bytes()

    invocations = (
        ("update", ("--add-label", "area:planning", "--json")),
        ("validate", ()),
        ("close", ("--json",)),
    )
    for leaf, extra in invocations:
        refused = run_aw(
            root, "wi", "epic", leaf, change, *extra, expect_success=False
        )
        _assert_refused(refused, must_mention=change)
        assert _issue_path(root, change, "open").read_bytes() == before, (
            f"`aw wi epic {leaf}` mutated a non-epic target it should have refused"
        )


def verify() -> list[str]:
    with project_fixture() as root:
        epic = _verify_type_axis_surface(root)
        _verify_rollup_refusal(root, epic)
        _verify_non_epic_targets_refused(root)
    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
