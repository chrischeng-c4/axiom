# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tasks"
# dimension = "behavior"
# case = "run_coroutine_threadsafe_tests__test_run_coroutine_threadsafe_task_cancelled"
# subject = "cpython.test_tasks.RunCoroutineThreadsafeTests.test_run_coroutine_threadsafe_task_cancelled"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_tasks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_tasks
_suite = unittest.defaultTestLoader.loadTestsFromName("RunCoroutineThreadsafeTests.test_run_coroutine_threadsafe_task_cancelled", test_tasks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython RunCoroutineThreadsafeTests.test_run_coroutine_threadsafe_task_cancelled did not pass"
print("RunCoroutineThreadsafeTests::test_run_coroutine_threadsafe_task_cancelled: ok")
