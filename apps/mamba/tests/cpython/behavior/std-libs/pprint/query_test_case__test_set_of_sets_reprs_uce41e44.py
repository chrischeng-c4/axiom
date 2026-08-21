# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "query_test_case__test_set_of_sets_reprs_uce41e44"
# subject = "cpython.test_pprint.QueryTestCase.test_set_of_sets_reprs"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pprint
_suite = unittest.defaultTestLoader.loadTestsFromName("QueryTestCase.test_set_of_sets_reprs", test_pprint)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueryTestCase.test_set_of_sets_reprs did not pass"
print("QueryTestCase::test_set_of_sets_reprs: ok")
