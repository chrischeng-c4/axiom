# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "startfile"
# dimension = "behavior"
# case = "test_case__test_nonexisting"
# subject = "cpython.test_startfile.TestCase.test_nonexisting"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_startfile.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_startfile.py::TestCase::test_nonexisting
"""Auto-ported test: TestCase::test_nonexisting."""


import os


if not hasattr(os, "startfile"):
    print("TestCase::test_nonexisting: skipped os.startfile unavailable")
else:
    try:
        os.startfile("nonexisting.vbs")
    except OSError:
        print("TestCase::test_nonexisting: ok")
    else:
        raise AssertionError("expected OSError for nonexisting.vbs")
