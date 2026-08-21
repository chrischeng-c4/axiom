# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_python"
# subject = "cpython.test_startfile.TestCase.test_python"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_python
"""Auto-ported test: TestCase::test_python."""


import os
import platform
import sys
from os import path


if not hasattr(os, "startfile"):
    print("TestCase::test_python: skipped os.startfile unavailable")
elif hasattr(platform, "win32_is_iot") and platform.win32_is_iot():
    print("TestCase::test_python: skipped Windows IoT Core or nanoserver")
else:
    cwd, name = path.split(sys.executable)
    os.startfile(name, arguments="-V", cwd=cwd)
    os.startfile(name, arguments="-V", cwd=cwd, show_cmd=0)
    print("TestCase::test_python: ok")
