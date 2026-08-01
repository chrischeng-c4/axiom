"""Black-box contract for Python-only artifact-model routing (#3298)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_ID = "workflow-root-runner-python-only-artifact-model-routing"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "python-only-artifact-model-routing"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-python-only-artifact-model-routing"
)
ASSERTIONS = (
    "a freshly validated Change work item with no spec_model configured anywhere (an omitted value) routes its live aw goal wi next.command to the canonical Python-artifact table's exact EC-first command, proving omission resolves to the Python lifecycle rather than any legacy Markdown/section-payload path",
    "directly relabeling that same real, on-disk work item with a stale pre-EC-first tracker phase (td_contract_in_progress, a phase name from the retired TD-first lifecycle) still makes the live aw goal wi next.command the identical EC-first command, never an aw td command, proving a stale legacy phase can never restart the workflow inside TD",
    "relabeling the identical work item a second time with a genuinely current, recognized phase (ec_reviewed) makes the same live aw goal wi next.command become a real aw td check invocation, proving the very same routing table can and does reach TD -- so the stale-phase result above is specifically because that phase is unrecognized, not because TD is unreachable in general",
)

_EC_FIRST_COMMAND = "aw ec check --project demo --wi {slug}"


def _change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nDemonstrate that only the Python artifact lifecycle is ever reachable.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing phase-routing table\n"
        "Progress Evidence: the public goal wi envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace Python-only phase routing.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: stale legacy phases never resolve to TD.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| routing-trace | update | complete-platform.md |\n"
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


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(
            root,
            "Trace Python-only phase routing",
            "change",
            "--body",
            _change_body("trace Python-only phase routing"),
        )
        slug = created["slug"]

        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        expected_ec_first = _EC_FIRST_COMMAND.format(slug=slug)

        # Cluster 1: a freshly validated WI with no spec_model configured
        # anywhere routes to the canonical Python-artifact table's exact
        # EC-first command.
        hop0 = final_json(run_aw(root, "goal", "wi", slug))
        assert hop0["next"]["command"] == expected_ec_first, hop0

        # Cluster 2: directly relabeling the same real, on-disk work item
        # with a stale, pre-EC-first tracker phase (a real retired-lifecycle
        # phase name, not a nonsense string) still routes to the identical
        # EC-first command -- never TD.
        _relabel_phase(root, slug, "td_contract_in_progress")
        hop1 = final_json(run_aw(root, "goal", "wi", slug))
        assert hop1["next"]["command"] == expected_ec_first, hop1
        assert "aw td" not in hop1["next"]["command"], hop1

        # Cluster 3: relabeling the identical work item with a genuinely
        # current, recognized phase (`ec_reviewed`) makes the very same
        # routing table reach a real `aw td check` invocation -- proving
        # cluster 2's result is specifically because that phase is stale and
        # unrecognized, not because TD is unreachable in general.
        _relabel_phase(root, slug, "ec_reviewed")
        hop2 = final_json(run_aw(root, "goal", "wi", slug))
        final_command = hop2["next"]["command"]
        assert final_command.startswith("aw td check "), hop2
        assert not final_command.startswith("aw ec"), hop2
        assert final_command.endswith(f"--project demo --wi {slug}"), hop2

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
