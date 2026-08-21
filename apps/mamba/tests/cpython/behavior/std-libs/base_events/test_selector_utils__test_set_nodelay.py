# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base_events"
# dimension = "behavior"
# case = "test_selector_utils__test_set_nodelay"
# subject = "cpython.test_base_events.TestSelectorUtils.test_set_nodelay"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_asyncio/test_base_events.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_asyncio import test_base_events
_suite = unittest.defaultTestLoader.loadTestsFromName("TestSelectorUtils.test_set_nodelay", test_base_events)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestSelectorUtils.test_set_nodelay did not pass"
print("TestSelectorUtils::test_set_nodelay: ok")
