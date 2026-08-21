# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread_pool"
# dimension = "behavior"
# case = "thread_pool_executor_test__test_default_workers_ucd6a9c4"
# subject = "cpython.test_thread_pool.ThreadPoolExecutorTest.test_default_workers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_thread_pool.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_thread_pool
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolExecutorTest.test_default_workers", test_thread_pool)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolExecutorTest.test_default_workers did not pass"
print("ThreadPoolExecutorTest::test_default_workers: ok")
