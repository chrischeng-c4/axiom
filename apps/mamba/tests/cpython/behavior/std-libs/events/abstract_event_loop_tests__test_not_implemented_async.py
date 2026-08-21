# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "events"
# dimension = "behavior"
# case = "abstract_event_loop_tests__test_not_implemented_async"
# subject = "cpython.test_events.AbstractEventLoopTests.test_not_implemented_async"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_events
_suite = unittest.defaultTestLoader.loadTestsFromName("AbstractEventLoopTests.test_not_implemented_async", test_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AbstractEventLoopTests.test_not_implemented_async did not pass"
print("AbstractEventLoopTests::test_not_implemented_async: ok")
