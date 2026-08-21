# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_put_tests__test_put_cancel_drop_uc7f60fb"
# subject = "cpython.test_queues.QueuePutTests.test_put_cancel_drop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueuePutTests.test_put_cancel_drop", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueuePutTests.test_put_cancel_drop did not pass"
print("QueuePutTests::test_put_cancel_drop: ok")
