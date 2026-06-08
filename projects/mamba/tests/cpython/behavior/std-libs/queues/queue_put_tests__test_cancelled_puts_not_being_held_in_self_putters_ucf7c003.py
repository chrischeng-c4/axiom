# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_put_tests__test_cancelled_puts_not_being_held_in_self_putters_ucf7c003"
# subject = "cpython.test_queues.QueuePutTests.test_cancelled_puts_not_being_held_in_self_putters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueuePutTests.test_cancelled_puts_not_being_held_in_self_putters", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueuePutTests.test_cancelled_puts_not_being_held_in_self_putters did not pass"
print("QueuePutTests::test_cancelled_puts_not_being_held_in_self_putters: ok")
