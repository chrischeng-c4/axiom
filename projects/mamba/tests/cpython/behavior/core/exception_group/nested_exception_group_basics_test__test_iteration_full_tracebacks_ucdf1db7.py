# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "exception_group"
# dimension = "behavior"
# case = "nested_exception_group_basics_test__test_iteration_full_tracebacks_ucdf1db7"
# subject = "cpython.test_exception_group.NestedExceptionGroupBasicsTest.test_iteration_full_tracebacks"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_exception_group.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_exception_group
_suite = unittest.defaultTestLoader.loadTestsFromName("NestedExceptionGroupBasicsTest.test_iteration_full_tracebacks", test_exception_group)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython NestedExceptionGroupBasicsTest.test_iteration_full_tracebacks did not pass"
print("NestedExceptionGroupBasicsTest::test_iteration_full_tracebacks: ok")
