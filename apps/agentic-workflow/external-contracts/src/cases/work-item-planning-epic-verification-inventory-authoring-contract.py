"""Black-box contract for the epic verification inventory authoring contract (#3304)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw

CASE_ID = "work-item-planning-epic-verification-inventory-authoring-contract"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-verification-inventory-authoring-contract"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case work-item-planning-epic-verification-inventory-authoring-contract"
)
ASSERTIONS = (
    "a freshly created epic sits in draft state with no `## Verification Inventory` section and "
    "draft creation raises no complaint, while its live aw wi fill-section brief explicitly demands "
    "a Verification Inventory that MUST map every Requirement to a runnable Gate and observable "
    "Oracle -- and the identical fill-section brief for a change never mentions Verification "
    "Inventory at all, proving the section is planner-owned and epic-specific rather than a generic "
    "template fragment",
    "promoting that same two-requirement draft epic via aw wi validate fails closed naming both "
    "unmapped requirements, still fails after a body update that maps only the first requirement -- "
    "naming exactly the second and never the first -- proving enforcement is per-Requirement rather "
    "than an all-or-nothing presence check",
    "mapping the second requirement's Gate and Oracle too makes the identical aw wi validate call "
    "pass, promotes state_promoted=true with new_state=open, and a follow-up aw wi show confirms the "
    "tracker record itself now reads open -- proving the fails-closed promotion gate is real and its "
    "removal genuinely unblocks the exact same command rather than the failures above being "
    "coincidental",
)

_BASE_EPIC_BODY = (
    "## Problem\n\ndemo\n\n"
    "## Capability Alignment\n\nCapability: x\nCapability Gap: y\nProgress Evidence: z\n\n"
    "## Requirements\n\n- R1: cover requirement one.\n- R2: cover requirement two.\n\n"
    "## Scope\n\n### In Scope\n- a\n\n### Out of Scope\n- b\n\n"
    "## Acceptance Criteria\n\n- AC1: c\n\n"
    "## Reference Context\n\n### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n| x | modify | x.md |\n"
)

_CHANGE_BODY = (
    "## Problem\n\ndemo\n\n## Capability Alignment\n\nCapability: x\nCapability Gap: y\n"
    "Progress Evidence: z\n\n## Requirements\n\n- R1: x.\n\n## Scope\n\n### In Scope\n- a\n\n"
    "### Out of Scope\n- b\n\n## Acceptance Criteria\n\n- AC1: c\n\n## Reference Context\n\n"
    "### Related Specs\n| Spec | Relevance |\n|------|-----------|\n| x.md | high |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n|---------|--------|---------------|\n"
    "| x | modify | x.md |\n"
)


def _with_inventory(rows: str) -> str:
    inventory = (
        "## Verification Inventory\n\n"
        "| Requirement | Gate | Oracle | Depends On |\n"
        "|-------------|------|--------|------------|\n"
        f"{rows}\n\n"
    )
    return _BASE_EPIC_BODY.replace("## Reference Context", inventory + "## Reference Context")


def verify() -> list[str]:
    with project_fixture() as root:
        slug = create(root, "Epic verification inventory", "epic", "--priority", "p1", "--body", _BASE_EPIC_BODY)[
            "slug"
        ]

        # Assertion 1: draft creation is valid by construction; the epic brief
        # demands Verification Inventory, the change brief never mentions it.
        shown = final_json(run_aw(root, "wi", "show", slug, "--json"))
        issue = shown.get("issue", shown)
        assert issue.get("state") == "draft", issue
        assert "## Verification Inventory" not in issue.get("body", ""), issue

        epic_brief = run_aw(root, "wi", "fill-section", "--slug", slug, "--section", "all")
        assert epic_brief.returncode == 0, epic_brief
        assert "Verification Inventory" in epic_brief.stdout, epic_brief.stdout
        assert "MUST map every Requirement to a runnable Gate and observable Oracle" in epic_brief.stdout, (
            epic_brief.stdout
        )

        change_slug = create(root, "Change fill brief check", "change", "--body", _CHANGE_BODY)["slug"]
        change_brief = run_aw(root, "wi", "fill-section", "--slug", change_slug, "--section", "all")
        assert change_brief.returncode == 0, change_brief
        assert "Verification Inventory" not in change_brief.stdout, change_brief.stdout

        # Assertion 2: promotion fails closed, per Requirement.
        no_inventory = json.loads(
            run_aw(root, "wi", "validate", slug, "--json", expect_success=False).stdout
        )
        assert no_inventory["passed"] is False, no_inventory
        assert any("map R1" in e for e in no_inventory["errors"]), no_inventory
        assert any("map R2" in e for e in no_inventory["errors"]), no_inventory

        r1_only_path = root / "epic-r1-only.md"
        r1_only_path.write_text(_with_inventory("| R1 | `aw wi plan` | R1 verified. | - |"), encoding="utf-8")
        final_json(run_aw(root, "wi", "update", slug, "--body-file", str(r1_only_path), "--json"))

        r1_only = json.loads(run_aw(root, "wi", "validate", slug, "--json", expect_success=False).stdout)
        assert r1_only["passed"] is False, r1_only
        assert not any("map R1" in e for e in r1_only["errors"]), r1_only
        assert any("map R2" in e for e in r1_only["errors"]), r1_only

        # Assertion 3: full coverage passes and actually promotes tracker state.
        both_path = root / "epic-both.md"
        both_path.write_text(
            _with_inventory(
                "| R1 | `aw wi plan` | R1 verified. | - |\n| R2 | `aw wi graph` | R2 verified. | R1 |"
            ),
            encoding="utf-8",
        )
        final_json(run_aw(root, "wi", "update", slug, "--body-file", str(both_path), "--json"))

        both = final_json(run_aw(root, "wi", "validate", slug, "--json"))
        assert both["passed"] is True, both
        assert both["state_promoted"] is True, both
        assert both["new_state"] == "open", both

        shown2 = final_json(run_aw(root, "wi", "show", slug, "--json"))
        issue2 = shown2.get("issue", shown2)
        assert issue2.get("state") == "open", issue2

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
