# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "context_manager_test_case__test_recursive"
# subject = "cpython.test_contextlib.ContextManagerTestCase.test_recursive"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib
_suite = unittest.defaultTestLoader.loadTestsFromName("ContextManagerTestCase.test_recursive", test_contextlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ContextManagerTestCase.test_recursive did not pass"
print("ContextManagerTestCase::test_recursive: ok")
