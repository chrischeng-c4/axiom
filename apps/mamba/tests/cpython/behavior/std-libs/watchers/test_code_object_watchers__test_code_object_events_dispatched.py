# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "watchers"
# dimension = "behavior"
# case = "test_code_object_watchers__test_code_object_events_dispatched"
# subject = "cpython.test_watchers.TestCodeObjectWatchers.test_code_object_events_dispatched"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_watchers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_watchers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestCodeObjectWatchers.test_code_object_events_dispatched", test_watchers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestCodeObjectWatchers.test_code_object_events_dispatched did not pass"
print("TestCodeObjectWatchers::test_code_object_events_dispatched: ok")
