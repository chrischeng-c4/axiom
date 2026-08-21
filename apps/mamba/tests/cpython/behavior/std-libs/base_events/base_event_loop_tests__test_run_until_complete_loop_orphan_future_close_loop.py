# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "base_event_loop_tests__test_run_until_complete_loop_orphan_future_close_loop"
# subject = "cpython.test_base_events.BaseEventLoopTests.test_run_until_complete_loop_orphan_future_close_loop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("BaseEventLoopTests.test_run_until_complete_loop_orphan_future_close_loop", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BaseEventLoopTests.test_run_until_complete_loop_orphan_future_close_loop did not pass"
print("BaseEventLoopTests::test_run_until_complete_loop_orphan_future_close_loop: ok")
