from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

_HERE = Path(__file__).resolve().parent
_SRC_DIR = _HERE.parents[1] / "src"
_RUNNER_PATH = _SRC_DIR / "runner.py"

spec = importlib.util.spec_from_file_location("runner_module", _RUNNER_PATH)
assert spec is not None and spec.loader is not None
runner = importlib.util.module_from_spec(spec)
sys.modules["runner_module"] = runner
spec.loader.exec_module(runner)

EXPECTED_ARITIES = {
    "version-stamp-emission-behavior": 14,
    "version-stamp-emission-security": 13,
    "best-effort-degradation-behavior": 14,
    "best-effort-degradation-security": 12,
    "directive-channel-integrity-behavior": 13,
    "directive-channel-integrity-security": 14,
}


class TestRunnerProtocol(unittest.TestCase):
    def setUp(self) -> None:
        self._orig_env = dict(os.environ)
        self._orig_argv = list(sys.argv)
        self._orig_stdout = sys.stdout

    def tearDown(self) -> None:
        os.environ.clear()
        os.environ.update(self._orig_env)
        sys.argv = list(self._orig_argv)
        sys.stdout = self._orig_stdout

    def _run_runner(self, command: str, env: dict[str, str]) -> tuple[int, dict, dict | None]:
        os.environ.update(env)
        sys.argv = ["runner.py", command]

        stdout_buf = io.StringIO()
        sys.stdout = stdout_buf

        exit_code = 0
        try:
            runner.main()
        except SystemExit as e:
            exit_code = e.code if isinstance(e.code, int) else 1

        output = stdout_buf.getvalue().strip()
        envelope = json.loads(output) if output else {}

        ev_dir = env.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", "")
        ev_file = Path(ev_dir) / f"{command}.json" if ev_dir and command else None
        evidence_content = json.loads(ev_file.read_text(encoding="utf-8")) if ev_file and ev_file.is_file() else None

        return exit_code, envelope, evidence_content

    def test_all_cases_pass_and_write_evidence(self) -> None:
        for case_name, expected_count in EXPECTED_ARITIES.items():
            with tempfile.TemporaryDirectory() as tmp_dir:
                env = {
                    "AW_PYTHON_ARTIFACT_PROTOCOL": "aw.python-artifact.v1",
                    "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "src123digest",
                    "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "lock456digest",
                    "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": tmp_dir,
                }
                exit_code, envelope, evidence = self._run_runner(case_name, env)

                self.assertEqual(exit_code, 0, f"Case {case_name} failed with exit code {exit_code}")
                self.assertEqual(envelope.get("status"), "passed")
                self.assertEqual(envelope.get("schema_version"), "aw.python-artifact.result.v1")
                self.assertEqual(envelope.get("source_digest"), "src123digest")
                self.assertEqual(envelope.get("dependency_lock_digest"), "lock456digest")
                self.assertEqual(envelope.get("evidence"), [f"evidence/{case_name}.json"])

                self.assertIsNotNone(evidence, f"Missing evidence file for {case_name}")
                if evidence:
                    self.assertEqual(evidence.get("case_id"), case_name)
                    self.assertEqual(evidence.get("minimum_checks"), expected_count)
                    self.assertEqual(len(evidence.get("checks", [])), expected_count)
                    self.assertTrue(evidence.get("passed"))

    def test_fail_closed_unknown_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            env = {
                "AW_PYTHON_ARTIFACT_PROTOCOL": "aw.python-artifact.v1",
                "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "src123digest",
                "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "lock456digest",
                "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": tmp_dir,
            }
            exit_code, envelope, evidence = self._run_runner("unknown-command", env)
            self.assertEqual(exit_code, 1)
            self.assertEqual(envelope.get("status"), "failed")

    def test_fail_closed_invalid_protocol(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            env = {
                "AW_PYTHON_ARTIFACT_PROTOCOL": "invalid.protocol.v2",
                "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "src123digest",
                "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "lock456digest",
                "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": tmp_dir,
            }
            exit_code, envelope, evidence = self._run_runner("version-stamp-emission-behavior", env)
            self.assertEqual(exit_code, 1)
            self.assertEqual(envelope.get("status"), "failed")

    def test_fail_closed_missing_env_var(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            env = {
                "AW_PYTHON_ARTIFACT_PROTOCOL": "aw.python-artifact.v1",
                "AW_PYTHON_ARTIFACT_SOURCE_DIGEST": "",
                "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST": "lock456digest",
                "AW_PYTHON_ARTIFACT_EVIDENCE_DIR": tmp_dir,
            }
            exit_code, envelope, evidence = self._run_runner("version-stamp-emission-behavior", env)
            self.assertEqual(exit_code, 1)
            self.assertEqual(envelope.get("status"), "failed")


if __name__ == "__main__":
    unittest.main()
