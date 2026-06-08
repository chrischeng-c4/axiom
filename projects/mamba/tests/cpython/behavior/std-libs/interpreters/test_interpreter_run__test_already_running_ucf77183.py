# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_interpreter_run__test_already_running_ucf77183"
# subject = "cpython.test_interpreters.TestInterpreterRun.test_already_running"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_interpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterpreterRun.test_already_running", test_interpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterpreterRun.test_already_running did not pass"
print("TestInterpreterRun::test_already_running: ok")
