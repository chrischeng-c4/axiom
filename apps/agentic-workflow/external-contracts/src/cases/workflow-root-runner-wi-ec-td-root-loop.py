"""Black-box contract for the WI/EC/TD root loop gate (#3298)."""

from __future__ import annotations

import json
import shlex
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import (
    create,
    final_json,
    project_fixture,
    run_aw,
    write_python_artifact_unit_test,
)


CASE_ID = "workflow-root-runner-wi-ec-td-root-loop"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "wi-ec-td-root-loop"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-wi-ec-td-root-loop"
)
ASSERTIONS = (
    "a fresh, validated Change root's live next.command is an aw ec command, never an aw td command, before any Python EC work exists for that root",
    "once the scaffolded EC inventory is authored for real and aw ec check reports clean, the live next.command still routes to aw ec review rather than any aw td command -- a clean structural check alone does not unlock TD",
    "only after the EC review is independently accepted and aw ec lock reports clean does the same root's live next.command become a real aw td check invocation, with the loop's own emitted guard literally reading 'EC -> TD'",
)

_CLAP_REJECTED_PREFIX = "error: the following required arguments were not provided:"

_RUNNER_SOURCE = """from __future__ import annotations

import json
from pathlib import Path


evidence = Path("external-contracts/evidence/{case_id}.json")
evidence.parent.mkdir(parents=True, exist_ok=True)
evidence.write_text(
    json.dumps(
        {{
            "protocol": "aw.python-ec.evidence.v1",
            "case_id": "{case_id}",
            "status": "passed",
            "assertions": ["the fixture EC runner executed"],
        }},
        sort_keys=True,
    )
    + "\\n",
    encoding="utf-8",
)
"""

_CASE_SOURCE = (
    "from __future__ import annotations\n"
    "\n"
    "\n"
    "def verify() -> list[str]:\n"
    '    return ["fixture EC case executed"]\n'
)

_CAPABILITIES_DOCUMENT = """# Loop Fixture Capabilities

## Brief

Isolated WI/EC/TD root-loop fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Loop Fixture | - | planned | none | smoke | blocked | root-loop fixture |

### Loop Fixture

ID: loop-fixture-capability
Type: DeveloperTool
Surfaces: CLI: `aw goal wi <slug>` - drives the WI/EC/TD root loop.
EC Dimensions: behavior: `true` - isolated black-box root-loop contract.
Root WI: -
Status: planned
Required Verification: smoke
Promise:
Prove EC precedes TD and TD opens only after EC evidence is locked.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Loop through EC before TD | change | - | planned | none | smoke | `true` |
"""


def _change_body(in_scope: str) -> str:
    return (
        "## Problem\n\nDemonstrate the WI/EC/TD root loop gate end to end.\n\n"
        "## Capability Alignment\n\n"
        "Capability: Workflow root runner\n"
        "Capability Gap: none, this fixture only drives the existing loop\n"
        "Progress Evidence: the public goal wi envelope is the evidence\n\n"
        "## Requirements\n\n- R1: trace the EC-before-TD gate.\n\n"
        f"## Scope\n\n### In Scope\n- {in_scope}\n\n"
        "### Out of Scope\n- Rework unrelated lifecycle stages.\n\n"
        "## Acceptance Criteria\n\n- AC1: TD opens only after EC is accepted and locked.\n\n"
        "## Reference Context\n\n### Related Specs\n"
        "| Spec | Relevance |\n|------|-----------|\n"
        "| complete-platform.md | describes the environment |\n\n"
        "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
        "|---------|--------|---------------|\n"
        "| root-loop-trace | update | complete-platform.md |\n"
    )


def _is_clap_rejected(stderr: str) -> bool:
    return stderr.strip().startswith(_CLAP_REJECTED_PREFIX)


def _run_captured(root: Path, command: str) -> subprocess.CompletedProcess[str]:
    assert command.startswith("aw "), command
    argv = shlex.split(command)[1:]
    return run_aw(root, *argv, expect_success=None)


