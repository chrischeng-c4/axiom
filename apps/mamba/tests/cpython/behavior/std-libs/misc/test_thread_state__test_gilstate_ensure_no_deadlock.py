# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test_thread_state__test_gilstate_ensure_no_deadlock"
# subject = "cpython.test_misc.TestThreadState.test_gilstate_ensure_no_deadlock"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestThreadState.test_gilstate_ensure_no_deadlock", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestThreadState.test_gilstate_ensure_no_deadlock did not pass"
print("TestThreadState::test_gilstate_ensure_no_deadlock: ok")
