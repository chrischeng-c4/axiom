"""Unit tests proving the health EC oracle rejects false-green evidence."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from oracles.project_health_contract import (  # noqa: E402
    assert_alignment,
    assert_ec_accepts_td,
    assert_two_cell_health,
)


def healthy_cells() -> dict[str, object]:
    return {
        "ec_accepts_td": {
            "evaluation": "passed",
            "case_count": 1,
            "passed_count": 1,
            "failed_cases": [],
            "missing_evidence_cases": [],
            "findings": [],
        },
        "ec_td_alignment": {
            "evaluation": "passed",
            "missing_in_td": [],
            "missing_in_ec": [],
        },
    }


class ProjectHealthContractTest(unittest.TestCase):
    def test_accepts_exact_matching_two_cell_projection(self) -> None:
        result = {"assessment": "healthy", "semantic_health": healthy_cells()}
        payload = {"assessment": "healthy", "semantic_health": healthy_cells()}

        assert_two_cell_health(result, payload)

    def test_rejects_a_third_semantic_cell(self) -> None:
        cells = healthy_cells()
        cells["mutation"] = {"evaluation": "passed"}
        result = {"assessment": "healthy", "semantic_health": cells}
        payload = {"assessment": "healthy", "semantic_health": cells}

        with self.assertRaises(AssertionError):
            assert_two_cell_health(result, payload)

    def test_rejects_result_payload_divergence(self) -> None:
        result = {"assessment": "healthy", "semantic_health": healthy_cells()}
        payload = {"assessment": "blocked", "semantic_health": healthy_cells()}

        with self.assertRaises(AssertionError):
            assert_two_cell_health(result, payload)

    def test_alignment_oracle_checks_both_directions(self) -> None:
        alignment = {
            "evaluation": "failed",
            "missing_in_td": ["artifact:demo/public#ec-only"],
            "missing_in_ec": ["artifact:demo/public#td-only"],
        }

        assert_alignment(
            alignment,
            missing_in_td=["artifact:demo/public#ec-only"],
            missing_in_ec=["artifact:demo/public#td-only"],
        )

    def test_ec_acceptance_oracle_rejects_passed_zero_case_false_green(self) -> None:
        zero_case = {
            "evaluation": "passed",
            "case_count": 0,
            "passed_count": 0,
            "failed_cases": [],
            "missing_evidence_cases": [],
            "findings": [],
        }

        with self.assertRaises(AssertionError):
            assert_ec_accepts_td(
                zero_case,
                evaluation="passed",
                case_count=1,
                passed_count=1,
                failed_cases=[],
                missing_evidence_cases=[],
            )

    def test_ec_acceptance_oracle_pins_failed_case_identity(self) -> None:
        failed = {
            "evaluation": "failed",
            "case_count": 1,
            "passed_count": 0,
            "failed_cases": ["fixture-health"],
            "missing_evidence_cases": [],
            "findings": ["fixture evidence records exit_code=17"],
        }

        assert_ec_accepts_td(
            failed,
            evaluation="failed",
            case_count=1,
            passed_count=0,
            failed_cases=["fixture-health"],
            missing_evidence_cases=[],
        )


if __name__ == "__main__":
    unittest.main()
