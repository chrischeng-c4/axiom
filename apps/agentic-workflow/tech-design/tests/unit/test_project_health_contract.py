"""Unit tests for the executable two-cell TD design."""

from __future__ import annotations

import sys
import unittest
import inspect
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from agentic_workflow.health.project_health_total_observation import (  # noqa: E402
    CellEvaluation,
    EcAcceptsTd,
    EcTdAlignment,
    HealthAssessment,
    SemanticHealth,
    aggregate_exit_code,
    reduce_health,
)
from agentic_workflow.public_contracts import (  # noqa: E402
    aw_core_client,
    capability_control_plane,
    existing_project_standardization,
    manual_evidence_artifacts,
    project_local_td_and_ec_gates,
    td_cb_lifecycle_automation,
    work_item_planning,
    workflow_root_runner,
)


PUBLIC_CONTRACTS = (
    aw_core_client,
    capability_control_plane,
    existing_project_standardization,
    manual_evidence_artifacts,
    project_local_td_and_ec_gates,
    td_cb_lifecycle_automation,
    work_item_planning,
    workflow_root_runner,
)


class ProjectHealthDesignTest(unittest.TestCase):
    def test_public_capability_contracts_are_executable_and_unique(self) -> None:
        artifact_ids = {
            module.__aw_artifact_id__
            for module in PUBLIC_CONTRACTS
        }
        self.assertEqual(len(artifact_ids), len(PUBLIC_CONTRACTS))
        for module in PUBLIC_CONTRACTS:
            self.assertIs(module.__aw_public_contract__, True)
            functions = [
                function
                for _, function in inspect.getmembers(module, inspect.isfunction)
                if function.__module__ == module.__name__
            ]
            self.assertTrue(functions, module.__name__)
            self.assertTrue(all(function() for function in functions))

    def test_both_green_is_healthy(self) -> None:
        health = SemanticHealth(
            ec_accepts_td=EcAcceptsTd(CellEvaluation.PASSED),
            ec_td_alignment=EcTdAlignment(),
        )

        self.assertEqual(reduce_health(health), HealthAssessment.HEALTHY)
        self.assertEqual(aggregate_exit_code(reduce_health(health)), 0)

    def test_ec_rejection_blocks(self) -> None:
        health = SemanticHealth(
            ec_accepts_td=EcAcceptsTd(CellEvaluation.FAILED, ("case failed",)),
            ec_td_alignment=EcTdAlignment(),
        )

        self.assertEqual(reduce_health(health), HealthAssessment.BLOCKED)

    def test_bidirectional_alignment_gap_blocks(self) -> None:
        health = SemanticHealth(
            ec_accepts_td=EcAcceptsTd(CellEvaluation.PASSED),
            ec_td_alignment=EcTdAlignment(
                missing_in_td=("artifact:demo/public#ec-only",),
                missing_in_ec=("artifact:demo/public#td-only",),
            ),
        )

        self.assertEqual(
            health.ec_td_alignment.evaluation,
            CellEvaluation.FAILED,
        )
        self.assertEqual(reduce_health(health), HealthAssessment.BLOCKED)

    def test_missing_ec_evaluation_is_indeterminate(self) -> None:
        health = SemanticHealth(
            ec_accepts_td=EcAcceptsTd(CellEvaluation.NOT_EVALUATED),
            ec_td_alignment=EcTdAlignment(),
        )

        self.assertEqual(reduce_health(health), HealthAssessment.INDETERMINATE)
        self.assertEqual(aggregate_exit_code(reduce_health(health)), 1)


if __name__ == "__main__":
    unittest.main()
