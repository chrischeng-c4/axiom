# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "events"
# dimension = "behavior"
# case = "handle_tests__test_coroutine_like_object_debug_formatting"
# subject = "cpython.test_events.HandleTests.test_coroutine_like_object_debug_formatting"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_events
_suite = unittest.defaultTestLoader.loadTestsFromName("HandleTests.test_coroutine_like_object_debug_formatting", test_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HandleTests.test_coroutine_like_object_debug_formatting did not pass"
print("HandleTests::test_coroutine_like_object_debug_formatting: ok")
