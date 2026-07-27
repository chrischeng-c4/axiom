"""Black-box proof that the public EC lifecycle is Python-only."""

from __future__ import annotations

import json
from pathlib import Path

from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "python-ec-only-authoring"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "python-ec-only-authoring"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case python-ec-only-authoring"
)
ASSERTIONS = (
    "public EC help exposes no Markdown fill or generation authoring command",
    "draft, check, review, lock, and verify consume one Python EC project",
)


def _write_executable_python_ec(root: Path) -> None:
    ec_root = root / "external-contracts"
    (ec_root / "src/python_only.py").write_text(
        """\
from __future__ import annotations


def verify_python_only() -> None:
    assert True, "the external runner owns the independent assertion"
""",
        encoding="utf-8",
    )
    (ec_root / "src/runner.py").write_text(
        """\
from __future__ import annotations

import json
from pathlib import Path


evidence = Path("external-contracts/evidence/python-only.json")
evidence.parent.mkdir(parents=True, exist_ok=True)
evidence.write_text(
    json.dumps(
        {
            "protocol": "aw.python-ec.evidence.v1",
            "case_id": "python-only",
            "status": "passed",
            "assertions": ["the Python EC runner executed"],
        },
        sort_keys=True,
    )
    + "\\n",
    encoding="utf-8",
)
""",
        encoding="utf-8",
    )
    (ec_root / "pyproject.toml").write_text(
        """\
[project]
name = "python-only-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "agent:fixture-author"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "python-only"
artifact_id = "artifact:demo/python-only"
capability_id = "fixture-capability"
use_case_id = "python-only-lifecycle"
dimension = "behavior"
applicability = "td"
test_path = "src/python_only.py"
promise = "The public EC lifecycle executes only the project-local Python contract."
oracle = "An independent black-box fixture checks CLI help, artifact paths, review evidence, and runner output."
target = "rust"
command = "python3 external-contracts/src/runner.py"
evidence_paths = ["evidence/python-only.json"]
""",
        encoding="utf-8",
    )


def _accept_review(root: Path, pending: dict[str, object]) -> dict[str, object]:
    raw_path = Path(str(pending["payload_path"]))
    payload_path = raw_path if raw_path.is_absolute() else root / raw_path
    record = json.loads(payload_path.read_text(encoding="utf-8"))
    record.update(
        {
            "decision": "accepted",
            "reviewer_kind": "agent",
            "reviewed_by": "agent:fixture-reviewer",
            "reviewed_at": "2026-07-27T00:00:00Z",
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
        config = root / "aw.toml"
        config.write_text(
            config.read_text(encoding="utf-8").replace(
                'name = "demo"\n',
                'name = "demo"\nspec_model = "legacy"\n',
                1,
            ),
            encoding="utf-8",
        )
        (root / "CAPABILITIES.md").write_text(
            """\
# Demo Capabilities

## Brief

Demo capability contract for the Python-only EC fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Fixture Capability | - | implemented | verified | smoke | ready | Python-only EC fixture |

### Fixture Capability

ID: fixture-capability
Type: DeveloperTool
Surfaces:
- CLI: demo
EC Dimensions:
- behavior: Python-only lifecycle boundary
Root WI: -
Status: verified
Required Verification: smoke
Promise:
The fixture lifecycle remains externally observable.
Gate Inventory:
- external-contracts/evidence/python-only.json
""",
            encoding="utf-8",
        )

        help_result = run_aw(root, "ec", "--help")
        help_text = help_result.stdout.lower()
        assert "draft" in help_text
        assert "check" in help_text
        assert "review" in help_text
        assert "lock" in help_text
        assert "verify" in help_text
        assert "\n  fill" not in help_text
        assert "\n  gen" not in help_text
        assert "markdown" not in help_text
        assert "compatibility" not in help_text
        orientation = run_aw(root, "llm", "--topic", "ec").stdout.lower()
        assert "draft -> check -> review -> lock" in orientation
        assert "draft -> fill" not in orientation
        assert "aw ec fill" not in orientation
        assert "aw ec gen" not in orientation

        draft = final_json(
            run_aw(
                root,
                "ec",
                "draft",
                "python-only",
                "--project",
                "demo",
                "--capability-id",
                "fixture-capability",
                "--title",
                "Python-only EC fixture",
                "--json",
            )
        )
        assert draft["action"] == "python_ec_scaffold_created"
        assert all(
            path == "external-contracts/pyproject.toml"
            or path.endswith(".py")
            for path in draft["artifacts"]
        )
        assert not list((root / "external-contracts").rglob("*.md"))

        for retired in ("fill", "gen"):
            rejected = run_aw(
                root,
                "ec",
                retired,
                "--project",
                "demo",
                expect_success=False,
            )
            assert "unrecognized subcommand" in rejected.stderr.lower()

        _write_executable_python_ec(root)
        checked = final_json(
            run_aw(root, "ec", "check", "--project", "demo", "--json")
        )
        assert checked["clean"] is True
        assert checked["case_count"] == 1

        pending = final_json(
            run_aw(root, "ec", "review", "--project", "demo", "--json")
        )
        assert pending["status"] == "pending_agent_review"
        assert pending["payload_path"]
        accepted = _accept_review(root, pending)
        assert accepted["status"] == "accepted"
        assert accepted["next"] == "aw ec lock --project demo"

        locked = final_json(
            run_aw(root, "ec", "lock", "--project", "demo", "--json")
        )
        assert locked["clean"] is True
        assert locked["source_count"] >= 2

        verified = final_json(
            run_aw(root, "ec", "verify", "--project", "demo", "--json")
        )
        assert verified["clean"] is True
        assert verified["failed_count"] == 0
        assert (root / "external-contracts/evidence/python-only.json").is_file()
        assert not list((root / "external-contracts").rglob("*.md"))

    return [
        "public EC help and retired-command failures expose no Markdown fill or generation authoring path",
        "a legacy-configured fixture still drafts, checks, reviews, locks, and verifies one Python EC project",
    ]
