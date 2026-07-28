"""Independent assertions for the two-cell project-health contract."""

from __future__ import annotations

from typing import Any


SEMANTIC_CELLS = frozenset({"ec_accepts_td", "ec_td_alignment"})


def assert_two_cell_health(
    result: dict[str, Any],
    payload: dict[str, Any],
) -> None:
    result_cells = result["semantic_health"]
    payload_cells = payload["semantic_health"]
    assert set(result_cells) == SEMANTIC_CELLS
    assert set(payload_cells) == SEMANTIC_CELLS
    assert payload_cells == result_cells
    assert payload["assessment"] == result["assessment"]


def assert_alignment(
    alignment: dict[str, Any],
    *,
    missing_in_td: list[str],
    missing_in_ec: list[str],
) -> None:
    assert alignment["missing_in_td"] == sorted(missing_in_td)
    assert alignment["missing_in_ec"] == sorted(missing_in_ec)
    expected = "passed" if not missing_in_td and not missing_in_ec else "failed"
    assert alignment["evaluation"] == expected


def assert_ec_accepts_td(
    cell: dict[str, Any],
    *,
    evaluation: str,
    case_count: int,
    passed_count: int,
    failed_cases: list[str],
    missing_evidence_cases: list[str],
) -> None:
    assert cell["evaluation"] == evaluation
    assert cell["case_count"] == case_count
    assert cell["passed_count"] == passed_count
    assert cell["failed_cases"] == sorted(failed_cases)
    assert cell["missing_evidence_cases"] == sorted(missing_evidence_cases)
    if evaluation == "passed":
        assert cell["findings"] == []
    else:
        assert cell["findings"]
