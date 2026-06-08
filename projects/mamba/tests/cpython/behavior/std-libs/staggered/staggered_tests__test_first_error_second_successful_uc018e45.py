# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "staggered"
# dimension = "behavior"
# case = "staggered_tests__test_first_error_second_successful_uc018e45"
# subject = "cpython.test_staggered.StaggeredTests.test_first_error_second_successful"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_staggered.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_staggered
_suite = unittest.defaultTestLoader.loadTestsFromName("StaggeredTests.test_first_error_second_successful", test_staggered)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StaggeredTests.test_first_error_second_successful did not pass"
print("StaggeredTests::test_first_error_second_successful: ok")
