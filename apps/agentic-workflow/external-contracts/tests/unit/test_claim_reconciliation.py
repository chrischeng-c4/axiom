"""Negative tests for the retired synthetic claim-closure projection guard."""

from __future__ import annotations

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
            preserved.write_text('"""ordinary EC"""\\n', encoding="utf-8")
            with mock.patch.object(subject, "CASE_ROOT", root):
                selected = subject._owned_modules()

        self.assertEqual(selected, (owned,))


if __name__ == "__main__":
    unittest.main()
