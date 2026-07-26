"""Independent negative tests for the claim-reconciliation producer."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

import sys


SRC_ROOT = Path(__file__).resolve().parents[1] / "src"
sys.path.insert(0, str(SRC_ROOT))

import claim_reconciliation as subject  # noqa: E402


def report() -> dict[str, object]:
    return {
        "blockers": [],
        "capabilities": [
            {
                "id": "cap",
                "claims": [
                    {
                        "id": "claim",
                        "required_for_verified": True,
                        "user_story": "Observe the claim",
                        "oracle": "the exact behavior remains observable",
                        "gates": [
                            {"command": "cargo test -p demo --lib exact_behavior"},
                            {"command": "reference-only-token"},
                        ],
                    }
                ],
            }
        ],
        "python_artifact": {
            "td_module_ids": ["artifact:cap/claim"],
            "cases": [],
        },
    }


class ClaimReconciliationTest(unittest.TestCase):
    def test_rejects_unsupported_work_root_kind(self) -> None:
        candidate = report()
        candidate["blockers"] = [
            "capability `cap` work root `claim` has invalid Kind `task`"
        ]

        with self.assertRaisesRegex(RuntimeError, "unsupported Work Root"):
            subject._validate_catalog_and_python_td_refs(candidate)

    def test_rejects_unresolved_python_td_artifact_id(self) -> None:
        candidate = report()
        candidate["python_artifact"]["td_module_ids"] = [
            "artifact:cap/missing-claim"
        ]

        with self.assertRaisesRegex(RuntimeError, "do not resolve"):
            subject._validate_catalog_and_python_td_refs(candidate)

    def test_missing_claim_case_generates_all_machine_gates_and_classifies_tokens(
        self,
    ) -> None:
        cases = subject._claims_to_generate(report(), set())

        self.assertEqual(len(cases), 1)
        self.assertEqual(
            cases[0].commands,
            ("cargo test -p demo --lib exact_behavior",),
        )
        self.assertEqual(cases[0].reference_tokens, ("reference-only-token",))
        self.assertIn(cases[0].claim_oracle, subject._render_module(cases[0]))

    def test_stale_generated_module_is_reported_as_drift(self) -> None:
        cases = subject._claims_to_generate(report(), set())
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            case_root = root / "src" / "cases"
            case_root.mkdir(parents=True)
            stale = case_root / f"{cases[0].case_id}.py"
            stale.write_text(
                f'\"\"\"stale\"\"\"\\n\\n{subject.GENERATED_MARKER}\\n',
                encoding="utf-8",
            )
            with (
                mock.patch.object(subject, "CASE_ROOT", case_root),
                mock.patch.object(subject, "REPOSITORY_ROOT", root),
            ):
                state = subject._reconciliation_state(cases, "[project]\n")

        self.assertEqual(
            state.drifted_modules,
            (f"src/cases/{cases[0].case_id}.py",),
        )


if __name__ == "__main__":
    unittest.main()
