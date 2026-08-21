# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutdown"
# dimension = "behavior"
# case = "thread_pool_shutdown_test__test_context_manager_shutdown_uce5273b"
# subject = "cpython.test_shutdown.ThreadPoolShutdownTest.test_context_manager_shutdown"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_concurrent_futures/test_shutdown.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_concurrent_futures import test_shutdown
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadPoolShutdownTest.test_context_manager_shutdown", test_shutdown)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadPoolShutdownTest.test_context_manager_shutdown did not pass"
print("ThreadPoolShutdownTest::test_context_manager_shutdown: ok")
