# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_interpreter_attrs__test_main_isolated_ucb3553e"
# subject = "cpython.test_interpreters.TestInterpreterAttrs.test_main_isolated"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_interpreters
_suite = unittest.defaultTestLoader.loadTestsFromName("TestInterpreterAttrs.test_main_isolated", test_interpreters)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestInterpreterAttrs.test_main_isolated did not pass"
print("TestInterpreterAttrs::test_main_isolated: ok")
