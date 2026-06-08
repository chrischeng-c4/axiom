# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "test_fork_in_thread__test_forkinthread_uc021150"
# subject = "cpython.test_thread.TestForkInThread.test_forkinthread"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_thread
_suite = unittest.defaultTestLoader.loadTestsFromName("TestForkInThread.test_forkinthread", test_thread)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestForkInThread.test_forkinthread did not pass"
print("TestForkInThread::test_forkinthread: ok")
