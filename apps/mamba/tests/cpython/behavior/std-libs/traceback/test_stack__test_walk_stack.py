# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "behavior"
# case = "test_stack__test_walk_stack"
# subject = "cpython.test_traceback.TestStack.test_walk_stack"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_traceback.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_traceback
_suite = unittest.defaultTestLoader.loadTestsFromName("TestStack.test_walk_stack", test_traceback)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestStack.test_walk_stack did not pass"
print("TestStack::test_walk_stack: ok")
