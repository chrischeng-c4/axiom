# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "thread_running_tests__test_starting_threads_uc43f59c"
# subject = "cpython.test_thread.ThreadRunningTests.test_starting_threads"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_thread
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadRunningTests.test_starting_threads", test_thread)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadRunningTests.test_starting_threads did not pass"
print("ThreadRunningTests::test_starting_threads: ok")
