# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_func_test_case__test_run_with_pathlike_path"
# subject = "cpython.test_subprocess.RunFuncTestCase.test_run_with_pathlike_path"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_subprocess
_suite = unittest.defaultTestLoader.loadTestsFromName("RunFuncTestCase.test_run_with_pathlike_path", test_subprocess)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunFuncTestCase.test_run_with_pathlike_path did not pass"
print("RunFuncTestCase::test_run_with_pathlike_path: ok")
