from __future__ import annotations

import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

CASE_COUNTS = {
    "crash-safe-replacement-behavior": 14,
    "crash-safe-replacement-security": 13,
    "torn-tail-recovery-behavior": 14,
    "torn-tail-recovery-security": 14,
    "sequence-ordered-snapshots-behavior": 13,
    "sequence-ordered-snapshots-security": 14,
}

_HERE = Path(__file__).resolve().parent
_RUNNER_PATH = _HERE.parents[1] / "src" / "runner.py"

spec = importlib.util.spec_from_file_location("runner_module", _RUNNER_PATH)
assert spec is not None and spec.loader is not None
runner_mod = importlib.util.module_from_spec(spec)
sys.modules["runner_module"] = runner_mod
spec.loader.exec_module(runner_mod)

class TestRunnerProtocol(unittest.TestCase):
    def test_cases_runner_execution_and_evidence(self) -> None:
        for cmd, expected_count in CASE_COUNTS.items():
            with self.subTest(command=cmd):
                with tempfile.TemporaryDirectory() as tmp_dir:
                    old_env = os.environ.copy()
                    old_argv = list(sys.argv)
                    old_stdout = sys.stdout
                    old_stderr = sys.stderr

                    try:
                        os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                        os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "digest-source-123"
                        os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "digest-lock-456"
                        os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmp_dir
                        sys.argv = ["runner.py", cmd]

                        captured_stdout = io.StringIO()
                        captured_stderr = io.StringIO()
                        sys.stdout = captured_stdout
                        sys.stderr = captured_stderr

                        exit_code = None
                        try:
                            runner_mod.main()
                        except SystemExit as se:
                            exit_code = se.code

                        self.assertEqual(exit_code, 0, f"Command {cmd} failed with exit code {exit_code}")

                        stdout_val = captured_stdout.getvalue().strip()
                        lines = [line for line in stdout_val.splitlines() if line.strip()]
                        self.assertEqual(len(lines), 1, f"Expected exactly one line on stdout for {cmd}")

                        out_obj = json.loads(lines[0])
                        self.assertEqual(out_obj.get("schema_version"), "aw.python-artifact.result.v1")
                        self.assertEqual(out_obj.get("status"), "passed")
                        self.assertEqual(out_obj.get("source_digest"), "digest-source-123")
                        self.assertEqual(out_obj.get("dependency_lock_digest"), "digest-lock-456")
                        self.assertEqual(out_obj.get("evidence"), [f"evidence/{cmd}.json"])

                        ev_file = Path(tmp_dir) / f"{cmd}.json"
                        self.assertTrue(ev_file.is_file())
                        ev_data = json.loads(ev_file.read_text())
                        self.assertEqual(ev_data.get("case_id"), cmd)
                        self.assertEqual(ev_data.get("minimum_checks"), expected_count)
                        self.assertGreaterEqual(len(ev_data.get("checks", [])), expected_count)
                        self.assertTrue(ev_data.get("passed"))
                    finally:
                        os.environ.clear()
                        os.environ.update(old_env)
                        sys.argv = old_argv
                        sys.stdout = old_stdout
                        sys.stderr = old_stderr

    def test_fail_closed_invalid_protocol(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            old_env = os.environ.copy()
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "wrong.protocol.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "digest-source"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "digest-lock"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmp_dir
                sys.argv = ["runner.py", "crash-safe-replacement-behavior"]

                captured_stdout = io.StringIO()
                captured_stderr = io.StringIO()
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                exit_code = None
                try:
                    runner_mod.main()
                except SystemExit as se:
                    exit_code = se.code

                self.assertEqual(exit_code, 1)
                stdout_val = captured_stdout.getvalue().strip()
                lines = [line for line in stdout_val.splitlines() if line.strip()]
                self.assertEqual(len(lines), 1)
                out_obj = json.loads(lines[0])
                self.assertEqual(out_obj.get("status"), "failed")
            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr

    def test_fail_closed_unknown_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            old_env = os.environ.copy()
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "digest-source"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "digest-lock"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmp_dir
                sys.argv = ["runner.py", "unknown-command"]

                captured_stdout = io.StringIO()
                captured_stderr = io.StringIO()
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                exit_code = None
                try:
                    runner_mod.main()
                except SystemExit as se:
                    exit_code = se.code

                self.assertEqual(exit_code, 1)
                stdout_val = captured_stdout.getvalue().strip()
                lines = [line for line in stdout_val.splitlines() if line.strip()]
                self.assertEqual(len(lines), 1)
                out_obj = json.loads(lines[0])
                self.assertEqual(out_obj.get("status"), "failed")
            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr

    def test_fail_closed_missing_env(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            old_env = os.environ.copy()
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                # Missing AW_PYTHON_ARTIFACT_SOURCE_DIGEST
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmp_dir
                sys.argv = ["runner.py", "crash-safe-replacement-behavior"]

                captured_stdout = io.StringIO()
                captured_stderr = io.StringIO()
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                exit_code = None
                try:
                    runner_mod.main()
                except SystemExit as se:
                    exit_code = se.code

                self.assertEqual(exit_code, 1)
                stdout_val = captured_stdout.getvalue().strip()
                lines = [line for line in stdout_val.splitlines() if line.strip()]
                self.assertEqual(len(lines), 1)
                out_obj = json.loads(lines[0])
                self.assertEqual(out_obj.get("status"), "failed")
            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr

if __name__ == "__main__":
    unittest.main()
