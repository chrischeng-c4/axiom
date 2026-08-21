# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_get_tests__test_get_with_waiting_putters_uc454e5b"
# subject = "cpython.test_queues.QueueGetTests.test_get_with_waiting_putters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueueGetTests.test_get_with_waiting_putters", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueueGetTests.test_get_with_waiting_putters did not pass"
print("QueueGetTests::test_get_with_waiting_putters: ok")
