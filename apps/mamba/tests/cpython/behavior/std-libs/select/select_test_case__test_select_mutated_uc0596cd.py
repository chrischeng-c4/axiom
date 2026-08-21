# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "behavior"
# case = "select_test_case__test_select_mutated_uc0596cd"
# subject = "cpython.test_select.SelectTestCase.test_select_mutated"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_select.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_select
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectTestCase.test_select_mutated", test_select)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectTestCase.test_select_mutated did not pass"
print("SelectTestCase::test_select_mutated: ok")
