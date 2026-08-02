"Tech design for WI #3341: scope Python CB code checks to declared WI targets.\n\n@spec #3341"

from __future__ import annotations

from pathlib import PurePosixPath


__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/scope-python-cb-code-checks-to-declared-wi-targets"
__aw_work_item__ = "3341"
__aw_changes__ = [
    {
        "path": "apps/agentic-workflow/src/services/python_artifact_code_check.rs",
        "action": "modify",
        "description": (
            "Scope slug-driven Python artifact verification to the completing WI's "
            "declared TD/CB target paths while preserving full-project aggregate "
            "verification when no completing slug is supplied."
        ),
    },
    {
        "path": "apps/agentic-workflow/src/cli/cb.rs",
        "action": "modify",
        "description": (
            "Wire the real `aw cb check <slug>` terminal path to pass the slug's "
            "resolved declared scope into Python artifact code-check selection."
        ),
    },
    {
        "path": (
            "apps/agentic-workflow/external-contracts/src/cases/"
            "capability-control-plane-python-artifact-readiness.py"
        ),
        "action": "modify",
        "description": (
            "Add a black-box regression that demonstrates scoped `aw cb check <slug>` "
            "behavior for pure EC targets versus aggregate no-slug/full-project checks."
        ),
    },
]


def _assert_relative_repo_path(path: str) -> None:
    pure = PurePosixPath(path)
    assert not pure.is_absolute(), path
    assert ".." not in pure.parts, path


def design_contract() -> str:
    """Executable boundary assertions for WI #3341."""

    # Frozen decision D1: completing-slug CB checks are scoped to declared targets.
    slug_scope_targets = (
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-control-plane-python-artifact-readiness.py",
    )
    assert slug_scope_targets == (
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-control-plane-python-artifact-readiness.py",
    )

    # Frozen decision D2: pure EC slug keeps lock + own identity/target checks,
    # and skips unrelated native workspace ownership/cold/test requirements.
    pure_ec_slug_expected_checks = {
        "td_lock_check": True,
        "ec_lock_check": True,
        "current_td_to_ec_identity_target_edge": True,
        "unrelated_native_workspace_ownership_required": False,
        "unrelated_native_workspace_cold_parity_required": False,
        "unrelated_native_workspace_tests_required": False,
    }
    assert pure_ec_slug_expected_checks["td_lock_check"]
    assert pure_ec_slug_expected_checks["ec_lock_check"]
    assert pure_ec_slug_expected_checks["current_td_to_ec_identity_target_edge"]
    assert not pure_ec_slug_expected_checks["unrelated_native_workspace_ownership_required"]
    assert not pure_ec_slug_expected_checks["unrelated_native_workspace_cold_parity_required"]
    assert not pure_ec_slug_expected_checks["unrelated_native_workspace_tests_required"]

    # Frozen decision D3: selected native workspace remains fail-closed.
    native_target_failure_modes = {
        "unresolved_ownership": "fail",
        "cold_drift": "fail",
        "configured_native_test_failure": "fail",
    }
    assert set(native_target_failure_modes) == {
        "unresolved_ownership",
        "cold_drift",
        "configured_native_test_failure",
    }
    assert all(outcome == "fail" for outcome in native_target_failure_modes.values())

    # Frozen decision D4: no-slug/full-project health remains aggregate.
    no_slug_scope_mode = "aggregate_full_project"
    assert no_slug_scope_mode == "aggregate_full_project"

    # Frozen decision D5: reuse existing safe parsing/matching surfaces.
    required_reuse = {
        "safe_td_cb_change_path_parsing": True,
        "configured_workspace_matching": True,
        "parse_executable_python": False,
        "infer_ownership_from_broad_glob": False,
    }
    assert required_reuse["safe_td_cb_change_path_parsing"]
    assert required_reuse["configured_workspace_matching"]
    assert not required_reuse["parse_executable_python"]
    assert not required_reuse["infer_ownership_from_broad_glob"]

    # Frozen decision D6: public contract must cover real `aw cb check <slug>`.
    public_contract_expectation = {
        "exercise_real_cb_check_slug_branch": True,
        "distinguish_pure_ec_scoped_slug_from_no_slug_aggregate_check": True,
        "rust_unit_tests_only_is_insufficient": True,
    }
    assert public_contract_expectation["exercise_real_cb_check_slug_branch"]
    assert public_contract_expectation[
        "distinguish_pure_ec_scoped_slug_from_no_slug_aggregate_check"
    ]
    assert public_contract_expectation["rust_unit_tests_only_is_insufficient"]

    # Bounded change surface assertions.
    expected_paths = tuple(change["path"] for change in __aw_changes__)
    assert expected_paths == (
        "apps/agentic-workflow/src/services/python_artifact_code_check.rs",
        "apps/agentic-workflow/src/cli/cb.rs",
        (
            "apps/agentic-workflow/external-contracts/src/cases/"
            "capability-control-plane-python-artifact-readiness.py"
        ),
    )
    for path in expected_paths:
        _assert_relative_repo_path(path)

    return "ok"
