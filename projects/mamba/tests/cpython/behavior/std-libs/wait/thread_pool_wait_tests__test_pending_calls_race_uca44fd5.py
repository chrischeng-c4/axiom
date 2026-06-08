# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wait"
# dimension = "behavior"
# case = "thread_pool_wait_tests__test_pending_calls_race_uca44fd5"
# subject = "cpython.test_wait.ThreadPoolWaitTests.test_pending_calls_race"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_wait.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_wait
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolWaitTests.test_pending_calls_race", test_wait)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolWaitTests.test_pending_calls_race did not pass"
print("ThreadPoolWaitTests::test_pending_calls_race: ok")
