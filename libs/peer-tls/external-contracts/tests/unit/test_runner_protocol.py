from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path


class TestRunnerProtocol(unittest.TestCase):

    @classmethod
    def setUpClass(cls) -> None:
        cls.here = Path(__file__).resolve().parent  # tests/unit
        cls.ext_contracts_dir = cls.here.parents[1]  # external-contracts
        cls.runner_path = cls.ext_contracts_dir / "src" / "runner.py"

        spec = importlib.util.spec_from_file_location("runner_module", cls.runner_path)
        assert spec is not None and spec.loader is not None
        cls.runner_module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(cls.runner_module)

    def test_verbatim_digests_and_exit_0_on_passed(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured_stdout = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "material-validation-behavior"]
                sys.stdout = captured_stdout

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 0)
                output = captured_stdout.getvalue().strip()
                envelope = json.loads(output)

                self.assertEqual(envelope["schema_version"], "aw.python-artifact.result.v1")
                self.assertEqual(envelope["status"], "passed")
                self.assertEqual(envelope["source_digest"], "src_digest_test_123")
                self.assertEqual(envelope["dependency_lock_digest"], "dep_digest_test_456")
                self.assertEqual(envelope["evidence"], ["evidence/material-validation-behavior.json"])

                evidence_file = Path(tmpdir) / "material-validation-behavior.json"
                self.assertTrue(evidence_file.is_file())
                self.assertGreater(evidence_file.stat().st_size, 0)
                ev_content = json.loads(evidence_file.read_text())
                self.assertTrue(ev_content.get("passed"))
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout

    def test_unknown_command_yields_failed_status_and_exit_1(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured_stdout = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "aw.python-artifact.v1"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "non-existent-command"]
                sys.stdout = captured_stdout

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 1)
                output = captured_stdout.getvalue().strip()
                envelope = json.loads(output)

                self.assertEqual(envelope["schema_version"], "aw.python-artifact.result.v1")
                self.assertEqual(envelope["status"], "failed")
                self.assertEqual(envelope["source_digest"], "src_digest_test_123")
                self.assertEqual(envelope["dependency_lock_digest"], "dep_digest_test_456")
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout

    def test_protocol_mismatch_yields_failed_status_and_exit_1(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            orig_env = os.environ.copy()
            orig_argv = sys.argv[:]
            orig_stdout = sys.stdout
            captured_stdout = io.StringIO()

            try:
                os.environ["AW_PYTHON_ARTIFACT_PROTOCOL"] = "wrong.protocol.v9"
                os.environ["AW_PYTHON_ARTIFACT_SOURCE_DIGEST"] = "src_digest_test_123"
                os.environ["AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST"] = "dep_digest_test_456"
                os.environ["AW_PYTHON_ARTIFACT_EVIDENCE_DIR"] = tmpdir

                sys.argv = ["runner.py", "material-validation-behavior"]
                sys.stdout = captured_stdout

                with self.assertRaises(SystemExit) as cm:
                    self.runner_module.main()

                self.assertEqual(cm.exception.code, 1)
                output = captured_stdout.getvalue().strip()
                envelope = json.loads(output)

                self.assertEqual(envelope["schema_version"], "aw.python-artifact.result.v1")
                self.assertEqual(envelope["status"], "failed")
            finally:
                os.environ.clear()
                os.environ.update(orig_env)
                sys.argv = orig_argv
                sys.stdout = orig_stdout


if __name__ == "__main__":
    unittest.main()
