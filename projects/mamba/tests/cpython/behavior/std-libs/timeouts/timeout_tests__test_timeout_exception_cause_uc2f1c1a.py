# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "timeouts"
# dimension = "behavior"
# case = "timeout_tests__test_timeout_exception_cause_uc2f1c1a"
# subject = "cpython.test_timeouts.TimeoutTests.test_timeout_exception_cause"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_timeouts.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_timeouts
_suite = unittest.defaultTestLoader.loadTestsFromName("TimeoutTests.test_timeout_exception_cause", test_timeouts)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TimeoutTests.test_timeout_exception_cause did not pass"
print("TimeoutTests::test_timeout_exception_cause: ok")
