# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "base_event_loop_with_selector_tests__test_getnameinfo"
# subject = "cpython.test_base_events.BaseEventLoopWithSelectorTests.test_getnameinfo"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseEventLoopWithSelectorTests.test_getnameinfo", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseEventLoopWithSelectorTests.test_getnameinfo did not pass"
print("BaseEventLoopWithSelectorTests::test_getnameinfo: ok")
