"""Black-box contract for the shared Python EC fixture's prebuilt AW binary."""

from __future__ import annotations

import os
import stat
import subprocess
import tempfile
from pathlib import Path
from unittest.mock import call, patch

from wi_contract_fixture import project_fixture, resolve_aw_binary, run_aw


CASE_ID = "python-ec-fixture-no-cargo-delegation"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "no-cargo-ec-fixture-execution"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case python-ec-fixture-no-cargo-delegation"
)
ASSERTIONS = (
    "a calibrated execution-boundary interceptor rejects a Cargo command, then run_aw preflights and launches exactly the selected AW executable",
    "missing, non-executable, truncated, non-native, and unlaunchable configured AW executables fail before the product command",
)


def verify() -> list[str]:
    previous = os.environ.get("AW_EC_AW_BINARY")
    try:
        with tempfile.TemporaryDirectory() as raw_root:
            root = Path(raw_root)
            binary = root / "aw"
            header = bytearray(64)
            header[:7] = b"\x7fELF\x02\x01\x01"
            header[20:24] = (1).to_bytes(4, "little")
            header[52:54] = (64).to_bytes(2, "little")
            binary.write_bytes(header)
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            os.environ["AW_EC_AW_BINARY"] = str(binary)

            def reject_build_delegation(
                command: list[str], **_: object
            ) -> subprocess.CompletedProcess[str]:
                rendered = [str(part) for part in command]
                if any(part == "cargo" or part == "rustup" or "cargo" in part for part in rendered):
                    raise AssertionError("Cargo/rustup delegation is forbidden in Python EC execution")
                if rendered == [str(binary.resolve()), "--version"]:
                    return subprocess.CompletedProcess(command, 0, "aw 0.0.0\n", "")
                if rendered == [str(binary.resolve()), "wi", "show"]:
                    return subprocess.CompletedProcess(command, 0, "{}", "")
                raise AssertionError(f"unexpected subprocess command: {rendered}")

            with project_fixture() as fixture_root:
                with patch(
                    "wi_contract_fixture.subprocess.run", side_effect=reject_build_delegation
                ) as launched:
                    try:
                        subprocess.run(["cargo", "build"], check=False)
                    except AssertionError as error:
                        assert "Cargo/rustup delegation" in str(error)
                    else:
                        raise AssertionError("negative control must reject a Cargo subprocess")
                    completed = run_aw(fixture_root, "wi", "show", expect_success=None)
            assert launched.call_count == 3
            assert launched.call_args_list[0] == call(["cargo", "build"], check=False)
            preflight_args, preflight_kwargs = launched.call_args_list[1]
            assert preflight_args == ([str(binary.resolve()), "--version"],)
            assert preflight_kwargs["text"] is True
            assert preflight_kwargs["capture_output"] is True
            assert preflight_kwargs["check"] is False
            assert preflight_kwargs["timeout"] == 10
            launch_args, launch_kwargs = launched.call_args_list[2]
            assert launch_args == ([str(binary.resolve()), "wi", "show"],)
            assert launch_kwargs["cwd"] == fixture_root
            assert launch_kwargs["env"]["AW_FIXTURE_LOCAL_BACKEND"] == "1"
            assert launch_kwargs["env"]["AW_DISABLE_CAP"] == "1"
            assert launch_kwargs["text"] is True
            assert launch_kwargs["capture_output"] is True
            assert launch_kwargs["check"] is False
            assert completed.args == [str(binary.resolve()), "wi", "show"]

            os.environ["AW_EC_AW_BINARY"] = str(root / "missing-aw")
            with patch("wi_contract_fixture.subprocess.run") as launched:
                try:
                    run_aw(root, "--version")
                except AssertionError as error:
                    assert "Prebuilt AW binary missing" in str(error)
                else:
                    raise AssertionError("missing prebuilt AW binary must fail closed")
            launched.assert_not_called()

            unlaunchable = root / "unlaunchable-aw"
            unlaunchable.write_bytes(header)
            unlaunchable.chmod(unlaunchable.stat().st_mode | stat.S_IXUSR)
            os.environ["AW_EC_AW_BINARY"] = str(unlaunchable)
            with patch(
                "wi_contract_fixture.subprocess.run",
                side_effect=OSError(8, "Exec format error"),
            ) as launched:
                try:
                    run_aw(root, "wi", "show")
                except AssertionError as error:
                    assert "could not execute its AW preflight" in str(error)
                else:
                    raise AssertionError("unlaunchable AW binary must fail before product command")
            launched.assert_called_once_with(
                [str(unlaunchable.resolve()), "--version"],
                text=True,
                capture_output=True,
                check=False,
                timeout=10,
            )

            non_executable = root / "non-executable-aw"
            non_executable.write_bytes(header)
            os.environ["AW_EC_AW_BINARY"] = str(non_executable)
            with patch("wi_contract_fixture.subprocess.run") as launched:
                try:
                    run_aw(root, "--version")
                except AssertionError as error:
                    assert "not executable" in str(error)
                else:
                    raise AssertionError("non-executable AW binary must fail closed")
            launched.assert_not_called()

            truncated = root / "truncated-elf-aw"
            truncated.write_bytes(b"\x7fELFfixture-aw")
            truncated.chmod(truncated.stat().st_mode | stat.S_IXUSR)
            os.environ["AW_EC_AW_BINARY"] = str(truncated)
            with patch("wi_contract_fixture.subprocess.run") as launched:
                try:
                    run_aw(root, "--version")
                except AssertionError as error:
                    assert "not a supported native executable" in str(error)
                else:
                    raise AssertionError("truncated AW binary must fail closed")
            launched.assert_not_called()

            malformed = root / "shell-script-aw"
            malformed.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            malformed.chmod(malformed.stat().st_mode | stat.S_IXUSR)
            os.environ["AW_EC_AW_BINARY"] = str(malformed)
            with patch("wi_contract_fixture.subprocess.run") as launched:
                try:
                    run_aw(root, "--version")
                except AssertionError as error:
                    assert "not a supported native executable" in str(error)
                else:
                    raise AssertionError("non-native AW binary must fail closed")
            launched.assert_not_called()
    finally:
        if previous is None:
            os.environ.pop("AW_EC_AW_BINARY", None)
        else:
            os.environ["AW_EC_AW_BINARY"] = previous
    return list(ASSERTIONS)
