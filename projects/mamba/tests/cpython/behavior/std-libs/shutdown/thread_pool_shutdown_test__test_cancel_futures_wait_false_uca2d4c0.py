# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutdown"
# dimension = "behavior"
# case = "thread_pool_shutdown_test__test_cancel_futures_wait_false_uca2d4c0"
# subject = "cpython.test_shutdown.ThreadPoolShutdownTest.test_cancel_futures_wait_false"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_shutdown.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_shutdown
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolShutdownTest.test_cancel_futures_wait_false", test_shutdown)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolShutdownTest.test_cancel_futures_wait_false did not pass"
print("ThreadPoolShutdownTest::test_cancel_futures_wait_false: ok")
