# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tracemalloc"
# dimension = "behavior"
# case = "test_tracemalloc_enabled__test_no_incomplete_frames_uc4edf03"
# subject = "cpython.test_tracemalloc.TestTracemallocEnabled.test_no_incomplete_frames"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_tracemalloc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_tracemalloc
_suite = unittest.defaultTestLoader.loadTestsFromName("TestTracemallocEnabled.test_no_incomplete_frames", test_tracemalloc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestTracemallocEnabled.test_no_incomplete_frames did not pass"
print("TestTracemallocEnabled::test_no_incomplete_frames: ok")
