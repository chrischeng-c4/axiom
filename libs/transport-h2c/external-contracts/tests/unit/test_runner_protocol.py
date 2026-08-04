from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

CASES = (
    ("least-loaded-stream-dispatch-behavior", 15),
    ("least-loaded-stream-dispatch-security", 12),
    ("connection-health-recovery-behavior", 16),
    ("connection-health-recovery-security", 14),
)


class TestRunnerProtocol(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.here = Path(__file__).resolve().parent
        cls.ext_contracts = cls.here.parents[1]
        cls.src_dir = cls.ext_contracts / "src"
        cls.runner_path = cls.src_dir / "runner.py"

        spec = importlib.util.spec_from_file_location("runner_module", cls.runner_path)
        assert spec is not None and spec.loader is not None
        cls.runner_module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = cls.runner_module
        spec.loader.exec_module(cls.runner_module)

    def _invoke(self, command, tmpdir, protocol="aw.python-artifact.v1", evidence=True):
        orig_env = os.environ.copy()
        orig_argv = sys.argv[:]
        orig_stdout = sys.stdout
        captured = io.StringIO()
        try:
            os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = protocol
            os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
            os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
            if evidence:
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir
            else:
                os.environ.pop("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", None)
            sys.argv = ["runner.py", command]
            sys.stdout = captured
            with self.assertRaises(SystemExit) as cm:
                self.runner_module.main()
            return cm.exception.code, json.loads(captured.getvalue().strip())
        finally:
            os.environ.clear()
            os.environ.update(orig_env)
            sys.argv = orig_argv
            sys.stdout = orig_stdout

    def test_every_case_passes_and_reports_its_declared_check_count(self) -> None:
        for case_id, expected_checks in CASES:
            with self.subTest(case_id=case_id):
                with tempfile.TemporaryDirectory() as tmpdir:
                    code, envelope = self._invoke(case_id, tmpdir)
                    self.assertEqual(code, 0)
                    self.assertEqual(
                        envelope["schema_version"], "aw.python-artifact.result.v1"
                    )
                    self.assertEqual(envelope["status"], "passed")
                    self.assertEqual(envelope["source_digest"], "src_digest_test_123")
                    self.assertEqual(
                        envelope["dependency_lock_digest"], "dep_digest_test_456"
                    )
                    self.assertEqual(
                        envelope["evidence"], [f"evidence/{case_id}.json"]
                    )

                    ev_path = Path(tmpdir) / f"{case_id}.json"
                    self.assertTrue(ev_path.is_file())
                    ev_data = json.loads(ev_path.read_text(encoding="utf-8"))
                    self.assertEqual(ev_data["case_id"], case_id)
                    self.assertTrue(ev_data["passed"])
                    self.assertEqual(ev_data["minimum_checks"], expected_checks)
                    self.assertEqual(len(ev_data["checks"]), expected_checks)
                    self.assertTrue(all(c["passed"] for c in ev_data["checks"]))

    def test_every_check_name_is_unique_within_its_case(self) -> None:
        for case_id, expected_checks in CASES:
            with self.subTest(case_id=case_id):
                with tempfile.TemporaryDirectory() as tmpdir:
                    self._invoke(case_id, tmpdir)
                    ev_data = json.loads(
                        (Path(tmpdir) / f"{case_id}.json").read_text(encoding="utf-8")
                    )
                    names = [c["name"] for c in ev_data["checks"]]
                    self.assertEqual(len(set(names)), expected_checks)

    def test_an_unknown_command_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            code, envelope = self._invoke("no-such-case", tmpdir)
            self.assertEqual(code, 1)
            self.assertEqual(envelope["status"], "failed")
            self.assertEqual(
                envelope["schema_version"], "aw.python-artifact.result.v1"
            )

    def test_a_protocol_mismatch_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            code, envelope = self._invoke(
                CASES[0][0], tmpdir, protocol="aw.python-artifact.v99"
            )
            self.assertEqual(code, 1)
            self.assertEqual(envelope["status"], "failed")

    def test_a_missing_evidence_directory_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            code, envelope = self._invoke(CASES[0][0], tmpdir, evidence=False)
            self.assertEqual(code, 1)
            self.assertEqual(envelope["status"], "failed")


if __name__ == "__main__":
    unittest.main()
