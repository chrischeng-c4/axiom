# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread_pool"
# dimension = "behavior"
# case = "thread_pool_executor_test__test_idle_thread_reuse_uc7ed032"
# subject = "cpython.test_thread_pool.ThreadPoolExecutorTest.test_idle_thread_reuse"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_thread_pool.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_thread_pool
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolExecutorTest.test_idle_thread_reuse", test_thread_pool)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolExecutorTest.test_idle_thread_reuse did not pass"
print("ThreadPoolExecutorTest::test_idle_thread_reuse: ok")
