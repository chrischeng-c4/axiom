# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "win32_process_test_case__test_invalid_args"
# subject = "cpython.test_subprocess.Win32ProcessTestCase.test_invalid_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("Win32ProcessTestCase.test_invalid_args", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Win32ProcessTestCase.test_invalid_args did not pass"
print("Win32ProcessTestCase::test_invalid_args: ok")
