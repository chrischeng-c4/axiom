# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_basic_tests__test_order_ucaccddd"
# subject = "cpython.test_queues.QueueBasicTests.test_order"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueueBasicTests.test_order", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueueBasicTests.test_order did not pass"
print("QueueBasicTests::test_order: ok")
