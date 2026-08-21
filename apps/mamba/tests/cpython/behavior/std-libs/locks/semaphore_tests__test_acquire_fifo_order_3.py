# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "semaphore_tests__test_acquire_fifo_order_3"
# subject = "cpython.test_locks.SemaphoreTests.test_acquire_fifo_order_3"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_locks
_suite = unittest.defaultTestLoader.loadTestsFromName("SemaphoreTests.test_acquire_fifo_order_3", test_locks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SemaphoreTests.test_acquire_fifo_order_3 did not pass"
print("SemaphoreTests::test_acquire_fifo_order_3: ok")
