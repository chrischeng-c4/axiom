# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_empty"
# subject = "cpython.test_startfile.TestCase.test_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_empty
"""Auto-ported test: TestCase::test_empty."""


import os
import platform
import sys
import tempfile
from os import path


if not hasattr(os, "startfile"):
    print("TestCase::test_empty: skipped os.startfile unavailable")
elif hasattr(platform, "win32_is_iot") and platform.win32_is_iot():
    print("TestCase::test_empty: skipped Windows IoT Core or nanoserver")
else:
    with tempfile.TemporaryDirectory() as tmp:
        empty = path.join(tmp, "empty.vbs")
        with open(empty, "w", encoding="utf-8") as handle:
            handle.write("' empty script\n")
        cwd = path.dirname(sys.executable)
        os.startfile(empty)
        os.startfile(empty, "open")
        os.startfile(empty, cwd=cwd)
    print("TestCase::test_empty: ok")
