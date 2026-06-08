# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "behavior"
# case = "test_context_decorator__test_decorator_with_exception"
# subject = "cpython.test_contextlib.TestContextDecorator.test_decorator_with_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_contextlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_contextlib
_suite = unittest.defaultTestLoader.loadTestsFromName("TestContextDecorator.test_decorator_with_exception", test_contextlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestContextDecorator.test_decorator_with_exception did not pass"
print("TestContextDecorator::test_decorator_with_exception: ok")
