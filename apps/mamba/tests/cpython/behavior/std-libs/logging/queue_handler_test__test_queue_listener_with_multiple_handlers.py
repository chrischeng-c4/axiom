# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "behavior"
# case = "queue_handler_test__test_queue_listener_with_multiple_handlers"
# subject = "cpython.test_logging.QueueHandlerTest.test_queue_listener_with_multiple_handlers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_logging.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_logging
_suite = unittest.defaultTestLoader.loadTestsFromName("QueueHandlerTest.test_queue_listener_with_multiple_handlers", test_logging)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython QueueHandlerTest.test_queue_listener_with_multiple_handlers did not pass"
print("QueueHandlerTest::test_queue_listener_with_multiple_handlers: ok")
