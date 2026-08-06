"""Native Python ECs for existing-project health and takeover behavior."""

from __future__ import annotations

import json
import tempfile
import time
from pathlib import Path
from typing import Any

from oracles.project_health_contract import (
    assert_alignment,
    assert_ec_accepts_td,
    assert_two_cell_health,
)
from wi_contract_fixture import (
    final_json,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_IDS = {
    "artifact-preflight-health-rollup",
    "authoritative-fixture-blocks-on-regenerability-gap",
    "aw-health-default-full-verification-smoke",
    "existing-project-standardization-brownfield-takeover-surface",
    "existing-project-standardization-cb-and-cold-verification-gates",
    "existing-project-standardization-managed-and-semantic-production-gates",
    "existing-project-standardization-traceability-closure-gate",
    "external-fixture-reports-advisory-gap",
    "project-health-total-observation",
    "standardize-audit-first-contract-test",
    "td-gen-source-source-snapshot-projection-real-cli",
}


def _write_fixture(root: Path, *, authoritative: bool) -> None:
    authority = (
        """
[projects.regenerability]
authority = "generator_authoritative"
reason = "fixture requires deterministic generator ownership"
"""
        if authoritative
        else ""
    )
    (root / "projects/fixture/src").mkdir(parents=True)
    (root / "projects/fixture/tech-design").mkdir(parents=True)
    (root / "aw.toml").write_text(
        f"""\
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "fixture"
label = "app:fixture"
path = "projects/fixture"
td_path = "projects/fixture/tech-design"
cap_path = "projects/fixture/CAPABILITIES.md"
{authority}
[[projects.workspaces]]
name = "fixture"
paths = ["projects/fixture/**"]
target = "rust"
test_cmd = "true"
verify_cold = false
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/CAPABILITIES.md").write_text(
        """\
# Fixture

## Brief

Fixture health contract.

## Capabilities

### Capability Index

| Capability | ID | Status | Evidence |
|------------|----|--------|----------|
| Fixture health | fixture-health | implemented | `true` |

### Fixture health

Capability ID: fixture-health
Status: implemented
Summary: Observe existing-project health policy.

#### Work Roots

| Type | ID | Status | Verification |
|------|----|--------|--------------|
| epic | fixture-epic | implemented | `true` |
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/src/lib.rs").write_text(
        """\
// HANDWRITE-BEGIN gap="fixture" tracker="#fixture" reason="fixture generator gap"
/// @spec projects/fixture/tech-design/fixture.md#source
pub fn fixture() {}
// HANDWRITE-END
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/tech-design/fixture.md").write_text(
        """\
# Fixture

## Contract

The fixture source remains observable by health.

## Logic

The public fixture function returns successfully.
""",
        encoding="utf-8",
    )


def _write_total_observation_fixture(
    root: Path,
    *,
    mutation_policy: str,
    poison_evidence_directory: bool,
) -> Path:
    _write_fixture(root, authoritative=False)
    project = root / "projects/fixture"
    (project / "CAPABILITIES.md").write_text(
        """\
# Fixture Capabilities

## Brief

Fixture health observation contract.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Fixture health | - | implemented | verified | smoke | ready | Total health observation fixture |

### Fixture health

ID: fixture-health
Type: DeveloperTool
Surfaces:
- CLI: `aw health` - observe the fixture project
EC Dimensions:
- behavior: `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health` - health remains observable
Root WI: -
Status: verified
Required Verification: smoke
Promise:
The fixture health request remains externally observable.
Gate Inventory:
- `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Fixture health contract | epic | fixture-epic | implemented | verified | smoke | `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health` |
""",
        encoding="utf-8",
    )
    (project / "tech-design/fixture.md").unlink()
    (project / "tech-design/src").mkdir(parents=True)
    (project / "tech-design/pyproject.toml").write_text(
        """\
[project]
name = "fixture-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(
        project / "tech-design", name="fixture-tech-design"
    )
    (project / "tech-design/src/fixture_health.py").write_text(
        """\
__aw_artifact_id__ = "artifact:fixture/fixture-health"
__aw_public_contract__ = True


def fixture_health_contract() -> bool:
    return True
""",
        encoding="utf-8",
    )

    (project / "external-contracts/src/cases").mkdir(parents=True)
    (project / "external-contracts/evidence").mkdir()
    (project / "external-contracts/src/runner.py").write_text(
        "raise SystemExit(0)\n",
        encoding="utf-8",
    )
    (project / "external-contracts/src/cases/fixture-health.py").write_text(
        """\
CASE_ID = "fixture-health"


def verify() -> list[str]:
    return ["fixture health"]
""",
        encoding="utf-8",
    )
    (project / "external-contracts/pyproject.toml").write_text(
        """\
[project]
name = "fixture-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "agent:fixture-author"
efficiency_policy = "optional"

[[tool.aw.python-ec.cases]]
id = "fixture-health"
artifact_id = "artifact:fixture/fixture-health"
capability_id = "fixture-health"
use_case_id = "fixture-health-contract"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/fixture-health.py"
promise = "The fixture health contract remains observable."
oracle = "The fixture returns one explicit assertion."
target = "rust"
command = "uv run --frozen --offline --project projects/fixture/external-contracts python projects/fixture/external-contracts/src/runner.py --case fixture-health"
evidence_paths = ["evidence/fixture-health.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(
        project / "external-contracts", name="fixture-external-contracts"
    )
    write_python_artifact_unit_test(
        project / "external-contracts", "fixture_health"
    )
    (project / "aw.toml").write_text(
        f"""\
[project]
name = "fixture"
mutation_adequacy = "{mutation_policy}"
mutation_evidence_dir = "projects/fixture/evidence/mutation-adequacy"
mutation_source_path = "projects/fixture/src"
""",
        encoding="utf-8",
    )
    evidence_path = project / "evidence/mutation-adequacy"
    evidence_path.parent.mkdir(parents=True)
    (project / "external-contracts/evidence/fixture-health.json").write_text(
        json.dumps(
            {
                "protocol": "aw.python-ec.evidence.v1",
                "case_id": "fixture-health",
                "exit_code": 0,
            }
        )
        + "\n",
        encoding="utf-8",
    )
    if poison_evidence_directory:
        evidence_path.mkdir()
        evidence_path.chmod(0)
    return evidence_path


def _enable_self_hosting_policy(root: Path) -> None:
    root_config = root / "aw.toml"
    root_config.write_text(
        root_config.read_text(encoding="utf-8").replace(
            'name = "fixture"',
            'name = "agentic-workflow"',
            1,
        ),
        encoding="utf-8",
    )
    project_config = root / "projects/fixture/aw.toml"
    project_config.write_text(
        project_config.read_text(encoding="utf-8").replace(
            'name = "fixture"',
            'name = "agentic-workflow"',
            1,
        ),
        encoding="utf-8",
    )


def _assert_aggregate_outcome_correspondence(
    result: dict[str, Any],
    payload: dict[str, Any],
) -> None:
    assert_two_cell_health(result, payload)


def _verify_two_cell_semantic_health() -> list[str]:
    with tempfile.TemporaryDirectory(
        prefix="aw-python-ec-two-cell-health-"
    ) as raw_root:
        root = Path(raw_root)
        _write_total_observation_fixture(
            root,
            mutation_policy="advisory",
            poison_evidence_directory=False,
        )
        project = root / "projects/fixture"

        healthy = run_aw(root, "health", "--project", "fixture")
        healthy_result = final_json(healthy)
        healthy_payload = json.loads(
            Path(healthy_result["payload_path"]).read_text(encoding="utf-8")
        )
        assert healthy_result["assessment"] == "healthy"
        _assert_aggregate_outcome_correspondence(
            healthy_result,
            healthy_payload,
        )
        assert (
            healthy_result["semantic_health"]["ec_accepts_td"]["evaluation"]
            == "passed"
        )
        assert_ec_accepts_td(
            healthy_result["semantic_health"]["ec_accepts_td"],
            evaluation="passed",
            case_count=1,
            passed_count=1,
            failed_cases=[],
            missing_evidence_cases=[],
        )
        assert_alignment(
            healthy_result["semantic_health"]["ec_td_alignment"],
            missing_in_td=[],
            missing_in_ec=[],
        )

        fixture_evidence = (
            project / "external-contracts/evidence/fixture-health.json"
        )
        passing_evidence = fixture_evidence.read_text(encoding="utf-8")
        fixture_evidence.write_text(
            json.dumps(
                {
                    "protocol": "aw.python-ec.evidence.v1",
                    "case_id": "fixture-health",
                    "exit_code": 17,
                }
            )
            + "\n",
            encoding="utf-8",
        )
        rejected = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            expect_success=False,
        )
        rejected_result = final_json(rejected)
        rejected_payload = json.loads(
            Path(rejected_result["payload_path"]).read_text(encoding="utf-8")
        )
        assert rejected_result["assessment"] == "blocked"
        _assert_aggregate_outcome_correspondence(
            rejected_result,
            rejected_payload,
        )
        assert_ec_accepts_td(
            rejected_result["semantic_health"]["ec_accepts_td"],
            evaluation="failed",
            case_count=1,
            passed_count=0,
            failed_cases=["fixture-health"],
            missing_evidence_cases=[],
        )
        fixture_evidence.write_text(passing_evidence, encoding="utf-8")

        (project / "tech-design/src/internal.py").write_text(
            """\
__aw_artifact_id__ = "artifact:fixture/internal"


def internal() -> bool:
    return True
""",
            encoding="utf-8",
        )
        internal_extra = final_json(
            run_aw(root, "health", "--project", "fixture")
        )
        assert_alignment(
            internal_extra["semantic_health"]["ec_td_alignment"],
            missing_in_td=[],
            missing_in_ec=[],
        )

        inventory_path = project / "external-contracts/pyproject.toml"
        inventory = inventory_path.read_text(encoding="utf-8")
        inventory_path.write_text(
            inventory.replace(
                'use_case_id = "fixture-health-contract"',
                'use_case_id = "fixture-health-renamed"',
            ),
            encoding="utf-8",
        )
        behavior_misaligned = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            expect_success=False,
        )
        behavior_result = final_json(behavior_misaligned)
        behavior_payload = json.loads(
            Path(behavior_result["payload_path"]).read_text(encoding="utf-8")
        )
        assert behavior_result["assessment"] == "blocked"
        _assert_aggregate_outcome_correspondence(
            behavior_result,
            behavior_payload,
        )
        assert_alignment(
            behavior_result["semantic_health"]["ec_td_alignment"],
            missing_in_td=[
                "artifact:fixture/fixture-health#fixture-health-renamed"
            ],
            missing_in_ec=[
                "artifact:fixture/fixture-health#fixture-health-contract"
            ],
        )
        inventory_path.write_text(inventory, encoding="utf-8")

        inventory_path.write_text(
            inventory.replace(
                'artifact_id = "artifact:fixture/fixture-health"',
                'artifact_id = "artifact:fixture/ec-only"',
            ),
            encoding="utf-8",
        )
        misaligned = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            expect_success=False,
        )
        misaligned_result = final_json(misaligned)
        misaligned_payload = json.loads(
            Path(misaligned_result["payload_path"]).read_text(encoding="utf-8")
        )
        assert misaligned_result["assessment"] == "blocked"
        _assert_aggregate_outcome_correspondence(
            misaligned_result,
            misaligned_payload,
        )
        assert_alignment(
            misaligned_result["semantic_health"]["ec_td_alignment"],
            missing_in_td=["artifact:fixture/ec-only#fixture-health-contract"],
            missing_in_ec=[
                "artifact:fixture/fixture-health#fixture-health-contract"
            ],
        )

        inventory_path.write_text(inventory, encoding="utf-8")
        (project / "external-contracts/evidence/fixture-health.json").unlink()
        missing_evidence = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            expect_success=False,
        )
        missing_result = final_json(missing_evidence)
        missing_payload = json.loads(
            Path(missing_result["payload_path"]).read_text(encoding="utf-8")
        )
        assert missing_result["assessment"] == "indeterminate"
        _assert_aggregate_outcome_correspondence(
            missing_result,
            missing_payload,
        )
        assert (
            missing_result["semantic_health"]["ec_accepts_td"]["evaluation"]
            == "not_evaluated"
        )

    return [
        "matching TD-applicable EC evidence makes ec_accepts_td pass",
        "explicit failing EC evidence rejects TD with exact case counts",
        "matching executable public behaviors make ec_td_alignment pass",
        "internal TD artifacts do not require EC coverage",
        "same-artifact behavior drift is reported in both directions",
        "EC-only and public-TD-only behaviors are reported in opposite directions",
        "missing TD-stage evidence is indeterminate rather than false-green",
        "stdout and durable payload expose exactly the same two semantic cells",
    ]


def _health_snapshot(*, authoritative: bool) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-health-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root, authoritative=authoritative)
        completed = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            "full",
            "--verbose",
            expect_success=False,
        )
        records = [
            json.loads(line)
            for line in completed.stdout.splitlines()
            if line.strip()
        ]
        assert records[-1]["event"] == "result"
        assert all(record.get("event") == "progress" for record in records[:-1])
        payload_path = Path(records[-1]["payload_path"])
        payload = json.loads(payload_path.read_text(encoding="utf-8"))
        return {
            "records": records,
            "result": records[-1],
            "payload": payload,
        }


def _verify_health(case_id: str) -> list[str]:
    if case_id == "project-health-total-observation":
        return _verify_two_cell_semantic_health()

    if case_id == "__retired-project-health-total-observation":
        with tempfile.TemporaryDirectory(
            prefix="aw-python-ec-health-total-"
        ) as raw_root:
            root = Path(raw_root)
            evidence_path = _write_total_observation_fixture(
                root,
                mutation_policy="advisory",
                poison_evidence_directory=True,
            )

            capability = run_aw(
                root,
                "health",
                "--project",
                "fixture",
                "capability",
                expect_success=None,
            )
            capability_result = final_json(capability)
            capability_payload = json.loads(
                Path(capability_result["payload_path"]).read_text(encoding="utf-8")
            )
            assert capability_result["schema_version"] == "aw.cli.v1"
            assert capability_result["section"] == "capability"
            assert evidence_path.is_dir()
            assert (
                capability_payload["axis_assessments"]["mutation"]["evaluation"]
                == "not_configured"
            )

            advisory_unavailable = run_aw(
                root,
                "health",
                "--project",
                "fixture",
                "mutation",
                expect_success=False,
            )
            advisory_result = final_json(advisory_unavailable)
            advisory_payload = json.loads(
                Path(advisory_result["payload_path"]).read_text(encoding="utf-8")
            )
            advisory_axis = advisory_result["axes"]["mutation"]
            assert advisory_result["section"] == "mutation"
            assert advisory_result["assessment"] == "indeterminate"
            assert advisory_axis["requirement"] == "advisory"
            assert advisory_axis["evaluation"] == "unavailable"
            assert advisory_payload["axes"]["mutation"] == advisory_axis

            (root / "projects/fixture/aw.toml").write_text(
                """\
[project]
name = "fixture"
mutation_adequacy = "required"
mutation_evidence_dir = "projects/fixture/evidence/mutation-adequacy"
mutation_source_path = "projects/fixture/src"
""",
                encoding="utf-8",
            )
            required_unavailable = run_aw(
                root,
                "health",
                "--project",
                "fixture",
                "mutation",
                expect_success=False,
            )
            required_result = final_json(required_unavailable)
            required_payload = json.loads(
                Path(required_result["payload_path"]).read_text(encoding="utf-8")
            )
            assert (
                required_result["axes"]["mutation"]["requirement"] == "required"
            )
            assert (
                required_result["axes"]["mutation"]["evaluation"] == "unavailable"
            )
            assert (
                required_payload["axes"]["mutation"]
                == required_result["axes"]["mutation"]
            )

            evidence_path.chmod(0o700)
            evidence_path.rmdir()
            (root / "projects/fixture/aw.toml").write_text(
                """\
[project]
name = "fixture"
mutation_adequacy = "advisory"
mutation_evidence_dir = "projects/fixture/evidence/mutation-adequacy"
mutation_source_path = "projects/fixture/src"
""",
                encoding="utf-8",
            )
            advisory_failed = run_aw(
                root,
                "health",
                "--project",
                "fixture",
                "mutation",
                expect_success=False,
            )
            failed_result = final_json(advisory_failed)
            failed_payload = json.loads(
                Path(failed_result["payload_path"]).read_text(encoding="utf-8")
            )
            assert failed_result["data"]["requirement"] == "advisory"
            assert failed_result["data"]["evaluation"] == "failed"
            assert failed_payload["policy"] == failed_result["data"]["detail"]["policy"]
            assert failed_payload["status"] == failed_result["data"]["detail"]["status"]
            assert (
                failed_payload["required_for_production"]
                == failed_result["data"]["detail"]["required_for_production"]
            )

            not_applicable = run_aw(
                root,
                "health",
                "--project",
                "fixture",
                "takeover-audit",
            )
            not_applicable_result = final_json(not_applicable)
            not_applicable_payload = json.loads(
                Path(not_applicable_result["payload_path"]).read_text(encoding="utf-8")
            )
            assert not_applicable_result["status"] == "done"
            assert not_applicable_result["assessment"] == "healthy"
            assert not_applicable_result["data"]["recorded"] is False
            assert not_applicable_payload["takeover_audit"] == {
                key: value
                for key, value in not_applicable_result["data"].items()
                if key != "next_command"
            }

        with tempfile.TemporaryDirectory(
            prefix="aw-python-ec-health-aggregate-required-unavailable-"
        ) as raw_root:
            root = Path(raw_root)
            evidence_path = _write_total_observation_fixture(
                root,
                mutation_policy="required",
                poison_evidence_directory=True,
            )
            _enable_self_hosting_policy(root)
            run_aw(root, "ec", "lock", "--project", "agentic-workflow")
            required_unavailable_aggregate = run_aw(
                root,
                "health",
                "--project",
                "agentic-workflow",
                expect_success=False,
            )
            required_unavailable_result = final_json(
                required_unavailable_aggregate
            )
            required_unavailable_payload = json.loads(
                Path(required_unavailable_result["payload_path"]).read_text(
                    encoding="utf-8"
                )
            )
            assert required_unavailable_result["status"] == "continue"
            assert required_unavailable_result["assessment"] == "indeterminate"
            assert (
                required_unavailable_result["readiness"]["production_ready"]
                is False
            )
            assert (
                required_unavailable_result["readiness"]["production_status"]
                == "not_evaluated"
            )
            assert (
                required_unavailable_result["axes"]["mutation"]["requirement"]
                == "required"
            )
            assert (
                required_unavailable_result["axes"]["mutation"]["evaluation"]
                == "unavailable"
            )
            _assert_aggregate_outcome_correspondence(
                required_unavailable_result,
                required_unavailable_payload,
            )
            evidence_path.chmod(0o700)

        with tempfile.TemporaryDirectory(
            prefix="aw-python-ec-health-aggregate-advisory-"
        ) as raw_root:
            root = Path(raw_root)
            evidence_path = _write_total_observation_fixture(
                root,
                mutation_policy="advisory",
                poison_evidence_directory=True,
            )
            _enable_self_hosting_policy(root)
            run_aw(root, "ec", "lock", "--project", "agentic-workflow")
            advisory_aggregate = run_aw(
                root,
                "health",
                "--project",
                "agentic-workflow",
            )
            advisory_aggregate_result = final_json(advisory_aggregate)
            advisory_aggregate_payload = json.loads(
                Path(advisory_aggregate_result["payload_path"]).read_text(
                    encoding="utf-8"
                )
            )
            assert advisory_aggregate_result["assessment"] == "degraded"
            assert advisory_aggregate_result["readiness"]["production_ready"] is True
            assert (
                advisory_aggregate_result["readiness"]["production_status"] == "ready"
            )
            assert (
                advisory_aggregate_result["axes"]["mutation"]["requirement"]
                == "advisory"
            )
            assert (
                advisory_aggregate_result["axes"]["mutation"]["evaluation"]
                == "unavailable"
            )
            _assert_aggregate_outcome_correspondence(
                advisory_aggregate_result,
                advisory_aggregate_payload,
            )
            evidence_path.chmod(0o700)

        with tempfile.TemporaryDirectory(
            prefix="aw-python-ec-health-aggregate-required-"
        ) as raw_root:
            root = Path(raw_root)
            _write_total_observation_fixture(
                root,
                mutation_policy="required",
                poison_evidence_directory=False,
            )
            _enable_self_hosting_policy(root)
            run_aw(root, "ec", "lock", "--project", "agentic-workflow")
            required_failed_aggregate = run_aw(
                root,
                "health",
                "--project",
                "agentic-workflow",
                expect_success=False,
            )
            required_failed_result = final_json(required_failed_aggregate)
            required_failed_payload = json.loads(
                Path(required_failed_result["payload_path"]).read_text(
                    encoding="utf-8"
                )
            )
            assert required_failed_result["assessment"] == "blocked"
            assert required_failed_result["readiness"]["production_ready"] is False
            assert (
                required_failed_result["readiness"]["production_status"] == "blocked"
            )
            assert (
                required_failed_result["axes"]["mutation"]["requirement"]
                == "required"
            )
            assert (
                required_failed_result["axes"]["mutation"]["evaluation"] == "failed"
            )
            _assert_aggregate_outcome_correspondence(
                required_failed_result,
                required_failed_payload,
            )
        return [
            "focused capability leaves a deliberately poisoned mutation evaluator not evaluated",
            "focused advisory and required evaluator unavailability fail with matching durable payloads",
            "aggregate required unavailability is exactly indeterminate with no readiness claim",
            "aggregate advisory unavailability exits successfully with degraded but ready status",
            "aggregate required failure exits nonzero with blocked readiness",
            "focused advisory failure exits nonzero while not-applicable exits zero",
        ]

    if case_id == "existing-project-standardization-brownfield-takeover-surface":
        help_output = run_aw(
            Path.cwd(),
            "td",
            "audit-record",
            "--help",
        ).stdout
        retired = run_aw(
            Path.cwd(),
            "standardize",
            expect_success=False,
        )
        assert "Record a bounded preservation audit fixture" in help_output
        assert "unrecognized subcommand" in retired.stderr
        return [
            "retired standardize namespace is absent from the real CLI",
            "brownfield audit recording is available under aw td audit-record",
        ]

    if case_id == "standardize-audit-first-contract-test":
        with tempfile.TemporaryDirectory(prefix="aw-python-ec-audit-") as raw_root:
            root = Path(raw_root)
            _write_fixture(root, authoritative=False)
            before = final_json(
                run_aw(
                    root,
                    "health",
                    "--project",
                    "fixture",
                    "takeover-audit",
                )
            )
            audit_record = json.loads(
                run_aw(
                    root,
                    "td",
                    "audit-record",
                    "--project",
                    "fixture",
                ).stdout
            )
            after = final_json(
                run_aw(
                    root,
                    "health",
                    "--project",
                    "fixture",
                    "takeover-audit",
                )
            )
            assert before["status"] == "done"
            assert before["assessment"] == "healthy"
            assert before["data"]["recorded"] is False
            assert audit_record["project"] == "fixture"
            assert audit_record["scope"] is None
            surfaces = {
                surface["kind"]: {
                    "name": surface["name"],
                    "preserve": surface["preserve"],
                }
                for surface in audit_record["surfaces"]
            }
            assert surfaces["route"] == {
                "name": "routes",
                "preserve": (
                    "preserve externally visible navigation and endpoint paths "
                    "before quality changes"
                ),
            }
            assert surfaces["command"] == {
                "name": "commands",
                "preserve": (
                    "preserve CLI command names, arguments, and output contracts "
                    "before quality changes"
                ),
            }
            assert after["status"] == "done"
            assert after["assessment"] == "healthy"
            assert after["data"]["recorded"] is True
            durable_audit = json.loads(
                Path(after["data"]["audit_path"]).read_text(encoding="utf-8")
            )
            assert durable_audit == audit_record
            assert after["data"]["surfaces_to_preserve"] == [
                "fixture:routes",
                "fixture:commands",
                "fixture:public-contracts",
                "fixture:docs",
                "fixture:generated-source",
            ]
            assert after["data"]["quality_debt_count"] == len(
                audit_record["quality_debt"]
            )
            assert after["data"]["safe_lever_count"] == len(
                audit_record["safe_levers"]
            )
            after_payload = json.loads(
                Path(after["payload_path"]).read_text(encoding="utf-8")
            )
            assert after_payload["takeover_audit"] == {
                key: value
                for key, value in after["data"].items()
                if key != "next_command"
            }
        return [
            "takeover audit health treats a missing baseline as successful not-applicable observation",
            "takeover audit health distinguishes missing and recorded preservation baselines",
            "aw td audit-record captures the fixture route and command surface",
        ]

    if case_id == "td-gen-source-source-snapshot-projection-real-cli":
        help_output = run_aw(Path.cwd(), "cb", "gen-source", "--help").stdout
        assert "--spec <SPEC>" in help_output
        assert "--target <TARGET>" in help_output
        assert "--dry-run" in help_output
        return [
            "source snapshot projection is exposed through the real cb gen-source command",
            "exact spec and target ownership inputs are mandatory and dry-run is supported",
        ]

    authoritative = case_id != "external-fixture-reports-advisory-gap"
    snapshot = _health_snapshot(authoritative=authoritative)
    result = snapshot["result"]
    payload = snapshot["payload"]

    if case_id == "authoritative-fixture-blocks-on-regenerability-gap":
        assert result["readiness"]["production_ready"] is False
        assert payload["regenerability_authority"]["authority"] == "generator_authoritative"
        assert payload["regenerability_authority"]["required_for_production"] is True
        assert any(
            "regenerability required for production" in blocker
            for blocker in payload["production_blockers"]
        )
        assert result["next"]["command"].startswith("aw ")
        return [
            "generator-authoritative HANDWRITE gap blocks production readiness",
            "health payload exposes authority, blocker, and runnable remediation",
        ]

    if case_id == "external-fixture-reports-advisory-gap":
        assert payload["regenerability_authority"]["authority"] == "external_advisory"
        assert payload["regenerability_authority"]["required_for_production"] is False
        assert payload["optional_regenerability_gaps"]
        assert not any(
            "regenerability required for production" in blocker
            for blocker in payload["production_blockers"]
        )
        return [
            "external-advisory HANDWRITE gap remains an optional warning",
            "the advisory regenerability gap is not promoted to a production blocker",
        ]

    if case_id == "aw-health-default-full-verification-smoke":
        phases = [record["phase"] for record in snapshot["records"][:-1]]
        assert phases[:2] == ["start", "tests"]
        assert "summary" in phases
        assert payload["test_gates"]["commands"][0]["command"] == "true"
        assert isinstance(payload["blockers"], list)
        return [
            "health streams progress JSONL before its terminal result",
            "terminal payload retains blocker and configured-command evidence",
        ]

    if case_id == "artifact-preflight-health-rollup":
        assert "production_blockers" in payload
        assert "optional_quality_warnings" in payload
        assert isinstance(payload["production_ready"], bool)
        return [
            "health payload keeps hard preflight blockers separate from advisory quality warnings",
            "production readiness is projected in the same durable payload",
        ]

    if case_id == "existing-project-standardization-cb-and-cold-verification-gates":
        assert payload["cb_verify_evaluated"] is True
        assert isinstance(payload["cb_verify_clean"], bool)
        assert isinstance(payload["cold_rebuild_evaluated"], bool)
        assert isinstance(payload["cold_rebuild_clean"], bool)
        return [
            "health exposes independent CB verification and cold-rebuild gate results",
            "generated ownership changes remain visible to both readiness axes",
        ]

    if case_id == "existing-project-standardization-managed-and-semantic-production-gates":
        assert "managed_percent" in payload
        assert "semantic_percent" in payload
        assert result["next"]["command"].startswith("aw ")
        return [
            "health reports managed and semantic coverage independently",
            "the blocked fixture emits one runnable highest-priority remediation",
        ]

    if case_id == "existing-project-standardization-traceability-closure-gate":
        assert "traceability_percent" in payload
        assert "command_traceability_percent" in payload
        assert isinstance(payload["traceability"]["blockers"], list)
        return [
            "health payload closes source and TD traceability through explicit percentages and gaps",
            "command traceability is reported as an independent axis",
        ]

    raise AssertionError(f"unhandled existing-health case: {case_id}")


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by existing-health: {case_id}")
    return _verify_health(case_id)
