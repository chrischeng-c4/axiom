# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_put_tests__test_get_cancel_drop_one_pending_reader_uc8a6ed7"
# subject = "cpython.test_queues.QueuePutTests.test_get_cancel_drop_one_pending_reader"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueuePutTests.test_get_cancel_drop_one_pending_reader", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueuePutTests.test_get_cancel_drop_one_pending_reader did not pass"
print("QueuePutTests::test_get_cancel_drop_one_pending_reader: ok")
