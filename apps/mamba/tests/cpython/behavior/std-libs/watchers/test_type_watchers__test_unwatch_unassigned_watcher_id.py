# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "watchers"
# dimension = "behavior"
# case = "test_type_watchers__test_unwatch_unassigned_watcher_id"
# subject = "cpython.test_watchers.TestTypeWatchers.test_unwatch_unassigned_watcher_id"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_watchers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_watchers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTypeWatchers.test_unwatch_unassigned_watcher_id", test_watchers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTypeWatchers.test_unwatch_unassigned_watcher_id did not pass"
print("TestTypeWatchers::test_unwatch_unassigned_watcher_id: ok")
