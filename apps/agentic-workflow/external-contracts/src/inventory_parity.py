"""Prove the canonical Python inventory preserves every legacy EC contract."""

from __future__ import annotations

import ast
import tomllib
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
PROJECT_ROOT = EC_ROOT.parent
RUNNER_PREFIX = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py --case "
)
OPERATIONAL_CONTRACTS = {
    "aw-core-client": (
        "aw-core-client-model-workitem-first-artifact-lifecycle",
        "agent-first-cli-product-model",
        "cargo test -p agentic-workflow --lib "
        "agent_first_product_contracts_reject_removed_architecture -- --nocapture",
    ),
    "capability-control-plane": (
        "capability-control-plane",
        "markdown-capability-schema",
        "cargo test -p agentic-workflow --lib markdown_capability_tables "
        "-- --nocapture",
    ),
    "existing-project-standardization": (
        "existing-project-standardization",
        "brownfield-takeover-surface",
        "cargo test -p agentic-workflow --test cli_tests "
        "standardize_subcommands_registered -- --nocapture",
    ),
    "manual-evidence-artifacts": (
        "manual-evidence-artifacts",
        "manual-runner-output-convention",
        "cargo test -p agentic-workflow --lib "
        "ec_doc_gen_writes_manual_from_inventory -- --nocapture",
    ),
    "project-local-td-and-ec-gates": (
        "project-local-td-and-ec-gates",
        "cb-generation-and-standardize-scan-defaults",
        "cargo test -p agentic-workflow --lib "
        "cb_gen_force_regen_defaults_td_root_to_project_tech_design "
        "-- --nocapture",
    ),
    "td-cb-lifecycle-automation": (
        "td-cb-lifecycle-automation",
        "td-lifecycle-dispatch",
        "cargo test -p agentic-workflow --lib "
        "td_branch_activation_only_uses_main -- --nocapture",
    ),
    "work-item-planning": (
        "work-item-planning",
        "epic-to-change-atomization",
        "cargo test -p agentic-workflow --lib "
        "prioritize_lanes_put_bounded_bug_in_ready_now -- --nocapture",
    ),
}
REQUIRED_OPERATIONAL_CASES = {
    f"{prefix}-operational-{dimension}"
    for prefix in OPERATIONAL_CONTRACTS
    for dimension in ("efficiency", "stability")
}
MIGRATED_LEGACY_OVERRIDES = {
    "ec-artifact-producer-cli-fixture": {
        "command": (
            "cargo test -p agentic-workflow --test ec_python_inventory_check "
            "ec_python_draft_creates_only_python_scaffold_and_checks_clean "
            "-- --nocapture"
        ),
        "assertions": (
            "aw ec draft creates the canonical Python pyproject, runner, and "
            "bounded case module without Markdown fallback",
            "draft emits aw.cli.v1 with the exact aw ec check continuation",
            "the generated Python inventory preserves the requested capability "
            "and passes structural check",
        ),
    },
    "external-fixture-reports-advisory-gap": {
        "command": (
            "cargo test -p agentic-workflow --test cli_tests "
            "regenerability_gaps_are_advisory_when_production_gates_clean "
            "-- --nocapture"
        ),
    },
}


def _case_constants(path: Path) -> dict[str, object]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    values: dict[str, object] = {}
    for node in tree.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Name):
            values[target.id] = ast.literal_eval(node.value)
    return values


def main() -> None:
    legacy_document = tomllib.loads(
        (PROJECT_ROOT / "aw.toml").read_text(encoding="utf-8")
    )
    python_document = tomllib.loads(
        (EC_ROOT / "pyproject.toml").read_text(encoding="utf-8")
    )
    legacy_cases = {
        case["id"]: case
        for case in legacy_document["aw"]["ec"]["generated"]["cases"]
    }
    python_cases = {
        case["id"]: case
        for case in python_document["tool"]["aw"]["python-ec"]["cases"]
    }
    assert legacy_cases.keys() <= python_cases.keys()
    assert python_cases.keys() - legacy_cases.keys() == REQUIRED_OPERATIONAL_CASES

    for case_id, legacy in legacy_cases.items():
        canonical = python_cases[case_id]
        migration_override = MIGRATED_LEGACY_OVERRIDES.get(case_id, {})
        expected_command = migration_override.get("command", legacy["command"])
        expected_assertions = migration_override.get(
            "assertions", tuple(legacy["assertions"])
        )
        assert canonical["capability_id"] == legacy["capability_id"]
        assert canonical["use_case_id"] == legacy["claim_id"]
        assert canonical["dimension"] == legacy["category"]
        assert canonical["promise"] == "; ".join(expected_assertions)
        assert canonical["target"] == "rust"

        command_prefix = f"{RUNNER_PREFIX}{case_id} -- "
        assert canonical["command"].startswith(command_prefix)
        assert canonical["command"][len(command_prefix) :] == expected_command

        source = _case_constants(EC_ROOT / canonical["test_path"])
        assert source["CASE_ID"] == case_id
        assert source["CAPABILITY_ID"] == legacy["capability_id"]
        assert source["USE_CASE_ID"] == legacy["claim_id"]
        assert source["DIMENSION"] == legacy["category"]
        assert source["LEGACY_TEST_PATH"] == legacy["test_path"]
        assert source["TARGET_COMMAND"] == expected_command
        assert source["ASSERTIONS"] == expected_assertions

    for prefix, (capability_id, use_case_id, target_command) in (
        OPERATIONAL_CONTRACTS.items()
    ):
        for dimension in ("efficiency", "stability"):
            case_id = f"{prefix}-operational-{dimension}"
            canonical = python_cases[case_id]
            assert canonical["capability_id"] == capability_id
            assert canonical["use_case_id"] == use_case_id
            assert canonical["dimension"] == dimension
            command_prefix = (
                f"{RUNNER_PREFIX}{case_id} --mode {dimension} "
                "--threshold-seconds 120 -- "
            )
            assert canonical["command"] == command_prefix + target_command

            source = _case_constants(EC_ROOT / canonical["test_path"])
            assert source["CASE_ID"] == case_id
            assert source["CAPABILITY_ID"] == capability_id
            assert source["USE_CASE_ID"] == use_case_id
            assert source["DIMENSION"] == dimension
            assert source["TARGET_COMMAND"] == target_command

    print(
        f"preserved {len(legacy_cases)} legacy case identities, applied "
        f"{len(MIGRATED_LEGACY_OVERRIDES)} explicit compatibility corrections, "
        f"and added {len(REQUIRED_OPERATIONAL_CASES)} required operational cases"
    )


if __name__ == "__main__":
    main()
