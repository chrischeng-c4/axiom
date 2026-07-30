"""Native Python EC for globally unique Python TD artifact identities."""

from __future__ import annotations

import hashlib
import json
import tempfile
from pathlib import Path

from wi_contract_fixture import run_aw


CASE_ID = "python-td-global-artifact-identity"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "global-python-td-artifact-identity"
DIMENSION = "behavior"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case python-td-global-artifact-identity"
ASSERTIONS = (
    "the complete 1002-module Agentic Workflow Python TD project has unique identities",
    "the two normalized logic-emitter paths retain exact role-distinct identities",
    "duplicate project artifact IDs emit a complete stable sorted diagnostic",
    "the exact typed mutation inventory has unique IDs and covers both logic emitters",
)

EXPECTED_MODULE_COUNT = 1002
EXPECTED_ARTIFACT_COUNT = 995
EXPECTED_MUTATION_COUNT = 8756
EXPECTED_MUTATION_ID_DIGEST = (
    "sha256:d730df1eeda12e477df35c67b643efcb1b9d207a6ff0f1c6465033d5f5c225d0"
)
EXPECTED_MISSING_ARTIFACT_PATHS = {
    "src/agentic_workflow/work_items/intake_health.py",
    "src/agentic_workflow/work_items/report_triage.py",
    "src/agentic_workflow/work_items/shared_cli_report.py",
    "src/agentic_workflow/work_items/spike_terminal.py",
    "src/agentic_workflow/work_items/taxonomy.py",
    "src/agentic_workflow/work_items/templates.py",
    "src/agentic_workflow/work_items/vocabulary.py",
}
EXPECTED_LOGIC_EMITTER_IDENTITIES = {
    "src/agentic_workflow/migrated/core/generate/gen/rust/logic_emitter_d49cdd8e.py":
        "artifact:core-generate/core-generate-gen-rust-logic-emitter",
    "src/agentic_workflow/migrated/core/generate/gen/rust/logic_emitter_339241df.py":
        "artifact:core-generate/core-generate-gen-rust-logic-emitter-generated-projection-339241df",
}


def verify() -> list[str]:
    td_root = "apps/agentic-workflow/tech-design"
    ast = run_aw(Path.cwd(), "td", "ast", td_root)
    ir = json.loads(ast.stdout)
    modules = ir["modules"]
    assert len(modules) == EXPECTED_MODULE_COUNT
    module_ids = [module["id"] for module in ir["modules"]]
    artifact_ids = [
        module["artifact_id"]
        for module in modules
        if module.get("artifact_id") is not None
    ]
    missing_artifact_paths = {
        module["path"] for module in modules if module.get("artifact_id") is None
    }
    assert len(artifact_ids) == EXPECTED_ARTIFACT_COUNT
    assert missing_artifact_paths == EXPECTED_MISSING_ARTIFACT_PATHS
    assert len(module_ids) == len(set(module_ids))
    assert len(artifact_ids) == len(set(artifact_ids))
    actual_logic_emitters = {
        module["path"]: module["artifact_id"]
        for module in modules
        if module["path"] in EXPECTED_LOGIC_EMITTER_IDENTITIES
    }
    assert actual_logic_emitters == EXPECTED_LOGIC_EMITTER_IDENTITIES

    with tempfile.TemporaryDirectory(prefix="aw-python-td-duplicate-id-") as raw_root:
        duplicate_root = Path(raw_root)
        (duplicate_root / "src").mkdir()
        (duplicate_root / "src/z_zeta.py").write_text(
            '__aw_artifact_id__ = "artifact:fixture/zeta"\n'
            "\n"
            "def zeta_projection() -> str:\n"
            '    return "z"\n',
            encoding="utf-8",
        )
        (duplicate_root / "src/a_zeta.py").write_text(
            '__aw_artifact_id__ = "artifact:fixture/zeta"\n'
            "\n"
            "def zeta_design() -> str:\n"
            '    return "a"\n',
            encoding="utf-8",
        )
        (duplicate_root / "src/y_alpha.py").write_text(
            '__aw_artifact_id__ = "artifact:fixture/alpha"\n'
            "\n"
            "def alpha_projection() -> str:\n"
            '    return "y"\n',
            encoding="utf-8",
        )
        (duplicate_root / "src/b_alpha.py").write_text(
            '__aw_artifact_id__ = "artifact:fixture/alpha"\n'
            "\n"
            "def alpha_design() -> str:\n"
            '    return "b"\n',
            encoding="utf-8",
        )
        expected_diagnostic = (
            "Error: Python TD diagnostic [duplicate-project-artifact-id]: "
            "every __aw_artifact_id__ must be globally unique; conflicts: "
            "`artifact:fixture/alpha`: src/b_alpha.py, src/y_alpha.py; "
            "`artifact:fixture/zeta`: src/a_zeta.py, src/z_zeta.py"
        )
        first_duplicate = run_aw(
            Path.cwd(),
            "td",
            "ast",
            str(duplicate_root),
            expect_success=False,
        )
        second_duplicate = run_aw(
            Path.cwd(),
            "td",
            "ast",
            str(duplicate_root),
            expect_success=False,
        )
        assert first_duplicate.stdout == ""
        assert first_duplicate.stderr.strip() == expected_diagnostic
        assert second_duplicate.stderr.strip() == expected_diagnostic

    mutation_inventory = run_aw(
        Path.cwd(),
        "td",
        "ast",
        td_root,
        "--mutations",
    )
    mutation_projection = json.loads(mutation_inventory.stdout)
    assert mutation_projection["mutation_schema"] == "aw.python-td-mutation.v1"
    mutations = mutation_projection["mutations"]
    assert len(mutations) == EXPECTED_MUTATION_COUNT
    mutation_ids = [mutation["id"] for mutation in mutations]
    assert len(mutation_ids) == len(set(mutation_ids))
    mutation_id_digest = "sha256:" + hashlib.sha256(
        "\n".join(sorted(mutation_ids)).encode()
    ).hexdigest()
    assert mutation_id_digest == EXPECTED_MUTATION_ID_DIGEST
    logic_emitter_mutations = {
        module_id: [
            mutation["id"]
            for mutation in mutations
            if mutation["module_id"] == module_id
        ]
        for module_id in EXPECTED_LOGIC_EMITTER_IDENTITIES.values()
    }
    assert {
        module_id: len(ids) for module_id, ids in logic_emitter_mutations.items()
    } == {
        module_id: 8 for module_id in EXPECTED_LOGIC_EMITTER_IDENTITIES.values()
    }
    assert all(
        len(ids) == len(set(ids)) for ids in logic_emitter_mutations.values()
    )
    return [
        "the canonical 1002-module Python TD IR has 995 unique explicit artifact identities",
        "the semantic and generated logic-emitter modules have exact role-distinct identities",
        "a duplicate fixture emits the exact complete sorted diagnostic twice",
        "the exact 8756-descriptor mutation inventory has unique IDs and covers both logic emitters",
    ]
