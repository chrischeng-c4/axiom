# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cppext"
# dimension = "behavior"
# case = "test_cpp_ext__test_build"
# subject = "cpython.test_cppext.TestCPPExt.test_build"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cppext/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""CPython C++ extension smoke build imports cleanly without C API warnings."""

import io
import os
import tempfile
import unittest
from test import test_cppext


original_cwd = os.getcwd()
stream = io.StringIO()

with tempfile.TemporaryDirectory(prefix="mamba-cpython-cppext-") as tmpdir:
    os.chdir(tmpdir)
    try:
        suite = unittest.TestSuite([test_cppext.TestCPPExt("test_build")])
        result = unittest.TextTestRunner(stream=stream, verbosity=0).run(suite)
    finally:
        os.chdir(original_cwd)

assert result.testsRun == 1, result.testsRun
assert not result.failures, stream.getvalue()
assert not result.errors, stream.getvalue()

if result.skipped:
    print("test_cpp_ext__test_build skipped:", result.skipped[0][1])
else:
    print("test_cpp_ext__test_build OK")
