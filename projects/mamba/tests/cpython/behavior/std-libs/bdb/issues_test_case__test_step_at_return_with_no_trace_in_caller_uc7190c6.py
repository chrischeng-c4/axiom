# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "issues_test_case__test_step_at_return_with_no_trace_in_caller_uc7190c6"
# subject = "cpython.test_bdb.IssuesTestCase.test_step_at_return_with_no_trace_in_caller"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("IssuesTestCase.test_step_at_return_with_no_trace_in_caller", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IssuesTestCase.test_step_at_return_with_no_trace_in_caller did not pass"
print("IssuesTestCase::test_step_at_return_with_no_trace_in_caller: ok")
