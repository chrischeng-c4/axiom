# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_basics_test__test_nested_exception_group_tracebacks_uc65aab5"
# subject = "cpython.test_exception_group.NestedExceptionGroupBasicsTest.test_nested_exception_group_tracebacks"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("NestedExceptionGroupBasicsTest.test_nested_exception_group_tracebacks", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NestedExceptionGroupBasicsTest.test_nested_exception_group_tracebacks did not pass"
print("NestedExceptionGroupBasicsTest::test_nested_exception_group_tracebacks: ok")
