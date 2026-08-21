# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_interpreter_close__test_still_running_uc996fb6"
# subject = "cpython.test_interpreters.TestInterpreterClose.test_still_running"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_interpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterpreterClose.test_still_running", test_interpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterpreterClose.test_still_running did not pass"
print("TestInterpreterClose::test_still_running: ok")
