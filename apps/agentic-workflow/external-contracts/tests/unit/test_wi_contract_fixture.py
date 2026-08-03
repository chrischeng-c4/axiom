"""Focused tests for prebuilt AW resolution in the Python EC fixture."""

from __future__ import annotations

import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch


SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from wi_contract_fixture import run_aw  # noqa: E402


class WiContractFixtureTest(unittest.TestCase):
    def setUp(self) -> None:
        original = dict(os.environ)

        def restore_environment() -> None:
            os.environ.clear()
            os.environ.update(original)

        self.addCleanup(restore_environment)

    def make_native_executable(self, root: Path) -> Path:
        binary = root / "aw"
        header = bytearray(64)
        header[:7] = b"\x7fELF\x02\x01\x01"
        header[20:24] = (1).to_bytes(4, "little")
        header[52:54] = (64).to_bytes(2, "little")
        binary.write_bytes(header)
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
        return binary

    def test_run_aw_uses_selected_prebuilt_binary(self) -> None:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            binary = self.make_native_executable(root)
            os.environ["AW_EC_AW_BINARY"] = str(binary)
            expected = binary.resolve()
            preflight = subprocess.CompletedProcess([str(expected), "--version"], 0, "aw 0.0.0\n", "")
            completed = subprocess.CompletedProcess([str(expected), "wi", "show"], 0, "{}", "")

            def reject_build_delegation(command: list[str], **_: object) -> subprocess.CompletedProcess[str]:
                rendered = [str(part) for part in command]
                if any(part == "cargo" or part == "rustup" or "cargo" in part for part in rendered):
                    raise AssertionError("Cargo/rustup delegation is forbidden in Python EC execution")
                if rendered == [str(expected), "--version"]:
                    return preflight
                if rendered == [str(expected), "wi", "show"]:
                    return completed
                raise AssertionError(f"unexpected subprocess command: {rendered}")
                return completed

            with patch(
                "wi_contract_fixture.subprocess.run", side_effect=reject_build_delegation
            ) as run:
                result = run_aw(root, "wi", "show")
            self.assertEqual(result.returncode, 0)
            self.assertEqual(run.call_count, 2)
            self.assertEqual(run.call_args_list[0].args[0], [str(expected), "--version"])
            self.assertEqual(run.call_args_list[1].args[0], [str(expected), "wi", "show"])

    def test_invalid_prebuilt_binary_fails_before_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            os.environ["AW_EC_AW_BINARY"] = str(root / "missing-aw")
            with patch("wi_contract_fixture.subprocess.run") as run:
                with self.assertRaisesRegex(AssertionError, "Prebuilt AW binary missing"):
                    run_aw(root, "wi", "show")
            run.assert_not_called()

    def test_non_executable_or_non_native_prebuilt_binary_fails_before_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            non_executable = root / "non-executable-aw"
            native_header = bytearray(64)
            native_header[:7] = b"\x7fELF\x02\x01\x01"
            native_header[20:24] = (1).to_bytes(4, "little")
            native_header[52:54] = (64).to_bytes(2, "little")
            non_executable.write_bytes(native_header)
            malformed = root / "shell-script-aw"
            malformed.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            malformed.chmod(malformed.stat().st_mode | stat.S_IXUSR)

            for binary, diagnostic in (
                (non_executable, "not executable"),
                (malformed, "not a supported native executable"),
            ):
                with self.subTest(binary=binary.name):
                    os.environ["AW_EC_AW_BINARY"] = str(binary)
                    with patch("wi_contract_fixture.subprocess.run") as run:
                        with self.assertRaisesRegex(AssertionError, diagnostic):
                            run_aw(root, "wi", "show")
                    run.assert_not_called()

    def test_truncated_native_header_fails_before_subprocess(self) -> None:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            truncated = root / "truncated-elf-aw"
            truncated.write_bytes(b"\x7fELFfixture-aw")
            truncated.chmod(truncated.stat().st_mode | stat.S_IXUSR)
            os.environ["AW_EC_AW_BINARY"] = str(truncated)
            with patch("wi_contract_fixture.subprocess.run") as run:
                with self.assertRaisesRegex(AssertionError, "not a supported native executable"):
                    run_aw(root, "wi", "show")
            run.assert_not_called()

    def test_unlaunchable_native_header_fails_before_product_command(self) -> None:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            binary = self.make_native_executable(root)
            os.environ["AW_EC_AW_BINARY"] = str(binary)
            with patch(
                "wi_contract_fixture.subprocess.run",
                side_effect=OSError(8, "Exec format error"),
            ) as run:
                with self.assertRaisesRegex(AssertionError, "could not execute its AW preflight"):
                    run_aw(root, "wi", "show")
            run.assert_called_once()
            self.assertEqual(run.call_args.args[0], [str(binary.resolve()), "--version"])


if __name__ == "__main__":
    unittest.main()
