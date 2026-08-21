# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tasks"
# dimension = "behavior"
# case = "coroutine_gather_tests__test_duplicate_coroutines"
# subject = "cpython.test_tasks.CoroutineGatherTests.test_duplicate_coroutines"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_tasks
_suite = unittest.defaultTestLoader.loadTestsFromName("CoroutineGatherTests.test_duplicate_coroutines", test_tasks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CoroutineGatherTests.test_duplicate_coroutines did not pass"
print("CoroutineGatherTests::test_duplicate_coroutines: ok")
