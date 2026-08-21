# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread_pool"
# dimension = "behavior"
# case = "thread_pool_executor_test__test_process_fork_from_a_threadpool_uc10c579"
# subject = "cpython.test_thread_pool.ThreadPoolExecutorTest.test_process_fork_from_a_threadpool"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_thread_pool.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_thread_pool
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolExecutorTest.test_process_fork_from_a_threadpool", test_thread_pool)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolExecutorTest.test_process_fork_from_a_threadpool did not pass"
print("ThreadPoolExecutorTest::test_process_fork_from_a_threadpool: ok")
