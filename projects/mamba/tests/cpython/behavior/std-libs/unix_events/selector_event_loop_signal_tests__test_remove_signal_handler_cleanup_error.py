# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "selector_event_loop_signal_tests__test_remove_signal_handler_cleanup_error"
# subject = "cpython.test_unix_events.SelectorEventLoopSignalTests.test_remove_signal_handler_cleanup_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("SelectorEventLoopSignalTests.test_remove_signal_handler_cleanup_error", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SelectorEventLoopSignalTests.test_remove_signal_handler_cleanup_error did not pass"
print("SelectorEventLoopSignalTests::test_remove_signal_handler_cleanup_error: ok")
