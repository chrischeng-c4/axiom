# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "thread"
# dimension = "behavior"
# case = "thread_running_tests__test_nt_and_posix_stack_size_uc88434c"
# subject = "cpython.test_thread.ThreadRunningTests.test_nt_and_posix_stack_size"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_thread.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_thread
_suite = unittest.defaultTestLoader.loadTestsFromName("ThreadRunningTests.test_nt_and_posix_stack_size", test_thread)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ThreadRunningTests.test_nt_and_posix_stack_size did not pass"
print("ThreadRunningTests::test_nt_and_posix_stack_size: ok")
