# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_get_tests__test_why_are_getters_waiting_uc162d08"
# subject = "cpython.test_queues.QueueGetTests.test_why_are_getters_waiting"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueueGetTests.test_why_are_getters_waiting", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueueGetTests.test_why_are_getters_waiting did not pass"
print("QueueGetTests::test_why_are_getters_waiting: ok")
