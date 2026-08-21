# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tasks"
# dimension = "behavior"
# case = "coroutine_gather_tests__test_cancellation_broadcast"
# subject = "cpython.test_tasks.CoroutineGatherTests.test_cancellation_broadcast"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_tasks
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineGatherTests.test_cancellation_broadcast", test_tasks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineGatherTests.test_cancellation_broadcast did not pass"
print("CoroutineGatherTests::test_cancellation_broadcast: ok")
