# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "behavior"
# case = "test_plutil__test_lint_status"
# subject = "cpython.test_plistlib.TestPlutil.test_lint_status"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_plistlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: TestPlutil::test_lint_status (CPython 3.12 oracle)."""

import plistlib
import subprocess
import sys
import tempfile
import unittest
from test.test_plistlib import TestPlutil


if sys.platform != "darwin":
    case = TestPlutil("test_lint_status")
    result = unittest.TestResult()
    case.run(result)
    assert result.wasSuccessful(), result
    assert len(result.skipped) == 1, result.skipped
    assert result.skipped[0][1] == "plutil utility is for Mac os"
else:
    properties = {
        "fname": "H",
        "lname": "A",
        "marks": {"a": 100, "b": 0x10},
    }
    with tempfile.TemporaryDirectory() as tmpdir:
        file_name = f"{tmpdir}/plutil_test.plist"
        with open(file_name, "wb") as file:
            plistlib.dump(properties, file, fmt=plistlib.FMT_BINARY)
        result = subprocess.run(
            ["plutil", "-lint", file_name],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0, result
        assert result.stderr == "", result.stderr
        assert result.stdout == f"{file_name}: OK\n", result.stdout

print("TestPlutil::test_lint_status: ok")
