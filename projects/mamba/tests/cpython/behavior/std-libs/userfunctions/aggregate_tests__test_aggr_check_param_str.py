# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "userfunctions"
# dimension = "behavior"
# case = "aggregate_tests__test_aggr_check_param_str"
# subject = "cpython.test_userfunctions.AggregateTests.test_aggr_check_param_str"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_userfunctions.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_userfunctions
_suite = unittest.defaultTestLoader.loadTestsFromName("AggregateTests.test_aggr_check_param_str", test_userfunctions)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AggregateTests.test_aggr_check_param_str did not pass"
print("AggregateTests::test_aggr_check_param_str: ok")
