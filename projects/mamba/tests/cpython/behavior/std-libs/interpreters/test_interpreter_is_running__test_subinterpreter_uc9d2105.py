# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_interpreter_is_running__test_subinterpreter_uc9d2105"
# subject = "cpython.test_interpreters.TestInterpreterIsRunning.test_subinterpreter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_interpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterpreterIsRunning.test_subinterpreter", test_interpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterpreterIsRunning.test_subinterpreter did not pass"
print("TestInterpreterIsRunning::test_subinterpreter: ok")
