# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "events"
# dimension = "behavior"
# case = "kqueue_event_loop_tests__test_read_pty_output"
# subject = "cpython.test_events.KqueueEventLoopTests.test_read_pty_output"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_events
_suite = unittest.defaultTestLoader.loadTestsFromName("KqueueEventLoopTests.test_read_pty_output", test_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython KqueueEventLoopTests.test_read_pty_output did not pass"
print("KqueueEventLoopTests::test_read_pty_output: ok")
