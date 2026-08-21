# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "test_branch_and_jump_events__test_loop"
# subject = "cpython.test_monitoring.TestBranchAndJumpEvents.test_loop"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_monitoring
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBranchAndJumpEvents.test_loop", test_monitoring)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBranchAndJumpEvents.test_loop did not pass"
print("TestBranchAndJumpEvents::test_loop: ok")
