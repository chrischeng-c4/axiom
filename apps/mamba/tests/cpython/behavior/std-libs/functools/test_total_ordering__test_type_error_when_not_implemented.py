# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "test_total_ordering__test_type_error_when_not_implemented"
# subject = "cpython.test_functools.TestTotalOrdering.test_type_error_when_not_implemented"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_functools
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTotalOrdering.test_type_error_when_not_implemented", test_functools)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTotalOrdering.test_type_error_when_not_implemented did not pass"
print("TestTotalOrdering::test_type_error_when_not_implemented: ok")
