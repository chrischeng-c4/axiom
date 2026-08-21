# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queues"
# dimension = "behavior"
# case = "queue_get_tests__test_cancelled_getters_not_being_held_in_self_getters_uc9fec82"
# subject = "cpython.test_queues.QueueGetTests.test_cancelled_getters_not_being_held_in_self_getters"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_queues.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_queues
_suite = unittest.defaultTestLoader.loadTestsFromName("QueueGetTests.test_cancelled_getters_not_being_held_in_self_getters", test_queues)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueueGetTests.test_cancelled_getters_not_being_held_in_self_getters did not pass"
print("QueueGetTests::test_cancelled_getters_not_being_held_in_self_getters: ok")
