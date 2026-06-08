# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "staggered"
# dimension = "behavior"
# case = "staggered_tests__test_empty_uc89a881"
# subject = "cpython.test_staggered.StaggeredTests.test_empty"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_staggered.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_staggered
_suite = unittest.defaultTestLoader.loadTestsFromName("StaggeredTests.test_empty", test_staggered)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython StaggeredTests.test_empty did not pass"
print("StaggeredTests::test_empty: ok")
