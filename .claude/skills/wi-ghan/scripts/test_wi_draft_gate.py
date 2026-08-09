#!/usr/bin/env python3
from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

SCRIPT = Path(__file__).with_name("wi_draft_gate.py")
SPEC = importlib.util.spec_from_file_location("wi_draft_gate", SCRIPT)
assert SPEC and SPEC.loader
wi_draft_gate = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(wi_draft_gate)


def make_body(title: str, rows: list[str]) -> str:
    table_rows = "\n".join(rows)
    return f"""# {title}

## Goal

Test body for wi_draft_gate baseline measurement.

## How

### Verified premises

- .agents/rules/authoring/agent-instruction-ghan.md:21 - test premise.

### Change points

- .claude/skills/wi-ghan/scripts/wi_draft_gate.py - test change point.

### Frozen decisions

- Test frozen decision.

## Acceptance

| # | command | current | target | why it cannot hold by accident |
|---|---------|---------|--------|--------------------------------|
{table_rows}

### Negative control

At the revision these premises were read, `shasum -a 256 .claude/skills/wi-ghan/scripts/wi_draft_gate.py` reports `b8d07f3c8fca541b006ad4b581b7090678a61e223f04f66bf3e8b6f0c9d2c596`; that digest changes once this work lands, so record the post-change digest immediately before mutating. Then delete only the new measurement so the script returns to printing `PASS` on the structural verdict alone, leaving both the suite and the reference body untouched, and rerun row 1. The gate must go red, and the report must quote the failure verbatim, including the failing test name and the assertion values. Restore the file by writing the original bytes back with an editor, never with `cp -p` or any copy that preserves mtime, confirm `shasum -a 256` reports the digest recorded before the mutation, and rerun both rows to return them to green.

## Never

This addresses the worker implementing this work item, not the controller reviewing it.

### Must not touch

- None

### Must not do

- None
"""


class WIDraftGateTest(unittest.TestCase):
    def test_body_whose_rows_all_succeed_is_refused(self) -> None:
        body = make_body(
            "All Green Body",
            ["| 1 | `python3 -c \"import sys; sys.exit(0)\"` | exits 1 | exits 0 | test |"],
        )
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
            f.write(body)
            f.flush()
            body_path = Path(f.name)

        try:
            buf = io.StringIO()
            with patch("sys.argv", ["wi_draft_gate.py", str(body_path), "--project", "agentic-workflow"]):
                with contextlib.redirect_stdout(buf):
                    rc = wi_draft_gate.main()
            out = buf.getvalue()
            self.assertEqual(rc, 1, f"expected refused exit code 1, got {rc}. Output:\n{out}")
            self.assertIn("FAIL", out)
            self.assertIn("already succeed", out)
            self.assertIn("python3 -c \"import sys; sys.exit(0)\"", out)
        finally:
            body_path.unlink(missing_ok=True)

    def test_body_with_one_failing_row_passes(self) -> None:
        body = make_body(
            "Failing Body",
            ["| 1 | `python3 -c \"import sys; sys.exit(1)\"` | exits 1 | exits 0 | test |"],
        )
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
            f.write(body)
            f.flush()
            body_path = Path(f.name)

        try:
            buf = io.StringIO()
            with patch("sys.argv", ["wi_draft_gate.py", str(body_path), "--project", "agentic-workflow"]):
                with contextlib.redirect_stdout(buf):
                    rc = wi_draft_gate.main()
            out = buf.getvalue()
            self.assertEqual(rc, 0, f"expected PASS exit code 0, got {rc}. Output:\n{out}")
            self.assertIn("PASS", out)
        finally:
            body_path.unlink(missing_ok=True)

    def test_body_mixing_failing_row_and_regression_guard_passes(self) -> None:
        body = make_body(
            "Mixed Body",
            [
                "| 1 | `python3 -c \"import sys; sys.exit(1)\"` | exits 1 | exits 0 | test failing |",
                "| 2 | `python3 -c \"import sys; sys.exit(0)\"` | exits 1 | exits 0 | test green guard |",
            ],
        )
        with tempfile.NamedTemporaryFile("w", suffix=".md", delete=False) as f:
            f.write(body)
            f.flush()
            body_path = Path(f.name)

        try:
            buf = io.StringIO()
            with patch("sys.argv", ["wi_draft_gate.py", str(body_path), "--project", "agentic-workflow"]):
                with contextlib.redirect_stdout(buf):
                    rc = wi_draft_gate.main()
            out = buf.getvalue()
            self.assertEqual(rc, 0, f"expected PASS exit code 0, got {rc}. Output:\n{out}")
            self.assertIn("PASS", out)
        finally:
            body_path.unlink(missing_ok=True)

    def test_measured_reference_body_passes(self) -> None:
        ref_body = Path(__file__).resolve().parents[1] / "references" / "measured-body.md"
        self.assertTrue(ref_body.is_file(), f"reference body not found at {ref_body}")
        buf = io.StringIO()
        with patch("sys.argv", ["wi_draft_gate.py", str(ref_body), "--project", "agentic-workflow"]):
            with contextlib.redirect_stdout(buf):
                rc = wi_draft_gate.main()
        out = buf.getvalue()
        self.assertEqual(rc, 0, f"expected PASS exit code 0 for reference body, got {rc}. Output:\n{out}")
        self.assertIn("PASS", out)


if __name__ == "__main__":
    unittest.main()
