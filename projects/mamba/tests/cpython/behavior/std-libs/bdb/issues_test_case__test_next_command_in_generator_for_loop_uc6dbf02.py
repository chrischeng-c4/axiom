# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "issues_test_case__test_next_command_in_generator_for_loop_uc6dbf02"
# subject = "cpython.test_bdb.IssuesTestCase.test_next_command_in_generator_for_loop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_bdb.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_bdb
_suite = unittest.defaultTestLoader.loadTestsFromName("IssuesTestCase.test_next_command_in_generator_for_loop", test_bdb)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IssuesTestCase.test_next_command_in_generator_for_loop did not pass"
print("IssuesTestCase::test_next_command_in_generator_for_loop: ok")
