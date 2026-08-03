"""Negative tests for the retired synthetic claim-closure projection guard."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

import claim_reconciliation as subject  # noqa: E402


def inventory(
    *,
    case_id: str = "native-case",
    test_path: str = "src/cases/native-case.py",
    command: str = "python3 native_oracle.py",
) -> str:
    return f"""
[tool.aw.python-ec]

[[tool.aw.python-ec.cases]]
id = "{case_id}"
test_path = "{test_path}"
command = "{command}"
"""


def sample_candidate_toml(
    cases: list[dict[str, str]] | None = None,
) -> str:
    if cases is None:
        cases = [
            {
                "id": "case-1",
                "capability_id": "cap-a",
                "use_case_id": "uc-1",
                "dimension": "behavior",
            },
            {
                "id": "case-2",
                "capability_id": "cap-a",
                "use_case_id": "uc-2",
                "dimension": "behavior",
            },
        ]
    lines = []
    for c in cases:
        lines.append("[[tool.aw.python-ec.cases]]")
        for k, v in c.items():
            lines.append(f'{k} = "{v}"')
    return "\n".join(lines) + "\n"


def sample_expected_json(
    mappings: list[dict[str, str]] | None = None,
) -> str:
    if mappings is None:
        mappings = [
            {
                "case_id": "case-1",
                "capability_id": "cap-a",
                "use_case_id": "uc-1",
                "dimension": "behavior",
            },
            {
                "case_id": "case-2",
                "capability_id": "cap-a",
                "use_case_id": "uc-2",
                "dimension": "behavior",
            },
        ]
    return json.dumps(
        {
            "schema_version": "aw.python-ec.expected-mapping.v1",
            "mappings": mappings,
        }
    )


class ClaimReconciliationTest(unittest.TestCase):
    def test_clean_native_inventory_has_no_findings(self) -> None:
        findings = subject._projection_findings(inventory())

        self.assertEqual(
            findings,
            {
                "synthetic_claim_cases": [],
                "cargo_delegating_commands": [],
                "legacy_test_paths": [],
                "rust_invariant_cases": [],
            },
        )

    def test_rejects_synthetic_self_oracle_case(self) -> None:
        findings = subject._projection_findings(
            inventory(case_id="claim-closure-cap-claim")
        )

        self.assertEqual(
            findings["synthetic_claim_cases"],
            ["claim-closure-cap-claim"],
        )

    def test_rejects_cargo_delegation_and_legacy_rust_path(self) -> None:
        findings = subject._projection_findings(
            inventory(
                test_path="apps/agentic-workflow/tests/legacy.rs",
                command="cargo test -p agentic-workflow --test legacy",
            )
        )

        self.assertEqual(findings["cargo_delegating_commands"], ["native-case"])
        self.assertEqual(findings["legacy_test_paths"], ["native-case"])

    def test_generated_block_is_removed_without_touching_native_inventory(self) -> None:
        text = (
            inventory()
            + f"\n{subject.BLOCK_START}\n"
            + "[[tool.aw.python-ec.cases]]\n"
            + 'id = "claim-closure-cap-claim"\n'
            + 'test_path = "src/cases/claim-closure-cap-claim.py"\n'
            + 'command = "python3 claim_gate.py"\n'
            + f"{subject.BLOCK_END}\n"
        )

        cleaned = subject._without_generated_block(text)

        self.assertIn('id = "native-case"', cleaned)
        self.assertNotIn("claim-closure-cap-claim", cleaned)
        self.assertNotIn(subject.BLOCK_START, cleaned)

    def test_rust_invariant_case_is_removed_without_touching_native_inventory(self) -> None:
        retired = next(iter(subject._rust_invariant_case_ids()))
        text = inventory() + inventory(
            case_id=retired,
            test_path=f"src/cases/{retired}.py",
            command=f"python3 {retired}.py",
        )

        cleaned = subject._without_rust_invariant_cases(text)

        self.assertIn('id = "native-case"', cleaned)
        self.assertNotIn(retired, cleaned)

    def test_only_marker_owned_modules_are_selected_for_retirement(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            owned = root / "claim-closure-owned.py"
            owned.write_text(
                f'"""generated"""\n\n{subject.GENERATED_MARKER}\n',
                encoding="utf-8",
            )
            preserved = root / "claim-closure-hand-authored.py"
            preserved.write_text('"""ordinary EC"""\n', encoding="utf-8")
            with mock.patch.object(subject, "CASE_ROOT", root):
                selected = subject._owned_modules()

        self.assertEqual(selected, (owned,))

    def test_reconcile_copied_inventory_clean(self) -> None:
        res = subject.reconcile_copied_inventory(
            sample_candidate_toml(), sample_expected_json()
        )
        self.assertEqual(res["status"], "clean")
        self.assertEqual(res["case_count"], 2)
        self.assertEqual(len(res["case_mapping"]), 2)
        self.assertEqual(res["schema_version"], "aw.python-ec.expected-mapping.v1")
        self.assertEqual(res["findings"]["missing_expected_mappings"], [])
        self.assertEqual(res["findings"]["unexpected_mappings"], [])
        self.assertEqual(res["findings"]["duplicate_case_ids"], [])
        self.assertEqual(res["findings"]["malformed_inputs"], [])
        self.assertEqual(res["findings"]["binding_mismatches"], [])

    def test_reconcile_copied_inventory_missing_expected_case(self) -> None:
        cand_toml = sample_candidate_toml(
            [
                {
                    "id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-1",
                    "dimension": "behavior",
                }
            ]
        )
        res = subject.reconcile_copied_inventory(cand_toml, sample_expected_json())
        self.assertEqual(res["status"], "drifted")
        self.assertEqual(
            res["findings"]["missing_expected_mappings"],
            [
                {
                    "case_id": "case-2",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-2",
                    "dimension": "behavior",
                }
            ],
        )
        self.assertEqual(res["findings"]["unexpected_mappings"], [])

    def test_reconcile_copied_inventory_duplicate_case_id(self) -> None:
        cand_toml = sample_candidate_toml(
            [
                {
                    "id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-1",
                    "dimension": "behavior",
                },
                {
                    "id": "case-2",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-2",
                    "dimension": "behavior",
                },
                {
                    "id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-1",
                    "dimension": "behavior",
                },
            ]
        )
        res = subject.reconcile_copied_inventory(cand_toml, sample_expected_json())
        self.assertEqual(res["status"], "drifted")
        self.assertEqual(res["findings"]["duplicate_case_ids"], ["case-1"])
        self.assertEqual(res["case_count"], 3)
        self.assertEqual(len(res["case_mapping"]), 3)

    def test_reconcile_copied_inventory_misbound_case(self) -> None:
        cand_toml = sample_candidate_toml(
            [
                {
                    "id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "MUTATED",
                    "dimension": "behavior",
                },
                {
                    "id": "case-2",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-2",
                    "dimension": "behavior",
                },
            ]
        )
        res = subject.reconcile_copied_inventory(cand_toml, sample_expected_json())
        self.assertEqual(res["status"], "drifted")
        self.assertEqual(
            res["findings"]["missing_expected_mappings"],
            [
                {
                    "case_id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "uc-1",
                    "dimension": "behavior",
                }
            ],
        )
        self.assertEqual(
            res["findings"]["unexpected_mappings"],
            [
                {
                    "case_id": "case-1",
                    "capability_id": "cap-a",
                    "use_case_id": "MUTATED",
                    "dimension": "behavior",
                }
            ],
        )
        self.assertEqual(res["findings"]["binding_mismatches"], ["case-1"])

    def test_reconcile_copied_inventory_malformed_inputs(self) -> None:
        bad_toml = subject.reconcile_copied_inventory(
            "not valid toml", sample_expected_json()
        )
        self.assertEqual(bad_toml["status"], "drifted")
        self.assertTrue(bad_toml["findings"]["malformed_inputs"])

        bad_json = subject.reconcile_copied_inventory(
            sample_candidate_toml(), json.dumps({"schema_version": "wrong.v1"})
        )
        self.assertEqual(bad_json["status"], "drifted")
        self.assertTrue(bad_json["findings"]["malformed_inputs"])

    def test_cli_one_argument_exits_2(self) -> None:
        with mock.patch.object(
            sys, "argv", ["claim_reconciliation.py", "--inventory", "foo.toml"]
        ):
            with self.assertRaises(SystemExit) as cm:
                subject.main()
            self.assertEqual(cm.exception.code, 2)

    def test_no_argument_live_invocation_remains_compatible(self) -> None:
        text = inventory()
        res = subject._result(text)
        self.assertEqual(res["schema_version"], "aw.python-ec.claim-reconciliation.v2")

    def test_canonical_no_argument_v2_reconciliation_is_clean(self) -> None:
        text = subject.PYPROJECT_PATH.read_text(encoding="utf-8")
        res = subject._result(text)
        self.assertEqual(res["schema_version"], "aw.python-ec.claim-reconciliation.v2")
        self.assertEqual(res["status"], "clean")
        self.assertEqual(res["rust_invariant_case_count"], 11)


if __name__ == "__main__":
    unittest.main()
