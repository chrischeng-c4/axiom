# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "watchers"
# dimension = "behavior"
# case = "test_dict_watchers__test_no_event_if_not_watched"
# subject = "cpython.test_watchers.TestDictWatchers.test_no_event_if_not_watched"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_watchers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_watchers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDictWatchers.test_no_event_if_not_watched", test_watchers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDictWatchers.test_no_event_if_not_watched did not pass"
print("TestDictWatchers::test_no_event_if_not_watched: ok")
