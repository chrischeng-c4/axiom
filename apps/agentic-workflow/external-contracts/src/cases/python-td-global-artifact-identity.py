"""Native Python EC for globally unique Python TD artifact identities."""

from __future__ import annotations

import json
import tempfile
from pathlib import Path

from wi_contract_fixture import run_aw


CASE_ID = "python-td-global-artifact-identity"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "global-python-td-artifact-identity"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case python-td-global-artifact-identity"
)
ASSERTIONS = (
    "the whole Agentic Workflow Python TD project has globally unique module and artifact identities",
    "every design module outside the shrinking legacy allowlist declares an explicit identity",
    "the two normalized logic-emitter paths retain exact role-distinct identities",
    "duplicate project artifact IDs emit a complete stable sorted diagnostic",
    "the typed mutation inventory is unique, reproducible, and covers both logic emitters",
)

# Lower bounds, not snapshots: they prove the case read the real whole-project IR
# instead of an empty or truncated one, and they do not need editing every time a
# design module or mutation descriptor is added.
MINIMUM_MODULE_COUNT = 1000
MINIMUM_MUTATION_COUNT = 9000

# A package marker and a unit test carry no design identity by construction.
IDENTITY_EXEMPT_BASENAMES = {"__init__.py"}
IDENTITY_EXEMPT_PREFIXES = ("tests/",)

# Legacy design modules that still lack an explicit identity. This allowlist may
# only shrink: a newly added design module without `__aw_artifact_id__` fails.
LEGACY_UNIDENTIFIED_DESIGN_MODULES = {
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
    assert len(modules) >= MINIMUM_MODULE_COUNT, len(modules)
    module_ids = [module["id"] for module in ir["modules"]]
    artifact_ids = [
        module["artifact_id"]
        for module in modules
        if module.get("artifact_id") is not None
    ]
    unidentified_design_modules = {
        module["path"]
        for module in modules
        if module.get("artifact_id") is None
        and module["path"].rsplit("/", 1)[-1] not in IDENTITY_EXEMPT_BASENAMES
        and not module["path"].startswith(IDENTITY_EXEMPT_PREFIXES)
    }
    assert unidentified_design_modules <= LEGACY_UNIDENTIFIED_DESIGN_MODULES, sorted(
        unidentified_design_modules - LEGACY_UNIDENTIFIED_DESIGN_MODULES
    )
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
    assert len(mutations) >= MINIMUM_MUTATION_COUNT, len(mutations)
    mutation_ids = [mutation["id"] for mutation in mutations]
    assert len(mutation_ids) == len(set(mutation_ids))
    # Reproducibility is proven by re-deriving the inventory rather than by
    # comparing against a digest that has to be re-baselined on every TD edit.
    repeated_projection = json.loads(
        run_aw(Path.cwd(), "td", "ast", td_root, "--mutations").stdout
    )
    repeated_ids = [mutation["id"] for mutation in repeated_projection["mutations"]]
    assert repeated_ids == mutation_ids
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
    return list(ASSERTIONS)
