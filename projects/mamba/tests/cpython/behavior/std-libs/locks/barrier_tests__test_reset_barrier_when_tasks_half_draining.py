# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "barrier_tests__test_reset_barrier_when_tasks_half_draining"
# subject = "cpython.test_locks.BarrierTests.test_reset_barrier_when_tasks_half_draining"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_locks
_suite = unittest.defaultTestLoader.loadTestsFromName("BarrierTests.test_reset_barrier_when_tasks_half_draining", test_locks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BarrierTests.test_reset_barrier_when_tasks_half_draining did not pass"
print("BarrierTests::test_reset_barrier_when_tasks_half_draining: ok")
