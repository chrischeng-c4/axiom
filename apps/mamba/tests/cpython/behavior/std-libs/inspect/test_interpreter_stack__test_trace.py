# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_interpreter_stack__test_trace"
# subject = "cpython.test_inspect.TestInterpreterStack.test_trace"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterpreterStack.test_trace", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterpreterStack.test_trace did not pass"
print("TestInterpreterStack::test_trace: ok")
