# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "selector_events"
# dimension = "behavior"
# case = "base_selector_event_loop_tests__test_read_from_self_exception"
# subject = "cpython.test_selector_events.BaseSelectorEventLoopTests.test_read_from_self_exception"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_selector_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_selector_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseSelectorEventLoopTests.test_read_from_self_exception", test_selector_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseSelectorEventLoopTests.test_read_from_self_exception did not pass"
print("BaseSelectorEventLoopTests::test_read_from_self_exception: ok")
