# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "watchers"
# dimension = "behavior"
# case = "test_func_watchers__test_dealloc_watcher_raises_error"
# subject = "cpython.test_watchers.TestFuncWatchers.test_dealloc_watcher_raises_error"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_watchers.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_watchers
_suite = unittest.defaultTestLoader.loadTestsFromName("TestFuncWatchers.test_dealloc_watcher_raises_error", test_watchers)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestFuncWatchers.test_dealloc_watcher_raises_error did not pass"
print("TestFuncWatchers::test_dealloc_watcher_raises_error: ok")
