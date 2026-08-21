# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unix_events"
# dimension = "behavior"
# case = "abstract_child_watcher_tests__test_warns_on_subclassing"
# subject = "cpython.test_unix_events.AbstractChildWatcherTests.test_warns_on_subclassing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_unix_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_unix_events
_suite = unittest.defaultTestLoader.loadTestsFromName("AbstractChildWatcherTests.test_warns_on_subclassing", test_unix_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython AbstractChildWatcherTests.test_warns_on_subclassing did not pass"
print("AbstractChildWatcherTests::test_warns_on_subclassing: ok")
