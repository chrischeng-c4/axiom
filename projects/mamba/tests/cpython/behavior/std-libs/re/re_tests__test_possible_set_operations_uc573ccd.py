# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_possible_set_operations_uc573ccd"
# subject = "cpython.test_re.ReTests.test_possible_set_operations"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_re
_suite = unittest.defaultTestLoader.loadTestsFromName("ReTests.test_possible_set_operations", test_re)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ReTests.test_possible_set_operations did not pass"
print("ReTests::test_possible_set_operations: ok")
