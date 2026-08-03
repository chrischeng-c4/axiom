"""Black-box contract for claim-closure validators and the claim reconciliation producer."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

from wi_contract_fixture import (
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "capability-control-plane-capability-catalog-and-td-claim-linkage-consistency"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-catalog-and-td-claim-linkage-consistency"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-capability-catalog-and-td-claim-linkage-consistency"
)
ASSERTIONS = (
    "a production EC case with no capability_id is rejected as unmapped",
    "a production EC case referencing an unknown capability is rejected by id",
    "a production EC case referencing an unknown claim under a known capability is rejected by id",
    "a production EC case that correctly names a real capability and claim becomes that claim's ec_case_ids evidence",
    "the read-only claim reconciliation producer independently reports supplemental drift evidence over the real inventory rather than acting as claim closure's sole oracle",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated claim-closure linkage fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Claim-closure linkage fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw health --project demo claims` - reports claim closure evidence.
EC Dimensions: behavior: `true` - isolated black-box claim-linkage contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one claim used only to prove claim-closure linkage validation.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo coverage | change | - | implemented | verified | smoke | `true` |
"""


def _write_fixture(root: Path) -> None:
    project = root / "project"
    td_root = project / "tech-design"
    ec_root = project / "external-contracts"
    (root / ".git").mkdir(exist_ok=True)
    (td_root / "src/demo/public_contracts").mkdir(parents=True, exist_ok=True)
    (ec_root / "src/cases").mkdir(parents=True, exist_ok=True)
    (ec_root / "evidence").mkdir(exist_ok=True)
    (root / "aw.toml").write_text(
        """version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "project"
td_path = "project/tech-design"
cap_path = "project/CAPABILITIES.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["project/**"]
target = "python"
test_cmd = "true"
""",
        encoding="utf-8",
    )
    (project / "CAPABILITIES.md").write_text(_capability_document(), encoding="utf-8")
    (td_root / "pyproject.toml").write_text(
        """[project]
name = "demo-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    (td_root / "src/demo/public_contracts/claim.py").write_text(
        '''__aw_artifact_id__ = "artifact:demo/claim"
__aw_public_contract__ = True


def demo_claim() -> str:
    return "Demo claim"
''',
        encoding="utf-8",
    )
    (ec_root / "src/runner.py").write_text(
        'print("fixture runner")\n',
        encoding="utf-8",
    )
    (ec_root / "src/cases/claim.py").write_text(
        'def verify() -> list[str]:\n    return ["claim is externally observable"]\n',
        encoding="utf-8",
    )
    (ec_root / "pyproject.toml").write_text(
        """[project]
name = "demo-external-contracts"
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
author = "fixture:external"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "demo-mapped-case"
artifact_id = "artifact:demo/claim"
capability_id = "demo-capability"
use_case_id = "demo-coverage"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "the correctly mapped case becomes claim evidence"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unmapped-case"
artifact_id = "artifact:demo/claim"
capability_id = "unmapped"
use_case_id = "n-a"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "an unmapped case must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unknown-capability-case"
artifact_id = "artifact:demo/claim"
capability_id = "nonexistent-capability"
use_case_id = "n-a"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "a case naming an unknown capability must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unknown-claim-case"
artifact_id = "artifact:demo/claim"
capability_id = "demo-capability"
use_case_id = "nonexistent-claim"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "a case naming an unknown claim under a known capability must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "claim")


def _health_claims(root: Path) -> dict[str, object]:
    completed = subprocess.run(
        [str(_aw_binary()), "health", "--project", "demo", "claims"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def _claim_reconciliation_report(
    *,
    inventory_path: Path | None = None,
    expected_mapping_path: Path | None = None,
) -> dict[str, object]:
    script = (
        _repository_root()
        / "apps/agentic-workflow/external-contracts/src/claim_reconciliation.py"
    )
    args = [sys.executable, str(script)]
    if inventory_path is not None:
        args.extend(["--inventory", str(inventory_path)])
    if expected_mapping_path is not None:
        args.extend(["--expected-mapping", str(expected_mapping_path)])
    completed = subprocess.run(
        args,
        cwd=_repository_root(),
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def _mapping_record(mapping: tuple[str, str, str, str]) -> dict[str, str]:
    case_id, capability_id, use_case_id, dimension = mapping
    return {
        "case_id": case_id,
        "capability_id": capability_id,
        "use_case_id": use_case_id,
        "dimension": dimension,
    }


def _inventory_mapping(path: Path) -> list[tuple[str, str, str, str]]:
    document = tomllib.loads(path.read_text(encoding="utf-8"))
    cases = document["tool"]["aw"]["python-ec"]["cases"]
    return [
        (
            str(case["id"]),
            str(case["capability_id"]),
            str(case["use_case_id"]),
            str(case["dimension"]),
        )
        for case in cases
    ]


def _inventory_case_blocks(text: str) -> tuple[str, list[str]]:
    header = "[[tool.aw.python-ec.cases]]"
    prefix, *blocks = text.split(header)
    return prefix, [header + block for block in blocks]


def _write_case_blocks(path: Path, prefix: str, blocks: list[str]) -> None:
    path.write_text(prefix + "".join(blocks), encoding="utf-8")


def _claim(data: dict[str, object]) -> dict[str, object]:
    return next(
        claim
        for claim in data["claims"]
        if claim["capability_id"] == "demo-capability"
        and claim["claim_id"] == "demo-coverage"
    )


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-python-claim-catalog-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root)
        payload = _health_claims(root)
        data = payload["data"]
        blockers = data["blockers"]
        claims = data["claims"]

        assert (
            "claim closure EC case `demo-unmapped-case` is unmapped; "
            "production cases must name capability_id and claim_id" in blockers
        ), blockers
        assert (
            "claim closure EC case `demo-unknown-capability-case` references "
            "unknown capability `nonexistent-capability`" in blockers
        ), blockers
        assert (
            "claim closure EC case `demo-unknown-claim-case` references unknown "
            "claim `nonexistent-claim` for capability `demo-capability`" in blockers
        ), blockers
        assert not any(
            "demo-mapped-case" in blocker
            and ("is unmapped" in blocker or "unknown" in blocker)
            for blocker in blockers
        ), blockers

        entry = next(
            claim
            for claim in claims
            if claim["capability_id"] == "demo-capability"
            and claim["claim_id"] == "demo-coverage"
        )
        assert entry["ec_case_ids"] == ["demo-mapped-case"], entry

        inventory_path = root / "project/external-contracts/pyproject.toml"
        inventory_text = inventory_path.read_text(encoding="utf-8")
        prefix, blocks = _inventory_case_blocks(inventory_text)
        mapped_block = next(
            block for block in blocks if 'id = "demo-mapped-case"' in block
        )

        # Missing: removing the sole exact mapping leaves the claim with no
        # case evidence and emits the precise missing-production-case drift.
        _write_case_blocks(
            inventory_path,
            prefix,
            [block for block in blocks if block != mapped_block],
        )
        missing_data = _health_claims(root)["data"]
        missing_claim = _claim(missing_data)
        assert missing_claim["ec_case_ids"] == [], missing_data
        assert missing_claim["blockers"][0] == (
            "missing required production EC case"
        ), missing_data

        # Duplicate: an independent inventory read proves the exact duplicate
        # identity, while the production claim scanner fails closed instead of
        # returning a misleading partial mapping.
        _write_case_blocks(inventory_path, prefix, [*blocks, mapped_block])
        duplicate_mapping = _inventory_mapping(inventory_path)
        duplicate_ids = [case_id for case_id, *_ in duplicate_mapping]
        assert duplicate_ids.count("demo-mapped-case") == 2, duplicate_mapping
        duplicate_data = _health_claims(root)["data"]
        assert duplicate_data["claims"] == [], duplicate_data
        assert duplicate_data["blockers"] == [
            "claim closure unavailable: failed to scan TD capability_refs"
        ], duplicate_data

        # Misbound: retaining the case identity but changing its claim binding
        # must name the exact bad edge and leave the real claim unbound.
        misbound_blocks = [
            block.replace(
                'use_case_id = "demo-coverage"',
                'use_case_id = "wrong-claim"',
            )
            if block == mapped_block
            else block
            for block in blocks
        ]
        _write_case_blocks(inventory_path, prefix, misbound_blocks)
        misbound_data = _health_claims(root)["data"]
        assert (
            "claim closure EC case `demo-mapped-case` references unknown claim "
            "`wrong-claim` for capability `demo-capability`"
            in misbound_data["blockers"]
        ), misbound_data
        misbound_claim = _claim(misbound_data)
        assert misbound_claim["ec_case_ids"] == [], misbound_data
        assert misbound_claim["blockers"][0] == (
            "missing required production EC case"
        ), misbound_data

    reconciliation = _claim_reconciliation_report()
    assert reconciliation["schema_version"] == (
        "aw.python-ec.claim-reconciliation.v2"
    ), reconciliation
    assert reconciliation["status"] == "clean", reconciliation
    canonical_mapping = _inventory_mapping(
        _repository_root() / "apps/agentic-workflow/external-contracts/pyproject.toml"
    )
    canonical_ids = [case_id for case_id, *_ in canonical_mapping]
    assert len(canonical_ids) == len(set(canonical_ids)), canonical_mapping
    assert reconciliation["case_count"] == len(canonical_mapping), reconciliation
    assert {
        mapping
        for mapping in canonical_mapping
        if mapping[0].startswith("capability-control-plane-")
        and mapping[2]
        in {
            "capability-project-sweep",
            "agent-facing-dx-baseline-trait",
            "one-way-wi-reference-direction",
            "default-cap-path-flips-to-capabilities-md",
            "capability-catalog-and-td-claim-linkage-consistency",
            "python-artifact-readiness",
        }
    } == {
        (
            "capability-control-plane-capability-project-sweep",
            "capability-control-plane",
            "capability-project-sweep",
            "behavior",
        ),
        (
            "capability-control-plane-agent-facing-dx-baseline-trait",
            "capability-control-plane",
            "agent-facing-dx-baseline-trait",
            "behavior",
        ),
        (
            "capability-control-plane-one-way-wi-reference-direction",
            "capability-control-plane",
            "one-way-wi-reference-direction",
            "behavior",
        ),
        (
            "capability-control-plane-default-cap-path-flips-to-capabilities-md",
            "capability-control-plane",
            "default-cap-path-flips-to-capabilities-md",
            "behavior",
        ),
        (
            "capability-control-plane-capability-catalog-and-td-claim-linkage-consistency",
            "capability-control-plane",
            "capability-catalog-and-td-claim-linkage-consistency",
            "behavior",
        ),
        (
            "capability-control-plane-python-artifact-readiness",
            "capability-control-plane",
            "python-artifact-readiness",
            "behavior",
        ),
    }, canonical_mapping
    assert reconciliation["next"] is None, reconciliation

    # Exercise the producer itself against an independent frozen mapping and
    # copied candidate inventories. The Rust health checks above remain the
    # primary claim-closure oracle; this producer is supplemental exact drift
    # evidence and must never certify its own live input by fiat.
    with tempfile.TemporaryDirectory(prefix="aw-claim-reconciliation-copy-") as raw_tmp:
        temp_root = Path(raw_tmp)
        inventory_path = temp_root / "candidate-pyproject.toml"
        expected_mapping_path = (
            _repository_root()
            / "apps/agentic-workflow/external-contracts/fixtures/claim-reconciliation/capability-catalog-td-claim-linkage-expected-mapping.json"
        )
        canonical_inventory_path = (
            _repository_root()
            / "apps/agentic-workflow/external-contracts/pyproject.toml"
        )
        canonical_inventory = canonical_inventory_path.read_text(encoding="utf-8")
        expected_doc = json.loads(expected_mapping_path.read_text(encoding="utf-8"))
        assert expected_doc["schema_version"] == "aw.python-ec.expected-mapping.v1"
        expected_records = expected_doc["mappings"]
        assert len(expected_records) == 110

        inventory_path.write_text(canonical_inventory, encoding="utf-8")
        copied_clean = _claim_reconciliation_report(
            inventory_path=inventory_path,
            expected_mapping_path=expected_mapping_path,
        )
        assert copied_clean["status"] == "clean", copied_clean
        assert copied_clean["case_count"] == len(expected_records), copied_clean
        assert copied_clean["case_mapping"] == expected_records, copied_clean
        assert copied_clean["findings"]["missing_expected_mappings"] == [], (
            copied_clean
        )
        assert copied_clean["findings"]["unexpected_mappings"] == [], copied_clean
        assert copied_clean["findings"]["duplicate_case_ids"] == [], copied_clean

        prefix, blocks = _inventory_case_blocks(canonical_inventory)
        target_id = "capability-control-plane-capability-project-sweep"
        target_block = next(
            block for block in blocks if f'id = "{target_id}"' in block
        )
        target_mapping = next(
            record for record in expected_records if record["case_id"] == target_id
        )

        _write_case_blocks(
            inventory_path,
            prefix,
            [block for block in blocks if block != target_block],
        )
        missing = _claim_reconciliation_report(
            inventory_path=inventory_path,
            expected_mapping_path=expected_mapping_path,
        )
        assert missing["status"] == "drifted", missing
        assert missing["findings"]["missing_expected_mappings"] == [
            target_mapping
        ], missing
        assert missing["findings"]["unexpected_mappings"] == [], missing
        assert missing["findings"]["duplicate_case_ids"] == [], missing

        _write_case_blocks(inventory_path, prefix, [*blocks, target_block])
        duplicate = _claim_reconciliation_report(
            inventory_path=inventory_path,
            expected_mapping_path=expected_mapping_path,
        )
        assert duplicate["status"] == "drifted", duplicate
        assert duplicate["findings"]["missing_expected_mappings"] == [], duplicate
        assert duplicate["findings"]["unexpected_mappings"] == [], duplicate
        assert duplicate["findings"]["duplicate_case_ids"] == [target_id], duplicate

        misbound_block = target_block.replace(
            'use_case_id = "capability-project-sweep"',
            'use_case_id = "wrong-claim"',
        )
        _write_case_blocks(
            inventory_path,
            prefix,
            [misbound_block if block == target_block else block for block in blocks],
        )
        misbound = _claim_reconciliation_report(
            inventory_path=inventory_path,
            expected_mapping_path=expected_mapping_path,
        )
        unexpected_mapping = dict(target_mapping)
        unexpected_mapping["use_case_id"] = "wrong-claim"
        assert misbound["status"] == "drifted", misbound
        assert misbound["findings"]["missing_expected_mappings"] == [
            target_mapping
        ], misbound
        assert misbound["findings"]["unexpected_mappings"] == [
            unexpected_mapping
        ], misbound
        assert misbound["findings"]["duplicate_case_ids"] == [], misbound

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
