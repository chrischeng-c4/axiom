from __future__ import annotations

import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

COMMANDS = [
    "lock-free-accumulation-behavior",
    "lock-free-accumulation-security",
    "exposition-encoding-behavior",
    "exposition-encoding-security",
    "label-value-containment-behavior",
    "label-value-containment-security",
]


class TestRunnerProtocol(unittest.TestCase):
    def setUp(self) -> None:
        self.src_dir = Path(__file__).resolve().parents[2] / "src"
        self.runner_path = self.src_dir / "runner.py"

        spec = importlib.util.spec_from_file_location("runner_module", self.runner_path)
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        self.runner_mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.runner_mod)

    def test_runner_executes_all_six_commands_successfully(self) -> None:
        for cmd in COMMANDS:
            with tempfile.TemporaryDirectory() as tmpdir:
                old_env = dict(os.environ)
                old_argv = list(sys.argv)
                old_stdout = sys.stdout
                old_stderr = sys.stderr

                captured_stdout = io.StringIO()
                captured_stderr = io.StringIO()

                try:
                    os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                    os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "srcdigest123"
                    os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "lockdigest456"
                    os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                    sys.argv = ["runner.py", cmd]
                    sys.stdout = captured_stdout
                    sys.stderr = captured_stderr

                    with self.assertRaises(SystemExit) as cm:
                        self.runner_mod.main()

                    self.assertEqual(cm.exception.code, 0, f"Command {cmd} failed with exit code {cm.exception.code}")

                    stdout_lines = [ln for ln in captured_stdout.getvalue().split("\n") if ln]
                    self.assertEqual(len(stdout_lines), 1, f"Command {cmd} emitted multiple or no lines on stdout")

                    envelope = json.loads(stdout_lines[0])
                    self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
                    self.assertEqual(envelope.get("status"), "passed")
                    self.assertEqual(envelope.get("source_digest"), "srcdigest123")
                    self.assertEqual(envelope.get("dependency_lock_digest"), "lockdigest456")
                    self.assertEqual(envelope.get("evidence"), [f"evidence/{cmd}.json"])

                    evidence_file = Path(tmpdir) / f"{cmd}.json"
                    self.assertTrue(evidence_file.exists(), f"Evidence file missing for {cmd}")

                    with open(evidence_file, "r", encoding="utf-8") as f:
                        ev_data = json.load(f)

                    self.assertEqual(ev_data.get("case_id"), cmd)
                    self.assertEqual(ev_data.get("minimum_checks"), 14)
                    self.assertGreaterEqual(len(ev_data.get("checks", [])), 14)
                    self.assertIs(ev_data.get("passed"), True)

                finally:
                    os.environ.clear()
                    os.environ.update(old_env)
                    sys.argv = old_argv
                    sys.stdout = old_stdout
                    sys.stderr = old_stderr

    def test_runner_fail_closed_wrong_protocol(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            old_env = dict(os.environ)
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            captured_stdout = io.StringIO()
            captured_stderr = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "wrong.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "srcdigest123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "lockdigest456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "lock-free-accumulation-behavior"]
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                with self.assertRaises(SystemExit) as cm:
                    self.runner_mod.main()

                self.assertEqual(cm.exception.code, 1)

                stdout_lines = [ln for ln in captured_stdout.getvalue().split("\n") if ln]
                self.assertEqual(len(stdout_lines), 1)

                envelope = json.loads(stdout_lines[0])
                self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
                self.assertEqual(envelope.get("status"), "failed")

            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr

    def test_runner_fail_closed_unknown_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            old_env = dict(os.environ)
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            captured_stdout = io.StringIO()
            captured_stderr = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "srcdigest123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "lockdigest456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "non-existent-command"]
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                with self.assertRaises(SystemExit) as cm:
                    self.runner_mod.main()

                self.assertEqual(cm.exception.code, 1)

                stdout_lines = [ln for ln in captured_stdout.getvalue().split("\n") if ln]
                self.assertEqual(len(stdout_lines), 1)

                envelope = json.loads(stdout_lines[0])
                self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
                self.assertEqual(envelope.get("status"), "failed")

            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr

    def test_runner_fail_closed_missing_env_var(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            old_env = dict(os.environ)
            old_argv = list(sys.argv)
            old_stdout = sys.stdout
            old_stderr = sys.stderr

            captured_stdout = io.StringIO()
            captured_stderr = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                # AW_PYTHON_ARTIFACT_SOURCE_DIGEST missing
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "lockdigest456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "lock-free-accumulation-behavior"]
                sys.stdout = captured_stdout
                sys.stderr = captured_stderr

                with self.assertRaises(SystemExit) as cm:
                    self.runner_mod.main()

                self.assertEqual(cm.exception.code, 1)

                stdout_lines = [ln for ln in captured_stdout.getvalue().split("\n") if ln]
                self.assertEqual(len(stdout_lines), 1)

                envelope = json.loads(stdout_lines[0])
                self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
                self.assertEqual(envelope.get("status"), "failed")

            finally:
                os.environ.clear()
                os.environ.update(old_env)
                sys.argv = old_argv
                sys.stdout = old_stdout
                sys.stderr = old_stderr


if __name__ == "__main__":
    unittest.main()
