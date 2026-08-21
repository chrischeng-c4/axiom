# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "regrtest"
# dimension = "behavior"
# case = "args_test_case__test_python_command"
# subject = "cpython.test_regrtest.ArgsTestCase.test_python_command"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_regrtest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_regrtest
_suite = unittest.defaultTestLoader.loadTestsFromName("ArgsTestCase.test_python_command", test_regrtest)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ArgsTestCase.test_python_command did not pass"
print("ArgsTestCase::test_python_command: ok")