def _accept_review(root: Path, pending: dict[str, object]) -> dict[str, object]:
    raw_path = Path(str(pending["payload_path"]))
    payload_path = raw_path if raw_path.is_absolute() else root / raw_path
    record = json.loads(payload_path.read_text(encoding="utf-8"))
    record.update(
        {
            "decision": "accepted",
            "reviewer_kind": "agent",
            "reviewed_by": "agent:fixture-reviewer",
            "reviewed_at": "2026-08-02T00:00:00Z",
            "summary": "Independent fixture review found specific assertions, an external oracle, and no false-green path.",
            "checklist": {
                "capability_claim_coverage": True,
                "required_dimensions": True,
                "assertions_specific": True,
                "oracle_independent": True,
                "loopholes_checked": True,
                "false_green_risk_checked": True,
            },
            "findings": [],
            "target_path": "",
        }
    )
    payload_path.write_text(
        json.dumps(record, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return final_json(
        run_aw(
            root,
            "ec",
            "review",
            "--project",
            "demo",
            "--evidence-file",
            str(payload_path),
            "--json",
        )
    )


def verify() -> list[str]:
    with project_fixture() as root:
        created = create(
            root,
            "Loop through EC before TD",
            "change",
            "--body",
            _change_body("trace the EC-before-TD gate"),
        )
        slug = created["slug"]

        validated = final_json(run_aw(root, "wi", "validate", slug))
        assert validated["passed"] is True, validated
        assert validated["new_state"] == "open", validated

        # Hop 0: a fresh root with no EC work yet must route to EC, never TD.
        hop0 = final_json(run_aw(root, "goal", "wi", slug))
        assert hop0["current"] == {"kind": "change", "id": slug}, hop0
        first_command = hop0["next"]["command"]
        assert first_command.startswith("aw ec "), hop0
        assert "aw td" not in first_command, hop0

        # Hop 1: the real check command scaffolds the EC inventory.
        hop1 = _run_captured(root, first_command)
        assert not _is_clap_rejected(hop1.stderr), hop1.stderr
        hop1_json = final_json(hop1)
        assert hop1_json["action"] == "python_ec_scaffold_required", hop1_json
        draft_command = hop1_json["next"]["command"]

        hop2 = _run_captured(root, draft_command)
        assert not _is_clap_rejected(hop2.stderr), hop2.stderr
        hop2_json = final_json(hop2)
        assert hop2_json["action"] == "python_ec_scaffold_created", hop2_json

        # Wire the scaffold into a real, checkable, reviewable EC case: a
        # placeholder capability_id/use_case_id/oracle/command would either
        # fail `aw ec check` outright or trip `aw ec review`'s false-green
        # detector, so the fixture must author genuine content exactly as a
        # real author would.
        (root / "CAPABILITIES.md").write_text(
            _CAPABILITIES_DOCUMENT, encoding="utf-8"
        )

        ec_root = root / "external-contracts"
        inventory_path = ec_root / "pyproject.toml"
        inventory = inventory_path.read_text(encoding="utf-8")
        case_id = f"{slug}-behavior"
        replacements = {
            'capability_id = "replace-with-capability-id"': (
                'capability_id = "loop-fixture-capability"'
            ),
            f'use_case_id = "{slug}"': 'use_case_id = "wi-ec-td-root-loop-fixture"',
            'oracle = "replace-with-independent-oracle"': (
                'oracle = "the outer EC independently inspects the real aw process output"'
            ),
            f'command = "test -s external-contracts/evidence/{case_id}.json"': (
                'command = "uv run --frozen --offline --project external-contracts '
                'python external-contracts/src/runner.py"'
            ),
        }
        for needle, replacement in replacements.items():
            assert needle in inventory, (needle, inventory)
            inventory = inventory.replace(needle, replacement)
        assert "replace-with" not in inventory, inventory
        inventory_path.write_text(inventory, encoding="utf-8")

        (ec_root / "src" / "runner.py").write_text(
            _RUNNER_SOURCE.format(case_id=case_id), encoding="utf-8"
        )
        (ec_root / "src" / f"{slug}.py").write_text(_CASE_SOURCE, encoding="utf-8")
        write_python_artifact_unit_test(ec_root)

        check = final_json(
            run_aw(root, "ec", "check", "--project", "demo", "--wi", slug, "--json")
        )
        assert check["clean"] is True, check

        # Hop 3: an EC-clean-but-not-yet-reviewed root still routes to
        # review, never TD -- a clean structural check alone is not
        # sufficient to unlock TD.
        hop3 = final_json(run_aw(root, "goal", "wi", slug))
        pre_review_command = hop3["next"]["command"]
        assert pre_review_command.startswith("aw ec review"), hop3
        assert "aw td" not in pre_review_command, hop3

        # Independently review, accept, and lock the EC evidence.
        pending = final_json(
            run_aw(root, "ec", "review", "--project", "demo", "--wi", slug, "--json")
        )
        assert pending["status"] == "pending_agent_review", pending
        accepted = _accept_review(root, pending)
        assert accepted["status"] == "accepted", accepted
        assert accepted["clean"] is True, accepted

        lock = final_json(
            run_aw(root, "ec", "lock", "--project", "demo", "--wi", slug, "--json")
        )
        assert lock["clean"] is True, lock
        assert lock["status"] == "locked", lock

        # Hop 4: only now does the same root's live next.command become a
        # real TD command, and the loop's own guard names the transition.
        hop4 = final_json(run_aw(root, "goal", "wi", slug))
        final_command = hop4["next"]["command"]
        assert "aw td" in final_command, hop4
        assert "EC -> TD" in hop4["prompt_contract"]["guards"], hop4

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
