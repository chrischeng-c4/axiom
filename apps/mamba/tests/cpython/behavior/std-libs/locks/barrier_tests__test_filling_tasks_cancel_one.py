# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locks"
# dimension = "behavior"
# case = "barrier_tests__test_filling_tasks_cancel_one"
# subject = "cpython.test_locks.BarrierTests.test_filling_tasks_cancel_one"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_locks.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_locks
_suite = unittest.defaultTestLoader.loadTestsFromName("BarrierTests.test_filling_tasks_cancel_one", test_locks)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BarrierTests.test_filling_tasks_cancel_one did not pass"
print("BarrierTests::test_filling_tasks_cancel_one: ok")
